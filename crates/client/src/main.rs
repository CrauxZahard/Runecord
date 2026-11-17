// Release builds are GUI apps: no console window and no `conhost.exe` child.
// Debug builds keep the console for logs. This keys off the profile, not
// `debug_assertions`, because the release profile enables assertions too.
#![cfg_attr(all(windows, release_build), windows_subsystem = "windows")]

fn main() {
    runecord_client::run();
}
