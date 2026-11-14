use super::*;

pub(crate) fn soundboard_sounds() -> [(&'static str, &'static str, f32); 6] {
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

pub(crate) fn play_soundboard(name: &str) {
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

pub(crate) fn mix_soundboard(pcm: &mut [u8]) -> bool {
    let queue = SOUNDBOARD.get_or_init(|| Mutex::new(Vec::new()));
    match queue.lock() {
        Ok(mut queue) => mix_into(pcm, &mut queue),
        Err(_) => false,
    }
}

/// Mixes queued samples into little-endian s16 stereo-pair PCM, consuming what it used.

pub(crate) fn mix_into(pcm: &mut [u8], queue: &mut Vec<i16>) -> bool {
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

pub(crate) fn soundboard_pcm(name: &str) -> Vec<i16> {
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

pub(crate) fn attack_release(t: f32, duration: f32, attack: f32, release: f32) -> f32 {
    let rise = (t / attack.max(0.001)).min(1.0);
    let fall = ((duration - t) / release.max(0.001)).clamp(0.0, 1.0);
    rise * fall
}
