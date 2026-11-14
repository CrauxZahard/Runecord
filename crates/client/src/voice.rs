//! Voice audio transport.
//!
//! Capture and playback use FFmpeg's DirectShow/Opus support, while the app
//! exchanges the resulting Ogg/Opus stream over a small WebSocket relay.
//! `RUNECORD_VOICE_SYNTH=1` substitutes a generated tone for the microphone and
//! `RUNECORD_VOICE_TEST=1` decodes received audio to a log instead of speakers,
//! which lets two headless clients verify the transport without hardware.

use runecord_protocol::Id;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, connect};

/// Capture presets exposed in the voice stage. `Auto` starts at 720p and steps
/// down when the outbound frame queue backs up, then steps back up after a
/// stable period.
/// What the receiver asks to see. `Auto` starts high and drops to the low
/// simulcast layer if its decoder cannot keep up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewQuality {
    Auto,
    High,
    Low,
}

impl ViewQuality {
    pub fn label(self) -> &'static str {
        match self {
            ViewQuality::Auto => "Auto",
            ViewQuality::High => "High",
            ViewQuality::Low => "Data saver",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            ViewQuality::Auto => "auto",
            ViewQuality::High => "high",
            ViewQuality::Low => "low",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "high" => ViewQuality::High,
            "low" => ViewQuality::Low,
            _ => ViewQuality::Auto,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoQuality {
    P360,
    P480,
    P720,
    P1080,
    Auto,
}

impl VideoQuality {
    pub fn label(self) -> &'static str {
        match self {
            VideoQuality::P360 => "360p",
            VideoQuality::P480 => "480p",
            VideoQuality::P720 => "720p",
            VideoQuality::P1080 => "1080p",
            VideoQuality::Auto => "Auto",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            VideoQuality::P360 => "360p",
            VideoQuality::P480 => "480p",
            VideoQuality::P720 => "720p",
            VideoQuality::P1080 => "1080p",
            VideoQuality::Auto => "auto",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "360p" => VideoQuality::P360,
            "480p" => VideoQuality::P480,
            "720p" => VideoQuality::P720,
            "1080p" => VideoQuality::P1080,
            _ => VideoQuality::Auto,
        }
    }

    pub fn resolution(self) -> (u32, u32) {
        match self {
            VideoQuality::P360 => (640, 360),
            VideoQuality::P480 => (854, 480),
            VideoQuality::P720 | VideoQuality::Auto => (1280, 720),
            VideoQuality::P1080 => (1920, 1080),
        }
    }

    pub fn fps(self) -> u32 {
        match self {
            VideoQuality::P360 => 15,
            VideoQuality::P480 => 24,
            VideoQuality::P720 | VideoQuality::Auto => 30,
            VideoQuality::P1080 => 30,
        }
    }

    pub fn qscale(self) -> u32 {
        match self {
            VideoQuality::P360 => 8,
            VideoQuality::P480 => 6,
            VideoQuality::P720 | VideoQuality::Auto => 4,
            VideoQuality::P1080 => 3,
        }
    }

    pub fn target_kbps(self) -> u32 {
        match self {
            VideoQuality::P360 => 500,
            VideoQuality::P480 => 900,
            VideoQuality::P720 | VideoQuality::Auto => 1800,
            VideoQuality::P1080 => 3500,
        }
    }

    /// Auto only moves between 360p and 720p.
    fn step_down(self) -> Self {
        match self {
            VideoQuality::P1080 | VideoQuality::P720 => VideoQuality::P480,
            VideoQuality::P480 => VideoQuality::P360,
            other => other,
        }
    }

    fn step_up(self) -> Self {
        match self {
            VideoQuality::P360 => VideoQuality::P480,
            VideoQuality::P480 => VideoQuality::P720,
            other => other,
        }
    }
}

struct Session {
    channel: Id,
    stop: Arc<AtomicBool>,
    muted: Arc<AtomicBool>,
    deafened: Arc<AtomicBool>,
    video: mpsc::Sender<VideoCommand>,
}

struct VideoCommand {
    kind: Option<String>,
    quality: Option<VideoQuality>,
}

struct VideoFrame {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

struct VideoMeter {
    width: u32,
    height: u32,
    frames: u32,
    started: std::time::Instant,
    fps: u32,
    target: u32,
    slow: u32,
}

struct VideoDecoder {
    latest: Arc<Mutex<Option<Vec<u8>>>>,
    signal: mpsc::SyncSender<()>,
    stop: Arc<AtomicBool>,
}

static VIDEO_DECODERS: OnceLock<Mutex<HashMap<Id, VideoDecoder>>> = OnceLock::new();
static VIDEO_METERS: OnceLock<Mutex<HashMap<Id, VideoMeter>>> = OnceLock::new();
static QUALITY: OnceLock<Mutex<VideoQuality>> = OnceLock::new();
static VIEWER_QUALITY: OnceLock<Mutex<ViewQuality>> = OnceLock::new();
static LOCAL_USER: AtomicI64 = AtomicI64::new(0);
static AUDIO_BITRATE_KBPS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(64);
static INPUT_DEVICE: OnceLock<Mutex<Option<String>>> = OnceLock::new();
static OUTPUT_DEVICE: OnceLock<Mutex<Option<String>>> = OnceLock::new();
static CAMERA_DEVICE: OnceLock<Mutex<Option<String>>> = OnceLock::new();
static INPUT_SENSITIVITY: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(0.02f32.to_bits());
static AUTO_SENSITIVITY: AtomicBool = AtomicBool::new(true);
static VOICE_ACTIVITY: AtomicBool = AtomicBool::new(true);
static PTT_MODE: AtomicBool = AtomicBool::new(false);
static PTT_HELD: AtomicBool = AtomicBool::new(false);
static PTT_POLLER: OnceLock<()> = OnceLock::new();
static AUDIO_HEADER_REQUEST: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetAsyncKeyState(vkey: i32) -> i16;
}

fn ensure_ptt_poller() {
    PTT_POLLER.get_or_init(|| {
        thread::spawn(|| {
            let mut last = false;
            loop {
                let held = if PTT_MODE.load(Ordering::Relaxed) {
                    #[cfg(windows)]
                    {
                        // F8 is the default Push-to-Talk key.
                        unsafe { (GetAsyncKeyState(0x77) as u16 & 0x8000) != 0 }
                    }
                    #[cfg(not(windows))]
                    {
                        false
                    }
                } else {
                    false
                };
                PTT_HELD.store(held, Ordering::Relaxed);
                if held != last {
                    last = held;
                    log(&format!("push to talk held={held}"));
                }
                thread::sleep(Duration::from_millis(25));
            }
        });
    });
}
static AUTO_FLOOR: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(0.0005f32.to_bits());
static INPUT_VOLUME: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(1.0f32.to_bits());
static OUTPUT_VOLUME: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(1.0f32.to_bits());
static MIC_LEVEL: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static MIC_TEST_STOP: OnceLock<Mutex<Option<Arc<AtomicBool>>>> = OnceLock::new();
static CAMERA_TEST_STOP: OnceLock<Mutex<Option<Arc<AtomicBool>>>> = OnceLock::new();
static CAMERA_TEST_FRAME: OnceLock<Mutex<Option<rune::Pixels>>> = OnceLock::new();
static CAMERA_TEST_VERSION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static VIEWER_EPOCH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static WAKER: OnceLock<Mutex<Option<rune::Waker>>> = OnceLock::new();

pub fn set_waker(waker: rune::Waker) {
    let slot = WAKER.get_or_init(|| Mutex::new(None));
    if let Ok(mut current) = slot.lock() {
        *current = Some(waker);
    }
}

fn wake() {
    if let Some(slot) = WAKER.get() {
        if let Ok(current) = slot.lock() {
            if let Some(waker) = current.as_ref() {
                waker();
            }
        }
    }
}

pub fn set_quality(quality: VideoQuality) {
    let slot = QUALITY.get_or_init(|| Mutex::new(VideoQuality::Auto));
    if let Ok(mut current) = slot.lock() {
        *current = quality;
    }
    if let Some(session) = SESSION.get() {
        if let Ok(current) = session.lock() {
            if let Some(session) = current.as_ref() {
                let _ = session.video.send(VideoCommand {
                    kind: None,
                    quality: Some(quality),
                });
            }
        }
    }
}

pub fn set_viewer_quality(quality: ViewQuality) {
    let slot = VIEWER_QUALITY.get_or_init(|| Mutex::new(ViewQuality::Auto));
    if let Ok(mut current) = slot.lock() {
        *current = quality;
    }
    VIEWER_EPOCH.fetch_add(1, Ordering::Relaxed);
}

pub fn set_input_device(device: String) {
    let slot = INPUT_DEVICE.get_or_init(|| Mutex::new(None));
    if let Ok(mut current) = slot.lock() {
        *current = (!device.is_empty()).then_some(device);
    }
}

pub fn set_output_device(device: String) {
    let slot = OUTPUT_DEVICE.get_or_init(|| Mutex::new(None));
    if let Ok(mut current) = slot.lock() {
        *current = (!device.is_empty()).then_some(device);
    }
}

fn selected_output_device() -> Option<String> {
    OUTPUT_DEVICE
        .get()
        .and_then(|slot| slot.lock().ok().and_then(|slot| slot.clone()))
}

pub fn set_camera_device(device: String) {
    let slot = CAMERA_DEVICE.get_or_init(|| Mutex::new(None));
    if let Ok(mut current) = slot.lock() {
        *current = (!device.is_empty()).then_some(device);
    }
}

fn selected_input_device() -> Option<String> {
    INPUT_DEVICE
        .get()
        .and_then(|slot| slot.lock().ok().and_then(|slot| slot.clone()))
}

fn selected_camera_device() -> Option<String> {
    CAMERA_DEVICE
        .get()
        .and_then(|slot| slot.lock().ok().and_then(|slot| slot.clone()))
}

pub fn set_input_sensitivity(value: f32) {
    INPUT_SENSITIVITY.store(value.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
}

pub fn set_voice_activity(enabled: bool) {
    VOICE_ACTIVITY.store(enabled, Ordering::Relaxed);
}

pub fn set_push_to_talk(enabled: bool) {
    PTT_MODE.store(enabled, Ordering::Relaxed);
    if enabled {
        ensure_ptt_poller();
    } else {
        PTT_HELD.store(false, Ordering::Relaxed);
    }
}

pub fn push_to_talk_held() -> bool {
    PTT_HELD.load(Ordering::Relaxed)
}

pub fn set_auto_sensitivity(enabled: bool) {
    AUTO_SENSITIVITY.store(enabled, Ordering::Relaxed);
}

pub fn set_input_volume(value: f32) {
    INPUT_VOLUME.store(value.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
}

pub fn set_output_volume(value: f32) {
    OUTPUT_VOLUME.store(value.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
}

fn input_sensitivity() -> f32 {
    f32::from_bits(INPUT_SENSITIVITY.load(Ordering::Relaxed))
}

fn input_volume() -> f32 {
    f32::from_bits(INPUT_VOLUME.load(Ordering::Relaxed))
}

fn output_volume() -> f32 {
    f32::from_bits(OUTPUT_VOLUME.load(Ordering::Relaxed))
}

fn pcm_rms(samples: &[u8]) -> f32 {
    let mut sum = 0.0f64;
    let mut count = 0u64;
    for pair in samples.chunks_exact(2) {
        let value = i16::from_le_bytes([pair[0], pair[1]]) as f64 / 32768.0;
        sum += value * value;
        count += 1;
    }
    if count == 0 {
        0.0
    } else {
        (sum / count as f64).sqrt() as f32
    }
}

fn shape_pcm(samples: &mut [u8], muted: &AtomicBool, gate: bool, synth: bool) {
    let volume = input_volume();
    let rms = pcm_rms(samples);
    let ptt_blocked = PTT_MODE.load(Ordering::Relaxed) && !PTT_HELD.load(Ordering::Relaxed);
    if gate && ptt_blocked && !muted.load(Ordering::Relaxed) {
        samples.fill(0);
    } else if gate
        && !synth
        && VOICE_ACTIVITY.load(Ordering::Relaxed)
        && !muted.load(Ordering::Relaxed)
    {
        let threshold = if AUTO_SENSITIVITY.load(Ordering::Relaxed) {
            let previous = f32::from_bits(AUTO_FLOOR.load(Ordering::Relaxed));
            let floor = (previous * 0.95 + rms * 0.05).max(0.0005);
            AUTO_FLOOR.store(floor.to_bits(), Ordering::Relaxed);
            (floor * 3.5).clamp(0.008, 0.25)
        } else {
            input_sensitivity().max(0.001)
        };
        if rms < threshold {
            samples.fill(0);
        }
    }
    if (volume - 1.0).abs() > f32::EPSILON {
        for pair in samples.chunks_exact_mut(2) {
            let value = i16::from_le_bytes([pair[0], pair[1]]) as f32 * volume;
            let value = value.clamp(-32768.0, 32767.0) as i16;
            pair.copy_from_slice(&value.to_le_bytes());
        }
    }
}

pub fn set_audio_bitrate(kbps: u32) {
    AUDIO_BITRATE_KBPS.store(kbps.clamp(64, 384), Ordering::Relaxed);
}

fn audio_bitrate() -> u32 {
    AUDIO_BITRATE_KBPS.load(Ordering::Relaxed)
}

pub fn set_local_user(user: Id) {
    LOCAL_USER.store(user, Ordering::Relaxed);
}

fn local_user() -> Option<Id> {
    let user = LOCAL_USER.load(Ordering::Relaxed);
    (user > 0).then_some(user)
}

fn current_viewer_quality() -> ViewQuality {
    VIEWER_QUALITY
        .get()
        .and_then(|slot| slot.lock().ok().map(|value| *value))
        .unwrap_or(ViewQuality::Auto)
}

fn current_quality() -> VideoQuality {
    QUALITY
        .get()
        .and_then(|slot| slot.lock().ok().map(|value| *value))
        .unwrap_or(VideoQuality::Auto)
}

pub fn video_stats() -> HashMap<Id, (u32, u32, u32)> {
    VIDEO_METERS
        .get()
        .and_then(|meters| {
            meters.lock().ok().map(|meters| {
                meters
                    .iter()
                    .map(|(user, meter)| (*user, (meter.width, meter.height, meter.fps)))
                    .collect()
            })
        })
        .unwrap_or_default()
}

static VIDEO_LATEST: OnceLock<Mutex<HashMap<Id, VideoFrame>>> = OnceLock::new();

static VIDEO_STOPPED: OnceLock<Mutex<Vec<Id>>> = OnceLock::new();

pub fn drain_video() -> (Vec<(Id, u32, u32, Vec<u8>)>, Vec<Id>) {
    let frames = VIDEO_LATEST.get_or_init(|| Mutex::new(HashMap::new()));
    let stopped = VIDEO_STOPPED.get_or_init(|| Mutex::new(Vec::new()));
    let frames = frames
        .lock()
        .map(|mut frames| {
            frames
                .drain()
                .map(|(user, frame)| (user, frame.width, frame.height, frame.rgba))
                .collect()
        })
        .unwrap_or_default();
    let stopped = stopped
        .lock()
        .map(|mut stopped| std::mem::take(&mut *stopped))
        .unwrap_or_default();
    (frames, stopped)
}

struct ReceiveStream {
    tx: Sender<Vec<u8>>,
    stop: Arc<AtomicBool>,
    serial: u32,
}

static SESSION: OnceLock<Mutex<Option<Session>>> = OnceLock::new();

pub fn update(base: &str, token: &str, channel: Option<Id>, mute: bool, deaf: bool) {
    let slot = SESSION.get_or_init(|| Mutex::new(None));
    let Ok(mut current) = slot.lock() else {
        return;
    };
    if let Some(session) = current.as_ref() {
        if channel == Some(session.channel) {
            let was_muted = session.muted.load(Ordering::Relaxed);
            let was_deafened = session.deafened.load(Ordering::Relaxed);
            session.muted.store(mute, Ordering::Relaxed);
            session.deafened.store(deaf, Ordering::Relaxed);
            if was_muted != mute || was_deafened != deaf {
                log(&format!("mute={mute} deaf={deaf}"));
            }
            return;
        }
    }
    if let Some(old) = current.take() {
        old.stop.store(true, Ordering::Relaxed);
    }
    let Some(channel) = channel else {
        return;
    };
    let stop = Arc::new(AtomicBool::new(false));
    let muted = Arc::new(AtomicBool::new(mute));
    let deafened = Arc::new(AtomicBool::new(deaf));
    let (video, video_rx) = mpsc::channel();
    let session = Session {
        channel,
        stop: stop.clone(),
        muted: muted.clone(),
        deafened: deafened.clone(),
        video,
    };
    *current = Some(session);
    let base = base.to_string();
    let token = token.to_string();
    thread::spawn(move || run(base, token, channel, stop, muted, deafened, video_rx));
}

pub fn set_video(base: &str, _token: &str, channel: Option<Id>, kind: &str) {
    let _ = (base, channel);
    if let Some(slot) = SESSION.get() {
        if let Ok(current) = slot.lock() {
            if let Some(session) = current.as_ref() {
                let _ = session.video.send(VideoCommand {
                    kind: Some(kind.to_string()),
                    quality: None,
                });
            }
        }
    }
}

pub fn shutdown() {
    if let Some(slot) = SESSION.get() {
        if let Ok(mut current) = slot.lock() {
            if let Some(session) = current.take() {
                session.stop.store(true, Ordering::Relaxed);
            }
        }
    }
}

fn log(message: &str) {
    if let Ok(path) = std::env::var("RUNECORD_VOICE_LOG") {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let millis = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|value| value.as_millis())
                .unwrap_or(0);
            let _ = writeln!(file, "{millis} {message}");
        }
    }
}

fn start_video(
    out: &mpsc::SyncSender<(u8, Vec<u8>)>,
    kind: String,
    stop: &Arc<AtomicBool>,
    quality: VideoQuality,
) -> Vec<Arc<AtomicBool>> {
    let mut flags = Vec::new();
    for (layer_quality, low) in [(quality, false), (VideoQuality::P360, true)] {
        let flag = Arc::new(AtomicBool::new(false));
        flags.push(flag.clone());
        let out = out.clone();
        let stop = stop.clone();
        let kind = kind.clone();
        thread::spawn(move || video_capture(out, kind, stop, flag, layer_quality, low));
    }
    flags
}

fn send_frame(
    socket: &mut tungstenite::WebSocket<MaybeTlsStream<std::net::TcpStream>>,
    kind: u8,
    chunk: &[u8],
) -> Result<(), tungstenite::Error> {
    let mut frame = Vec::with_capacity(1 + chunk.len());
    frame.push(kind);
    frame.extend_from_slice(chunk);
    socket.send(Message::Binary(frame.into()))
}

fn run(
    base: String,
    token: String,
    channel: Id,
    stop: Arc<AtomicBool>,
    muted: Arc<AtomicBool>,
    deafened: Arc<AtomicBool>,
    commands: Receiver<VideoCommand>,
) {
    let ws_base = base
        .replace("https://", "wss://")
        .replace("http://", "ws://");
    let url = format!(
        "{ws_base}/voice?token={}&channel={channel}",
        urlencoding::encode(&token)
    );
    let mut socket = match connect(url) {
        Ok((socket, _)) => socket,
        Err(error) => {
            log(&format!("connect error: {error}"));
            return;
        }
    };
    if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
        let _ = stream.set_read_timeout(Some(Duration::from_millis(10)));
    }
    log(&format!("joined channel {channel}"));
    let (audio_tx, audio_rx) = mpsc::channel::<(u8, Vec<u8>)>();
    let (video_tx, video_rx) = mpsc::sync_channel::<(u8, Vec<u8>)>(4);
    let synth = std::env::var("RUNECORD_VOICE_SYNTH").ok().as_deref() == Some("1");
    let capture_stop = stop.clone();
    let capture_mute = muted.clone();
    thread::spawn(move || capture_encoder(audio_tx, capture_stop, capture_mute, synth));

    let mut video_stops: Vec<Arc<AtomicBool>> = Vec::new();
    let mut video_kind: Option<String> = None;
    let mut viewer_epoch = VIEWER_EPOCH.load(Ordering::Relaxed);
    let mut quality = current_quality();
    log(&format!(
        "video quality: {} {}x{}@{}fps q{}",
        quality.label(),
        quality.resolution().0,
        quality.resolution().1,
        quality.fps(),
        quality.qscale()
    ));

    let test = std::env::var("RUNECORD_VOICE_TEST").ok().as_deref() == Some("1");
    let mut receivers: HashMap<Id, ReceiveStream> = HashMap::new();
    let mut header_requests: HashMap<Id, std::time::Instant> = HashMap::new();
    while !stop.load(Ordering::Relaxed) {
        while let Ok(command) = commands.try_recv() {
            if let Some(kind) = command.kind {
                log(&format!("video command: {kind}"));
                for old in video_stops.drain(..) {
                    old.store(true, Ordering::Relaxed);
                }
                video_kind = if kind == "none" { None } else { Some(kind) };
                if let Some(kind) = &video_kind {
                    video_stops = start_video(&video_tx, kind.clone(), &stop, quality);
                }
            }
            if let Some(next) = command.quality {
                quality = next;
                log(&format!(
                    "video quality set: {} {}x{}@{}fps q{}",
                    quality.label(),
                    quality.resolution().0,
                    quality.resolution().1,
                    quality.fps(),
                    quality.qscale()
                ));
                if let Some(kind) = &video_kind {
                    for old in video_stops.drain(..) {
                        old.store(true, Ordering::Relaxed);
                    }
                    video_stops = start_video(&video_tx, kind.clone(), &stop, quality);
                }
            }
        }
        let next_epoch = VIEWER_EPOCH.load(Ordering::Relaxed);
        if next_epoch != viewer_epoch {
            viewer_epoch = next_epoch;
            if let Some(decoders) = VIDEO_DECODERS.get() {
                if let Ok(mut decoders) = decoders.lock() {
                    for (_, decoder) in decoders.drain() {
                        decoder.stop.store(true, Ordering::Relaxed);
                    }
                }
            }
            if let Some(frames) = VIDEO_LATEST.get() {
                if let Ok(mut frames) = frames.lock() {
                    frames.clear();
                }
            }
            if let Some(meters) = VIDEO_METERS.get() {
                if let Ok(mut meters) = meters.lock() {
                    meters.clear();
                }
            }
            log(&format!(
                "simulcast: viewer layer {}",
                current_viewer_quality().label()
            ));
            wake();
        }
        let viewer = current_viewer_quality();
        match socket.read() {
            Ok(Message::Binary(data)) => {
                if data.len() >= 9 {
                    let user = Id::from_le_bytes(data[..8].try_into().unwrap_or_default());
                    let kind = data[8];
                    let payload = data[9..].to_vec();
                    if (kind == 3 && viewer != ViewQuality::Low)
                        || (kind == 5 && viewer == ViewQuality::Low)
                    {
                        if let Some((width, height, fps)) = parse_status(&payload) {
                            log(&format!(
                                "video status user={user} layer={} {}x{}@{}fps",
                                if kind == 3 { "high" } else { "low" },
                                width,
                                height,
                                fps
                            ));
                            note_status(user, width, height, fps);
                        }
                    } else if kind == 2 {
                        header_requests.remove(&user);
                        if let Some(stream) = receivers.remove(&user) {
                            stream.stop.store(true, Ordering::Relaxed);
                        }
                        let frames = VIDEO_LATEST.get_or_init(|| Mutex::new(HashMap::new()));
                        if let Ok(mut frames) = frames.lock() {
                            frames.remove(&user);
                        }
                        let stopped = VIDEO_STOPPED.get_or_init(|| Mutex::new(Vec::new()));
                        if let Ok(mut stopped) = stopped.lock() {
                            stopped.push(user);
                        }
                        if let Some(meters) = VIDEO_METERS.get() {
                            if let Ok(mut meters) = meters.lock() {
                                meters.remove(&user);
                            }
                        }
                        if let Some(decoders) = VIDEO_DECODERS.get() {
                            if let Ok(mut decoders) = decoders.lock() {
                                if let Some(decoder) = decoders.remove(&user) {
                                    decoder.stop.store(true, Ordering::Relaxed);
                                }
                            }
                        }
                        wake();
                        log(&format!("video stopped user={user}"));
                    } else if kind == 1 && viewer != ViewQuality::Low {
                        attach_video(user, payload, &stop);
                    } else if kind == 4 && viewer == ViewQuality::Low {
                        attach_video(user, payload, &stop);
                    } else if kind == 6 {
                        AUDIO_HEADER_REQUEST.store(true, Ordering::Relaxed);
                        log(&format!("audio header request from user={user}"));
                    } else if !deafened.load(Ordering::Relaxed) {
                        let begins_stream = payload.starts_with(b"OggS")
                            && payload.len() > 5
                            && payload[5] & 0x02 != 0;
                        let incoming_serial = ogg_serial(&payload);
                        if begins_stream {
                            let refresh = incoming_serial
                                .is_some_and(|serial| stream_is_current(&receivers, user, serial));
                            if refresh {
                                // The sender repeats its headers periodically for late
                                // joiners. Existing decoders already have them, so ignore
                                // the duplicate pages instead of rebuilding FFmpeg.
                                log(&format!(
                                    "audio header refresh ignored user={user} serial={}",
                                    incoming_serial.unwrap_or(0)
                                ));
                            } else {
                                if let Some(old) = receivers.remove(&user) {
                                    old.stop.store(true, Ordering::Relaxed);
                                }
                                header_requests.remove(&user);
                                let (tx, rx) = mpsc::channel::<Vec<u8>>();
                                let stream_stop = Arc::new(AtomicBool::new(false));
                                let _ = tx.send(payload);
                                receivers.insert(
                                    user,
                                    ReceiveStream {
                                        tx: tx.clone(),
                                        stop: stream_stop.clone(),
                                        serial: incoming_serial.unwrap_or(0),
                                    },
                                );
                                let stop = stop.clone();
                                let deafened = deafened.clone();
                                thread::spawn(move || {
                                    receive_stream(user, rx, stop, deafened, test, stream_stop)
                                });
                            }
                        } else if let Some(stream) = receivers.get(&user) {
                            let _ = stream.tx.send(payload);
                        } else if header_requests
                            .get(&user)
                            .is_none_or(|at| at.elapsed() >= Duration::from_secs(2))
                        {
                            header_requests.insert(user, std::time::Instant::now());
                            let _ = send_frame(&mut socket, 6, &[]);
                            log(&format!("audio headers requested user={user}"));
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(tungstenite::Error::Io(error))
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut => {}
            Err(error) => {
                log(&format!("socket error: {error}"));
                break;
            }
        }
        while let Ok((kind, chunk)) = audio_rx.try_recv() {
            if send_frame(&mut socket, kind, &chunk).is_err() {
                break;
            }
        }
        // Test hook: hold the relay so the bounded video queue fills and Auto
        // can be exercised without a genuinely slow network.
        if let Some(ms) = std::env::var("RUNECORD_VIDEO_BACKPRESSURE_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|ms| *ms > 0)
        {
            thread::sleep(Duration::from_millis(ms));
        }
        while let Ok((kind, chunk)) = video_rx.try_recv() {
            if send_frame(&mut socket, kind, &chunk).is_err() {
                break;
            }
        }
    }
    stop.store(true, Ordering::Relaxed);
    for stream in receivers.values() {
        stream.stop.store(true, Ordering::Relaxed);
    }
    if let Some(decoders) = VIDEO_DECODERS.get() {
        if let Ok(mut decoders) = decoders.lock() {
            for (_, decoder) in decoders.drain() {
                decoder.stop.store(true, Ordering::Relaxed);
            }
        }
    }
    log(&format!("left channel {channel}"));
}

fn ffmpeg_path() -> String {
    std::env::var("RUNECORD_FFMPEG").unwrap_or_else(|_| "ffmpeg".to_string())
}

fn parse_status(payload: &[u8]) -> Option<(u32, u32, u32)> {
    let text = std::str::from_utf8(payload).ok()?;
    let (dims, fps) = text.split_once('@')?;
    let (width, height) = dims.split_once('x')?;
    Some((width.parse().ok()?, height.parse().ok()?, fps.parse().ok()?))
}

fn parse_size(value: &str) -> Option<(u32, u32)> {
    let (width, height) = value.trim().split_once(['x', 'X'])?;
    let width = width.parse::<u32>().ok()?;
    let height = height.parse::<u32>().ok()?;
    (width > 0 && height > 0).then_some((width, height))
}

fn jpeg_dimensions(jpeg: &[u8]) -> Option<(u32, u32)> {
    if jpeg.len() < 4 || jpeg[0] != 0xFF || jpeg[1] != 0xD8 {
        return None;
    }
    let mut index = 2usize;
    while index + 4 <= jpeg.len() {
        if jpeg[index] != 0xFF {
            index += 1;
            continue;
        }
        let marker = jpeg[index + 1];
        if marker == 0xD8 || marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            index += 2;
            continue;
        }
        let length = u16::from_be_bytes([jpeg[index + 2], jpeg[index + 3]]) as usize;
        if length < 2 {
            return None;
        }
        let is_sof = (0xC0..=0xCF).contains(&marker) && !matches!(marker, 0xC4 | 0xC8 | 0xCC);
        if is_sof && index + 9 <= jpeg.len() {
            let height = u16::from_be_bytes([jpeg[index + 5], jpeg[index + 6]]) as u32;
            let width = u16::from_be_bytes([jpeg[index + 7], jpeg[index + 8]]) as u32;
            return (width > 0 && height > 0).then_some((width, height));
        }
        index += 2 + length;
    }
    None
}

fn ogg_serial(page: &[u8]) -> Option<u32> {
    if page.len() >= 18 && page.starts_with(b"OggS") {
        Some(u32::from_le_bytes([page[14], page[15], page[16], page[17]]))
    } else {
        None
    }
}

fn stream_is_current(receivers: &HashMap<Id, ReceiveStream>, user: Id, serial: u32) -> bool {
    receivers
        .get(&user)
        .is_some_and(|stream| stream.serial == serial)
}

fn take_ogg_page(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    if buffer.len() < 27 {
        return None;
    }
    if &buffer[..4] != b"OggS" {
        buffer.clear();
        return None;
    }
    let segments = buffer[26] as usize;
    if buffer.len() < 27 + segments {
        return None;
    }
    let body: usize = buffer[27..27 + segments]
        .iter()
        .map(|value| *value as usize)
        .sum();
    let total = 27 + segments + body;
    if buffer.len() < total {
        return None;
    }
    Some(buffer.drain(..total).collect())
}

fn capture_encoder(
    out: Sender<(u8, Vec<u8>)>,
    stop: Arc<AtomicBool>,
    muted: Arc<AtomicBool>,
    synth: bool,
) {
    if let Some(queue) = SOUNDBOARD.get() {
        if let Ok(mut queue) = queue.lock() {
            queue.clear();
        }
    }
    let bitrate = audio_bitrate();
    log(&format!("voice bitrate: {bitrate} kbps"));
    let rate_arg = format!("{bitrate}k");
    let mut encoder = match Command::new(ffmpeg_path())
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "s16le",
            "-ar",
            "48000",
            "-ac",
            "1",
            "-i",
            "pipe:0",
            "-c:a",
            "libopus",
            "-b:a",
            &rate_arg,
            "-f",
            "ogg",
            "-flush_packets",
            "1",
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            log(&format!("encoder error: {error}"));
            return;
        }
    };
    let mut encoder_in = match encoder.stdin.take() {
        Some(stdin) => stdin,
        None => return,
    };
    if let Some(mut stdout) = encoder.stdout.take() {
        let out = out.clone();
        let stop = stop.clone();
        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            let mut pending: Vec<u8> = Vec::new();
            let mut header: Vec<Vec<u8>> = Vec::new();
            let mut header_sent = std::time::Instant::now();
            loop {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                match stdout.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        pending.extend_from_slice(&buffer[..n]);
                        while let Some(page) = take_ogg_page(&mut pending) {
                            if header.len() < 2 {
                                header.push(page.clone());
                            }
                            let requested = AUDIO_HEADER_REQUEST.swap(false, Ordering::Relaxed);
                            if header.len() == 2
                                && (header_sent.elapsed().as_secs() >= 15 || requested)
                            {
                                for cached in &header {
                                    if out.send((0, cached.clone())).is_err() {
                                        return;
                                    }
                                }
                                header_sent = std::time::Instant::now();
                                log(if requested {
                                    "audio headers resent on request"
                                } else {
                                    "audio headers resent after 15s"
                                });
                            }
                            if out.send((0, page)).is_err() {
                                return;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    if synth {
        let mut phase = 0.0f32;
        let step = std::f32::consts::TAU * 440.0 / 48000.0;
        let mut samples = [0u8; 1920];
        while !stop.load(Ordering::Relaxed) {
            let open_mic = !muted.load(Ordering::Relaxed);
            if open_mic {
                for chunk in samples.chunks_exact_mut(2) {
                    let value = (phase.sin() * 0.25 * 32767.0) as i16;
                    chunk.copy_from_slice(&value.to_le_bytes());
                    phase += step;
                    if phase > std::f32::consts::TAU {
                        phase -= std::f32::consts::TAU;
                    }
                }
                shape_pcm(&mut samples, &muted, true, true);
            } else {
                samples.fill(0);
            }
            let soundboard = mix_soundboard(&mut samples);
            if (open_mic || soundboard) && encoder_in.write_all(&samples).is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
    } else if let Some(device) = microphone() {
        log(&format!("capture device: {device}"));
        let mut capture = match Command::new(ffmpeg_path())
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "dshow",
                "-i",
                &format!("audio={device}"),
                "-f",
                "s16le",
                "-ar",
                "48000",
                "-ac",
                "1",
                "pipe:1",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                log(&format!("capture error: {error}"));
                return;
            }
        };
        let mut buffer = [0u8; 1920];
        if let Some(mut stdout) = capture.stdout.take() {
            while !stop.load(Ordering::Relaxed) {
                match stdout.read_exact(&mut buffer) {
                    Ok(()) => {
                        shape_pcm(&mut buffer, &muted, true, false);
                        let soundboard = mix_soundboard(&mut buffer);
                        if (!muted.load(Ordering::Relaxed) || soundboard)
                            && encoder_in.write_all(&buffer).is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
        let _ = capture.kill();
    } else {
        log("no microphone found");
    }
    let _ = encoder.kill();
}

pub fn list_devices() -> (Vec<String>, Vec<String>) {
    let mut audio = Vec::new();
    let mut video = Vec::new();
    let Ok(output) = Command::new(ffmpeg_path())
        .args([
            "-hide_banner",
            "-list_devices",
            "true",
            "-f",
            "dshow",
            "-i",
            "dummy",
        ])
        .output()
    else {
        return (audio, video);
    };
    let text = String::from_utf8_lossy(&output.stderr);
    for line in text.lines() {
        let kind = if line.contains("(audio)") {
            Some(true)
        } else if line.contains("(video)") {
            Some(false)
        } else {
            None
        };
        let Some(is_audio) = kind else { continue };
        let Some(name) = line.find('"').and_then(|start| {
            line[start + 1..]
                .find('"')
                .map(|end| &line[start + 1..start + 1 + end])
        }) else {
            continue;
        };
        if is_audio {
            audio.push(name.to_string());
        } else {
            video.push(name.to_string());
        }
    }
    (audio, video)
}

static SOUNDBOARD: OnceLock<Mutex<Vec<i16>>> = OnceLock::new();

/// Names of the built-in soundboard effects, in the order the panel shows them.
pub fn soundboard_sounds() -> [(&'static str, &'static str, f32); 6] {
    [
        ("airhorn", "Airhorn", 0.9),
        ("quack", "Quack", 0.4),
        ("cricket", "Crickets", 0.9),
        ("boom", "Boom", 0.7),
        ("ding", "Ding", 0.9),
        ("bruh", "Bruh", 0.6),
    ]
}

/// Queues a built-in sound for the next capture chunks. Because the PCM is mixed
/// into the outgoing stream before Opus encoding, everyone in the channel hears it.
pub fn play_soundboard(name: &str) {
    let pcm = soundboard_pcm(name);
    if pcm.is_empty() {
        return;
    }
    let queue = SOUNDBOARD.get_or_init(|| Mutex::new(Vec::new()));
    if let Ok(mut queue) = queue.lock() {
        queue.extend_from_slice(&pcm);
        // Never let a stuck queue grow without bound.
        let max = 48_000 * 8;
        if queue.len() > max {
            let excess = queue.len() - max;
            queue.drain(..excess);
        }
    }
    log(&format!("soundboard: {name}"));
}

fn mix_soundboard(pcm: &mut [u8]) -> bool {
    let queue = SOUNDBOARD.get_or_init(|| Mutex::new(Vec::new()));
    match queue.lock() {
        Ok(mut queue) => mix_into(pcm, &mut queue),
        Err(_) => false,
    }
}

/// Mixes queued samples into little-endian s16 stereo-pair PCM, consuming what it used.
fn mix_into(pcm: &mut [u8], queue: &mut Vec<i16>) -> bool {
    if queue.is_empty() {
        return false;
    }
    let frames = pcm.len() / 2;
    let take = frames.min(queue.len());
    for (index, pair) in pcm.chunks_exact_mut(2).take(take).enumerate() {
        let base = i16::from_le_bytes([pair[0], pair[1]]) as i32;
        let mixed = (base + queue[index] as i32).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        pair.copy_from_slice(&mixed.to_le_bytes());
    }
    queue.drain(..take);
    true
}

fn soundboard_pcm(name: &str) -> Vec<i16> {
    const RATE: f32 = 48_000.0;
    let (seconds, build): (f32, fn(f32, f32) -> f32) = match name {
        "airhorn" => (0.9, |t, d| {
            let vibrato = 1.0 + 0.06 * (t * 42.0).sin();
            let base = 330.0 * vibrato;
            let saw = 2.0 * ((t * base) % 1.0) - 1.0;
            let square = if (t * base * 2.0).fract() < 0.5 {
                1.0
            } else {
                -1.0
            };
            let env = attack_release(t, d, 0.04, 0.25);
            (saw * 0.45 + square * 0.3) * env * 0.55
        }),
        "quack" => (0.4, |t, d| {
            let progress = t / d;
            let freq = 760.0 - 500.0 * progress + 40.0 * (t * 28.0).sin();
            let wobble = 0.25 * (t * 90.0).sin();
            let env = attack_release(t, d, 0.02, 0.18);
            ((t * freq).sin() * (1.0 + wobble)) * env * 0.5
        }),
        "cricket" => (0.9, |t, d| {
            let chirp = (t * 9.0) % 1.0;
            let gate = if chirp < 0.35 { 1.0 } else { 0.0 };
            let env = attack_release(t, d, 0.01, 0.2);
            ((t * 4200.0).sin() * gate) * env * 0.22
        }),
        "boom" => (0.7, |t, d| {
            let env = (-t * 6.0).exp();
            let thump = (t * (90.0 - 40.0 * t / d)).sin();
            let noise = ((t * 12_345.0).sin() * (t * 7_777.0).sin()) * (-t * 22.0).exp();
            (thump * env * 0.7 + noise * 0.25) * 0.8
        }),
        "ding" => (0.9, |t, d| {
            let env = (-t * 5.0).exp();
            let a = (t * 1046.5 * std::f32::consts::TAU).sin();
            let b = (t * 1568.0 * std::f32::consts::TAU).sin() * 0.5;
            let fade = attack_release(t, d, 0.004, 0.3);
            (a + b) * env * fade * 0.34
        }),
        "bruh" => (0.6, |t, d| {
            let progress = t / d;
            let freq = 190.0 - 110.0 * progress;
            let saw = 2.0 * ((t * freq) % 1.0) - 1.0;
            let env = attack_release(t, d, 0.02, 0.2);
            saw * env * 0.45
        }),
        _ => (0.0, |_, _| 0.0),
    };
    if seconds <= 0.0 {
        return Vec::new();
    }
    let count = (RATE * seconds) as usize;
    let mut out = Vec::with_capacity(count);
    for index in 0..count {
        let t = index as f32 / RATE;
        let sample = build(t, seconds).clamp(-1.0, 1.0);
        out.push((sample * 32767.0) as i16);
    }
    out
}

fn attack_release(t: f32, duration: f32, attack: f32, release: f32) -> f32 {
    let rise = (t / attack.max(0.001)).min(1.0);
    let fall = ((duration - t) / release.max(0.001)).clamp(0.0, 1.0);
    rise * fall
}

pub fn list_output_devices() -> Vec<String> {
    #[cfg(windows)]
    {
        return crate::wasapi::list_output_devices();
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

pub fn mic_level() -> f32 {
    f32::from_bits(MIC_LEVEL.load(Ordering::Relaxed))
}

pub fn mic_testing() -> bool {
    MIC_TEST_STOP
        .get()
        .and_then(|slot| slot.lock().ok().map(|slot| slot.is_some()))
        .unwrap_or(false)
}

pub fn start_mic_test() {
    stop_mic_test();
    let stop = Arc::new(AtomicBool::new(false));
    let slot = MIC_TEST_STOP.get_or_init(|| Mutex::new(None));
    if let Ok(mut current) = slot.lock() {
        *current = Some(stop.clone());
    }
    thread::spawn(move || {
        let synthetic = std::env::var("RUNECORD_VOICE_SYNTH").ok().as_deref() == Some("1");
        let device = selected_input_device().or_else(microphone);
        let mut command = Command::new(ffmpeg_path());
        command.args(["-hide_banner", "-loglevel", "error"]);
        if synthetic || device.is_none() {
            command.args([
                "-re",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000",
            ]);
        } else if let Some(device) = device {
            command.args(["-f", "dshow", "-i", &format!("audio={device}")]);
        }
        command.args(["-f", "s16le", "-ar", "48000", "-ac", "1", "pipe:1"]);
        let Ok(mut child) = command.stdout(Stdio::piped()).stderr(Stdio::null()).spawn() else {
            log("mic test could not start");
            return;
        };
        let mut buffer = [0u8; 1920];
        if let Some(mut stdout) = child.stdout.take() {
            while !stop.load(Ordering::Relaxed) {
                match stdout.read_exact(&mut buffer) {
                    Ok(()) => {
                        let level = (pcm_rms(&buffer) * 4.0).clamp(0.0, 1.0);
                        MIC_LEVEL.store(level.to_bits(), Ordering::Relaxed);
                    }
                    Err(_) => break,
                }
            }
        }
        let _ = child.kill();
        MIC_LEVEL.store(0.0f32.to_bits(), Ordering::Relaxed);
        log("mic test stopped");
    });
}

pub fn stop_mic_test() {
    if let Some(slot) = MIC_TEST_STOP.get() {
        if let Ok(mut current) = slot.lock() {
            if let Some(stop) = current.take() {
                stop.store(true, Ordering::Relaxed);
            }
        }
    }
    MIC_LEVEL.store(0.0f32.to_bits(), Ordering::Relaxed);
}

pub fn camera_test_version() -> u64 {
    CAMERA_TEST_VERSION.load(Ordering::Relaxed)
}

pub fn camera_test_frame() -> Option<rune::Pixels> {
    CAMERA_TEST_FRAME
        .get()
        .and_then(|slot| slot.lock().ok().and_then(|slot| slot.clone()))
}

pub fn camera_testing() -> bool {
    CAMERA_TEST_STOP
        .get()
        .and_then(|slot| slot.lock().ok().map(|slot| slot.is_some()))
        .unwrap_or(false)
}

pub fn start_camera_test() {
    stop_camera_test();
    let Some(device) = selected_camera_device().or_else(camera) else {
        log("no camera for test");
        return;
    };
    let stop = Arc::new(AtomicBool::new(false));
    let slot = CAMERA_TEST_STOP.get_or_init(|| Mutex::new(None));
    if let Ok(mut current) = slot.lock() {
        *current = Some(stop.clone());
    }
    thread::spawn(move || {
        let Ok(mut child) = Command::new(ffmpeg_path())
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "dshow",
                "-framerate",
                "15",
                "-i",
                &format!("video={device}"),
                "-an",
                "-vf",
                "scale=640:-2",
                "-q:v",
                "5",
                "-f",
                "mjpeg",
                "pipe:1",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        else {
            log("camera test could not start");
            return;
        };
        let mut buffer = [0u8; 65536];
        let mut pending = Vec::new();
        log(&format!("camera test started device={device}"));
        if let Some(mut stdout) = child.stdout.take() {
            while !stop.load(Ordering::Relaxed) {
                match stdout.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        pending.extend_from_slice(&buffer[..n]);
                        while let Some(frame) = take_jpeg(&mut pending) {
                            if let Ok(image) = image::load_from_memory(&frame) {
                                let rgba = image.to_rgba8();
                                let (width, height) = rgba.dimensions();
                                let pixels = rune::Pixels::new();
                                pixels.set(width, height, rgba.into_raw());
                                let slot = CAMERA_TEST_FRAME.get_or_init(|| Mutex::new(None));
                                if let Ok(mut current) = slot.lock() {
                                    *current = Some(pixels);
                                }
                                CAMERA_TEST_VERSION.fetch_add(1, Ordering::Relaxed);
                                log(&format!("camera test frame {width}x{height}"));
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        }
        let _ = child.kill();
        log("camera test stopped");
    });
}

pub fn stop_camera_test() {
    if let Some(slot) = CAMERA_TEST_STOP.get() {
        if let Ok(mut current) = slot.lock() {
            if let Some(stop) = current.take() {
                stop.store(true, Ordering::Relaxed);
            }
        }
    }
    if let Some(slot) = CAMERA_TEST_FRAME.get() {
        if let Ok(mut current) = slot.lock() {
            *current = None;
        }
    }
}

fn microphone() -> Option<String> {
    if let Some(device) = selected_input_device() {
        return Some(device);
    }
    let output = Command::new(ffmpeg_path())
        .args([
            "-hide_banner",
            "-list_devices",
            "true",
            "-f",
            "dshow",
            "-i",
            "dummy",
        ])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stderr);
    for line in text.lines() {
        if line.contains("(audio)") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    return Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

fn receive_stream(
    user: Id,
    rx: Receiver<Vec<u8>>,
    stop: Arc<AtomicBool>,
    deafened: Arc<AtomicBool>,
    test: bool,
    stream_stop: Arc<AtomicBool>,
) {
    let mut decoder = match Command::new(ffmpeg_path())
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "ogg",
            "-i",
            "pipe:0",
            "-f",
            "s16le",
            "-ar",
            "48000",
            "-ac",
            "1",
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            log(&format!("decoder error: {error}"));
            return;
        }
    };
    let mut decoder_in = match decoder.stdin.take() {
        Some(stdin) => stdin,
        None => return,
    };
    let stop_writer = stop.clone();
    thread::spawn(move || {
        while let Ok(chunk) = rx.recv() {
            if stop_writer.load(Ordering::Relaxed) || stream_stop.load(Ordering::Relaxed) {
                break;
            }
            if decoder_in.write_all(&chunk).is_err() {
                break;
            }
        }
        log("audio decoder input closed");
    });

    if test {
        let mut stdout = match decoder.stdout.take() {
            Some(stdout) => stdout,
            None => return,
        };
        let mut buffer = [0u8; 1920];
        let mut counter = 0u32;
        while !stop.load(Ordering::Relaxed) {
            match stdout.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if deafened.load(Ordering::Relaxed) {
                        continue;
                    }
                    counter += 1;
                    if counter % 10 == 0 {
                        let rms = buffer[..n]
                            .chunks_exact(2)
                            .map(|sample| {
                                let value = i16::from_le_bytes([sample[0], sample[1]]) as f64;
                                value * value
                            })
                            .sum::<f64>()
                            / (n as f64 / 2.0).max(1.0);
                        log(&format!("user={user} rms={:.0} bytes={n}", rms.sqrt()));
                    }
                }
                Err(_) => break,
            }
        }
    } else {
        enum Playback {
            Native(crate::wasapi::WasapiPlayer),
            Ffplay(Child, std::process::ChildStdin),
        }

        let device = selected_output_device().unwrap_or_default();
        let mut playback = None;
        #[cfg(windows)]
        {
            playback =
                crate::wasapi::WasapiPlayer::start(&device, stop.clone()).map(Playback::Native);
        }
        if playback.is_none() {
            log(if device.is_empty() {
                "native WASAPI output unavailable; falling back to FFplay on the default endpoint"
            } else {
                "native WASAPI output unavailable for the selected device; falling back to FFplay"
            });
            let child = Command::new(ffplay_path())
                .args([
                    "-hide_banner",
                    "-loglevel",
                    "quiet",
                    "-nodisp",
                    "-autoexit",
                    "-f",
                    "s16le",
                    "-ar",
                    "48000",
                    "-ch_layout",
                    "mono",
                    "-",
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            if let Ok(mut child) = child {
                if let Some(stdin) = child.stdin.take() {
                    playback = Some(Playback::Ffplay(child, stdin));
                }
            }
        }
        let Some(mut playback) = playback else {
            log("player error: no audio output available");
            return;
        };
        log(&format!(
            "playback device: {} ({})",
            if device.is_empty() {
                "Default"
            } else {
                &device
            },
            if matches!(&playback, Playback::Native(_)) {
                "native WASAPI"
            } else {
                "FFplay fallback"
            }
        ));
        if let Some(mut stdout) = decoder.stdout.take() {
            log(&format!("playback started for user={user}"));
            let mut buffer = [0u8; 1920];
            let mut bytes = 0u64;
            while !stop.load(Ordering::Relaxed) {
                match stdout.read(&mut buffer) {
                    Ok(0) => {
                        log("playback decoder output ended");
                        break;
                    }
                    Ok(n) => {
                        bytes += n as u64;
                        let payload = scale_pcm(&buffer[..n], output_volume());
                        let deaf = deafened.load(Ordering::Relaxed);
                        match &mut playback {
                            Playback::Native(sink) => sink.write(&payload, deaf),
                            Playback::Ffplay(_, stdin) => {
                                if !deaf && stdin.write_all(&payload).is_err() {
                                    log("playback player stdin closed");
                                    break;
                                }
                            }
                        }
                    }
                    Err(error) => {
                        log(&format!("playback decoder read error: {error}"));
                        break;
                    }
                }
            }
            log(&format!("playback stopped for user={user} bytes={bytes}"));
        }
        match playback {
            Playback::Native(sink) => sink.finish(),
            Playback::Ffplay(mut child, _) => {
                let _ = child.kill();
            }
        }
    }
    let _ = decoder.kill();
}

fn attach_video(user: Id, frame: Vec<u8>, session_stop: &Arc<AtomicBool>) {
    let decoders = VIDEO_DECODERS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = match decoders.lock() {
        Ok(map) => map,
        Err(_) => return,
    };
    if let Some(decoder) = map.get(&user) {
        if let Ok(mut latest) = decoder.latest.lock() {
            *latest = Some(frame);
        }
        let _ = decoder.signal.try_send(());
        return;
    }
    let (native_width, native_height) = jpeg_dimensions(&frame).unwrap_or((1280, 720));
    let latest = Arc::new(Mutex::new(Some(frame)));
    let (signal, signal_rx) = mpsc::sync_channel(1);
    let decoder_stop = Arc::new(AtomicBool::new(false));
    map.insert(
        user,
        VideoDecoder {
            latest: latest.clone(),
            signal: signal.clone(),
            stop: decoder_stop.clone(),
        },
    );
    drop(map);
    let session_stop = session_stop.clone();
    thread::spawn(move || {
        video_decoder(
            user,
            latest,
            signal_rx,
            session_stop,
            decoder_stop,
            native_width,
            native_height,
        )
    });
}

fn video_decoder(
    user: Id,
    latest: Arc<Mutex<Option<Vec<u8>>>>,
    signal: mpsc::Receiver<()>,
    session_stop: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    native_width: u32,
    native_height: u32,
) {
    let capped = std::env::var("RUNECORD_VIDEO_DECODE_CAP")
        .ok()
        .and_then(|value| parse_size(&value));
    let (width, height) = capped.unwrap_or((native_width, native_height));
    let mut args: Vec<String> = vec![
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-fflags".into(),
        "nobuffer".into(),
        "-flags".into(),
        "low_delay".into(),
        "-probesize".into(),
        "32".into(),
        "-analyzeduration".into(),
        "0".into(),
        "-f".into(),
        "image2pipe".into(),
        "-vcodec".into(),
        "mjpeg".into(),
        "-i".into(),
        "pipe:0".into(),
    ];
    if capped.is_some() {
        args.extend([
            "-vf".into(),
            format!(
                "scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2"
            ),
        ]);
    }
    args.extend([
        "-f".into(),
        "rawvideo".into(),
        "-pix_fmt".into(),
        "rgba".into(),
        "-flush_packets".into(),
        "1".into(),
        "pipe:1".into(),
    ]);
    log(&format!(
        "video decoder user={user} native={native_width}x{native_height} output={width}x{height}"
    ));
    let mut child = match Command::new(ffmpeg_path())
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            log(&format!("video decoder unavailable: {error}"));
            video_decoder_fallback(user, latest, signal, session_stop, stop);
            return;
        }
    };
    let (Some(mut stdin), Some(mut stdout)) = (child.stdin.take(), child.stdout.take()) else {
        let _ = child.kill();
        return;
    };

    // Write the newest pending JPEG continuously so the demuxer can read
    // ahead; reading in this thread lets FFmpeg decode at its own pace.
    let writer_latest = latest.clone();
    let writer_stop = stop.clone();
    let writer_session = session_stop.clone();
    let writer = thread::spawn(move || {
        loop {
            if writer_stop.load(Ordering::Relaxed) || writer_session.load(Ordering::Relaxed) {
                break;
            }
            let frame = writer_latest
                .lock()
                .ok()
                .and_then(|mut latest| latest.take());
            if let Some(jpeg) = frame {
                if stdin.write_all(&jpeg).is_err() {
                    break;
                }
                continue;
            }
            match signal.recv_timeout(Duration::from_millis(100)) {
                Ok(()) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    let mut buffer = vec![0u8; (width * height * 4) as usize];
    let mut logged = false;
    loop {
        if stop.load(Ordering::Relaxed) || session_stop.load(Ordering::Relaxed) {
            break;
        }
        if stdout.read_exact(&mut buffer).is_err() {
            break;
        }
        if !logged {
            logged = true;
            log(&format!("video decoded user={user} {}x{}", width, height));
        }
        store_decoded(user, width, height, &buffer);
    }
    let _ = child.kill();
    let _ = writer.join();
    log(&format!("video decoder stopped user={user}"));
}

fn video_decoder_fallback(
    user: Id,
    latest: Arc<Mutex<Option<Vec<u8>>>>,
    signal: mpsc::Receiver<()>,
    session_stop: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
) {
    loop {
        if stop.load(Ordering::Relaxed) || session_stop.load(Ordering::Relaxed) {
            break;
        }
        match signal.recv_timeout(Duration::from_millis(200)) {
            Ok(()) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        let jpeg = latest.lock().ok().and_then(|mut latest| latest.take());
        let Some(jpeg) = jpeg else {
            continue;
        };
        if let Ok(image) = image::load_from_memory(&jpeg) {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            store_decoded(user, width, height, &rgba.into_raw());
        }
    }
}

fn note_status(user: Id, width: u32, height: u32, fps: u32) {
    let meters = VIDEO_METERS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut meters) = meters.lock() {
        let now = std::time::Instant::now();
        let meter = meters.entry(user).or_insert(VideoMeter {
            width,
            height,
            frames: 0,
            started: now,
            fps,
            target: fps,
            slow: 0,
        });
        meter.width = width;
        meter.height = height;
        meter.fps = fps;
        meter.target = fps;
        meter.frames = 0;
        meter.started = now;
    }
    wake();
}

fn stop_preview(user: Id) {
    if let Some(frames) = VIDEO_LATEST.get() {
        if let Ok(mut frames) = frames.lock() {
            frames.remove(&user);
        }
    }
    if let Some(meters) = VIDEO_METERS.get() {
        if let Ok(mut meters) = meters.lock() {
            meters.remove(&user);
        }
    }
    if let Some(decoders) = VIDEO_DECODERS.get() {
        if let Ok(mut decoders) = decoders.lock() {
            if let Some(decoder) = decoders.remove(&user) {
                decoder.stop.store(true, Ordering::Relaxed);
            }
        }
    }
    if let Some(stopped) = VIDEO_STOPPED.get() {
        if let Ok(mut stopped) = stopped.lock() {
            stopped.push(user);
        }
    }
    wake();
}

fn store_decoded(user: Id, width: u32, height: u32, rgba: &[u8]) {
    let frames = VIDEO_LATEST.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut frames) = frames.lock() {
        frames.insert(
            user,
            VideoFrame {
                width,
                height,
                rgba: rgba.to_vec(),
            },
        );
    }
    let meters = VIDEO_METERS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut meters) = meters.lock() {
        let now = std::time::Instant::now();
        let Some(meter) = meters.get_mut(&user) else {
            wake();
            return;
        };
        meter.frames += 1;
        let elapsed = meter.started.elapsed().as_secs_f64();
        if elapsed >= 2.0 {
            meter.fps = (meter.frames as f64 / elapsed).round() as u32;
            log(&format!(
                "video decoded user={user} fps={} frames={} elapsed={elapsed:.2}",
                meter.fps, meter.frames
            ));
            if current_viewer_quality() == ViewQuality::Auto && meter.target >= 24 && meter.fps < 15
            {
                meter.slow += 1;
                if meter.slow >= 2 {
                    log("simulcast: auto switching to data saver");
                    set_viewer_quality(ViewQuality::Low);
                }
            } else {
                meter.slow = 0;
            }
            meter.frames = 0;
            meter.started = now;
        }
    }
    wake();
}

fn capture_args(kind: &str, quality: VideoQuality) -> Vec<String> {
    let (width, height) = quality.resolution();
    let fps = quality.fps().to_string();
    let synthetic = std::env::var("RUNECORD_VIDEO_SYNTH").ok().as_deref() == Some("1");
    let mut args: Vec<String> = vec!["-hide_banner".into(), "-loglevel".into(), "error".into()];
    if synthetic {
        args.extend([
            "-re".into(),
            "-f".into(),
            "lavfi".into(),
            "-i".into(),
            format!("testsrc=size={width}x{height}:rate={fps}"),
        ]);
    } else if kind == "screen" {
        args.extend([
            "-f".into(),
            "gdigrab".into(),
            "-framerate".into(),
            fps.clone(),
            "-i".into(),
            "desktop".into(),
        ]);
    } else {
        let Some(device) = camera() else {
            log("no camera found");
            return args;
        };
        log(&format!("camera device: {device}"));
        args.extend([
            "-f".into(),
            "dshow".into(),
            "-framerate".into(),
            fps.clone(),
            "-i".into(),
            format!("video={device}"),
        ]);
    }
    args.extend([
        "-an".into(),
        "-vf".into(),
        format!("scale={width}:-2"),
        "-r".into(),
        fps,
        "-q:v".into(),
        quality.qscale().to_string(),
        "-b:v".into(),
        format!("{}k", quality.target_kbps()),
        "-f".into(),
        "mjpeg".into(),
        "pipe:1".into(),
    ]);
    args
}

fn video_capture(
    out: mpsc::SyncSender<(u8, Vec<u8>)>,
    kind: String,
    stop: Arc<AtomicBool>,
    video_stop: Arc<AtomicBool>,
    quality: VideoQuality,
    low: bool,
) {
    let auto = quality == VideoQuality::Auto && !low;
    let mut preset = if low {
        VideoQuality::P360
    } else if auto {
        VideoQuality::P720
    } else {
        quality
    };
    let frame_kind: u8 = if low { 4 } else { 1 };
    let status_kind: u8 = if low { 5 } else { 3 };
    loop {
        let (width, height) = preset.resolution();
        log(&format!(
            "video capture: {} {}x{}@{}fps q{} target={}kbps",
            preset.label(),
            width,
            height,
            preset.fps(),
            preset.qscale(),
            preset.target_kbps()
        ));
        let mut child = match Command::new(ffmpeg_path())
            .args(capture_args(&kind, preset))
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                log(&format!("video error: {error}"));
                return;
            }
        };
        let (status_width, status_height) = preset.resolution();
        let status = format!("{status_width}x{status_height}@{}", preset.fps()).into_bytes();
        let _ = out.try_send((status_kind, status.clone()));
        if !low {
            if let Some(user) = local_user() {
                if let Some((width, height, fps)) = parse_status(&status) {
                    note_status(user, width, height, fps);
                    log(&format!(
                        "self preview started user={user} at {}x{}@{}fps",
                        width, height, fps
                    ));
                }
            }
        }
        log(&format!("video source: {kind}"));
        let mut restart = false;
        if let Some(mut stdout) = child.stdout.take() {
            let mut buffer = [0u8; 65536];
            let mut pending: Vec<u8> = Vec::new();
            let mut sent = 0u32;
            let mut total = 0u64;
            let mut dropped = 0u32;
            let mut window = std::time::Instant::now();
            let mut changed_at = std::time::Instant::now();
            while !stop.load(Ordering::Relaxed) && !video_stop.load(Ordering::Relaxed) {
                match stdout.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        pending.extend_from_slice(&buffer[..n]);
                        while let Some(frame) = take_jpeg(&mut pending) {
                            if !low {
                                if let Some(user) = local_user() {
                                    attach_video(user, frame.clone(), &stop);
                                }
                            }
                            match out.try_send((frame_kind, frame)) {
                                Ok(()) => {
                                    sent += 1;
                                    total += 1;
                                    if total == 1 || total % 60 == 0 {
                                        log(&format!(
                                            "video sent preset={} frames={total}",
                                            preset.label()
                                        ));
                                    }
                                }
                                Err(mpsc::TrySendError::Full(_)) => dropped += 1,
                                Err(mpsc::TrySendError::Disconnected(_)) => {
                                    let _ = child.kill();
                                    return;
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
                if window.elapsed() >= Duration::from_secs(2) {
                    let total = sent + dropped;
                    if auto && total > 0 {
                        let ratio = dropped as f64 / total as f64;
                        log(&format!(
                            "video window preset={} sent={sent} dropped={dropped} drop_ratio={ratio:.2}",
                            preset.label()
                        ));
                        let down = preset.step_down();
                        let up = preset.step_up();
                        if ratio > 0.2 && down != preset {
                            preset = down;
                            restart = true;
                        } else if ratio == 0.0
                            && up != preset
                            && changed_at.elapsed() >= Duration::from_secs(12)
                        {
                            preset = up;
                            restart = true;
                        }
                    }
                    sent = 0;
                    dropped = 0;
                    window = std::time::Instant::now();
                    if restart {
                        changed_at = std::time::Instant::now();
                        break;
                    }
                }
            }
        }
        let _ = child.kill();
        if !restart || stop.load(Ordering::Relaxed) || video_stop.load(Ordering::Relaxed) {
            break;
        }
        log(&format!(
            "video quality changed: {} {}x{}@{}fps",
            preset.label(),
            preset.resolution().0,
            preset.resolution().1,
            preset.fps()
        ));
    }
    let _ = out.send((2, Vec::new()));
    if !low {
        if let Some(user) = local_user() {
            stop_preview(user);
            log(&format!("self preview stopped user={user}"));
        }
    }
    log("video stopped");
}

fn camera() -> Option<String> {
    if let Some(device) = selected_camera_device() {
        return Some(device);
    }
    let output = Command::new(ffmpeg_path())
        .args([
            "-hide_banner",
            "-list_devices",
            "true",
            "-f",
            "dshow",
            "-i",
            "dummy",
        ])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stderr);
    for line in text.lines() {
        if line.contains("(video)") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    return Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

fn take_jpeg(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    let start = buffer.windows(2).position(|pair| pair == [0xFF, 0xD8])?;
    if start > 0 {
        buffer.drain(..start);
    }
    let end = buffer.windows(2).position(|pair| pair == [0xFF, 0xD9])?;
    Some(buffer.drain(..end + 2).collect())
}

fn ffplay_path() -> String {
    std::env::var("RUNECORD_FFPLAY").unwrap_or_else(|_| "ffplay".to_string())
}

fn scale_pcm(samples: &[u8], volume: f32) -> Vec<u8> {
    if (volume - 1.0).abs() < f32::EPSILON {
        return samples.to_vec();
    }
    let mut out = samples.to_vec();
    for pair in out.chunks_exact_mut(2) {
        let value = i16::from_le_bytes([pair[0], pair[1]]) as f32 * volume;
        let value = value.clamp(-32768.0, 32767.0) as i16;
        pair.copy_from_slice(&value.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        ReceiveStream, jpeg_dimensions, mix_into, ogg_serial, parse_size, pcm_rms, soundboard_pcm,
        soundboard_sounds, stream_is_current, take_ogg_page,
    };
    use std::collections::HashMap;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, mpsc};

    #[test]
    fn soundboard_mixes_with_saturation_and_consumes_the_queue() {
        let mut pcm = 1000i16.to_le_bytes().to_vec();
        pcm.extend_from_slice(&(-1000i16).to_le_bytes());
        let mut queue = vec![3000i16, -32_000i16, 500];
        assert!(mix_into(&mut pcm, &mut queue));
        assert_eq!(i16::from_le_bytes([pcm[0], pcm[1]]), 4000);
        assert_eq!(i16::from_le_bytes([pcm[2], pcm[3]]), i16::MIN);
        assert_eq!(queue, vec![500]);
        assert!(!mix_into(&mut [0u8; 4], &mut Vec::new()));
    }

    #[test]
    fn every_soundboard_effect_builds_audible_pcm() {
        for (name, _, seconds) in soundboard_sounds() {
            let pcm = soundboard_pcm(name);
            assert_eq!(pcm.len(), (48_000.0 * seconds) as usize, "{name} length");
            let peak = pcm.iter().map(|s| (*s as i32).abs()).max().unwrap_or(0);
            assert!(peak > 3000, "{name} peak {peak}");
        }
        assert!(soundboard_pcm("missing").is_empty());
    }

    #[test]
    fn pcm_rms_measures_silence_and_full_scale() {
        assert_eq!(pcm_rms(&[0, 0, 0, 0]), 0.0);
        let full = 32767i16.to_le_bytes();
        let rms = pcm_rms(&[full[0], full[1], full[0], full[1]]);
        assert!((rms - 1.0).abs() < 0.01);
    }

    fn page(flags: u8, body: &[u8]) -> Vec<u8> {
        let mut out = Vec::from(*b"OggS");
        out.push(0);
        out.push(flags);
        out.extend_from_slice(&[0u8; 8]);
        out.extend_from_slice(&[0u8; 4]);
        out.extend_from_slice(&[0u8; 4]);
        out.extend_from_slice(&[0u8; 4]);
        out.push(1);
        out.push(body.len() as u8);
        out.extend_from_slice(body);
        out
    }

    #[test]
    fn viewer_quality_keys_and_parse() {
        use super::ViewQuality;

        assert_eq!(ViewQuality::parse("high"), ViewQuality::High);
        assert_eq!(ViewQuality::parse("LOW"), ViewQuality::Low);
        assert_eq!(ViewQuality::parse("auto"), ViewQuality::Auto);
        assert_eq!(ViewQuality::parse("nonsense"), ViewQuality::Auto);
        assert_eq!(ViewQuality::High.label(), "High");
        assert_eq!(ViewQuality::Low.label(), "Data saver");
        assert_eq!(ViewQuality::Low.key(), "low");
    }

    #[test]
    fn video_quality_presets_scale_and_parse() {
        use super::VideoQuality;

        assert_eq!(VideoQuality::parse("480p"), VideoQuality::P480);
        assert_eq!(VideoQuality::parse(" AUTO "), VideoQuality::Auto);
        assert_eq!(VideoQuality::parse("nonsense"), VideoQuality::Auto);
        assert_eq!(VideoQuality::P360.resolution(), (640, 360));
        assert_eq!(VideoQuality::P1080.resolution(), (1920, 1080));
        assert_eq!(VideoQuality::P480.fps(), 24);
        assert_eq!(VideoQuality::P720.qscale(), 4);
        assert_eq!(VideoQuality::P1080.target_kbps(), 3500);
        assert_eq!(VideoQuality::P720.step_down(), VideoQuality::P480);
        assert_eq!(VideoQuality::P480.step_down(), VideoQuality::P360);
        assert_eq!(VideoQuality::P360.step_down(), VideoQuality::P360);
        assert_eq!(VideoQuality::P360.step_up(), VideoQuality::P480);
        assert_eq!(VideoQuality::P720.step_up(), VideoQuality::P720);
    }

    #[test]
    fn reads_jpeg_dimensions_and_cap_sizes() {
        use image::{Rgb, RgbImage};
        let image = RgbImage::from_pixel(1280, 720, Rgb([1, 2, 3]));
        let mut bytes = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Jpeg,
            )
            .expect("jpeg");
        assert_eq!(jpeg_dimensions(&bytes), Some((1280, 720)));
        assert_eq!(parse_size("640x360"), Some((640, 360)));
        assert_eq!(parse_size("1920X1080"), Some((1920, 1080)));
        assert_eq!(parse_size("bad"), None);
        assert_eq!(jpeg_dimensions(b"not a jpeg"), None);
    }

    #[test]
    fn reads_ogg_serial_and_detects_header_refreshes() {
        let mut page = page(0x02, b"head");
        page[14..18].copy_from_slice(&77u32.to_le_bytes());
        assert_eq!(ogg_serial(&page), Some(77));
        assert!(ogg_serial(b"not ogg").is_none());

        let mut receivers = HashMap::new();
        assert!(!stream_is_current(&receivers, 1, 77));
        let (tx, _rx) = mpsc::channel();
        receivers.insert(
            1,
            ReceiveStream {
                tx,
                stop: Arc::new(AtomicBool::new(false)),
                serial: 77,
            },
        );
        assert!(stream_is_current(&receivers, 1, 77));
        assert!(!stream_is_current(&receivers, 1, 78));
    }

    #[test]
    fn splits_streamed_ogg_pages() {
        let first = page(0x02, b"head");
        let second = page(0x00, b"tags-and-audio");
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&first);
        buffer.extend_from_slice(&second);
        assert_eq!(
            take_ogg_page(&mut buffer).as_deref(),
            Some(first.as_slice())
        );
        assert_eq!(
            take_ogg_page(&mut buffer).as_deref(),
            Some(second.as_slice())
        );
        assert!(take_ogg_page(&mut buffer).is_none());
    }
}
