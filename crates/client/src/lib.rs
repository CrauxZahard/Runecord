//! Runecord client library.
//!
//! The binary is a thin launcher. Keeping the application as a library makes
//! the cache, protocol and headless helpers usable from the workspace tools.
//!
//! This is the desktop client (Windows and Linux). A browser target is no
//! longer built from this repository.

mod app;
pub mod bootstrap;
pub mod cache;
mod config;
#[cfg(feature = "embedded-server")]
mod embedded;
mod files;
mod i18n;
mod icons;
mod image_anim;
mod net;
mod overlay;
mod paths;
mod presence_scan;
mod server_info;
mod theme;
mod ui;
mod update;
mod voice;
#[cfg(windows)]
mod wasapi;

/// Starts the Rune application.
pub fn run() {
    bootstrap::run();
}
