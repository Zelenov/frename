fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("frename-icon.ico");

        if let Ok(app_version) = std::env::var("APP_VERSION") {
            res.set("FileVersion", &app_version);
            res.set("ProductVersion", &app_version);
        }

        res.compile().unwrap();
    }
}
