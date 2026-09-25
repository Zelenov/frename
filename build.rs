fn main() {
    #[cfg(windows)]
    windows_resources();
}

/// Embeds the icon and version into the Windows executable. `winres` is a Windows-only
/// build dependency, so this only compiles when building on Windows.
#[cfg(windows)]
fn windows_resources() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("frename-icon.ico");

    if let Ok(app_version) = std::env::var("APP_VERSION") {
        res.set("FileVersion", &app_version);
        res.set("ProductVersion", &app_version);
    }

    res.compile().unwrap();
}
