//! Per-install client configuration.
//!
//! Runecord is self-hosted, so the backend endpoint is not baked into the
//! binary. It is stored next to the session files and read once at launch;
//! `RUNECORD_URL` still overrides it for scripted runs.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_SERVER_URL: &str = "http://127.0.0.1:47800";

#[derive(Default, Serialize, Deserialize)]
struct Config {
    #[serde(default)]
    server_url: String,
}

fn config_path() -> Option<PathBuf> {
    let base = std::env::var("APPDATA")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())?;
    Some(base.join("Runecord").join("config.json"))
}

/// The URL stored by the Application settings, falling back to the local server.
pub fn stored_server_url() -> String {
    startup().url
}

pub struct Startup {
    pub url: String,
    pub error: Option<String>,
}

/// Reads the saved endpoint once at launch. A hand-edited invalid value falls
/// back to the local default but is reported so the auth screen can explain
/// why the client is not connected.
pub fn startup() -> Startup {
    let Some(path) = config_path() else {
        return Startup {
            url: DEFAULT_SERVER_URL.to_string(),
            error: None,
        };
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return Startup {
            url: DEFAULT_SERVER_URL.to_string(),
            error: None,
        };
    };
    let config: Config = match serde_json::from_str(&text) {
        Ok(config) => config,
        Err(_) => {
            return Startup {
                url: DEFAULT_SERVER_URL.to_string(),
                error: Some(
                    "The saved server setting can't be read; using the local default.".into(),
                ),
            };
        }
    };
    if config.server_url.trim().is_empty() {
        return Startup {
            url: DEFAULT_SERVER_URL.to_string(),
            error: None,
        };
    }
    match normalize(&config.server_url) {
        Ok(url) => Startup { url, error: None },
        Err(reason) => Startup {
            url: DEFAULT_SERVER_URL.to_string(),
            error: Some(format!(
                "The saved server URL “{}” is invalid ({reason}). Using the local default.",
                config.server_url.trim()
            )),
        },
    }
}

/// The URL this process should use. `RUNECORD_URL` wins so harness runs can
/// target a throwaway server without touching the saved preference.
pub fn effective_server_url() -> String {
    std::env::var("RUNECORD_URL")
        .ok()
        .and_then(|value| normalize(&value).ok())
        .unwrap_or_else(stored_server_url)
}

/// Effective URL plus the saved-file error, for the startup screen.
pub fn startup_effective() -> Startup {
    if let Ok(value) = std::env::var("RUNECORD_URL")
        && let Ok(url) = normalize(&value)
    {
        return Startup { url, error: None };
    }
    startup()
}

/// Whether the app should start the bundled backend. A configured remote
/// deployment must not spawn a second server on this machine.
pub fn uses_bundled_server(url: &str) -> bool {
    normalize(url).is_ok_and(|url| url == DEFAULT_SERVER_URL)
}

/// Quick startup TCP probe. It does not authenticate; it only distinguishes a
/// refused or unroutable endpoint from one that is listening, so the auth screen
/// can say the client can't connect instead of waiting forever.
pub fn server_reachable(url: &str) -> bool {
    use std::net::{TcpStream, ToSocketAddrs};
    let Some((host, port)) = endpoint(url) else {
        return false;
    };
    let Ok(addresses) = (host.as_str(), port).to_socket_addrs() else {
        return false;
    };
    addresses.into_iter().any(|address| {
        TcpStream::connect_timeout(&address, std::time::Duration::from_millis(600)).is_ok()
    })
}

fn endpoint(url: &str) -> Option<(String, u16)> {
    let (secure, rest) = if let Some(rest) = url.strip_prefix("https://") {
        (true, rest)
    } else {
        let rest = url.strip_prefix("http://")?;
        (false, rest)
    };
    let authority = rest.split('/').next()?;
    let (host, port) = if let Some(bracketed) = authority.strip_prefix('[') {
        let (host, after) = bracketed.split_once(']')?;
        let port = after.strip_prefix(':');
        (host.to_string(), port)
    } else if let Some((host, port)) = authority.rsplit_once(':') {
        (host.to_string(), Some(port))
    } else {
        (authority.to_string(), None)
    };
    let port = match port {
        Some(port) => port.parse().ok()?,
        None if secure => 443,
        None => 80,
    };
    Some((host, port))
}

pub fn normalize(input: &str) -> Result<String, String> {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("Enter a server URL.".into());
    }
    let rest = trimmed
        .strip_prefix("http://")
        .or_else(|| trimmed.strip_prefix("https://"));
    let Some(rest) = rest else {
        return Err("The server URL must start with http:// or https://.".into());
    };
    if rest.is_empty() || rest.chars().any(char::is_whitespace) {
        return Err("Enter a host after the scheme, with no spaces.".into());
    }
    let (authority, _path) = match rest.split_once('/') {
        Some((authority, path)) => (authority, Some(path)),
        None => (rest, None),
    };
    if authority.is_empty() || authority.contains("://") {
        return Err("Enter one server URL, not two joined together.".into());
    }
    if authority.contains('@') {
        return Err("Do not put credentials in the server URL.".into());
    }
    let (host, port) = if let Some(bracketed) = authority.strip_prefix('[') {
        let Some((host, after)) = bracketed.split_once(']') else {
            return Err("Close the IPv6 address with ].".into());
        };
        let port = match after.strip_prefix(':') {
            Some(port) => Some(port),
            None if after.is_empty() => None,
            None => return Err("Only a port may follow the IPv6 address.".into()),
        };
        (host, port)
    } else {
        match authority.rsplit_once(':') {
            Some((host, port)) if !host.contains(':') => (host, Some(port)),
            Some(_) => return Err("Put IPv6 addresses in [brackets].".into()),
            None => (authority, None),
        }
    };
    if host.is_empty()
        || !host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ':'))
    {
        return Err("The server URL has an invalid host.".into());
    }
    if let Some(port) = port {
        let valid = port.parse::<u16>().ok().is_some_and(|port| port != 0);
        if !valid {
            return Err("The server URL has an invalid port.".into());
        }
    }
    Ok(trimmed.to_string())
}

/// Saves the URL and clears saved sessions when it points at a different
/// server, because those tokens belong to the old backend.
pub fn save_server_url(input: &str) -> Result<(String, bool), String> {
    let url = normalize(input)?;
    let previous = stored_server_url();
    let path = config_path().ok_or("Could not find the configuration folder.")?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|error| format!("Could not create the config folder: {error}"))?;
    }
    let encoded = serde_json::to_vec_pretty(&Config {
        server_url: url.clone(),
    })
    .map_err(|error| format!("Could not encode the configuration: {error}"))?;
    std::fs::write(&path, encoded)
        .map_err(|error| format!("Could not save the server URL: {error}"))?;
    let changed = previous != url;
    if changed {
        clear_sessions();
    }
    Ok((url, changed))
}

fn clear_sessions() {
    let Some(dir) = config_path().and_then(|path| path.parent().map(PathBuf::from)) else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("session-") && name.ends_with(".txt") {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_SERVER_URL, endpoint, normalize, uses_bundled_server};

    #[test]
    fn server_urls_are_normalized_and_validated() {
        assert_eq!(
            normalize("  http://127.0.0.1:47800/  ").unwrap(),
            "http://127.0.0.1:47800"
        );
        assert_eq!(
            normalize("https://chat.example.com/").unwrap(),
            "https://chat.example.com"
        );
        assert!(normalize("chat.example.com").is_err());
        assert!(normalize("ftp://chat.example.com").is_err());
        assert!(normalize("http://").is_err());
        assert!(normalize("http://two words").is_err());
        assert!(normalize("   ").is_err());
        assert!(normalize("http://127.0.0.1:47800https://chat.example.com").is_err());
        assert!(normalize("https://chat.example.com:0").is_err());
        assert!(normalize("https://chat.example.com:99999").is_err());
        assert!(normalize("https://user:pass@chat.example.com").is_err());
        assert_eq!(
            normalize("https://chat.example.com:8443/base").unwrap(),
            "https://chat.example.com:8443/base"
        );
        assert_eq!(
            normalize("http://[::1]:47800").unwrap(),
            "http://[::1]:47800"
        );
        assert_eq!(DEFAULT_SERVER_URL, "http://127.0.0.1:47800");
        assert!(uses_bundled_server("http://127.0.0.1:47800/"));
        assert!(!uses_bundled_server("https://chat.example.com"));
        assert!(!uses_bundled_server("http://127.0.0.1:47801"));
        assert_eq!(
            endpoint("https://chat.example.com"),
            Some(("chat.example.com".into(), 443))
        );
        assert_eq!(
            endpoint("http://127.0.0.1:47993/base"),
            Some(("127.0.0.1".into(), 47993))
        );
        assert_eq!(endpoint("http://[::1]:47800"), Some(("::1".into(), 47800)));
    }
}
