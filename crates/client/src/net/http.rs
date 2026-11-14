//! Blocking HTTP worker.

use super::*;
pub(crate) fn http_worker(shared: Arc<Shared>, rx: mpsc::Receiver<Command>) {
    while let Ok(cmd) = rx.recv() {
        match cmd {
            Command::Ping => {}
            Command::Login { username, password } => {
                match shared.post("/api/login", &AuthRequest { username, password }) {
                    Ok(v) => match serde_json::from_value::<AuthResponse>(v) {
                        Ok(auth) => complete_auth(&shared, auth),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::Register {
                username,
                display_name,
                password,
            } => {
                let body = serde_json::json!({ "username": username, "display_name": display_name, "password": password });
                match shared.post("/api/register", &body) {
                    Ok(v) => match serde_json::from_value::<AuthResponse>(v) {
                        Ok(auth) => complete_auth(&shared, auth),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::Logout => {
                let _ = shared.post("/api/logout", &serde_json::json!({}));
                if let Ok(mut token) = shared.token.lock() {
                    *token = None;
                }
                write_token(&shared.profile, None);
                shared.emit(NetEvent::Error(String::new()));
            }
            Command::Bootstrap => fetch_bootstrap(&shared),
            Command::LoadGuildDetail { guild } => match shared.get(&format!("/api/guilds/{guild}"))
            {
                Ok(v) => match serde_json::from_value::<GuildDetail>(v) {
                    Ok(detail) => shared.emit(NetEvent::GuildDetail(Box::new(detail))),
                    Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                },
                Err(e) => shared.emit(NetEvent::Error(e)),
            },
            Command::LoadMembers {
                guild,
                channel,
                offset,
                query,
                replace,
            } => {
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
            Command::LoadUsers { users } => {
                let body = serde_json::json!({ "ids": users });
                match shared.post("/api/users/batch", &body) {
                    Ok(v) => match serde_json::from_value::<Vec<User>>(v) {
                        Ok(users) => shared.emit(NetEvent::Users(users)),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::LookupUser { query } => {
                let path = format!("/api/users/lookup?q={}", urlencoding::encode(&query));
                match shared.get(&path) {
                    Ok(v) => match serde_json::from_value::<Vec<User>>(v) {
                        Ok(users) => shared.emit(NetEvent::Lookup(users)),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::LoadMessages { channel, before } => {
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
            Command::Send {
                channel,
                content,
                reply_to,
                attachments,
                nonce: _,
            } => {
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
            Command::Edit { message, content } => {
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
            Command::Delete { message } => {
                if let Err(e) = shared.delete(&format!("/api/messages/{message}")) {
                    shared.emit(NetEvent::Error(e));
                }
            }
            Command::React { message, emoji } => {
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
            Command::Pin { message } => {
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
            Command::Typing { channel } => {
                let _ = shared.post(
                    &format!("/api/channels/{channel}/typing"),
                    &serde_json::json!({}),
                );
            }
            Command::MarkRead { channel, message } => {
                let _ = shared.post(
                    "/api/read",
                    &ReadUpdate {
                        channel_id: channel,
                        last_read_id: message,
                    },
                );
            }
            Command::UpdateProfileExtras {
                activity,
                connections,
            } => {
                let body = serde_json::json!({
                    "activity": activity,
                    "clear_activity": activity.is_none(),
                    "connections": connections,
                });
                match shared.patch("/api/me", &body) {
                    Ok(v) => match serde_json::from_value::<User>(v) {
                        Ok(user) => shared.emit(NetEvent::UserUpdate(Box::new(user))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::UploadActivityArtwork { path } => match std::fs::read(&path) {
                Ok(bytes) => {
                    let token = shared.token.lock().ok().and_then(|t| t.clone());
                    let mut req = shared
                        .client
                        .post(format!("{}/api/me/activity/artwork", shared.base))
                        .header("Content-Type", "application/octet-stream")
                        .body(bytes);
                    if let Some(t) = token {
                        req = req.bearer_auth(t);
                    }
                    match req.send() {
                        Ok(res) if res.status().is_success() => match res.json::<User>() {
                            Ok(user) => shared.emit(NetEvent::UserUpdate(Box::new(user))),
                            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                        },
                        Ok(res) => shared.emit(NetEvent::Error(error_text(res))),
                        Err(e) => shared.emit(NetEvent::Error(format!(
                            "Can't reach the local server: {e}"
                        ))),
                    }
                }
                Err(e) => shared.emit(NetEvent::Error(format!("Could not read image: {e}"))),
            },
            Command::RemoveActivityArtwork => {
                match shared.delete_json("/api/me/activity/artwork") {
                    Ok(v) => match serde_json::from_value::<User>(v) {
                        Ok(user) => shared.emit(NetEvent::UserUpdate(Box::new(user))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::LoadGuildEvents { guild } => {
                match shared.get(&format!("/api/guilds/{guild}/events")) {
                    Ok(v) => match serde_json::from_value::<Vec<GuildEvent>>(v) {
                        Ok(events) => shared.emit(NetEvent::GuildEvents(events)),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::CreateGuildEvent {
                guild,
                name,
                description,
                start_at,
                channel,
            } => {
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
            Command::DeleteGuildEvent { guild, event } => {
                match shared.delete_json(&format!("/api/guilds/{guild}/events/{event}")) {
                    Ok(_) => shared.emit(NetEvent::GuildEventDeleted { guild, event }),
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::UpdateProfileStyle {
                effect,
                nameplate,
                widget,
            } => {
                let body = serde_json::json!({
                    "profile_effect": effect,
                    "nameplate": nameplate,
                    "profile_widget": widget,
                });
                match shared.patch("/api/me", &body) {
                    Ok(v) => match serde_json::from_value::<User>(v) {
                        Ok(user) => shared.emit(NetEvent::UserUpdate(Box::new(user))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::UpdateProfileStyle {
                effect,
                nameplate,
                widget,
            } => {
                let body = serde_json::json!({
                    "profile_effect": effect,
                    "nameplate": nameplate,
                    "profile_widget": widget,
                });
                match shared.patch("/api/me", &body) {
                    Ok(v) => match serde_json::from_value::<User>(v) {
                        Ok(user) => shared.emit(NetEvent::UserUpdate(Box::new(user))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::LoadInbox => match shared.get("/api/inbox") {
                Ok(v) => match serde_json::from_value::<Inbox>(v) {
                    Ok(inbox) => shared.emit(NetEvent::Inbox(Box::new(inbox))),
                    Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                },
                Err(e) => shared.emit(NetEvent::Error(e)),
            },
            Command::Search { q } => {
                let path = format!("/api/search?q={}", urlencoding::encode(&q));
                match shared.get(&path) {
                    Ok(v) => match serde_json::from_value::<SearchResults>(v) {
                        Ok(r) => shared.emit(NetEvent::Search(Box::new(r))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::CreateGuild { name, color } => {
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
            Command::LeaveGuild { guild } => {
                match shared.post(
                    &format!("/api/guilds/{guild}/leave"),
                    &serde_json::json!({}),
                ) {
                    Ok(_) => shared.emit(NetEvent::GuildLeft(guild)),
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::SetChannelNotify {
                channel,
                level,
                suppress,
                clear,
            } => {
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
            Command::SetGuildNotify { guild, settings } => {
                match shared.patch(&format!("/api/guilds/{guild}/notifications"), &settings) {
                    Ok(v) => match serde_json::from_value::<GuildNotificationSettings>(v) {
                        Ok(settings) => {
                            shared.emit(NetEvent::GuildNotifyUpdated { guild, settings })
                        }
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::LoadPublicGuilds => match shared.get("/api/public-guilds") {
                Ok(v) => match serde_json::from_value::<Vec<PublicGuild>>(v) {
                    Ok(guilds) => shared.emit(NetEvent::PublicGuilds(guilds)),
                    Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                },
                Err(e) => shared.emit(NetEvent::Error(e)),
            },
            Command::JoinPublicGuild { guild } => {
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
            Command::LoadPins { channel } => {
                match shared.get(&format!("/api/channels/{channel}/pins")) {
                    Ok(v) => match serde_json::from_value::<Vec<Message>>(v) {
                        Ok(messages) => shared.emit(NetEvent::Pins(messages)),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::SetPrivacy { dm, friend } => {
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
            Command::SetGuildDiscovery {
                guild,
                discoverable,
            } => {
                let body = serde_json::json!({ "discoverable": discoverable });
                match shared.post(&format!("/api/guilds/{guild}/discoverable"), &body) {
                    Ok(_) => shared.emit(NetEvent::GuildDiscovery {
                        guild,
                        discoverable,
                    }),
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::Video { channel, kind } => {
                let token = shared.token.lock().ok().and_then(|t| t.clone());
                if let Some(token) = token {
                    crate::voice::set_video(&shared.base, &token, channel, &kind);
                }
            }
            Command::CreateChannel {
                guild,
                name,
                kind,
                parent,
                topic,
            } => {
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
            Command::UpdateChannel {
                channel,
                name,
                topic,
                status,
                slowmode,
                nsfw,
            } => {
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
            Command::DeleteChannel { channel } => {
                match shared.delete(&format!("/api/channels/{channel}")) {
                    Ok(()) => shared.emit(NetEvent::ChannelDelete(channel)),
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::KickMember { guild, user } => {
                match shared.post(
                    &format!("/api/guilds/{guild}/members/{user}/kick"),
                    &serde_json::json!({}),
                ) {
                    Ok(_) => shared.emit(NetEvent::MemberRemove { guild, user }),
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::BanMember {
                guild,
                user,
                reason,
            } => {
                let body = serde_json::json!({ "reason": reason });
                match shared.post(&format!("/api/guilds/{guild}/members/{user}/ban"), &body) {
                    Ok(_) => shared.emit(NetEvent::MemberRemove { guild, user }),
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::TimeoutMember {
                guild,
                user,
                seconds,
            } => {
                let body = serde_json::json!({ "seconds": seconds });
                match shared.post(
                    &format!("/api/guilds/{guild}/members/{user}/timeout"),
                    &body,
                ) {
                    Ok(_) => shared.emit(NetEvent::GuildChanged),
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::LoadBans { guild } => match shared.get(&format!("/api/guilds/{guild}/bans")) {
                Ok(v) => match serde_json::from_value::<Vec<Ban>>(v) {
                    Ok(bans) => shared.emit(NetEvent::Bans(bans)),
                    Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                },
                Err(e) => shared.emit(NetEvent::Error(e)),
            },
            Command::UnbanMember { guild, user } => {
                match shared.delete(&format!("/api/guilds/{guild}/bans/{user}")) {
                    Ok(()) => shared.emit(NetEvent::Unbanned { guild, user }),
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::UpdateGuildSettings {
                guild,
                name,
                description,
                color,
                tag,
                discoverable,
            } => {
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
            Command::PrunePreview { guild, days, roles } => {
                let body = serde_json::json!({ "days": days, "role_ids": roles });
                match shared.post(&format!("/api/guilds/{guild}/prune/preview"), &body) {
                    Ok(v) => {
                        let count =
                            v.get("count").and_then(|value| value.as_u64()).unwrap_or(0) as u32;
                        shared.emit(NetEvent::PrunePreview { days, count });
                    }
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::UploadGuildImage {
                guild,
                banner,
                path,
            } => {
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
                            Ok(res) if res.status().is_success() => {
                                match res.json::<GuildSummary>() {
                                    Ok(summary) => {
                                        shared.emit(NetEvent::GuildUpdated(Box::new(summary)))
                                    }
                                    Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                                }
                            }
                            Ok(res) => shared.emit(NetEvent::Error(error_text(res))),
                            Err(e) => shared.emit(NetEvent::Error(format!(
                                "Can't reach the local server: {e}"
                            ))),
                        }
                    }
                    Err(e) => shared.emit(NetEvent::Error(format!("Could not read image: {e}"))),
                }
            }
            Command::RemoveGuildImage { guild, banner } => {
                let kind = if banner { "banner" } else { "icon" };
                match shared.delete_json(&format!("/api/guilds/{guild}/{kind}")) {
                    Ok(v) => match serde_json::from_value::<GuildSummary>(v) {
                        Ok(summary) => shared.emit(NetEvent::GuildUpdated(Box::new(summary))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::UploadUserImage { banner, path } => {
                let kind = if banner { "banner" } else { "avatar" };
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        let token = shared.token.lock().ok().and_then(|t| t.clone());
                        let mut req = shared
                            .client
                            .post(format!("{}/api/me/{kind}", shared.base))
                            .header("Content-Type", "application/octet-stream")
                            .body(bytes);
                        if let Some(t) = token {
                            req = req.bearer_auth(t);
                        }
                        match req.send() {
                            Ok(res) if res.status().is_success() => match res.json::<User>() {
                                Ok(user) => shared.emit(NetEvent::UserUpdate(Box::new(user))),
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
            Command::RemoveUserImage { banner } => {
                let kind = if banner { "banner" } else { "avatar" };
                match shared.delete_json(&format!("/api/me/{kind}")) {
                    Ok(v) => match serde_json::from_value::<User>(v) {
                        Ok(user) => shared.emit(NetEvent::UserUpdate(Box::new(user))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::RenameDm { channel, name } => {
                let body = serde_json::json!({ "name": name });
                match shared.patch(&format!("/api/dms/{channel}"), &body) {
                    Ok(v) => match serde_json::from_value::<DmChannel>(v) {
                        Ok(dm) => shared.emit(NetEvent::DmUpdated(Box::new(dm))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::UploadDmIcon { channel, path } => match std::fs::read(&path) {
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
            },
            Command::RemoveDmIcon { channel } => {
                match shared.delete_json(&format!("/api/dms/{channel}/icon")) {
                    Ok(v) => match serde_json::from_value::<DmChannel>(v) {
                        Ok(dm) => shared.emit(NetEvent::DmUpdated(Box::new(dm))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::AddDmMember { channel, user } => {
                let body = serde_json::json!({ "user_id": user });
                match shared.post(&format!("/api/dms/{channel}/members"), &body) {
                    Ok(v) => match serde_json::from_value::<DmChannel>(v) {
                        Ok(dm) => shared.emit(NetEvent::DmUpdated(Box::new(dm))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::RemoveDmMember { channel, user } => {
                match shared.delete_json(&format!("/api/dms/{channel}/members/{user}")) {
                    Ok(_) => {}
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::LeaveDm { channel } => {
                match shared.post(&format!("/api/dms/{channel}/leave"), &serde_json::json!({})) {
                    Ok(_) => shared.emit(NetEvent::DmRemoved { channel }),
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::CreateGroupDm { users, name } => {
                let body = serde_json::json!({ "user_ids": users, "name": name });
                match shared.post("/api/dms/group", &body) {
                    Ok(v) => match serde_json::from_value::<DmChannel>(v) {
                        Ok(dm) => shared.emit(NetEvent::Dm(Box::new(dm))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::LoadMutuals { user } => {
                match shared.get(&format!("/api/users/{user}/mutuals")) {
                    Ok(v) => match serde_json::from_value::<Vec<GuildSummary>>(v) {
                        Ok(guilds) => shared.emit(NetEvent::Mutuals { user, guilds }),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::BoostGuild { guild, on } => {
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
            Command::LoadAppeals { guild } => {
                match shared.get(&format!("/api/guilds/{guild}/appeals")) {
                    Ok(v) => match serde_json::from_value::<Vec<BanAppeal>>(v) {
                        Ok(appeals) => shared.emit(NetEvent::Appeals(appeals)),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::ResolveAppeal {
                guild,
                appeal,
                accept,
            } => {
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
            Command::SubmitAppeal { code, message } => {
                let body = serde_json::json!({ "message": message });
                match shared.post(&format!("/api/invites/{code}/appeal"), &body) {
                    Ok(_) => shared.emit(NetEvent::AppealSubmitted),
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::PruneMembers { guild, days, roles } => {
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
            Command::CreateInvite { guild } => {
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
            Command::JoinInvite { code } => {
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
            Command::OpenDm { user } => {
                match shared.post(&format!("/api/dms/{user}"), &serde_json::json!({})) {
                    Ok(v) => match serde_json::from_value::<DmChannel>(v) {
                        Ok(dm) => shared.emit(NetEvent::Dm(Box::new(dm))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::FriendRequest { user } => {
                let _ = shared.post(
                    &format!("/api/friends/{user}/request"),
                    &serde_json::json!({}),
                );
                shared.emit(NetEvent::RelationshipChanged);
            }
            Command::FriendAccept { user } => {
                let _ = shared.post(
                    &format!("/api/friends/{user}/accept"),
                    &serde_json::json!({}),
                );
                shared.emit(NetEvent::RelationshipChanged);
            }
            Command::FriendRemove { user } => {
                let _ = shared.post(
                    &format!("/api/friends/{user}/remove"),
                    &serde_json::json!({}),
                );
                shared.emit(NetEvent::RelationshipChanged);
            }
            Command::FriendBlock { user } => {
                let _ = shared.post(
                    &format!("/api/friends/{user}/block"),
                    &serde_json::json!({}),
                );
                shared.emit(NetEvent::RelationshipChanged);
            }
            Command::CreateThread {
                channel,
                message,
                name,
            } => {
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
            Command::ArchiveThread { channel, archived } => {
                let body = serde_json::json!({ "archived": archived });
                match shared.patch(&format!("/api/channels/{channel}/thread"), &body) {
                    Ok(v) => match serde_json::from_value::<Channel>(v) {
                        Ok(channel) => shared.emit(NetEvent::ChannelUpdated(Box::new(channel))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::LoadBots => match shared.get("/api/bots") {
                Ok(v) => match serde_json::from_value::<Vec<User>>(v) {
                    Ok(bots) => shared.emit(NetEvent::Bots(bots)),
                    Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                },
                Err(e) => shared.emit(NetEvent::Error(e)),
            },
            Command::CreateBot { name, guild } => {
                let body = serde_json::json!({ "name": name, "guild_id": guild });
                match shared.post("/api/bots", &body) {
                    Ok(v) => match serde_json::from_value::<User>(v["bot"].clone()) {
                        Ok(bot) => {
                            let token = v["token"].as_str().unwrap_or_default().to_string();
                            shared.emit(NetEvent::BotCreated { bot, token });
                        }
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::DeleteBot { id } => match shared.delete(&format!("/api/bots/{id}")) {
                Ok(()) => {
                    shared.emit(NetEvent::Notice("Bot revoked".into()));
                    if let Ok(v) = shared.get("/api/bots") {
                        if let Ok(bots) = serde_json::from_value::<Vec<User>>(v) {
                            shared.emit(NetEvent::Bots(bots));
                        }
                    }
                }
                Err(e) => shared.emit(NetEvent::Error(e)),
            },
            Command::LoadApplications => match shared.get("/api/applications") {
                Ok(v) => match serde_json::from_value::<Vec<Application>>(v) {
                    Ok(apps) => shared.emit(NetEvent::Applications(apps)),
                    Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                },
                Err(e) => shared.emit(NetEvent::Error(e)),
            },
            Command::CreateApplication { name, description } => {
                let body = serde_json::json!({
                    "name": name,
                    "description": description,
                    "color": 0x5865f2u32,
                });
                match shared.post("/api/applications", &body) {
                    Ok(v) => {
                        match serde_json::from_value::<Application>(v["application"].clone()) {
                            Ok(application) => {
                                let token = v["token"].as_str().unwrap_or_default().to_string();
                                shared.emit(NetEvent::ApplicationCreated { application, token });
                            }
                            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                        }
                    }
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::DeleteApplication { id } => {
                match shared.delete(&format!("/api/applications/{id}")) {
                    Ok(()) => {
                        shared.emit(NetEvent::Notice("Application deleted".into()));
                        if let Ok(v) = shared.get("/api/applications") {
                            if let Ok(apps) = serde_json::from_value::<Vec<Application>>(v) {
                                shared.emit(NetEvent::Applications(apps));
                            }
                        }
                    }
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::ApplicationToken { id, name } => {
                match shared.post(
                    &format!("/api/applications/{id}/token"),
                    &serde_json::json!({}),
                ) {
                    Ok(v) => {
                        let token = v["token"].as_str().unwrap_or_default().to_string();
                        shared.emit(NetEvent::TokenRevealed {
                            label: format!("{name} token"),
                            token,
                        });
                    }
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::ApplicationInstallLink { id } => {
                match shared.post(
                    &format!("/api/applications/{id}/install-link"),
                    &serde_json::json!({}),
                ) {
                    Ok(v) => {
                        let code = v["code"].as_str().unwrap_or_default().to_string();
                        shared.emit(NetEvent::InstallLink(code));
                    }
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::InstallApplication { code, guild } => {
                let body = serde_json::json!({
                    "guild_id": guild,
                    "permissions": 0,
                    "scopes": ["bot", "applications.commands"],
                });
                match shared.post(&format!("/api/applications/install/{code}"), &body) {
                    Ok(v) => match serde_json::from_value::<Application>(v) {
                        Ok(application) => {
                            shared.emit(NetEvent::ApplicationInstalled(application));
                            shared.emit(NetEvent::Notice("Application installed".into()));
                        }
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::UpdateProfile {
                display_name,
                pronouns,
                badges,
                avatar_decoration,
                custom_status,
                bio,
                color,
                avatar_border,
                banner_color,
            } => {
                let body = UpdateProfile {
                    display_name,
                    pronouns,
                    badges,
                    avatar_decoration,
                    avatar_color: color,
                    avatar_border,
                    banner_color,
                    bio,
                    custom_status,
                };
                match shared.patch("/api/me", &body) {
                    Ok(v) => match serde_json::from_value::<User>(v) {
                        Ok(user) => shared.emit(NetEvent::UserUpdate(Box::new(user))),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::ChangePassword { current, new } => {
                let body = serde_json::json!({ "current": current, "new": new });
                match shared.post("/api/me/password", &body) {
                    Ok(_) => shared.emit(NetEvent::Notice("Password updated".into())),
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::FetchImage { url } => {
                if let Ok(res) = shared.client.get(format!("{}{}", shared.base, url)).send() {
                    if let Ok(bytes) = res.bytes() {
                        if let Some(animation) = crate::image_anim::decode(&bytes) {
                            let first = animation.frames[0].rgba.clone();
                            shared.emit(NetEvent::Image {
                                url,
                                width: animation.width,
                                height: animation.height,
                                rgba: first,
                                frames: animation.frames,
                            });
                        } else {
                            match image::load_from_memory(&bytes) {
                                Ok(img) => {
                                    let rgba = img.to_rgba8();
                                    let (w, h) = rgba.dimensions();
                                    shared.emit(NetEvent::Image {
                                        url,
                                        width: w,
                                        height: h,
                                        rgba: rgba.into_raw(),
                                        frames: Vec::new(),
                                    });
                                }
                                Err(e) => shared.emit(NetEvent::Error(format!("Image: {e}"))),
                            }
                        }
                    }
                }
            }
            Command::UpdateRole {
                guild,
                role,
                name,
                color,
                permissions,
                hoist,
                mentionable,
                position,
            } => {
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
            Command::CreateRole {
                guild,
                name,
                color,
                permissions,
            } => {
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
            Command::DeleteRole { guild, role } => {
                match shared.delete(&format!("/api/guilds/{guild}/roles/{role}")) {
                    Ok(()) => shared.emit(NetEvent::RoleDeleted { guild, role }),
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::UploadRoleIcon { guild, role, path } => match std::fs::read(&path) {
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
            },
            Command::RemoveRoleIcon { guild, role } => {
                match shared.delete_json(&format!("/api/guilds/{guild}/roles/{role}/icon")) {
                    Ok(v) => match serde_json::from_value::<Role>(v) {
                        Ok(role) => shared.emit(NetEvent::RoleUpdated(role)),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::LoadRoleLinks { guild, role } => {
                match shared.get(&format!("/api/guilds/{guild}/roles/{role}/links")) {
                    Ok(v) => match serde_json::from_value::<Vec<RoleLink>>(v) {
                        Ok(links) => shared.emit(NetEvent::RoleLinks { role, links }),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::AddRoleLink {
                guild,
                role,
                label,
                url,
            } => {
                let body = serde_json::json!({ "label": label, "url": url });
                match shared.post(&format!("/api/guilds/{guild}/roles/{role}/links"), &body) {
                    Ok(v) => match serde_json::from_value::<Vec<RoleLink>>(v) {
                        Ok(links) => shared.emit(NetEvent::RoleLinks { role, links }),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::DeleteRoleLink { guild, role, link } => {
                match shared.delete_json(&format!("/api/guilds/{guild}/roles/{role}/links/{link}"))
                {
                    Ok(v) => match serde_json::from_value::<Vec<RoleLink>>(v) {
                        Ok(links) => shared.emit(NetEvent::RoleLinks { role, links }),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::UpdateMemberRoles { guild, user, roles } => {
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
            Command::LoadEmojis { guild } => {
                match shared.get(&format!("/api/guilds/{guild}/emojis")) {
                    Ok(v) => match serde_json::from_value::<Vec<Emoji>>(v) {
                        Ok(emojis) => shared.emit(NetEvent::Emojis(emojis)),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::LoadGuildCommands { guild } => {
                match shared.get(&format!("/api/guilds/{guild}/commands")) {
                    Ok(v) => match serde_json::from_value::<Vec<ApplicationCommand>>(v) {
                        Ok(commands) => shared.emit(NetEvent::GuildCommands(commands)),
                        Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                    },
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::InvokeCommand {
                channel,
                command,
                options,
                response,
            } => {
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
            Command::LoadAudit { guild, before } => {
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
            Command::CreateEmoji { guild, name, path } => match std::fs::read(&path) {
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
            },
            Command::DeleteEmoji { guild, id } => {
                match shared.delete(&format!("/api/guilds/{guild}/emojis/{id}")) {
                    Ok(()) => shared.emit(NetEvent::EmojiDeleted { guild, id }),
                    Err(e) => shared.emit(NetEvent::Error(e)),
                }
            }
            Command::SetOverwrite {
                channel,
                target,
                kind,
                allow,
                deny,
            } => {
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
            Command::ClearOverwrite {
                channel,
                target,
                kind,
            } => {
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
            Command::Voice {
                guild,
                channel,
                mute,
                deaf,
            } => {
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
            Command::Upload { channel, path } => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "file".into());
                let (w, h) = image::image_dimensions(&path).unwrap_or((0, 0));
                let content_type = match path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase())
                    .as_deref()
                {
                    Some("png") => "image/png",
                    Some("jpg") | Some("jpeg") => "image/jpeg",
                    Some("gif") => "image/gif",
                    Some("webp") => "image/webp",
                    _ => "application/octet-stream",
                };
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        let token = shared.token.lock().ok().and_then(|t| t.clone());
                        let res = shared
                            .client
                            .post(format!(
                                "{}{}",
                                shared.base,
                                format!("/api/channels/{channel}/upload")
                            ))
                            .query(&[
                                ("name", name),
                                ("content_type", content_type.to_string()),
                                ("width", w.to_string()),
                                ("height", h.to_string()),
                            ])
                            .bearer_auth(token.unwrap_or_default())
                            .body(bytes)
                            .send();
                        match res {
                            Ok(r) if r.status().is_success() => match r.json::<UploadResponse>() {
                                Ok(u) => shared.emit(NetEvent::Uploaded(Box::new(u.attachment))),
                                Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                            },
                            Ok(r) => shared.emit(NetEvent::Error(error_text(r))),
                            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
                        }
                    }
                    Err(e) => shared.emit(NetEvent::Error(format!("Could not read file: {e}"))),
                }
            }
        }
    }
}

pub(crate) fn fetch_bootstrap(shared: &Arc<Shared>) {
    if let Ok(v) = shared.get("/api/bootstrap") {
        if let Ok(b) = serde_json::from_value::<Bootstrap>(v) {
            shared.bootstrap_seen.store(true, Ordering::SeqCst);
            shared.emit(NetEvent::Bootstrap(Box::new(b)));
        }
    }
}

pub(crate) fn complete_auth(shared: &Arc<Shared>, auth: AuthResponse) {
    if let Ok(mut token) = shared.token.lock() {
        *token = Some(auth.token.clone());
    }
    write_token(&shared.profile, Some(&auth.token));
    shared.emit(NetEvent::Session(Box::new(auth)));
    fetch_bootstrap(shared);
}
