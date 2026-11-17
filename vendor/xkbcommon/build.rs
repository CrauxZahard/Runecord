//! Keeps native Linux builds linking the system libxkbcommon.
//!
//! The crate's `#[link(name = "xkbcommon")]` was removed because zig's lld on a
//! Windows host cannot open a library found through `-L` (it builds an absolute
//! Windows path and fails). Cross-builds pass the vendored `.so` as a direct
//! linker input instead (`scripts/build-linux.ps1`); a build on Linux itself
//! uses the distribution's library through the normal `-lxkbcommon`.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("linux") {
        return;
    }
    let host = std::env::var("HOST").unwrap_or_default();
    let target = std::env::var("TARGET").unwrap_or_default();
    if host == target {
        println!("cargo:rustc-link-lib=dylib=xkbcommon");
    }
}
