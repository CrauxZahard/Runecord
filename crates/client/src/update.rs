//! Notify-only release check.
//!
//! The app asks GitHub for the newest published release and compares it with
//! the version it was built from. It never downloads or installs anything: an
//! available version becomes a toast and a row in Settings → Application with
//! a link to the release page.
//!
//! `RUNECORD_NO_UPDATE_CHECK=1` disables the launch check (air-gapped or
//! scripted runs). A failed check stays silent unless the user pressed the
//! button.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Where releases are published. Change this if the repository moves.
const RELEASES_API: &str = "https://api.github.com/repos/CrauxZahard/Runecord/releases/latest";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Idle,
    Checking,
    Latest,
    Available(Release),
    Failed(String),
}

static STARTED: AtomicBool = AtomicBool::new(false);
static RESULT: Mutex<Option<Status>> = Mutex::new(None);

/// Starts the background check once, or again when `force` is set.
pub fn start(force: bool) {
    if force {
        STARTED.store(false, Ordering::SeqCst);
    }
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    set(Status::Checking);
    std::thread::Builder::new()
        .name("runecord-update".into())
        .spawn(|| set(check()))
        .ok();
}

/// Takes the finished status, leaving `None` until the next check.
pub fn take() -> Option<Status> {
    let mut slot = RESULT.lock().ok()?;
    match slot.as_ref() {
        Some(Status::Checking) | None => None,
        Some(_) => slot.take(),
    }
}

fn set(status: Status) {
    if let Ok(mut slot) = RESULT.lock() {
        *slot = Some(status);
    }
}

fn check() -> Status {
    if std::env::var("RUNECORD_NO_UPDATE_CHECK").as_deref() == Ok("1") {
        return Status::Idle;
    }
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(6))
        .user_agent(format!("Runecord/{}", env!("CARGO_PKG_VERSION")))
        .build()
    {
        Ok(client) => client,
        Err(error) => return Status::Failed(error.to_string()),
    };
    let response = match client
        .get(RELEASES_API)
        .header("Accept", "application/vnd.github+json")
        .send()
    {
        Ok(response) => response,
        Err(error) => return Status::Failed(error.to_string()),
    };
    if !response.status().is_success() {
        return Status::Failed(format!(
            "The release feed answered HTTP {}.",
            response.status()
        ));
    }
    let body: serde_json::Value = match response.json() {
        Ok(body) => body,
        Err(error) => return Status::Failed(error.to_string()),
    };
    if body
        .get("draft")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return Status::Latest;
    }
    let tag = body
        .get("tag_name")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .trim_start_matches(['v', 'V'])
        .to_string();
    let url = body
        .get("html_url")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    if tag.is_empty() || url.is_empty() {
        return Status::Latest;
    }
    if is_newer(&tag, env!("CARGO_PKG_VERSION")) {
        Status::Available(Release { version: tag, url })
    } else {
        Status::Latest
    }
}

/// Numeric dotted comparison: `0.2.0` is newer than `0.1.9`. A suffix after
/// `-` (a prerelease) sorts below the plain version.
fn is_newer(candidate: &str, current: &str) -> bool {
    fn parts(value: &str) -> (Vec<u64>, bool) {
        let (numbers, prerelease) = match value.split_once('-') {
            Some((head, _)) => (head, true),
            None => (value, false),
        };
        (
            numbers
                .split('.')
                .map(|part| part.parse::<u64>().unwrap_or(0))
                .collect(),
            prerelease,
        )
    }
    let (next, next_pre) = parts(candidate);
    let (now, now_pre) = parts(current);
    let width = next.len().max(now.len());
    for index in 0..width {
        let a = next.get(index).copied().unwrap_or(0);
        let b = now.get(index).copied().unwrap_or(0);
        if a != b {
            return a > b;
        }
    }
    now_pre && !next_pre
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn compares_dotted_versions() {
        assert!(is_newer("0.2.0", "0.1.9"));
        assert!(is_newer("1.0", "0.9.9"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1", "0.1.0"));
        assert!(is_newer("0.1.1", "0.1.0"));
        // A prerelease still compares by its numbers, but sorts below the
        // plain release of the same version.
        assert!(is_newer("0.2.0-beta.1", "0.1.0"));
        assert!(is_newer("0.2.0", "0.2.0-rc.1"));
        assert!(!is_newer("0.2.0-rc.1", "0.2.0"));
    }
}
