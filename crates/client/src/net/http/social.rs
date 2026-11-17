//! HTTP command handlers: social.

use super::super::*;
use std::sync::Arc;

pub(crate) fn set_privacy(shared: &Arc<Shared>, dm: PrivacyLevel, friend: PrivacyLevel) {
    let body = UpdatePrivacy {
        dm_privacy: dm,
        friend_privacy: friend,
    };
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        "/api/me/privacy".to_string(),
        body,
        move |result: Result<User, String>| match result {
            Ok(user) => done.emit(NetEvent::UserUpdate(Box::new(user))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn rename_dm(shared: &Arc<Shared>, channel: Id, name: String) {
    let body = serde_json::json!({ "name": name });
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Patch,
        format!("/api/dms/{channel}").to_string(),
        body,
        move |result: Result<DmChannel, String>| match result {
            Ok(dm) => done.emit(NetEvent::DmUpdated(Box::new(dm))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn upload_dm_icon(shared: &Arc<Shared>, channel: Id, source: UploadSource) {
    let done = shared.clone();
    upload_then(
        shared,
        format!("/api/dms/{channel}/icon"),
        source,
        "application/octet-stream".to_string(),
        move |result: Result<DmChannel, String>| match result {
            Ok(dm) => done.emit(NetEvent::DmUpdated(Box::new(dm))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn remove_dm_icon(shared: &Arc<Shared>, channel: Id) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Delete,
        format!("/api/dms/{channel}/icon").to_string(),
        serde_json::Value::Null,
        move |result: Result<DmChannel, String>| match result {
            Ok(dm) => done.emit(NetEvent::DmUpdated(Box::new(dm))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn add_dm_member(shared: &Arc<Shared>, channel: Id, user: Id) {
    let body = serde_json::json!({ "user_id": user });
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/dms/{channel}/members").to_string(),
        body,
        move |result: Result<DmChannel, String>| match result {
            Ok(dm) => done.emit(NetEvent::DmUpdated(Box::new(dm))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn remove_dm_member(shared: &Arc<Shared>, channel: Id, user: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Delete,
        format!("/api/dms/{channel}/members/{user}"),
        serde_json::json!({}),
        move |result| {
            if let Err(e) = result {
                done.emit(NetEvent::Error(e));
            }
        },
    );
}

pub(crate) fn leave_dm(shared: &Arc<Shared>, channel: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Post,
        format!("/api/dms/{channel}/leave").to_string(),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => done.emit(NetEvent::DmRemoved { channel }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn create_group_dm(shared: &Arc<Shared>, users: Vec<Id>, name: String) {
    let body = serde_json::json!({ "user_ids": users, "name": name });
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        "/api/dms/group".to_string(),
        body,
        move |result: Result<DmChannel, String>| match result {
            Ok(dm) => done.emit(NetEvent::Dm(Box::new(dm))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_mutuals(shared: &Arc<Shared>, user: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/users/{user}/mutuals").to_string(),
        move |result: Result<Vec<GuildSummary>, String>| match result {
            Ok(guilds) => done.emit(NetEvent::Mutuals { user, guilds }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn open_dm(shared: &Arc<Shared>, user: Id) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/dms/{user}").to_string(),
        serde_json::json!({}),
        move |result: Result<DmChannel, String>| match result {
            Ok(dm) => done.emit(NetEvent::Dm(Box::new(dm))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn friend_request(shared: &Arc<Shared>, user: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Post,
        format!("/api/friends/{user}/request"),
        serde_json::json!({}),
        move |_| done.emit(NetEvent::RelationshipChanged),
    );
}

pub(crate) fn friend_accept(shared: &Arc<Shared>, user: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Post,
        format!("/api/friends/{user}/accept"),
        serde_json::json!({}),
        move |_| done.emit(NetEvent::RelationshipChanged),
    );
}

pub(crate) fn friend_remove(shared: &Arc<Shared>, user: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Post,
        format!("/api/friends/{user}/remove"),
        serde_json::json!({}),
        move |_| done.emit(NetEvent::RelationshipChanged),
    );
}

pub(crate) fn friend_block(shared: &Arc<Shared>, user: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Post,
        format!("/api/friends/{user}/block"),
        serde_json::json!({}),
        move |_| done.emit(NetEvent::RelationshipChanged),
    );
}
