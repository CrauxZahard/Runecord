//! HTTP command handlers: messages.

use super::super::*;
use std::sync::Arc;

pub(crate) fn load_messages(shared: &Arc<Shared>, channel: Id, before: Option<Id>) {
    let path = match before {
        Some(b) => format!("/api/channels/{channel}/messages?before={b}&limit=50"),
        None => format!("/api/channels/{channel}/messages?limit=50"),
    };
    let done = shared.clone();
    get_then(
        shared,
        path.to_string(),
        move |result: Result<Vec<Message>, String>| match result {
            Ok(messages) => done.emit(NetEvent::Messages {
                channel,
                messages,
                before: before.is_some(),
            }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
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
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/channels/{channel}/messages").to_string(),
        body,
        move |result: Result<Message, String>| match result {
            Ok(msg) => done.emit(NetEvent::MessageCreate(Box::new(msg))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn forward(shared: &Arc<Shared>, message: Id, channel: Id) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/channels/{channel}/messages/{message}/forward").to_string(),
        serde_json::json!({}),
        move |result: Result<Message, String>| match result {
            Ok(msg) => done.emit(NetEvent::MessageCreate(Box::new(msg))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn click_component(shared: &Arc<Shared>, message: Id, custom_id: String) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Post,
        format!("/api/messages/{message}/components/{custom_id}"),
        serde_json::json!({}),
        move |result| {
            if let Err(e) = result {
                done.emit(NetEvent::Error(e));
            }
        },
    );
}

pub(crate) fn create_poll(
    shared: &Arc<Shared>,
    channel: Id,
    question: String,
    options: Vec<String>,
    multiple: bool,
    anonymous: bool,
    duration_hours: Option<u32>,
) {
    let body = CreatePoll {
        question,
        options,
        multiple,
        anonymous,
        duration_hours,
    };
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/channels/{channel}/polls").to_string(),
        body,
        move |result: Result<Message, String>| match result {
            Ok(msg) => done.emit(NetEvent::MessageCreate(Box::new(msg))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn schedule_message(shared: &Arc<Shared>, channel: Id, content: String, send_at: i64) {
    let body = CreateScheduledMessage { content, send_at };
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/channels/{channel}/scheduled").to_string(),
        body,
        move |result: Result<ScheduledMessage, String>| match result {
            Ok(message) => done.emit(NetEvent::ScheduledCreated(message)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_scheduled(shared: &Arc<Shared>, channel: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/channels/{channel}/scheduled").to_string(),
        move |result: Result<Vec<ScheduledMessage>, String>| match result {
            Ok(messages) => done.emit(NetEvent::Scheduled(messages)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn delete_scheduled(shared: &Arc<Shared>, id: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Delete,
        format!("/api/scheduled/{id}"),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => done.emit(NetEvent::ScheduledDeleted(id)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn vote_poll(shared: &Arc<Shared>, message: Id, option: u32) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/messages/{message}/poll/vote").to_string(),
        PollVote { option },
        move |result: Result<Message, String>| match result {
            Ok(msg) => done.emit(NetEvent::MessageUpdate(Box::new(msg))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn edit(shared: &Arc<Shared>, message: Id, content: String) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Patch,
        format!("/api/messages/{message}").to_string(),
        EditMessage { content },
        move |result: Result<Message, String>| match result {
            Ok(msg) => done.emit(NetEvent::MessageUpdate(Box::new(msg))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn delete(shared: &Arc<Shared>, message: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Delete,
        format!("/api/messages/{message}"),
        serde_json::json!({}),
        move |result| {
            if let Err(e) = result {
                done.emit(NetEvent::Error(e));
            }
        },
    );
}

pub(crate) fn react(shared: &Arc<Shared>, message: Id, emoji: String) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/messages/{message}/reactions").to_string(),
        ReactionToggle { emoji },
        move |result: Result<Message, String>| match result {
            Ok(msg) => done.emit(NetEvent::MessageUpdate(Box::new(msg))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn pin(shared: &Arc<Shared>, message: Id) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/messages/{message}/pin").to_string(),
        serde_json::json!({}),
        move |result: Result<Message, String>| match result {
            Ok(msg) => done.emit(NetEvent::MessageUpdate(Box::new(msg))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn typing(shared: &Arc<Shared>, channel: Id) {
    fire_then(
        shared,
        HttpMethod::Post,
        format!("/api/channels/{channel}/typing"),
        serde_json::json!({}),
        |_| {},
    );
}

pub(crate) fn mark_read(shared: &Arc<Shared>, channel: Id, message: Id) {
    fire_then(
        shared,
        HttpMethod::Post,
        "/api/read".to_string(),
        ReadUpdate {
            channel_id: channel,
            last_read_id: message,
        },
        |_| {},
    );
}

pub(crate) fn search(shared: &Arc<Shared>, q: String) {
    let path = format!("/api/search?q={}", crate::net::encode_component(&q));
    let done = shared.clone();
    get_then(
        shared,
        path.to_string(),
        move |result: Result<SearchResults, String>| match result {
            Ok(r) => done.emit(NetEvent::Search(Box::new(r))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_pins(shared: &Arc<Shared>, channel: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/channels/{channel}/pins").to_string(),
        move |result: Result<Vec<Message>, String>| match result {
            Ok(messages) => done.emit(NetEvent::Pins(messages)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn create_thread(
    shared: &Arc<Shared>,
    channel: Id,
    message: Option<Id>,
    name: String,
    tag_ids: Vec<Id>,
) {
    let body = CreateMessageThread {
        name,
        message_id: message,
        tag_ids,
    };
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/channels/{channel}/threads").to_string(),
        body,
        move |result: Result<Channel, String>| match result {
            Ok(thread) => done.emit(NetEvent::ThreadCreated(Box::new(thread))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn set_forum_tags(shared: &Arc<Shared>, channel: Id, tags: Vec<ForumTag>) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Put,
        format!("/api/channels/{channel}/forum-tags").to_string(),
        tags,
        move |result: Result<Vec<ForumTag>, String>| match result {
            Ok(saved) => done.emit(NetEvent::ForumTags(saved)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn set_thread_tags(shared: &Arc<Shared>, thread: Id, tags: Vec<Id>) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Put,
        format!("/api/channels/{thread}/thread-tags").to_string(),
        tags,
        move |result: Result<Channel, String>| match result {
            Ok(channel) => done.emit(NetEvent::ChannelUpdated(Box::new(channel))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn archive_thread(shared: &Arc<Shared>, channel: Id, archived: bool) {
    let body = serde_json::json!({ "archived": archived });
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Patch,
        format!("/api/channels/{channel}/thread").to_string(),
        body,
        move |result: Result<Channel, String>| match result {
            Ok(channel) => done.emit(NetEvent::ChannelUpdated(Box::new(channel))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}
