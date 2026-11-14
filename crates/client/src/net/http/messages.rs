//! HTTP command handlers: messages.

use super::super::*;
use std::sync::Arc;

pub(crate) fn load_messages(shared: &Arc<Shared>, channel: Id, before: Option<Id>) {
    let path = match before {
        Some(b) => format!("/api/channels/{channel}/messages?before={b}&limit=50"),
        None => format!("/api/channels/{channel}/messages?limit=50"),
    };
    match shared.get(&path) {
        Ok(v) => match serde_json::from_value::<Vec<Message>>(v) {
            Ok(messages) => shared.emit(NetEvent::Messages {
                channel,
                messages,
                before: before.is_some(),
            }),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn send(
    shared: &Arc<Shared>,
    channel: Id,
    content: String,
    reply_to: Option<Id>,
    attachments: Vec<Id>,
) {
    let body = SendMessage {
        content,
        reply_to,
        attachment_ids: attachments,
        nonce: String::new(),
    };
    match shared.post(&format!("/api/channels/{channel}/messages"), &body) {
        Ok(v) => {
            if let Ok(msg) = serde_json::from_value::<Message>(v) {
                shared.emit(NetEvent::MessageCreate(Box::new(msg)));
            }
        }
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn edit(shared: &Arc<Shared>, message: Id, content: String) {
    match shared.patch(
        &format!("/api/messages/{message}"),
        &EditMessage { content },
    ) {
        Ok(v) => {
            if let Ok(msg) = serde_json::from_value::<Message>(v) {
                shared.emit(NetEvent::MessageUpdate(Box::new(msg)));
            }
        }
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn delete(shared: &Arc<Shared>, message: Id) {
    if let Err(e) = shared.delete(&format!("/api/messages/{message}")) {
        shared.emit(NetEvent::Error(e));
    }
}

pub(crate) fn react(shared: &Arc<Shared>, message: Id, emoji: String) {
    match shared.post(
        &format!("/api/messages/{message}/reactions"),
        &ReactionToggle { emoji },
    ) {
        Ok(v) => {
            if let Ok(msg) = serde_json::from_value::<Message>(v) {
                shared.emit(NetEvent::MessageUpdate(Box::new(msg)));
            }
        }
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn pin(shared: &Arc<Shared>, message: Id) {
    match shared.post(
        &format!("/api/messages/{message}/pin"),
        &serde_json::json!({}),
    ) {
        Ok(v) => {
            if let Ok(msg) = serde_json::from_value::<Message>(v) {
                shared.emit(NetEvent::MessageUpdate(Box::new(msg)));
            }
        }
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn typing(shared: &Arc<Shared>, channel: Id) {
    let _ = shared.post(
        &format!("/api/channels/{channel}/typing"),
        &serde_json::json!({}),
    );
}

pub(crate) fn mark_read(shared: &Arc<Shared>, channel: Id, message: Id) {
    let _ = shared.post(
        "/api/read",
        &ReadUpdate {
            channel_id: channel,
            last_read_id: message,
        },
    );
}

pub(crate) fn search(shared: &Arc<Shared>, q: String) {
    let path = format!("/api/search?q={}", urlencoding::encode(&q));
    match shared.get(&path) {
        Ok(v) => match serde_json::from_value::<SearchResults>(v) {
            Ok(r) => shared.emit(NetEvent::Search(Box::new(r))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_pins(shared: &Arc<Shared>, channel: Id) {
    match shared.get(&format!("/api/channels/{channel}/pins")) {
        Ok(v) => match serde_json::from_value::<Vec<Message>>(v) {
            Ok(messages) => shared.emit(NetEvent::Pins(messages)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn create_thread(shared: &Arc<Shared>, channel: Id, message: Option<Id>, name: String) {
    let body = CreateMessageThread {
        name,
        message_id: message,
    };
    match shared.post(&format!("/api/channels/{channel}/threads"), &body) {
        Ok(v) => match serde_json::from_value::<Channel>(v) {
            Ok(thread) => shared.emit(NetEvent::ThreadCreated(Box::new(thread))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn archive_thread(shared: &Arc<Shared>, channel: Id, archived: bool) {
    let body = serde_json::json!({ "archived": archived });
    match shared.patch(&format!("/api/channels/{channel}/thread"), &body) {
        Ok(v) => match serde_json::from_value::<Channel>(v) {
            Ok(channel) => shared.emit(NetEvent::ChannelUpdated(Box::new(channel))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}
