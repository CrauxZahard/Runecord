//! The installer puts the build tree at `<install>\src\Runecord`. This is a
//! separate test binary so the env override cannot affect another test.

use std::path::PathBuf;

use runecord_client::aiedit::install;

fn temp_dir(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "runecord-aiedit-source-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&path).expect("create the temp directory");
    path
}

#[test]
fn the_installed_source_bundle_is_found_next_to_the_base_exe() {
    let app = temp_dir("app");
    let exe = app.join("Runecord.exe");
    std::fs::write(&exe, b"not a real executable").expect("write the fake base exe");

    let workspace = app.join("src").join("Runecord");
    std::fs::create_dir_all(workspace.join("crates/client/src/ui"))
        .expect("create the bundled workspace");
    std::fs::write(workspace.join("Cargo.toml"), b"[workspace]\n")
        .expect("write the bundled manifest");

    // SAFETY: this test binary contains one test. The variables are set before
    // the source lookup and no other test reads them.
    unsafe {
        std::env::set_var("RUNECORD_BASE_EXE", &exe);
        std::env::remove_var("RUNECORD_UI_SRC");
    }

    let found = install::source_root().expect("the installed source bundle");
    assert_eq!(found, workspace);

    let _ = std::fs::remove_dir_all(&app);
}
