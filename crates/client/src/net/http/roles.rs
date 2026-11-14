//! HTTP command handlers: roles.

use super::super::*;
use std::sync::Arc;

pub(crate) fn update_role(
    shared: &Arc<Shared>,
    guild: Id,
    role: Id,
    name: Option<String>,
    color: Option<u32>,
    permissions: Option<u64>,
    hoist: Option<bool>,
    mentionable: Option<bool>,
    position: Option<i32>,
) {
    let body = serde_json::json!({
        "name": name,
        "color": color,
        "permissions": permissions,
        "hoist": hoist,
        "mentionable": mentionable,
        "position": position,
    });
    match shared.patch(&format!("/api/guilds/{guild}/roles/{role}"), &body) {
        Ok(v) => match serde_json::from_value::<Role>(v) {
            Ok(role) => shared.emit(NetEvent::RoleUpdated(role)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn create_role(
    shared: &Arc<Shared>,
    guild: Id,
    name: String,
    color: u32,
    permissions: u64,
) {
    let body = serde_json::json!({
        "name": name,
        "color": color,
        "permissions": permissions,
    });
    match shared.post(&format!("/api/guilds/{guild}/roles"), &body) {
        Ok(v) => match serde_json::from_value::<Role>(v) {
            Ok(role) => shared.emit(NetEvent::RoleCreated(role)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn delete_role(shared: &Arc<Shared>, guild: Id, role: Id) {
    match shared.delete(&format!("/api/guilds/{guild}/roles/{role}")) {
        Ok(()) => shared.emit(NetEvent::RoleDeleted { guild, role }),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn upload_role_icon(
    shared: &Arc<Shared>,
    guild: Id,
    role: Id,
    path: std::path::PathBuf,
) {
    match std::fs::read(&path) {
        Ok(bytes) => {
            let token = shared.token.lock().ok().and_then(|t| t.clone());
            let mut req = shared
                .client
                .post(format!(
                    "{}/api/guilds/{guild}/roles/{role}/icon",
                    shared.base
                ))
                .header("Content-Type", "application/octet-stream")
                .body(bytes);
            if let Some(t) = token {
                req = req.bearer_auth(t);
            }
            match req.send() {
                Ok(res) if res.status().is_success() => match res.json::<Role>() {
                    Ok(role) => shared.emit(NetEvent::RoleUpdated(role)),
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

pub(crate) fn remove_role_icon(shared: &Arc<Shared>, guild: Id, role: Id) {
    match shared.delete_json(&format!("/api/guilds/{guild}/roles/{role}/icon")) {
        Ok(v) => match serde_json::from_value::<Role>(v) {
            Ok(role) => shared.emit(NetEvent::RoleUpdated(role)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_role_links(shared: &Arc<Shared>, guild: Id, role: Id) {
    match shared.get(&format!("/api/guilds/{guild}/roles/{role}/links")) {
        Ok(v) => match serde_json::from_value::<Vec<RoleLink>>(v) {
            Ok(links) => shared.emit(NetEvent::RoleLinks { role, links }),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn add_role_link(shared: &Arc<Shared>, guild: Id, role: Id, label: String, url: String) {
    let body = serde_json::json!({ "label": label, "url": url });
    match shared.post(&format!("/api/guilds/{guild}/roles/{role}/links"), &body) {
        Ok(v) => match serde_json::from_value::<Vec<RoleLink>>(v) {
            Ok(links) => shared.emit(NetEvent::RoleLinks { role, links }),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn delete_role_link(shared: &Arc<Shared>, guild: Id, role: Id, link: Id) {
    match shared.delete_json(&format!("/api/guilds/{guild}/roles/{role}/links/{link}")) {
        Ok(v) => match serde_json::from_value::<Vec<RoleLink>>(v) {
            Ok(links) => shared.emit(NetEvent::RoleLinks { role, links }),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn update_member_roles(shared: &Arc<Shared>, guild: Id, user: Id, roles: Vec<Id>) {
    let body = serde_json::json!({ "user_id": user, "role_ids": roles });
    match shared.patch(&format!("/api/guilds/{guild}/members/{user}"), &body) {
        Ok(v) => match serde_json::from_value::<Member>(v) {
            Ok(member) => {
                shared.emit(NetEvent::MemberUpdate(Box::new(member)));
                shared.emit(NetEvent::Notice("Member roles updated".into()));
            }
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_emojis(shared: &Arc<Shared>, guild: Id) {
    match shared.get(&format!("/api/guilds/{guild}/emojis")) {
        Ok(v) => match serde_json::from_value::<Vec<Emoji>>(v) {
            Ok(emojis) => shared.emit(NetEvent::Emojis(emojis)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_guild_commands(shared: &Arc<Shared>, guild: Id) {
    match shared.get(&format!("/api/guilds/{guild}/commands")) {
        Ok(v) => match serde_json::from_value::<Vec<ApplicationCommand>>(v) {
            Ok(commands) => shared.emit(NetEvent::GuildCommands(commands)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn invoke_command(
    shared: &Arc<Shared>,
    channel: Id,
    command: String,
    options: serde_json::Value,
    response: String,
) {
    let body = serde_json::json!({
        "command": command,
        "channel_id": channel,
        "options": options,
    });
    match shared.post("/api/interactions", &body) {
        Ok(v) => {
            let id = v["id"].as_str().unwrap_or_default().to_string();
            let token = v["token"].as_str().unwrap_or_default().to_string();
            let callback = serde_json::json!({
                "type": 4,
                "data": { "content": response },
            });
            match shared.post(
                &format!("/api/interactions/{id}/{token}/callback"),
                &callback,
            ) {
                Ok(_) => {}
                Err(e) => shared.emit(NetEvent::Error(e)),
            }
        }
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn create_emoji(
    shared: &Arc<Shared>,
    guild: Id,
    name: String,
    path: std::path::PathBuf,
) {
    match std::fs::read(&path) {
        Ok(bytes) => {
            let token = shared.token.lock().ok().and_then(|t| t.clone());
            let res = shared
                .client
                .post(format!("{}/api/guilds/{guild}/emojis", shared.base))
                .query(&[("name", name), ("sticker", "false".to_string())])
                .bearer_auth(token.unwrap_or_default())
                .header("content-type", "image/png")
                .body(bytes)
                .send();
            match res {
                Ok(r) if r.status().is_success() => match r.json::<Emoji>() {
                    Ok(emoji) => shared.emit(NetEvent::EmojiCreated(emoji)),
                    Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                },
                Ok(r) => shared.emit(NetEvent::Error(error_text(r))),
                Err(e) => shared.emit(NetEvent::Error(e.to_string())),
            }
        }
        Err(e) => shared.emit(NetEvent::Error(format!("Can't read image: {e}"))),
    }
}

pub(crate) fn delete_emoji(shared: &Arc<Shared>, guild: Id, id: Id) {
    match shared.delete(&format!("/api/guilds/{guild}/emojis/{id}")) {
        Ok(()) => shared.emit(NetEvent::EmojiDeleted { guild, id }),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}
