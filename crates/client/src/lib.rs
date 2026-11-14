//! Runecord client library.
//!
//! The binary is a thin launcher. Keeping the application as a library makes
//! the cache, protocol and headless helpers usable from the workspace tools.

mod app;
pub mod bootstrap;
pub mod cache;
mod config;
#[cfg(feature = "embedded-server")]
mod embedded;
mod icons;
mod image_anim;
mod net;
mod theme;
mod ui;
mod voice;
mod wasapi;

/// Starts the Rune application.
pub fn run() {
    bootstrap::run();
}
