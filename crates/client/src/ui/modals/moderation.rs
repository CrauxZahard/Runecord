//! Modal sheets: moderation.

use super::*;

pub(crate) fn confirm_ban_sheet(s: &App, p: Palette, _guild: Id, user: Id) -> El {
    let name = s
        .user(user)
        .map(|user| user.display_name.clone())
        .unwrap_or_default();
    modal_frame(
        "confirm-ban",
        &format!("Ban {name}"),
        "Banning removes the member and blocks them from rejoining until they are unbanned.",
        p,
        col()
            .w_full()
            .gap(8.0)
            .child(text("BAN REASON").font(theme::SECTION).color(p.text_muted))
            .child(
                text_input(&s.ban_reason, theme::SMALL)
                    .key("ban-reason")
                    .w_full()
                    .h(38.0)
                    .radius(4.0)
                    .fill(p.rail)
                    .color(p.text)
                    .role("textbox")
                    .label("Reason (optional)")
                    .placeholder("Reason (optional)", p.text_faint)
                    .padding(10.0)
                    .on_edit(|s: &mut App| &mut s.ban_reason),
            )
            .child(
                text(format!(
                    "{name} will be removed from the server immediately."
                ))
                .font(theme::TINY)
                .color(p.text_faint),
            ),
        ("Ban", |s: &mut App| s.ban_now()),
        p,
    )
}

pub(crate) fn bans_list(s: &App, p: Palette) -> El {
    let mut list = col().w_full().gap(4.0);
    for ban in &s.bans {
        let user = &ban.user;
        let id = user.id;
        let name = user.display_name.clone();
        let reason = if ban.reason.trim().is_empty() {
            "No reason given".to_string()
        } else {
            ban.reason.clone()
        };
        let actor = s.user_name(ban.actor_id);
        list = list.child(
            row()
                .key(format!("ban-{}", id))
                .w_full()
                .min_h(64.0)
                .px(12.0)
                .py(8.0)
                .radius(6.0)
                .gap(10.0)
                .center_y()
                .fill(p.card)
                .child(avatar(user, 36.0, p))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(
                            text(name.clone())
                                .font(theme::SMALL_MED)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(
                            text(reason)
                                .font(theme::TINY)
                                .color(p.text_muted)
                                .max_lines(2),
                        )
                        .child(
                            text(format!(
                                "Banned by {actor} · {}",
                                date_label(ban.created_at)
                            ))
                            .font(theme::TINY)
                            .color(p.text_faint),
                        ),
                )
                .child(
                    el().key(format!("unban-{}", id))
                        .h(30.0)
                        .px(12.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Unban {name}"))
                        .fill(p.rail)
                        .hover(p.danger.alpha(0.2))
                        .on_press(move |s: &mut App| s.unban(id))
                        .child(text("Unban").font(theme::LABEL).color(p.danger)),
                ),
        );
    }
    if s.bans.is_empty() {
        list = list.child(
            col()
                .w_full()
                .py(48.0)
                .items_center()
                .gap(8.0)
                .child(icon(&icons::SHIELD, 40.0).color(p.text_faint))
                .child(
                    text("No banned users")
                        .font(theme::BODY)
                        .color(p.text_muted),
                )
                .child(
                    text("Ban a member from their right-click menu.")
                        .font(theme::SMALL)
                        .color(p.text_faint),
                ),
        );
    }
    list
}

pub(crate) fn prune_sheet(s: &App, p: Palette) -> El {
    let roles: Vec<&Role> = s
        .guild_roles(s.active_guild)
        .iter()
        .filter(|role| role.name != "@everyone")
        .collect();
    let mut role_rows = col().w_full().gap(2.0);
    for role in &roles {
        let id = role.id;
        let checked = s.prune_roles.contains(&id);
        let label = role.name.clone();
        role_rows = role_rows.child(
            row()
                .key(format!("prune-role-{}", id))
                .w_full()
                .h(32.0)
                .px(8.0)
                .radius(4.0)
                .gap(8.0)
                .center_y()
                .role("checkbox")
                .label(format!("Include {label}"))
                .checked(checked)
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.toggle_prune_role(id))
                .child(
                    el().size(16.0, 16.0)
                        .radius(3.0)
                        .fill(if checked { p.green } else { p.input })
                        .border(1.0, p.divider),
                )
                .child(
                    el().size(10.0, 10.0)
                        .radius(5.0)
                        .fill(theme::color(&format!("#{:06x}", role.color))),
                )
                .child(
                    text(label)
                        .font(theme::SMALL)
                        .color(p.text)
                        .grow(1.0)
                        .truncate(),
                ),
        );
    }

    let preview = match (s.prune_loading, s.prune_preview) {
        (true, _) => "Checking inactive members…".to_string(),
        (false, Some(1)) => "1 member would be pruned.".to_string(),
        (false, Some(count)) => format!("{count} members would be pruned."),
        _ => "Preview to count the members this would remove.".to_string(),
    };
    let ready = s.prune_preview.is_some() && !s.prune_loading;
    let count = s.prune_preview.unwrap_or(0);

    col()
        .key("prune-members")
        .w(460.0)
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
                    text("Prune Members")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Remove members who have not been active recently.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(text("INACTIVITY").font(theme::SECTION).color(p.text_muted))
        .child(
            row()
                .w_full()
                .h(40.0)
                .gap(10.0)
                .center_y()
                .child(
                    el().key("prune-days-minus")
                        .size(30.0, 30.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("One day less")
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .on_press(|s: &mut App| s.prune_days_delta(-1))
                        .child(text("−").font(theme::TITLE).color(p.text_bright)),
                )
                .child(
                    el().key("prune-days")
                        .w(74.0)
                        .h(32.0)
                        .radius(4.0)
                        .center()
                        .fill(p.input)
                        .child(
                            text(format!("{} days", s.prune_days))
                                .font(theme::SMALL_MED)
                                .color(p.text_bright),
                        ),
                )
                .child(
                    el().key("prune-days-plus")
                        .size(30.0, 30.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("One day more")
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .on_press(|s: &mut App| s.prune_days_delta(1))
                        .child(text("+").font(theme::TITLE).color(p.text_bright)),
                )
                .child(
                    text("1–30 days. Bots, the owner and you are never pruned.")
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        )
        .child(
            text("INCLUDE ROLES")
                .font(theme::SECTION)
                .color(p.text_muted)
                .pt(4.0),
        )
        .child(
            text("Members with roles are protected unless you include them here.")
                .font(theme::TINY)
                .color(p.text_faint),
        )
        .child(
            col()
                .w_full()
                .max_h(170.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(role_rows),
        )
        .child(
            row()
                .w_full()
                .min_h(36.0)
                .px(10.0)
                .radius(4.0)
                .center_y()
                .fill(p.input)
                .child(text(preview).font(theme::SMALL).color(if ready {
                    p.text_bright
                } else {
                    p.text_faint
                })),
        )
        .child(
            row()
                .w_full()
                .justify_end()
                .gap(8.0)
                .child(
                    el().key("prune-preview")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Preview members")
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .on_press(|s: &mut App| s.preview_prune())
                        .child(text("Preview").font(theme::SMALL_MED).color(p.text_bright)),
                )
                .child(
                    el().key("prune-confirm")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Prune {count} members"))
                        .fill(if ready { p.danger } else { p.rail })
                        .hover(if ready { p.danger_hover } else { p.rail })
                        .on_press(|s: &mut App| {
                            if s.prune_preview.is_some() {
                                s.prune_now();
                            }
                        })
                        .child(
                            text(if ready && count > 0 {
                                format!("Prune {count}")
                            } else {
                                "Prune".to_string()
                            })
                            .font(theme::SMALL_MED)
                            .color(if ready {
                                Color::WHITE
                            } else {
                                p.text_faint
                            }),
                        ),
                ),
        )
}

pub(crate) fn confirm_kick_sheet(s: &App, p: Palette, _guild: Id, user: Id) -> El {
    let name = s
        .user(user)
        .map(|user| user.display_name.clone())
        .unwrap_or_default();
    modal_frame(
        "confirm-kick",
        &format!("Kick {name}"),
        "Are you sure you want to kick this member? They can be invited back later.",
        p,
        col().w_full().gap(8.0).child(
            text(format!(
                "{name} will lose access to this server and its channels."
            ))
            .font(theme::SMALL)
            .color(p.text_muted),
        ),
        ("Kick", |s: &mut App| s.kick_now()),
        p,
    )
}

pub(crate) fn timeout_sheet(s: &App, p: Palette, guild: Id, user: Id) -> El {
    let name = s
        .user(user)
        .map(|user| user.display_name.clone())
        .unwrap_or_default();
    let until = s
        .member(guild, user)
        .map(|member| member.timeout_until)
        .unwrap_or(0);
    let active = until > now_ms();
    let entered = s.timeout_amount.text.trim().parse::<u64>().ok();
    let seconds =
        entered.map(|value| value.min(s.timeout_unit.max_value()) * s.timeout_unit.seconds());
    let expiry = seconds
        .filter(|seconds| *seconds > 0)
        .map(|seconds| expiry_label(now_ms() + seconds as i64 * 1000));
    let hint = match (entered, expiry) {
        (None, _) => "Enter a whole number.".to_string(),
        (Some(0), _) => "Use at least one unit.".to_string(),
        (Some(value), Some(expiry)) => {
            let max = s.timeout_unit.max_value();
            if value > max {
                format!(
                    "Clamped to {max} {}. Expires {expiry}.",
                    s.timeout_unit.label().to_lowercase()
                )
            } else {
                format!("Expires {expiry}.")
            }
        }
        _ => String::new(),
    };

    let mut units = row().w_full().gap(6.0);
    for unit in [TimeoutUnit::Minutes, TimeoutUnit::Hours, TimeoutUnit::Days] {
        let selected = s.timeout_unit == unit;
        units = units.child(
            el().key(format!("timeout-unit-{}", unit.label().to_lowercase()))
                .h(34.0)
                .px(12.0)
                .radius(4.0)
                .center()
                .role("radio")
                .label(unit.label())
                .checked(selected)
                .fill(if selected { p.selected } else { p.rail })
                .hover(p.rail_hover)
                .on_press(move |s: &mut App| s.set_timeout_unit(unit))
                .child(
                    text(unit.label())
                        .font(theme::SMALL_MED)
                        .color(if selected {
                            p.text_bright
                        } else {
                            p.text_muted
                        }),
                ),
        );
    }

    let mut footer = row().w_full().gap(8.0).justify_end().pt(4.0);
    if active {
        footer = footer.child(
            el().key("timeout-clear")
                .h(36.0)
                .px(14.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Remove timeout")
                .fill(p.rail)
                .hover(p.danger.alpha(0.2))
                .on_press(|s: &mut App| s.timeout_now(0))
                .child(
                    text("Remove Timeout")
                        .font(theme::SMALL_MED)
                        .color(p.danger),
                ),
        );
    }
    footer = footer.child(
        el().key("timeout-apply")
            .h(36.0)
            .px(16.0)
            .radius(4.0)
            .center()
            .role("button")
            .label("Apply timeout")
            .fill(p.accent)
            .hover(p.accent_hover)
            .on_press(|s: &mut App| s.apply_timeout())
            .child(
                text("Apply Timeout")
                    .font(theme::SMALL_MED)
                    .color(Color::WHITE),
            ),
    );

    col()
        .key("timeout-sheet")
        .w(430.0)
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
                    text(format!("Timeout {name}"))
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text(if active {
                format!(
                    "Currently timed out until {}. Choose a new duration to replace it.",
                    expiry_label(until)
                )
            } else {
                "Timed-out members cannot send messages or join voice.".to_string()
            })
            .font(theme::SMALL)
            .color(p.text_muted),
        )
        .child(text("DURATION").font(theme::SECTION).color(p.text_muted))
        .child(
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .child(
                    text_input(&s.timeout_amount, theme::BODY)
                        .key("timeout-amount")
                        .w(96.0)
                        .h(38.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Timeout amount")
                        .placeholder("10", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.timeout_amount),
                )
                .child(units),
        )
        .child(
            row()
                .w_full()
                .justify_between()
                .child(text(hint).font(theme::TINY).color(if entered.is_none() {
                    p.danger
                } else {
                    p.text_faint
                }))
                .child(
                    text("Maximum 28 days")
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        )
        .child(footer)
}

pub(crate) fn bans_sheet(s: &App, p: Palette) -> El {
    col()
        .key("bans")
        .w(520.0)
        .h(520.0)
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
                    text("Banned Users")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Banned members cannot rejoin until they are unbanned.")
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
                .child(appeals_list(s, p))
                .child(bans_list(s, p)),
        )
}

pub(crate) fn ban_appeal_sheet(s: &App, p: Palette) -> El {
    col()
        .key("ban-appeal")
        .w(470.0)
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
                    text("Submit an Appeal")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("You are banned from this server. Explain why the ban should be lifted; server managers see this in Server Settings → Bans.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            text(format!("Invite: {}", s.appeal_code))
                .font(theme::TINY)
                .color(p.text_faint),
        )
        .child(
            text_input(&s.appeal_message, theme::BODY)
                .key("appeal-message")
                .w_full()
                .h(110.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Appeal message")
                .placeholder("What should the moderators know?", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.appeal_message),
        )
        .child(
            row()
                .w_full()
                .justify_end()
                .child(
                    el().key("appeal-submit")
                        .h(36.0)
                        .px(16.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Submit appeal")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(|s: &mut App| s.submit_appeal())
                        .child(
                            text("Submit Appeal")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        ),
                ),
        )
}

pub(crate) fn appeals_list(s: &App, p: Palette) -> El {
    let mut list = col().w_full().gap(4.0);
    for appeal in &s.appeals {
        let id = appeal.id;
        let name = appeal.user.display_name.clone();
        list = list.child(
            row()
                .key(format!("appeal-{}", id))
                .w_full()
                .min_h(72.0)
                .px(12.0)
                .py(8.0)
                .radius(6.0)
                .gap(10.0)
                .center_y()
                .fill(p.card)
                .child(avatar(&appeal.user, 36.0, p))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(
                            row()
                                .gap(8.0)
                                .center_y()
                                .child(text(name).font(theme::SMALL_MED).color(p.text_bright))
                                .child(
                                    text(format!("Submitted {}", date_label(appeal.created_at)))
                                        .font(theme::TINY)
                                        .color(p.text_faint),
                                ),
                        )
                        .child(
                            text(appeal.message.clone())
                                .font(theme::SMALL)
                                .color(p.text)
                                .max_lines(3),
                        ),
                )
                .child(
                    el().key(format!("appeal-deny-{}", id))
                        .h(30.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Deny appeal")
                        .fill(p.rail)
                        .hover(p.danger.alpha(0.2))
                        .on_press(move |s: &mut App| s.resolve_appeal(id, false))
                        .child(text("Deny").font(theme::LABEL).color(p.danger)),
                )
                .child(
                    el().key(format!("appeal-accept-{}", id))
                        .h(30.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Accept appeal")
                        .fill(p.green)
                        .hover(p.green.alpha(0.85))
                        .on_press(move |s: &mut App| s.resolve_appeal(id, true))
                        .child(text("Accept").font(theme::LABEL).color(Color::WHITE)),
                ),
        );
    }
    if s.appeals.is_empty() {
        return el();
    }
    col()
        .w_full()
        .gap(8.0)
        .child(
            text(format!("PENDING APPEALS — {}", s.appeals.len()))
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(list)
}
