// Release builds are GUI apps: no console window and no `conhost.exe` child.
// Debug builds keep the console for logs. This keys off the profile, not
// `debug_assertions`, because the release profile enables assertions too.
#![cfg_attr(all(windows, release_build), windows_subsystem = "windows")]

fn main() {
    // The restart helper is a copy of this executable: it starts the staged
    // build, waits for its ready marker and falls back when it never appears.
    // It runs headless and exits, so it must not reach the app setup below.
    if runecord_client::aiedit::supervise_cli() {
        return;
    }
    // The installed app is also the launcher: with a staged current version it
    // starts that build (and exits), otherwise it runs its built-in UI.
    if runecord_client::aiedit::launch_current() {
        return;
    }
    runecord_client::run();
}
