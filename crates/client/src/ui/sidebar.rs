//! Channel strip surface: the server, its channels and your account run as one
//! horizontal band directly under the navigation dock.

use super::*;
use runecord_protocol::NotificationLevel;

pub(crate) enum ChannelListRow {
    Category(Channel),
    Channel(Channel),
    VoiceUser(Id),
}

/// The signature detail: a vertical sunset edge, three thin strips of amber,
/// magenta and violet stacked into one glowing seam on the active channel.
fn sunset_edge(height: f32, p: Palette) -> El {
    col()
        .key("sunset-edge")
        .w(3.0)
        .h(height)
        .radius(2.0)
        .clip_rounded(2.0)
        .child(el().w_full().grow(1.0).fill(theme::sunset_amber(p.dark)))
        .child(el().w_full().grow(1.0).fill(theme::sunset_magenta(p.dark)))
        .child(el().w_full().grow(1.0).fill(theme::sunset_violet(p.dark)))
}

pub(crate) fn sidebar(s: &App, p: Palette) -> El {
    let guild = s.active_guild().cloned().unwrap_or_default();
    let header = row()
        .key("guild-header")
        .w(s.sidebar_width.clamp(140.0, 300.0))
        .h(36.0)
        .px(10.0)
        .radius(16.0)
        .gap(6.0)
        .center_y()
        .fill(p.sidebar_active)
        .gradient(p.sidebar_active, p.sidebar_hover)
        .border(1.0, theme::gloss_edge(p))
        .hover(p.sidebar_hover)
        .window_drag()
        .role("button")
        .label(format!("{} server menu", guild.name))
        .motion(motion::SNAPPY)
        .on_press(|s: &mut App| s.modal = Modal::ServerMenu)
        .child(guild_icon(s, &guild, 22.0, 8.0, p))
        .child(
            text(guild.name.clone())
                .font(theme::SMALL_MED)
                .color(p.text_bright)
                .grow(1.0)
                .min_w(0.0)
                .truncate(),
        )
        .child(icon(&icons::CHEVRON_DOWN, 14.0).color(p.accent));

    let channels = s.guild_channels(s.active_guild);
    let mut ordered: Vec<&Channel> = channels.iter().collect();
    ordered.sort_by_key(|c| c.position);
    let mut rows: Vec<ChannelListRow> = Vec::new();
    for category in ordered.iter().filter(|c| c.kind == ChannelKind::Category) {
        rows.push(ChannelListRow::Category((*category).clone()));
        if s.collapsed.contains(&category.id) {
            continue;
        }
        for channel in ordered.iter().filter(|c| c.parent_id == Some(category.id)) {
            rows.push(ChannelListRow::Channel((*channel).clone()));
            push_voice_users(&mut rows, channel, s);
        }
    }
    for channel in ordered.iter().filter(|c| {
        c.parent_id.is_none() && c.kind != ChannelKind::Category && c.kind != ChannelKind::Thread
    }) {
        rows.push(ChannelListRow::Channel((*channel).clone()));
        push_voice_users(&mut rows, channel, s);
    }

    let mut row_list = row().key("channel-rows").gap(4.0).center_y().min_w(0.0);
    if channels.is_empty() && s.guild_detail(s.active_guild).is_none() {
        row_list = row_list.child(skeleton_channel_rows(2, p, s.clock));
    }
    for item in &rows {
        row_list = row_list.child(match item {
            ChannelListRow::Category(category) => el()
                .key(format!("cat-{}", category.id))
                .child(category_row(category, s, p)),
            ChannelListRow::Channel(channel) => el()
                .key(format!("ch-{}", channel.id))
                .child(channel_row(channel, s, p)),
            ChannelListRow::VoiceUser(user) => el()
                .key(format!("sidebar-voice-{user}"))
                .child(voice_user_row(*user, s, p)),
        });
    }
    let list = row().key("channel-list").gap(4.0).center_y().min_w(0.0);

    row()
        .key("sidebar")
        .w_full()
        .h(54.0)
        .px(10.0)
        .gap(8.0)
        .center_y()
        .fill(p.sidebar)
        .border_sides([0.0, 0.0, 1.0, 0.0], p.divider_soft)
        .child(header)
        .child(strip_grip(p))
        .child(shortcuts(p))
        .child(
            row()
                .key("channels-scroll")
                .grow(1.0)
                .min_w(0.0)
                .gap(4.0)
                .center_y()
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list.child(row_list)),
        )
}

/// A short drag edge between the server plate and the channel pills; it sizes
/// the plate, which is the one part of the strip with a width to give away.
fn strip_grip(p: Palette) -> El {
    el().key("sidebar-resize")
        .w(5.0)
        .h(28.0)
        .radius(3.0)
        .fill(Color::TRANSPARENT)
        .hover(p.accent.alpha(0.45))
        .cursor(rune::Cursor::ResizeColumn)
        .role("separator")
        .label("Resize the channel list")
        .help("Drag to resize; double-click to reset")
        .on_drag(
            |s: &mut App, drag: &mut rune::Drag, _: &mut rune::Tx| match drag.phase {
                rune::Phase::Down => drag.data = s.sidebar_width,
                rune::Phase::Move => {
                    s.sidebar_width = (drag.data + drag.dx).clamp(140.0, 300.0).round();
                }
                rune::Phase::Up => {}
            },
        )
        .on_double_press_tx(|s: &mut App, _: &mut rune::Tx| {
            s.sidebar_width = 240.0;
        })
}

fn shortcuts(p: Palette) -> El {
    let short = |key: &'static str, ic: &'static Icon, label: String, run: fn(&mut App)| {
        el().key(key)
            .size(30.0, 30.0)
            .radius(15.0)
            .center()
            .role("button")
            .label(label.clone())
            .help(label)
            .hover(p.sidebar_hover)
            .on_press(run)
            .child(icon(ic, 16.0).color(p.text_muted))
    };
    row()
        .key("server-shortcuts")
        .gap(2.0)
        .center_y()
        .child(short(
            "sidebar-events",
            &icons::CLOCK,
            crate::i18n::t("Events"),
            |s: &mut App| s.open_events(),
        ))
        .child(short(
            "sidebar-members",
            &icons::USERS,
            crate::i18n::t("Members"),
            |s: &mut App| {
                s.open_server_settings();
                s.server_tab = crate::app::ServerTab::Members;
            },
        ))
        .child(short(
            "sidebar-channels-roles",
            &icons::SETTINGS,
            "Channels and Roles".to_string(),
            |s: &mut App| {
                s.open_server_settings();
                s.server_tab = crate::app::ServerTab::Roles;
            },
        ))
}

pub(crate) fn voice_connected_panel(s: &App, p: Palette) -> Option<El> {
    let channel_id = s.my_voice_channel()?;
    let channel = s.channel(channel_id)?;
    let guild = s.active_guild();
    let action = |key: &'static str,
                  icon_id: &'static Icon,
                  label: &'static str,
                  active: bool,
                  p: Palette,
                  run: fn(&mut App)| {
        el().key(key)
            .size(26.0, 26.0)
            .radius(13.0)
            .center()
            .role("button")
            .label(label)
            .help(label)
            .fill(if active {
                p.green.alpha(0.25)
            } else {
                Color::TRANSPARENT
            })
            .hover(p.rail_hover)
            .on_press(run)
            .child(icon(icon_id, 15.0).color(if active { p.green } else { p.text_bright }))
    };
    Some(
        row()
            .key("voice-connected")
            .w(258.0)
            .h(40.0)
            .px(8.0)
            .gap(4.0)
            .center_y()
            .radius(20.0)
            .fill(p.user_panel)
            .border(1.0, p.green.alpha(0.45))
            .child(icon(&icons::VOICE, 16.0).color(p.green))
            .child(
                col()
                    .grow(1.0)
                    .min_w(0.0)
                    .justify_center()
                    .child(text("Voice Connected").font(theme::TINY).color(p.green))
                    .child(
                        text(match guild {
                            Some(guild) => format!("{} / {}", channel.name, guild.name),
                            None => channel.name.clone(),
                        })
                        .font(theme::TINY)
                        .color(p.text_muted)
                        .truncate(),
                    ),
            )
            .child(action(
                "voice-camera",
                &icons::VIDEO,
                "Turn on camera",
                s.voice_camera,
                p,
                App::toggle_voice_camera,
            ))
            .child(action(
                "voice-screen",
                &icons::MONITOR,
                "Share your screen",
                s.voice_screen,
                p,
                App::toggle_voice_screen,
            ))
            .child(action(
                "voice-chat",
                &icons::CHAT,
                "Voice chat",
                s.voice_chat,
                p,
                |s: &mut App| s.voice_chat = !s.voice_chat,
            ))
            .child(action(
                "voice-soundboard",
                &icons::MEGAPHONE,
                "Soundboard",
                false,
                p,
                |s: &mut App| s.modal = Modal::Soundboard,
            ))
            .child(
                el().key("voice-disconnect")
                    .size(26.0, 26.0)
                    .radius(13.0)
                    .center()
                    .role("button")
                    .label("Disconnect from voice")
                    .help("Disconnect")
                    .hover(p.rail_hover)
                    .on_press(|s: &mut App| s.set_voice(None))
                    .child(icon(&icons::PHONE, 15.0).color(p.text_bright)),
            ),
    )
}

pub(crate) fn category_row(cat: &Channel, s: &App, p: Palette) -> El {
    let id = cat.id;
    let collapsed = s.collapsed.contains(&id);
    let manage = s.guild_can(perm::MANAGE_CHANNELS);
    row()
        .key(format!("category-{}", cat.id))
        .h(30.0)
        .px(6.0)
        .radius(10.0)
        .gap(4.0)
        .center_y()
        .role("button")
        .label(format!(
            "{} category, {}",
            cat.name,
            if collapsed { "collapsed" } else { "expanded" }
        ))
        .expanded(!collapsed)
        .motion(motion::SNAPPY)
        .hover(p.sidebar_hover)
        .hover_group()
        .on_press(move |s: &mut App| {
            if !s.collapsed.remove(&id) {
                s.collapsed.insert(id);
            }
        })
        .child(
            icon(&icons::CHEVRON_DOWN, 11.0)
                .color(p.text_faint)
                .icon_rotation(if collapsed {
                    -std::f32::consts::FRAC_PI_2
                } else {
                    0.0
                }),
        )
        .child(
            text(cat.name.to_uppercase())
                .font(theme::SECTION)
                .color(p.text_link)
                .max_w(150.0)
                .truncate(),
        )
        .children(manage.then(|| {
            el().key(format!("category-plus-{}", cat.id))
                .size(16.0, 16.0)
                .radius(8.0)
                .center()
                .role("button")
                .label(format!("Create channel in {}", cat.name))
                .help("Create Channel")
                .hover(p.sidebar_hover)
                .on_press(move |s: &mut App| s.open_create_channel(Some(id)))
                .child(icon(&icons::PLUS, 12.0).color(p.text_muted))
        }))
}

fn push_voice_users(rows: &mut Vec<ChannelListRow>, channel: &Channel, s: &App) {
    if !channel.kind.is_voice_like() {
        return;
    }
    let mut users: Vec<Id> = s
        .voice
        .values()
        .filter(|state| state.channel_id == Some(channel.id))
        .map(|state| state.user_id)
        .collect();
    users.sort_by_key(|user| {
        s.user(*user)
            .map(|user| user.display_name.clone())
            .unwrap_or_default()
    });
    for user in users {
        rows.push(ChannelListRow::VoiceUser(user));
    }
}

fn voice_user_row(user_id: Id, s: &App, p: Palette) -> El {
    let user = s.user(user_id).cloned().unwrap_or_default();
    let muted = s.voice.get(&user_id).is_some_and(|state| state.self_mute);
    row()
        .key(format!("sidebar-voice-{user_id}"))
        .h(30.0)
        .px(4.0)
        .radius(10.0)
        .gap(4.0)
        .center_y()
        .role("button")
        .label(format!("{} connected to voice", user.display_name))
        .hover(p.sidebar_hover)
        .on_press(move |s: &mut App| s.modal = Modal::Profile(user_id))
        .children({
            let speaking = crate::voice::user_speaking(user_id);
            let avatar = avatar_status(&user, 20.0, p, p.sidebar, true);
            [if speaking {
                avatar.border(2.0, theme::sunset_magenta(p.dark))
            } else {
                avatar
            }]
        })
        .child(icon(&icons::MIC, 11.0).color(if muted { p.danger } else { p.text_faint }))
}

pub(crate) fn channel_row(ch: &Channel, s: &App, p: Palette) -> El {
    let selected = s.screen == Screen::Guild && s.active_channel == ch.id;
    let unread = s.unread.get(&ch.id).copied().unwrap_or(0);
    let id = ch.id;
    let ic = match ch.kind {
        ChannelKind::Voice | ChannelKind::Stage => &icons::VOICE,
        ChannelKind::Announcement => &icons::MEGAPHONE,
        ChannelKind::Forum => &icons::FORUM,
        ChannelKind::Thread => &icons::THREAD,
        _ => &icons::HASH,
    };
    let kind_label = match ch.kind {
        ChannelKind::Voice => "Voice channel",
        ChannelKind::Stage => "Stage channel",
        ChannelKind::Announcement => "Announcement channel",
        ChannelKind::Forum => "Forum channel",
        ChannelKind::Thread => "Thread",
        ChannelKind::Category => "Category",
        _ => "Text channel",
    };
    let mut label = if unread > 0 {
        format!("{kind_label} {}, {unread} unread", ch.name)
    } else {
        format!("{kind_label} {}", ch.name)
    };
    if ch.kind.is_voice_like() && !ch.status.is_empty() {
        label = format!("{label}, {}", ch.status);
    }
    let mut row = row()
        .key(format!("channel-{}", ch.id))
        .h(32.0)
        .px(10.0)
        .radius(16.0)
        .gap(6.0)
        .center_y()
        .role("button")
        .label(label)
        .motion(motion::SNAPPY)
        // The selected pill keeps the sunset: magenta washing down into the
        // aubergine, ringed by the thin bright edge every card carries.
        .fill(if selected { p.selected } else { p.input })
        .gradient(
            if selected {
                theme::sunset_magenta(p.dark).alpha(0.55)
            } else {
                p.input
            },
            if selected { p.selected } else { p.input },
        )
        .border(
            1.0,
            if selected {
                theme::gloss_edge(p)
            } else {
                Color::TRANSPARENT
            },
        )
        .hover(if selected {
            p.selected
        } else {
            p.sidebar_hover
        })
        .on_secondary_press(move |s: &mut App, press, _: &mut rune::Tx| {
            s.context_menu = Some(crate::app::ContextMenu {
                kind: crate::app::ContextMenuKind::Channel(id),
                anchor: crate::app::ContextMenuAnchor::Pointer {
                    x: press.x,
                    y: press.y,
                },
            });
        })
        .on_press(move |s: &mut App| {
            s.screen = Screen::Guild;
            s.switch_channel(id);
        })
        .children(selected.then(|| sunset_edge(18.0, p)))
        .child(icon(ic, 16.0).color(if selected {
            theme::sunset_amber(p.dark)
        } else {
            p.text_faint
        }))
        .child(
            text(ch.name.clone())
                .font(theme::SMALL_MED)
                .color(if selected || unread > 0 {
                    p.text_bright
                } else {
                    p.text_muted
                })
                .max_w(170.0)
                .truncate(),
        );
    if s.channel_notify.get(&ch.id).copied().unwrap_or_default() == NotificationLevel::None {
        row = row.child(icon(&icons::BELL_OFF, 12.0).color(p.text_faint));
    }
    if unread > 0 {
        row = row.child(
            el().key("badge")
                .h(16.0)
                .min_w(16.0)
                .px(4.0)
                .radius(8.0)
                .fill(p.danger)
                .center()
                .child(
                    text(unread.to_string())
                        .font(theme::TINY)
                        .color(Color::WHITE),
                ),
        );
    }
    row
}

pub(crate) fn user_panel(s: &App, p: Palette) -> El {
    let status = s.me.custom_status.clone();
    row()
        .key("user-panel")
        .w(206.0)
        .h(40.0)
        .px(6.0)
        .gap(2.0)
        .center_y()
        .radius(20.0)
        .fill(p.user_panel)
        .border(1.0, theme::gloss_edge(p))
        .role("group")
        .label("Your account controls")
        .child(
            el().key("my-status")
                .size(30.0, 30.0)
                .radius(15.0)
                .center()
                .role("button")
                .label("Set your status")
                .help("Set your status")
                .on_press(|s: &mut App| s.modal = Modal::StatusPicker)
                .child(avatar_status(&s.me, 30.0, p, p.user_panel, true)),
        )
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .justify_center()
                .pl(4.0)
                .child(
                    text(s.me.display_name.clone())
                        .font(theme::SMALL_MED)
                        .color(p.text_bright)
                        .truncate(),
                )
                .child(
                    text(if status.is_empty() {
                        s.me.status.label().to_string()
                    } else {
                        status
                    })
                    .font(theme::TINY)
                    .color(p.text_muted)
                    .truncate(),
                ),
        )
        .child(
            panel_button("mic", &icons::MIC, p, s.voice_mute)
                .on_press(|s: &mut App| s.toggle_voice_mute()),
        )
        .child(
            panel_button("deafen", &icons::HEADPHONES, p, s.voice_deaf)
                .on_press(|s: &mut App| s.toggle_voice_deafen()),
        )
        .child(
            panel_button("settings", &icons::SETTINGS, p, false)
                .on_press(|s: &mut App| s.open_settings(SettingsTab::Account)),
        )
}
