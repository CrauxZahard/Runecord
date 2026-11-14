//! HTTP command handlers: bots.

use super::super::*;
use std::sync::Arc;

pub(crate) fn load_bots(shared: &Arc<Shared>) {
    match shared.get("/api/bots") {
        Ok(v) => match serde_json::from_value::<Vec<User>>(v) {
            Ok(bots) => shared.emit(NetEvent::Bots(bots)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn create_bot(shared: &Arc<Shared>, name: String, guild: Option<Id>) {
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

pub(crate) fn delete_bot(shared: &Arc<Shared>, id: Id) {
    match shared.delete(&format!("/api/bots/{id}")) {
        Ok(()) => {
            shared.emit(NetEvent::Notice("Bot revoked".into()));
            if let Ok(v) = shared.get("/api/bots") {
                if let Ok(bots) = serde_json::from_value::<Vec<User>>(v) {
                    shared.emit(NetEvent::Bots(bots));
                }
            }
        }
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn load_applications(shared: &Arc<Shared>) {
    match shared.get("/api/applications") {
        Ok(v) => match serde_json::from_value::<Vec<Application>>(v) {
            Ok(apps) => shared.emit(NetEvent::Applications(apps)),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn create_application(shared: &Arc<Shared>, name: String, description: String) {
    let body = serde_json::json!({
        "name": name,
        "description": description,
        "color": 0x5865f2u32,
    });
    match shared.post("/api/applications", &body) {
        Ok(v) => match serde_json::from_value::<Application>(v["application"].clone()) {
            Ok(application) => {
                let token = v["token"].as_str().unwrap_or_default().to_string();
                shared.emit(NetEvent::ApplicationCreated { application, token });
            }
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn delete_application(shared: &Arc<Shared>, id: Id) {
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

pub(crate) fn application_token(shared: &Arc<Shared>, id: Id, name: String) {
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

pub(crate) fn application_install_link(shared: &Arc<Shared>, id: Id) {
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

pub(crate) fn install_application(shared: &Arc<Shared>, code: String, guild: Id) {
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
