//! Install layout, staged versions and the restart supervisor.
//!
//! ```text
//! %APPDATA%\Runecord\aiedit\
//!   versions\ui-<stamp>\...     current, last-good and pending binaries only
//!   source\objects\<hash>       content-addressed UI source files
//!   source\versions\<name>\     full-tree manifests, kept after binaries are pruned
//!   backup\<stamp>\...          per-edit rollback files
//!   state.json                  current, last_good, pending, base_exe
//! ```
//!
//! A version is a source snapshot plus an optional binary. Only the current and
//! last-good binaries stay on disk; older versions are rebuilt from their
//! source manifest when selected. The installed app can act as a launcher: it
//! reads `state.json`, starts the current version, and falls back to the
//! last-good build or its own built-in UI.
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
    /// The installed app that serves as the built-in fallback and launcher.
    #[serde(default)]
    pub base_exe: String,
    /// The app version the user chose to keep an older custom UI for. A new
    /// app version warns again.
    #[serde(default)]
    pub kept_app_version: String,
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
    if let Ok(value) = std::env::var("RUNECORD_UI_SRC")
        && let Some(path) = workspace_at(Path::new(&value))
    {
        return Ok(path);
    }
    // An installed app keeps the build tree in <install>\src\Runecord. A
    // version exe runs from AppData, so ask the base executable where it lives.
    if let Some(base) = base_exe()
        && let Some(dir) = base.parent()
        && let Some(path) = workspace_at(&dir.join("src"))
    {
        return Ok(path);
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
    if let Ok(dir) = root()
        && let Some(path) = workspace_at(&dir.join("src"))
    {
        return Ok(path);
    }
    Err(
        "The UI source isn't installed. Reinstall Runecord with its source bundle, or set RUNECORD_UI_SRC."
            .into(),
    )
}

/// Accepts either a workspace folder or a bundle folder that contains
/// `Runecord\Cargo.toml`.
fn workspace_at(dir: &Path) -> Option<PathBuf> {
    if dir.join("Cargo.toml").is_file() {
        return Some(dir.to_path_buf());
    }
    let nested = dir.join("Runecord");
    if nested.join("Cargo.toml").is_file() {
        return Some(nested);
    }
    None
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
    /// The pre-edit backup this build was made from (legacy undo).
    #[serde(default)]
    pub backup: String,
    /// The source snapshot that can rebuild this version.
    #[serde(default)]
    pub source: String,
    /// The app version this UI was built against.
    #[serde(default)]
    pub app_version: String,
    /// True when the binary was produced by a later rebuild, not the first edit.
    #[serde(default)]
    pub rebuilt: bool,
}

/// One entry in the version list.
#[derive(Clone, Debug)]
pub struct Version {
    pub name: String,
    pub goal: String,
    pub when: String,
    /// The app version this UI was built against.
    pub app_version: String,
    pub current: bool,
    pub pending: bool,
    /// A runnable binary is present.
    pub binary: bool,
    /// A full source snapshot is present, so the version can be rebuilt.
    pub source: bool,
    pub rebuilt: bool,
}

/// Copies a built binary into `versions/<name>`, records the full source
/// snapshot, and marks it pending.
pub fn stage(exe: &Path, goal: &str, files: &[String], backup: &str) -> Result<String, String> {
    let dir = root()?;
    record_base();
    let name = format!("ui-{}", stamp());
    let version = dir.join("versions").join(&name);
    std::fs::create_dir_all(&version)
        .map_err(|error| format!("Could not create {}: {error}", version.display()))?;
    let target = version.join(exe_name());
    std::fs::copy(exe, &target)
        .map_err(|error| format!("Could not copy the new build: {error}"))?;
    copy_assets(&version);
    if let Err(error) = super::source::snapshot(&name, goal) {
        let _ = std::fs::remove_dir_all(&version);
        return Err(error);
    }
    let meta = VersionMeta {
        goal: goal.to_string(),
        files: files.to_vec(),
        created_ms: now_ms(),
        backup: backup.to_string(),
        source: name.clone(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        rebuilt: false,
    };
    write_meta(&version, &meta)?;
    let mut state = state();
    state.pending = Some(name.clone());
    save_state(&state)?;
    prune_binaries();
    Ok(name)
}

/// Adds a rebuilt binary to an existing source-backed version.
pub fn adopt_build(name: &str, exe: &Path) -> Result<(), String> {
    if !super::source::exists(name) {
        return Err(format!("The source snapshot for {name} is missing."));
    }
    record_base();
    let dir = root()?;
    let version = dir.join("versions").join(name);
    std::fs::create_dir_all(&version)
        .map_err(|error| format!("Could not create {}: {error}", version.display()))?;
    std::fs::copy(exe, version.join(exe_name()))
        .map_err(|error| format!("Could not copy the rebuilt binary: {error}"))?;
    copy_assets(&version);
    let mut meta = version_meta(name).unwrap_or_else(|| {
        super::source::manifest(name)
            .map(|manifest| VersionMeta {
                goal: manifest.goal,
                created_ms: manifest.created_ms,
                source: manifest.version,
                app_version: manifest.app_version,
                ..VersionMeta::default()
            })
            .unwrap_or_default()
    });
    meta.source = name.to_string();
    meta.rebuilt = true;
    meta.app_version = env!("CARGO_PKG_VERSION").to_string();
    write_meta(&version, &meta)?;
    let mut state = state();
    state.pending = Some(name.to_string());
    save_state(&state)?;
    prune_binaries();
    Ok(())
}

/// Every known version, newest first: runnable binaries plus source-only
/// versions whose binary was pruned.
pub fn versions() -> Vec<Version> {
    let Ok(dir) = root() else {
        return Vec::new();
    };
    let state = state();
    let mut out: Vec<(i64, Version)> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir.join("versions")) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "base" {
                seen.push(name);
                continue;
            }
            let binary = entry.path().join(exe_name()).exists();
            let source = super::source::exists(&name);
            if !binary && !source {
                continue;
            }
            let meta: VersionMeta = std::fs::read(entry.path().join("meta.json"))
                .ok()
                .and_then(|bytes| parse_json(&bytes))
                .unwrap_or_default();
            let goal = if meta.goal.trim().is_empty() {
                "UI edit".to_string()
            } else {
                one_line(meta.goal.trim(), 96)
            };
            out.push((
                meta.created_ms,
                Version {
                    current: state.current == name,
                    pending: state.pending.as_deref() == Some(name.as_str()),
                    when: when_label(meta.created_ms),
                    app_version: meta.app_version,
                    goal,
                    name: name.clone(),
                    binary,
                    source,
                    rebuilt: meta.rebuilt,
                },
            ));
            seen.push(name);
        }
    }
    for manifest in super::source::list() {
        if seen.iter().any(|name| name == &manifest.version) || manifest.version == "base" {
            continue;
        }
        let goal = if manifest.goal.trim().is_empty() {
            "UI edit".to_string()
        } else {
            one_line(manifest.goal.trim(), 96)
        };
        out.push((
            manifest.created_ms,
            Version {
                current: state.current == manifest.version,
                pending: state.pending.as_deref() == Some(manifest.version.as_str()),
                when: when_label(manifest.created_ms),
                app_version: manifest.app_version,
                goal,
                name: manifest.version,
                binary: false,
                source: true,
                rebuilt: false,
            },
        ));
    }
    out.sort_by_key(|(stamp, _)| std::cmp::Reverse(*stamp));
    out.into_iter().map(|(_, version)| version).collect()
}

/// Keeps only the runnable versions a restart can use. Source manifests stay;
/// the binaries for older versions are deleted and rebuilt on demand.
fn prune_binaries() {
    let Ok(dir) = root() else {
        return;
    };
    let state = state();
    let base = base_exe();
    let keep = [
        state.current.clone(),
        state.last_good.clone(),
        state.pending.clone().unwrap_or_default(),
    ];
    let Ok(entries) = std::fs::read_dir(dir.join("versions")) else {
        return;
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "base" {
            let base_copy = entry.path().join(exe_name());
            let keep_base = base
                .as_ref()
                .is_some_and(|base| same_path(base, &base_copy));
            if !keep_base {
                let _ = std::fs::remove_dir_all(entry.path());
            }
            continue;
        }
        if keep.iter().any(|kept| kept == &name) {
            continue;
        }
        let _ = std::fs::remove_dir_all(entry.path());
    }
}

/// Startup enforcement: only current, last-good and pending binaries stay.
pub fn prune_now() {
    prune_binaries();
}

/// What a delete removed.
#[derive(Clone, Debug, Default)]
pub struct DeleteReport {
    /// Bytes freed on disk.
    pub freed: u64,
    /// How many build folders were removed.
    pub builds: usize,
}

/// The folder that holds the editor's custom UI files, shown in Settings so the
/// same files can be found outside the app.
pub fn store_dir() -> String {
    root()
        .map(|dir| dir.display().to_string())
        .unwrap_or_else(|_| "%APPDATA%\\Runecord\\aiedit".to_string())
}

/// Deletes every saved custom UI build, source snapshot and rollback copy.
///
/// Provider settings and the API key live in their own files and stay. Windows
/// cannot delete a running executable, so a build that is in use has to be left
/// with "Reset to default UI" first.
pub fn delete_custom_ui() -> Result<DeleteReport, String> {
    let dir = root()?;
    if let Ok(me) = std::env::current_exe()
        && is_version_exe(&me)
    {
        return Err(
            "This window is the stored UI build. Reset to the default UI first, then delete the files."
                .into(),
        );
    }
    let builds = std::fs::read_dir(dir.join("versions"))
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.path().is_dir())
                .count()
        })
        .unwrap_or(0);
    let freed = delete_ui_files(&dir)?;
    let mut state = state();
    state.current.clear();
    state.last_good.clear();
    state.pending = None;
    state.kept_app_version.clear();
    save_state(&state)?;
    Ok(DeleteReport { freed, builds })
}

/// Removes the three UI folders under `dir` and reports the bytes freed.
pub(super) fn delete_ui_files(dir: &Path) -> Result<u64, String> {
    let mut freed = 0;
    for folder in ["versions", "source", "backup"] {
        let path = dir.join(folder);
        if !path.exists() {
            continue;
        }
        freed += dir_size(&path);
        std::fs::remove_dir_all(&path)
            .map_err(|error| format!("Could not delete {}: {error}", path.display()))?;
    }
    // Ready flags belong to the restart helper, not to a build.
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("ready-") && name.ends_with(".flag") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
    Ok(freed)
}

/// Whether a version can be rebuilt from its source snapshot.
pub fn has_source(name: &str) -> bool {
    super::source::exists(name)
}

/// Restores a version's writable trees from its source snapshot.
pub fn restore_source(name: &str) -> Result<usize, String> {
    super::source::restore(name)
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

/// Records the installed app path the launcher/fallback should use.
///
/// A staged child receives `RUNECORD_BASE_EXE` from the launcher so it never
/// mistakes itself for the built-in app.
pub fn record_base() {
    let mut state = state();
    if let Some(path) = std::env::var_os("RUNECORD_BASE_EXE").map(PathBuf::from) {
        let value = path.to_string_lossy().to_string();
        if state.base_exe != value {
            state.base_exe = value;
            let _ = save_state(&state);
        }
        return;
    }
    // Keep the first known-good base. A dev checkout should not replace a
    // packaged build just because `run.bat` happened to start a debug exe.
    if !state.base_exe.is_empty() && Path::new(&state.base_exe).exists() {
        return;
    }
    let Some(path) = std::env::current_exe()
        .ok()
        .filter(|exe| !is_version_exe(exe))
    else {
        return;
    };
    let value = path.to_string_lossy().to_string();
    if state.base_exe != value {
        state.base_exe = value;
        let _ = save_state(&state);
    }
}

/// The installed app that owns the built-in UI.
pub fn base_exe() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("RUNECORD_BASE_EXE").map(PathBuf::from)
        && path.exists()
    {
        return Some(path);
    }
    let state = state();
    if !state.base_exe.is_empty() {
        let path = PathBuf::from(&state.base_exe);
        if path.exists() {
            return Some(path);
        }
    }
    std::env::current_exe()
        .ok()
        .filter(|exe| !is_version_exe(exe))
}

/// Whether a path is one of the staged `versions/ui-*` executables.
fn is_version_exe(exe: &Path) -> bool {
    let Ok(root) = root() else {
        return false;
    };
    let versions = root.join("versions");
    exe.parent()
        .and_then(|parent| parent.parent())
        .is_some_and(|parent| same_path(parent, &versions))
}

/// A development checkout should run the source the user just built, not the
/// staged build. Installed apps have no workspace manifest above them.
fn is_dev_checkout(exe: &Path) -> bool {
    let Some(dir) = exe.parent() else {
        return false;
    };
    dir.ancestors().take(6).any(|candidate| {
        candidate.join("crates/client/Cargo.toml").exists() && candidate.join("Cargo.toml").exists()
    })
}

fn same_path(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn write_meta(version: &Path, meta: &VersionMeta) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(meta).map_err(|error| error.to_string())?;
    std::fs::write(version.join("meta.json"), bytes)
        .map_err(|error| format!("Could not record the build metadata: {error}"))
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

fn fallback_exe(state: &State) -> Option<PathBuf> {
    version_exe(&state.last_good).ok().or_else(base_exe)
}

/// Starts the staged build through a supervisor and returns once it is spawned.
pub fn restart_staged() -> Result<(), String> {
    let state = state();
    let pending = state
        .pending
        .clone()
        .ok_or("There is no staged build to restart into.")?;
    let child = version_exe(&pending)?;
    if super::source::exists(&pending) {
        super::source::restore(&pending)?;
    }
    let fallback = fallback_exe(&state).unwrap_or_else(|| child.clone());
    supervise_spawn(&child, &fallback)
}

/// Starts the last good build, used by undo.
pub fn restart_last_good() -> Result<(), String> {
    let state = state();
    if let Ok(child) = version_exe(&state.last_good) {
        if super::source::exists(&state.last_good) {
            let _ = super::source::restore(&state.last_good);
        }
        let fallback = base_exe().unwrap_or_else(|| child.clone());
        return supervise_spawn(&child, &fallback);
    }
    restart_base()
}

/// Switches the next restart to one of the staged builds.
///
/// Its source snapshot is restored first, so the files on disk match the build
/// that is about to run; otherwise the next edit would start from a newer tree
/// than the UI the user is looking at.
pub fn restart_into(name: &str) -> Result<(), String> {
    let child = version_exe(name)?;
    if super::source::exists(name) {
        super::source::restore(name)?;
    } else if let Some(meta) = version_meta(name)
        && !meta.backup.is_empty()
    {
        let _ = restore_backup(&meta.backup);
    }
    let mut state = state();
    state.pending = Some(name.to_string());
    save_state(&state)?;
    let fallback = fallback_exe(&state).unwrap_or_else(|| child.clone());
    supervise_spawn(&child, &fallback)
}

/// Restarts into the installed app's built-in UI and clears the custom state.
pub fn restart_base() -> Result<(), String> {
    let base = base_exe().ok_or(
        "The installed Runecord executable is unknown, so the default UI cannot be started.",
    )?;
    let mut state = state();
    state.current.clear();
    state.last_good.clear();
    state.pending = None;
    state.kept_app_version.clear();
    save_state(&state)?;
    supervise_spawn(&base, &base)
}

/// Promotes the pending build when the caller has already verified it starts.
/// The supervisor does this automatically; the cleanup uses it for a build that
/// was launched manually.
pub fn promote_pending() -> Result<(), String> {
    let Some(state) = finish_pending() else {
        return Err("There is no pending build to promote.".into());
    };
    save_state(&state)
}

/// Records that the user chose to keep the current custom UI for this app
/// version. The update warning returns when the app itself changes again.
pub fn keep_current_version() -> Result<(), String> {
    let mut state = state();
    state.kept_app_version = super::app_version().to_string();
    save_state(&state)
}

/// The app version the current UI was built for, when it differs from this app
/// and the user has not acknowledged it for this version.
pub fn outdated_current() -> Option<String> {
    let state = state();
    if state.current.is_empty() || state.kept_app_version == super::app_version() {
        return None;
    }
    let built = version_meta(&state.current)
        .map(|meta| meta.app_version)
        .filter(|version| !version.trim().is_empty())
        .or_else(|| {
            super::source::manifest(&state.current)
                .map(|manifest| manifest.app_version)
                .filter(|version| !version.trim().is_empty())
        })?;
    if built.trim() == super::app_version() {
        None
    } else {
        Some(built.trim().to_string())
    }
}

/// One-time and startup cleanup: keep the current and last-good binaries, delete
/// every other executable, and leave their source manifests in place.
#[derive(Clone, Debug, Default)]
pub struct CleanupReport {
    pub kept: Vec<String>,
    pub removed: Vec<String>,
    pub freed: u64,
}

pub fn cleanup_versions() -> Result<CleanupReport, String> {
    record_base();
    let dir = root()?;
    let mut state = state();
    if state.current == "base" {
        state.current.clear();
    }
    if state.last_good == "base" {
        state.last_good.clear();
    }
    if !state.current.is_empty() && version_exe(&state.current).is_err() {
        state.current.clear();
    }
    if !state.last_good.is_empty() && version_exe(&state.last_good).is_err() {
        state.last_good.clear();
    }
    if state.current == state.last_good {
        state.last_good.clear();
    }
    if state.current.is_empty() {
        state.kept_app_version.clear();
    }
    // A pending build is kept until the user restarts; never silently promote an
    // unverified executable.
    save_state(&state)?;

    // Record the source behind the current version when this is the first run of
    // the snapshot store.
    if !state.current.is_empty() && !super::source::exists(&state.current) {
        let goal = version_meta(&state.current)
            .map(|meta| meta.goal)
            .unwrap_or_else(|| "Current UI".to_string());
        let _ = super::source::snapshot(&state.current, &goal);
    }

    let keep = [
        state.current.clone(),
        state.last_good.clone(),
        state.pending.clone().unwrap_or_default(),
    ];
    let base = base_exe();
    let mut report = CleanupReport {
        kept: keep
            .iter()
            .filter(|name| !name.is_empty())
            .cloned()
            .collect(),
        ..CleanupReport::default()
    };
    let Ok(entries) = std::fs::read_dir(dir.join("versions")) else {
        return Ok(report);
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let keep_this = if name == "base" {
            base.as_ref()
                .is_some_and(|base| same_path(base, &entry.path().join(exe_name())))
        } else {
            keep.iter().any(|kept| kept == &name)
        };
        if keep_this {
            continue;
        }
        report.freed += dir_size(&entry.path());
        if std::fs::remove_dir_all(entry.path()).is_ok() {
            report.removed.push(name);
        }
    }
    Ok(report)
}

fn dir_size(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                dir_size(&path)
            } else {
                entry.metadata().map(|meta| meta.len()).unwrap_or(0)
            }
        })
        .sum()
}

/// The installed app dispatches to the current version on startup. Returns true
/// when a child was started and this process should exit.
pub fn launch_current() -> bool {
    if std::env::var_os("RUNECORD_AIEDIT_CHILD").is_some() {
        return false;
    }
    let Ok(me) = std::env::current_exe() else {
        return false;
    };
    // A development checkout must run the source that was just built.
    if is_version_exe(&me) || is_dev_checkout(&me) {
        return false;
    }
    let state = state();
    if state.current.is_empty() || state.current == "base" {
        return false;
    }
    let Ok(child) = version_exe(&state.current) else {
        return false;
    };
    if same_path(&child, &me) {
        return false;
    }
    record_base();
    prune_binaries();
    let fallback = fallback_exe(&state).unwrap_or_else(|| me.clone());
    match supervise_spawn(&child, &fallback) {
        Ok(()) => true,
        Err(error) => {
            eprintln!("aiedit launcher: {error}");
            false
        }
    }
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
    if let Some(base) = base_exe() {
        command.env("RUNECORD_BASE_EXE", base);
    }
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
    let base = std::env::var("RUNECORD_BASE_EXE").unwrap_or_default();
    if start_and_wait(child, Path::new(ready), extra, &base) {
        if let Some(state) = finish_pending() {
            let _ = save_state(&state);
        }
        prune_binaries();
        if let Ok(dir) = root() {
            let _ = std::fs::remove_file(dir.join("last-failure.txt"));
        }
        return;
    }

    // The new build never painted. Try the previous build with the same proof
    // before falling back to the installed app's built-in UI.
    if !fallback.is_empty() && fallback != child {
        let second = fallback_ready(ready, "last-good");
        if start_and_wait(fallback, &second, extra, &base) {
            note_failure("The new UI build did not start; Runecord reopened the previous build.");
            mark_current_from_path(fallback);
            prune_binaries();
            return;
        }
    }
    if !base.is_empty() && base != child && base != fallback {
        let third = fallback_ready(ready, "built-in");
        if start_and_wait(&base, &third, extra, &base) {
            note_failure("The new UI build did not start; Runecord reopened the built-in UI.");
            clear_current();
            prune_binaries();
            return;
        }
    }
    record_failure("The new UI build did not start, so Runecord reopened the previous one.");
}

/// Starts an executable and waits for its ready marker. On success the child is
/// deliberately left running; on failure it is killed.
fn start_and_wait(exe: &str, ready: &Path, extra: &[String], base: &str) -> bool {
    let _ = std::fs::remove_file(ready);
    let mut command = std::process::Command::new(exe);
    command
        .env("RUNECORD_READY_FILE", ready)
        .env("RUNECORD_AIEDIT_CHILD", "1");
    if !base.is_empty() {
        command.env("RUNECORD_BASE_EXE", base);
    }
    command.args(extra);
    let mut spawned = match command.spawn() {
        Ok(child) => child,
        Err(_) => return false,
    };
    let deadline = Instant::now() + Duration::from_secs(90);
    while Instant::now() < deadline {
        if ready.exists() {
            let _ = std::fs::remove_file(ready);
            return true;
        }
        match spawned.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {}
            Err(_) => break,
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    let _ = spawned.kill();
    let _ = std::fs::remove_file(ready);
    false
}

fn fallback_ready(ready: &str, label: &str) -> PathBuf {
    let path = Path::new(ready);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("ready");
    parent.join(format!("{stem}-{label}-{}.flag", stamp()))
}

fn version_name_for_path(path: &str) -> Option<String> {
    let path = Path::new(path);
    let versions = root().ok()?.join("versions");
    let parent = path.parent()?;
    let grand = parent.parent()?;
    if !same_path(grand, &versions) {
        return None;
    }
    parent
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
}

fn mark_current_from_path(path: &str) {
    let mut state = state();
    state.pending = None;
    if let Some(name) = version_name_for_path(path) {
        state.current = name;
        state.last_good.clear();
    } else {
        state.current.clear();
        state.last_good.clear();
    }
    let _ = save_state(&state);
}

fn clear_current() {
    let mut state = state();
    state.current.clear();
    state.last_good.clear();
    state.pending = None;
    state.kept_app_version.clear();
    let _ = save_state(&state);
}

fn note_failure(note: &str) {
    if let Ok(dir) = root() {
        let _ = std::fs::write(dir.join("last-failure.txt"), note);
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
    state.kept_app_version.clear();
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
