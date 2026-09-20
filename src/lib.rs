//! Ordo core: the priority engine, natural-language parser, recurrence rules,
//! analytics, persistence and the HTTP API that serves the embedded web UI.
//!
//! Two front doors use this library:
//! - `src/main.rs`: the CLI/server binary (`cargo run`), which opens a browser tab.
//! - `src-tauri/`: the desktop application, which runs the same server in-process
//!   and shows the UI in a native window.

pub mod ai;
pub mod analytics;
pub mod api;
pub mod llama;
pub mod model;
pub mod parser;
pub mod priority;
pub mod qwen2;
pub mod recurrence;
pub mod seed;
pub mod store;
