use super::*;

pub(crate) fn list_devices() -> (Vec<String>, Vec<String>) {
    let mut audio = Vec::new();
    let mut video = Vec::new();
    let Ok(output) = helper(&ffmpeg_path())
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

pub(crate) fn list_output_devices() -> Vec<String> {
    #[cfg(windows)]
    {
        crate::wasapi::list_output_devices()
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

pub(crate) fn microphone() -> Option<String> {
    if let Some(device) = selected_input_device() {
        return Some(device);
    }
    let output = helper(&ffmpeg_path())
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
        if line.contains("(audio)")
            && let Some(start) = line.find('"')
            && let Some(end) = line[start + 1..].find('"')
        {
            return Some(line[start + 1..start + 1 + end].to_string());
        }
    }
    None
}

pub(crate) fn camera() -> Option<String> {
    if let Some(device) = selected_camera_device() {
        return Some(device);
    }
    let output = helper(&ffmpeg_path())
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
        if line.contains("(video)")
            && let Some(start) = line.find('"')
            && let Some(end) = line[start + 1..].find('"')
        {
            return Some(line[start + 1..start + 1 + end].to_string());
        }
    }
    None
}
