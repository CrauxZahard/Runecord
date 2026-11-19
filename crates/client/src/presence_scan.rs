//! Automatic Rich Presence: match a running executable against a small catalog
//! and publish it as the user's activity.

/// Returns the activity kind and title for a known executable.
pub(crate) fn activity_for(process: &str) -> Option<(&'static str, &'static str)> {
    let name = process.trim().to_ascii_lowercase();
    let name = name.rsplit(['\\', '/']).next().unwrap_or(&name);
    let (kind, title) = match name {
        "rune.exe" => ("playing", "Rune"),
        "cs2.exe" => ("playing", "Counter-Strike 2"),
        "valorant.exe" => ("playing", "VALORANT"),
        "minecraft.exe" => ("playing", "Minecraft"),
        "factorio.exe" => ("playing", "Factorio"),
        "hades.exe" | "hades2.exe" => ("playing", "Hades"),
        "stardew valley.exe" => ("playing", "Stardew Valley"),
        "code.exe" => ("playing", "Visual Studio Code"),
        "spotify.exe" => ("spotify", "Spotify"),
        _ => return None,
    };
    Some((kind, title))
}

/// Names of the executables currently running, best effort.
///
/// Reads the process list directly instead of running `tasklist`: this scan
/// happens every fifteen seconds, and starting an executable that often for a
/// presence check costs a visible process creation each time.
#[cfg(windows)]
pub(crate) fn running_processes() -> Vec<String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::ProcessStatus::EnumProcesses;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::core::PWSTR;

    /// The longest image path worth reporting; a longer one is left out.
    const PATH_MAX: usize = 1024;

    // The call reports the pids that fit in the buffer; a fuller system asks
    // for another round with more room.
    let mut pids = vec![0u32; 1024];
    let count = loop {
        let mut bytes = 0u32;
        // SAFETY: the buffer holds `pids.len()` u32s and `bytes` is a valid out-pointer.
        if unsafe { EnumProcesses(pids.as_mut_ptr(), (pids.len() * 4) as u32, &mut bytes) }.is_err()
        {
            return Vec::new();
        }
        let count = bytes as usize / 4;
        if count < pids.len() {
            break count;
        }
        pids.resize(pids.len() * 2, 0);
    };

    let mut path = [0u16; PATH_MAX];
    let mut names = Vec::with_capacity(count);
    for &pid in &pids[..count] {
        // SAFETY: a plain handle request with nothing inherited; a process that
        // ended or that protects itself answers null, and is skipped.
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) };
        let Ok(handle) = handle else {
            continue;
        };
        let mut len = path.len() as u32;
        // SAFETY: the handle is open for this call, and the buffer is valid for
        // `len` UTF-16 units.
        let ok = unsafe {
            QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_WIN32,
                PWSTR(path.as_mut_ptr()),
                &mut len,
            )
        }
        .is_ok();
        // SAFETY: `handle` came from `OpenProcess` and is closed exactly once.
        unsafe { _ = CloseHandle(handle) };
        if ok {
            names.push(String::from_utf16_lossy(&path[..len as usize]));
        }
    }
    names
}

#[cfg(not(windows))]
pub(crate) fn running_processes() -> Vec<String> {
    let output = std::process::Command::new("ps")
        .args(["-A", "-o", "comm="])
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

/// The first catalog entry that is running, if any.
pub(crate) fn detect_activity() -> Option<(&'static str, &'static str)> {
    for process in running_processes() {
        if let Some(found) = activity_for(&process) {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::activity_for;

    #[test]
    fn known_executables_map_to_activities() {
        assert_eq!(activity_for("Rune.exe"), Some(("playing", "Rune")));
        assert_eq!(
            activity_for("cs2.exe"),
            Some(("playing", "Counter-Strike 2"))
        );
        assert_eq!(activity_for("Spotify.exe"), Some(("spotify", "Spotify")));
        assert_eq!(
            activity_for("C:\\Program Files\\Microsoft VS Code\\Code.exe"),
            Some(("playing", "Visual Studio Code"))
        );
        assert_eq!(activity_for("explorer.exe"), None);
    }

    /// The native enumeration must see this very process; `tasklist` did.
    #[cfg(windows)]
    #[test]
    fn process_list_includes_this_process() {
        let exe = std::env::current_exe().expect("a test executable");
        let own = exe.file_name().unwrap().to_string_lossy().to_lowercase();
        let names = super::running_processes();
        assert!(
            names.iter().any(|name| name.to_lowercase().ends_with(&own)),
            "{own} missing from {} processes",
            names.len()
        );
    }
}
