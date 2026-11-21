//! Store-level end-to-end test for the AI UI editor.
//!
//! No model, no release build, no process spawn: a dummy executable stands in
//! for the compiled binary and the real store functions handle staging,
//! promotion, pruning and the failure note. This is the cheap check that the
//! build → stage → restart → fallback state machine keeps its promises.
//!
//! This file is its own test binary, so the `RUNECORD_DATA` and
//! `RUNECORD_UI_SRC` overrides cannot affect another test.

use std::path::{Path, PathBuf};
use std::time::Duration;

use runecord_client::aiedit::install;

fn temp_dir(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "runecord-aiedit-store-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&path).expect("create the temp directory");
    path
}

fn write_source(root: &Path) {
    std::fs::write(root.join("Cargo.toml"), b"[workspace]\n").expect("write the source manifest");
    for (path, body) in [
        ("crates/client/src/ui/one.rs", "fn one() {}\n"),
        ("crates/ui/src/two.rs", "fn two() {}\n"),
    ] {
        let target = root.join(path);
        std::fs::create_dir_all(target.parent().expect("a parent")).expect("create source dirs");
        std::fs::write(target, body).expect("write a source file");
    }
}

fn exe_name() -> &'static str {
    if cfg!(windows) {
        "runecord.exe"
    } else {
        "runecord"
    }
}

fn stage(dummy: &Path, goal: &str) -> String {
    // The stamp is milliseconds; keep consecutive versions distinct.
    std::thread::sleep(Duration::from_millis(2));
    install::stage(
        dummy,
        goal,
        &["crates/client/src/ui/one.rs".to_string()],
        "test-backup",
    )
    .expect("stage the dummy build")
}

#[test]
fn a_build_is_staged_promoted_falls_back_and_keeps_its_source() {
    let data = temp_dir("data");
    let source = temp_dir("source");
    let bin = temp_dir("bin");
    write_source(&source);

    // SAFETY: this test binary contains one test, and nothing reads these
    // variables before the store is opened below.
    unsafe {
        std::env::set_var("RUNECORD_DATA", &data);
        std::env::set_var("RUNECORD_UI_SRC", &source);
    }

    let dummy = bin.join(exe_name());
    std::fs::write(&dummy, b"not a real executable").expect("write the dummy binary");
    let store = data.join("aiedit");

    let first = stage(&dummy, "first");
    assert_eq!(
        install::state().pending.as_deref(),
        Some(first.as_str()),
        "a build is pending until it starts"
    );
    assert!(install::version_exe(&first).is_ok(), "the binary is copied");
    assert!(install::has_source(&first), "the source is snapshotted");

    install::promote_pending().expect("promote the first build");
    assert_eq!(install::state().current, first);
    assert!(install::state().last_good.is_empty());

    let second = stage(&dummy, "second");
    install::promote_pending().expect("promote the second build");
    assert_eq!(install::state().current, second);
    assert_eq!(install::state().last_good, first);

    let third = stage(&dummy, "third");
    install::promote_pending().expect("promote the third build");
    install::prune_now();
    assert_eq!(install::state().current, third);
    assert_eq!(install::state().last_good, second);
    assert!(
        install::version_exe(&first).is_err(),
        "the old last-good executable is pruned"
    );
    assert!(
        install::has_source(&first),
        "its source manifest survives the pruned binary"
    );
    let first_version = install::versions()
        .into_iter()
        .find(|version| version.name == first)
        .expect("the source-only version is still listed");
    assert!(!first_version.binary && first_version.source);

    let fourth = stage(&dummy, "fourth");
    install::record_failure("the new UI did not paint");
    install::prune_now();
    assert_eq!(
        install::state().pending,
        None,
        "the failed build is not pending"
    );
    assert_eq!(
        install::state().current,
        third,
        "the previous build stays current"
    );
    assert_eq!(install::state().last_good, second);
    assert!(
        install::version_exe(&fourth).is_err(),
        "the failed build's executable is pruned"
    );
    assert!(
        install::has_source(&fourth),
        "its source remains for a rebuild"
    );
    assert_eq!(
        install::take_failure().as_deref(),
        Some("the new UI did not paint"),
        "the next launch can explain the fallback"
    );

    // A UI built for another app version warns until the user acknowledges it.
    let meta_path = store.join("versions").join(&third).join("meta.json");
    let mut meta: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&meta_path).expect("read the version meta"))
            .expect("parse the version meta");
    meta["app_version"] = serde_json::Value::String("0.0.1".into());
    std::fs::write(
        &meta_path,
        serde_json::to_vec_pretty(&meta).expect("serialize the version meta"),
    )
    .expect("write the version meta");
    assert_eq!(
        install::outdated_current().as_deref(),
        Some("0.0.1"),
        "an older build is reported before the update is acknowledged"
    );
    install::keep_current_version().expect("acknowledge the old build");
    assert_eq!(
        install::outdated_current(),
        None,
        "keep silences the warning for this app version"
    );

    let _ = std::fs::remove_dir_all(&data);
    let _ = std::fs::remove_dir_all(&source);
    let _ = std::fs::remove_dir_all(&bin);
}
