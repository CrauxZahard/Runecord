//! Runecord: a local Discord clone built with Rune.

use crate::app::{App, Modal, Screen, ServerTab, SettingsTab, TimeoutUnit};
use crate::voice::ViewQuality;
use crate::{app, ui};

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
    #[cfg(feature = "embedded-server")]
    crate::embedded::ensure_backend();

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
            ("light", |s: &mut App| s.dark = false),
            ("dms", |s: &mut App| {
                s.screen = Screen::Dms;
                s.dm_home = false;
                s.active_dm = 900;
            }),
            ("friends", |s: &mut App| {
                s.screen = Screen::Dms;
                s.dm_home = true;
            }),
            ("settings", |s: &mut App| {
                s.open_settings(SettingsTab::Appearance)
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
            ("role-icon", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                s.server_tab = ServerTab::Roles;
                s.select_role(902);
                if let Ok(path) = std::env::var("RUNECORD_ROLE_ICON") {
                    s.upload_role_icon(902, std::path::PathBuf::from(path));
                }
            }),
            ("message-menu", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_channel = 201;
                s.modal = Modal::MessageMenu(1011)
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
                    s.begin_dm_icon_crop(channel, std::path::PathBuf::from(path));
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
            ("threads", |s: &mut App| s.modal = Modal::Threads),
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
                    s.begin_user_image_crop(std::path::PathBuf::from(path), false);
                }
            }),
            ("user-banner-crop", |s: &mut App| {
                s.open_settings(SettingsTab::Profile);
                if let Ok(path) = std::env::var("RUNECORD_USER_BANNER") {
                    s.begin_user_image_crop(std::path::PathBuf::from(path), true);
                }
            }),
            ("guild-icon-crop", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                if let Ok(path) = std::env::var("RUNECORD_GUILD_ICON") {
                    s.begin_guild_image_crop(std::path::PathBuf::from(path), false);
                }
            }),
            ("guild-banner-crop", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                if let Ok(path) = std::env::var("RUNECORD_GUILD_BANNER") {
                    s.begin_guild_image_crop(std::path::PathBuf::from(path), true);
                }
            }),
            ("guild-image", |s: &mut App| {
                s.screen = Screen::Guild;
                s.active_guild = 100;
                s.open_server_settings();
                if let Ok(path) = std::env::var("RUNECORD_GUILD_ICON") {
                    s.upload_guild_image(std::path::PathBuf::from(path), false);
                }
                if let Ok(path) = std::env::var("RUNECORD_GUILD_BANNER") {
                    s.upload_guild_image(std::path::PathBuf::from(path), true);
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
    let app = rune::App::new(spec).titlebar(rune::Titlebar::new(32.0));

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

    app.run();
}
