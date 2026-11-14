//! HTTP command handlers: media.

use super::super::*;
use std::sync::Arc;

pub(crate) fn video(shared: &Arc<Shared>, channel: Option<Id>, kind: String) {
    let token = shared.token.lock().ok().and_then(|t| t.clone());
    if let Some(token) = token {
        crate::voice::set_video(&shared.base, &token, channel, &kind);
    }
}

pub(crate) fn upload_user_image(shared: &Arc<Shared>, banner: bool, path: std::path::PathBuf) {
    let kind = if banner { "banner" } else { "avatar" };
    match std::fs::read(&path) {
        Ok(bytes) => {
            let token = shared.token.lock().ok().and_then(|t| t.clone());
            let mut req = shared
                .client
                .post(format!("{}/api/me/{kind}", shared.base))
                .header("Content-Type", "application/octet-stream")
                .body(bytes);
            if let Some(t) = token {
                req = req.bearer_auth(t);
            }
            match req.send() {
                Ok(res) if res.status().is_success() => match res.json::<User>() {
                    Ok(user) => shared.emit(NetEvent::UserUpdate(Box::new(user))),
                    Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                },
                Ok(res) => shared.emit(NetEvent::Error(error_text(res))),
                Err(e) => shared.emit(NetEvent::Error(format!(
                    "Can't reach the local server: {e}"
                ))),
            }
        }
        Err(e) => shared.emit(NetEvent::Error(format!("Could not read image: {e}"))),
    }
}

pub(crate) fn remove_user_image(shared: &Arc<Shared>, banner: bool) {
    let kind = if banner { "banner" } else { "avatar" };
    match shared.delete_json(&format!("/api/me/{kind}")) {
        Ok(v) => match serde_json::from_value::<User>(v) {
            Ok(user) => shared.emit(NetEvent::UserUpdate(Box::new(user))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn fetch_image(shared: &Arc<Shared>, url: String) {
    if let Ok(res) = shared.client.get(format!("{}{}", shared.base, url)).send() {
        if let Ok(bytes) = res.bytes() {
            if let Some(animation) = crate::image_anim::decode(&bytes) {
                let first = animation.frames[0].rgba.clone();
                shared.emit(NetEvent::Image {
                    url,
                    width: animation.width,
                    height: animation.height,
                    rgba: first,
                    frames: animation.frames,
                });
            } else {
                match image::load_from_memory(&bytes) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        let (w, h) = rgba.dimensions();
                        shared.emit(NetEvent::Image {
                            url,
                            width: w,
                            height: h,
                            rgba: rgba.into_raw(),
                            frames: Vec::new(),
                        });
                    }
                    Err(e) => shared.emit(NetEvent::Error(format!("Image: {e}"))),
                }
            }
        }
    }
}

pub(crate) fn upload(shared: &Arc<Shared>, channel: Id, path: std::path::PathBuf) {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".into());
    let (w, h) = image::image_dimensions(&path).unwrap_or((0, 0));
    let content_type = match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    };
    match std::fs::read(&path) {
        Ok(bytes) => {
            let token = shared.token.lock().ok().and_then(|t| t.clone());
            let res = shared
                .client
                .post(format!(
                    "{}{}",
                    shared.base,
                    format!("/api/channels/{channel}/upload")
                ))
                .query(&[
                    ("name", name),
                    ("content_type", content_type.to_string()),
                    ("width", w.to_string()),
                    ("height", h.to_string()),
                ])
                .bearer_auth(token.unwrap_or_default())
                .body(bytes)
                .send();
            match res {
                Ok(r) if r.status().is_success() => match r.json::<UploadResponse>() {
                    Ok(u) => shared.emit(NetEvent::Uploaded(Box::new(u.attachment))),
                    Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                },
                Ok(r) => shared.emit(NetEvent::Error(error_text(r))),
                Err(e) => shared.emit(NetEvent::Error(e.to_string())),
            }
        }
        Err(e) => shared.emit(NetEvent::Error(format!("Could not read file: {e}"))),
    }
}
