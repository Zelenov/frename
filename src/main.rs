//! frename - File Renaming Utility for Windows
//! 
//! A GUI-based file renaming tool with preview and batch operations.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console in release mode

use iced::{Task, window, event};
use simplelog::{
    CombinedLogger, ColorChoice, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;

mod app;
mod features;

use app::FrenameApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize file + console logging
    let log_file = File::create("frename_debug.log")?;

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
    .window(window::Settings {
        size: iced::Size::new(800.0, 600.0),
        resizable: true,
        ..window::Settings::default()
    })
    .title(FrenameApp::title)
    .antialiasing(false)
    .subscription(|_| {
        event::listen_with(|event, _status, _id| {
            match event {
                iced::Event::Window(window::Event::FileDropped(path)) => {
                    Some(app::Message::DragDrop(
                        features::drag_drop::Message::FileDropped(path)
                    ))
                }
                _ => None,
            }
        })
    })
    .run()?)
}
