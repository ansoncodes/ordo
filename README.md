# Ordo — priority-first task management

Ordo is a to-do app written in Rust that behaves like a modern SaaS product: a single binary that serves a polished web UI on localhost, keeps your data in one JSON file, and does the thinking a plain checklist never does — ranking your work, planning your day, and telling you why.

```
cargo run --release
```

That opens `http://localhost:3000` in your browser with a demo workspace you can explore or reset.

## What makes it different from a traditional to-do list

| Feature | What it does |
|---|---|
| **Focus score** | Every open task gets an explainable 0–100 score from priority, deadline proximity, whether it is planned for today, starring, waiting time, quick wins and checklist momentum. The **Focus** view is your backlog sorted by that score; click a score to see the reasons. |
| **Priority Matrix** | An Eisenhower 2×2 (Do first / Schedule / Do quickly / Someday). Importance comes from the P1–P4 priority, urgency from deadlines and today's plan. Drag tasks between quadrants; Ordo adjusts priority and planning and tells you when a deadline keeps a task urgent. |
| **Plan my day** | Set a daily capacity (default 6 h). Ordo fills today with the highest-scoring tasks that fit, always including anything overdue or due today, and shows planned time against capacity. |
| **Natural-language quick add** | `Send invoice to Acme tomorrow 5pm !p1 #Work @finance ~45m every friday` becomes a titled, dated, prioritised, estimated, recurring task in the right project — with live chips showing what was understood. Parsing runs in Rust. |
| **Focus timer** | Pomodoro-style sessions per task (configurable length). Time is logged to the task and to analytics so you can compare estimates with reality. |
| **Analytics** | Completions per day, streaks, on-time rate, most productive weekdays, backlog by priority, project progress, cycle time, estimate accuracy. |
| **Recurring tasks** | Daily, weekdays, weekly on a day, every N weeks/months, yearly. Completing one spawns the next occurrence, never a pile of past dates. |
| **Checklists, tags, projects, notes, starring** | The basics, done properly. Double-click a title to rename it in place; hover a row for star, plan-today, move-to-tomorrow, focus and trash. |
| **Today hero** | A greeting, a daily progress ring, and planned time against capacity, in one card. First-run tips explain the three things worth knowing, then get out of the way. |
| **Trash with undo** | Deletion is soft; restore from Trash or undo from the toast. |
| **Command palette & shortcuts** | `Ctrl+K` for everything; single-key navigation (`1`–`8`), `N` new task, `J`/`K` move, `C` complete, `T` plan today. |
| **Bulk actions** | Shift-click to select several tasks, then complete, prioritise, move, plan or delete them together. |
| **Themes, responsive, offline** | System/light/dark, two dark styles (soft tinted or plain pure black), nine preset accents plus any custom hex colour, all switchable live from Settings or the palette; works on a phone-sized window; no external requests at all. |
| **Export / import** | Full JSON export and merge/replace import. |

## Quick-add syntax

| Write | Meaning |
|---|---|
| `#Work`, `#Home Renovation` | Project (created if it does not exist; multi-word names match existing projects) |
| `@finance` | Tag |
| `!p1` `!high` `!!!` `p2` | Priority (p1 critical, p2 high, p3 medium, p4 low) |
| `today` `tomorrow` `friday` `next monday` `in 3 days` `in 2 weeks` `sep 15` `2026-10-01` `eow` `eom` `weekend` `tonight` | Due date |
| `5pm` `at 17:30` `noon` | Due time |
| `~45m` `1h30m` `2 hours` `in 10 mins` `for about 1h` | Estimate |
| `every day` `every weekday` `every friday` `every 2 weeks` `monthly` `yearly` | Recurrence |
| `^tomorrow` `^next monday` | Planned-for date (when you will work on it), separate from the deadline |
| `+outline +draft the post` | Checklist steps |
| `// any text` | Notes (everything after the double slash; URLs in the title are left alone) |
| `hacking challenge project`, `for SMEC Technologies` | Existing projects are recognised from plain words too. Single-word names need "project" after them or a `#`, so "personal email" stays a title. |
| `*` | Star |

Everything at once: `Send invoice to Acme tomorrow 5pm !p1 #Work @finance ~45m every friday ^today +draft +send // ask for the PO number`.

Ambiguous words stay in the title: "Buy sun cream" is not due on Sunday. Typos in keywords, project names and tags are tolerated (`tommorow`, `fridy`, `#hacking chalenge`, `@finnance`) within a small edit distance; short words are never "corrected".

## On-device AI (optional, no internet, no API key)

For sentences the rules cannot read ("reply to sir tommorow arnd 3 in the evening, high prio"), Ordo can run a small language model on your own computer. Open Settings → On-device AI, download a model once, and turn on the AI button. Three tiers are offered:

| Tier | Model | Download | RAM in use | Accuracy on the 10-sentence benchmark | Per sentence |
|---|---|---|---|---|---|
| Accurate | Qwen 2.5 0.5B (8-bit) | 676 MB | ~800 MB | 10 / 10 | ~0.8 s |
| Light | Qwen 2.5 0.5B (4-bit) | 429 MB | ~550 MB | 8 / 10 | ~0.8 s |
| Ultra light | SmolLM2 135M (8-bit) | 145 MB | ~200 MB | 7 / 10 | ~0.7 s |

Timings are from a laptop CPU after warm-up. The first sentence after launch also loads the model (1 to 3 s) and processes the fixed instructions once (3 to 6 s). The model is unloaded automatically after 10 minutes without use. Whatever the model gets wrong, the rules layer still applies afterwards, and the rewrite is shown for review before the task is added unless automatic mode is on. The model rewrites your sentence into the quick-add syntax above ("reply to sir tomorrow 3pm !p2"), shows you the result, and the deterministic parser does the date arithmetic. You can also let it rewrite every new task automatically.

- Inference runs with [candle](https://github.com/huggingface/candle) on the CPU in the same process; nothing you type leaves the machine.
- The model stays in memory after first use (about 1 GB for Qwen) and can be unloaded from Settings.
- Models are stored in `%LOCALAPPDATA%\Ordo\models` and shared by the CLI and desktop builds.
- Any GGUF model with a Qwen2 or Llama architecture can be added to `src/ai.rs`, including a purpose-trained tiny one.
- Speed: the fixed part of the prompt is processed once per day and cached, so a sentence takes well under a second on a laptop CPU after the first use (which loads the model, a few seconds). `.cargo/config.toml` enables AVX2/FMA so the SIMD kernels are used; the resulting binaries need a CPU from 2013 or later.

## Running

Requirements: a Rust toolchain (1.75+). No database, Node, or build step.

```
cargo run --release              # builds and starts, opens the browser
cargo run --release -- --no-open # do not open the browser
cargo run --release -- --port 8080 --data ~/ordo-data
cargo test                       # parser, priority engine and recurrence tests
```

| Option / env | Default | Purpose |
|---|---|---|
| `--port`, `ORDO_PORT` | 3000 | Port (falls forward to the next free port up to +9) |
| `--data`, `ORDO_DATA_DIR` | `./data` | Directory holding `ordo.json` |
| `--no-open`, `ORDO_NO_OPEN` | open | Skip launching the browser |

Data is written atomically (temp file + rename) after every change, so a crash never leaves a half-written file.

### Troubleshooting: "An Application Control policy has blocked this file (os error 4551)"

On Windows 11 with **Smart App Control** turned on, Cargo may be refused permission to run the tiny build-script executables it compiles for dependencies (`proc-macro2`, `serde`, `httparse`, …). The failure looks like `failed to run custom build command for …` (or `can't find crate for windows_interface` when a proc-macro DLL is blocked) and happens for any fresh profile or target directory, so `cargo build --release` and the Tauri desktop build can fail even when the existing `cargo build` (debug) works. Fixes: turn Smart App Control off (Windows Security → App & browser control → Smart App Control settings; note Windows does not let you turn it back on without a reinstall), build on a machine without it, or keep using the debug build with `cargo run`.

## Desktop app (Windows installer)

The standard way to ship Ordo as a Windows application is the Tauri shell in `src-tauri/`. It runs the same Rust server in-process on a random loopback port and shows the UI in a native WebView2 window, with per-user data in `%APPDATA%\com.ordo.desktop\ordo.json`.

```
cargo install tauri-cli --version "^2"   # once
cargo tauri dev                           # native window, debug build
cargo tauri build                         # installers
```

`cargo tauri build` produces `target\release\bundle\nsis\Ordo_0.1.0_x64-setup.exe` and `target\release\bundle\msi\Ordo_0.1.0_x64_en-US.msi`. Both register a Start Menu entry and an uninstaller, and bootstrap the WebView2 runtime if a machine lacks it. Icons live in `src-tauri/icons/`; regenerate them from any square PNG with `cargo tauri icon your-icon.png`.

If bundling fails with `failed to bundle project: timeout: global`, the CLI's download of the NSIS or WiX toolset was too slow. Fetch them once by hand into Tauri's cache and rerun the build: extract `nsis-3.11.zip` (from `github.com/tauri-apps/binary-releases`) to `%LOCALAPPDATA%\tauri\NSIS`, put `nsis_tauri_utils.dll` (from `github.com/tauri-apps/nsis-tauri-utils`) in `%LOCALAPPDATA%\tauri\NSIS\Plugins\x86-unicode\additional\`, and extract `wix314-binaries.zip` (from `github.com/wixtoolset/wix3`) to `%LOCALAPPDATA%\tauri\WixTools314`.

Before distributing to other people, sign the binaries (set `bundle.windows.certificateThumbprint` in `src-tauri/tauri.conf.json`, or use Azure Trusted Signing). Unsigned installers trigger SmartScreen warnings and are refused outright by Smart App Control on users' PCs. Auto-updates can be added later with `tauri-plugin-updater`.

## Architecture

```
src/
  lib.rs         The `ordo` library: everything below, shared by both front doors
  main.rs        CLI/server binary: options, port binding, opens a browser tab
  api.rs         Axum routes, request/response types, static assets (embedded with include_str!)
  model.rs       Task, Project, Settings, Recurrence, Database
  priority.rs    Focus score + Eisenhower quadrant (explainable)
  parser.rs      Natural-language quick-add parser
  recurrence.rs  Next-occurrence calculation
  analytics.rs   Streaks, rates, time series
  store.rs       In-memory database with atomic JSON persistence
  seed.rs        Demo workspace
static/
  index.html, app.css, app.js   The web UI (vanilla JS, no framework, no CDN)
src-tauri/
  src/lib.rs, src/main.rs       Desktop app: starts the server, opens a native window
  tauri.conf.json, icons/, capabilities/   Tauri configuration and bundling assets
```

The Rust side owns all domain logic. The browser only renders what the API returns and re-fetches `/api/bootstrap` after each mutation, so scores, quadrants and counts are always computed by the same engine.

### API overview

| Method & path | Purpose |
|---|---|
| `GET /api/bootstrap` | Everything the UI needs: settings, projects, scored tasks, tags |
| `POST /api/tasks/quick` · `GET /api/parse?q=` | Natural-language create / live preview |
| `POST /api/tasks` · `PATCH /api/tasks/{id}` · `DELETE /api/tasks/{id}` | CRUD (delete is soft) |
| `POST /api/tasks/{id}/complete` `…/reopen` `…/restore` `…/duplicate` | Lifecycle; complete returns the next recurrence |
| `POST /api/tasks/{id}/quadrant` | Move within the priority matrix |
| `POST /api/tasks/{id}/time` · `…/subtasks` | Log focus time, manage checklist |
| `POST /api/tasks/bulk` | Bulk complete / priority / project / schedule / delete |
| `POST /api/plan-day` | Capacity-aware day planning |
| `GET /api/analytics` · `GET /api/search?q=` | Insights and search |
| `GET/POST/PATCH/DELETE /api/projects…` · `GET/PATCH /api/settings` | Projects and settings |
| `GET /api/export` · `POST /api/import?mode=merge|replace` | Backup and restore |

## Keyboard shortcuts

`N` new task · `/` search · `Ctrl+K` palette · `1`–`8` views · `J`/`K` next/previous · `C` complete · `S` star · `T` plan today · `F` focus session · `Del` trash · `D` dark mode · `?` help · `Esc` close.
