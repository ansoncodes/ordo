//! On-device AI: a small quantized language model, run with candle on the CPU, that
//! rewrites a messy sentence ("reply to sir tommorow arnd 3 in the evening, high prio")
//! into Ordo quick-add syntax ("reply to sir tomorrow 3pm !p2"). The deterministic parser
//! then does the date arithmetic, so the model only has to understand language, which is
//! what small models are good at.
//!
//! Nothing leaves the machine at inference time. The only network use is the one-time
//! model download that the user starts from Settings.

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use crate::{llama, qwen2};
use chrono::NaiveDate;
use serde::Serialize;
use tokenizers::Tokenizer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Arch {
    Qwen2,
    Llama,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub size_bytes: u64,
    pub arch: Arch,
    /// How many few-shot examples this model gets; tiny models get confused by too many.
    pub examples: usize,
    pub gguf_file: &'static str,
    #[serde(skip)]
    pub gguf_url: &'static str,
    #[serde(skip)]
    pub tokenizer_url: &'static str,
}

/// Models the app knows how to download and run. Any GGUF model with one of the
/// supported architectures (including a purpose-trained one) can be added here.
pub static MODELS: [ModelSpec; 3] = [
    ModelSpec {
        id: "qwen2.5-0.5b",
        label: "Accurate · Qwen 2.5 0.5B",
        description: "Most accurate on unusual phrasing. About 800 MB of RAM while loaded.",
        size_bytes: 675_710_816,
        arch: Arch::Qwen2,
        examples: 14,
        gguf_file: "qwen2.5-0.5b-instruct-q8_0.gguf",
        gguf_url: "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q8_0.gguf",
        tokenizer_url: "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct/resolve/main/tokenizer.json",
    },
    ModelSpec {
        id: "qwen2.5-0.5b-q4",
        label: "Light · Qwen 2.5 0.5B (4-bit)",
        description: "Same model, compressed. Nearly the same accuracy; about 550 MB of RAM while loaded.",
        size_bytes: 428_730_208,
        arch: Arch::Qwen2,
        examples: 14,
        gguf_file: "qwen2.5-0.5b-instruct-q4_0.gguf",
        gguf_url: "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_0.gguf",
        tokenizer_url: "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct/resolve/main/tokenizer.json",
    },
    ModelSpec {
        id: "smollm2-135m",
        label: "Ultra light · SmolLM2 135M",
        description: "Runs on almost anything; about 200 MB of RAM. Fine for typos and everyday phrasing, unreliable on complex sentences.",
        size_bytes: 144_811_360,
        arch: Arch::Llama,
        examples: 10,
        gguf_file: "smollm2-135m-instruct-q8_0.gguf",
        gguf_url: "https://huggingface.co/bartowski/SmolLM2-135M-Instruct-GGUF/resolve/main/SmolLM2-135M-Instruct-Q8_0.gguf",
        tokenizer_url: "https://huggingface.co/HuggingFaceTB/SmolLM2-135M-Instruct/resolve/main/tokenizer.json",
    },
];

pub fn spec(id: &str) -> Option<&'static ModelSpec> {
    MODELS.iter().find(|m| m.id == id)
}

/// Shared between the CLI and the desktop app so a model is only downloaded once.
pub fn default_models_dir() -> PathBuf {
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(local).join("Ordo").join("models");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".ordo").join("models");
    }
    PathBuf::from("data").join("models")
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Status {
    Missing,
    Downloading { done: u64, total: u64 },
    Ready { loaded: bool },
    Error { message: String },
}

#[derive(Default)]
struct Progress {
    done: AtomicU64,
    total: AtomicU64,
    finished: AtomicBool,
    error: Mutex<Option<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Normalized {
    pub input: String,
    pub output: String,
    pub model: String,
    /// Time spent generating, excluding model load.
    pub elapsed_ms: u128,
    /// Time spent loading the model into memory (0 when it was already loaded).
    pub load_ms: u128,
}

pub struct Ai {
    dir: PathBuf,
    downloads: Mutex<HashMap<String, Arc<Progress>>>,
    engine: Mutex<Option<Engine>>,
    last_used: Mutex<Instant>,
}

impl Ai {
    pub fn new(dir: PathBuf) -> Self {
        Ai { dir, downloads: Mutex::new(HashMap::new()), engine: Mutex::new(None), last_used: Mutex::new(Instant::now()) }
    }

    /// Frees the model's memory after it has been idle for `idle`. Checks once a minute.
    pub fn spawn_idle_unloader(self: &Arc<Self>, idle: std::time::Duration) {
        let weak = Arc::downgrade(self);
        std::thread::Builder::new()
            .name("ordo-ai-idle".into())
            .spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
                let Some(ai) = weak.upgrade() else { break };
                let idle_for = ai.last_used.lock().map(|t| t.elapsed()).unwrap_or_default();
                if idle_for >= idle {
                    let mut engine = ai.engine.lock().unwrap_or_else(|e| e.into_inner());
                    if engine.is_some() {
                        *engine = None;
                    }
                }
            })
            .ok();
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn gguf_path(&self, spec: &ModelSpec) -> PathBuf {
        self.dir.join(spec.id).join(spec.gguf_file)
    }

    fn tokenizer_path(&self, spec: &ModelSpec) -> PathBuf {
        self.dir.join(spec.id).join("tokenizer.json")
    }

    /// A model is ready when the weights file has exactly the published size and the tokenizer exists.
    pub fn is_ready(&self, spec: &ModelSpec) -> bool {
        let weights_ok = fs::metadata(self.gguf_path(spec)).map(|m| m.len() == spec.size_bytes).unwrap_or(false);
        weights_ok && self.tokenizer_path(spec).exists()
    }

    pub fn is_loaded(&self, spec: &ModelSpec) -> bool {
        self.engine.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(|e| e.model_id == spec.id).unwrap_or(false)
    }

    pub fn status(&self, spec: &ModelSpec) -> Status {
        let downloads = self.downloads.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(p) = downloads.get(spec.id) {
            if !p.finished.load(Ordering::Relaxed) {
                return Status::Downloading { done: p.done.load(Ordering::Relaxed), total: p.total.load(Ordering::Relaxed).max(spec.size_bytes) };
            }
            if let Some(message) = p.error.lock().unwrap_or_else(|e| e.into_inner()).clone() {
                if !self.is_ready(spec) {
                    return Status::Error { message };
                }
            }
        }
        drop(downloads);
        if self.is_ready(spec) {
            Status::Ready { loaded: self.is_loaded(spec) }
        } else {
            Status::Missing
        }
    }

    /// Starts a background download; returns immediately. Safe to call repeatedly.
    pub fn start_download(self: &Arc<Self>, spec: &'static ModelSpec) -> Result<(), String> {
        if self.is_ready(spec) {
            return Ok(());
        }
        let progress = {
            let mut downloads = self.downloads.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(p) = downloads.get(spec.id) {
                if !p.finished.load(Ordering::Relaxed) {
                    return Ok(());
                }
            }
            let p = Arc::new(Progress::default());
            p.total.store(spec.size_bytes, Ordering::Relaxed);
            downloads.insert(spec.id.to_string(), p.clone());
            p
        };
        let dir = self.dir.join(spec.id);
        let gguf = self.gguf_path(spec);
        let tokenizer = self.tokenizer_path(spec);
        std::thread::Builder::new()
            .name(format!("ordo-download-{}", spec.id))
            .spawn(move || {
                let result = (|| -> Result<(), String> {
                    fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
                    fetch(spec.tokenizer_url, &tokenizer, None)?;
                    let part = gguf.with_extension("part");
                    fetch(spec.gguf_url, &part, Some(&progress))?;
                    let len = fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
                    if len != spec.size_bytes {
                        let _ = fs::remove_file(&part);
                        return Err(format!("download incomplete ({len} of {} bytes)", spec.size_bytes));
                    }
                    fs::rename(&part, &gguf).map_err(|e| format!("cannot finalise download: {e}"))?;
                    Ok(())
                })();
                if let Err(e) = result {
                    *progress.error.lock().unwrap_or_else(|e| e.into_inner()) = Some(e);
                }
                progress.finished.store(true, Ordering::Relaxed);
            })
            .map_err(|e| format!("cannot start download thread: {e}"))?;
        Ok(())
    }

    /// Deletes the model files (and unloads it if it is in memory).
    pub fn remove(&self, spec: &ModelSpec) -> Result<(), String> {
        if self.is_loaded(spec) {
            self.unload();
        }
        let dir = self.dir.join(spec.id);
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(|e| format!("cannot delete {}: {e}", dir.display()))?;
        }
        self.downloads.lock().unwrap_or_else(|e| e.into_inner()).remove(spec.id);
        Ok(())
    }

    /// Frees the memory used by the loaded model.
    pub fn unload(&self) {
        *self.engine.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }

    /// Rewrites `text` into quick-add syntax. Blocking: call from a blocking thread.
    pub fn normalize(&self, spec: &'static ModelSpec, text: &str, today: NaiveDate, projects: &[String]) -> Result<Normalized, String> {
        if !self.is_ready(spec) {
            return Err("The on-device model is not downloaded yet. Open Settings and choose Download.".into());
        }
        let mut guard = self.engine.lock().unwrap_or_else(|e| e.into_inner());
        let mut load_ms = 0;
        if guard.as_ref().map(|e| e.model_id != spec.id).unwrap_or(true) {
            let started = Instant::now();
            *guard = None;
            *guard = Some(Engine::load(spec, &self.gguf_path(spec), &self.tokenizer_path(spec))?);
            load_ms = started.elapsed().as_millis();
        }
        let engine = guard.as_mut().expect("engine loaded");
        let style = prompt_style(spec);
        let (prefix, suffix) = build_prompt_parts_styled(text, today, projects, style, spec.examples);
        if std::env::var("ORDO_AI_NOCACHE").is_ok() {
            engine.prefix = None;
        }
        let started = Instant::now();
        let raw = engine.generate(&prefix, &suffix, 64)?;
        let output = drop_unknown_projects(&clean_output(&raw, text), projects);
        if let Ok(mut t) = self.last_used.lock() {
            *t = Instant::now();
        }
        Ok(Normalized { input: text.to_string(), output, model: spec.id.to_string(), elapsed_ms: started.elapsed().as_millis(), load_ms })
    }
}

fn fetch(url: &str, dest: &Path, progress: Option<&Progress>) -> Result<(), String> {
    let mut resp = ureq::get(url).call().map_err(|e| format!("download failed: {e}"))?;
    if let Some(p) = progress {
        if let Some(len) = resp.headers().get("content-length").and_then(|v| v.to_str().ok()).and_then(|s| s.parse::<u64>().ok()) {
            p.total.store(len, Ordering::Relaxed);
        }
    }
    let mut reader = resp.body_mut().with_config().limit(u64::MAX).reader();
    let mut file = fs::File::create(dest).map_err(|e| format!("cannot write {}: {e}", dest.display()))?;
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = reader.read(&mut buf).map_err(|e| format!("download interrupted: {e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| format!("cannot write {}: {e}", dest.display()))?;
        if let Some(p) = progress {
            p.done.fetch_add(n as u64, Ordering::Relaxed);
        }
    }
    file.flush().map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Inference engine
// ---------------------------------------------------------------------------

enum Weights {
    Qwen2(qwen2::ModelWeights),
    Llama(llama::ModelWeights),
}

impl Weights {
    fn kv_cache(&self) -> Vec<Option<(Tensor, Tensor)>> {
        match self {
            Weights::Qwen2(m) => m.kv_cache(),
            Weights::Llama(m) => m.kv_cache(),
        }
    }

    fn set_kv_cache(&mut self, cache: Vec<Option<(Tensor, Tensor)>>) {
        match self {
            Weights::Qwen2(m) => m.set_kv_cache(cache),
            Weights::Llama(m) => m.set_kv_cache(cache),
        }
    }
}

/// The fixed part of the prompt (instructions, examples, today's date, project list)
/// processed once; per request only the user's sentence is fed through the model.
struct PrefixCache {
    key: String,
    len: usize,
    cache: Vec<Option<(Tensor, Tensor)>>,
}

struct Engine {
    model_id: String,
    weights: Weights,
    tokenizer: Tokenizer,
    device: Device,
    eos: Vec<u32>,
    prefix: Option<PrefixCache>,
}

impl Engine {
    fn load(spec: &ModelSpec, gguf: &Path, tokenizer_path: &Path) -> Result<Engine, String> {
        let device = Device::Cpu;
        let mut file = fs::File::open(gguf).map_err(|e| format!("cannot open model: {e}"))?;
        let content = gguf_file::Content::read(&mut file).map_err(|e| format!("cannot read model: {e}"))?;
        let weights = match spec.arch {
            Arch::Qwen2 => Weights::Qwen2(
                qwen2::ModelWeights::from_gguf(content, &mut file, &device).map_err(|e| format!("cannot load model: {e}"))?,
            ),
            Arch::Llama => Weights::Llama(
                llama::ModelWeights::from_gguf(content, &mut file, &device).map_err(|e| format!("cannot load model: {e}"))?,
            ),
        };
        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| format!("cannot load tokenizer: {e}"))?;
        let eos: Vec<u32> = ["<|im_end|>", "<|endoftext|>", "</s>", "<|eot_id|>"].iter().filter_map(|t| tokenizer.token_to_id(t)).collect();
        Ok(Engine { model_id: spec.id.to_string(), weights, tokenizer, device, eos, prefix: None })
    }

    fn encode(&self, text: &str) -> Result<Vec<u32>, String> {
        Ok(self.tokenizer.encode(text, false).map_err(|e| format!("tokenizer: {e}"))?.get_ids().to_vec())
    }

    fn batch(&self, ids: &[u32]) -> Result<Tensor, String> {
        Tensor::new(ids, &self.device).and_then(|t| t.unsqueeze(0)).map_err(|e| format!("inference: {e}"))
    }

    fn forward(&mut self, x: &Tensor, pos: usize) -> candle_core::Result<Tensor> {
        match &mut self.weights {
            Weights::Qwen2(m) => m.forward(x, pos),
            Weights::Llama(m) => m.forward(x, pos),
        }
    }

    fn clear(&mut self) {
        match &mut self.weights {
            Weights::Qwen2(m) => m.clear_kv_cache(),
            Weights::Llama(m) => m.clear_kv_cache(),
        }
    }

    /// Greedy decoding of a single line. The `prefix` (instructions, examples, date, projects)
    /// is processed once and its KV cache reused; `suffix` carries the user's sentence.
    fn generate(&mut self, prefix: &str, suffix: &str, max_new_tokens: usize) -> Result<String, String> {
        let err = |e: candle_core::Error| format!("inference: {e}");
        let reusable = self.prefix.as_ref().filter(|p| p.key == prefix).map(|p| (p.len, p.cache.clone()));
        let mut pos = match reusable {
            Some((len, cache)) => {
                self.weights.set_kv_cache(cache);
                len
            }
            None => {
                self.clear();
                let ids = self.encode(prefix)?;
                let input = self.batch(&ids)?;
                self.forward(&input, 0).map_err(err)?;
                let cache = self.weights.kv_cache();
                self.prefix = Some(PrefixCache { key: prefix.to_string(), len: ids.len(), cache });
                ids.len()
            }
        };
        let ids = self.encode(suffix)?;
        if ids.is_empty() {
            return Err("empty prompt".into());
        }
        let input = self.batch(&ids)?;
        let logits = self.forward(&input, pos).and_then(|l| l.squeeze(0)).map_err(err)?;
        pos += ids.len();
        let mut sampler = LogitsProcessor::new(7, None, None);
        let mut next = sampler.sample(&logits).map_err(err)?;
        let mut generated: Vec<u32> = Vec::new();
        while generated.len() < max_new_tokens {
            if self.eos.contains(&next) {
                break;
            }
            generated.push(next);
            let so_far = self.tokenizer.decode(&generated, true).map_err(|e| format!("tokenizer: {e}"))?;
            if so_far.contains('\n') {
                break;
            }
            let input = self.batch(&[next])?;
            let logits = self.forward(&input, pos).and_then(|l| l.squeeze(0)).map_err(err)?;
            pos += 1;
            next = sampler.sample(&logits).map_err(err)?;
        }
        self.tokenizer.decode(&generated, true).map_err(|e| format!("tokenizer: {e}"))
    }
}

// ---------------------------------------------------------------------------
// Prompt
// ---------------------------------------------------------------------------

const EXAMPLES: [(&str, &str); 14] = [
    ("reply to sir tommorow arnd 3 in the evening for the hacking chalenge, high prio", "reply to sir tomorrow 3pm !p2 #Hacking Challenge"),
    ("pay the electricty bill evry month, takes 5 min", "pay the electricity bill every month ~5m"),
    ("urgent!! send the report to maya by fri morning", "send the report to maya friday 9am !p1"),
    ("book dentist sometime nxt week, low priority", "book dentist next week !p4"),
    ("buy milk", "buy milk"),
    ("call mom this evening", "call mom today 6pm"),
    ("standup evry weekday at 9 30 for the workshop", "standup every weekday 9:30am #Workshop"),
    ("finish the slides by wed, will take 2 hrs, steps: outline then design", "finish the slides wednesday ~2h +outline +design"),
    ("groceries tomorrow afternoon for personal stuff, remember eggs", "groceries tomorrow 2pm #Personal // remember eggs"),
    ("read the new rust book chapter when i get time", "read the new rust book chapter"),
    ("remind me to renew the car insurance on the 20th, not important", "renew the car insurance on the 20th !p4"),
    ("need to fix the printer asap!!", "fix the printer today !p1"),
    ("team lunch with the workshop guys next thu at 1", "team lunch next thursday 1pm #Workshop"),
    ("water the plants evry 3 days", "water the plants every 3 days"),
];

/// How the few-shot examples are presented to the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptStyle {
    /// Examples listed inside the system prompt as "Input:/Output:" pairs.
    Block,
    /// Examples as alternating user/assistant chat turns, which very small
    /// instruction-tuned models imitate more reliably.
    Turns,
}

/// Chat-turn examples measured best for every model tested (Qwen 0.5B: 10/10 vs 8/10 with
/// a block of examples in the system prompt). `ORDO_AI_STYLE=block` switches for experiments.
fn prompt_style(_spec: &ModelSpec) -> PromptStyle {
    match std::env::var("ORDO_AI_STYLE").as_deref() {
        Ok("block") => PromptStyle::Block,
        _ => PromptStyle::Turns,
    }
}

pub fn build_prompt(text: &str, today: NaiveDate, projects: &[String]) -> String {
    let (prefix, suffix) = build_prompt_parts(text, today, projects);
    format!("{prefix}{suffix}")
}

pub fn build_prompt_parts(text: &str, today: NaiveDate, projects: &[String]) -> (String, String) {
    build_prompt_parts_styled(text, today, projects, PromptStyle::Turns, EXAMPLES.len())
}

/// Returns (fixed prefix, per-request suffix). The prefix only changes with the date or
/// the project list, so its KV cache can be reused across requests.
pub fn build_prompt_parts_styled(
    text: &str,
    today: NaiveDate,
    projects: &[String],
    style: PromptStyle,
    example_count: usize,
) -> (String, String) {
    let examples = &EXAMPLES[..example_count.min(EXAMPLES.len())];
    let project_list = if projects.is_empty() { "(none)".to_string() } else { projects.join(", ") };
    let mut system = String::new();
    system.push_str("You rewrite a to-do sentence into Ordo quick-add syntax. Fix spelling, keep the meaning, drop filler words. Reply with ONE line and nothing else.\n");
    system.push_str("Syntax you may use:\n");
    system.push_str("- date words: today, tomorrow, monday..sunday, next monday, next week, in 3 days, sep 15, eow, eom\n");
    system.push_str("- time: 3pm, 9:30am, 15:00, noon, morning, afternoon, evening\n");
    system.push_str("- priority: !p1 for critical/urgent/asap, !p2 for high/important, !p3 normal, !p4 for low/someday\n");
    system.push_str(&format!("- project: #Name, only when the sentence refers to one of these projects: {project_list}\n"));
    system.push_str("- tags: @tag; estimate: ~30m, ~1h; repeat: every day, every monday, every 2 weeks, monthly, yearly\n");
    system.push_str("- planned day: ^tomorrow; checklist steps: +step; notes: // text\n");
    system.push_str("Keep everything else as the task title; never drop meaningful words. Add a priority only when the sentence says how urgent or important it is, and an estimate only when it gives a duration. Never invent dates, projects or tags that are not implied.\n");
    system.push_str(&format!("Today is {}.\n", today.format("%A %Y-%m-%d")));
    match style {
        PromptStyle::Block => {
            system.push_str("\nExamples (their projects were Hacking Challenge, Workshop and Personal):\n");
            for (input, output) in examples {
                system.push_str(&format!("Input: {input}\nOutput: {output}\n"));
            }
            let prefix = format!("<|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n");
            let suffix = format!("Input: {text}\nOutput:<|im_end|>\n<|im_start|>assistant\n");
            (prefix, suffix)
        }
        PromptStyle::Turns => {
            system.push_str("The examples that follow used the projects Hacking Challenge, Workshop and Personal.\n");
            let mut prefix = format!("<|im_start|>system\n{system}<|im_end|>\n");
            for (input, output) in examples {
                prefix.push_str(&format!("<|im_start|>user\n{input}<|im_end|>\n<|im_start|>assistant\n{output}<|im_end|>\n"));
            }
            let suffix = format!("<|im_start|>user\n{text}<|im_end|>\n<|im_start|>assistant\n");
            (prefix, suffix)
        }
    }
}

fn clean_output(raw: &str, fallback: &str) -> String {
    let mut line = raw.lines().next().unwrap_or("").trim().to_string();
    for prefix in ["Output:", "output:"] {
        if let Some(rest) = line.strip_prefix(prefix) {
            line = rest.trim().to_string();
        }
    }
    let line = line.trim_matches(|c| c == '"' || c == '\'' || c == '`').trim().to_string();
    if line.is_empty() { fallback.trim().to_string() } else { line }
}

/// Small models sometimes invent `#Project` names. Only existing projects may be referenced,
/// so an unknown `#word` loses its `#` and stays in the title instead of creating a project.
fn drop_unknown_projects(line: &str, projects: &[String]) -> String {
    let words: Vec<&str> = line.split_whitespace().collect();
    let lower: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();
    let mut out: Vec<String> = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        if let Some(rest) = words[i].strip_prefix('#') {
            let mut matched = 0;
            for len in (1..=4.min(words.len() - i)).rev() {
                let candidate = lower[i..i + len].join(" ");
                let candidate = candidate.trim_start_matches('#').trim_end_matches([',', '.', ';']);
                if projects.iter().any(|p| p.to_lowercase() == candidate) {
                    matched = len;
                    break;
                }
            }
            if matched > 0 {
                out.extend(words[i..i + matched].iter().map(|w| w.to_string()));
                i += matched;
            } else {
                if !rest.is_empty() {
                    out.push(rest.to_string());
                }
                i += 1;
            }
            continue;
        }
        out.push(words[i].to_string());
        i += 1;
    }
    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_projects_are_not_created() {
        let projects = vec!["Hacking Challenge".to_string(), "Personal".to_string()];
        assert_eq!(drop_unknown_projects("fix the login bug tomorrow !p1 #Bug", &projects), "fix the login bug tomorrow !p1 Bug");
        assert_eq!(drop_unknown_projects("reply to sir 3pm #Hacking Challenge !p2", &projects), "reply to sir 3pm #Hacking Challenge !p2");
        assert_eq!(drop_unknown_projects("groceries #personal", &projects), "groceries #personal");
    }

    #[test]
    fn prompt_lists_projects_and_date() {
        let p = build_prompt("hello", NaiveDate::from_ymd_opt(2026, 9, 11).unwrap(), &["Workshop".into()]);
        assert!(p.contains("Friday 2026-09-11"));
        assert!(p.contains("projects: Workshop"));
        assert!(p.ends_with("<|im_start|>assistant\n"));
    }

    #[test]
    fn output_is_cleaned() {
        assert_eq!(clean_output("Output: \"call mom tomorrow\"\nextra", "x"), "call mom tomorrow");
        assert_eq!(clean_output("   \n", "fallback text"), "fallback text");
    }
}
