//! Runecord: a local chat app built with Rune.

use crate::app::{App, Modal, Screen, ServerTab, SettingsTab, TimeoutUnit};
use crate::voice::ViewQuality;
use crate::{app, ui};

enum ResetRequest {
    None,
    User(String),
    MissingUser,
}

fn reset_request() -> ResetRequest {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--reset-password" {
            return match args.next() {
                Some(user) if !user.starts_with('-') => ResetRequest::User(user),
                _ => ResetRequest::MissingUser,
            };
        }
    }
    ResetRequest::None
}

#[cfg(feature = "embedded-server")]
fn reset_local_password(user: &str) {
    let data = crate::paths::data_dir()
        .map(|dir| dir.join("server"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    match runecord_server::reset_password(&data, user) {
        Ok(password) => {
            // The message box is the interactive path; the file makes the same
            // result readable from a script and survives an accidental close.
            let file = data.join("reset-password.txt");
            let _ = std::fs::write(
                &file,
                format!(
                    "username: {user}\npassword: {password}\n\n\
                     All sessions for this account were signed out.\n\
                     Delete this file after signing in.\n"
                ),
            );
            show_reset_message(
                "Runecord password reset",
                &format!(
                    "New password for {user}:\n\n{password}\n\n\
                     All sessions for this account were signed out.\n\n\
                     Also written to: {}",
                    file.display()
                ),
            )
        }
        Err(error) => show_reset_message("Runecord password reset failed", &error),
    }
}

#[cfg(not(feature = "embedded-server"))]
fn reset_local_password(_user: &str) {
    show_reset_message(
        "Runecord password reset",
        "This build has no embedded database. Run:\n\nrunecord-server --reset-password USER",
    );
}

/// A release build is a GUI app on Windows, so the reset result is a message
/// box rather than console output.
fn show_reset_message(title: &str, message: &str) {
    #[cfg(windows)]
    {
        use windows::Win32::UI::WindowsAndMessaging::{MB_ICONINFORMATION, MB_OK, MessageBoxW};
        use windows::core::PCWSTR;
        let title: Vec<u16> = title.encode_utf16().chain([0]).collect();
        let message: Vec<u16> = message.encode_utf16().chain([0]).collect();
        // SAFETY: both strings are NUL-terminated and live for the call.
        unsafe {
            MessageBoxW(
                None,
                PCWSTR(message.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_OK | MB_ICONINFORMATION,
            );
        }
    }
    #[cfg(not(windows))]
    {
        println!("{title}\n{message}");
    }
}

fn moderation_target() -> runecord_protocol::Id {
    std::env::var("RUNECORD_MOD_TARGET")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3)
}

const FONTS: &[&[u8]] = &[
    include_bytes!("../../../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../../../assets/fonts/Inter-Medium.ttf"),
    include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf"),
    include_bytes!("../../../assets/fonts/IBMPlexMono-Regular.ttf"),
];

/// The window, taskbar, tray and toast icon, next to the working directory or the executable.
fn icon_path() -> std::path::PathBuf {
    let icon = std::path::PathBuf::from("assets/icon.png");
    if icon.exists() {
        icon
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|d| d.join("assets").join("icon.png")))
            .unwrap_or(icon)
    }
}

/// Scales the app icon to the 32 px tray size.
fn tray_icon(bytes: &[u8]) -> Option<rune::TrayIcon> {
    let image = image::load_from_memory(bytes).ok()?.into_rgba8();
    let image = image::imageops::resize(&image, 32, 32, image::imageops::FilterType::Lanczos3);
    Some(rune::TrayIcon::rgba(32, 32, image.into_raw()))
}

pub fn run() {
    match reset_request() {
        ResetRequest::None => {}
        ResetRequest::User(user) => {
            reset_local_password(&user);
            return;
        }
        ResetRequest::MissingUser => {
            show_reset_message(
                "Runecord password reset",
                "Pass a local username: Runecord.exe --reset-password USER",
            );
            return;
        }
    }

    #[cfg(feature = "embedded-server")]
    crate::embedded::ensure_backend();
    crate::i18n::set_locale(crate::i18n::load_locale());

    let icon_path = icon_path();
    let icon_bytes = std::fs::read(&icon_path).ok();
    if let Some(bytes) = icon_bytes.as_deref() {
        let _ = rune::set_app_icon(bytes.to_vec());
    }
    let _ = rune_notify::register(
        &rune_notify::Identity::new("com.runecord.local", "Runecord").icon(icon_path),
    );
    let spec = rune::AppSpec {
        min_size: Some((960.0, 640.0)),
        fonts: FONTS,
        families: ("Inter", "IBM Plex Mono"),
        scenes: &[
            ("auth", |s: &mut App| s.logged_in = false),
            ("auth-register", |s: &mut App| {
                s.logged_in = false;
                s.auth_register = true;
            }),
            ("app-settings", |s: &mut App| {
                s.logged_in = true;
                s.open_settings(SettingsTab::Application);
            }),
            ("appearance", |s: &mut App| {
                s.logged_in = true;
                s.open_settings(SettingsTab::Appearance);
            }),
            ("light", |s: &mut App| s.dark = false),
            ("chat", |s: &mut App| {
                s.logged_in = true;
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 212;
            }),
            ("forum", |s: &mut App| {
                s.logged_in = true;
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 215;
            }),
            ("composer-format", |s: &mut App| {
                s.logged_in = true;
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 211;
                s.composer = rune::TextEdit::new("**bold** and `code` with a ||spoiler||");
            }),
            ("profile-edit", |s: &mut App| {
                s.logged_in = true;
                s.open_settings(SettingsTab::Profile);
                s.profile_display = rune::TextEdit::new("Nova Draft");
                s.profile_status = rune::TextEdit::new("previewing locally");
                s.profile_effect = "aurora".into();
                s.profile_nameplate = "midnight".into();
                s.profile_color = crate::theme::avatar_color_value(3);
            }),
            ("dms", |s: &mut App| {
                s.screen = Screen::Dms;
                s.dm_home = false;
                s.active_dm = 900;
            }),
            ("friends", |s: &mut App| {
                s.screen = Screen::Dms;
                s.dm_home = true;
            }),
            ("add-friend-panel", |s: &mut App| {
                s.screen = Screen::Dms;
                s.dm_home = true;
                s.add_friend_open = true;
            }),
            ("settings", |s: &mut App| {
                s.open_settings(SettingsTab::Appearance)
            }),
            ("create-channel", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_create_channel(Some(210));
            }),
            ("skeletons", |s: &mut App| {
                s.logged_in = true;
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 211;
                s.messages.clear();
                s.guild_details.clear();
                s.members.clear();
                if let Some(state) = s.member_state.get_mut(&100) {
                    state.loading = true;
                }
            }),
            ("app-settings", |s: &mut App| {
                s.open_settings(SettingsTab::Application)
            }),
            ("notifications", |s: &mut App| {
                s.open_settings(SettingsTab::Notifications)
            }),
            ("keybinds", |s: &mut App| {
                s.open_settings(SettingsTab::Keybinds)
            }),
            ("account", |s: &mut App| {
                s.open_settings(SettingsTab::Account)
            }),
            ("bots", |s: &mut App| s.open_settings(SettingsTab::Bots)),
            ("server-settings", |s: &mut App| s.open_server_settings()),
            ("server-overview", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                s.server_tab = ServerTab::Overview;
            }),
            ("role-icon", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                s.server_tab = ServerTab::Roles;
                s.select_role(902);
                if let Ok(path) = std::env::var("RUNECORD_ROLE_ICON") {
                    s.upload_role_icon(
                        902,
                        crate::net::UploadSource::Path(std::path::PathBuf::from(path)),
                    );
                }
            }),
            ("message-menu", |s: &mut App| {
                s.logged_in = true;
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 211;
                s.context_menu = Some(crate::app::ContextMenu {
                    kind: crate::app::ContextMenuKind::Message(1011),
                    anchor: crate::app::ContextMenuAnchor::Element("msg-actions-1011".into()),
                });
            }),
            ("everyone-message", |s: &mut App| {
                s.logged_in = true;
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 201;
                if let Some(messages) = s.messages.get_mut(&201) {
                    for message in messages.iter_mut() {
                        message.content = format!("@everyone {}", message.content);
                        message.mention_everyone = true;
                    }
                }
            }),
            ("everyone-mention", |s: &mut App| {
                s.logged_in = true;
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 201;
                s.composer = rune::TextEdit::new("@every");
            }),
            ("role-everyone", |s: &mut App| {
                s.logged_in = true;
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                s.server_tab = ServerTab::Roles;
                s.select_role(900);
            }),
            ("forward-message", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 211;
                s.modal = Modal::ForwardMessage(1011);
            }),
            ("gifs", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 211;
                s.gifs = vec![
                    runecord_protocol::GifItem {
                        slug: "party-parrot".into(),
                        title: "Party".into(),
                        tags: "party celebrate yay dance".into(),
                        url: String::new(),
                    },
                    runecord_protocol::GifItem {
                        slug: "thumbs-up".into(),
                        title: "Thumbs Up".into(),
                        tags: "nice good yes approve".into(),
                        url: String::new(),
                    },
                    runecord_protocol::GifItem {
                        slug: "mind-blown".into(),
                        title: "Mind Blown".into(),
                        tags: "wow shock amazing".into(),
                        url: String::new(),
                    },
                    runecord_protocol::GifItem {
                        slug: "loading".into(),
                        title: "Loading".into(),
                        tags: "wait spin loading".into(),
                        url: String::new(),
                    },
                    runecord_protocol::GifItem {
                        slug: "wave-hello".into(),
                        title: "Hello".into(),
                        tags: "hi hello wave hey".into(),
                        url: String::new(),
                    },
                    runecord_protocol::GifItem {
                        slug: "sparkles".into(),
                        title: "Sparkles".into(),
                        tags: "sparkle shine magic".into(),
                        url: String::new(),
                    },
                ];
                s.modal = Modal::GifPicker;
            }),
            ("locale", |s: &mut App| {
                crate::i18n::set_locale(crate::i18n::Locale::German);
                s.open_settings(SettingsTab::Appearance);
            }),
            ("overlay", |s: &mut App| {
                s.open_settings(SettingsTab::Application);
            }),
            ("stage", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 223;
                s.voice.insert(
                    1,
                    runecord_protocol::VoiceState {
                        user_id: 1,
                        guild_id: 100,
                        channel_id: Some(223),
                        self_mute: false,
                        self_deaf: false,
                    },
                );
                s.voice.insert(
                    3,
                    runecord_protocol::VoiceState {
                        user_id: 3,
                        guild_id: 100,
                        channel_id: Some(223),
                        self_mute: false,
                        self_deaf: false,
                    },
                );
                s.voice.insert(
                    2,
                    runecord_protocol::VoiceState {
                        user_id: 2,
                        guild_id: 100,
                        channel_id: Some(223),
                        self_mute: true,
                        self_deaf: false,
                    },
                );
                s.stages.insert(
                    223,
                    runecord_protocol::StageState {
                        channel_id: 223,
                        topic: "Roadmap AMA with the team".into(),
                        speakers: vec![
                            runecord_protocol::StageSpeaker {
                                user_id: 1,
                                speaking: true,
                                requested: false,
                            },
                            runecord_protocol::StageSpeaker {
                                user_id: 3,
                                speaking: true,
                                requested: false,
                            },
                            runecord_protocol::StageSpeaker {
                                user_id: 2,
                                speaking: false,
                                requested: true,
                            },
                        ],
                    },
                );
            }),
            ("user-volume", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 221;
                s.open_user_volume(2);
                crate::voice::set_user_volume(2, 1.4, false);
            }),
            ("events-rsvp", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 211;
                s.guild_events = vec![
                    runecord_protocol::GuildEvent {
                        id: 1,
                        guild_id: 100,
                        creator_id: 1,
                        name: "Weekly design review".into(),
                        description: "Bring screens and questions.".into(),
                        start_at: 1_760_000_000_000,
                        channel_id: Some(211),
                        created_at: 0,
                        recurrence: "weekly".into(),
                        duration_minutes: 45,
                        reminder_minutes: 10,
                        interested_count: 6,
                        my_status: "going".into(),
                    },
                    runecord_protocol::GuildEvent {
                        id: 2,
                        guild_id: 100,
                        creator_id: 2,
                        name: "Rune build party".into(),
                        description: String::new(),
                        start_at: 1_760_100_000_000,
                        channel_id: Some(213),
                        created_at: 0,
                        recurrence: String::new(),
                        duration_minutes: 120,
                        reminder_minutes: 0,
                        interested_count: 3,
                        my_status: "interested".into(),
                    },
                ];
                s.modal = Modal::Events;
            }),
            ("forum", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 215;
            }),
            ("forum-list", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 215;
                s.forum_list_layout = true;
            }),
            ("thread-tags", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.thread_parent = Some(215);
                s.active_channel = 216;
                s.open_thread_tags(216);
            }),
            ("thread-view", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.thread_parent = Some(215);
                s.active_channel = 216;
                s.channel_seen.insert(216, runecord_protocol::now_ms());
            }),
            ("thread-archived", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.thread_parent = Some(201);
                s.active_channel = 219;
                s.channel_seen.insert(219, runecord_protocol::now_ms());
            }),
            ("forum-post", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.thread_parent = Some(215);
                s.active_channel = 217;
                s.channel_seen.insert(217, runecord_protocol::now_ms());
            }),
            ("thread-create", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 201;
                s.profile_edit = rune::TextEdit::new("Ideas for the welcome channel");
                s.thread_body = rune::TextEdit::new(
                    "The intro could point at the events calendar and the roles picker.",
                );
                s.forum_guidelines_ok = false;
                s.modal = Modal::ThreadCreate {
                    parent: 201,
                    message: None,
                };
            }),
            ("forum-new-post", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 215;
                s.profile_edit = rune::TextEdit::new("Feedback on the new sidebar");
                s.thread_body = rune::TextEdit::new(
                    "The grouping reads much better at 1440p. A few notes on spacing below.",
                );
                s.thread_tag_ids = vec![2302];
                s.forum_guidelines_ok = false;
                s.modal = Modal::ThreadCreate {
                    parent: 215,
                    message: None,
                };
            }),
            ("audit", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                s.select_server_tab(ServerTab::Audit);
                s.audit_action = Some(22);
                s.audit = vec![
                    runecord_protocol::AuditEntry {
                        id: 9,
                        guild_id: 100,
                        actor_id: 1,
                        action_type: 22,
                        target_id: Some(5),
                        changes: serde_json::json!([{ "key": "reason", "new_value": "spam" }]),
                        created_at: 1_760_000_000_000,
                    },
                    runecord_protocol::AuditEntry {
                        id: 8,
                        guild_id: 100,
                        actor_id: 1,
                        action_type: 22,
                        target_id: Some(4),
                        changes: serde_json::json!([]),
                        created_at: 1_759_999_000_000,
                    },
                    runecord_protocol::AuditEntry {
                        id: 7,
                        guild_id: 100,
                        actor_id: 2,
                        action_type: 22,
                        target_id: Some(6),
                        changes: serde_json::json!([]),
                        created_at: 1_759_998_000_000,
                    },
                ];
            }),
            ("integrations", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                s.select_server_tab(ServerTab::Integrations);
                s.webhooks = vec![
                    runecord_protocol::WebhookInfo {
                        id: 1,
                        guild_id: 100,
                        channel_id: 211,
                        name: "Build bot".into(),
                        avatar_color: 0x5865f2,
                        token: None,
                    },
                    runecord_protocol::WebhookInfo {
                        id: 2,
                        guild_id: 100,
                        channel_id: 213,
                        name: "Design feed".into(),
                        avatar_color: 0xeb459e,
                        token: None,
                    },
                ];
                s.webhook_token = Some((1, "rcwh.9f2c4a7b1d3e5f6089ab".into()));
            }),
            ("templates", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                s.select_server_tab(ServerTab::Templates);
                s.templates = vec![
                    runecord_protocol::GuildTemplate {
                        id: 1,
                        guild_id: 100,
                        code: "bxu7j3wb9c00".into(),
                        name: "Community starter".into(),
                        description: "Channels and roles for a small community".into(),
                        uses: 4,
                        channel_count: 11,
                        role_count: 2,
                        created_at: 0,
                    },
                    runecord_protocol::GuildTemplate {
                        id: 2,
                        guild_id: 100,
                        code: "c9k2m4zz01ab".into(),
                        name: "Dev team".into(),
                        description: String::new(),
                        uses: 0,
                        channel_count: 6,
                        role_count: 4,
                        created_at: 0,
                    },
                ];
            }),
            ("automod", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                s.select_server_tab(ServerTab::AutoMod);
                s.automod_rules = vec![
                    runecord_protocol::AutoModRule {
                        id: 1,
                        name: "No spoilers".into(),
                        trigger: "keyword".into(),
                        pattern: "spoilers, leak".into(),
                        action: "block".into(),
                        timeout_minutes: 10,
                        enabled: true,
                    },
                    runecord_protocol::AutoModRule {
                        id: 2,
                        name: "Link spam".into(),
                        trigger: "link".into(),
                        pattern: String::new(),
                        action: "timeout".into(),
                        timeout_minutes: 10,
                        enabled: true,
                    },
                    runecord_protocol::AutoModRule {
                        id: 3,
                        name: "Shouting".into(),
                        trigger: "caps".into(),
                        pattern: "70".into(),
                        action: "alert".into(),
                        timeout_minutes: 10,
                        enabled: false,
                    },
                ];
            }),
            ("spoilers", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 211;
            }),
            ("channel-perms", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 211;
                s.open_channel_permissions(211);
            }),
            ("create-poll", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 211;
                s.open_create_poll();
            }),
            ("poll-message", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 211;
                if let Some(messages) = s.messages.get_mut(&211) {
                    messages.push(runecord_protocol::Message {
                        id: 9_999_999,
                        channel_id: 211,
                        author_id: 2,
                        timestamp: runecord_protocol::now_ms(),
                        poll: Some(runecord_protocol::Poll {
                            question: "Which channel should we build next?".into(),
                            options: vec![
                                runecord_protocol::PollOption {
                                    index: 0,
                                    text: "A design showcase".into(),
                                    votes: 7,
                                    me: true,
                                },
                                runecord_protocol::PollOption {
                                    index: 1,
                                    text: "A dev log".into(),
                                    votes: 3,
                                    me: false,
                                },
                                runecord_protocol::PollOption {
                                    index: 2,
                                    text: "A community art wall".into(),
                                    votes: 2,
                                    me: false,
                                },
                            ],
                            total_votes: 12,
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }
            }),
            ("channel-edit", |s: &mut App| {
                let id = std::env::var("RUNECORD_CHANNEL")
                    .ok()
                    .and_then(|value| value.parse::<runecord_protocol::Id>().ok())
                    .unwrap_or(211);
                s.open_channel_edit(id);
            }),
            ("dm-settings", |s: &mut App| {
                let channel = std::env::var("RUNECORD_DM")
                    .ok()
                    .and_then(|value| value.parse::<runecord_protocol::Id>().ok())
                    .unwrap_or(1140);
                s.screen = Screen::Dms;
                s.dm_home = false;
                s.active_dm = channel;
                s.open_dm(channel);
                s.open_dm_settings(channel);
                if let Ok(path) = std::env::var("RUNECORD_GROUP_ICON") {
                    s.begin_dm_icon_crop(
                        channel,
                        crate::net::UploadSource::Path(std::path::PathBuf::from(path)),
                    );
                }
            }),
            ("voice-settings", |s: &mut App| {
                s.open_settings(SettingsTab::Voice);
            }),
            ("profile-settings", |s: &mut App| {
                s.open_settings(SettingsTab::Profile)
            }),
            ("developer", |s: &mut App| s.developer = true),
            ("guild", |s: &mut App| {
                s.screen = Screen::Guild;
                s.dm_home = false;
            }),
            ("profile", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                let id = std::env::var("RUNECORD_PROFILE")
                    .ok()
                    .and_then(|value| value.parse::<runecord_protocol::Id>().ok())
                    .unwrap_or(2);
                s.modal = Modal::Profile(id);
            }),
            ("soundboard-live", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 221;
                s.ensure_guild(100);
            }),
            ("events", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 211;
                s.open_events();
            }),
            ("create-event", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 211;
                s.open_events();
                s.open_create_event();
            }),
            ("inbox", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 212;
                s.ensure_guild(100);
                s.open_inbox();
            }),
            ("profile-full", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                let id = std::env::var("RUNECORD_PROFILE")
                    .ok()
                    .and_then(|value| value.parse::<runecord_protocol::Id>().ok())
                    .unwrap_or(2);
                s.modal = Modal::FullProfile(id);
            }),
            ("reaction", |s: &mut App| s.modal = Modal::Reaction(1011)),
            ("emoji", |s: &mut App| s.modal = Modal::EmojiPicker),
            ("search", |s: &mut App| s.modal = Modal::Search),
            ("quick-switcher", |s: &mut App| {
                s.modal = Modal::QuickSwitcher
            }),
            ("invite", |s: &mut App| s.modal = Modal::Invite),
            ("add-friend", |s: &mut App| s.modal = Modal::AddFriend),
            ("threads", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.active_channel = 201;
                s.modal = Modal::Threads;
            }),
            ("leave", |s: &mut App| s.modal = Modal::ConfirmLeave(100)),
            ("notify", |s: &mut App| {
                s.modal = Modal::ChannelNotifications(211)
            }),
            ("privacy", |s: &mut App| s.modal = Modal::Privacy),
            ("kick", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.confirm_kick(moderation_target());
            }),
            ("ban", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.confirm_ban(moderation_target());
            }),
            ("timeout", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_timeout(moderation_target());
            }),
            ("timeout-days", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_timeout(moderation_target());
                s.timeout_amount = rune::TextEdit::new("3");
                s.timeout_unit = TimeoutUnit::Days;
            }),
            ("profile-mod", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.modal = Modal::Profile(moderation_target());
            }),
            ("profile-settings", |s: &mut App| {
                s.open_settings(SettingsTab::Profile);
            }),
            ("user-avatar-crop", |s: &mut App| {
                s.open_settings(SettingsTab::Profile);
                if let Ok(path) = std::env::var("RUNECORD_AVATAR") {
                    s.begin_user_image_crop(
                        crate::net::UploadSource::Path(std::path::PathBuf::from(path)),
                        false,
                    );
                }
            }),
            ("user-banner-crop", |s: &mut App| {
                s.open_settings(SettingsTab::Profile);
                if let Ok(path) = std::env::var("RUNECORD_USER_BANNER") {
                    s.begin_user_image_crop(
                        crate::net::UploadSource::Path(std::path::PathBuf::from(path)),
                        true,
                    );
                }
            }),
            ("guild-icon-crop", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                if let Ok(path) = std::env::var("RUNECORD_GUILD_ICON") {
                    s.begin_guild_image_crop(
                        crate::net::UploadSource::Path(std::path::PathBuf::from(path)),
                        false,
                    );
                }
            }),
            ("guild-banner-crop", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                if let Ok(path) = std::env::var("RUNECORD_GUILD_BANNER") {
                    s.begin_guild_image_crop(
                        crate::net::UploadSource::Path(std::path::PathBuf::from(path)),
                        true,
                    );
                }
            }),
            ("guild-image", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                if let Ok(path) = std::env::var("RUNECORD_GUILD_ICON") {
                    s.upload_guild_image(
                        crate::net::UploadSource::Path(std::path::PathBuf::from(path)),
                        false,
                    );
                }
                if let Ok(path) = std::env::var("RUNECORD_GUILD_BANNER") {
                    s.upload_guild_image(
                        crate::net::UploadSource::Path(std::path::PathBuf::from(path)),
                        true,
                    );
                }
            }),
            ("overview-edit", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                s.overview_name = rune::TextEdit::new("Rune Lounge HQ");
                s.overview_description =
                    rune::TextEdit::new("A home for interface obsessives, with an overview.");
                s.overview_tag = rune::TextEdit::new("HQ");
                s.overview_color = 0xeb459e;
                s.overview_discoverable = false;
            }),
            ("join", |s: &mut App| {
                s.open_invite_join();
                if let Ok(code) = std::env::var("RUNECORD_INVITE") {
                    s.join_code = rune::TextEdit::new(code);
                }
            }),
            ("appeal-form", |s: &mut App| {
                let code = std::env::var("RUNECORD_INVITE").unwrap_or_default();
                s.open_appeal(code);
            }),
            ("quality-set", |s: &mut App| {
                let quality = std::env::var("RUNECORD_VIDEO_QUALITY").unwrap_or_default();
                s.set_video_quality(crate::voice::VideoQuality::parse(&quality));
                s.screen = Screen::Guild;
            }),
            ("quality", |s: &mut App| {
                s.screen = Screen::Guild;
                s.open_video_quality();
            }),
            ("view-quality", |s: &mut App| {
                let quality = std::env::var("RUNECORD_VIEW_QUALITY").unwrap_or_default();
                s.set_viewer_quality(ViewQuality::parse(&quality));
                s.screen = Screen::Guild;
            }),
            ("view-quality-sheet", |s: &mut App| {
                let quality = std::env::var("RUNECORD_VIEW_QUALITY").unwrap_or_default();
                s.set_viewer_quality(ViewQuality::parse(&quality));
                s.screen = Screen::Guild;
                s.open_video_quality();
            }),
            ("prune", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                s.select_server_tab(ServerTab::Members);
                s.open_prune();
            }),
            ("server-bans", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                s.select_server_tab(ServerTab::Bans);
            }),
            ("bans", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_bans();
            }),
            ("modal", |s: &mut App| s.modal = Modal::CreateServer),
        ],
        poll: Some(App::poll),
        on_key: Some(app::on_key),
        setup: Some(App::setup),
        ..rune::AppSpec::new("Runecord", (1360.0, 860.0), App::new, ui::view)
    };
    let mut app = rune::App::new(spec)
        .titlebar(rune::Titlebar::new(ui::TITLEBAR_HEIGHT))
        .maximized(true);
    crate::overlay::install(&mut app);

    {
        let tray_icon = icon_bytes
            .as_deref()
            .and_then(tray_icon)
            .unwrap_or_else(|| {
                let mut rgba = Vec::with_capacity(32 * 32 * 4);
                for y in 0..32 {
                    for x in 0..32 {
                        let dx = x as f32 - 15.5;
                        let dy = y as f32 - 15.5;
                        if dx * dx + dy * dy <= 15.0 * 15.0 {
                            rgba.extend_from_slice(&[88, 101, 242, 255]);
                        } else {
                            rgba.extend_from_slice(&[0, 0, 0, 0]);
                        }
                    }
                }
                rune::TrayIcon::rgba(32, 32, rgba)
            });
        let handle = app.handle();
        let _tray = rune::Tray::new(
            rune::TraySpec::new("runecord", "Runecord", tray_icon)
                .tooltip("Runecord")
                .menu(vec![
                    rune::TrayItem::action("open", "Open Runecord"),
                    rune::TrayItem::Separator,
                    rune::TrayItem::action("quit", "Quit Runecord"),
                ]),
            move |event| match event {
                rune::TrayEvent::Activate { .. } => rune_notify::bring_to_front(),
                rune::TrayEvent::Menu(id) if id == "open" => rune_notify::bring_to_front(),
                rune::TrayEvent::Menu(id) if id == "quit" => handle.update(|_, tx| tx.quit()),
                _ => {}
            },
        )
        .ok();
    }

    app.run();
}
