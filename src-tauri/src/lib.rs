//! Ordo desktop: runs the Ordo server in-process on a random localhost port and
//! opens the web UI in a native WebView2 window.

use std::sync::Arc;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Per-user data directory, e.g. %APPDATA%\com.ordo.desktop\ordo.json
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let store = Arc::new(ordo::store::Store::open(data_dir.join("ordo.json"), || {
                ordo::seed::demo_database(chrono::Local::now().date_naive())
            })?);
            let ai = Arc::new(ordo::ai::Ai::new(ordo::ai::default_models_dir()));
            ai.spawn_idle_unloader(std::time::Duration::from_secs(10 * 60));
            let router = ordo::api::router(store, ai);

            // Bind to a free port on loopback only; nothing is reachable from the network.
            let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
            let port = listener.local_addr()?.port();
            listener.set_nonblocking(true)?;
            tauri::async_runtime::spawn(async move {
                let listener = tokio::net::TcpListener::from_std(listener).expect("tokio listener");
                if let Err(e) = axum::serve(listener, router).await {
                    eprintln!("Ordo server error: {e}");
                }
            });

            let url: tauri::Url = format!("http://127.0.0.1:{port}/").parse()?;
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("Ordo")
                .inner_size(1440.0, 900.0)
                .min_inner_size(900.0, 600.0)
                .center()
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Ordo");
}
