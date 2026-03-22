//! frename - File Renaming Utility for Windows
//!
//! A GUI-based file renaming tool with preview and batch operations.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console in release mode

use iced::{Task, window};
use simplelog::{
    CombinedLogger, ColorChoice, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;

mod app;
mod crash_guard;
mod features;
mod tag_colors;
mod theme;
mod widgets;

use app::FrenameApp;
use frename_core::{
    install_file_tagger, InMemoryFileTagger, ProductionFileTagger,
    AppDatabase, AppStateStore, Initializable, LoggingAppStateStore,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install the file tagger backend before any file operations.
    //   Release build (default): ProductionFileTagger — actually renames files on disk.
    //   Debug build or --debug flag: InMemoryFileTagger — no disk changes (safe for testing).
    //   --production flag: force ProductionFileTagger even in debug builds.
    let args: Vec<String> = std::env::args().collect();
    let debug_flag = args.iter().any(|a| a == "--debug");
    let production_flag = args.iter().any(|a| a == "--production");
    if production_flag || (!cfg!(debug_assertions) && !debug_flag) {
        install_file_tagger(Box::new(ProductionFileTagger));
    } else {
        install_file_tagger(Box::new(InMemoryFileTagger::default()));
    }
    // Initialize file + console logging (log file next to the executable)
    let log_path = std::env::current_exe()?
        .parent()
        .expect("executable must have a parent directory")
        .join("frename_debug.log");
    let log_file = File::create(log_path)?;

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            log_file,
        ),
    ])?;

    log::info!("frename application started");

    // Check GStreamer availability
    match gstreamer::init() {
        Ok(()) => {
            let version = gstreamer::version();
            log::info!(
                "GStreamer initialized successfully: {}.{}.{}.{}",
                version.0, version.1, version.2, version.3
            );
        }
        Err(e) => {
            log::error!("GStreamer initialization failed: {e}");
            log::error!("Please install GStreamer. See GSTREAMER_SETUP.md for instructions.");
            return Err(format!(
                "GStreamer not found or failed to initialize: {e}. \
                 Please install GStreamer. See GSTREAMER_SETUP.md for instructions."
            ).into());
        }
    }

    // Initialize app database (migrations) before iced; decorator logs.
    let _ = LoggingAppStateStore::new(AppDatabase::new()).initialize();

    // Restore saved window geometry (size + position), or use defaults.
    let saved = AppDatabase::new().get_window_state();
    let window_size = saved.map(|g| iced::Size::new(g.width, g.height))
        .unwrap_or(iced::Size::new(1200.0, 600.0));
    let window_position = saved.map(|g| window::Position::Specific(iced::Point::new(g.x, g.y)))
        .unwrap_or(window::Position::Centered);

    // Load window icon from embedded .ico bytes.
    let icon_bytes = include_bytes!("../frename-icon.ico");
    let window_icon = image::load_from_memory(icon_bytes)
        .ok()
        .and_then(|img| {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            window::icon::from_rgba(rgba.into_raw(), w, h).ok()
        });

    // Install AFTER gstreamer::init() so our filter is registered last.
    crash_guard::install();

    iced::application(
        || (FrenameApp::new(), Task::none()),
        FrenameApp::update,
        FrenameApp::view,
    )
    .theme(iced::Theme::Dark)
    .window(window::Settings {
        size: window_size,
        position: window_position,
        resizable: true,
        icon: window_icon,
        ..window::Settings::default()
    })
    .title(FrenameApp::title)
    .antialiasing(false)
    .subscription(FrenameApp::subscription)
    .run()?;

    std::process::exit(0);
}
