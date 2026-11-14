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
    match shared.post(&format!("/api/channels/{channel}/notifications"), &body) {
        Ok(_) => shared.emit(NetEvent::ChannelNotify {
            channel,
            level,
            suppress,
            clear,
        }),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn set_guild_notify(
    shared: &Arc<Shared>,
    guild: Id,
    settings: GuildNotificationSettings,
) {
    match shared.patch(&format!("/api/guilds/{guild}/notifications"), &settings) {
        Ok(v) => match serde_json::from_value::<GuildNotificationSettings>(v) {
            Ok(settings) => shared.emit(NetEvent::GuildNotifyUpdated { guild, settings }),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn kick_member(shared: &Arc<Shared>, guild: Id, user: Id) {
    match shared.post(
        &format!("/api/guilds/{guild}/members/{user}/kick"),
        &serde_json::json!({}),
    ) {
        Ok(_) => shared.emit(NetEvent::MemberRemove { guild, user }),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn ban_member(shared: &Arc<Shared>, guild: Id, user: Id, reason: String) {
    let body = serde_json::json!({ "reason": reason });
    match shared.post(&format!("/api/guilds/{guild}/members/{user}/ban"), &body) {
        Ok(_) => shared.emit(NetEvent::MemberRemove { guild, user }),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn timeout_member(shared: &Arc<Shared>, guild: Id, user: Id, seconds: u64) {
    let body = serde_json::json!({ "seconds": seconds });
    match shared.post(
        &format!("/api/guilds/{guild}/members/{user}/timeout"),
        &body,
    ) {
        Ok(_) => shared.emit(NetEvent::GuildChanged),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_bans(shared: &Arc<Shared>, guild: Id) {
    match shared.get(&format!("/api/guilds/{guild}/bans")) {
        Ok(v) => match serde_json::from_value::<Vec<Ban>>(v) {
            Ok(bans) => shared.emit(NetEvent::Bans(bans)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn unban_member(shared: &Arc<Shared>, guild: Id, user: Id) {
    match shared.delete(&format!("/api/guilds/{guild}/bans/{user}")) {
        Ok(()) => shared.emit(NetEvent::Unbanned { guild, user }),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn prune_preview(shared: &Arc<Shared>, guild: Id, days: u32, roles: Vec<Id>) {
    let body = serde_json::json!({ "days": days, "role_ids": roles });
    match shared.post(&format!("/api/guilds/{guild}/prune/preview"), &body) {
        Ok(v) => {
            let count = v.get("count").and_then(|value| value.as_u64()).unwrap_or(0) as u32;
            shared.emit(NetEvent::PrunePreview { days, count });
        }
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_appeals(shared: &Arc<Shared>, guild: Id) {
    match shared.get(&format!("/api/guilds/{guild}/appeals")) {
        Ok(v) => match serde_json::from_value::<Vec<BanAppeal>>(v) {
            Ok(appeals) => shared.emit(NetEvent::Appeals(appeals)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn resolve_appeal(shared: &Arc<Shared>, guild: Id, appeal: Id, accept: bool) {
    let action = if accept { "accept" } else { "deny" };
    match shared.post(
        &format!("/api/guilds/{guild}/appeals/{appeal}/{action}"),
        &serde_json::json!({}),
    ) {
        Ok(_) => shared.emit(NetEvent::AppealResolved {
            appeal,
            accepted: accept,
        }),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn submit_appeal(shared: &Arc<Shared>, code: String, message: String) {
    let body = serde_json::json!({ "message": message });
    match shared.post(&format!("/api/invites/{code}/appeal"), &body) {
        Ok(_) => shared.emit(NetEvent::AppealSubmitted),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn prune_members(shared: &Arc<Shared>, guild: Id, days: u32, roles: Vec<Id>) {
    let body = serde_json::json!({ "days": days, "role_ids": roles });
    match shared.post(&format!("/api/guilds/{guild}/prune"), &body) {
        Ok(v) => {
            let count = v
                .get("pruned")
                .and_then(|value| value.as_u64())
                .unwrap_or(0) as u32;
            shared.emit(NetEvent::Pruned { count });
            shared.emit(NetEvent::GuildChanged);
        }
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}
