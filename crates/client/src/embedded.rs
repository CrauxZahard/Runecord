//! Single-executable packaging.
//!
//! Built with the `embedded-server` feature, the client hosts the backend in
//! its own process on the configured loopback port. Double-clicking the binary
//! is enough: there is no launcher, no child process and no console window.
//!
//! The default build keeps the two-process model and expects `runecord-server`
//! to be started separately.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// How long a single health probe may take before the server is treated as down.
const HEALTH_TIMEOUT: Duration = Duration::from_millis(500);
/// How long startup waits for the embedded server before letting the UI report it.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
/// Delay between startup health probes.
const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Starts the backend in this process when the configured endpoint is a local
/// address that is not already serving Runecord.
pub fn ensure_backend() {
    let url = crate::config::effective_server_url();
    let Some(client) = health_client() else {
        return;
    };
    if healthy(&client, &url) {
        return;
    }
    let Some((host, port)) = loopback(&url) else {
        return;
    };

    let data_dir = server_data_dir();
    init_logging(&data_dir);
    tracing::info!(
        "starting embedded server on {url} with data in {}",
        data_dir.display()
    );

    let thread_data_dir = data_dir.clone();
    let spawned = std::thread::Builder::new()
        .name("runecord-server".into())
        .spawn(move || {
            let config = runecord_server::config::Config {
                host,
                port,
                data_dir: thread_data_dir,
                demo_seed: true,
            };
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    tracing::error!("could not start the embedded server runtime: {error}");
                    return;
                }
            };
            runtime.block_on(runecord_server::bootstrap::run(config));
        });

    if let Err(error) = spawned {
        tracing::error!("could not start the embedded server thread: {error}");
        return;
    }

    let deadline = Instant::now() + STARTUP_TIMEOUT;
    while Instant::now() < deadline {
        if healthy(&client, &url) {
            tracing::info!("embedded server is ready");
            return;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
    tracing::error!("embedded server did not answer at {url}; opening the UI anyway");
}

fn health_client() -> Option<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(HEALTH_TIMEOUT)
        .build()
        .ok()
}

fn healthy(client: &reqwest::blocking::Client, url: &str) -> bool {
    client
        .get(format!("{url}/api/version"))
        .send()
        .ok()
        .and_then(|response| response.json::<serde_json::Value>().ok())
        .is_some_and(|value| value.get("name").and_then(|name| name.as_str()) == Some("runecord"))
}

/// Extracts a bindable loopback address from the configured URL.
///
/// Only literal loopback addresses are embedded; `localhost` and remote URLs
/// keep the normal "start a server yourself" behaviour.
fn loopback(url: &str) -> Option<(IpAddr, u16)> {
    let rest = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))?;
    let authority = rest.split('/').next()?;

    if let Some(stripped) = authority.strip_prefix('[') {
        let (host, port) = stripped.split_once("]:")?;
        if host != "::1" {
            return None;
        }
        return Some((IpAddr::V6(Ipv6Addr::LOCALHOST), port.parse().ok()?));
    }

    let (host, port) = authority.rsplit_once(':')?;
    if host != "127.0.0.1" {
        return None;
    }
    Some((IpAddr::V4(Ipv4Addr::LOCALHOST), port.parse().ok()?))
}

/// Persistent data directory for the bundled server.
fn server_data_dir() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .or_else(|_| std::env::current_dir())
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("Runecord").join("server")
}

/// Sends server logs to `server.log` next to the embedded data.
fn init_logging(data_dir: &Path) {
    if std::fs::create_dir_all(data_dir).is_err() {
        return;
    }
    let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(data_dir.join("server.log"))
    else {
        return;
    };
    let writer = LogFile(Arc::new(Mutex::new(file)));
    let _ = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(writer)
        .try_init();
}

/// Cloneable file writer for the tracing subscriber.
#[derive(Clone)]
struct LogFile(Arc<Mutex<std::fs::File>>);

impl std::io::Write for LogFile {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        match self.0.lock() {
            Ok(mut file) => file.write(buffer),
            Err(_) => Ok(buffer.len()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.0.lock() {
            Ok(mut file) => file.flush(),
            Err(_) => Ok(()),
        }
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogFile {
    type Writer = LogFile;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}
