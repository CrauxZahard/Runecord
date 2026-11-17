//! HTTP command handlers: roles.

use super::super::*;
use std::sync::Arc;

#[allow(clippy::too_many_arguments)]
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
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Patch,
        format!("/api/guilds/{guild}/roles/{role}").to_string(),
        body,
        move |result: Result<Role, String>| match result {
            Ok(role) => done.emit(NetEvent::RoleUpdated(role)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
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
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/roles").to_string(),
        body,
        move |result: Result<Role, String>| match result {
            Ok(role) => done.emit(NetEvent::RoleCreated(role)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn delete_role(shared: &Arc<Shared>, guild: Id, role: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Delete,
        format!("/api/guilds/{guild}/roles/{role}"),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => done.emit(NetEvent::RoleDeleted { guild, role }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn upload_role_icon(shared: &Arc<Shared>, guild: Id, role: Id, source: UploadSource) {
    let done = shared.clone();
    upload_then(
        shared,
        format!("/api/guilds/{guild}/roles/{role}/icon"),
        source,
        "application/octet-stream".to_string(),
        move |result: Result<Role, String>| match result {
            Ok(role) => done.emit(NetEvent::RoleUpdated(role)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn remove_role_icon(shared: &Arc<Shared>, guild: Id, role: Id) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Delete,
        format!("/api/guilds/{guild}/roles/{role}/icon").to_string(),
        serde_json::Value::Null,
        move |result: Result<Role, String>| match result {
            Ok(role) => done.emit(NetEvent::RoleUpdated(role)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_role_links(shared: &Arc<Shared>, guild: Id, role: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/guilds/{guild}/roles/{role}/links").to_string(),
        move |result: Result<Vec<RoleLink>, String>| match result {
            Ok(links) => done.emit(NetEvent::RoleLinks { role, links }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn add_role_link(shared: &Arc<Shared>, guild: Id, role: Id, label: String, url: String) {
    let body = serde_json::json!({ "label": label, "url": url });
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/roles/{role}/links").to_string(),
        body,
        move |result: Result<Vec<RoleLink>, String>| match result {
            Ok(links) => done.emit(NetEvent::RoleLinks { role, links }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn delete_role_link(shared: &Arc<Shared>, guild: Id, role: Id, link: Id) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Delete,
        format!("/api/guilds/{guild}/roles/{role}/links/{link}").to_string(),
        serde_json::Value::Null,
        move |result: Result<Vec<RoleLink>, String>| match result {
            Ok(links) => done.emit(NetEvent::RoleLinks { role, links }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn update_member_roles(shared: &Arc<Shared>, guild: Id, user: Id, roles: Vec<Id>) {
    let body = serde_json::json!({ "user_id": user, "role_ids": roles });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Patch,
        format!("/api/guilds/{guild}/members/{user}"),
        body,
        move |result| match result {
            Ok(v) => match serde_json::from_value::<Member>(v) {
                Ok(member) => {
                    done.emit(NetEvent::MemberUpdate(Box::new(member)));
                    done.emit(NetEvent::Notice("Member roles updated".into()));
                }
                Err(e) => done.emit(NetEvent::Error(e.to_string())),
            },
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_emojis(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/guilds/{guild}/emojis").to_string(),
        move |result: Result<Vec<Emoji>, String>| match result {
            Ok(emojis) => done.emit(NetEvent::Emojis(emojis)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_guild_commands(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/guilds/{guild}/commands").to_string(),
        move |result: Result<Vec<ApplicationCommand>, String>| match result {
            Ok(commands) => done.emit(NetEvent::GuildCommands(commands)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
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
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        "/api/interactions".to_string(),
        body,
        move |result| match result {
            Ok(v) => {
                let id = v["id"].as_str().unwrap_or_default().to_string();
                let token = v["token"].as_str().unwrap_or_default().to_string();
                let callback = serde_json::json!({
                    "type": 4,
                    "data": { "content": response },
                });
                fire_then(
                    &done,
                    HttpMethod::Post,
                    format!("/api/interactions/{id}/{token}/callback"),
                    callback,
                    |_| {},
                );
            }
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn create_emoji(
    shared: &Arc<Shared>,
    guild: Id,
    name: String,
    sticker: bool,
    source: UploadSource,
) {
    let done = shared.clone();
    let path = format!(
        "/api/guilds/{guild}/emojis?name={}&sticker={sticker}",
        crate::net::encode_component(&name)
    );
    upload_then(
        shared,
        path,
        source,
        "image/png".to_string(),
        move |result: Result<Emoji, String>| match result {
            Ok(emoji) => done.emit(NetEvent::EmojiCreated(emoji)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn delete_emoji(shared: &Arc<Shared>, guild: Id, id: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Delete,
        format!("/api/guilds/{guild}/emojis/{id}"),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => done.emit(NetEvent::EmojiDeleted { guild, id }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}
