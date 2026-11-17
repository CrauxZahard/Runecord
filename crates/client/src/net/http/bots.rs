//! HTTP command handlers: bots.

use super::super::*;
use std::sync::Arc;

pub(crate) fn load_bots(shared: &Arc<Shared>) {
    let done = shared.clone();
    get_then(
        shared,
        "/api/bots".to_string(),
        move |result: Result<Vec<User>, String>| match result {
            Ok(bots) => done.emit(NetEvent::Bots(bots)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn create_bot(shared: &Arc<Shared>, name: String, guild: Option<Id>) {
    let body = serde_json::json!({ "name": name, "guild_id": guild });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        "/api/bots".to_string(),
        body,
        move |result| match result {
            Ok(v) => match serde_json::from_value::<User>(v["bot"].clone()) {
                Ok(bot) => {
                    let token = v["token"].as_str().unwrap_or_default().to_string();
                    done.emit(NetEvent::BotCreated { bot, token });
                }
                Err(e) => done.emit(NetEvent::Error(e.to_string())),
            },
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn delete_bot(shared: &Arc<Shared>, id: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Delete,
        format!("/api/bots/{id}"),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => {
                done.emit(NetEvent::Notice("Bot revoked".into()));
                load_bots(&done);
            }
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn load_applications(shared: &Arc<Shared>) {
    let done = shared.clone();
    get_then(
        shared,
        "/api/applications".to_string(),
        move |result: Result<Vec<Application>, String>| match result {
            Ok(apps) => done.emit(NetEvent::Applications(apps)),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn create_application(shared: &Arc<Shared>, name: String, description: String) {
    let body = serde_json::json!({
        "name": name,
        "description": description,
        "color": 0x5865f2u32,
    });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        "/api/applications".to_string(),
        body,
        move |result| match result {
            Ok(v) => match serde_json::from_value::<Application>(v["application"].clone()) {
                Ok(application) => {
                    let token = v["token"].as_str().unwrap_or_default().to_string();
                    done.emit(NetEvent::ApplicationCreated { application, token });
                }
                Err(e) => done.emit(NetEvent::Error(e.to_string())),
            },
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn delete_application(shared: &Arc<Shared>, id: Id) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Delete,
        format!("/api/applications/{id}"),
        serde_json::json!({}),
        move |result| match result {
            Ok(()) => {
                done.emit(NetEvent::Notice("Application deleted".into()));
                load_applications(&done);
            }
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn application_token(shared: &Arc<Shared>, id: Id, name: String) {
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/applications/{id}/token"),
        serde_json::json!({}),
        move |result| match result {
            Ok(v) => {
                let token = v["token"].as_str().unwrap_or_default().to_string();
                done.emit(NetEvent::TokenRevealed {
                    label: format!("{name} token"),
                    token,
                });
            }
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn application_install_link(shared: &Arc<Shared>, id: Id) {
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/applications/{id}/install-link"),
        serde_json::json!({}),
        move |result| match result {
            Ok(v) => {
                let code = v["code"].as_str().unwrap_or_default().to_string();
                done.emit(NetEvent::InstallLink(code));
            }
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn install_application(shared: &Arc<Shared>, code: String, guild: Id) {
    let body = serde_json::json!({
        "guild_id": guild,
        "permissions": 0,
        "scopes": ["bot", "applications.commands"],
    });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        format!("/api/applications/install/{code}"),
        body,
        move |result| match result {
            Ok(v) => match serde_json::from_value::<Application>(v) {
                Ok(application) => {
                    done.emit(NetEvent::ApplicationInstalled(application));
                    done.emit(NetEvent::Notice("Application installed".into()));
                }
                Err(e) => done.emit(NetEvent::Error(e.to_string())),
            },
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}
