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
mod features;
mod theme;
mod widgets;

use app::FrenameApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize file + console logging (log file next to the executable)
    let log_path = std::env::current_exe()?
        .parent()
        .expect("executable must have a parent directory")
        .join("frename_debug.log");
    let log_file = File::create(log_path)?;

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
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

    Ok(iced::application(
        || (FrenameApp::default(), Task::none()),
        FrenameApp::update,
        FrenameApp::view,
    )
    .theme(iced::Theme::Dark)
    .centered()
    .window(window::Settings {
        size: iced::Size::new(800.0, 600.0),
        resizable: true,
        ..window::Settings::default()
    })
    .title(FrenameApp::title)
    .antialiasing(false)
    .subscription(FrenameApp::subscription)
    .run()?)
}
