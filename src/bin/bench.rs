//! Gold-set benchmark for Ordo's language understanding.
//!
//! Scores the deterministic parser against hand-written examples in `eval/gold.json`,
//! per slot and per category, so any replacement model has a number to beat.
//!
//! ```text
//! cargo run --bin bench                      # the table
//! cargo run --bin bench -- -v                # every failure, slot by slot
//! cargo run --bin bench -- --category typo   # one slice
//! cargo run --bin bench -- --skip semantic-inference
//! cargo run --bin bench -- --export-spans eval/spans.jsonl
//! ```
//!
//! The engine column is deliberately just the parser for now: it is the baseline the
//! encoder has to beat, and it needs no model download. `Scored` is engine-agnostic,
//! so a second column slots in beside it later.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use chrono::{NaiveDate, NaiveTime};
use ordo::model::Priority;
use ordo::parser;
use serde::Deserialize;

const GOLD_PATH: &str = "eval/gold.json";

// ---------------------------------------------------------------------------
// Gold file
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct Gold {
    #[serde(default)]
    projects: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    text: String,
    today: String,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    spans: Vec<GoldSpan>,
    /// Marks an expectation that encodes a product decision someone still has to confirm.
    #[serde(default)]
    review: bool,
    #[serde(default)]
    note: Option<String>,
    expect: Expect,
}

/// A span is stored as its surface text, not as offsets: offsets in a hand-edited file
/// silently rot the moment `text` changes. Offsets are resolved here instead.
#[derive(Deserialize)]
struct GoldSpan {
    label: String,
    text: String,
    /// 1-based, required only when the surface text appears more than once.
    #[serde(default)]
    occurrence: Option<usize>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Expect {
    title: String,
    project: Option<String>,
    tags: Vec<String>,
    priority: Option<String>,
    due_date: Option<String>,
    due_time: Option<String>,
    estimate_minutes: Option<u32>,
    recurrence: Option<String>,
    scheduled: Option<String>,
    subtasks: Vec<String>,
    notes: String,
}

// ---------------------------------------------------------------------------
// Scoring
// ---------------------------------------------------------------------------

/// One slot's outcome for one case. Engine-agnostic: the parser fills these in today,
/// a tagger fills the same shape tomorrow.
struct Scored {
    slot: &'static str,
    ok: bool,
    gold_present: bool,
    pred_present: bool,
    gold: String,
    pred: String,
}

const SLOTS: [&str; 10] =
    ["Title", "Date", "Time", "Priority", "Project", "Tags", "Duration", "Recurrence", "Scheduled", "Notes"];

/// Lowercase, collapse whitespace, drop trailing punctuation - so the score measures
/// understanding rather than comma placement.
fn norm(s: &str) -> String {
    let joined = s.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    joined.trim_matches(|c: char| matches!(c, ',' | '.' | ';' | '!' | '?' | ':')).to_string()
}

fn prio_str(p: &Priority) -> &'static str {
    match p {
        Priority::P1 => "p1",
        Priority::P2 => "p2",
        Priority::P3 => "p3",
        Priority::P4 => "p4",
    }
}

fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| format!("bad date {s:?}: {e}"))
}

fn parse_time(s: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(s, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(s, "%H:%M"))
        .map_err(|e| format!("bad time {s:?}: {e}"))
}

fn show<T: ToString>(v: &Option<T>) -> String {
    v.as_ref().map(|x| x.to_string()).unwrap_or_else(|| "-".into())
}

fn opt_slot<T: PartialEq + ToString>(slot: &'static str, gold: Option<T>, pred: Option<T>) -> Scored {
    Scored {
        slot,
        ok: gold == pred,
        gold_present: gold.is_some(),
        pred_present: pred.is_some(),
        gold: show(&gold),
        pred: show(&pred),
    }
}

fn score(
    case: &Case,
    input: &str,
    title_override: Option<&str>,
    today: NaiveDate,
    projects: &[String],
    tags: &[String],
) -> Result<Vec<Scored>, String> {
    let mut p = parser::parse(input, today, projects, tags);
    if let Some(t) = title_override {
        p.title = t.to_string();
    }
    let e = &case.expect;

    let gold_date = e.due_date.as_deref().map(parse_date).transpose()?;
    let gold_time = e.due_time.as_deref().map(parse_time).transpose()?;
    let gold_sched = e.scheduled.as_deref().map(parse_date).transpose()?;

    let gold_tags: Vec<String> = e.tags.iter().map(|t| norm(t)).collect();
    let pred_tags: Vec<String> = p.tags.iter().map(|t| norm(t)).collect();
    let (mut gt, mut pt) = (gold_tags.clone(), pred_tags.clone());
    gt.sort();
    pt.sort();

    let gold_steps: Vec<String> = e.subtasks.iter().map(|s| norm(s)).collect();
    let pred_steps: Vec<String> = p.subtasks.iter().map(|s| norm(s)).collect();

    Ok(vec![
        Scored {
            slot: "Title",
            ok: norm(&e.title) == norm(&p.title),
            gold_present: true,
            pred_present: !p.title.trim().is_empty(),
            gold: e.title.clone(),
            pred: p.title.clone(),
        },
        opt_slot("Date", gold_date, p.due_date),
        opt_slot("Time", gold_time, p.due_time),
        opt_slot(
            "Priority",
            e.priority.as_deref().map(|s| s.to_lowercase()),
            p.priority.as_ref().map(|x| prio_str(x).to_string()),
        ),
        opt_slot("Project", e.project.as_deref().map(norm), p.project.as_deref().map(norm)),
        Scored {
            slot: "Tags",
            ok: gt == pt,
            gold_present: !gt.is_empty(),
            pred_present: !pt.is_empty(),
            gold: gold_tags.join(","),
            pred: pred_tags.join(","),
        },
        opt_slot("Duration", e.estimate_minutes, p.estimate_minutes),
        opt_slot(
            "Recurrence",
            e.recurrence.as_deref().map(norm),
            p.recurrence.as_ref().map(|r| norm(&r.label())),
        ),
        opt_slot("Scheduled", gold_sched, p.scheduled),
        Scored {
            slot: "Notes",
            // Steps and notes share a slot: both are free text the parser lifts out of the title.
            ok: norm(&e.notes) == norm(&p.notes) && gold_steps == pred_steps,
            gold_present: !e.notes.trim().is_empty() || !gold_steps.is_empty(),
            pred_present: !p.notes.trim().is_empty() || !pred_steps.is_empty(),
            gold: format!("{}{}", e.notes, fmt_steps(&gold_steps)),
            pred: format!("{}{}", p.notes, fmt_steps(&pred_steps)),
        },
    ])
}

fn fmt_steps(steps: &[String]) -> String {
    if steps.is_empty() {
        String::new()
    } else {
        format!(" [+{}]", steps.join(" +"))
    }
}

// ---------------------------------------------------------------------------
// Tagger predictions
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct Prediction {
    id: String,
    #[serde(default)]
    entities: Vec<PredSpan>,
}

#[derive(Deserialize)]
struct PredSpan {
    label: String,
    text: String,
}

/// Abbreviations the tagger correctly identifies but the parser cannot yet resolve.
/// This is the "cheap normalization" layer; it belongs in `parser.rs` eventually, and
/// living here for now keeps it visible as a separate contribution to the score.
const ABBREVIATIONS: [(&str, &str); 14] = [
    ("tmrw", "tomorrow"),
    ("tmw", "tomorrow"),
    ("tmmrw", "tomorrow"),
    ("2moro", "tomorrow"),
    ("2morrow", "tomorrow"),
    ("2day", "today"),
    ("nxt", "next"),
    ("arnd", "around"),
    ("mrng", "morning"),
    ("evng", "evening"),
    ("eod", "today"),
    ("thurs", "thursday"),
    ("tues", "tuesday"),
    ("mins", "minutes"),
];

/// Leading words that carry no meaning once a span has already been identified as a date
/// or a time. "in" and "at" are deliberately absent: "in 3 days" and "at 9:30" need them.
const DROP_LEADING: [&str; 8] = ["by", "before", "due", "sometime", "on", "around", "about", "roughly"];

fn normalize_span(text: &str) -> String {
    let mut words: Vec<String> = text
        .split_whitespace()
        .map(|w| {
            let lower = w.to_lowercase();
            let base = lower.trim_end_matches(|c: char| c == ',' || c == '.');
            let base = base.strip_suffix("ish").filter(|r| !r.is_empty()).unwrap_or(base);
            ABBREVIATIONS
                .iter()
                .find(|(from, _)| *from == base)
                .map(|(_, to)| to.to_string())
                .unwrap_or_else(|| base.to_string())
        })
        .collect();
    while words.first().map(|w| DROP_LEADING.contains(&w.as_str())).unwrap_or(false) && words.len() > 1 {
        words.remove(0);
    }
    words.join(" ")
}

/// Splits tagged spans into (metadata string, title).
///
/// The title is taken verbatim from its span and never re-parsed, and the metadata string
/// holds only the other spans. This mirrors what a real tagger pipeline does - each span is
/// resolved independently - rather than round-tripping the whole sentence back through the
/// parser, which would let an unresolvable fragment ("around 5" -> "5") leak into the title.
fn reconstruct(pred: &Prediction) -> (String, String) {
    let (mut title, mut parts, mut notes) = (Vec::new(), Vec::new(), Vec::new());
    for e in &pred.entities {
        let raw = e.text.trim();
        if raw.is_empty() {
            continue;
        }
        match e.label.as_str() {
            "TITLE" => title.push(raw.to_string()),
            "DATE" | "TIME" | "DURATION" | "RECURRENCE" => parts.push(normalize_span(raw)),
            "PRIORITY_P1" => parts.push("!p1".into()),
            "PRIORITY_P2" => parts.push("!p2".into()),
            "PRIORITY_P3" => parts.push("!p3".into()),
            "PRIORITY_P4" => parts.push("!p4".into()),
            "PROJECT" => {
                let name = raw.trim_start_matches('#');
                let name = name
                    .split_once(' ')
                    .filter(|(head, _)| matches!(head.to_lowercase().as_str(), "for" | "in" | "under" | "into"))
                    .map(|(_, rest)| rest)
                    .unwrap_or(name);
                parts.push(format!("#{name}"));
            }
            "TAG" => parts.push(format!("@{}", raw.trim_start_matches('@'))),
            "SCHEDULED" => parts.push(format!("^{}", normalize_span(raw.trim_start_matches('^')))),
            "STEP" => parts.push(format!("+{}", raw.trim_start_matches('+'))),
            "NOTE" => notes.push(raw.trim_start_matches('/').trim().to_string()),
            _ => {}
        }
    }
    let mut out = parts.join(" ");
    if !notes.is_empty() {
        out.push_str(" // ");
        out.push_str(&notes.join(" "));
    }
    (out.trim().to_string(), title.join(" "))
}

fn load_predictions(path: &str) -> Result<BTreeMap<String, (String, String)>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let mut map = BTreeMap::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let pred: Prediction =
            serde_json::from_str(line).map_err(|e| format!("{path} line {}: {e}", i + 1))?;
        map.insert(pred.id.clone(), reconstruct(&pred));
    }
    Ok(map)
}

// ---------------------------------------------------------------------------
// Aggregation
// ---------------------------------------------------------------------------

#[derive(Default, Clone, Copy)]
struct Tally {
    total: u32,
    correct: u32,
    present: u32,
    present_correct: u32,
    /// Gold had no value and the engine produced one anyway.
    invented: u32,
}

fn pct(n: u32, d: u32) -> String {
    if d == 0 {
        "   -  ".into()
    } else {
        format!("{:5.1}%", (n as f64 / d as f64) * 100.0)
    }
}

// ---------------------------------------------------------------------------
// Span resolution / export
// ---------------------------------------------------------------------------

fn resolve_span(text: &str, span: &GoldSpan) -> Result<(usize, usize), String> {
    let hits: Vec<usize> = text.match_indices(span.text.as_str()).map(|(i, _)| i).collect();
    let start = match (hits.len(), span.occurrence) {
        (0, _) => return Err(format!("{} span {:?} does not occur in the text", span.label, span.text)),
        (_, Some(n)) => *hits
            .get(n.saturating_sub(1))
            .ok_or_else(|| format!("{} span {:?} has no occurrence {n}", span.label, span.text))?,
        (1, None) => hits[0],
        (n, None) => {
            return Err(format!(
                "{} span {:?} is ambiguous ({n} matches) - add \"occurrence\"",
                span.label, span.text
            ))
        }
    };
    Ok((start, start + span.text.len()))
}

/// Writes offset-resolved cases as JSONL for the Python training pipeline.
fn export_spans(gold: &Gold, path: &str) -> Result<usize, String> {
    let mut out = String::new();
    for case in &gold.cases {
        let mut ents = Vec::new();
        for span in &case.spans {
            let (start, end) = resolve_span(&case.text, span)?;
            // PRIORITY carries its class in the label, matching the tagger's scheme, so an
            // export of the gold spans can be fed straight back in as an oracle prediction.
            let label = match (span.label.as_str(), case.expect.priority.as_deref()) {
                ("PRIORITY", Some(p)) => format!("PRIORITY_{}", p.to_uppercase()),
                _ => span.label.clone(),
            };
            ents.push(serde_json::json!({ "start": start, "end": end, "label": label, "text": span.text }));
        }
        let line = serde_json::json!({
            "id": case.id,
            "text": case.text,
            "today": case.today,
            "categories": case.categories,
            "entities": ents,
        });
        let _ = writeln!(out, "{line}");
    }
    std::fs::write(path, out).map_err(|e| format!("cannot write {path}: {e}"))?;
    Ok(gold.cases.len())
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

struct Opts {
    verbose: bool,
    category: Option<String>,
    skip: Vec<String>,
    id: Option<String>,
    export: Option<String>,
    predictions: Option<String>,
}

fn parse_opts() -> Opts {
    let mut o =
        Opts { verbose: false, category: None, skip: Vec::new(), id: None, export: None, predictions: None };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-v" | "--verbose" => o.verbose = true,
            "--category" | "-c" => {
                i += 1;
                o.category = args.get(i).cloned();
            }
            "--skip" => {
                i += 1;
                o.skip.extend(args.get(i).cloned());
            }
            "--id" => {
                i += 1;
                o.id = args.get(i).cloned();
            }
            "--export-spans" => {
                i += 1;
                o.export = args.get(i).cloned();
            }
            "--predictions" => {
                i += 1;
                o.predictions = args.get(i).cloned();
            }
            "-h" | "--help" => {
                println!(
                    "Ordo language benchmark\n\n\
                     USAGE:\n  cargo run --bin bench -- [OPTIONS]\n\n\
                     OPTIONS:\n\
                     \x20 -v, --verbose            Show every failure, slot by slot\n\
                     \x20 -c, --category NAME      Only cases carrying this category\n\
                     \x20     --skip NAME          Exclude a category (repeatable)\n\
                     \x20     --id ID              Run a single case\n\
                     \x20     --export-spans PATH  Write offset-resolved spans as JSONL\n\
                     \x20     --predictions PATH   Score tagger spans instead of raw text\n\
                     \x20 -h, --help               Show this help"
                );
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown argument: {other} (try --help)");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    o
}

fn main() {
    let opts = parse_opts();
    let text = match std::fs::read_to_string(GOLD_PATH) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Cannot read {GOLD_PATH}: {e}\nRun this from the project root.");
            std::process::exit(1);
        }
    };
    let gold: Gold = match serde_json::from_str(&text) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{GOLD_PATH} is not valid: {e}");
            std::process::exit(1);
        }
    };

    // Spans are validated on every run, so a bad edit is caught immediately rather than
    // at training time.
    let mut span_errors = Vec::new();
    for case in &gold.cases {
        for span in &case.spans {
            if let Err(e) = resolve_span(&case.text, span) {
                span_errors.push(format!("  {}: {e}", case.id));
            }
        }
    }
    if !span_errors.is_empty() {
        eprintln!("Span problems in {GOLD_PATH}:");
        for e in &span_errors {
            eprintln!("{e}");
        }
        std::process::exit(1);
    }

    if let Some(path) = &opts.export {
        match export_spans(&gold, path) {
            Ok(n) => println!("Wrote {n} cases with resolved offsets to {path}"),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        return;
    }

    let selected: Vec<&Case> = gold
        .cases
        .iter()
        .filter(|c| opts.id.as_ref().map(|id| &c.id == id).unwrap_or(true))
        .filter(|c| opts.category.as_ref().map(|k| c.categories.iter().any(|x| x == k)).unwrap_or(true))
        .filter(|c| !c.categories.iter().any(|x| opts.skip.contains(x)))
        .collect();

    if selected.is_empty() {
        eprintln!("No cases matched the filters.");
        std::process::exit(1);
    }

    let mut by_slot: BTreeMap<&'static str, Tally> = SLOTS.iter().map(|s| (*s, Tally::default())).collect();
    let mut by_category: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    let mut exact = 0u32;
    let mut failures: Vec<(String, Vec<Scored>)> = Vec::new();
    let mut review_count = 0u32;

    let predictions = match &opts.predictions {
        Some(path) => match load_predictions(path) {
            Ok(p) => Some(p),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
        None => None,
    };
    let mut missing = 0u32;

    for case in &selected {
        if case.review {
            review_count += 1;
        }
        let today = match parse_date(&case.today) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{}: {e}", case.id);
                std::process::exit(1);
            }
        };
        // With predictions, the parser runs over the tagger's reconstructed quick-add
        // string rather than the raw sentence - same resolvers, so the numbers compare.
        let (input, title_override) = match &predictions {
            Some(map) => match map.get(&case.id) {
                Some((meta, title)) => (meta.clone(), Some(title.clone())),
                None => {
                    missing += 1;
                    (String::new(), Some(String::new()))
                }
            },
            None => (case.text.clone(), None),
        };
        let scored = match score(case, &input, title_override.as_deref(), today, &gold.projects, &gold.tags) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}: {e}", case.id);
                std::process::exit(1);
            }
        };

        let all_ok = scored.iter().all(|s| s.ok);
        if all_ok {
            exact += 1;
        } else {
            failures.push((case.id.clone(), scored.iter().filter(|s| !s.ok).map(clone_scored).collect()));
        }

        for s in &scored {
            let t = by_slot.get_mut(s.slot).expect("known slot");
            t.total += 1;
            if s.ok {
                t.correct += 1;
            }
            if s.gold_present {
                t.present += 1;
                if s.ok {
                    t.present_correct += 1;
                }
            } else if s.pred_present {
                t.invented += 1;
            }
        }
        for cat in &case.categories {
            let e = by_category.entry(cat.clone()).or_insert((0, 0));
            e.1 += 1;
            if all_ok {
                e.0 += 1;
            }
        }
    }

    let n = selected.len() as u32;
    let engine = match &opts.predictions {
        Some(p) => format!("tagger ({p}) + parser resolution"),
        None => "parser only".to_string(),
    };
    println!("\n  Ordo language benchmark  ·  {n} cases  ·  engine: {engine}\n");
    if missing > 0 {
        println!("  warning: {missing} case(s) had no prediction and scored as empty\n");
    }
    println!("  {:<12}{:<18}{:<20}{}", "Slot", "Accuracy", "Recall (present)", "Invented");
    println!("  {}", "-".repeat(62));
    for slot in SLOTS {
        let t = by_slot[slot];
        let acc = format!("{}  ({}/{})", pct(t.correct, t.total), t.correct, t.total);
        let recall = if slot == "Title" {
            "-".to_string()
        } else {
            format!("{}  ({}/{})", pct(t.present_correct, t.present), t.present_correct, t.present)
        };
        let invented = if t.invented == 0 { "-".to_string() } else { t.invented.to_string() };
        println!("  {slot:<12}{acc:<18}{recall:<20}{invented}");
    }
    println!("  {}", "-".repeat(62));
    println!("  {:<12}{}  ({}/{})", "Exact match", pct(exact, n), exact, n);

    println!("\n  Weakest categories (exact match)\n");
    let mut cats: Vec<(&String, &(u32, u32))> = by_category.iter().collect();
    cats.sort_by(|a, b| {
        let ra = a.1 .0 as f64 / a.1 .1 as f64;
        let rb = b.1 .0 as f64 / b.1 .1 as f64;
        ra.partial_cmp(&rb).unwrap_or(std::cmp::Ordering::Equal).then_with(|| b.1 .1.cmp(&a.1 .1))
    });
    for (name, (ok, total)) in cats.iter().take(12) {
        println!("  {:<24}{}  ({}/{})", name, pct(*ok, *total), ok, total);
    }

    if review_count > 0 {
        println!("\n  {review_count} case(s) flagged \"review\": the expected value is a product decision,");
        println!("  not a fact. Confirm those before trusting the headline number.");
    }

    if !failures.is_empty() {
        println!("\n  {} failing case(s)", failures.len());
        if opts.verbose {
            for (id, slots) in &failures {
                let case = selected.iter().find(|c| &c.id == id).expect("case");
                println!("\n  {id}  {:?}", case.text);
                if let Some(note) = &case.note {
                    println!("    note: {note}");
                }
                for s in slots {
                    println!("    {:<11} want {:<34} got {}", s.slot, truncate(&s.gold, 32), truncate(&s.pred, 32));
                }
            }
        } else {
            let ids: Vec<&str> = failures.iter().map(|(id, _)| id.as_str()).collect();
            println!("  {}", ids.join(", "));
            println!("\n  Re-run with -v for slot-level detail.");
        }
    }
    println!();
}

fn clone_scored(s: &Scored) -> Scored {
    Scored {
        slot: s.slot,
        ok: s.ok,
        gold_present: s.gold_present,
        pred_present: s.pred_present,
        gold: s.gold.clone(),
        pred: s.pred.clone(),
    }
}

fn truncate(s: &str, n: usize) -> String {
    let s = if s.trim().is_empty() { "-" } else { s };
    if s.chars().count() > n {
        format!("{}…", s.chars().take(n - 1).collect::<String>())
    } else {
        s.to_string()
    }
}
