//! Runs the release build and reports its lines while it runs.
//!
//! The build is a child process, never a library call: a compile error here is
//! data to show the user, not a panic in the app. Compiling the whole client
//! crate is expected (1-3 minutes warm); the UI crate split is what later makes
//! it shorter.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

const TAIL: usize = 300;

/// Builds the release binary and returns its path.
pub fn release(root: &Path, target: &Path) -> Result<PathBuf, String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    if !Command::new(&cargo)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
    {
        return Err(format!(
            "The Rust toolchain is not available: `{cargo}` could not be run. \
             The editor needs the one-time toolchain download."
        ));
    }
    super::log(&format!("cargo build --release in {}", root.display()));
    let mut child = Command::new(&cargo)
        .current_dir(root)
        .args([
            "build",
            "--release",
            "-p",
            "runecord",
            "--features",
            "embedded-server",
            "--message-format",
            "short",
        ])
        .env("CARGO_TARGET_DIR", target)
        .env("CARGO_TERM_COLOR", "never")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Could not start the build: {error}"))?;

    let tail: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let mut readers = Vec::new();
    if let Some(out) = child.stdout.take() {
        readers.push(pump(out, tail.clone()));
    }
    if let Some(err) = child.stderr.take() {
        readers.push(pump(err, tail.clone()));
    }
    let status = child
        .wait()
        .map_err(|error| format!("Could not wait for the build: {error}"))?;
    for reader in readers {
        let _ = reader.join();
    }
    if !status.success() {
        let lines = tail.lock().map(|tail| tail.clone()).unwrap_or_default();
        let shown = lines
            .iter()
            .rev()
            .take(40)
            .rev()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!(
            "The build failed ({}).\n{shown}",
            status.code().unwrap_or(-1)
        ));
    }
    let exe = target.join("release").join(if cfg!(windows) {
        "runecord.exe"
    } else {
        "runecord"
    });
    if !exe.exists() {
        return Err(format!(
            "The build finished but {} is missing.",
            exe.display()
        ));
    }
    Ok(exe)
}

fn pump<R: std::io::Read + Send + 'static>(
    pipe: R,
    tail: Arc<Mutex<Vec<String>>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for line in BufReader::new(pipe).lines().map_while(Result::ok) {
            let line = line.trim_end();
            if line.is_empty() {
                continue;
            }
            super::log(line);
            if let Ok(mut tail) = tail.lock() {
                tail.push(line.to_string());
                if tail.len() > TAIL {
                    tail.remove(0);
                }
            }
        }
    })
}
