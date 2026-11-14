//! Modal sheets: social.

use super::*;

pub(crate) fn add_friend_sheet(s: &App, p: Palette) -> El {
    modal_frame(
        "add-friend",
        "Add Friend",
        "Add friends by their username. This is a local account on your machine.",
        p,
        col()
            .w_full()
            .gap(8.0)
            .child(text("USERNAME").font(theme::SECTION).color(p.text_muted))
            .child(
                text_input(&s.search, theme::BODY)
                    .key("friend-username")
                    .w_full()
                    .h(40.0)
                    .radius(4.0)
                    .fill(p.rail)
                    .color(p.text)
                    .role("textbox")
                    .label("Username")
                    .placeholder("maya", p.text_faint)
                    .padding(10.0)
                    .on_edit(|s: &mut App| &mut s.search)
                    .on_submit(|s: &mut App, _: &mut rune::Tx| s.add_friend()),
            )
            .child(
                text("Try maya, leo, ana or sam.")
                    .font(theme::TINY)
                    .color(p.text_faint),
            ),
        ("Send Friend Request", |s: &mut App| s.add_friend()),
        p,
    )
}

pub(crate) fn group_dm_sheet(s: &App, p: Palette) -> El {
    let mut rows = col().w_full().gap(2.0);
    let mut friends = 0;
    for rel in &s.relationships {
        if rel.kind != RelationshipKind::Friend {
            continue;
        }
        friends += 1;
        let id = rel.user.id;
        let selected = s.group_dm_selected.contains(&id);
        rows = rows.child(
            row()
                .key(format!("group-dm-user-{id}"))
                .w_full()
                .h(44.0)
                .px(8.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .role("checkbox")
                .label(format!("Add {}", rel.user.display_name))
                .checked(selected)
                .fill(if selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.toggle_group_dm(id))
                .child(
                    el().size(18.0, 18.0)
                        .radius(4.0)
                        .fill(if selected { p.green } else { p.input })
                        .border(1.0, p.divider)
                        .center()
                        .children(selected.then(|| icon(&icons::CHECK, 12.0).color(Color::WHITE))),
                )
                .child(avatar_status(&rel.user, 28.0, p, p.card, false))
                .child(
                    text(rel.user.display_name.clone())
                        .font(theme::SMALL_MED)
                        .color(p.text_bright)
                        .grow(1.0)
                        .truncate(),
                ),
        );
    }
    if friends == 0 {
        rows = rows.child(
            text("Add a friend first.")
                .font(theme::SMALL)
                .color(p.text_muted)
                .padding(8.0),
        );
    }
    let count = s.group_dm_selected.len();
    col()
        .key("group-dm")
        .w(480.0)
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
                    text("Create Group DM")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Pick at least two friends. You can name the group now or leave it as the member list.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            text("GROUP NAME (OPTIONAL)")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(
            text_input(&s.group_dm_name, theme::SMALL)
                .key("group-dm-name")
                .w_full()
                .h(36.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Group name")
                .placeholder("Weekend crew", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.group_dm_name),
        )
        .child(
            col()
                .w_full()
                .max_h(300.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(rows),
        )
        .child(
            row()
                .w_full()
                .gap(10.0)
                .center_y()
                .child(
                    text(format!("{count} selected"))
                        .font(theme::TINY)
                        .color(p.text_muted)
                        .grow(1.0),
                )
                .child(
                    el().key("group-dm-cancel")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Cancel")
                        .hover(p.card_hover)
                        .on_press(|s: &mut App| s.modal = Modal::None)
                        .child(text("Cancel").font(theme::SMALL).color(p.text_muted)),
                )
                .child(
                    el().key("group-dm-create")
                        .h(34.0)
                        .px(16.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Create group DM")
                        .fill(if count >= 2 { p.green } else { p.rail })
                        .hover(if count >= 2 { p.green.alpha(0.85) } else { p.rail })
                        .on_press(|s: &mut App| s.create_group_dm())
                        .child(
                            text("Create")
                                .font(theme::SMALL_MED)
                                .color(if count >= 2 { Color::WHITE } else { p.text_faint }),
                        ),
                ),
        )
}

pub(crate) fn invite_join_sheet(s: &App, p: Palette) -> El {
    col()
        .key("invite-join")
        .w(440.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(20.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text("Join a Server")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Paste an invite code or link. If you are banned, you can appeal from here.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            text_input(&s.join_code, theme::BODY)
                .key("join-code")
                .w_full()
                .h(40.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Invite code or link")
                .placeholder("https://…/invite/abc123 or abc123", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.join_code)
                .on_submit(|s: &mut App, _: &mut rune::Tx| s.join_invite_now()),
        )
        .child(
            row().w_full().justify_end().child(
                el().key("join-submit")
                    .h(36.0)
                    .px(16.0)
                    .radius(4.0)
                    .center()
                    .role("button")
                    .label("Join server")
                    .fill(p.accent)
                    .hover(p.accent_hover)
                    .on_press(|s: &mut App| s.join_invite_now())
                    .child(
                        text("Join Server")
                            .font(theme::SMALL_MED)
                            .color(Color::WHITE),
                    ),
            ),
        )
}

pub(crate) fn invite_sheet(s: &App, p: Palette) -> El {
    let code = s.invite_code.clone();
    let url = code
        .as_ref()
        .map(|c| format!("{}/invite/{c}", s.server_base));
    let body = col()
        .w_full()
        .gap(12.0)
        .child(
            text("Send a server invite link to a friend.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            row()
                .w_full()
                .h(44.0)
                .radius(4.0)
                .fill(p.rail)
                .px(12.0)
                .gap(10.0)
                .center_y()
                .child(
                    text(url.clone().unwrap_or_else(|| "Generating invite…".into()))
                        .font(theme::MONO)
                        .color(p.text)
                        .grow(1.0)
                        .min_w(0.0)
                        .truncate(),
                )
                .child(
                    el().h(30.0)
                        .px(12.0)
                        .radius(4.0)
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .center()
                        .on_press_tx(move |_: &mut App, tx: &mut rune::Tx| {
                            if let Some(url) = &url {
                                tx.copy(url.clone());
                            }
                        })
                        .child(text("Copy").font(theme::SMALL_MED).color(Color::WHITE)),
                ),
        )
        .child(
            text("Invite codes can be joined from any local Runecord account.")
                .font(theme::TINY)
                .color(p.text_faint)
                .wrap(),
        );
    modal_frame_done(
        "invite",
        "Invite friends",
        "Let’s get some people in here.",
        p,
        body,
        ("Done", |s: &mut App| s.modal = Modal::None),
        p,
    )
}

pub(crate) fn dm_settings_sheet(s: &App, p: Palette, channel: Id) -> El {
    let Some(dm) = s.dm_for(channel).cloned() else {
        return el();
    };
    let group = dm.channel.kind == ChannelKind::GroupDm;
    let owner = dm.channel.owner_id == Some(s.me.id);
    let icon_url = dm.channel.icon_url.clone();
    let total = dm.recipients.len() + 1;

    let icon_preview =
        if let Some(pixels) = s.images.get(&icon_url).filter(|_| !icon_url.is_empty()) {
            el().size(80.0, 80.0)
                .radius(40.0)
                .clip_rounded(40.0)
                .pixels_fit(pixels, rune::SurfaceFit::Fill)
        } else {
            dm_avatar(s, &dm, 80.0, p, p.modal, false)
        };

    let mut icon_actions = col().gap(6.0).justify_center();
    if owner {
        icon_actions = icon_actions.child(
            el().key("dm-icon-upload")
                .w(150.0)
                .h(32.0)
                .px(12.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Change group icon")
                .fill(p.accent)
                .hover(p.accent_hover)
                .on_press_tx(move |_s: &mut App, tx: &mut rune::Tx| {
                    let options = rune::FileDialogOptions {
                        title: "Upload group icon".to_string(),
                        directory: None,
                        file_name: None,
                        filters: Vec::new(),
                    }
                    .filter("Images", &["png", "jpg", "jpeg", "gif", "webp"]);
                    tx.open_file(options, move |s: &mut App, result, _| {
                        if let Ok(Some(path)) = result {
                            s.begin_dm_icon_crop(channel, path);
                        }
                    });
                })
                .child(text("Change Icon").font(theme::LABEL).color(Color::WHITE)),
        );
        if !icon_url.is_empty() {
            icon_actions = icon_actions.child(
                el().key("dm-icon-remove")
                    .w(150.0)
                    .h(30.0)
                    .px(10.0)
                    .radius(4.0)
                    .center()
                    .role("button")
                    .label("Remove group icon")
                    .hover(p.danger.alpha(0.2))
                    .on_press(move |s: &mut App| s.remove_dm_icon(channel))
                    .child(text("Remove").font(theme::LABEL).color(p.danger)),
            );
        }
    }

    let mut members = col().w_full().gap(2.0);
    members = members.child(
        row()
            .w_full()
            .h(40.0)
            .px(8.0)
            .gap(10.0)
            .center_y()
            .child(avatar_status(&s.me, 28.0, p, p.modal, true))
            .child(
                text(format!("{} (you)", s.me.display_name))
                    .font(theme::SMALL)
                    .color(p.text_bright)
                    .grow(1.0),
            )
            .children(
                (owner && group).then(|| text("Owner").font(theme::TINY).color(p.text_faint)),
            ),
    );
    for user in &dm.recipients {
        let user_id = user.id;
        members = members.child(
            row()
                .w_full()
                .h(40.0)
                .px(8.0)
                .gap(10.0)
                .center_y()
                .child(avatar_status(user, 28.0, p, p.modal, true))
                .child(
                    text(user.display_name.clone())
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .grow(1.0)
                        .truncate(),
                )
                .children((owner && group).then(|| {
                    el().key(format!("dm-remove-{user_id}"))
                        .size(28.0, 28.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Remove {}", user.display_name))
                        .help("Remove from group")
                        .hover(p.danger.alpha(0.2))
                        .on_press(move |s: &mut App| s.remove_dm_member(channel, user_id))
                        .child(icon(&icons::CLOSE, 14.0).color(p.danger))
                })),
        );
    }

    let mut addable = col().w_full().gap(2.0);
    let mut candidates = 0;
    for rel in &s.relationships {
        if rel.kind != RelationshipKind::Friend {
            continue;
        }
        if rel.user.id == s.me.id
            || dm.recipients.iter().any(|user| user.id == rel.user.id)
            || !group
        {
            continue;
        }
        candidates += 1;
        let user_id = rel.user.id;
        addable = addable.child(
            row()
                .w_full()
                .h(36.0)
                .px(8.0)
                .gap(10.0)
                .center_y()
                .child(avatar_status(&rel.user, 24.0, p, p.modal, true))
                .child(
                    text(rel.user.display_name.clone())
                        .font(theme::SMALL)
                        .color(p.text_muted)
                        .grow(1.0)
                        .truncate(),
                )
                .child(
                    el().key(format!("dm-add-{user_id}"))
                        .h(28.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Add {}", rel.user.display_name))
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .on_press(move |s: &mut App| s.add_dm_member(channel, user_id))
                        .child(text("Add").font(theme::TINY).color(p.text_bright)),
                ),
        );
    }

    col()
        .key("dm-settings")
        .w(500.0)
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
                    text(if group {
                        "Group DM Settings"
                    } else {
                        "Direct Message Settings"
                    })
                    .font(theme::TITLE)
                    .color(p.text_bright)
                    .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            row()
                .w_full()
                .gap(14.0)
                .center_y()
                .child(icon_preview)
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(6.0)
                        .child(
                            text(format!(
                                "{total} {}",
                                if total == 1 { "member" } else { "members" }
                            ))
                            .font(theme::SMALL)
                            .color(p.text_muted),
                        )
                        .child(icon_actions),
                ),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(text("GROUP NAME").font(theme::SECTION).color(p.text_muted))
        .child(if owner {
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .child(
                    text_input(&s.dm_settings_name, theme::SMALL)
                        .key("dm-settings-name")
                        .grow(1.0)
                        .h(36.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Group name")
                        .placeholder("Group name", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.dm_settings_name),
                )
                .child(
                    el().key("dm-settings-save")
                        .h(36.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Save group name")
                        .fill(p.green)
                        .hover(p.green.alpha(0.85))
                        .on_press(|s: &mut App| s.save_dm_name())
                        .child(text("Save").font(theme::SMALL_MED).color(Color::WHITE)),
                )
        } else {
            el().w_full()
                .h(36.0)
                .px(10.0)
                .radius(4.0)
                .fill(p.input)
                .center_y()
                .child(
                    text(dm.channel.name.clone())
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .truncate(),
                )
        })
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(text("MEMBERS").font(theme::SECTION).color(p.text_muted))
        .child(
            col()
                .w_full()
                .max_h(180.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(members),
        )
        .children((group && candidates > 0).then(|| {
            col()
                .w_full()
                .gap(4.0)
                .child(
                    text("ADD A FRIEND")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(
                    col()
                        .w_full()
                        .max_h(150.0)
                        .scroll(ScrollSpec {
                            stick_bottom: false,
                        })
                        .child(addable),
                )
        }))
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .children((group).then(|| {
                    el().key("dm-settings-leave")
                        .h(34.0)
                        .px(12.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Leave group")
                        .hover(p.danger.alpha(0.2))
                        .on_press(move |s: &mut App| s.leave_dm(channel))
                        .child(text("Leave Group").font(theme::SMALL).color(p.danger))
                }))
                .child(el().grow(1.0))
                .child(
                    el().key("dm-settings-close")
                        .h(34.0)
                        .px(16.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Close group settings")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(|s: &mut App| s.modal = Modal::None)
                        .child(text("Done").font(theme::SMALL_MED).color(Color::WHITE)),
                ),
        )
}
