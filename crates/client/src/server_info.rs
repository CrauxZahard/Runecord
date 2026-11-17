//! Reads the server's public policy before signing in.
//!
//! The sign-in screen needs to know whether registration is open, code-gated
//! or closed. `/api/version` is unauthenticated, so one background request is
//! enough. Failures leave the default (`open`) and the server's own error is
//! shown if the user tries to register anyway.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    pub signup: String,
}

static STARTED: AtomicBool = AtomicBool::new(false);
static INFO: Mutex<Option<Info>> = Mutex::new(None);

pub fn start(base: &str) {
    if std::env::var("RUNECORD_NO_SERVER_PROBE").as_deref() == Ok("1") {
        return;
    }
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let base = base.trim_end_matches('/').to_string();
    std::thread::Builder::new()
        .name("runecord-server-info".into())
        .spawn(move || {
            let Ok(client) = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(4))
                .user_agent(format!("Runecord/{}", env!("CARGO_PKG_VERSION")))
                .build()
            else {
                return;
            };
            let Ok(response) = client.get(format!("{base}/api/version")).send() else {
                return;
            };
            if !response.status().is_success() {
                return;
            }
            let Ok(body) = response.json::<serde_json::Value>() else {
                return;
            };
            if body.get("name").and_then(|value| value.as_str()) != Some("runecord") {
                return;
            }
            let signup = body
                .get("signup")
                .and_then(|value| value.as_str())
                .unwrap_or("open")
                .to_string();
            if let Ok(mut slot) = INFO.lock() {
                *slot = Some(Info { signup });
            }
        })
        .ok();
}

pub fn take() -> Option<Info> {
    INFO.lock().ok().and_then(|mut slot| slot.take())
}
