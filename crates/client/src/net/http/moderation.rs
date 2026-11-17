//! HTTP command handlers: moderation.

use super::super::*;
use std::sync::Arc;

pub(crate) fn set_channel_notify(
    shared: &Arc<Shared>,
    channel: Id,
    level: NotificationLevel,
    suppress: bool,
    clear: bool,
) {
    let body = serde_json::json!({
        "level": level,
        "suppress_everyone": suppress,
        "clear": clear,
    });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/channels/{channel}/notifications").to_string(),
        body,
        move |result| match result {
            Ok(_) => done.emit(NetEvent::ChannelNotify {
                channel,
                level,
                suppress,
                clear,
            }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn set_guild_notify(
    shared: &Arc<Shared>,
    guild: Id,
    settings: GuildNotificationSettings,
) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Patch,
        format!("/api/guilds/{guild}/notifications").to_string(),
        settings,
        move |result: Result<GuildNotificationSettings, String>| match result {
            Ok(settings) => done.emit(NetEvent::GuildNotifyUpdated { guild, settings }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn kick_member(shared: &Arc<Shared>, guild: Id, user: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/members/{user}/kick").to_string(),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => done.emit(NetEvent::MemberRemove { guild, user }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn ban_member(shared: &Arc<Shared>, guild: Id, user: Id, reason: String) {
    let body = serde_json::json!({ "reason": reason });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/members/{user}/ban").to_string(),
        body,
        move |result| match result {
            Ok(_) => done.emit(NetEvent::MemberRemove { guild, user }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn timeout_member(shared: &Arc<Shared>, guild: Id, user: Id, seconds: u64) {
    let body = serde_json::json!({ "seconds": seconds });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/members/{user}/timeout").to_string(),
        body,
        move |result| match result {
            Ok(_) => done.emit(NetEvent::GuildChanged),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_bans(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/guilds/{guild}/bans").to_string(),
        move |result: Result<Vec<Ban>, String>| match result {
            Ok(bans) => done.emit(NetEvent::Bans(bans)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn unban_member(shared: &Arc<Shared>, guild: Id, user: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Delete,
        format!("/api/guilds/{guild}/bans/{user}"),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => done.emit(NetEvent::Unbanned { guild, user }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn prune_preview(shared: &Arc<Shared>, guild: Id, days: u32, roles: Vec<Id>) {
    let body = serde_json::json!({ "days": days, "role_ids": roles });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/prune/preview"),
        body,
        move |result| match result {
            Ok(v) => {
                let count = v.get("count").and_then(|value| value.as_u64()).unwrap_or(0) as u32;
                done.emit(NetEvent::PrunePreview { days, count });
            }
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_appeals(shared: &Arc<Shared>, guild: Id) {
    let done = shared.clone();
    get_then(
        shared,
        format!("/api/guilds/{guild}/appeals").to_string(),
        move |result: Result<Vec<BanAppeal>, String>| match result {
            Ok(appeals) => done.emit(NetEvent::Appeals(appeals)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn resolve_appeal(shared: &Arc<Shared>, guild: Id, appeal: Id, accept: bool) {
    let action = if accept { "accept" } else { "deny" };
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/appeals/{appeal}/{action}").to_string(),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => done.emit(NetEvent::AppealResolved {
                appeal,
                accepted: accept,
            }),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn submit_appeal(shared: &Arc<Shared>, code: String, message: String) {
    let body = serde_json::json!({ "message": message });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/invites/{code}/appeal").to_string(),
        body,
        move |result| match result {
            Ok(_) => done.emit(NetEvent::AppealSubmitted),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn prune_members(shared: &Arc<Shared>, guild: Id, days: u32, roles: Vec<Id>) {
    let body = serde_json::json!({ "days": days, "role_ids": roles });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/guilds/{guild}/prune"),
        body,
        move |result| match result {
            Ok(v) => {
                let count = v
                    .get("pruned")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(0) as u32;
                done.emit(NetEvent::Pruned { count });
                done.emit(NetEvent::GuildChanged);
            }
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}
