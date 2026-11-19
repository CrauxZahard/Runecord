//! Runecord client library.
//!
//! The binary is a thin launcher. Keeping the application as a library makes
//! the cache, protocol and headless helpers usable from the workspace tools.

pub mod aiedit;
mod app;
pub mod bootstrap;
pub mod cache;
mod config;
#[cfg(feature = "embedded-server")]
mod embedded;
mod files;
mod i18n;
mod image_anim;
mod net;
mod overlay;
mod paths;
mod presence_scan;
mod server_info;
mod ui;

// The view tokens live in their own crate so the palette, type scale and icon
// catalogue can be edited without pulling in the network or session code.
// Re-exported at the root, `crate::theme` and `crate::icons` keep working.
pub use runecord_ui::{icons, theme};
mod update;
mod voice;
#[cfg(windows)]
mod wasapi;

/// Starts the Rune application.
pub fn run() {
    bootstrap::run();
}
