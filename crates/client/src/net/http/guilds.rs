//! HTTP command handlers: guilds.

use super::super::*;
use std::sync::Arc;

pub(crate) fn load_guild_detail(shared: &Arc<Shared>, guild: Id) {
    match shared.get(&format!("/api/guilds/{guild}")) {
        Ok(v) => match serde_json::from_value::<GuildDetail>(v) {
            Ok(detail) => shared.emit(NetEvent::GuildDetail(Box::new(detail))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_members(
    shared: &Arc<Shared>,
    guild: Id,
    channel: Id,
    offset: u32,
    query: String,
    replace: bool,
) {
    let mut path = format!("/api/guilds/{guild}/members?offset={offset}&limit=100");
    if channel != 0 {
        path.push_str(&format!("&channel_id={channel}"));
    }
    if !query.trim().is_empty() {
        path.push_str("&q=");
        path.push_str(&urlencoding::encode(query.trim()));
    }
    match shared.get(&path) {
        Ok(v) => match serde_json::from_value::<MemberPage>(v) {
            Ok(page) => shared.emit(NetEvent::MembersPage {
                guild,
                channel,
                page: Box::new(page),
                replace,
            }),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_users(shared: &Arc<Shared>, users: Vec<Id>) {
    let body = serde_json::json!({ "ids": users });
    match shared.post("/api/users/batch", &body) {
        Ok(v) => match serde_json::from_value::<Vec<User>>(v) {
            Ok(users) => shared.emit(NetEvent::Users(users)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn lookup_user(shared: &Arc<Shared>, query: String) {
    let path = format!("/api/users/lookup?q={}", urlencoding::encode(&query));
    match shared.get(&path) {
        Ok(v) => match serde_json::from_value::<Vec<User>>(v) {
            Ok(users) => shared.emit(NetEvent::Lookup(users)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_guild_events(shared: &Arc<Shared>, guild: Id) {
    match shared.get(&format!("/api/guilds/{guild}/events")) {
        Ok(v) => match serde_json::from_value::<Vec<GuildEvent>>(v) {
            Ok(events) => shared.emit(NetEvent::GuildEvents(events)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn create_guild_event(
    shared: &Arc<Shared>,
    guild: Id,
    name: String,
    description: String,
    start_at: i64,
    channel: Option<Id>,
) {
    let body = serde_json::json!({
        "name": name,
        "description": description,
        "start_at": start_at,
        "channel_id": channel,
    });
    match shared.post(&format!("/api/guilds/{guild}/events"), &body) {
        Ok(v) => match serde_json::from_value::<GuildEvent>(v) {
            Ok(event) => shared.emit(NetEvent::GuildEventCreated(event)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn delete_guild_event(shared: &Arc<Shared>, guild: Id, event: Id) {
    match shared.delete_json(&format!("/api/guilds/{guild}/events/{event}")) {
        Ok(_) => shared.emit(NetEvent::GuildEventDeleted { guild, event }),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_inbox(shared: &Arc<Shared>) {
    match shared.get("/api/inbox") {
        Ok(v) => match serde_json::from_value::<Inbox>(v) {
            Ok(inbox) => shared.emit(NetEvent::Inbox(Box::new(inbox))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn create_guild(shared: &Arc<Shared>, name: String, color: u32) {
    match shared.post(
        "/api/guilds",
        &CreateGuild {
            name,
            icon_color: color,
        },
    ) {
        Ok(v) => {
            if let Ok(g) = serde_json::from_value::<Guild>(v) {
                shared.emit(NetEvent::ChannelCreate(Box::new(
                    g.channels.first().cloned().unwrap_or_default(),
                )));
                shared.emit(NetEvent::GuildChanged);
            }
        }
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn leave_guild(shared: &Arc<Shared>, guild: Id) {
    match shared.post(
        &format!("/api/guilds/{guild}/leave"),
        &serde_json::json!({}),
    ) {
        Ok(_) => shared.emit(NetEvent::GuildLeft(guild)),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_public_guilds(shared: &Arc<Shared>) {
    match shared.get("/api/public-guilds") {
        Ok(v) => match serde_json::from_value::<Vec<PublicGuild>>(v) {
            Ok(guilds) => shared.emit(NetEvent::PublicGuilds(guilds)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn join_public_guild(shared: &Arc<Shared>, guild: Id) {
    match shared.post(
        &format!("/api/public-guilds/{guild}/join"),
        &serde_json::json!({}),
    ) {
        Ok(_) => {
            shared.emit(NetEvent::JoinedPublicGuild(guild));
            shared.emit(NetEvent::GuildChanged);
        }
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn set_guild_discovery(shared: &Arc<Shared>, guild: Id, discoverable: bool) {
    let body = serde_json::json!({ "discoverable": discoverable });
    match shared.post(&format!("/api/guilds/{guild}/discoverable"), &body) {
        Ok(_) => shared.emit(NetEvent::GuildDiscovery {
            guild,
            discoverable,
        }),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn create_channel(
    shared: &Arc<Shared>,
    guild: Id,
    name: String,
    kind: ChannelKind,
    parent: Option<Id>,
    topic: String,
) {
    let body = CreateChannel {
        name,
        kind,
        parent_id: parent,
        topic,
        status: String::new(),
        slowmode: 0,
        nsfw: false,
    };
    match shared.post(&format!("/api/guilds/{guild}/channels"), &body) {
        Ok(_) => shared.emit(NetEvent::GuildChanged),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn update_channel(
    shared: &Arc<Shared>,
    channel: Id,
    name: String,
    topic: String,
    status: String,
    slowmode: u32,
    nsfw: bool,
) {
    let body = serde_json::json!({
        "name": name,
        "kind": "text",
        "parent_id": null,
        "topic": topic,
        "status": status,
        "slowmode": slowmode,
        "nsfw": nsfw,
    });
    match shared.patch(&format!("/api/channels/{channel}"), &body) {
        Ok(v) => match serde_json::from_value::<Channel>(v) {
            Ok(channel) => shared.emit(NetEvent::ChannelUpdated(Box::new(channel))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn delete_channel(shared: &Arc<Shared>, channel: Id) {
    match shared.delete(&format!("/api/channels/{channel}")) {
        Ok(()) => shared.emit(NetEvent::ChannelDelete(channel)),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn update_guild_settings(
    shared: &Arc<Shared>,
    guild: Id,
    name: String,
    description: String,
    color: u32,
    tag: String,
    discoverable: bool,
) {
    let body = serde_json::json!({
        "name": name,
        "description": description,
        "icon_color": color,
        "tag": tag,
        "discoverable": discoverable,
    });
    match shared.patch(&format!("/api/guilds/{guild}"), &body) {
        Ok(v) => match serde_json::from_value::<GuildSummary>(v) {
            Ok(summary) => shared.emit(NetEvent::GuildUpdated(Box::new(summary))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn upload_guild_image(
    shared: &Arc<Shared>,
    guild: Id,
    banner: bool,
    path: std::path::PathBuf,
) {
    let kind = if banner { "banner" } else { "icon" };
    match std::fs::read(&path) {
        Ok(bytes) => {
            let token = shared.token.lock().ok().and_then(|t| t.clone());
            let mut req = shared
                .client
                .post(format!("{}/api/guilds/{guild}/{kind}", shared.base))
                .header("Content-Type", "application/octet-stream")
                .body(bytes);
            if let Some(t) = token {
                req = req.bearer_auth(t);
            }
            match req.send() {
                Ok(res) if res.status().is_success() => match res.json::<GuildSummary>() {
                    Ok(summary) => shared.emit(NetEvent::GuildUpdated(Box::new(summary))),
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

pub(crate) fn remove_guild_image(shared: &Arc<Shared>, guild: Id, banner: bool) {
    let kind = if banner { "banner" } else { "icon" };
    match shared.delete_json(&format!("/api/guilds/{guild}/{kind}")) {
        Ok(v) => match serde_json::from_value::<GuildSummary>(v) {
            Ok(summary) => shared.emit(NetEvent::GuildUpdated(Box::new(summary))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn boost_guild(shared: &Arc<Shared>, guild: Id, on: bool) {
    let path = format!("/api/guilds/{guild}/boost");
    let result = if on {
        shared.post(&path, &serde_json::json!({}))
    } else {
        shared.delete_json(&path)
    };
    match result {
        Ok(v) => match serde_json::from_value::<GuildSummary>(v) {
            Ok(summary) => shared.emit(NetEvent::GuildUpdated(Box::new(summary))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn create_invite(shared: &Arc<Shared>, guild: Id) {
    match shared.post(
        &format!("/api/guilds/{guild}/invites"),
        &CreateInvite { max_uses: None },
    ) {
        Ok(v) => {
            if let Some(code) = v.get("code").and_then(|c| c.as_str()) {
                shared.emit(NetEvent::Error(format!("INVITE:{code}")));
            }
        }
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn join_invite(shared: &Arc<Shared>, code: String) {
    match shared.post(&format!("/api/invites/{code}/join"), &serde_json::json!({})) {
        Ok(v) => {
            if let Ok(guild) = serde_json::from_value::<Guild>(v) {
                if guild.id != 0 {
                    shared.emit(NetEvent::JoinedPublicGuild(guild.id));
                } else {
                    shared.emit(NetEvent::GuildChanged);
                }
            } else {
                shared.emit(NetEvent::GuildChanged);
            }
        }
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_audit(shared: &Arc<Shared>, guild: Id, before: Option<Id>) {
    let mut path = format!("/api/guilds/{guild}/audit-logs?limit=50");
    if let Some(before) = before {
        path.push_str(&format!("&before={before}"));
    }
    match shared.get(&path) {
        Ok(v) => match serde_json::from_value::<Vec<AuditEntry>>(v) {
            Ok(entries) => shared.emit(NetEvent::AuditPage {
                entries,
                older: before.is_some(),
            }),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn set_overwrite(
    shared: &Arc<Shared>,
    channel: Id,
    target: Id,
    kind: String,
    allow: u64,
    deny: u64,
) {
    let body = serde_json::json!({ "kind": kind, "allow": allow, "deny": deny });
    let url = format!(
        "{}/api/channels/{channel}/permissions/{target}",
        shared.base
    );
    let token = shared
        .token
        .lock()
        .ok()
        .and_then(|t| t.clone())
        .unwrap_or_default();
    let _ = shared.client.put(url).bearer_auth(token).json(&body).send();
    shared.emit(NetEvent::GuildChanged);
}

pub(crate) fn clear_overwrite(shared: &Arc<Shared>, channel: Id, target: Id, kind: String) {
    let token = shared
        .token
        .lock()
        .ok()
        .and_then(|t| t.clone())
        .unwrap_or_default();
    let _ = shared
        .client
        .delete(format!(
            "{}/api/channels/{channel}/permissions/{target}",
            shared.base
        ))
        .bearer_auth(token)
        .json(&serde_json::json!({ "kind": kind }))
        .send();
    shared.emit(NetEvent::GuildChanged);
}

pub(crate) fn voice(shared: &Arc<Shared>, guild: Id, channel: Option<Id>, mute: bool, deaf: bool) {
    let body = serde_json::json!({ "guild_id": guild, "channel_id": channel, "self_mute": mute, "self_deaf": deaf });
    match shared.post("/api/voice", &body) {
        Ok(_) => {
            let token = shared.token.lock().ok().and_then(|t| t.clone());
            if let Some(token) = token {
                crate::voice::update(&shared.base, &token, channel, mute, deaf);
            }
        }
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}
