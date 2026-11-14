//! HTTP command handlers: social.

use super::super::*;
use std::sync::Arc;

pub(crate) fn set_privacy(shared: &Arc<Shared>, dm: PrivacyLevel, friend: PrivacyLevel) {
    let body = UpdatePrivacy {
        dm_privacy: dm,
        friend_privacy: friend,
    };
    match shared.post("/api/me/privacy", &body) {
        Ok(v) => match serde_json::from_value::<User>(v) {
            Ok(user) => shared.emit(NetEvent::UserUpdate(Box::new(user))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn rename_dm(shared: &Arc<Shared>, channel: Id, name: String) {
    let body = serde_json::json!({ "name": name });
    match shared.patch(&format!("/api/dms/{channel}"), &body) {
        Ok(v) => match serde_json::from_value::<DmChannel>(v) {
            Ok(dm) => shared.emit(NetEvent::DmUpdated(Box::new(dm))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn upload_dm_icon(shared: &Arc<Shared>, channel: Id, path: std::path::PathBuf) {
    match std::fs::read(&path) {
        Ok(bytes) => {
            let token = shared.token.lock().ok().and_then(|t| t.clone());
            let mut req = shared
                .client
                .post(format!("{}/api/dms/{channel}/icon", shared.base))
                .header("Content-Type", "application/octet-stream")
                .body(bytes);
            if let Some(t) = token {
                req = req.bearer_auth(t);
            }
            match req.send() {
                Ok(res) if res.status().is_success() => match res.json::<DmChannel>() {
                    Ok(dm) => shared.emit(NetEvent::DmUpdated(Box::new(dm))),
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

pub(crate) fn remove_dm_icon(shared: &Arc<Shared>, channel: Id) {
    match shared.delete_json(&format!("/api/dms/{channel}/icon")) {
        Ok(v) => match serde_json::from_value::<DmChannel>(v) {
            Ok(dm) => shared.emit(NetEvent::DmUpdated(Box::new(dm))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn add_dm_member(shared: &Arc<Shared>, channel: Id, user: Id) {
    let body = serde_json::json!({ "user_id": user });
    match shared.post(&format!("/api/dms/{channel}/members"), &body) {
        Ok(v) => match serde_json::from_value::<DmChannel>(v) {
            Ok(dm) => shared.emit(NetEvent::DmUpdated(Box::new(dm))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn remove_dm_member(shared: &Arc<Shared>, channel: Id, user: Id) {
    match shared.delete_json(&format!("/api/dms/{channel}/members/{user}")) {
        Ok(_) => {}
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn leave_dm(shared: &Arc<Shared>, channel: Id) {
    match shared.post(&format!("/api/dms/{channel}/leave"), &serde_json::json!({})) {
        Ok(_) => shared.emit(NetEvent::DmRemoved { channel }),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn create_group_dm(shared: &Arc<Shared>, users: Vec<Id>, name: String) {
    let body = serde_json::json!({ "user_ids": users, "name": name });
    match shared.post("/api/dms/group", &body) {
        Ok(v) => match serde_json::from_value::<DmChannel>(v) {
            Ok(dm) => shared.emit(NetEvent::Dm(Box::new(dm))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_mutuals(shared: &Arc<Shared>, user: Id) {
    match shared.get(&format!("/api/users/{user}/mutuals")) {
        Ok(v) => match serde_json::from_value::<Vec<GuildSummary>>(v) {
            Ok(guilds) => shared.emit(NetEvent::Mutuals { user, guilds }),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn open_dm(shared: &Arc<Shared>, user: Id) {
    match shared.post(&format!("/api/dms/{user}"), &serde_json::json!({})) {
        Ok(v) => match serde_json::from_value::<DmChannel>(v) {
            Ok(dm) => shared.emit(NetEvent::Dm(Box::new(dm))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn friend_request(shared: &Arc<Shared>, user: Id) {
    let _ = shared.post(
        &format!("/api/friends/{user}/request"),
        &serde_json::json!({}),
    );
    shared.emit(NetEvent::RelationshipChanged);
}

pub(crate) fn friend_accept(shared: &Arc<Shared>, user: Id) {
    let _ = shared.post(
        &format!("/api/friends/{user}/accept"),
        &serde_json::json!({}),
    );
    shared.emit(NetEvent::RelationshipChanged);
}

pub(crate) fn friend_remove(shared: &Arc<Shared>, user: Id) {
    let _ = shared.post(
        &format!("/api/friends/{user}/remove"),
        &serde_json::json!({}),
    );
    shared.emit(NetEvent::RelationshipChanged);
}

pub(crate) fn friend_block(shared: &Arc<Shared>, user: Id) {
    let _ = shared.post(
        &format!("/api/friends/{user}/block"),
        &serde_json::json!({}),
    );
    shared.emit(NetEvent::RelationshipChanged);
}
