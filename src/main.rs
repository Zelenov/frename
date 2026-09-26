//! frename - File Renaming Utility for Windows
//!
//! A GUI-based file renaming tool with preview and batch operations.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console in release mode

use iced::window;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;

mod app;
mod crash_guard;
mod demo;
mod features;
mod self_test;
mod tag_colors;
mod theme;
mod widgets;

use app::FrenameApp;
use frename_core::{
    app_data_dir, install_file_tagger, AppDatabase, AppStateStore, InMemoryFileTagger,
    Initializable, LoggingAppStateStore, ProductionFileTagger,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install the file tagger backend before any file operations.
    //   Release build (default): ProductionFileTagger — actually renames files on disk.
    //   Debug build or --debug flag: InMemoryFileTagger — no disk changes (safe for testing).
    //   --production flag: force ProductionFileTagger even in debug builds.
    let args: Vec<String> = std::env::args().collect();
    let debug_flag = args.iter().any(|a| a == "--debug");
    let production_flag = args.iter().any(|a| a == "--production");
    // Debug mode: debug build (or --debug flag) without --production override.
    let is_debug_mode = !production_flag && (cfg!(debug_assertions) || debug_flag);

    // Demo mode works in a throwaway folder: the staged clips, and the database and log, so the
    // user's own are never touched. Set before anything reads the data folder, while the process
    // is still single-threaded.
    let demo = match demo::demo_args(&args) {
        None => None,
        Some(Err(e)) => return Err(e.into()),
        Some(Ok(demo_args)) => {
            let work = demo::WorkDir::new();
            std::env::set_var(frename_core::DATA_DIR_VAR, work.path().join("data"));
            Some((demo_args, work))
        }
    };

    // A demo shows the app as it really is (comments, screenshots) on its throwaway copies.
    if is_debug_mode && demo.is_none() {
        install_file_tagger(Box::new(InMemoryFileTagger::default()));
    } else {
        install_file_tagger(Box::new(ProductionFileTagger));
    }
    // File + console logging. The data folder holds the log and the database; inside an
    // AppImage it is not the executable's folder, and may not exist yet.
    let data_dir = app_data_dir();
    std::fs::create_dir_all(&data_dir)?;
    let log_file = File::create(frename_core::log_path())?;

    // Log only this app's crates. Dependencies are far noisier than they look: cosmic_text emits a
    // `relayout` record per text layout and naga one per shader-validation step, which measured at
    // ~7000 records/second during interaction — each one an unbuffered write on the UI thread.
    // The target filter drops them inside the logger, before the record is formatted or written.
    let log_config = ConfigBuilder::new().add_filter_allow_str("frename").build();
    // Full detail while developing; release keeps the log small and the UI thread free.
    let log_level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    CombinedLogger::init(vec![
        TermLogger::new(
            log_level,
            log_config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(log_level, log_config, log_file),
    ])?;

    log::info!("frename application started");

    // Check GStreamer availability
    match gstreamer::init() {
        Ok(()) => {
            let version = gstreamer::version();
            log::info!(
                "GStreamer initialized successfully: {}.{}.{}.{}",
                version.0,
                version.1,
                version.2,
                version.3
            );
        }
        Err(e) => {
            log::error!("GStreamer initialization failed: {e}");
            log::error!("Please install GStreamer. See GSTREAMER_SETUP.md for instructions.");
            return Err(format!(
                "GStreamer not found or failed to initialize: {e}. \
                 Please install GStreamer. See GSTREAMER_SETUP.md for instructions."
            )
            .into());
        }
    }

    if let Some(paths) = self_test_paths(&args) {
        std::process::exit(self_test::run(&paths));
    }

    // Initialize app database (migrations) before iced; decorator logs.
    let _ = LoggingAppStateStore::new(AppDatabase::new()).initialize();

    // The work folder goes when `demo_work` is dropped: on an early error return, or below.
    let (demo, demo_work) = match demo {
        Some((demo_args, work)) => (Some(demo::prepare(&demo_args, work.path())?), Some(work)),
        None => (None, None),
    };

    // Restore saved window geometry (size + position + maximized), or use defaults.
    let saved = AppDatabase::new().get_window_state();
    let window_size = saved
        .map(|g| iced::Size::new(g.width, g.height))
        .unwrap_or(iced::Size::new(1200.0, 600.0));
    let window_position = saved
        .map(|g| window::Position::Specific(iced::Point::new(g.x, g.y)))
        .unwrap_or(window::Position::Centered);
    let start_maximized = saved.map(|g| g.is_maximized).unwrap_or(false);

    // Load window icon from embedded .ico bytes.
    let icon_bytes = include_bytes!("../frename-icon.ico");
    let window_icon = image::load_from_memory(icon_bytes).ok().and_then(|img| {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        window::icon::from_rgba(rgba.into_raw(), w, h).ok()
    });

    // Install AFTER gstreamer::init() so our filter is registered last.
    crash_guard::install();

    let main_window = window::Settings {
        size: window_size,
        position: window_position,
        resizable: true,
        maximized: start_maximized,
        icon: window_icon.clone(),
        ..window::Settings::default()
    };

    // A daemon rather than an application: the app opens more than one window (settings), and
    // the app itself decides that closing the main window ends it.
    iced::daemon(
        move || {
            let (id, open) = window::open(main_window.clone());
            let app = FrenameApp::new(id, window_icon.clone()).with_demo(demo.clone());
            (app, open.discard())
        },
        FrenameApp::update,
        FrenameApp::view,
    )
    .theme(iced::Theme::Dark)
    .title(FrenameApp::title)
    .antialiasing(false)
    .subscription(FrenameApp::subscription)
    .run()?;

    // Only after the app is gone: it still saves the open file's folder while closing.
    drop(demo_work);
    std::process::exit(0);
}

/// The paths given with `--self-test` (every argument after it that is not a flag), when the
/// app was started to test its GStreamer.
fn self_test_paths(args: &[String]) -> Option<Vec<std::path::PathBuf>> {
    let at = args.iter().position(|a| a == "--self-test")?;
    Some(
        args[at + 1..]
            .iter()
            .filter(|a| !a.starts_with("--"))
            .map(Into::into)
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn no_self_test_flag_means_a_normal_start() {
        assert_eq!(self_test_paths(&args(&["frename", "--debug"])), None);
    }

    #[test]
    fn paths_after_the_flag_are_tested_and_flags_between_them_are_skipped() {
        assert_eq!(
            self_test_paths(&args(&[
                "frename",
                "--self-test",
                "--debug",
                "a clip.mp4",
                "folder"
            ])),
            Some(vec![PathBuf::from("a clip.mp4"), PathBuf::from("folder")])
        );
    }

    #[test]
    fn the_flag_alone_tests_nothing() {
        assert_eq!(
            self_test_paths(&args(&["frename", "--self-test"])),
            Some(Vec::new())
        );
    }
}
