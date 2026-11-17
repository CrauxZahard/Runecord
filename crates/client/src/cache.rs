//! Warm-start snapshot cache.
//!
//! The client is authoritative for nothing, so a snapshot is disposable: it is
//! painted immediately on launch, then replaced by the first gateway/bootstrap
//! payload. Files are written off the UI thread and atomically replaced.

use runecord_protocol::{Bootstrap, GuildDetail, Id, Message};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock, mpsc};

pub const CACHE_VERSION: u32 = 1;
/// Newest messages kept per channel after trimming.
pub const MAX_MESSAGES_PER_CHANNEL: usize = 500;
/// Total messages kept across all channels before least-recently-used lists go.
pub const MAX_TOTAL_MESSAGES: usize = 5_000;
/// Channels included in a disk snapshot.
pub const SNAPSHOT_CHANNELS: usize = 8;
/// Messages per channel included in a disk snapshot.
pub const SNAPSHOT_MESSAGES_PER_CHANNEL: usize = 250;
/// Loaded members kept per guild.
pub const MAX_MEMBERS_PER_GUILD: usize = 300;
/// Guilds whose member pages stay resident.
pub const MAX_MEMBER_GUILDS: usize = 8;
/// Guild details kept resident.
pub const MAX_GUILD_DETAILS: usize = 6;
/// A snapshot older than this is ignored instead of painted.
const MAX_SNAPSHOT_AGE_MS: i64 = 14 * 24 * 60 * 60 * 1_000;

/// One channel's retained history inside a snapshot.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CachedChannel {
    pub channel_id: Id,
    pub messages: Vec<Message>,
}

/// Everything needed to paint the shell before the server answers.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: u32,
    pub saved_at: i64,
    pub bootstrap: Bootstrap,
    #[serde(default)]
    pub details: Vec<GuildDetail>,
    #[serde(default)]
    pub messages: Vec<CachedChannel>,
    pub active_guild: Id,
    pub active_channel: Id,
    pub active_dm: Id,
    pub dm_home: bool,
    #[serde(default)]
    pub dm_screen: bool,
    #[serde(default)]
    pub last_guild_channel: HashMap<Id, Id>,
    #[serde(default = "default_true")]
    pub notify_dm: bool,
    #[serde(default = "default_true")]
    pub notify_mentions: bool,
    #[serde(default)]
    pub compact: bool,
    #[serde(default)]
    pub developer: bool,
    #[serde(default)]
    pub keybinds: BTreeMap<String, String>,
    #[serde(default)]
    pub video_quality: String,
    #[serde(default)]
    pub view_quality: String,
    #[serde(default)]
    pub voice_input_device: String,
    #[serde(default)]
    pub voice_output_device: String,
    #[serde(default)]
    pub voice_camera_device: String,
    #[serde(default = "default_volume")]
    pub voice_input_volume: f32,
    #[serde(default = "default_volume")]
    pub voice_output_volume: f32,
    #[serde(default = "default_sensitivity")]
    pub voice_sensitivity: f32,
    #[serde(default = "default_true")]
    pub voice_auto_sensitivity: bool,
    #[serde(default = "default_voice_mode")]
    pub voice_mode: String,
    #[serde(default)]
    pub voice_noise_suppression: bool,
    #[serde(default = "default_ptt_key")]
    pub ptt_key: u32,
}

fn default_ptt_key() -> u32 {
    0x77
}

fn default_volume() -> f32 {
    1.0
}

fn default_sensitivity() -> f32 {
    0.02
}

fn default_voice_mode() -> String {
    "voice".to_string()
}

fn default_true() -> bool {
    true
}

struct Job {
    path: PathBuf,
    snapshot: Snapshot,
    epoch: u64,
}

static EPOCH: AtomicU64 = AtomicU64::new(0);
static FILE_LOCK: Mutex<()> = Mutex::new(());
static WRITER: OnceLock<mpsc::Sender<Job>> = OnceLock::new();

fn cache_path(profile: &str) -> Option<PathBuf> {
    Some(crate::paths::data_dir()?.join(format!("cache-{profile}.json")))
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Load a snapshot for a profile. Returns `None` for missing, old, or corrupt files.
pub fn load(profile: &str) -> Option<Snapshot> {
    let path = cache_path(profile)?;
    let bytes = std::fs::read(path).ok()?;
    let snapshot: Snapshot = serde_json::from_slice(&bytes).ok()?;
    if snapshot.version != CACHE_VERSION {
        return None;
    }
    if now_ms().saturating_sub(snapshot.saved_at) > MAX_SNAPSHOT_AGE_MS {
        return None;
    }
    Some(snapshot)
}

fn writer() -> &'static mpsc::Sender<Job> {
    WRITER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Job>();
        std::thread::spawn(move || {
            for job in rx {
                let _guard = FILE_LOCK.lock().ok();
                // A logout (or profile switch) bumps the epoch and invalidates
                // anything that was queued before it.
                if job.epoch != EPOCH.load(Ordering::SeqCst) {
                    continue;
                }
                let Ok(bytes) = serde_json::to_vec(&job.snapshot) else {
                    continue;
                };
                let Some(dir) = job.path.parent() else {
                    continue;
                };
                if std::fs::create_dir_all(dir).is_err() {
                    continue;
                }
                let tmp = job.path.with_extension("json.tmp");
                if std::fs::write(&tmp, bytes).is_err() {
                    continue;
                }
                // Windows rename does not reliably replace an existing file.
                let _ = std::fs::remove_file(&job.path);
                let _ = std::fs::rename(&tmp, &job.path);
            }
        });
        tx
    })
}

/// Queue a snapshot write. Serialization and disk IO happen on the writer thread.
///
/// A browser has no filesystem and cannot spawn a thread, so the web build
/// keeps everything in memory and skips the disk cache entirely.
#[cfg(target_arch = "wasm32")]
pub fn save(_profile: &str, _snapshot: Snapshot) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn save(profile: &str, snapshot: Snapshot) {
    let Some(path) = cache_path(profile) else {
        return;
    };
    let job = Job {
        path,
        snapshot,
        epoch: EPOCH.load(Ordering::SeqCst),
    };
    let _ = writer().send(job);
}

/// Drop the cached snapshot, including any writes still queued.
pub fn clear(profile: &str) {
    let _guard = FILE_LOCK.lock().ok();
    EPOCH.fetch_add(1, Ordering::SeqCst);
    if let Some(path) = cache_path(profile) {
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("json.tmp"));
    }
}
