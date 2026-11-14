use super::*;

pub(crate) fn video_stats() -> HashMap<Id, (u32, u32, u32)> {
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

pub(crate) fn drain_video() -> (Vec<(Id, u32, u32, Vec<u8>)>, Vec<Id>) {
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

pub(crate) fn parse_status(payload: &[u8]) -> Option<(u32, u32, u32)> {
    let text = std::str::from_utf8(payload).ok()?;
    let (dims, fps) = text.split_once('@')?;
    let (width, height) = dims.split_once('x')?;
    Some((width.parse().ok()?, height.parse().ok()?, fps.parse().ok()?))
}

pub(crate) fn ogg_serial(page: &[u8]) -> Option<u32> {
    if page.len() >= 18 && page.starts_with(b"OggS") {
        Some(u32::from_le_bytes([page[14], page[15], page[16], page[17]]))
    } else {
        None
    }
}

pub(crate) fn stream_is_current(
    receivers: &HashMap<Id, ReceiveStream>,
    user: Id,
    serial: u32,
) -> bool {
    receivers
        .get(&user)
        .is_some_and(|stream| stream.serial == serial)
}

pub(crate) fn take_ogg_page(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
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

pub(crate) fn attach_video(user: Id, frame: Vec<u8>, session_stop: &Arc<AtomicBool>) {
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

pub(crate) fn video_decoder(
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

pub(crate) fn video_decoder_fallback(
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

pub(crate) fn note_status(user: Id, width: u32, height: u32, fps: u32) {
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

pub(crate) fn stop_preview(user: Id) {
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

pub(crate) fn store_decoded(user: Id, width: u32, height: u32, rgba: &[u8]) {
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
