//! Ordo - priority-first task management.
//!
//! The CLI/server binary: starts the Ordo server from the `ordo` library and
//! opens the web UI in a browser tab. The desktop application lives in
//! `src-tauri/` and wraps the same server in a native window.

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use ordo::store::Store;
use ordo::{ai, api, seed};
use tokio::net::TcpListener;

struct Options {
    port: u16,
    data_path: PathBuf,
    open_browser: bool,
}

fn parse_options() -> Options {
    let mut port: u16 = env::var("ORDO_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000);
    let mut data_dir = env::var("ORDO_DATA_DIR").unwrap_or_else(|_| "data".to_string());
    let mut open_browser = env::var("ORDO_NO_OPEN").is_err();

    let args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--port" | "-p" => {
                i += 1;
                port = args.get(i).and_then(|p| p.parse().ok()).unwrap_or_else(|| {
                    eprintln!("--port expects a number");
                    std::process::exit(2);
                });
            }
            "--data" | "-d" => {
                i += 1;
                data_dir = args.get(i).cloned().unwrap_or_else(|| {
                    eprintln!("--data expects a directory");
                    std::process::exit(2);
                });
            }
            "--no-open" => open_browser = false,
            "--help" | "-h" => {
                println!(
                    "Ordo {}\n\nUSAGE:\n  ordo [--port N] [--data DIR] [--no-open]\n\nOPTIONS:\n  -p, --port N    Port to listen on (default 3000, env ORDO_PORT)\n  -d, --data DIR  Directory for ordo.json (default ./data, env ORDO_DATA_DIR)\n      --no-open   Do not open the browser on start (env ORDO_NO_OPEN)\n  -h, --help      Show this help",
                    env!("CARGO_PKG_VERSION")
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
    Options { port, data_path: PathBuf::from(data_dir).join("ordo.json"), open_browser }
}

async fn bind(preferred: u16) -> TcpListener {
    for offset in 0..10u16 {
        let port = preferred + offset;
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                if offset > 0 {
                    println!("Port {preferred} is busy, using {port} instead.");
                }
                return listener;
            }
            Err(_) => continue,
        }
    }
    eprintln!("Could not bind to any port between {preferred} and {}.", preferred + 9);
    std::process::exit(1);
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    println!("\nShutting down. Your data is saved in place.");
}

#[tokio::main]
async fn main() {
    let opts = parse_options();
    let today = chrono::Local::now().date_naive();
    let store = match Store::open(opts.data_path.clone(), || seed::demo_database(today)) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("Could not open {}: {e}", opts.data_path.display());
            std::process::exit(1);
        }
    };

    let ai = Arc::new(ai::Ai::new(ai::default_models_dir()));
    ai.spawn_idle_unloader(std::time::Duration::from_secs(10 * 60));
    let app = api::router(store.clone(), ai.clone());
    let listener = bind(opts.port).await;
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(opts.port);
    let url = format!("http://localhost:{port}");

    println!();
    println!("  Ordo v{}  -  priority-first task management", env!("CARGO_PKG_VERSION"));
    println!("  Open     {url}");
    println!("  Data     {}", store.path().display());
    println!("  Models   {}", ai.dir().display());
    println!("  Stop     Ctrl+C");
    println!();

    if opts.open_browser {
        let _ = open::that(&url);
    }

    if let Err(e) = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await {
        eprintln!("Server error: {e}");
    }
}
