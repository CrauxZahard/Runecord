//! HTTP command handlers: account.

use super::super::*;
use std::sync::Arc;

pub(crate) fn login(shared: &Arc<Shared>, username: String, password: String) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        "/api/login".to_string(),
        AuthRequest {
            username,
            password,
            signup_code: None,
        },
        move |result: Result<AuthResponse, String>| match result {
            Ok(auth) => complete_auth(&done, auth),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn register(
    shared: &Arc<Shared>,
    username: String,
    display_name: String,
    password: String,
    signup_code: String,
) {
    let body = serde_json::json!({
        "username": username,
        "display_name": display_name,
        "password": password,
        "signup_code": signup_code,
    });
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        "/api/register".to_string(),
        body,
        move |result: Result<AuthResponse, String>| match result {
            Ok(auth) => complete_auth(&done, auth),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn logout(shared: &Arc<Shared>) {
    let done = shared.clone();
    fire_then(
        shared,
        HttpMethod::Post,
        "/api/logout".to_string(),
        serde_json::json!({}),
        move |_| {
            if let Ok(mut token) = done.token.lock() {
                *token = None;
            }
            write_token(&done.profile, None);
            done.emit(NetEvent::Error(String::new()));
        },
    );
}

pub(crate) fn bootstrap(shared: &Arc<Shared>) {
    fetch_bootstrap(shared)
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
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Patch,
        "/api/me".to_string(),
        body,
        move |result: Result<User, String>| match result {
            Ok(user) => done.emit(NetEvent::UserUpdate(Box::new(user))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn connection_challenge(shared: &Arc<Shared>) {
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        "/api/connections/challenge".to_string(),
        serde_json::json!({}),
        move |result| match result {
            Ok(v) => {
                let code = v
                    .get("code")
                    .and_then(|code| code.as_str())
                    .unwrap_or_default()
                    .to_string();
                done.emit(NetEvent::ConnectionCode(code));
            }
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn verify_connection(shared: &Arc<Shared>, kind: String, handle: String, url: String) {
    let body = serde_json::json!({ "kind": kind, "handle": handle, "url": url });
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Post,
        "/api/connections/verify".to_string(),
        body,
        move |result: Result<User, String>| match result {
            Ok(user) => done.emit(NetEvent::UserUpdate(Box::new(user))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn upload_activity_artwork(shared: &Arc<Shared>, source: UploadSource) {
    let done = shared.clone();
    upload_then(
        shared,
        "/api/me/activity/artwork".to_string(),
        source,
        "application/octet-stream".to_string(),
        move |result: Result<User, String>| match result {
            Ok(user) => done.emit(NetEvent::UserUpdate(Box::new(user))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn remove_activity_artwork(shared: &Arc<Shared>) {
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Delete,
        "/api/me/activity/artwork".to_string(),
        serde_json::Value::Null,
        move |result: Result<User, String>| match result {
            Ok(user) => done.emit(NetEvent::UserUpdate(Box::new(user))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
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
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Patch,
        "/api/me".to_string(),
        body,
        move |result: Result<User, String>| match result {
            Ok(user) => done.emit(NetEvent::UserUpdate(Box::new(user))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

#[allow(clippy::too_many_arguments)]
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
    let done = shared.clone();
    send_then_json(
        shared,
        HttpMethod::Patch,
        "/api/me".to_string(),
        body,
        move |result: Result<User, String>| match result {
            Ok(user) => done.emit(NetEvent::UserUpdate(Box::new(user))),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}

pub(crate) fn change_password(shared: &Arc<Shared>, current: String, new: String) {
    let body = serde_json::json!({ "current": current, "new": new });
    let done = shared.clone();
    send_then(
        shared,
        HttpMethod::Post,
        "/api/me/password".to_string(),
        body,
        move |result| match result {
            Ok(_) => done.emit(NetEvent::Notice("Password updated".into())),
            Err(e) => done.emit(NetEvent::Error(e)),
        },
    );
}
