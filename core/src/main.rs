// Release builds are GUI programs (no console window); debug keeps the console for logs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use idm_lib::config::Config;
use idm_lib::service::{decode_idr_url, get_runtime, spawn, windows, QueueManager};
use idm_lib::ui::IcedApp;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;

fn main() -> anyhow::Result<()> {
    // 0. Initialize global async runtime & enter guard on main thread
    let rt = get_runtime();
    let _rt_guard = rt.enter();

    // 1. Parse command-line arguments (supports browser idr:// handoff)
    let args: Vec<String> = std::env::args().collect();
    let mut forwarded_url: Option<String> = None;

    if args.len() > 1 {
        let arg = &args[1];
        if let Some(decoded) = decode_idr_url(arg) {
            forwarded_url = Some(decoded);
        }
    }

    // 2. Windows Single-Instance Guard & IPC Handoff
    if windows::another_instance_is_running() {
        if let Some(ref url) = forwarded_url {
            println!("Another instance is running. Forwarding URL: {}", url);
            if windows::send_url_to_running_instance(url) {
                println!("Successfully forwarded URL to active instance.");
            } else {
                eprintln!("Failed to send URL to running instance via named pipe");
            }
        } else {
            println!("1IDM is already running.");
        }
        return Ok(());
    }

    // 3. Setup Logging
    let log_dir = Config::app_data_dir().join("logs");
    let _ = std::fs::create_dir_all(&log_dir);
    let file_appender = tracing_appender::rolling::never(&log_dir, "idr.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking).with_ansi(false));

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global tracing subscriber");

    info!("Starting 1nternet Downloader (1IDM) — Native Rust + Iced Architecture");

    // 4. Initialize Configuration & Persistence using database connection from utils
    let config = Config::load();
    let settings = config.get();

    let db_conn = idm_lib::utils::connect_default_sqlite()
        .expect("Failed to open SQLite connection from utils");
    idm_lib::model::database::init_schema(&db_conn)
        .expect("Failed to initialize database schema");
    let conn = Arc::new(std::sync::Mutex::new(db_conn));
    info!("Initialized SQLite database connection from utils and schema from model");

    // 5. Initialize Core Download Queue
    let queue_manager = QueueManager::new(conn, settings);
    info!("Initialized Core QueueManager and FIFO Scheduler");

    // 6. Start IPC Pipe Server to receive links from browser extensions
    let qm_ipc = Arc::clone(&queue_manager);
    windows::start_server(move |url: String| {
        let qm = Arc::clone(&qm_ipc);
        if let Some(clean_url) = decode_idr_url(&url) {
            info!("Received URL from IPC named pipe: {}", clean_url);
            spawn(async move {
                let _ = qm.add_download(&clean_url, None, true).await;
            });
        }
    });

    // 7. Handle initial forwarded URL if any
    if let Some(url) = forwarded_url {
        let qm_initial = Arc::clone(&queue_manager);
        spawn(async move {
            let _ = qm_initial.add_download(&url, None, true).await;
        });
    }

    // 8. Launch Native Iced Desktop UI
    info!("Launching Iced Native GUI");
    let qm_ui = Arc::clone(&queue_manager);

    iced::application(
        IcedApp::title,
        IcedApp::update,
        IcedApp::view,
    )
    .theme(IcedApp::theme)
    .subscription(IcedApp::subscription)
    .window(iced::window::Settings {
        // Close to IDM's default main window; fits 1366x768 and 1080p at 125% scaling.
        size: iced::Size::new(1000.0, 620.0),
        min_size: Some(iced::Size::new(820.0, 500.0)),
        position: iced::window::Position::Centered,
        // The app decides what closing means (hide to tray or exit); see ui::app.
        exit_on_close_request: false,
        icon: iced::window::icon::from_file_data(include_bytes!("../assets/icons/logo.png"), None).ok(),
        ..Default::default()
    })
    .run_with(move || IcedApp::new(qm_ui))?;

    info!("1IDM cleanly shut down");
    Ok(())
}
