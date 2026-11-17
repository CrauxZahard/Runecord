//! Where Runecord keeps its per-user files.
//!
//! Windows uses `%APPDATA%\Runecord`; Linux uses `$XDG_DATA_HOME/runecord` or
//! `~/.local/share/runecord`. This is the directory holding the config, the
//! session tokens, the warm cache and the embedded server's data.
//!
//! `RUNECORD_DATA` overrides it, which is what the tests and a portable launch
//! use.

use std::path::PathBuf;

pub fn data_dir() -> Option<PathBuf> {
    if let Some(base) = std::env::var_os("RUNECORD_DATA") {
        return Some(PathBuf::from(base));
    }
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return Some(PathBuf::from(appdata).join("Runecord"));
    }
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        return Some(PathBuf::from(xdg).join("runecord"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        return Some(
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("runecord"),
        );
    }
    std::env::current_dir().ok().map(|dir| dir.join("Runecord"))
}
