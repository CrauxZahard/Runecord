//! Roles and channel permissions.

use super::super::*;

const PERMISSION_GROUPS: &[(&str, &[(&str, u64)])] = &[
    (
        "GENERAL",
        &[
            ("View Channel", perm::VIEW_CHANNEL),
            ("Manage Channels", perm::MANAGE_CHANNELS),
            ("Manage Roles", perm::MANAGE_ROLES),
            ("Manage Server", perm::MANAGE_GUILD),
            ("Create Invite", perm::CREATE_INVITE),
        ],
    ),
    (
        "MEMBERSHIP",
        &[
            ("Kick Members", perm::KICK_MEMBERS),
            ("Ban Members", perm::BAN_MEMBERS),
            ("Timeout Members", perm::MODERATE_MEMBERS),
        ],
    ),
    (
        "TEXT",
        &[
            ("Send Messages", perm::SEND_MESSAGES),
            ("Manage Messages", perm::MANAGE_MESSAGES),
            ("Embed Links", perm::EMBED_LINKS),
            ("Attach Files", perm::ATTACH_FILES),
            ("Read History", perm::READ_MESSAGE_HISTORY),
            ("Mention Everyone", perm::MENTION_EVERYONE),
            ("Add Reactions", perm::ADD_REACTIONS),
            ("External Emoji", perm::USE_EXTERNAL_EMOJIS),
            ("Create Threads", perm::CREATE_THREADS),
            ("Manage Threads", perm::MANAGE_THREADS),
        ],
    ),
    (
        "VOICE",
        &[
            ("Connect", perm::CONNECT),
            ("Speak", perm::SPEAK),
            ("Video", perm::STREAM),
            ("Mute Members", perm::MUTE_MEMBERS),
            ("Deafen Members", perm::DEAFEN_MEMBERS),
            ("Move Members", perm::MOVE_MEMBERS),
            ("Priority Speaker", perm::PRIORITY_SPEAKER),
        ],
    ),
];

pub(crate) fn channel_permissions_sheet(s: &App, p: Palette, channel: Id) -> El {
    let detail = s.guild_detail(s.active_guild);
    let overwrites = detail
        .and_then(|guild| guild.overwrites.iter().find(|o| o.channel_id == channel))
        .map(|o| o.overwrites.as_slice())
        .unwrap_or(&[]);
    let target = s.perm_target;
    let target_kind = s.perm_target_kind.as_str();
    let existing = overwrites
        .iter()
        .find(|w| {
            w.target_id == target && (target_kind == "member") == (w.kind == OverwriteKind::Member)
        })
        .cloned();
    let allow = existing.as_ref().map(|o| o.allow).unwrap_or(0);
    let deny = existing.as_ref().map(|o| o.deny).unwrap_or(0);

    let mut targets = col().w(190.0).h_full().gap(1.0);
    targets = targets.child(
        text("ROLES")
            .font(theme::SECTION)
            .color(p.text_muted)
            .padding(8.0),
    );
    for role in s.guild_roles(s.active_guild) {
        let everyone = role.name == "@everyone";
        let id = if everyone { 0 } else { role.id };
        let selected = target_kind == "role" && target == id;
        let role_name = role.name.clone();
        targets = targets.child(
            row()
                .key(format!("perm-role-{id}"))
                .w_full()
                .h(30.0)
                .px(8.0)
                .radius(4.0)
                .gap(6.0)
                .center_y()
                .role("button")
                .label(format!("Edit permissions for {role_name}"))
                .fill(if selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| {
                    s.perm_target = id;
                    s.perm_target_kind = "role".into();
                })
                .child(
                    el().size(10.0, 10.0)
                        .radius(5.0)
                        .fill(theme::color(&format!("#{:06x}", role.color))),
                )
                .child(
                    text(role_name)
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .truncate(),
                ),
        );
    }
    targets = targets.child(
        text("MEMBERS")
            .font(theme::SECTION)
            .color(p.text_muted)
            .padding(8.0)
            .pt(14.0),
    );
    for member in s
        .members
        .get(&s.active_guild)
        .map(Vec::as_slice)
        .unwrap_or(&[])
    {
        let id = member.user.id;
        let selected = target_kind == "member" && target == id;
        let name = member.user.display_name.clone();
        targets = targets.child(
            row()
                .key(format!("perm-member-{id}"))
                .w_full()
                .h(30.0)
                .px(8.0)
                .radius(4.0)
                .gap(6.0)
                .center_y()
                .role("button")
                .label(format!("Edit permissions for {name}"))
                .fill(if selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| {
                    s.perm_target = id;
                    s.perm_target_kind = "member".into();
                })
                .child(avatar(&member.user, 20.0, p))
                .child(
                    text(name)
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .truncate(),
                ),
        );
    }

    let mut rows = col().w_full().gap(2.0);
    for (group, entries) in PERMISSION_GROUPS {
        rows = rows.child(
            text(*group)
                .font(theme::SECTION)
                .color(p.text_muted)
                .padding(8.0)
                .pt(10.0),
        );
        for (label, bit) in *entries {
            let state = if allow & bit != 0 {
                1
            } else if deny & bit != 0 {
                2
            } else {
                0
            };
            let (next_allow, next_deny) = match state {
                0 => (allow | bit, deny),
                1 => (allow & !bit, deny | bit),
                _ => (allow & !bit, deny & !bit),
            };
            let kind = target_kind.to_string();
            rows = rows.child(
                row()
                    .key(format!("perm-{target}-{bit}"))
                    .w_full()
                    .h(32.0)
                    .px(8.0)
                    .radius(4.0)
                    .gap(8.0)
                    .center_y()
                    .hover(p.card_hover)
                    .child(text(*label).font(theme::SMALL).color(p.text).grow(1.0))
                    .child(
                        el().h(24.0)
                            .px(10.0)
                            .radius(4.0)
                            .center()
                            .role("button")
                            .label(format!(
                                "{label}: {}",
                                match state {
                                    1 => "allow",
                                    2 => "deny",
                                    _ => "inherit",
                                }
                            ))
                            .fill(match state {
                                1 => p.green,
                                2 => p.danger,
                                _ => p.input,
                            })
                            .on_press(move |s: &mut App| {
                                if next_allow == 0 && next_deny == 0 {
                                    s.send(crate::net::Command::ClearOverwrite {
                                        channel,
                                        target,
                                        kind: kind.clone(),
                                    });
                                } else {
                                    s.send(crate::net::Command::SetOverwrite {
                                        channel,
                                        target,
                                        kind: kind.clone(),
                                        allow: next_allow,
                                        deny: next_deny,
                                    });
                                }
                            })
                            .child(
                                text(match state {
                                    1 => "Allow",
                                    2 => "Deny",
                                    _ => "Inherit",
                                })
                                .font(theme::TINY)
                                .color(if state == 0 {
                                    p.text_muted
                                } else {
                                    Color::WHITE
                                }),
                            ),
                    ),
            );
        }
    }

    col()
        .key("channel-perms")
        .w(sheet_w(760.0))
        .h(sheet_h(620.0))
        .radius(8.0)
        .clip_rounded(8.0)
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
                    text("Channel Permissions")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Pick a role or member, then cycle each permission through Inherit, Allow and Deny.")
                .font(theme::TINY)
                .color(p.text_faint),
        )
        .child(
            row()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .gap(12.0)
                .child(
                    col()
                        .w(190.0)
                        .h_full()
                        .scroll(ScrollSpec {
                            stick_bottom: false,
                        })
                        .child(targets),
                )
                .child(el().h_full().w(1.0).fill(p.divider))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .h_full()
                        .scroll(ScrollSpec {
                            stick_bottom: false,
                        })
                        .child(rows),
                ),
        )
}
