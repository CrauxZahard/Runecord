//! HTTP command handlers: media.

use super::super::*;
use std::sync::Arc;

pub(crate) fn video(shared: &Arc<Shared>, channel: Option<Id>, kind: String) {
    let token = shared.token.lock().ok().and_then(|t| t.clone());
    if let Some(token) = token {
        crate::voice::set_video(&shared.base, &token, channel, &kind);
    }
}

pub(crate) fn upload_user_image(shared: &Arc<Shared>, banner: bool, source: UploadSource) {
    let kind = if banner { "banner" } else { "avatar" };
    let done = shared.clone();
    upload_then(
        shared,
        format!("/api/me/{kind}"),
        source,
        "application/octet-stream".to_string(),
        move |result: Result<User, String>| match result {
            Ok(user) => done.emit(NetEvent::UserUpdate(Box::new(user))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn remove_user_image(shared: &Arc<Shared>, banner: bool) {
    let kind = if banner { "banner" } else { "avatar" };
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Delete,
        format!("/api/me/{kind}").to_string(),
        serde_json::Value::Null,
        move |result: Result<User, String>| match result {
            Ok(user) => done.emit(NetEvent::UserUpdate(Box::new(user))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn fetch_image(shared: &Arc<Shared>, url: String) {
    fetch_image_then(shared, url);
}

pub(crate) fn upload(shared: &Arc<Shared>, channel: Id, source: UploadSource) {
    let (name, bytes) = match source.read() {
        Ok(read) => read,
        Err(e) => {
            shared.emit(NetEvent::Error(e));
            return;
        }
    };
    let content_type = content_type_for(&name);
    let (width, height) = image::load_from_memory(&bytes)
        .map(|image| {
            let rgba = image.to_rgba8();
            rgba.dimensions()
        })
        .unwrap_or((0, 0));
    let path = format!(
        "/api/channels/{channel}/upload?name={}&content_type={}&width={width}&height={height}",
        crate::net::encode_component(&name),
        crate::net::encode_component(content_type),
    );
    let done = shared.clone();
    upload_then(
        shared,
        path,
        UploadSource::Bytes { name, bytes },
        content_type.to_string(),
        move |result: Result<UploadResponse, String>| match result {
            Ok(response) => done.emit(NetEvent::Uploaded(Box::new(response.attachment))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}
