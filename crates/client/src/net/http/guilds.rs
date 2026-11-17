//! HTTP command handlers: guilds.

use super::super::*;
use std::sync::Arc;

pub(crate) fn load_guild_detail(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/guilds/{guild}").to_string(),
        move |result: Result<GuildDetail, String>| match result {
            Ok(detail) => done.emit(NetEvent::GuildDetail(Box::new(detail))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
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
        path.push_str(&crate::net::encode_component(query.trim()));
    }
    let done = shared.clone();
    get_then(
        shared,
        path.to_string(),
        move |result: Result<MemberPage, String>| match result {
            Ok(page) => done.emit(NetEvent::MembersPage {
                guild,
                channel,
                page: Box::new(page),
                replace,
            }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_users(shared: &Arc<Shared>, users: Vec<Id>) {
    let body = serde_json::json!({ "ids": users });
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        "/api/users/batch".to_string(),
        body,
        move |result: Result<Vec<User>, String>| match result {
            Ok(users) => done.emit(NetEvent::Users(users)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn lookup_user(shared: &Arc<Shared>, query: String) {
    let path = format!(
        "/api/users/lookup?q={}",
        crate::net::encode_component(&query)
    );
    let done = shared.clone();
    get_then(
        shared,
        path.to_string(),
        move |result: Result<Vec<User>, String>| match result {
            Ok(users) => done.emit(NetEvent::Lookup(users)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_guild_events(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/guilds/{guild}/events").to_string(),
        move |result: Result<Vec<GuildEvent>, String>| match result {
            Ok(events) => done.emit(NetEvent::GuildEvents(events)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn create_guild_event(shared: &Arc<Shared>, guild: Id, event: CreateGuildEvent) {
    let body = serde_json::json!({
        "name": event.name,
        "description": event.description,
        "start_at": event.start_at,
        "channel_id": event.channel_id,
        "recurrence": event.recurrence,
        "duration_minutes": event.duration_minutes,
        "reminder_minutes": event.reminder_minutes,
    });
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/events").to_string(),
        body,
        move |result: Result<GuildEvent, String>| match result {
            Ok(event) => done.emit(NetEvent::GuildEventCreated(event)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn rsvp_event(shared: &Arc<Shared>, guild: Id, event: Id, status: String) {
    let body = serde_json::json!({ "status": status });
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Put,
        format!("/api/guilds/{guild}/events/{event}/rsvp").to_string(),
        body,
        move |result: Result<GuildEvent, String>| match result {
            Ok(event) => done.emit(NetEvent::GuildEventUpdated(event)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn delete_guild_event(shared: &Arc<Shared>, guild: Id, event: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Delete,
        format!("/api/guilds/{guild}/events/{event}").to_string(),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => done.emit(NetEvent::GuildEventDeleted { guild, event }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_inbox(shared: &Arc<Shared>) {
    let done = shared.clone();
    get_then(
        shared,
        "/api/inbox".to_string(),
        move |result: Result<Inbox, String>| match result {
            Ok(inbox) => done.emit(NetEvent::Inbox(Box::new(inbox))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn create_guild(shared: &Arc<Shared>, name: String, color: u32) {
    let done = shared.clone();
    let body = CreateGuild {
        name,
        icon_color: color,
    };
    send_then_json(
        shared,
        HttpMethod::Post,
        "/api/guilds".to_string(),
        body,
        move |result: Result<Guild, String>| match result {
            Ok(guild) => {
                let channel = guild.channels.first().cloned().unwrap_or_default();
                done.emit(NetEvent::ChannelCreate(Box::new(channel)));
                done.emit(NetEvent::GuildChanged);
            }
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn leave_guild(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/leave").to_string(),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => done.emit(NetEvent::GuildLeft(guild)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_public_guilds(shared: &Arc<Shared>) {
    let done = shared.clone();
    get_then(
        shared,
        "/api/public-guilds".to_string(),
        move |result: Result<Vec<PublicGuild>, String>| match result {
            Ok(guilds) => done.emit(NetEvent::PublicGuilds(guilds)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn join_public_guild(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Post,
        format!("/api/public-guilds/{guild}/join"),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => {
                done.emit(NetEvent::JoinedPublicGuild(guild));
                done.emit(NetEvent::GuildChanged);
            }
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn set_guild_discovery(shared: &Arc<Shared>, guild: Id, discoverable: bool) {
    let body = serde_json::json!({ "discoverable": discoverable });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/discoverable").to_string(),
        body,
        move |result| match result {
            Ok(_) => done.emit(NetEvent::GuildDiscovery {
                guild,
                discoverable,
            }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
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
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/channels").to_string(),
        body,
        move |result| match result {
            Ok(_) => done.emit(NetEvent::GuildChanged),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
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
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Patch,
        format!("/api/channels/{channel}").to_string(),
        body,
        move |result: Result<Channel, String>| match result {
            Ok(channel) => done.emit(NetEvent::ChannelUpdated(Box::new(channel))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn delete_channel(shared: &Arc<Shared>, channel: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Delete,
        format!("/api/channels/{channel}"),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => done.emit(NetEvent::ChannelDelete(channel)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
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
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Patch,
        format!("/api/guilds/{guild}").to_string(),
        body,
        move |result: Result<GuildSummary, String>| match result {
            Ok(summary) => done.emit(NetEvent::GuildUpdated(Box::new(summary))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn upload_guild_image(
    shared: &Arc<Shared>,
    guild: Id,
    banner: bool,
    source: UploadSource,
) {
    let kind = if banner { "banner" } else { "icon" };
    let done = shared.clone();
    upload_then(
        shared,
        format!("/api/guilds/{guild}/{kind}"),
        source,
        "application/octet-stream".to_string(),
        move |result: Result<GuildSummary, String>| match result {
            Ok(summary) => done.emit(NetEvent::GuildUpdated(Box::new(summary))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn remove_guild_image(shared: &Arc<Shared>, guild: Id, banner: bool) {
    let kind = if banner { "banner" } else { "icon" };
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Delete,
        format!("/api/guilds/{guild}/{kind}").to_string(),
        serde_json::Value::Null,
        move |result: Result<GuildSummary, String>| match result {
            Ok(summary) => done.emit(NetEvent::GuildUpdated(Box::new(summary))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn boost_guild(shared: &Arc<Shared>, guild: Id, on: bool) {
    let path = format!("/api/guilds/{guild}/reputation");
    let method = if on {
        HttpMethod::Post
    } else {
        HttpMethod::Delete
    };
    let done = shared.clone();
    send_then_json(
        shared,
        method,
        path,
        serde_json::json!({}),
        move |result: Result<GuildSummary, String>| match result {
            Ok(summary) => done.emit(NetEvent::GuildUpdated(Box::new(summary))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_automod(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/guilds/{guild}/automod").to_string(),
        move |result: Result<AutoModConfig, String>| match result {
            Ok(config) => done.emit(NetEvent::AutoModConfig(Box::new(config))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn save_automod(shared: &Arc<Shared>, guild: Id, rules: Vec<AutoModRule>) {
    let body = AutoModConfig { rules };
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Put,
        format!("/api/guilds/{guild}/automod").to_string(),
        serde_json::to_value(&body).unwrap_or_default(),
        move |result: Result<AutoModConfig, String>| match result {
            Ok(config) => done.emit(NetEvent::AutoModConfig(Box::new(config))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_templates(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/guilds/{guild}/templates").to_string(),
        move |result: Result<Vec<GuildTemplate>, String>| match result {
            Ok(list) => done.emit(NetEvent::Templates(list)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn create_template(shared: &Arc<Shared>, guild: Id, name: String, description: String) {
    let body = serde_json::json!({ "name": name, "description": description });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/templates").to_string(),
        body,
        move |result| match result {
            Ok(_) => load_templates(&done, guild),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn delete_template(shared: &Arc<Shared>, guild: Id, template: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Delete,
        format!("/api/guilds/{guild}/templates/{template}").to_string(),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => load_templates(&done, guild),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn apply_template(shared: &Arc<Shared>, code: String, name: String) {
    let body = serde_json::json!({ "name": name });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/templates/{code}").to_string(),
        body,
        move |result| match result {
            Ok(_) => fetch_bootstrap(&done),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_webhooks(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/guilds/{guild}/webhooks").to_string(),
        move |result: Result<Vec<WebhookInfo>, String>| match result {
            Ok(list) => done.emit(NetEvent::Webhooks(list)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn create_webhook(shared: &Arc<Shared>, channel: Id, name: String) {
    let body = serde_json::json!({ "name": name, "color": 0x5865f2 });
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/channels/{channel}/webhooks").to_string(),
        body,
        move |result: Result<WebhookInfo, String>| match result {
            Ok(hook) => done.emit(NetEvent::WebhookCreated(Box::new(hook))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn delete_webhook(shared: &Arc<Shared>, guild: Id, id: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Delete,
        format!("/api/webhooks/{id}").to_string(),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => load_webhooks(&done, guild),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn update_onboarding(shared: &Arc<Shared>, guild: Id, update: UpdateOnboarding) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Patch,
        format!("/api/guilds/{guild}/onboarding").to_string(),
        serde_json::to_value(&update).unwrap_or_default(),
        move |result: Result<GuildOnboarding, String>| match result {
            Ok(onboarding) => done.emit(NetEvent::OnboardingUpdated(Box::new(onboarding))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn accept_onboarding(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/onboarding/accept").to_string(),
        serde_json::json!({}),
        move |result: Result<GuildOnboarding, String>| match result {
            Ok(onboarding) => done.emit(NetEvent::OnboardingUpdated(Box::new(onboarding))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn create_invite(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/invites"),
        CreateInvite { max_uses: None },
        move |result| match result {
            Ok(v) => {
                if let Some(code) = v.get("code").and_then(|c| c.as_str()) {
                    done.emit(NetEvent::Error(format!("INVITE:{code}")));
                }
            }
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn join_invite(shared: &Arc<Shared>, code: String) {
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/invites/{code}/join"),
        serde_json::json!({}),
        move |result| match result {
            Ok(v) => {
                if let Ok(guild) = serde_json::from_value::<Guild>(v) {
                    if guild.id != 0 {
                        done.emit(NetEvent::JoinedPublicGuild(guild.id));
                    } else {
                        done.emit(NetEvent::GuildChanged);
                    }
                } else {
                    done.emit(NetEvent::GuildChanged);
                }
            }
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_audit(
    shared: &Arc<Shared>,
    guild: Id,
    before: Option<Id>,
    actor: Option<Id>,
    action: Option<i32>,
) {
    let mut path = format!("/api/guilds/{guild}/audit-logs?limit=50");
    if let Some(before) = before {
        path.push_str(&format!("&before={before}"));
    }
    if let Some(actor) = actor {
        path.push_str(&format!("&actor={actor}"));
    }
    if let Some(action) = action {
        path.push_str(&format!("&action={action}"));
    }
    let done = shared.clone();
    get_then(
        shared,
        path.to_string(),
        move |result: Result<Vec<AuditEntry>, String>| match result {
            Ok(entries) => done.emit(NetEvent::AuditPage {
                entries,
                older: before.is_some(),
            }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
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

pub(crate) fn load_gifs(shared: &Arc<Shared>, query: String) {
    let path = if query.trim().is_empty() {
        "/api/gifs".to_string()
    } else {
        format!("/api/gifs?q={}", urlencode(query.trim()))
    };
    let done = shared.clone();
    get_then(
        shared,
        path.to_string(),
        move |result: Result<Vec<GifItem>, String>| match result {
            Ok(list) => done.emit(NetEvent::Gifs(list)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn send_gif(shared: &Arc<Shared>, channel: Id, slug: String) {
    let body = serde_json::json!({ "slug": slug });
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/channels/{channel}/gifs").to_string(),
        body,
        move |result: Result<Message, String>| match result {
            Ok(message) => done.emit(NetEvent::MessageCreate(Box::new(message))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

fn urlencode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

pub(crate) fn load_activities(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/guilds/{guild}/activities").to_string(),
        move |result: Result<Vec<Activity>, String>| match result {
            Ok(list) => done.emit(NetEvent::Activities(list)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn start_activity(shared: &Arc<Shared>, channel: Id, activity: Id) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        format!("/api/channels/{channel}/activities/{activity}").to_string(),
        serde_json::json!({}),
        move |result: Result<ActivitySession, String>| match result {
            Ok(session) => done.emit(NetEvent::ActivityUpdated(Box::new(ActivityUpdate {
                channel_id: channel,
                session: Some(session),
            }))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn stop_activity(shared: &Arc<Shared>, channel: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Delete,
        format!("/api/channels/{channel}/activities").to_string(),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => done.emit(NetEvent::ActivityUpdated(Box::new(ActivityUpdate {
                channel_id: channel,
                session: None,
            }))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_stage(shared: &Arc<Shared>, channel: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/channels/{channel}/stage").to_string(),
        move |result: Result<StageState, String>| match result {
            Ok(stage) => done.emit(NetEvent::StageUpdated(Box::new(stage))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn stage_action(shared: &Arc<Shared>, channel: Id, action: String, user: Option<Id>) {
    let body = serde_json::json!({ "action": action, "user_id": user });
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Put,
        format!("/api/channels/{channel}/stage").to_string(),
        body,
        move |result: Result<StageState, String>| match result {
            Ok(stage) => done.emit(NetEvent::StageUpdated(Box::new(stage))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn voice(shared: &Arc<Shared>, guild: Id, channel: Option<Id>, mute: bool, deaf: bool) {
    let body = serde_json::json!({ "guild_id": guild, "channel_id": channel, "self_mute": mute, "self_deaf": deaf });
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Post,
        "/api/voice".to_string(),
        body,
        move |result| match result {
            Ok(()) => {
                let token = done.token.lock().ok().and_then(|t| t.clone());
                if let Some(token) = token {
                    crate::voice::update(&done.base, &token, channel, mute, deaf);
                }
            }
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

// Browser stub: this path reads or writes local files through the native client.
#[cfg(target_arch = "wasm32")]
pub(crate) fn check_server(shared: &Arc<Shared>, _url: String) {
    let _ = (_url);
    shared.emit(NetEvent::Error(
        "That action is not available in the web client yet.".to_string(),
    ));
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn set_overwrite(
    shared: &Arc<Shared>,
    _channel: Id,
    _target: Id,
    _kind: String,
    _allow: u64,
    _deny: u64,
) {
    let _ = (_channel, _target, _kind, _allow, _deny);
    shared.emit(NetEvent::Error(
        "That action is not available in the web client yet.".to_string(),
    ));
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn clear_overwrite(shared: &Arc<Shared>, _channel: Id, _target: Id, _kind: String) {
    let _ = (_channel, _target, _kind);
    shared.emit(NetEvent::Error(
        "That action is not available in the web client yet.".to_string(),
    ));
}
