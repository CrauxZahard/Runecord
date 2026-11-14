use super::*;

pub(crate) fn parse_size(value: &str) -> Option<(u32, u32)> {
    let (width, height) = value.trim().split_once(['x', 'X'])?;
    let width = width.parse::<u32>().ok()?;
    let height = height.parse::<u32>().ok()?;
    (width > 0 && height > 0).then_some((width, height))
}

pub(crate) fn jpeg_dimensions(jpeg: &[u8]) -> Option<(u32, u32)> {
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

pub(crate) fn camera_test_version() -> u64 {
    CAMERA_TEST_VERSION.load(Ordering::Relaxed)
}

pub(crate) fn camera_test_frame() -> Option<rune::Pixels> {
    CAMERA_TEST_FRAME
        .get()
        .and_then(|slot| slot.lock().ok().and_then(|slot| slot.clone()))
}

pub(crate) fn camera_testing() -> bool {
    CAMERA_TEST_STOP
        .get()
        .and_then(|slot| slot.lock().ok().map(|slot| slot.is_some()))
        .unwrap_or(false)
}

pub(crate) fn start_camera_test() {
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

pub(crate) fn stop_camera_test() {
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

pub(crate) fn capture_args(kind: &str, quality: VideoQuality) -> Vec<String> {
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

pub(crate) fn video_capture(
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

pub(crate) fn take_jpeg(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    let start = buffer.windows(2).position(|pair| pair == [0xFF, 0xD8])?;
    if start > 0 {
        buffer.drain(..start);
    }
    let end = buffer.windows(2).position(|pair| pair == [0xFF, 0xD9])?;
    Some(buffer.drain(..end + 2).collect())
}
