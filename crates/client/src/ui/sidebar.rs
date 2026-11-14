//! Channel sidebar surface.

use super::*;
use runecord_protocol::NotificationLevel;

pub(crate) enum ChannelListRow {
    Category(Channel),
    Channel(Channel),
    VoiceUser(Id),
}

pub(crate) fn sidebar(s: &App, p: Palette) -> El {
    let guild = s.active_guild().cloned().unwrap_or_default();
    let header = row()
        .key("guild-header")
        .w_full()
        .h(48.0)
        .px(16.0)
        .center_y()
        .gap(4.0)
        .fill(p.sidebar)
        .hover(p.sidebar_hover)
        .window_drag()
        .role("button")
        .label(format!("{} server menu", guild.name))
        .motion(motion::SNAPPY)
        .on_press(|s: &mut App| s.modal = Modal::ServerMenu)
        .child(
            text(guild.name.clone())
                .font(theme::NAME)
                .color(p.text_bright)
                .grow(1.0)
                .min_w(0.0)
                .truncate(),
        )
        .child(icon(&icons::CHEVRON_DOWN, 16.0).color(p.text_muted));

    let mut list = col()
        .key("channel-list")
        .w_full()
        .px(10.0)
        .pt(6.0)
        .pb(10.0)
        .gap(1.0);
    let channels = s.guild_channels(s.active_guild);
    if channels.is_empty() && s.guild_detail(s.active_guild).is_none() {
        list = list.child(skeleton_channel_rows(6, p, s.clock));
    }
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
    let mut row_list = col()
        .key("channel-rows")
        .w_full()
        .growth(GrowthSpec::default());
    for item in &rows {
        row_list = row_list.child(match item {
            ChannelListRow::Category(category) => el()
                .key(format!("cat-{}", category.id))
                .w_full()
                .child(category_row(category, s, p)),
            ChannelListRow::Channel(channel) => el()
                .key(format!("ch-{}", channel.id))
                .w_full()
                .child(channel_row(channel, s, p)),
            ChannelListRow::VoiceUser(user) => el()
                .key(format!("sidebar-voice-{user}"))
                .w_full()
                .child(voice_user_row(*user, s, p)),
        });
    }
    list = list.child(row_list);

    col()
        .key("sidebar")
        .w(240.0)
        .h_full()
        .fill(p.sidebar)
        .children((!guild.banner_url.is_empty()).then(|| guild_banner_el(s, &guild, 104.0, p)))
        .child(header)
        .child(
            col()
                .key("server-shortcuts")
                .w_full()
                .px(10.0)
                .pt(2.0)
                .pb(4.0)
                .gap(1.0)
                .child(
                    row()
                        .key("sidebar-events")
                        .w_full()
                        .h(30.0)
                        .px(8.0)
                        .radius(4.0)
                        .gap(8.0)
                        .center_y()
                        .role("button")
                        .label("Events")
                        .hover(p.sidebar_hover)
                        .on_press(|s: &mut App| s.open_events())
                        .child(icon(&icons::CLOCK, 16.0).color(p.text_muted))
                        .child(text("Events").font(theme::SMALL_MED).color(p.text)),
                )
                .child(
                    row()
                        .key("sidebar-members")
                        .w_full()
                        .h(30.0)
                        .px(8.0)
                        .radius(4.0)
                        .gap(8.0)
                        .center_y()
                        .role("button")
                        .label("Members")
                        .hover(p.sidebar_hover)
                        .on_press(|s: &mut App| {
                            s.open_server_settings();
                            s.server_tab = crate::app::ServerTab::Members;
                        })
                        .child(icon(&icons::USERS, 16.0).color(p.text_muted))
                        .child(text("Members").font(theme::SMALL_MED).color(p.text)),
                )
                .child(
                    row()
                        .key("sidebar-channels-roles")
                        .w_full()
                        .h(30.0)
                        .px(8.0)
                        .radius(4.0)
                        .gap(8.0)
                        .center_y()
                        .role("button")
                        .label("Channels and Roles")
                        .hover(p.sidebar_hover)
                        .on_press(|s: &mut App| {
                            s.open_server_settings();
                            s.server_tab = crate::app::ServerTab::Roles;
                        })
                        .child(icon(&icons::SETTINGS, 16.0).color(p.text_muted))
                        .child(
                            text("Channels & Roles")
                                .font(theme::SMALL_MED)
                                .color(p.text),
                        ),
                ),
        )
        .child(
            col()
                .key("channels-scroll")
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list),
        )
        .children(voice_connected_panel(s, p))
        .child(user_panel(s, p))
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
            .size(28.0, 28.0)
            .radius(4.0)
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
            .child(icon(icon_id, 16.0).color(if active { p.green } else { p.text_bright }))
    };
    Some(
        col()
            .key("voice-connected")
            .w_full()
            .fill(p.user_panel)
            .border_sides([1.0, 0.0, 0.0, 0.0], p.divider)
            .child(
                row()
                    .w_full()
                    .h(46.0)
                    .px(10.0)
                    .gap(8.0)
                    .center_y()
                    .child(icon(&icons::VOICE, 18.0).color(p.green))
                    .child(
                        col()
                            .grow(1.0)
                            .min_w(0.0)
                            .child(
                                text("Voice Connected")
                                    .font(theme::SMALL_MED)
                                    .color(p.green),
                            )
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
                    .child(
                        el().key("voice-disconnect")
                            .size(28.0, 28.0)
                            .radius(4.0)
                            .center()
                            .role("button")
                            .label("Disconnect from voice")
                            .help("Disconnect")
                            .hover(p.rail_hover)
                            .on_press(|s: &mut App| s.set_voice(None))
                            .child(icon(&icons::PHONE, 17.0).color(p.text_bright)),
                    ),
            )
            .child(
                row()
                    .w_full()
                    .h(34.0)
                    .px(8.0)
                    .pb(4.0)
                    .gap(4.0)
                    .center_y()
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
                    .child(el().grow(1.0)),
            ),
    )
}

pub(crate) fn category_row(cat: &Channel, s: &App, p: Palette) -> El {
    let id = cat.id;
    let collapsed = s.collapsed.contains(&id);
    let manage = s.guild_can(perm::MANAGE_CHANNELS);
    row()
        .key(format!("category-{}", cat.id))
        .w_full()
        .h(34.0)
        .px(10.0)
        .gap(6.0)
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
            icon(&icons::CHEVRON_DOWN, 12.0)
                .color(p.text_muted)
                .group_opacity(if collapsed { 1.0 } else { 0.0 }, 1.0)
                .icon_rotation(if collapsed {
                    -std::f32::consts::FRAC_PI_2
                } else {
                    0.0
                }),
        )
        .child(
            text(cat.name.to_uppercase())
                .font(theme::SECTION)
                .color(p.text_muted)
                .grow(1.0),
        )
        .children(manage.then(|| {
            el().key(format!("category-plus-{}", cat.id))
                .size(18.0, 18.0)
                .radius(4.0)
                .center()
                .role("button")
                .label(format!("Create channel in {}", cat.name))
                .help("Create Channel")
                .hover(p.sidebar_hover)
                .on_press(move |s: &mut App| {
                    s.create_parent = Some(id);
                    s.profile_edit.clear();
                    s.modal = Modal::CreateChannel;
                })
                .child(icon(&icons::PLUS, 14.0).color(p.text_muted))
        }))
}

fn push_voice_users(rows: &mut Vec<ChannelListRow>, channel: &Channel, s: &App) {
    if channel.kind != ChannelKind::Voice {
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
        .w_full()
        .h(30.0)
        .pl(36.0)
        .pr(10.0)
        .radius(4.0)
        .gap(6.0)
        .center_y()
        .role("button")
        .label(format!("{} connected to voice", user.display_name))
        .hover(p.sidebar_hover)
        .on_press(move |s: &mut App| s.modal = Modal::Profile(user_id))
        .child(avatar_status(&user, 20.0, p, p.sidebar, true))
        .child(
            text(user.display_name.clone())
                .font(theme::TINY)
                .color(p.text_muted)
                .grow(1.0)
                .truncate(),
        )
        .child(icon(&icons::MIC, 12.0).color(if muted { p.danger } else { p.text_faint }))
}

pub(crate) fn channel_row(ch: &Channel, s: &App, p: Palette) -> El {
    let selected = s.screen == Screen::Guild && s.active_channel == ch.id;
    let unread = s.unread.get(&ch.id).copied().unwrap_or(0);
    let id = ch.id;
    let ic = match ch.kind {
        ChannelKind::Voice => &icons::VOICE,
        ChannelKind::Announcement => &icons::MEGAPHONE,
        ChannelKind::Forum => &icons::FORUM,
        ChannelKind::Thread => &icons::THREAD,
        _ => &icons::HASH,
    };
    let kind_label = match ch.kind {
        ChannelKind::Voice => "Voice channel",
        ChannelKind::Announcement => "Announcement channel",
        ChannelKind::Forum => "Forum channel",
        ChannelKind::Thread => "Thread",
        ChannelKind::Category => "Category",
        _ => "Text channel",
    };
    let manage =
        has(ch.my_permissions, perm::MANAGE_CHANNELS) || s.guild_can(perm::MANAGE_CHANNELS);
    let subtitle = if ch.kind == ChannelKind::Voice {
        if !ch.status.is_empty() {
            Some(ch.status.clone())
        } else if manage {
            Some("Set a channel status".to_string())
        } else {
            None
        }
    } else {
        None
    };
    let mut row = row()
        .key(format!("channel-{}", ch.id))
        .w_full()
        .h(if subtitle.is_some() { 46.0 } else { 34.0 })
        .px(10.0)
        .radius(5.0)
        .gap(6.0)
        .center_y()
        .role("button")
        .label(if unread > 0 {
            format!("{kind_label} {}, {unread} unread", ch.name)
        } else {
            format!("{kind_label} {}", ch.name)
        })
        .motion(motion::SNAPPY)
        .fill(if selected {
            p.selected
        } else {
            Color::TRANSPARENT
        })
        .hover(if selected {
            p.selected
        } else {
            p.sidebar_hover
        })
        .on_secondary_press(move |_s: &mut App, press, tx: &mut rune::Tx| {
            if !rune::popup_menu_supported() {
                return;
            }
            let mut menu = rune::PopupMenu::new()
                .item(rune::PopupItem::new("open", "Open Channel"))
                .item(rune::PopupItem::new("copy", "Copy Link"))
                .item(rune::PopupItem::new("mute", "Mute Channel"));
            if manage {
                menu = menu
                    .separator()
                    .item(rune::PopupItem::new("edit", "Edit Channel"))
                    .item(rune::PopupItem::new("perms", "Edit Permissions"))
                    .item(rune::PopupItem::new("delete", "Delete Channel"));
            }
            tx.popup_menu(
                menu,
                press,
                move |s: &mut App, picked, tx: &mut rune::Tx| match picked {
                    Some("open") => {
                        s.screen = Screen::Guild;
                        s.switch_channel(id);
                    }
                    Some("copy") => tx.copy(format!("runecord://channel/{id}")),
                    Some("mute") => s.mute_channel(id),
                    Some("edit") => s.open_channel_edit(id),
                    Some("perms") => s.modal = Modal::ChannelPermissions(id),
                    Some("delete") => s.confirm_delete_channel(id),
                    _ => {}
                },
            );
        })
        .on_press(move |s: &mut App| {
            s.screen = Screen::Guild;
            s.switch_channel(id);
        })
        .child(icon(ic, 20.0).color(if selected {
            p.interactive
        } else {
            p.text_faint
        }))
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(0.0)
                .child(
                    text(ch.name.clone())
                        .font(theme::BODY_MED)
                        .color(if selected || unread > 0 {
                            p.text_bright
                        } else {
                            p.text_muted
                        })
                        .truncate(),
                )
                .children(subtitle.map(|subtitle| {
                    text(subtitle)
                        .font(theme::TINY)
                        .color(p.text_faint)
                        .truncate()
                })),
        );
    if s.channel_notify.get(&ch.id).copied().unwrap_or_default() == NotificationLevel::None {
        row = row.child(icon(&icons::BELL_OFF, 14.0).color(p.text_faint));
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
        .w_full()
        .h(56.0)
        .px(10.0)
        .gap(2.0)
        .center_y()
        .fill(p.user_panel)
        .role("group")
        .label("Your account controls")
        .child(avatar_status(&s.me, 32.0, p, p.user_panel, true))
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
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
