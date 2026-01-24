//! frename - File Renaming Utility for Windows
//! 
//! A GUI-based file renaming tool with preview and batch operations.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console in release mode

use iced::{Task, window, event};

mod app;
mod features;

use app::FrenameApp;

fn main() -> iced::Result {
    iced::application(
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
    .run()
}
