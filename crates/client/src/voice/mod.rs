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

mod camera;
mod devices;
mod mic;
/// Capture presets exposed in the voice stage. `Auto` starts at 720p and steps
/// down when the outbound frame queue backs up, then steps back up after a
/// stable period.
/// What the receiver asks to see. `Auto` starts high and drops to the low
/// simulcast layer if its decoder cannot keep up.
mod soundboard;
mod video;
pub(crate) use camera::*;
pub(crate) use devices::*;
pub(crate) use mic::*;
pub(crate) use soundboard::*;
pub(crate) use video::*;

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
    if let Some(slot) = WAKER.get()
        && let Ok(current) = slot.lock()
        && let Some(waker) = current.as_ref()
    {
        waker();
    }
}

pub fn set_quality(quality: VideoQuality) {
    let slot = QUALITY.get_or_init(|| Mutex::new(VideoQuality::Auto));
    if let Ok(mut current) = slot.lock() {
        *current = quality;
    }
    if let Some(session) = SESSION.get()
        && let Ok(current) = session.lock()
        && let Some(session) = current.as_ref()
    {
        let _ = session.video.send(VideoCommand {
            kind: None,
            quality: Some(quality),
        });
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

static VIDEO_LATEST: OnceLock<Mutex<HashMap<Id, VideoFrame>>> = OnceLock::new();

static VIDEO_STOPPED: OnceLock<Mutex<Vec<Id>>> = OnceLock::new();

pub(crate) struct ReceiveStream {
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
    if let Some(session) = current.as_ref()
        && channel == Some(session.channel)
    {
        let was_muted = session.muted.load(Ordering::Relaxed);
        let was_deafened = session.deafened.load(Ordering::Relaxed);
        session.muted.store(mute, Ordering::Relaxed);
        session.deafened.store(deaf, Ordering::Relaxed);
        if was_muted != mute || was_deafened != deaf {
            log(&format!("mute={mute} deaf={deaf}"));
        }
        return;
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
    if let Some(slot) = SESSION.get()
        && let Ok(current) = slot.lock()
        && let Some(session) = current.as_ref()
    {
        let _ = session.video.send(VideoCommand {
            kind: Some(kind.to_string()),
            quality: None,
        });
    }
}

pub fn shutdown() {
    if let Some(slot) = SESSION.get()
        && let Ok(mut current) = slot.lock()
        && let Some(session) = current.take()
    {
        session.stop.store(true, Ordering::Relaxed);
    }
}

fn log(message: &str) {
    if let Ok(path) = std::env::var("RUNECORD_VOICE_LOG")
        && let Ok(mut file) = std::fs::OpenOptions::new()
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
            if let Some(decoders) = VIDEO_DECODERS.get()
                && let Ok(mut decoders) = decoders.lock()
            {
                for (_, decoder) in decoders.drain() {
                    decoder.stop.store(true, Ordering::Relaxed);
                }
            }
            if let Some(frames) = VIDEO_LATEST.get()
                && let Ok(mut frames) = frames.lock()
            {
                frames.clear();
            }
            if let Some(meters) = VIDEO_METERS.get()
                && let Ok(mut meters) = meters.lock()
            {
                meters.clear();
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
                        if let Some(meters) = VIDEO_METERS.get()
                            && let Ok(mut meters) = meters.lock()
                        {
                            meters.remove(&user);
                        }
                        if let Some(decoders) = VIDEO_DECODERS.get()
                            && let Ok(mut decoders) = decoders.lock()
                            && let Some(decoder) = decoders.remove(&user)
                        {
                            decoder.stop.store(true, Ordering::Relaxed);
                        }
                        wake();
                        log(&format!("video stopped user={user}"));
                    } else if (kind == 1 && viewer != ViewQuality::Low)
                        || (kind == 4 && viewer == ViewQuality::Low)
                    {
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
    if let Some(decoders) = VIDEO_DECODERS.get()
        && let Ok(mut decoders) = decoders.lock()
    {
        for (_, decoder) in decoders.drain() {
            decoder.stop.store(true, Ordering::Relaxed);
        }
    }
    log(&format!("left channel {channel}"));
}

fn ffmpeg_path() -> String {
    std::env::var("RUNECORD_FFMPEG").unwrap_or_else(|_| "ffmpeg".to_string())
}

fn capture_encoder(
    out: Sender<(u8, Vec<u8>)>,
    stop: Arc<AtomicBool>,
    muted: Arc<AtomicBool>,
    synth: bool,
) {
    if let Some(queue) = SOUNDBOARD.get()
        && let Ok(mut queue) = queue.lock()
    {
        queue.clear();
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
                for chunk in samples.as_chunks_mut::<2>().0 {
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

static SOUNDBOARD: OnceLock<Mutex<Vec<i16>>> = OnceLock::new();

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
                    if counter.is_multiple_of(10) {
                        let rms = buffer[..n]
                            .as_chunks::<2>()
                            .0
                            .iter()
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
        #[cfg(windows)]
        let mut playback =
            crate::wasapi::WasapiPlayer::start(&device, stop.clone()).map(Playback::Native);
        #[cfg(not(windows))]
        let mut playback: Option<Playback> = None;
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
            if let Ok(mut child) = child
                && let Some(stdin) = child.stdin.take()
            {
                playback = Some(Playback::Ffplay(child, stdin));
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

fn ffplay_path() -> String {
    std::env::var("RUNECORD_FFPLAY").unwrap_or_else(|_| "ffplay".to_string())
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
