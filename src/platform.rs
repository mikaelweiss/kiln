use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Platform {
    IOS,
    MacOS,
    Android,
    Web,
    Tauri,
    ChromeExtension,
    CLI,
}

impl Platform {
    pub const ALL: &[Platform] = &[
        Platform::IOS,
        Platform::MacOS,
        Platform::Android,
        Platform::Web,
        Platform::Tauri,
        Platform::ChromeExtension,
        Platform::CLI,
    ];
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::IOS => write!(f, "iOS (Swift)"),
            Platform::MacOS => write!(f, "macOS (Swift)"),
            Platform::Android => write!(f, "Android (Kotlin)"),
            Platform::Web => write!(f, "Web"),
            Platform::Tauri => write!(f, "Tauri"),
            Platform::ChromeExtension => write!(f, "Chrome Extension"),
            Platform::CLI => write!(f, "CLI"),
        }
    }
}
