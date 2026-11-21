//! End-to-end check of the editor without a model.
//!
//! Ignored by default because it compiles the release binary (minutes). Run it
//! with:
//!
//! ```text
//! cargo test -p runecord --lib aiedit::tests -- --ignored --nocapture
//! ```
//!
//! The stub plays the model: it answers the first call with a plan and a file
//! request, then returns `ui/mod.rs` with `guild_shell`'s columns swapped. The
//! rest — parsing, the allowlist, backup, build, staging — is the real code.

use std::sync::atomic::{AtomicUsize, Ordering};

use super::{install, patch, provider, run_with};

const GOAL: &str = "Move the channel sidebar to the right of the chat.";

#[test]
#[ignore = "builds the release binary"]
fn a_canned_edit_builds_and_stages() {
    let root = patch::source_root().expect("source tree");
    // `install::stage` keeps the running binary as "base". Under a test that is
    // the test harness, which must never become the fallback the app restarts
    // into, so seed the real previous build first.
    seed_base(&root);

    let calls = std::sync::Arc::new(AtomicUsize::new(0));
    let counted = calls.clone();
    let ask = move |_cfg: &provider::Config, _messages: &[(String, String)]| {
        if counted.fetch_add(1, Ordering::SeqCst) == 0 {
            return Ok(format!(
                "<<<PLAN\nMove the channel sidebar to the right of the chat area.\n\
                 Keep the resize handle on the sidebar's left edge.\n\
                 Desktop shell only; keep the rest as it is.\n>>>\n\
                 <<<NEED crates/client/src/ui/mod.rs>>>\n"
            ));
        }
        let text = std::fs::read_to_string(root.join("crates/client/src/ui/mod.rs"))
            .expect("read ui/mod.rs");
        let changed = swap_shell(&text);
        Ok(format!(
            "<<<PLAN\nSidebar now sits after the chat column; the handle's direction is flipped.\n>>>\n\
             <<<FILE crates/client/src/ui/mod.rs\n{changed}\n>>>\n"
        ))
    };
    let config = provider::Config {
        base_url: "http://stub.invalid/v1".into(),
        model: "stub".into(),
        key: "stub".into(),
    };

    let version = run_with(GOAL, &config, &ask).expect("the edit should build and stage");
    let exe = install::version_exe(&version).expect("staged binary exists");
    println!("STAGED {} at {}", version, exe.display());
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "plan round then patch round"
    );
    assert!(exe.exists());
}

/// Runs a reply supplied on disk through parse → backup → write → build →
/// stage, with a canned plan on the first turn. This is how a real session is
/// rehearsed when no model key is available.
///
/// ```text
/// RUNECORD_CANNED_REPLY=%TEMP%\reply.txt RUNECORD_CANNED_GOAL="..." ^
///   cargo test -p runecord --lib aiedit::tests::a_supplied_reply_builds_and_stages -- --ignored --nocapture
/// ```
#[test]
#[ignore = "builds the release binary"]
fn a_supplied_reply_builds_and_stages() {
    let Ok(path) = std::env::var("RUNECORD_CANNED_REPLY") else {
        return;
    };
    let reply = std::fs::read_to_string(&path).expect("readable canned reply");
    let root = patch::source_root().expect("source tree");
    seed_base(&root);

    let calls = std::sync::Arc::new(AtomicUsize::new(0));
    let counted = calls.clone();
    let ask = move |_cfg: &provider::Config, _messages: &[(String, String)]| {
        if counted.fetch_add(1, Ordering::SeqCst) == 0 {
            return Ok(
                "<<<PLAN\nApply the supplied patch to the rail and the DM shell.\n>>>\n\
                      <<<NEED crates/client/src/ui/rail.rs>>>\n"
                    .to_string(),
            );
        }
        Ok(reply.clone())
    };
    let goal = std::env::var("RUNECORD_CANNED_GOAL")
        .unwrap_or_else(|_| "Move the DM conversations into the server rail.".to_string());
    let config = provider::Config {
        base_url: "http://stub.invalid/v1".into(),
        model: "stub".into(),
        key: "stub".into(),
    };
    let version = run_with(&goal, &config, &ask).expect("the supplied reply should build");
    let exe = install::version_exe(&version).expect("staged binary exists");
    println!("STAGED {} at {}", version, exe.display());
    assert!(exe.exists());
}

/// The previous release binary, so the test does not stage itself as `base`.
fn seed_base(root: &std::path::Path) {
    let Ok(dir) = install::root() else {
        return;
    };
    let base = dir.join("versions").join("base");
    let target = base.join(if cfg!(windows) {
        "runecord.exe"
    } else {
        "runecord"
    });
    if target.exists() {
        return;
    }
    let Ok(_) = std::fs::create_dir_all(&base) else {
        return;
    };
    let built = root.join("target").join("release").join(if cfg!(windows) {
        "runecord.exe"
    } else {
        "runecord"
    });
    if built.exists() {
        let _ = std::fs::copy(&built, &target);
    }
    let mut state = install::state();
    if state.last_good.is_empty() {
        state.last_good = "base".into();
    }
    if state.current.is_empty() {
        state.current = "base".into();
    }
    let _ = std::fs::write(
        dir.join("state.json"),
        serde_json::to_vec_pretty(&state).unwrap_or_default(),
    );
}

/// The model's intended edit, applied as a string so the test fails loudly if
/// `guild_shell` no longer has the shape it assumes.
fn swap_shell(text: &str) -> String {
    // Re-runnable: a previous run may have left the tree already swapped, in
    // which case the build still exercises the pipeline with an unchanged file.
    if text.contains(
        "        .child(chat(s, cx, p))\n        .child(resize_handle(\n            \"sidebar-resize\",\n            -1.0,",
    ) {
        return text.to_string();
    }
    let before = "        .child(rail(s, p))\n        .child(sidebar(s, p))\n        .child(resize_handle(\n            \"sidebar-resize\",\n            1.0,";
    let after = "        .child(rail(s, p))\n        .child(chat(s, cx, p))\n        .child(resize_handle(\n            \"sidebar-resize\",\n            -1.0,";
    assert_eq!(
        text.matches(before).count(),
        1,
        "guild_shell no longer starts with rail, sidebar, handle"
    );
    let swapped = text.replace(before, after);
    let tail_before = "        ))\n        .child(chat(s, cx, p));";
    let tail_after = "        ))\n        .child(sidebar(s, p));";
    assert_eq!(
        swapped.matches(tail_before).count(),
        1,
        "the chat column is no longer the last child of guild_shell"
    );
    swapped.replacen(tail_before, tail_after, 1)
}

#[test]
#[ignore = "writes the API key into the OS credential store"]
fn imports_the_key_from_the_environment() {
    let Ok(key) = std::env::var("RUNECORD_AI_KEY") else {
        return;
    };
    super::secret::save(&key).expect("save the key");
    let loaded = super::secret::load().expect("read the key back");
    println!("key stored and read back: {} chars", loaded.len());
    assert_eq!(loaded.trim(), key.trim());
}

#[test]
#[ignore = "calls the configured provider over the network"]
fn the_key_test_reports_the_provider_answer() {
    let base = std::env::var("RUNECORD_AI_URL")
        .unwrap_or_else(|_| "https://opencode.ai/inference/go/openai/v1".to_string());
    let model =
        std::env::var("RUNECORD_AI_MODEL").unwrap_or_else(|_| "deepseek-v4.1-flash".to_string());
    match super::test_key(&base, &model) {
        Ok(reply) => println!("KEY TEST OK: {reply}"),
        Err(error) => println!("KEY TEST ERR: {error}"),
    }
}

#[test]
#[ignore = "calls the configured provider and builds the app"]
fn a_live_edit_builds() {
    // The goal can come from a file so its text never appears in this process's
    // output (used when the prompt carries a secret seed).
    let goal = std::env::var("RUNECORD_AI_GOAL").ok().or_else(|| {
        std::env::var("RUNECORD_AI_GOAL_FILE")
            .ok()
            .and_then(|path| std::fs::read_to_string(path).ok())
    });
    let Some(goal) = goal else {
        return;
    };
    let base_url = std::env::var("RUNECORD_AI_URL")
        .unwrap_or_else(|_| "https://opencode.ai/inference/go/openai/v1".to_string());
    let model =
        std::env::var("RUNECORD_AI_MODEL").unwrap_or_else(|_| "deepseek-v4.1-flash".to_string());
    let key = super::secret::load().expect("a saved API key");
    println!(
        "goal: <{} chars, redacted>\nmodel: {model} at {base_url}",
        goal.chars().count()
    );
    let config = provider::Config {
        base_url,
        model,
        key,
    };
    let result = super::run(&goal, &config, &[], None);
    for event in super::take() {
        match event {
            super::Event::Stage(stage) => println!("STAGE {stage:?}"),
            super::Event::Line(line) => println!("LINE {line}"),
            super::Event::Brief { goal, text } => println!(
                "BRIEF for <{} chars> ({} lines)",
                goal.chars().count(),
                text.lines().count()
            ),
        }
    }
    match result {
        Ok(version) => println!("STAGED {version}"),
        Err(error) => println!("FAILED {error}"),
    }
}

#[test]
#[ignore = "writes the provider settings file"]
fn provider_settings_round_trip() {
    let before = super::install::load_provider().map(|p| p.base_url);
    super::install::save_provider("https://example.test/v1", "test-model").expect("save provider");
    let back = super::install::load_provider().expect("read provider back");
    assert_eq!(back.base_url, "https://example.test/v1");
    assert_eq!(back.model, "test-model");
    println!("provider round trip ok; previous: {before:?}");
}

/// Simulates a pruned binary and rebuilds it from its source snapshot. Only
/// runs against a throwaway `RUNECORD_DATA` when `RUNECORD_HYBRID_TEST=1`.
#[test]
#[ignore = "rebuilds from a source snapshot; needs a warm target"]
fn a_source_only_version_rebuilds() {
    if std::env::var_os("RUNECORD_HYBRID_TEST").is_none() {
        return;
    }
    let Some(version) = super::install::versions()
        .into_iter()
        .find(|version| version.source)
    else {
        return;
    };
    let exe = super::install::version_exe(&version.name).expect("staged binary");
    std::fs::remove_file(&exe).expect("remove the binary to simulate pruning");
    let rebuilt = super::rebuild_run(&version.name);
    assert!(rebuilt.is_ok(), "{rebuilt:?}");
    assert!(super::install::version_exe(&version.name).is_ok());
}

#[test]
fn history_shows_the_request_not_the_brief_heading() {
    let composed = "User request:\nadd a pink mascot\n\nImplementation brief\nINTENT — ...";
    assert_eq!(super::stage_label(composed), "add a pink mascot");
    assert_eq!(super::stage_label("move the sidebar"), "move the sidebar");
}

/// Promotes a manually verified pending build, records the current source, and
/// prunes every other staged executable on the real per-user store.
///
/// ```text
/// RUNECORD_BASE_EXE=... RUNECORD_HYBRID_PROMOTE=1 ^
///   cargo test -p runecord --lib aiedit::tests::prepare_and_clean_hybrid_store -- --ignored --nocapture
/// ```
#[test]
#[ignore = "promotes and prunes the real per-user store"]
fn prepare_and_clean_hybrid_store() {
    if std::env::var_os("RUNECORD_HYBRID_PROMOTE").is_some()
        && let Some(pending) = super::install::state().pending.clone()
        && super::install::version_exe(&pending).is_ok()
    {
        super::source::snapshot(&pending, "Hello Kitty theme")
            .expect("snapshot the current source");
        super::install::promote_pending().expect("promote the pending build");
    }
    let report = super::install::cleanup_versions().expect("cleanup the version store");
    println!("kept: {:?}", report.kept);
    println!("removed: {:?}", report.removed);
    println!("freed: {:.1} MB", report.freed as f64 / (1024.0 * 1024.0));
    for version in super::install::versions() {
        println!(
            "version {} binary={} source={} goal={}",
            version.name, version.binary, version.source, version.goal
        );
    }
}

#[test]
fn deleting_ui_files_clears_builds_snapshots_and_rollbacks() {
    let dir = std::env::temp_dir().join(format!("runecord-aiedit-delete-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for folder in ["versions/ui-1", "source/versions/ui-1", "backup/1"] {
        let path = dir.join(folder);
        std::fs::create_dir_all(&path).expect("create a store folder");
        std::fs::write(path.join("file.txt"), b"edit").expect("write a stored file");
    }
    std::fs::write(dir.join("provider.json"), b"{}").expect("write provider settings");

    let freed = install::delete_ui_files(&dir).expect("delete the ui files");

    assert!(freed > 0, "the report counts the bytes freed");
    assert!(!dir.join("versions").exists(), "builds are gone");
    assert!(!dir.join("source").exists(), "source copies are gone");
    assert!(!dir.join("backup").exists(), "rollbacks are gone");
    assert!(
        dir.join("provider.json").exists(),
        "provider settings are not UI files and stay"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_swap_only_reorders_the_shell() {
    let shell = "        .child(rail(s, p))\n        .child(sidebar(s, p))\n        .child(resize_handle(\n            \"sidebar-resize\",\n            1.0,\n            180.0,\n            460.0,\n            |s: &mut App| &mut s.sidebar_width,\n            p,\n        ))\n        .child(chat(s, cx, p));";
    let swapped = swap_shell(shell);
    let rail = swapped.find(".child(rail(s, p))").unwrap();
    let chat = swapped.find(".child(chat(s, cx, p))").unwrap();
    let sidebar = swapped.find(".child(sidebar(s, p))").unwrap();
    assert!(
        rail < chat && chat < sidebar,
        "order is rail, chat, sidebar"
    );
    assert!(swapped.contains("-1.0"), "handle direction is flipped");
}
