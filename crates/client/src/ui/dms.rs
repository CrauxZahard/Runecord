//! Direct messages surface.

use super::*;

pub(crate) fn dm_title(dm: &DmChannel) -> String {
    if dm.channel.kind == ChannelKind::GroupDm {
        if !dm.channel.name.trim().is_empty() {
            return dm.channel.name.clone();
        }
        let names: Vec<String> = dm
            .recipients
            .iter()
            .map(|user| user.display_name.clone())
            .collect();
        if names.is_empty() {
            "Group DM".to_string()
        } else {
            names.join(", ")
        }
    } else {
        dm.recipients
            .first()
            .map(|user| user.display_name.clone())
            .unwrap_or_default()
    }
}

pub(crate) fn dm_avatar(
    s: &App,
    dm: &DmChannel,
    size: f32,
    p: Palette,
    ring: Color,
    show_status: bool,
) -> El {
    if dm.channel.kind != ChannelKind::GroupDm || dm.recipients.len() < 2 {
        let user = dm.recipients.first().cloned().unwrap_or_default();
        return avatar_status(&user, size, p, ring, show_status);
    }
    if let Some(pixels) = s
        .images
        .get(&dm.channel.icon_url)
        .filter(|_| !dm.channel.icon_url.is_empty())
    {
        return el()
            .size(size, size)
            .radius(size / 2.0)
            .center()
            .clip_rounded(size / 2.0)
            .pixels_fit(pixels, rune::SurfaceFit::Fill);
    }
    let inner = (size * 0.72).max(12.0);
    let mut stack = row().size(size, size).center();
    for (index, user) in dm.recipients.iter().take(3).enumerate() {
        let mut avatar = avatar_status(user, inner, p, ring, show_status);
        if index > 0 {
            avatar = avatar.ml(-inner * 0.28);
        }
        stack = stack.child(avatar);
    }
    stack
}

pub(crate) fn dm_sidebar(s: &App, p: Palette) -> El {
    let mut list = col().key("dm-list").w_full().px(10.0).gap(1.0);
    for dm in &s.dms {
        let id = dm.channel.id;
        let selected = !s.dm_home && s.active_dm == id;
        let unread = s.unread.get(&id).copied().unwrap_or(0);
        let title = dm_title(dm);
        list = list.child(
            row()
                .key(format!("dm-{}", id))
                .w_full()
                .h(44.0)
                .px(10.0)
                .radius(5.0)
                .gap(10.0)
                .center_y()
                .fill(if selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.sidebar_hover)
                .role("button")
                .label(if unread > 0 {
                    format!("{title}, {unread} unread")
                } else {
                    title.clone()
                })
                .motion(motion::SNAPPY)
                .on_press(move |s: &mut App| {
                    s.add_friend_open = false;
                    s.open_dm(id);
                })
                .child(dm_avatar(s, dm, 32.0, p, p.sidebar, true))
                .child(
                    text(title)
                        .font(theme::BODY_MED)
                        .color(if unread > 0 {
                            p.text_bright
                        } else {
                            p.text_muted
                        })
                        .grow(1.0)
                        .truncate(),
                ),
        );
    }
    if s.dms.is_empty() {
        list = list.child(
            text("No direct messages yet")
                .font(theme::SMALL)
                .color(p.text_faint)
                .padding(8.0),
        );
    }

    col()
        .key("dm-sidebar")
        .w(240.0)
        .h_full()
        .fill(p.sidebar)
        .child(
            row()
                .key("dm-header")
                .w_full()
                .h(48.0)
                .px(16.0)
                .center_y()
                .gap(8.0)
                .fill(p.sidebar)
                .window_drag()
                .child(
                    text("Direct Messages")
                        .font(theme::SMALL_MED)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(
                    el().key("new-dm")
                        .size(24.0, 24.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Create a DM")
                        .hover(p.sidebar_hover)
                        .help("Create DM")
                        .on_press(|s: &mut App| s.toast("Pick a friend to start a DM"))
                        .child(icon(&icons::PLUS, 18.0).color(p.text_muted)),
                ),
        )
        .child(
            row().key("dm-search").w_full().px(8.0).pb(8.0).child(
                row()
                    .w_full()
                    .h(28.0)
                    .radius(4.0)
                    .fill(p.rail)
                    .px(6.0)
                    .gap(4.0)
                    .center_y()
                    .child(icon(&icons::SEARCH, 14.0).color(p.text_faint))
                    .child(
                        text("Find or start a conversation")
                            .font(TextStyle::new(13.0, 400.0))
                            .color(p.text_faint),
                    ),
            ),
        )
        .child(
            col()
                .key("dm-nav")
                .w_full()
                .px(10.0)
                .gap(1.0)
                .child({
                    let selected = s.dm_home && !s.add_friend_open;
                    row()
                        .key("dm-home")
                        .w_full()
                        .h(34.0)
                        .px(10.0)
                        .radius(5.0)
                        .gap(10.0)
                        .center_y()
                        .fill(if selected {
                            p.selected
                        } else {
                            Color::TRANSPARENT
                        })
                        .hover(p.sidebar_hover)
                        .role("button")
                        .label("Friends")
                        .on_press(|s: &mut App| {
                            s.dm_home = true;
                            s.add_friend_open = false;
                        })
                        .child(icon(&icons::HOME, 20.0).color(p.interactive))
                        .child(
                            text("Friends")
                                .font(theme::BODY)
                                .color(p.text_bright)
                                .grow(1.0),
                        )
                })
                .child(
                    row()
                        .key("dm-premium")
                        .w_full()
                        .h(34.0)
                        .px(10.0)
                        .radius(5.0)
                        .gap(10.0)
                        .center_y()
                        .hover(p.sidebar_hover)
                        .role("button")
                        .label("All features unlocked")
                        .on_press(|s: &mut App| {
                            s.toast("Every premium perk is unlocked in this build")
                        })
                        .child(icon(&icons::SPARKLES, 20.0).color(p.text_muted))
                        .child(
                            text("All Unlocked")
                                .font(theme::BODY)
                                .color(p.text_muted)
                                .grow(1.0),
                        ),
                ),
        )
        .child(el().w_full().h(1.0).fill(p.divider).my(4.0))
        .child(group_label("DIRECT MESSAGES", p).padding(8.0))
        .child(
            col()
                .key("dm-scroll")
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(col().w_full().child(list)),
        )
        .children(voice_connected_panel(s, p))
        .child(user_panel(s, p))
}

pub(crate) fn dm_main(s: &App, cx: &mut Cx<App>, p: Palette) -> El {
    if s.dm_home {
        row()
            .grow(1.0)
            .min_w(0.0)
            .h_full()
            .child(friends_page(s, p, cx.chrome_insets().trailing))
            .child(active_now_panel(s, p))
    } else {
        chat(s, cx, p)
    }
}

fn active_now_panel(s: &App, p: Palette) -> El {
    let mut active = col().w_full().gap(8.0).px(16.0).pt(14.0);
    for (user_id, state) in &s.voice {
        if state.channel_id.is_none() {
            continue;
        }
        let user = s.user(*user_id).cloned().unwrap_or_default();
        active = active.child(
            row()
                .w_full()
                .h(44.0)
                .gap(10.0)
                .center_y()
                .child(avatar_status(&user, 32.0, p, p.chat, true))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text(user.display_name.clone())
                                .font(theme::SMALL_MED)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(text("In voice").font(theme::TINY).color(p.text_muted)),
                ),
        );
    }
    col()
        .key("active-now")
        .w(300.0)
        .h_full()
        .fill(p.chat)
        .border_sides([1.0, 0.0, 0.0, 0.0], p.divider_soft)
        .child(
            text("Active Now")
                .font(theme::NAME)
                .color(p.text_bright)
                .padding(16.0)
                .pb(4.0),
        )
        .child(active)
        .child(
            col()
                .w_full()
                .py(40.0)
                .px(24.0)
                .items_center()
                .gap(8.0)
                .child(
                    text("It's quiet for nowâ€¦")
                        .font(theme::NAME)
                        .color(p.text_bright),
                )
                .child(
                    text("When a friend starts an activityâ€”like playing a game or hanging out on voiceâ€”we'll show it here!")
                        .font(theme::TINY)
                        .color(p.text_muted)
                        .max_lines(4),
                ),
        )
}

pub(crate) fn friends_page(s: &App, p: Palette, insets_trailing: f32) -> El {
    let tabs = [(0u8, "Online"), (1, "All"), (2, "Pending"), (3, "Blocked")];
    let make_tabs = || {
        let mut tab_row = row().gap(8.0).h_full().center_y();
        for (i, label) in tabs {
            let selected = !s.add_friend_open && s.friend_tab == i;
            tab_row = tab_row.child(
                el().key(format!("friend-tab-{i}"))
                    .h(32.0)
                    .px(8.0)
                    .radius(4.0)
                    .center()
                    .fill(if selected {
                        p.selected
                    } else {
                        Color::TRANSPARENT
                    })
                    .hover(p.sidebar_hover)
                    .on_press(move |s: &mut App| {
                        s.friend_tab = i;
                        s.add_friend_open = false;
                    })
                    .child(text(label).font(theme::SMALL_MED).color(if selected {
                        p.text_bright
                    } else {
                        p.text_muted
                    })),
            );
        }
        tab_row
    };

    let title_row = row()
        .gap(8.0)
        .center_y()
        .child(icon(&icons::USERS, 24.0).color(p.text_faint))
        .child(text("Friends").font(theme::NAME).color(p.text_bright));
    let add_friend = el()
        .key("add-friend-open")
        .h(32.0)
        .px(12.0)
        .radius(4.0)
        .center()
        .role("button")
        .label("Add Friend")
        .fill(p.green)
        .hover(p.green.alpha(0.85))
        .on_press(|s: &mut App| s.add_friend_open = true)
        .child(
            text("Add Friend")
                .font(theme::SMALL_MED)
                .color(Color::WHITE),
        );
    let group_dm = el()
        .key("new-group-dm")
        .size(30.0, 30.0)
        .radius(4.0)
        .center()
        .role("button")
        .label("New group DM")
        .help("New Group DM")
        .hover(p.sidebar_hover)
        .on_press(|s: &mut App| s.open_group_dm())
        .child(icon(&icons::USERS, 20.0).color(p.text_muted));
    let inbox = el()
        .key("friends-inbox")
        .size(30.0, 30.0)
        .radius(4.0)
        .center()
        .role("button")
        .label("Inbox")
        .help("Inbox")
        .hover(p.sidebar_hover)
        .on_press(|s: &mut App| s.modal = Modal::Search)
        .child(icon(&icons::INBOX, 20.0).color(p.text_muted));
    let help = el()
        .key("friends-help")
        .size(30.0, 30.0)
        .radius(4.0)
        .center()
        .role("button")
        .label("Help")
        .help("Help")
        .hover(p.sidebar_hover)
        .on_press(|s: &mut App| s.toast("Runecord help is local"))
        .child(icon(&icons::HELP, 20.0).color(p.text_muted));

    let header = row()
        .key("friends-header")
        .w_full()
        .h(48.0)
        .pl(16.0)
        .pr(8.0 + insets_trailing)
        .gap(16.0)
        .center_y()
        .window_drag()
        .border_sides([0.0, 0.0, 1.0, 0.0], p.divider_soft)
        .child(title_row)
        .child(el().w(1.0).h(24.0).fill(p.divider))
        .child(make_tabs())
        .child(add_friend)
        .child(el().grow(1.0))
        .child(group_dm)
        .child(inbox)
        .child(help);

    let body = if s.add_friend_open {
        add_friend_panel(s, p)
    } else {
        friend_list_panel(s, p, s.friend_tab)
    };

    col()
        .key("friends")
        .grow(1.0)
        .min_w(0.0)
        .h_full()
        .fill(p.chat)
        .child(header)
        .child(
            row()
                .key("friends-search")
                .w_full()
                .px(20.0)
                .pt(12.0)
                .child(
                    text_input(&s.friend_filter, theme::SMALL)
                        .key("friend-search-input")
                        .w_full()
                        .h(34.0)
                        .radius(4.0)
                        .fill(p.rail)
                        .color(p.text)
                        .role("textbox")
                        .label("Search friends")
                        .placeholder("Search", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.friend_filter),
                ),
        )
        .child(
            col()
                .key("friends-scroll")
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(body),
        )
}

pub(crate) fn add_friend_panel(s: &App, p: Palette) -> El {
    col()
        .key("add-friend")
        .w_full()
        .px(32.0)
        .pt(24.0)
        .gap(12.0)
        .child(text("ADD FRIEND").font(theme::TITLE).color(p.text_bright))
        .child(
            text("You can add friends with their Runecord username. It's case sensitive.")
                .wrap()
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            row()
                .key("add-friend-row")
                .w_full()
                .h(48.0)
                .radius(8.0)
                .fill(p.rail)
                .pl(12.0)
                .pr(6.0)
                .gap(8.0)
                .center_y()
                .child(
                    text_input(&s.search, theme::BODY)
                        .key("add-friend-input")
                        .grow(1.0)
                        .min_w(0.0)
                        .h_full()
                        .color(p.text)
                        .placeholder("You can add friends with their username.", p.text_faint)
                        .padding(0.0)
                        .pt(13.0)
                        .on_edit(|s: &mut App| &mut s.search)
                        .on_submit(|s: &mut App, _: &mut rune::Tx| s.add_friend()),
                )
                .child(
                    el().key("send-request")
                        .h(32.0)
                        .px(14.0)
                        .radius(4.0)
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .center()
                        .on_press(|s: &mut App| s.add_friend())
                        .child(
                            text("Send Friend Request")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        ),
                ),
        )
        .child(el().w_full().h(1.0).fill(p.divider).my(16.0))
        .child(text("PENDING").font(theme::SECTION).color(p.text_muted))
        .child({
            let mut pending = col().w_full();
            let mut shown = 0;
            for rel in s.relationships.iter().filter(|rel| {
                matches!(
                    rel.kind,
                    RelationshipKind::Incoming | RelationshipKind::Outgoing
                )
            }) {
                shown += 1;
                pending = pending.child(relationship_row(s, rel, p));
            }
            if shown == 0 {
                pending = pending.child(
                    text("No pending requests.")
                        .font(theme::SMALL)
                        .color(p.text_faint)
                        .py(8.0),
                );
            }
            pending
        })
}

pub(crate) fn friend_list_panel(s: &App, p: Palette, tab: u8) -> El {
    let mut list = col().key("friend-list").w_full().px(16.0).pt(16.0).gap(0.0);
    let mut shown = 0;
    let query = s.friend_filter.text.trim().to_lowercase();
    for rel in &s.relationships {
        if !query.is_empty() && !rel.user.display_name.to_lowercase().contains(&query) {
            continue;
        }
        let matches = match tab {
            0 => {
                rel.kind == RelationshipKind::Friend
                    && s.presence
                        .get(&rel.user.id)
                        .copied()
                        .unwrap_or(rel.user.status)
                        != Presence::Offline
            }
            1 => rel.kind == RelationshipKind::Friend,
            2 => matches!(
                rel.kind,
                RelationshipKind::Incoming | RelationshipKind::Outgoing
            ),
            3 => rel.kind == RelationshipKind::Blocked,
            _ => true,
        };
        if !matches {
            continue;
        }
        shown += 1;
        list = list.child(relationship_row(s, rel, p));
    }
    if shown == 0 {
        list = list.child(
            col()
                .w_full()
                .py(40.0)
                .items_center()
                .gap(8.0)
                .child(icon(&icons::USERS, 48.0).color(p.text_faint))
                .child(
                    text(match tab {
                        2 => "No pending friend requests.".to_string(),
                        3 => "No blocked users.".to_string(),
                        _ => "Nobody here yet.".to_string(),
                    })
                    .font(theme::BODY)
                    .color(p.text_muted),
                ),
        );
    }
    col()
        .w_full()
        .px(20.0)
        .pt(18.0)
        .child(
            text(match tab {
                0 => format!("ONLINE â€” {shown}"),
                1 => format!("ALL FRIENDS â€” {shown}"),
                2 => format!("PENDING â€” {shown}"),
                3 => format!("BLOCKED â€” {shown}"),
                _ => format!("FRIENDS â€” {shown}"),
            })
            .font(theme::SECTION)
            .color(p.text_muted)
            .pb(8.0),
        )
        .child(list)
}

pub(crate) fn relationship_row(s: &App, rel: &runecord_protocol::Relationship, p: Palette) -> El {
    let status = s
        .presence
        .get(&rel.user.id)
        .copied()
        .unwrap_or(rel.user.status);
    let status_label = match rel.kind {
        RelationshipKind::Incoming => "Incoming friend request".to_string(),
        RelationshipKind::Outgoing => "Outgoing friend request".to_string(),
        RelationshipKind::Blocked => "Blocked".to_string(),
        RelationshipKind::Friend => rel.user.custom_status.clone(),
    };
    let user_id = rel.user.id;
    row()
        .key(format!("friend-{}", user_id))
        .w_full()
        .h(62.0)
        .px(8.0)
        .radius(4.0)
        .gap(12.0)
        .center_y()
        .hover(p.chat_hover)
        .child(avatar_status(&rel.user, 32.0, p, p.chat, true))
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .child(
                    text(rel.user.display_name.clone())
                        .font(theme::NAME)
                        .color(p.text_bright)
                        .truncate(),
                )
                .child(
                    text(if status_label.is_empty() {
                        status.label().to_string()
                    } else {
                        status_label
                    })
                    .font(theme::SMALL)
                    .color(p.text_muted)
                    .truncate(),
                ),
        )
        .children((rel.kind == RelationshipKind::Incoming).then(|| {
            el().key(format!("friend-accept-{user_id}"))
                .size(36.0, 36.0)
                .radius(18.0)
                .center()
                .fill(p.rail)
                .hover(p.green.alpha(0.2))
                .role("button")
                .label(format!("Accept {}", rel.user.display_name))
                .help("Accept friend request")
                .on_press(move |s: &mut App| s.accept_friend(user_id))
                .child(icon(&icons::CHECK, 18.0).color(p.green))
        }))
        .children(
            matches!(
                rel.kind,
                RelationshipKind::Incoming | RelationshipKind::Outgoing
            )
            .then(|| {
                el().key(format!("friend-decline-{user_id}"))
                    .size(36.0, 36.0)
                    .radius(18.0)
                    .center()
                    .fill(p.rail)
                    .hover(p.danger.alpha(0.2))
                    .role("button")
                    .label(if rel.kind == RelationshipKind::Incoming {
                        format!("Ignore {}", rel.user.display_name)
                    } else {
                        format!("Cancel request to {}", rel.user.display_name)
                    })
                    .help("Remove friend request")
                    .on_press(move |s: &mut App| s.decline_friend(user_id))
                    .child(icon(&icons::CLOSE, 18.0).color(p.danger))
            }),
        )
        .children((rel.kind == RelationshipKind::Friend).then(|| {
            el().key("dm")
                .size(34.0, 34.0)
                .radius(4.0)
                .center()
                .hover(p.sidebar_hover)
                .role("button")
                .label("Message")
                .help("Message")
                .on_press(move |s: &mut App| s.start_dm(user_id))
                .child(icon(&icons::CHAT, 20.0).color(p.interactive))
        }))
        .child(
            el().key("more")
                .size(34.0, 34.0)
                .radius(4.0)
                .center()
                .hover(p.sidebar_hover)
                .child(icon(&icons::MORE, 20.0).color(p.interactive)),
        )
}

// ---------------------------------------------------------------------------
// Shared widgets
// ---------------------------------------------------------------------------
