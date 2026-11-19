//! Install layout, staged versions and the restart supervisor.
//!
//! ```text
//! %APPDATA%\Runecord\aiedit\
//!   versions\base\runecord.exe    the binary that was running when the first edit was staged
//!   versions\ui-<stamp>\...       a build the model produced
//!   backup\<stamp>\...            the source files as they were before that edit
//!   state.json                    current, last_good, pending
//! ```
//!
//! The app cannot replace its own image on Windows, so "restart" hands the swap
//! to a short-lived copy of the same executable: it starts the staged binary,
//! waits for that binary's ready marker, and falls back to the previous one when
//! the marker never appears. Reset is a button inside the app, which is exactly
//! what is missing when a build will not start, so the fallback cannot be
//! optional.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const SUPERVISE: &str = "--aiedit-supervise";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub current: String,
    #[serde(default)]
    pub last_good: String,
    #[serde(default)]
    pub pending: Option<String>,
}

/// Per-user data for this feature.
pub fn root() -> Result<PathBuf, String> {
    crate::paths::data_dir()
        .map(|dir| dir.join("aiedit"))
        .ok_or_else(|| "Runecord has no data folder, so the editor cannot store anything.".into())
}

/// The source tree the model may edit.
///
/// A development checkout is found by walking up from the executable to the
/// workspace manifest; a packaged install uses the copy the feature downloaded.
pub fn source_root() -> Result<PathBuf, String> {
    if let Ok(value) = std::env::var("RUNECORD_UI_SRC") {
        let path = PathBuf::from(value);
        if path.join("Cargo.toml").exists() {
            return Ok(path);
        }
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        for candidate in dir.ancestors().take(6) {
            if candidate.join("crates/client/Cargo.toml").exists()
                && candidate.join("Cargo.toml").exists()
            {
                return Ok(candidate.to_path_buf());
            }
        }
    }
    let packaged = root()?.join("src");
    if packaged.join("Cargo.toml").exists() {
        return Ok(packaged);
    }
    Err(
        "The UI source isn't installed. The editor needs the one-time source download before it can run."
            .into(),
    )
}

/// Where the compiler writes. A checkout reuses its warm `target/`; a packaged
/// install uses the prebuilt tree that came with the download.
pub fn target_dir() -> Result<PathBuf, String> {
    if let Ok(value) = std::env::var("RUNECORD_UI_TARGET") {
        return Ok(PathBuf::from(value));
    }
    let source = source_root()?;
    let local = source.join("target");
    if local.exists() {
        return Ok(local);
    }
    Ok(root()?.join("target"))
}

pub fn state() -> State {
    let Ok(path) = root().map(|root| root.join("state.json")) else {
        return State::default();
    };
    std::fs::read(&path)
        .ok()
        .and_then(|bytes| parse_json(&bytes))
        .unwrap_or_default()
}

fn save_state(state: &State) -> Result<(), String> {
    let dir = root()?;
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("Could not create {}: {error}", dir.display()))?;
    let bytes = serde_json::to_vec_pretty(state).map_err(|error| error.to_string())?;
    std::fs::write(dir.join("state.json"), bytes)
        .map_err(|error| format!("Could not record the version state: {error}"))
}

pub fn backup_id() -> String {
    stamp()
}

fn stamp() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or(0);
    millis.to_string()
}

/// What a build was made for, kept beside it so the history reads as goals
/// rather than timestamps.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VersionMeta {
    #[serde(default)]
    pub goal: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub created_ms: i64,
    /// The source backup this build was made from.
    #[serde(default)]
    pub backup: String,
}

/// One entry in the version list.
#[derive(Clone, Debug)]
pub struct Version {
    pub name: String,
    pub goal: String,
    pub when: String,
    pub current: bool,
    pub pending: bool,
}

/// Copies a built binary into `versions/<name>`, records what it was for, and
/// marks it pending.
pub fn stage(exe: &Path, goal: &str, files: &[String], backup: &str) -> Result<String, String> {
    let dir = root()?;
    ensure_base(&dir)?;
    let name = format!("ui-{}", stamp());
    let version = dir.join("versions").join(&name);
    std::fs::create_dir_all(&version)
        .map_err(|error| format!("Could not create {}: {error}", version.display()))?;
    let target = version.join(exe_name());
    std::fs::copy(exe, &target)
        .map_err(|error| format!("Could not copy the new build: {error}"))?;
    copy_assets(&version);
    let meta = VersionMeta {
        goal: goal.to_string(),
        files: files.to_vec(),
        created_ms: now_ms(),
        backup: backup.to_string(),
    };
    let bytes = serde_json::to_vec_pretty(&meta).unwrap_or_default();
    let _ = std::fs::write(version.join("meta.json"), bytes);
    let mut state = state();
    state.pending = Some(name.clone());
    save_state(&state)?;
    prune_versions(5);
    Ok(name)
}

/// Every staged build, newest first, with the original last.
pub fn versions() -> Vec<Version> {
    let Ok(dir) = root() else {
        return Vec::new();
    };
    let state = state();
    let mut out: Vec<(u64, Version)> = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir.join("versions")) else {
        return Vec::new();
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !entry.path().join(exe_name()).exists() {
            continue;
        }
        let meta: VersionMeta = std::fs::read(entry.path().join("meta.json"))
            .ok()
            .and_then(|bytes| parse_json(&bytes))
            .unwrap_or_default();
        let stamp = name
            .strip_prefix("ui-")
            .and_then(|rest| rest.parse::<u64>().ok())
            .unwrap_or(0);
        let goal = if name == "base" {
            "Original build".to_string()
        } else if meta.goal.trim().is_empty() {
            "UI edit".to_string()
        } else {
            one_line(meta.goal.trim(), 96)
        };
        out.push((
            stamp,
            Version {
                current: state.current == name,
                pending: state.pending.as_deref() == Some(name.as_str()),
                when: if name == "base" {
                    String::new()
                } else {
                    when_label(meta.created_ms)
                },
                goal,
                name,
            },
        ));
    }
    out.sort_by_key(|a| std::cmp::Reverse(a.0));
    out.into_iter().map(|(_, version)| version).collect()
}

/// Keeps the newest `keep` builds, never the ones a restart can fall back to.
fn prune_versions(keep: usize) {
    let Ok(dir) = root() else {
        return;
    };
    let state = state();
    let protected = [state.current, state.last_good, "base".to_string()];
    let Ok(entries) = std::fs::read_dir(dir.join("versions")) else {
        return;
    };
    let mut stamps: Vec<(u64, PathBuf)> = entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            let stamp = name
                .strip_prefix("ui-")
                .and_then(|rest| rest.parse::<u64>().ok())?;
            if protected.iter().any(|kept| kept == &name)
                || state.pending.as_deref() == Some(name.as_str())
            {
                return None;
            }
            Some((stamp, entry.path()))
        })
        .collect();
    stamps.sort_by_key(|a| std::cmp::Reverse(a.0));
    for (_, path) in stamps.into_iter().skip(keep) {
        let _ = std::fs::remove_dir_all(path);
    }
}

/// Collapses a goal to one display line. Stored goals are whole prompts, and a
/// row that grows to several lines pushes its buttons out of alignment.
fn one_line(text: &str, limit: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= limit {
        return flat;
    }
    let cut: String = flat.chars().take(limit).collect();
    format!("{cut}…")
}

fn when_label(ms: i64) -> String {
    if ms <= 0 {
        return String::new();
    }
    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ms)
        .map(|time| time.format("%d %b %H:%M").to_string())
        .unwrap_or_default()
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or(0)
}

/// The binary that was running before the first staged edit.
fn ensure_base(dir: &Path) -> Result<(), String> {
    let base = dir.join("versions").join("base");
    if base.join(exe_name()).exists() {
        return Ok(());
    }
    std::fs::create_dir_all(&base)
        .map_err(|error| format!("Could not create {}: {error}", base.display()))?;
    let current = std::env::current_exe()
        .map_err(|error| format!("Could not find the running executable: {error}"))?;
    std::fs::copy(&current, base.join(exe_name()))
        .map_err(|error| format!("Could not keep a copy of the current build: {error}"))?;
    copy_assets(&base);
    let mut state = state();
    if state.last_good.is_empty() {
        state.last_good = "base".into();
    }
    if state.current.is_empty() {
        state.current = "base".into();
    }
    save_state(&state)
}

/// The icon files an executable expects beside it, when they can be found.
fn copy_assets(version: &Path) {
    let mut sources: Vec<PathBuf> = Vec::new();
    if let Ok(current) = std::env::current_exe()
        && let Some(dir) = current.parent()
    {
        sources.push(dir.join("assets"));
    }
    if let Ok(root) = source_root() {
        sources.push(root.join("assets"));
        sources.push(root.join("dist").join("assets"));
    }
    let target = version.join("assets");
    for dir in sources {
        for file in ["icon.png", "icon.ico"] {
            let source = dir.join(file);
            if source.exists() && !target.join(file).exists() {
                let _ = std::fs::create_dir_all(&target);
                let _ = std::fs::copy(source, target.join(file));
            }
        }
    }
}

pub fn version_exe(name: &str) -> Result<PathBuf, String> {
    let path = root()?.join("versions").join(name).join(exe_name());
    if !path.exists() {
        return Err(format!("The build named {name} is missing."));
    }
    Ok(path)
}

/// Starts the staged build through a supervisor and returns once it is spawned.
pub fn restart_staged() -> Result<(), String> {
    let state = state();
    let pending = state
        .pending
        .clone()
        .ok_or("There is no staged build to restart into.")?;
    let child = version_exe(&pending)?;
    let fallback = if state.last_good.is_empty() {
        std::env::current_exe().map_err(|error| error.to_string())?
    } else {
        version_exe(&state.last_good).unwrap_or_else(|_| {
            std::env::current_exe().unwrap_or_else(|_| PathBuf::from(exe_name()))
        })
    };
    supervise_spawn(&child, &fallback)
}

/// Starts the last good build, used by reset.
pub fn restart_last_good() -> Result<(), String> {
    let state = state();
    let name = if state.last_good.is_empty() {
        "base".to_string()
    } else {
        state.last_good
    };
    let child = version_exe(&name)?;
    supervise_spawn(&child, &child)
}

/// Switches the next restart to one of the staged builds.
///
/// Its source backup is restored first, so the files on disk match the build
/// that is about to run; otherwise the next edit would start from a newer tree
/// than the UI the user is looking at. Choosing the original build restores
/// every backup.
pub fn restart_into(name: &str) -> Result<(), String> {
    let child = version_exe(name)?;
    if name == "base" {
        let _ = restore_all_backups();
    } else if let Some(meta) = version_meta(name)
        && !meta.backup.is_empty()
    {
        let _ = restore_backup(&meta.backup);
    }
    let mut state = state();
    state.pending = Some(name.to_string());
    save_state(&state)?;
    let fallback = version_exe(&state.current)
        .or_else(|_| version_exe(&state.last_good))
        .or_else(|_| std::env::current_exe().map_err(|error| error.to_string()))?;
    supervise_spawn(&child, &fallback)
}

fn version_meta(name: &str) -> Option<VersionMeta> {
    let path = root().ok()?.join("versions").join(name).join("meta.json");
    let bytes = std::fs::read(path).ok()?;
    parse_json(&bytes)
}

fn supervise_spawn(child: &Path, fallback: &Path) -> Result<(), String> {
    let dir = root()?;
    let ready = dir.join(format!("ready-{}.flag", stamp()));
    let _ = std::fs::remove_file(&ready);
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut command = std::process::Command::new(exe);
    command.arg(SUPERVISE).arg(child).arg(&ready).arg(fallback);
    for arg in std::env::args().skip(1) {
        if !arg.starts_with("--aiedit") {
            command.arg(arg);
        }
    }
    command
        .spawn()
        .map_err(|error| format!("Could not start the restart helper: {error}"))?;
    Ok(())
}

/// Handles the supervisor invocation. Returns true when this process was the
/// helper and has already exited.
pub fn supervise_from_args() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some(SUPERVISE) {
        return false;
    }
    let child = args.get(2).map(String::as_str).unwrap_or_default();
    let ready = args.get(3).map(String::as_str).unwrap_or_default();
    let fallback = args.get(4).map(String::as_str).unwrap_or_default();
    let extra: Vec<String> = args.iter().skip(5).cloned().collect();
    supervise(child, ready, fallback, &extra);
    true
}

fn supervise(child: &str, ready: &str, fallback: &str, extra: &[String]) {
    if child.is_empty() || ready.is_empty() {
        return;
    }
    let _ = std::fs::remove_file(ready);
    let mut spawned = std::process::Command::new(child)
        .env("RUNECORD_READY_FILE", ready)
        .args(extra)
        .spawn()
        .ok();
    let deadline = Instant::now() + Duration::from_secs(90);
    let mut started = false;
    while Instant::now() < deadline {
        if Path::new(ready).exists() {
            started = true;
            break;
        }
        if let Some(waiting) = spawned.as_mut() {
            match waiting.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {}
                Err(_) => break,
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    let _ = std::fs::remove_file(ready);
    if started {
        if let Some(state) = finish_pending() {
            let _ = save_state(&state);
        }
        if let Ok(dir) = root() {
            let _ = std::fs::remove_file(dir.join("last-failure.txt"));
        }
        return;
    }
    if let Some(mut waiting) = spawned.take() {
        let _ = waiting.kill();
    }
    record_failure("The new UI build did not start, so Runecord reopened the previous one.");
    if !fallback.is_empty() && fallback != child {
        let _ = std::process::Command::new(fallback).args(extra).spawn();
    }
}

/// Notes that a staged build never started, so the next launch can explain it.
pub fn record_failure(note: &str) {
    let mut state = state();
    state.pending = None;
    let _ = save_state(&state);
    if let Ok(dir) = root() {
        let _ = std::fs::write(dir.join("last-failure.txt"), note);
    }
}

/// Reads and clears the note from the previous failed start.
pub fn take_failure() -> Option<String> {
    let path = root().ok()?.join("last-failure.txt");
    let note = std::fs::read_to_string(&path).ok()?;
    let _ = std::fs::remove_file(path);
    Some(note)
}

/// Promotes the pending build to current once it has proved it can start.
fn finish_pending() -> Option<State> {
    let mut state = state();
    let pending = state.pending.take()?;
    state.last_good = state.current.clone();
    state.current = pending;
    Some(state)
}

/// Restores the source files of the most recent edit, for "Undo last change".
pub fn restore_latest_backup() -> Result<usize, String> {
    let latest = backup_stamps()?
        .pop()
        .ok_or_else(|| "There is no UI edit to undo yet.".to_string())?;
    restore_backup(&latest)
}

/// Restores every edit, newest first, so the oldest backup — the original files
/// — ends up in place. This is what "Reset to default UI" needs: undoing only
/// the last edit would leave the first one behind.
pub fn restore_all_backups() -> Result<usize, String> {
    let mut stamps = backup_stamps()?;
    if stamps.is_empty() {
        return Err("There is no UI edit to undo yet.".into());
    }
    stamps.reverse();
    let mut restored = 0;
    for id in stamps {
        restored += restore_backup(&id)?;
    }
    Ok(restored)
}

fn backup_stamps() -> Result<Vec<String>, String> {
    let dir = root()?.join("backup");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let mut stamps: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
        .collect();
    stamps.sort();
    Ok(stamps)
}

/// Restores one recorded backup. A file that did not exist before is removed.
pub fn restore_backup(id: &str) -> Result<usize, String> {
    let dir = root()?.join("backup").join(id);
    let manifest = std::fs::read(dir.join("manifest.json"))
        .map_err(|error| format!("Could not read the backup {id}: {error}"))?;
    let record: serde_json::Value =
        serde_json::from_slice(&manifest).map_err(|error| error.to_string())?;
    let root = source_root()?;
    let mut restored = 0;
    if let Some(files) = record["files"].as_array() {
        for name in files.iter().filter_map(|value| value.as_str()) {
            let saved = dir.join(name);
            let target = root.join(name);
            if saved.exists() {
                if let Some(parent) = target.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                std::fs::copy(&saved, &target)
                    .map_err(|error| format!("Could not restore {name}: {error}"))?;
                restored += 1;
            } else {
                let _ = std::fs::remove_file(&target);
                restored += 1;
            }
        }
    }
    Ok(restored)
}

fn exe_name() -> &'static str {
    if cfg!(windows) {
        "runecord.exe"
    } else {
        "runecord"
    }
}

/// Tolerant JSON read. A hand-edited file may carry a UTF-8 BOM, which serde
/// rejects; ignoring the whole file would also drop the version's backup link.
fn parse_json<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    let text = String::from_utf8_lossy(bytes);
    serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()
}

/// The endpoint and model the user last used. Not secret, so it is a plain file
/// next to the key.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Provider {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub model: String,
}

pub fn save_provider(base_url: &str, model: &str) -> Result<(), String> {
    let dir = root()?;
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("Could not create {}: {error}", dir.display()))?;
    let provider = Provider {
        base_url: base_url.trim().to_string(),
        model: model.trim().to_string(),
    };
    let bytes = serde_json::to_vec_pretty(&provider).map_err(|error| error.to_string())?;
    std::fs::write(dir.join("provider.json"), bytes)
        .map_err(|error| format!("Could not save the provider settings: {error}"))
}

pub fn load_provider() -> Option<Provider> {
    let path = root().ok()?.join("provider.json");
    let bytes = std::fs::read(path).ok()?;
    parse_json(&bytes)
}
