//! Marks release builds, and on Windows embeds the app icon and version data.
//!
//! `windows_subsystem = "windows"` cannot key off `debug_assertions`: the
//! workspace's release profile keeps debug assertions on for fast GPU startup,
//! so that would leave release builds as console apps and Windows would attach
//! `conhost.exe` to every launch. The profile name is the fact that matters.
//!
//! The executable resources come from windres (MinGW), so a machine without
//! the Windows SDK still produces an icon and a version tab. If windres is not
//! found the build continues and only the resources are missing.

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-check-cfg=cfg(release_build)");
    if std::env::var("PROFILE").as_deref() == Ok("release") {
        println!("cargo:rustc-cfg=release_build");
    }

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_windows_resources();
    }
}

fn embed_windows_resources() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let icon = manifest
        .join("..")
        .join("..")
        .join("assets")
        .join("icon.ico");
    println!("cargo:rerun-if-changed={}", icon.display());
    if !icon.exists() {
        println!("cargo:warning=assets/icon.ico is missing; run scripts/make-icon.py");
        return;
    }
    let Some(windres) = find_windres() else {
        println!("cargo:warning=windres not found; building without an exe icon or version");
        return;
    };

    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let numbers: Vec<u64> = version
        .split(['-', '+'])
        .next()
        .unwrap_or("0.0.0")
        .split('.')
        .map(|part| part.parse().unwrap_or(0))
        .collect();
    let tuple = format!(
        "{},{},{},0",
        numbers.first().copied().unwrap_or(0),
        numbers.get(1).copied().unwrap_or(0),
        numbers.get(2).copied().unwrap_or(0)
    );
    let icon_rc = icon.to_string_lossy().replace('\\', "/");

    let rc = format!(
        r#"1 ICON "{icon_rc}"
1 VERSIONINFO
FILEVERSION {tuple}
PRODUCTVERSION {tuple}
FILEOS 0x40004
FILETYPE 0x1
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "CompanyName", "Runecord\0"
      VALUE "FileDescription", "Runecord\0"
      VALUE "FileVersion", "{version}\0"
      VALUE "InternalName", "Runecord\0"
      VALUE "LegalCopyright", "MIT licensed\0"
      VALUE "OriginalFilename", "Runecord.exe\0"
      VALUE "ProductName", "Runecord\0"
      VALUE "ProductVersion", "{version}\0"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#
    );

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("out dir"));
    let rc_path = out.join("runecord.rc");
    let obj_path = out.join("runecord.res.o");
    if std::fs::write(&rc_path, rc).is_err() {
        println!("cargo:warning=could not write the resource script");
        return;
    }
    let status = Command::new(&windres)
        .arg("--input")
        .arg(&rc_path)
        .arg("--output")
        .arg(&obj_path)
        .arg("--output-format=coff")
        .status();
    match status {
        Ok(status) if status.success() && obj_path.exists() => {
            println!("cargo:rustc-link-arg={}", obj_path.display());
        }
        _ => println!("cargo:warning=windres failed; building without resources"),
    }
}

/// `windres` from MinGW on PATH, or the copy bundled in D:\tools\winlibs.
fn find_windres() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("WINDRES") {
        return Some(PathBuf::from(path));
    }
    let names = ["windres.exe", "windres"];
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        for name in names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let known = Path::new(r"D:\tools\winlibs\mingw64\bin\windres.exe");
    known.is_file().then(|| known.to_path_buf())
}
