//! Gateway socket worker.

use super::*;

/// Maps one gateway event onto the client's `NetEvent` boundary. Shared by the
/// desktop socket and the browser's WebSocket so both behave identically.
pub(crate) fn dispatch_event(shared: &Arc<Shared>, ev: GatewayEvent) {
    match ev {
        GatewayEvent::Ready(b) => {
            shared.bootstrap_seen.store(true, Ordering::SeqCst);
            shared.emit(NetEvent::Bootstrap(b));
        }
        GatewayEvent::DmCreate(dm) => shared.emit(NetEvent::Dm(dm)),
        GatewayEvent::DmUpdate(dm) => shared.emit(NetEvent::DmUpdated(dm)),
        GatewayEvent::DmRemove { channel_id, .. } => shared.emit(NetEvent::DmRemoved {
            channel: channel_id,
        }),
        GatewayEvent::MessageCreate(m) => shared.emit(NetEvent::MessageCreate(m)),
        GatewayEvent::MessageUpdate(m) => shared.emit(NetEvent::MessageUpdate(m)),
        GatewayEvent::ReactionUpdate(m) => shared.emit(NetEvent::MessageUpdate(m)),
        GatewayEvent::MessageDelete {
            channel_id,
            message_id,
        } => shared.emit(NetEvent::MessageDelete {
            channel: channel_id,
            message: message_id,
        }),
        GatewayEvent::TypingStart {
            channel_id,
            user_id,
            until_ms,
        } => shared.emit(NetEvent::Typing {
            channel: channel_id,
            user: user_id,
            until: until_ms,
        }),
        GatewayEvent::PresenceUpdate { user_id, status } => shared.emit(NetEvent::Presence {
            user: user_id,
            status,
        }),
        GatewayEvent::ChannelCreate(c) => shared.emit(NetEvent::ChannelCreate(c)),
        GatewayEvent::ChannelUpdate(channel) => shared.emit(NetEvent::ChannelUpdated(channel)),
        GatewayEvent::ChannelDelete { channel_id, .. } => {
            shared.emit(NetEvent::ChannelDelete(channel_id))
        }
        GatewayEvent::RoleCreate(role) => shared.emit(NetEvent::RoleCreated(*role)),
        GatewayEvent::RoleUpdate(role) => shared.emit(NetEvent::RoleUpdated(*role)),
        GatewayEvent::RoleDelete { guild_id, role_id } => shared.emit(NetEvent::RoleDeleted {
            guild: guild_id,
            role: role_id,
        }),
        GatewayEvent::GuildCreate(_)
        | GatewayEvent::GuildUpdate(_)
        | GatewayEvent::GuildDelete { .. } => shared.emit(NetEvent::GuildChanged),
        GatewayEvent::RelationshipUpdate(_) | GatewayEvent::RelationshipRemove { .. } => {
            shared.emit(NetEvent::RelationshipChanged)
        }
        GatewayEvent::MemberUpdate(m) => shared.emit(NetEvent::MemberUpdate(m)),
        GatewayEvent::UserUpdate(user) => shared.emit(NetEvent::UserUpdate(user)),
        GatewayEvent::GuildMembers { .. } => shared.emit(NetEvent::RelationshipChanged),
        GatewayEvent::MemberRemove { guild_id, user_id } => shared.emit(NetEvent::MemberRemove {
            guild: guild_id,
            user: user_id,
        }),
        GatewayEvent::MemberCountUpdate { guild_id, count } => {
            shared.emit(NetEvent::MemberCountUpdate {
                guild: guild_id,
                count,
            })
        }
        GatewayEvent::VoiceStateUpdate(v) => shared.emit(NetEvent::VoiceState(v)),
        GatewayEvent::StageUpdate(stage) => shared.emit(NetEvent::StageUpdated(stage)),
        GatewayEvent::ActivityUpdate(update) => shared.emit(NetEvent::ActivityUpdated(update)),
        GatewayEvent::InteractionCreate { .. } => {}
        GatewayEvent::ReadUpdate { .. } | GatewayEvent::ReadyCheck => {}
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn ws_worker(shared: Arc<Shared>) {
    let mut connected = false;
    let mut failed = false;
    loop {
        let token = shared.token.lock().ok().and_then(|t| t.clone());
        let Some(token) = token else {
            if connected {
                shared.emit(NetEvent::Connected(false));
                connected = false;
            }
            std::thread::sleep(Duration::from_millis(250));
            continue;
        };
        let url = shared
            .base
            .replacen("http://", "ws://", 1)
            .replacen("https://", "wss://", 1);
        let request = match url.parse::<tungstenite::http::Uri>() {
            Ok(_uri) => {
                let mut req = tungstenite::client::IntoClientRequest::into_client_request(format!(
                    "{url}/gateway"
                ))
                .ok();
                if let Some(r) = req.as_mut()
                    && let Ok(value) =
                        tungstenite::http::HeaderValue::from_str(&format!("Bearer {token}"))
                {
                    r.headers_mut().insert("Authorization", value);
                }
                req
            }
            Err(_) => None,
        };
        let Some(request) = request else {
            std::thread::sleep(Duration::from_secs(1));
            continue;
        };
        match tungstenite::connect(request) {
            Ok((mut socket, _)) => {
                failed = false;
                if !connected {
                    shared.emit(NetEvent::Connected(true));
                    connected = true;
                }
                let _ = socket.send(tungstenite::Message::Text(
                    "{\"op\":\"heartbeat\",\"d\":null}".into(),
                ));
                loop {
                    match socket.read() {
                        Ok(tungstenite::Message::Text(text)) => {
                            if let Ok(ev) = serde_json::from_str::<GatewayEvent>(&text) {
                                dispatch_event(&shared, ev);
                            }
                        }
                        Ok(tungstenite::Message::Ping(bytes)) => {
                            let _ = socket.send(tungstenite::Message::Pong(bytes));
                        }
                        Ok(tungstenite::Message::Close(_)) | Err(_) => break,
                        Ok(_) => {}
                    }
                    let now_token = shared.token.lock().ok().and_then(|t| t.clone());
                    if now_token.is_none() {
                        break;
                    }
                }
                if connected {
                    shared.emit(NetEvent::Connected(false));
                    connected = false;
                }
            }
            Err(_) => {
                if connected {
                    shared.emit(NetEvent::Connected(false));
                    connected = false;
                } else if !failed {
                    shared.emit(NetEvent::ConnectionFailed);
                    failed = true;
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn ws_worker(shared: Arc<Shared>) {
    crate::net::web::gateway_worker(shared);
}
