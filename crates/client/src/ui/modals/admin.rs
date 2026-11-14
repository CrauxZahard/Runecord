//! Roles and channel permissions.

use super::super::prelude::*;
use super::super::*;

pub(crate) fn channel_permissions_sheet(s: &App, p: Palette, channel: Id) -> El {
    let roles = s.guild_roles(s.active_guild);
    let detail = s.guild_detail(s.active_guild);
    let mut rows = col().w_full().gap(8.0);
    for role in roles {
        let target = role.id;
        let existing = detail
            .and_then(|guild| guild.overwrites.iter().find(|o| o.channel_id == channel))
            .and_then(|o| {
                o.overwrites
                    .iter()
                    .find(|w| w.target_id == target && w.kind == OverwriteKind::Role)
                    .cloned()
            });
        let allow = existing.as_ref().map(|o| o.allow).unwrap_or(0);
        let deny = existing.as_ref().map(|o| o.deny).unwrap_or(0);
        let mut chips = row().gap(4.0);
        for (label, bit) in [("View", perm::VIEW_CHANNEL), ("Send", perm::SEND_MESSAGES)] {
            let state = if allow & bit != 0 {
                1
            } else if deny & bit != 0 {
                2
            } else {
                0
            };
            let new_allow = if state == 1 {
                allow & !bit
            } else {
                allow | bit
            };
            let new_deny = if state == 2 { deny & !bit } else { deny | bit };
            chips = chips.child(
                el().h(26.0)
                    .px(8.0)
                    .radius(3.0)
                    .center()
                    .fill(match state {
                        1 => p.green,
                        2 => p.danger,
                        _ => p.input,
                    })
                    .on_press(move |s: &mut App| {
                        s.send(crate::net::Command::SetOverwrite {
                            channel,
                            target,
                            kind: "role".into(),
                            allow: new_allow,
                            deny: new_deny,
                        });
                        s.toast("Channel permission updated");
                    })
                    .child(
                        text(format!(
                            "{label}: {}",
                            match state {
                                1 => "allow",
                                2 => "deny",
                                _ => "inherit",
                            }
                        ))
                        .font(theme::TINY)
                        .color(Color::WHITE),
                    ),
            );
        }
        rows = rows.child(
            row()
                .h(36.0)
                .px(8.0)
                .gap(8.0)
                .center_y()
                .child(
                    text(role.name.clone())
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(chips),
        );
    }
    col()
        .key("channel-perms")
        .w(560.0)
        .max_h(600.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(18.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
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
            text("Allow / deny / inherit for this channel. Requires Manage Roles.")
                .font(theme::TINY)
                .color(p.text_faint),
        )
        .child(
            col()
                .w_full()
                .max_h(480.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(rows),
        )
}
