//! Modal sheets: notifications.

use super::*;

pub(crate) fn notify_channel_picker_sheet(s: &App, p: Palette) -> El {
    let guild = s.active_guild().cloned().unwrap_or_default();
    let mut rows = col().w_full().gap(2.0);
    let mut count = 0;
    for channel in s.channels.get(&guild.id).cloned().unwrap_or_default() {
        if !matches!(
            channel.kind,
            ChannelKind::Text | ChannelKind::Voice | ChannelKind::Announcement | ChannelKind::Forum
        ) {
            continue;
        }
        if s.channel_notify.contains_key(&channel.id) || s.channel_suppress.contains(&channel.id) {
            continue;
        }
        count += 1;
        let channel_id = channel.id;
        let channel_icon = match channel.kind {
            ChannelKind::Voice => &icons::VOICE,
            ChannelKind::Forum => &icons::FORUM,
            ChannelKind::Announcement => &icons::MEGAPHONE,
            _ => &icons::HASH,
        };
        rows = rows.child(
            row()
                .key(format!("guild-notify-pick-{channel_id}"))
                .w_full()
                .h(40.0)
                .px(8.0)
                .radius(4.0)
                .gap(8.0)
                .center_y()
                .role("button")
                .label(format!("Add override for {}", channel.name))
                .hover(p.card_hover)
                .on_press(move |s: &mut App| {
                    s.add_guild_notification_override(guild.id, channel_id)
                })
                .child(icon(channel_icon, 16.0).color(p.text_muted))
                .child(
                    text(channel.name.clone())
                        .font(theme::SMALL)
                        .color(p.text_bright),
                ),
        );
    }
    if count == 0 {
        rows = rows.child(
            text("Every channel already has an override.")
                .font(theme::SMALL)
                .color(p.text_muted)
                .padding(8.0),
        );
    }
    col()
        .key("notify-channel-picker")
        .w(420.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(18.0)
        .gap(10.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::ServerNotifications)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text("Add Notification Override")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            col()
                .w_full()
                .max_h(420.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(rows),
        )
}

pub(crate) fn server_notifications_sheet(s: &App, p: Palette) -> El {
    let guild = s.active_guild().cloned().unwrap_or_default();
    let settings = s.guild_notification(guild.id);
    let switch_row = |label: &'static str,
                      detail: Option<&'static str>,
                      on: bool,
                      key: &'static str,
                      toggle: fn(&mut App),
                      p: Palette| {
        row()
            .key(key)
            .w_full()
            .min_h(if detail.is_some() { 46.0 } else { 36.0 })
            .py(2.0)
            .gap(12.0)
            .center_y()
            .role("switch")
            .label(label)
            .checked(on)
            .hover(p.card_hover)
            .on_press(toggle)
            .child(
                col()
                    .grow(1.0)
                    .min_w(0.0)
                    .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
                    .children(detail.map(|detail| {
                        text(detail)
                            .font(theme::TINY)
                            .color(p.text_muted)
                            .max_lines(2)
                    })),
            )
            .child(
                el().size(42.0, 24.0)
                    .radius(12.0)
                    .fill(if on { p.green } else { p.text_faint })
                    .child(
                        el().size(18.0, 18.0)
                            .radius(9.0)
                            .fill(Color::WHITE)
                            .ml(if on { 21.0 } else { 3.0 })
                            .mt(3.0)
                            .center()
                            .children(on.then(|| icon(&icons::CHECK, 12.0).color(p.green))),
                    ),
            )
    };
    let radio = |level: NotificationLevel, label: &'static str, key: &'static str| {
        let selected = settings.level == level;
        row()
            .key(key)
            .w_full()
            .h(36.0)
            .px(8.0)
            .radius(4.0)
            .gap(10.0)
            .center_y()
            .role("radio")
            .label(label)
            .checked(selected)
            .hover(p.card_hover)
            .on_press(move |s: &mut App| s.set_active_guild_notify_level(level))
            .child(
                el().size(18.0, 18.0)
                    .radius(9.0)
                    .border(
                        2.0,
                        if selected {
                            p.text_bright
                        } else {
                            p.text_muted
                        },
                    )
                    .center()
                    .children(
                        selected.then(|| el().size(8.0, 8.0).radius(4.0).fill(p.text_bright)),
                    ),
            )
            .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
    };

    let mut overrides = col().w_full().gap(2.0);
    let mut override_count = 0;
    for channel in s.channels.get(&guild.id).cloned().unwrap_or_default() {
        if !matches!(
            channel.kind,
            ChannelKind::Text | ChannelKind::Voice | ChannelKind::Announcement | ChannelKind::Forum
        ) {
            continue;
        }
        if !s.channel_notify.contains_key(&channel.id) && !s.channel_suppress.contains(&channel.id)
        {
            continue;
        }
        override_count += 1;
        let level = s
            .channel_notify
            .get(&channel.id)
            .copied()
            .unwrap_or(NotificationLevel::All);
        let channel_id = channel.id;
        let level_label = match level {
            NotificationLevel::All => "All Messages",
            NotificationLevel::Mentions => "Only @mentions",
            NotificationLevel::None => "Nothing",
        };
        let channel_icon = match channel.kind {
            ChannelKind::Voice => &icons::VOICE,
            ChannelKind::Forum => &icons::FORUM,
            ChannelKind::Announcement => &icons::MEGAPHONE,
            _ => &icons::HASH,
        };
        overrides = overrides.child(
            row()
                .key(format!("guild-notify-override-{channel_id}"))
                .w_full()
                .h(34.0)
                .px(6.0)
                .radius(4.0)
                .gap(8.0)
                .center_y()
                .hover(p.card_hover)
                .child(icon(channel_icon, 16.0).color(p.text_muted))
                .child(
                    text(channel.name.clone())
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .grow(1.0)
                        .truncate(),
                )
                .child(text(level_label).font(theme::TINY).color(p.text_muted))
                .child(
                    el().key(format!("guild-notify-remove-{channel_id}"))
                        .size(26.0, 26.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Remove override for {}", channel.name))
                        .hover(p.danger.alpha(0.2))
                        .on_press(move |s: &mut App| s.clear_channel_notify(channel_id))
                        .child(icon(&icons::CLOSE, 12.0).color(p.danger)),
                ),
        );
    }
    if override_count == 0 {
        overrides = overrides.child(
            row().w_full().h(52.0).center().child(
                text("Add a channel to override its default notification settings")
                    .font(theme::TINY)
                    .color(p.text_muted),
            ),
        );
    }

    col()
        .key("server-notifications")
        .w(480.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(16.0)
        .gap(4.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text("Notification Settings")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text(guild.name.clone())
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(switch_row(
            "Mute Server",
            Some("Muting a server prevents unread indicators and notifications from appearing unless you're mentioned."),
            settings.muted,
            "guild-notify-mute",
            App::toggle_active_guild_mute,
            p,
        ))
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            text("SERVER NOTIFICATION SETTINGS")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(radio(NotificationLevel::All, "All Messages", "guild-notify-all"))
        .child(radio(
            NotificationLevel::Mentions,
            "Only @mentions",
            "guild-notify-mentions",
        ))
        .child(radio(
            NotificationLevel::None,
            "Nothing",
            "guild-notify-nothing",
        ))
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(switch_row(
            "Suppress @everyone and @here",
            None,
            settings.suppress_everyone,
            "guild-notify-suppress-everyone",
            App::toggle_guild_suppress_everyone,
            p,
        ))
        .child(switch_row(
            "Suppress All Role @mentions",
            None,
            settings.suppress_roles,
            "guild-notify-suppress-roles",
            App::toggle_guild_suppress_roles,
            p,
        ))
        .child(switch_row(
            "Suppress Highlights",
            Some("Highlights provide occasional updates when your friends are chatting in busy servers, and more."),
            settings.suppress_highlights,
            "guild-notify-suppress-highlights",
            App::toggle_guild_suppress_highlights,
            p,
        ))
        .child(
            text("Learn more about Highlights")
                .font(theme::TINY)
                .color(p.text_link)
                .on_press(|s: &mut App| s.toast("Highlights are not available in this build")),
        )
        .child(switch_row(
            "Mute New Events",
            None,
            settings.mute_new_events,
            "guild-notify-mute-events",
            App::toggle_guild_mute_new_events,
            p,
        ))
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            text("NOTIFICATION OVERRIDES")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(
            text("Add a channel to override its default notification settings")
                .font(theme::TINY)
                .color(p.text_muted),
        )
        .child(
            row()
                .w_full()
                .h(24.0)
                .px(6.0)
                .gap(8.0)
                .center_y()
                .child(
                    text("CHANNEL OR CATEGORY")
                        .font(theme::TINY)
                        .color(p.text_faint)
                        .grow(1.0),
                )
                .child(text("OVERRIDE").font(theme::TINY).color(p.text_faint)),
        )
        .child(
            el().key("guild-notify-add")
                .w_full()
                .h(34.0)
                .px(10.0)
                .radius(4.0)
                .center_y()
                .role("button")
                .label("Select a channel or category")
                .fill(p.input)
                .hover(p.card_hover)
                .on_press(|s: &mut App| s.modal = Modal::NotifyChannelPicker)
                .child(
                    text("Select a channel or category...")
                        .font(theme::SMALL)
                        .color(p.text_muted)
                        .grow(1.0),
                )
                .child(icon(&icons::CHEVRON_DOWN, 14.0).color(p.text_muted)),
        )
        .child(
            col()
                .w_full()
                .max_h(140.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(overrides),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            row()
                .w_full()
                .justify_end()
                .child(
                    el().key("guild-notify-done")
                        .h(34.0)
                        .px(18.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Done")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(|s: &mut App| s.modal = Modal::None)
                        .child(text("Done").font(theme::SMALL_MED).color(Color::WHITE)),
                ),
        )
}

pub(crate) fn channel_notifications_sheet(s: &App, p: Palette, channel: Id) -> El {
    let name = s
        .channel(channel)
        .map(|channel| channel.name.clone())
        .unwrap_or_default();
    let current = s.channel_notify.get(&channel).copied().unwrap_or_default();
    let suppress = s.channel_suppress.contains(&channel);
    let options = [
        (
            NotificationLevel::All,
            "All Messages",
            "Get notified for every message in this channel.",
        ),
        (
            NotificationLevel::Mentions,
            "Only @mentions",
            "Only notify me when someone mentions me.",
        ),
        (
            NotificationLevel::None,
            "Nothing",
            "Mute this channel completely.",
        ),
    ];
    let mut rows = col().w_full().gap(2.0);
    for (level, label, description) in options {
        let selected = current == level;
        rows = rows.child(
            row()
                .key(format!(
                    "notify-{}",
                    match level {
                        NotificationLevel::All => "all",
                        NotificationLevel::Mentions => "mentions",
                        NotificationLevel::None => "none",
                    }
                ))
                .w_full()
                .min_h(48.0)
                .px(10.0)
                .py(6.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .role("radio")
                .label(label)
                .checked(selected)
                .fill(if selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.set_channel_notify(channel, level))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
                        .child(text(description).font(theme::TINY).color(p.text_muted)),
                )
                .children(selected.then(|| icon(&icons::CHECK, 18.0).color(p.green))),
        );
    }
    col()
        .key("channel-notifications")
        .w(400.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(18.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text("Notification Settings")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text(format!("#{name}"))
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(rows)
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            row()
                .key("notify-suppress")
                .w_full()
                .min_h(48.0)
                .px(10.0)
                .py(6.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .role("switch")
                .label("Suppress @everyone and @here")
                .checked(suppress)
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.set_channel_suppress(channel, !suppress))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text("Suppress @everyone and @here")
                                .font(theme::SMALL_MED)
                                .color(p.text_bright),
                        )
                        .child(
                            text("You will only be notified for direct mentions.")
                                .font(theme::TINY)
                                .color(p.text_muted),
                        ),
                )
                .child(
                    el().size(40.0, 22.0)
                        .radius(11.0)
                        .fill(if suppress { p.green } else { p.input })
                        .child(
                            el().size(18.0, 18.0)
                                .radius(9.0)
                                .fill(Color::WHITE)
                                .ml(if suppress { 19.0 } else { 2.0 })
                                .mt(2.0),
                        ),
                ),
        )
        .child(
            row().w_full().justify_end().child(
                el().key("notify-done")
                    .h(34.0)
                    .px(16.0)
                    .radius(4.0)
                    .role("button")
                    .label("Done")
                    .fill(p.accent)
                    .hover(p.accent_hover)
                    .center()
                    .on_press(|s: &mut App| s.modal = Modal::None)
                    .child(text("Done").font(theme::SMALL_MED).color(Color::WHITE)),
            ),
        )
}
