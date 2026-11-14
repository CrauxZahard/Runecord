//! HTTP command handlers: account.

use super::super::*;
use std::sync::Arc;

pub(crate) fn login(shared: &Arc<Shared>, username: String, password: String) {
    match shared.post("/api/login", &AuthRequest { username, password }) {
        Ok(v) => match serde_json::from_value::<AuthResponse>(v) {
            Ok(auth) => complete_auth(&shared, auth),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn register(
    shared: &Arc<Shared>,
    username: String,
    display_name: String,
    password: String,
) {
    let body = serde_json::json!({ "username": username, "display_name": display_name, "password": password });
    match shared.post("/api/register", &body) {
        Ok(v) => match serde_json::from_value::<AuthResponse>(v) {
            Ok(auth) => complete_auth(&shared, auth),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn logout(shared: &Arc<Shared>) {
    let _ = shared.post("/api/logout", &serde_json::json!({}));
    if let Ok(mut token) = shared.token.lock() {
        *token = None;
    }
    write_token(&shared.profile, None);
    shared.emit(NetEvent::Error(String::new()));
}

pub(crate) fn bootstrap(shared: &Arc<Shared>) {
    fetch_bootstrap(&shared)
}

pub(crate) fn update_profile_extras(
    shared: &Arc<Shared>,
    activity: Option<ProfileActivity>,
    connections: Vec<ProfileConnection>,
) {
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

pub(crate) fn upload_activity_artwork(shared: &Arc<Shared>, path: std::path::PathBuf) {
    match std::fs::read(&path) {
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
    }
}

pub(crate) fn remove_activity_artwork(shared: &Arc<Shared>) {
    match shared.delete_json("/api/me/activity/artwork") {
        Ok(v) => match serde_json::from_value::<User>(v) {
            Ok(user) => shared.emit(NetEvent::UserUpdate(Box::new(user))),
            Err(e) => shared.emit(NetEvent::Error(e.to_string())),
        },
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}

pub(crate) fn check_server(shared: &Arc<Shared>, url: String) {
    let target = format!("{}/api/version", url.trim_end_matches('/'));
    let result = shared
        .client
        .get(target)
        .timeout(Duration::from_secs(6))
        .send();
    match result {
        Ok(response) if response.status().is_success() => {
            let body: serde_json::Value = response.json().unwrap_or(serde_json::Value::Null);
            let ok = body.get("name").and_then(|value| value.as_str()) == Some("runecord");
            shared.emit(NetEvent::ServerChecked {
                url,
                ok,
                error: if ok {
                    None
                } else {
                    Some("That address answered, but it is not a Runecord server.".into())
                },
            });
        }
        Ok(response) => shared.emit(NetEvent::ServerChecked {
            url,
            ok: false,
            error: Some(format!(
                "The server answered with HTTP {}.",
                response.status().as_u16()
            )),
        }),
        Err(error) => shared.emit(NetEvent::ServerChecked {
            url,
            ok: false,
            error: Some(if error.is_timeout() {
                "The server did not answer in time.".into()
            } else if error.is_connect() {
                "The connection was refused.".into()
            } else {
                error.to_string()
            }),
        }),
    }
}

pub(crate) fn update_profile_style(
    shared: &Arc<Shared>,
    effect: String,
    nameplate: String,
    widget: String,
) {
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

pub(crate) fn update_profile(
    shared: &Arc<Shared>,
    display_name: Option<String>,
    pronouns: Option<String>,
    badges: Option<Vec<String>>,
    avatar_decoration: Option<String>,
    custom_status: Option<String>,
    bio: Option<String>,
    color: Option<u32>,
    avatar_border: Option<u32>,
    banner_color: Option<u32>,
) {
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

pub(crate) fn change_password(shared: &Arc<Shared>, current: String, new: String) {
    let body = serde_json::json!({ "current": current, "new": new });
    match shared.post("/api/me/password", &body) {
        Ok(_) => shared.emit(NetEvent::Notice("Password updated".into())),
        Err(e) => shared.emit(NetEvent::Error(e)),
    }
}
