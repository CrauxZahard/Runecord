//! Modal sheets: discovery.

use super::*;

pub(crate) fn explore_sheet(s: &App, p: Palette) -> El {
    let mut list = col().w_full().gap(4.0);
    for guild in &s.public_guilds {
        let id = guild.id;
        let initials: String = guild
            .name
            .split_whitespace()
            .filter_map(|word| word.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase();
        list = list.child(
            row()
                .key(format!("public-{}", id))
                .w_full()
                .min_h(72.0)
                .px(12.0)
                .py(10.0)
                .radius(6.0)
                .gap(12.0)
                .center_y()
                .hover(p.card_hover)
                .child(
                    el().size(44.0, 44.0)
                        .radius(14.0)
                        .fill(theme::color(&format!("#{:06x}", guild.icon_color)))
                        .center()
                        .child(text(initials).font(theme::BODY_BOLD).color(Color::WHITE)),
                )
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(
                            text(guild.name.clone())
                                .font(theme::BODY_BOLD)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(
                            text(if guild.description.is_empty() {
                                "A public Rune server.".to_string()
                            } else {
                                guild.description.clone()
                            })
                            .font(theme::SMALL)
                            .color(p.text_muted)
                            .max_lines(2),
                        )
                        .child(
                            text(format!(
                                "{} member{}",
                                guild.member_count,
                                if guild.member_count == 1 { "" } else { "s" }
                            ))
                            .font(theme::TINY)
                            .color(p.text_faint),
                        ),
                )
                .child(
                    el().key(format!("join-{}", id))
                        .h(32.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Join {}", guild.name))
                        .fill(p.green)
                        .hover(p.green.alpha(0.85))
                        .on_press(move |s: &mut App| s.join_public_guild(id))
                        .child(
                            text("Join Server")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        ),
                ),
        );
    }
    if s.explore_loading {
        list = list.child(
            col().w_full().py(48.0).items_center().child(
                text("Loading public servers…")
                    .font(theme::SMALL)
                    .color(p.text_faint),
            ),
        );
    } else if s.public_guilds.is_empty() {
        list = list.child(
            col()
                .w_full()
                .py(48.0)
                .items_center()
                .gap(8.0)
                .child(icon(&icons::COMPASS, 48.0).color(p.text_faint))
                .child(
                    text("No public servers to join")
                        .font(theme::BODY)
                        .color(p.text_muted),
                )
                .child(
                    text("Create a server, or ask a friend for an invite.")
                        .font(theme::SMALL)
                        .color(p.text_faint),
                ),
        );
    }
    col()
        .key("explore")
        .w(sheet_w(680.0))
        .h(sheet_h(540.0))
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .px(20.0)
                .pt(20.0)
                .pb(12.0)
                .items_center()
                .child(
                    text("Explore Public Servers")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Find a community to join. Every server here is local to this machine.")
                .font(theme::SMALL)
                .color(p.text_muted)
                .px(20.0)
                .pb(12.0),
        )
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .px(8.0)
                .pb(12.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list),
        )
}

pub(crate) fn pins_sheet(s: &App, p: Palette) -> El {
    let channel = match s.screen {
        Screen::Guild => s.active_channel,
        Screen::Dms => s.active_dm,
    };
    let channel_name = s
        .channel(channel)
        .map(|channel| channel.name.clone())
        .unwrap_or_default();
    let mut list = col().w_full().gap(4.0);
    for message in &s.pinned {
        let id = message.id;
        let author = s.user(message.author_id).cloned().unwrap_or_default();
        let author_name = author.display_name.clone();
        let body = message.preview();
        let time = time_label(message.timestamp);
        list = list.child(
            col()
                .key(format!("pinned-{}", id))
                .w_full()
                .radius(6.0)
                .padding(10.0)
                .gap(6.0)
                .fill(p.card)
                .child(
                    row()
                        .gap(8.0)
                        .center_y()
                        .child(avatar(&author, 24.0, p))
                        .child(
                            text(author_name)
                                .font(theme::SMALL_MED)
                                .color(p.text_bright),
                        )
                        .child(
                            text(format!("#{channel_name} · {time}"))
                                .font(theme::TINY)
                                .color(p.text_faint)
                                .grow(1.0)
                                .truncate(),
                        ),
                )
                .child(text(body).font(theme::SMALL).color(p.text).max_lines(3))
                .child(
                    row()
                        .w_full()
                        .gap(6.0)
                        .justify_end()
                        .child(
                            el().key(format!("pin-jump-{}", id))
                                .h(26.0)
                                .px(10.0)
                                .radius(4.0)
                                .center()
                                .role("button")
                                .label("Jump to message")
                                .fill(p.rail)
                                .hover(p.rail_hover)
                                .on_press_tx(move |s: &mut App, tx: &mut rune::Tx| {
                                    s.jump_to_pinned(id, channel);
                                    tx.scroll_into_view(format!("msg-{}", id));
                                })
                                .child(text("Jump").font(theme::LABEL).color(p.text_bright)),
                        )
                        .child(
                            el().key(format!("pin-remove-{}", id))
                                .h(26.0)
                                .px(10.0)
                                .radius(4.0)
                                .center()
                                .role("button")
                                .label("Unpin message")
                                .fill(p.rail)
                                .hover(p.danger.alpha(0.2))
                                .on_press(move |s: &mut App| s.toggle_pin(id))
                                .child(text("Unpin").font(theme::LABEL).color(p.danger)),
                        ),
                ),
        );
    }
    if s.pinned.is_empty() {
        list = list.child(
            col()
                .w_full()
                .py(48.0)
                .items_center()
                .gap(8.0)
                .child(icon(&icons::PIN, 40.0).color(p.text_faint))
                .child(
                    text("No pinned messages in this channel yet.")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                )
                .child(
                    text("Right-click a message and choose Pin Message.")
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        );
    }
    col()
        .key("pins")
        .w(sheet_w(540.0))
        .h(sheet_h(560.0))
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .px(20.0)
                .pt(20.0)
                .pb(12.0)
                .items_center()
                .child(
                    text("Pinned Messages")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text(format!("#{channel_name}"))
                .font(theme::SMALL)
                .color(p.text_muted)
                .px(20.0)
                .pb(12.0),
        )
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .px(12.0)
                .pb(12.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list),
        )
}

pub(crate) fn privacy_sheet(s: &App, p: Palette) -> El {
    let mut dm_rows = col().w_full().gap(2.0);
    let mut friend_rows = col().w_full().gap(2.0);
    let options = [
        (
            PrivacyLevel::Everyone,
            "everyone",
            "Everyone",
            "Anyone can send you a direct message.",
        ),
        (
            PrivacyLevel::ServerMembers,
            "server-members",
            "Server Members",
            "Only people who share a server with you.",
        ),
        (
            PrivacyLevel::Nobody,
            "nobody",
            "Nobody",
            "No one can start a new conversation.",
        ),
    ];
    for (level, key, label, description) in options {
        let dm_selected = s.me.dm_privacy == level;
        dm_rows = dm_rows.child(
            row()
                .key(format!("privacy-dm-{key}"))
                .w_full()
                .min_h(48.0)
                .px(10.0)
                .py(6.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .role("radio")
                .label(label)
                .checked(dm_selected)
                .fill(if dm_selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.set_privacy(level, s.me.friend_privacy))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
                        .child(text(description).font(theme::TINY).color(p.text_muted)),
                )
                .children(dm_selected.then(|| icon(&icons::CHECK, 18.0).color(p.green))),
        );

        let friend_selected = s.me.friend_privacy == level;
        let friend_description = match level {
            PrivacyLevel::Everyone => "Anyone can send you a friend request.",
            PrivacyLevel::ServerMembers => "Only people who share a server with you.",
            PrivacyLevel::Nobody => "No one can send you a friend request.",
        };
        friend_rows = friend_rows.child(
            row()
                .key(format!("privacy-friend-{key}"))
                .w_full()
                .min_h(48.0)
                .px(10.0)
                .py(6.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .role("radio")
                .label(label)
                .checked(friend_selected)
                .fill(if friend_selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.set_privacy(s.me.dm_privacy, level))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
                        .child(
                            text(friend_description)
                                .font(theme::TINY)
                                .color(p.text_muted),
                        ),
                )
                .children(friend_selected.then(|| icon(&icons::CHECK, 18.0).color(p.green))),
        );
    }

    let mut body = col()
        .w_full()
        .gap(10.0)
        .child(
            text("DIRECT MESSAGES")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(dm_rows)
        .child(
            text("FRIEND REQUESTS")
                .font(theme::SECTION)
                .color(p.text_muted)
                .pt(10.0),
        )
        .child(friend_rows);

    if s.guild_can(perm::MANAGE_GUILD) {
        let discoverable = s
            .active_guild()
            .map(|guild| guild.discoverable)
            .unwrap_or(false);
        body = body
            .child(
                text("SERVER DISCOVERY")
                    .font(theme::SECTION)
                    .color(p.text_muted)
                    .pt(10.0),
            )
            .child(
                row()
                    .key("privacy-discovery")
                    .w_full()
                    .min_h(48.0)
                    .px(10.0)
                    .py(6.0)
                    .radius(4.0)
                    .gap(10.0)
                    .center_y()
                    .role("switch")
                    .label("Show server in Explore")
                    .checked(discoverable)
                    .hover(p.card_hover)
                    .on_press(|s: &mut App| s.toggle_guild_discovery())
                    .child(
                        col()
                            .grow(1.0)
                            .min_w(0.0)
                            .child(
                                text("Show in Explore Public Servers")
                                    .font(theme::SMALL_MED)
                                    .color(p.text_bright),
                            )
                            .child(
                                text("People who are not members can find and join this server.")
                                    .font(theme::TINY)
                                    .color(p.text_muted),
                            ),
                    )
                    .child(
                        el().size(40.0, 22.0)
                            .radius(11.0)
                            .fill(if discoverable { p.green } else { p.input })
                            .child(
                                el().size(18.0, 18.0)
                                    .radius(9.0)
                                    .fill(Color::WHITE)
                                    .ml(if discoverable { 19.0 } else { 2.0 })
                                    .mt(2.0),
                            ),
                    ),
            );
    }

    col()
        .key("privacy")
        .w(sheet_w(520.0))
        .radius(8.0)
        .fill(p.modal)
        .padding(18.0)
        .gap(10.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text("Privacy Settings")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Choose who can reach you and whether this server appears in Explore.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            col()
                .w_full()
                .max_h(560.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(body),
        )
        .child(
            row().w_full().justify_end().pt(6.0).child(
                el().key("privacy-done")
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
