//! Full source snapshots for staged UI builds.
//!
//! A version is more than the executable made from it: it is the exact UI
//! source tree at that moment. The files are stored by content hash in
//! `source/objects`, and each version has a manifest listing every writable
//! file. That keeps history small (text, deduplicated) and lets a version whose
//! binary was pruned be rebuilt later.
//!
//! Only the trees the AI editor can write are recorded. The rest of the app is
//! the installed base; rebuilding applies the snapshot on top of it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The source tree a build was made from.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub goal: String,
    #[serde(default)]
    pub created_ms: i64,
    #[serde(default)]
    pub app_version: String,
    #[serde(default)]
    pub toolchain: String,
    /// Relative path -> object hash.
    #[serde(default)]
    pub files: BTreeMap<String, String>,
}

impl Manifest {
    #[allow(dead_code)] // used by the snapshot tests and for diagnostics
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

/// A snapshot in the per-user store.
pub fn snapshot(name: &str, goal: &str) -> Result<Manifest, String> {
    let source = super::install::source_root()?;
    let store = super::install::root()?;
    snapshot_at(&source, &store, name, goal)
}

/// Rebuilds a version's writable trees from its manifest.
pub fn restore(name: &str) -> Result<usize, String> {
    let source = super::install::source_root()?;
    let store = super::install::root()?;
    restore_at(&source, &store, name)
}

pub fn exists(name: &str) -> bool {
    super::install::root()
        .ok()
        .and_then(|store| manifest_at(&store, name))
        .is_some()
}

pub fn manifest(name: &str) -> Option<Manifest> {
    manifest_at(&super::install::root().ok()?, name)
}

/// Source-backed versions, oldest first.
pub fn list() -> Vec<Manifest> {
    let Ok(store) = super::install::root() else {
        return Vec::new();
    };
    list_at(&store)
}

pub fn snapshot_at(
    source_root: &Path,
    store_root: &Path,
    name: &str,
    goal: &str,
) -> Result<Manifest, String> {
    let mut manifest = Manifest {
        version: name.to_string(),
        goal: goal.to_string(),
        created_ms: now_ms(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        toolchain: toolchain_at(source_root),
        files: BTreeMap::new(),
    };
    for file in collect(source_root)? {
        let bytes = std::fs::read(&file)
            .map_err(|error| format!("Could not read {}: {error}", file.display()))?;
        let hash = hash(&bytes);
        let object = store_root.join("source").join("objects").join(&hash);
        if !object.exists() {
            if let Some(parent) = object.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
            }
            let temp = object.with_extension("tmp");
            std::fs::write(&temp, &bytes)
                .map_err(|error| format!("Could not store {}: {error}", file.display()))?;
            std::fs::rename(&temp, &object)
                .map_err(|error| format!("Could not store {}: {error}", file.display()))?;
        }
        let relative = file
            .strip_prefix(source_root)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        manifest.files.insert(relative, hash);
    }
    if manifest.files.is_empty() {
        return Err("The UI source tree is empty, so there is nothing to record.".into());
    }
    write_manifest(store_root, &manifest)?;
    Ok(manifest)
}

pub fn restore_at(source_root: &Path, store_root: &Path, name: &str) -> Result<usize, String> {
    let manifest = manifest_at(store_root, name)
        .ok_or_else(|| format!("The source snapshot for {name} is missing."))?;
    let mut restored = 0usize;
    for (relative, hash) in &manifest.files {
        let object = store_root.join("source").join("objects").join(hash);
        let bytes = std::fs::read(&object).map_err(|error| {
            format!("The source file {relative} is missing from the store: {error}")
        })?;
        let target = source_root.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
        }
        std::fs::write(&target, bytes)
            .map_err(|error| format!("Could not restore {relative}: {error}"))?;
        restored += 1;
    }
    // A version is a complete tree: files deleted since then must stay deleted.
    for file in collect(source_root)? {
        let relative = file
            .strip_prefix(source_root)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        if !manifest.files.contains_key(&relative) {
            let _ = std::fs::remove_file(&file);
        }
    }
    Ok(restored)
}

pub fn manifest_at(store_root: &Path, name: &str) -> Option<Manifest> {
    let path = store_root
        .join("source")
        .join("versions")
        .join(name)
        .join("manifest.json");
    let bytes = std::fs::read(path).ok()?;
    parse_json(&bytes)
}

pub fn list_at(store_root: &Path) -> Vec<Manifest> {
    let dir = store_root.join("source").join("versions");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<Manifest> = entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            manifest_at(store_root, &name)
        })
        .collect();
    out.sort_by_key(|manifest| manifest.created_ms);
    out
}

fn write_manifest(store_root: &Path, manifest: &Manifest) -> Result<(), String> {
    let dir = store_root
        .join("source")
        .join("versions")
        .join(&manifest.version);
    std::fs::create_dir_all(&dir)
        .map_err(|error| format!("Could not create {}: {error}", dir.display()))?;
    let bytes = serde_json::to_vec_pretty(manifest).map_err(|error| error.to_string())?;
    std::fs::write(dir.join("manifest.json"), bytes)
        .map_err(|error| format!("Could not record the source snapshot: {error}"))
}

/// Every `.rs` file in the writable UI trees.
fn collect(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    for dir in super::patch::WRITABLE {
        let path = root.join(dir.trim_end_matches('/'));
        if !path.exists() {
            continue;
        }
        walk(&path, &mut out)
            .map_err(|error| format!("Could not list {}: {error}", path.display()))?;
    }
    out.sort();
    Ok(out)
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    Ok(())
}

fn hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn toolchain_at(source_root: &Path) -> String {
    std::fs::read_to_string(source_root.join("rust-toolchain.toml"))
        .map(|text| text.trim().to_string())
        .unwrap_or_default()
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or(0)
}

fn parse_json<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    let text = String::from_utf8_lossy(bytes);
    serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()
}

#[cfg(test)]
mod tests {
    use super::{list_at, restore_at, snapshot_at};

    fn temp_root(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "runecord-source-{label}-{}-{}",
            std::process::id(),
            super::now_ms()
        ));
        std::fs::create_dir_all(path.join("crates/client/src/ui")).unwrap();
        std::fs::create_dir_all(path.join("crates/ui/src")).unwrap();
        path
    }

    #[test]
    fn a_snapshot_restores_the_complete_tree() {
        let source = temp_root("restore");
        let store = temp_root("store");
        std::fs::write(source.join("crates/client/src/ui/one.rs"), "fn one() {}\n").unwrap();
        std::fs::write(source.join("crates/ui/src/two.rs"), "fn two() {}\n").unwrap();

        let manifest = snapshot_at(&source, &store, "v1", "first").unwrap();
        assert_eq!(manifest.file_count(), 2);
        assert_eq!(list_at(&store).len(), 1);

        std::fs::write(
            source.join("crates/client/src/ui/one.rs"),
            "fn changed() {}\n",
        )
        .unwrap();
        std::fs::write(
            source.join("crates/client/src/ui/extra.rs"),
            "fn extra() {}\n",
        )
        .unwrap();

        let restored = restore_at(&source, &store, "v1").unwrap();
        assert_eq!(restored, 2);
        assert_eq!(
            std::fs::read_to_string(source.join("crates/client/src/ui/one.rs")).unwrap(),
            "fn one() {}\n"
        );
        assert!(!source.join("crates/client/src/ui/extra.rs").exists());
        let _ = std::fs::remove_dir_all(source);
        let _ = std::fs::remove_dir_all(store);
    }

    #[test]
    fn unchanged_files_are_stored_once() {
        let source = temp_root("dedupe");
        let store = temp_root("dedupe-store");
        std::fs::write(source.join("crates/client/src/ui/one.rs"), "fn one() {}\n").unwrap();
        snapshot_at(&source, &store, "v1", "first").unwrap();
        std::fs::write(source.join("crates/client/src/ui/one.rs"), "fn two() {}\n").unwrap();
        snapshot_at(&source, &store, "v2", "second").unwrap();

        let objects = std::fs::read_dir(store.join("source/objects"))
            .unwrap()
            .count();
        assert_eq!(objects, 2, "one object per distinct body");
        let _ = std::fs::remove_dir_all(source);
        let _ = std::fs::remove_dir_all(store);
    }

    #[test]
    fn a_missing_snapshot_is_an_error() {
        let source = temp_root("missing");
        let store = temp_root("missing-store");
        assert!(restore_at(&source, &store, "nope").is_err());
        let _ = std::fs::remove_dir_all(source);
        let _ = std::fs::remove_dir_all(store);
    }
}
