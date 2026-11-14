use super::*;

pub(crate) fn pcm_rms(samples: &[u8]) -> f32 {
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

pub(crate) fn shape_pcm(samples: &mut [u8], muted: &AtomicBool, gate: bool, synth: bool) {
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

pub(crate) fn mic_level() -> f32 {
    f32::from_bits(MIC_LEVEL.load(Ordering::Relaxed))
}

pub(crate) fn mic_testing() -> bool {
    MIC_TEST_STOP
        .get()
        .and_then(|slot| slot.lock().ok().map(|slot| slot.is_some()))
        .unwrap_or(false)
}

pub(crate) fn start_mic_test() {
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

pub(crate) fn stop_mic_test() {
    if let Some(slot) = MIC_TEST_STOP.get() {
        if let Ok(mut current) = slot.lock() {
            if let Some(stop) = current.take() {
                stop.store(true, Ordering::Relaxed);
            }
        }
    }
    MIC_LEVEL.store(0.0f32.to_bits(), Ordering::Relaxed);
}

pub(crate) fn scale_pcm(samples: &[u8], volume: f32) -> Vec<u8> {
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
