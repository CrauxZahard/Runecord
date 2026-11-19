//! The files a generated patch may touch, the prompt format, and the writer.
//!
//! The allowlist is the enforcement: a reply that names any other path is
//! rejected whole, so a model cannot change networking, auth or storage even if
//! it decides to. The system prompt asks it not to; this module makes it
//! impossible.

use std::path::{Path, PathBuf};

/// Directories the editor writes to. Everything else is read-only.
pub const WRITABLE: [&str; 2] = ["crates/client/src/ui/", "crates/ui/src/"];

/// Files sent before the model has asked for anything: the shell, the shared
/// widgets and the tokens. Enough to plan against without spending the budget
/// on the whole tree.
const GROUNDING: [&str; 3] = [
    "crates/client/src/ui/mod.rs",
    "crates/client/src/ui/common.rs",
    "crates/ui/src/theme.rs",
];

/// Bytes of source sent up front. Roughly 20k tokens.
const BUDGET: usize = 80_000;

/// Whether a path from a reply may be written.
pub fn is_writable(path: &str) -> bool {
    let normal = path.trim().replace('\\', "/");
    if normal.is_empty() || normal.starts_with('/') || normal.contains(':') {
        return false;
    }
    if normal
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return false;
    }
    normal.ends_with(".rs") && WRITABLE.iter().any(|prefix| normal.starts_with(prefix))
}

/// The source tree the editor reads and writes.
pub fn source_root() -> Result<PathBuf, String> {
    super::install::source_root()
}

/// Files sent before the model has asked for anything.
pub fn grounding() -> Result<Vec<(String, String)>, String> {
    let root = source_root()?;
    let names = GROUNDING.iter().map(|path| path.to_string()).collect();
    read_budgeted(&root, names, BUDGET)
}

/// Every file the editor may touch, with its size, so the model can ask for
/// exactly what it needs instead of guessing from names.
pub fn manifest() -> Result<String, String> {
    let root = source_root()?;
    let mut files: Vec<(String, usize)> = Vec::new();
    for dir in ["crates/client/src/ui", "crates/ui/src"] {
        walk(&root.join(dir), &root, &mut files)
            .map_err(|error| format!("Could not list {dir}: {error}"))?;
    }
    if files.is_empty() {
        return Err(format!(
            "No UI source found under {}. The editor needs the source tree.",
            root.display()
        ));
    }
    files.sort();
    let mut out = String::new();
    for (path, lines) in files {
        let attached = GROUNDING.contains(&path.as_str());
        out.push_str(&format!(
            "{path:<55} {lines:>5} lines{}\n",
            if attached { "  (attached)" } else { "" }
        ));
    }
    Ok(out)
}

fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, usize)>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, root, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            let name = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let lines = std::fs::read_to_string(&path)
                .map(|text| text.lines().count())
                .unwrap_or(0);
            out.push((name, lines));
        }
    }
    Ok(())
}

/// Reads a specific list of files, for a `NEED` round.
pub fn read_named(root: &Path, names: &[String]) -> Result<Vec<(String, String)>, String> {
    read_budgeted(root, names.to_vec(), usize::MAX)
}

fn read_budgeted(
    root: &Path,
    names: Vec<String>,
    budget: usize,
) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    let mut used = 0usize;
    for name in names {
        if !is_writable(&name) {
            continue;
        }
        let path = root.join(&name);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if used + text.len() > budget && budget != usize::MAX {
            continue;
        }
        used += text.len();
        out.push((name, text));
    }
    if out.is_empty() {
        return Err(format!(
            "No UI source found under {}. The editor needs the source tree.",
            root.display()
        ));
    }
    Ok(out)
}

/// What a reply asked for: one of a plan, files, requests, or a note.
pub struct Reply {
    pub plan: Option<String>,
    pub notes: Vec<String>,
    pub files: Vec<(String, String)>,
    pub needs: Vec<String>,
}

impl Reply {
    /// Whether the reply contains nothing the host can act on.
    pub fn is_empty(&self) -> bool {
        self.plan.is_none()
            && self.notes.is_empty()
            && self.files.is_empty()
            && self.needs.is_empty()
    }
}

/// Parses `<<<PLAN`, `<<<NOTE`, `<<<NEED` and `<<<FILE` blocks.
///
/// Any path outside [`WRITABLE`] fails the whole reply: applying half a patch
/// that also edits the network layer would be worse than applying nothing.
pub fn parse(text: &str) -> Result<Reply, String> {
    let mut files = Vec::new();
    let mut needs = Vec::new();
    let mut notes = Vec::new();
    let mut plan = None;
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("<<<NEED ") {
            let name = rest.trim_end_matches(">>>").trim();
            if is_writable(name) {
                needs.push(name.to_string());
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("<<<NOTE") {
            let mut body = rest.trim().to_string();
            if let Some(head) = body.strip_suffix(">>>") {
                body = head.trim().to_string();
            } else {
                body.clear();
                for inner in lines.by_ref() {
                    if inner.trim() == ">>>" {
                        break;
                    }
                    body.push_str(inner.trim());
                    body.push(' ');
                }
                body = body.trim().to_string();
            }
            if !body.is_empty() {
                notes.push(body);
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("<<<PLAN") {
            let mut body = String::new();
            let inline = rest.trim();
            if let Some(head) = inline.strip_suffix(">>>") {
                body.push_str(head.trim());
            } else {
                for inner in lines.by_ref() {
                    if inner.trim() == ">>>" {
                        break;
                    }
                    body.push_str(inner.trim_end());
                    body.push('\n');
                }
            }
            if !body.trim().is_empty() {
                plan = Some(body.trim().to_string());
            }
            continue;
        }
        let Some(rest) = trimmed.strip_prefix("<<<FILE ") else {
            continue;
        };
        let name = rest.trim();
        let name = name.strip_suffix(">>>").unwrap_or(name).trim();
        if !is_writable(name) {
            return Err(format!(
                "The model tried to edit {name}, which is outside the UI. Nothing was applied."
            ));
        }
        let mut body = String::new();
        for inner in lines.by_ref() {
            if inner.trim() == ">>>" {
                break;
            }
            body.push_str(inner);
            body.push('\n');
        }
        if body.trim().is_empty() {
            return Err(format!("The model returned an empty file for {name}."));
        }
        files.push((name.to_string(), trim_fence(body)));
    }
    let reply = Reply {
        plan,
        notes,
        files,
        needs,
    };
    if reply.is_empty() {
        return Err(
            "The model's reply had nothing in the expected format. Nothing was applied.".into(),
        );
    }
    Ok(reply)
}

/// Drops a Markdown fence the model wrapped a file in, without touching fences
/// that are part of the file (a doc comment can legitimately contain one).
fn trim_fence(body: String) -> String {
    let mut lines: Vec<&str> = body.lines().collect();
    if lines
        .first()
        .is_some_and(|line| line.trim_start().starts_with("```"))
    {
        lines.remove(0);
    }
    if lines.last().is_some_and(|line| line.trim() == "```") {
        lines.pop();
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// Copies each target beside the tree, then writes the new contents.
///
/// Returns the backup id, which `install::reset` uses to restore the original
/// files, and the per-file line counts so the card can say what changed. A file
/// that did not exist before is recorded as absent.
pub fn snapshot_and_write(files: &[(String, String)]) -> Result<(String, Vec<Change>), String> {
    let root = source_root()?;
    let id = super::install::backup_id();
    let backup = super::install::root()?.join("backup").join(&id);
    std::fs::create_dir_all(&backup)
        .map_err(|error| format!("Could not create {}: {error}", backup.display()))?;
    let mut manifest: Vec<String> = Vec::new();
    let mut changes: Vec<Change> = Vec::new();
    for (name, body) in files {
        if manifest.iter().any(|seen| seen == name) {
            continue;
        }
        let source = root.join(name);
        let before = std::fs::read(&source).ok();
        let before_lines = before
            .as_deref()
            .map(|bytes| String::from_utf8_lossy(bytes).lines().count())
            .unwrap_or(0);
        if let Some(old) = before {
            let target = backup.join(name);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
            }
            std::fs::write(&target, old)
                .map_err(|error| format!("Could not back up {name}: {error}"))?;
        }
        changes.push(Change {
            name: name.clone(),
            before: before_lines,
            after: body.lines().count(),
        });
        manifest.push(name.clone());
    }
    let record = serde_json::json!({ "id": id, "files": manifest });
    std::fs::write(
        backup.join("manifest.json"),
        serde_json::to_vec_pretty(&record).unwrap_or_default(),
    )
    .map_err(|error| format!("Could not record the backup: {error}"))?;
    for (name, body) in files {
        let target = root.join(name);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
        }
        std::fs::write(&target, body)
            .map_err(|error| format!("Could not write {name}: {error}"))?;
    }
    Ok((id, changes))
}

/// What one file's edit did to its size, for the progress pane.
#[derive(Clone, Debug)]
pub struct Change {
    pub name: String,
    pub before: usize,
    pub after: usize,
}

#[cfg(test)]
mod tests {
    use super::{is_writable, parse};

    #[test]
    fn the_allowlist_covers_ui_and_nothing_else() {
        assert!(is_writable("crates/client/src/ui/sidebar.rs"));
        assert!(is_writable("crates/client/src/ui/settings/appearance.rs"));
        assert!(is_writable("crates/ui/src/theme.rs"));
        assert!(!is_writable("crates/client/src/net/mod.rs"));
        assert!(!is_writable("crates/client/src/app/mod.rs"));
        assert!(!is_writable("crates/client/src/ui/../net/mod.rs"));
        assert!(!is_writable("crates/client/src/ui/../../config.rs"));
        assert!(!is_writable("C:/temp/x.rs"));
        assert!(!is_writable("/etc/x.rs"));
        assert!(!is_writable("crates/ui/src/theme.rs.bak"));
        assert!(!is_writable("crates/client/src/ui/"));
    }

    #[test]
    fn parses_files_and_needs() {
        let reply = "<<<NEED crates/ui/src/icons.rs>>>\n\
                     <<<FILE crates/client/src/ui/sidebar.rs\nfn view() {}\n>>>\n";
        let parsed = parse(reply).expect("reply parses");
        assert_eq!(parsed.needs, vec!["crates/ui/src/icons.rs"]);
        assert_eq!(parsed.files.len(), 1);
        assert_eq!(parsed.files[0].0, "crates/client/src/ui/sidebar.rs");
        assert_eq!(parsed.files[0].1, "fn view() {}\n");
    }

    #[test]
    fn strips_a_markdown_fence_around_a_file() {
        let reply = "<<<FILE crates/client/src/ui/rail.rs\n```rust\nfn rail() {}\n```\n>>>\n";
        let parsed = parse(reply).expect("reply parses");
        assert_eq!(parsed.files[0].1, "fn rail() {}\n");
    }

    #[test]
    fn trims_the_path_before_writing() {
        let reply = "<<<FILE   crates/ui/src/theme.rs>>>\nfn x() {}\n>>>\n";
        let parsed = parse(reply).expect("reply parses");
        assert_eq!(parsed.files[0].0, "crates/ui/src/theme.rs");
    }

    #[test]
    fn rejects_a_file_outside_the_ui() {
        let reply = "<<<FILE crates/client/src/net/mod.rs\nfn x() {}\n>>>\n";
        assert!(parse(reply).is_err());
    }

    #[test]
    fn parses_a_plan_before_the_code() {
        let reply = "<<<PLAN\nMove the rail to the right.\nKeep the member list.\n>>>\n\
                     <<<NEED crates/client/src/ui/rail.rs>>>\n";
        let parsed = parse(reply).expect("reply parses");
        assert!(parsed.plan.as_deref().unwrap().contains("Move the rail"));
        assert_eq!(parsed.needs, vec!["crates/client/src/ui/rail.rs"]);
        assert!(parsed.files.is_empty());
    }

    #[test]
    fn a_note_alone_is_an_answer() {
        let reply = "<<<NOTE This needs a stored preference, which the view layer cannot add.>>>";
        let parsed = parse(reply).expect("reply parses");
        assert!(parsed.files.is_empty());
        assert!(parsed.needs.is_empty());
        assert_eq!(parsed.notes.len(), 1);
    }

    #[test]
    fn rejects_an_empty_reply() {
        assert!(parse("I moved the sidebar.").is_err());
    }
}
