use super::*;

pub(crate) fn roles_view(s: &App, p: Palette) -> El {
    let mut roles: Vec<Role> = s.guild_roles(s.active_guild).to_vec();
    roles.sort_by_key(|role| std::cmp::Reverse(role.position));
    let mut list = col().key("role-list").w(240.0).gap(4.0);
    let dragging = s.role_drag;
    for role in &roles {
        let id = role.id;
        let selected = s.selected_role == id;
        // The dragged row is lifted toward the pointer and shaded so the reorder
        // reads as a real drag, while the rows it passes slide aside on their own.
        let (lift, dragged) = match dragging {
            Some((dragged_id, applied, dy)) if dragged_id == id => {
                let residual = dy - applied as f32 * 42.0;
                (residual.clamp(-126.0, 126.0), true)
            }
            _ => (0.0, false),
        };
        let mut role_row = row()
            .key(format!("role-select-{id}"))
            .w_full()
            .h(42.0)
            .px(10.0)
            .radius(5.0)
            .gap(10.0)
            .center_y()
            .role("button")
            .label(format!("{} role", role.name))
            .fill(if dragged {
                p.selected
            } else if selected {
                p.selected
            } else {
                Color::TRANSPARENT
            })
            .border(if dragged { 1.0 } else { 0.0 }, p.accent)
            .hover(p.card_hover)
            .on_drag(
                move |s: &mut App, drag: &mut rune::Drag, _: &mut rune::Tx| match drag.phase {
                    rune::Phase::Down => {
                        s.select_role(id);
                        s.role_drag = Some((id, 0, 0.0));
                    }
                    rune::Phase::Move => {
                        let Some((dragged_id, applied, _)) = s.role_drag else {
                            return;
                        };
                        if dragged_id != id {
                            return;
                        }
                        let target = (drag.dy / 42.0).round() as i32;
                        if target != applied {
                            let direction = if target > applied { 1 } else { -1 };
                            s.move_selected_role(direction);
                        }
                        s.role_drag = Some((id, target, drag.dy));
                    }
                    rune::Phase::Up => {
                        if drag.cancelled()
                            && let Some((dragged_id, applied, _)) = s.role_drag
                            && dragged_id == id
                        {
                            let direction = if applied > 0 { -1 } else { 1 };
                            for _ in 0..applied.abs() {
                                s.move_selected_role(direction);
                            }
                        }
                        s.role_drag = None;
                    }
                },
            )
            .on_press(move |s: &mut App| s.select_role(id));
        if dragged {
            role_row = role_row.offset(0.0, lift).shadow(4.0, 10.0, p.shadow);
        } else {
            role_row = role_row.motion(motion::SNAPPY);
        }
        list = list.child(
            role_row
                .child(
                    if let Some(pixels) = s
                        .images
                        .get(&role.icon_url)
                        .filter(|_| !role.icon_url.is_empty())
                    {
                        el().size(20.0, 20.0)
                            .radius(4.0)
                            .clip_rounded(4.0)
                            .pixels_fit(pixels, rune::SurfaceFit::Fill)
                    } else {
                        el().size(12.0, 12.0)
                            .radius(6.0)
                            .fill(theme::color(&format!("#{:06x}", role.color)))
                    },
                )
                .child(
                    col().grow(1.0).min_w(0.0).child(
                        text(role.name.clone())
                            .font(theme::SMALL_MED)
                            .color(p.text_bright)
                            .truncate(),
                    ),
                ),
        );
    }

    let left = col()
        .w(240.0)
        .h_full()
        .gap(8.0)
        .child(
            row()
                .w_full()
                .gap(6.0)
                .child(
                    el().h(30.0)
                        .px(12.0)
                        .radius(4.0)
                        .key("role-new")
                        .role("button")
                        .label("Create role")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .center()
                        .on_press(|s: &mut App| s.create_role_now())
                        .child(text("New Role").font(theme::SMALL_MED).color(Color::WHITE)),
                )
                .child(
                    text("Drag roles to reorder")
                        .wrap()
                        .font(theme::TINY)
                        .color(p.text_faint)
                        .grow(1.0),
                ),
        )
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list),
        );

    let editor = role_editor(s, p);
    col()
        .key("roles-view")
        .w_full()
        .h_full()
        .min_h(0.0)
        .grow(1.0)
        .child(
            row()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .gap(18.0)
                .child(left)
                .child(el().h_full().w(1.0).fill(p.divider))
                .child(editor),
        )
        .child(role_footer(s, p))
}

pub(crate) fn role_footer(s: &App, p: Palette) -> El {
    let Some(role) = s
        .guild_roles(s.active_guild)
        .iter()
        .find(|role| role.id == s.selected_role)
        .cloned()
    else {
        return el();
    };
    let everyone = role.name == "@everyone";
    let dirty = s.role_name.text.trim() != role.name
        || s.role_color != role.color
        || s.role_permissions != role.permissions
        || s.role_hoist != role.hoist
        || s.role_mentionable != role.mentionable;
    row()
        .key("role-footer")
        .w_full()
        .h(64.0)
        .px(16.0)
        .gap(10.0)
        .center_y()
        .fill(p.modal_sidebar)
        .border_sides([1.0, 0.0, 0.0, 0.0], p.divider)
        .child(
            el().key("role-delete")
                .h(34.0)
                .px(12.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Delete role")
                .hover(p.danger.alpha(0.2))
                .on_press(move |s: &mut App| {
                    if !everyone {
                        s.delete_selected_role();
                    }
                })
                .child(text("Delete Role").font(theme::LABEL).color(if everyone {
                    p.text_faint
                } else {
                    p.danger
                })),
        )
        .child(el().grow(1.0))
        .children(dirty.then(|| {
            text("Careful - you have unsaved changes.")
                .font(theme::SMALL_MED)
                .color(p.text_bright)
        }))
        .child(
            el().key("role-reset")
                .h(34.0)
                .px(14.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Reset role changes")
                .hover(p.card_hover)
                .children(dirty.then(|| text("Reset").font(theme::SMALL_MED).color(p.text_muted))),
        )
        .child(
            el().h(34.0)
                .px(16.0)
                .radius(4.0)
                .key("role-save")
                .role("button")
                .label("Save role")
                .fill(if dirty { p.green } else { p.rail })
                .hover(if dirty { p.green.alpha(0.85) } else { p.rail })
                .center()
                .on_press(move |s: &mut App| {
                    if dirty {
                        s.save_role();
                    }
                })
                .child(text("Save Changes").font(theme::SMALL_MED).color(if dirty {
                    Color::WHITE
                } else {
                    p.text_faint
                })),
        )
}

pub(crate) fn manage_members_view(s: &App, role: &Role, p: Palette) -> El {
    let everyone = role.name == "@everyone";
    let role_id = role.id;
    let members = s
        .members
        .get(&s.active_guild)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let mut rows = col().w_full().gap(2.0);
    for member in members.iter() {
        let user_id = member.user.id;
        let assigned = everyone || member.roles.contains(&role_id);
        rows = rows.child(
            row()
                .key(format!("role-member-toggle-{user_id}"))
                .w_full()
                .h(44.0)
                .px(8.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .role("checkbox")
                .label(format!("{} has this role", member.user.display_name))
                .checked(assigned)
                .hover(p.card_hover)
                .on_press(move |s: &mut App| {
                    if !everyone {
                        s.toggle_role_for_member(role_id, user_id);
                    }
                })
                .child(avatar(&member.user, 28.0, p))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text(member.user.display_name.clone())
                                .font(theme::SMALL_MED)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(
                            text(format!("@{}", member.user.username))
                                .font(theme::TINY)
                                .color(p.text_muted)
                                .truncate(),
                        ),
                )
                .child(
                    el().size(16.0, 16.0)
                        .radius(4.0)
                        .fill(if assigned { p.green } else { p.input })
                        .border(1.0, p.divider)
                        .center()
                        .children(assigned.then(|| icon(&icons::CHECK, 11.0).color(Color::WHITE))),
                ),
        );
    }
    col()
        .key("role-members")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .gap(8.0)
        .child(
            text(if everyone {
                "Everyone in the server has this role."
            } else {
                "Click a member to add or remove this role."
            })
            .font(theme::TINY)
            .color(p.text_muted),
        )
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(rows),
        )
}

pub(crate) fn role_editor(s: &App, p: Palette) -> El {
    let Some(role) = s
        .guild_roles(s.active_guild)
        .iter()
        .find(|role| role.id == s.selected_role)
        .cloned()
    else {
        return col().key("role-editor").grow(1.0).center().child(
            text("Select or create a role.")
                .font(theme::SMALL)
                .color(p.text_faint),
        );
    };
    let everyone = role.name == "@everyone";
    let guild = s.active_guild;
    let role_member_count = s
        .members
        .get(&guild)
        .map(|list| {
            list.iter()
                .filter(|member| everyone || member.roles.contains(&role.id))
                .count()
        })
        .unwrap_or(0);
    let mut perms = col().w_full().gap(2.0);
    for (group, entries) in [
        (
            "GENERAL SERVER PERMISSIONS",
            &[
                ("View Channels", perm::VIEW_CHANNEL),
                ("Manage Channels", perm::MANAGE_CHANNELS),
                ("Manage Roles", perm::MANAGE_ROLES),
                ("Manage Server", perm::MANAGE_GUILD),
                ("Create Invite", perm::CREATE_INVITE),
            ][..],
        ),
        (
            "MEMBERSHIP",
            &[
                ("Kick Members", perm::KICK_MEMBERS),
                ("Ban Members", perm::BAN_MEMBERS),
                ("Timeout Members", perm::MODERATE_MEMBERS),
            ][..],
        ),
        (
            "TEXT CHANNEL PERMISSIONS",
            &[
                ("Send Messages", perm::SEND_MESSAGES),
                ("Manage Messages", perm::MANAGE_MESSAGES),
                ("Manage Threads", perm::MANAGE_THREADS),
                ("Mention Everyone", perm::MENTION_EVERYONE),
            ][..],
        ),
        ("ADVANCED", &[("Administrator", perm::ADMINISTRATOR)][..]),
    ] {
        perms = perms
            .child(
                text(group)
                    .font(theme::SECTION)
                    .color(p.text_muted)
                    .padding(8.0),
            )
            .child(el().w_full().h(1.0).fill(p.divider).mb(4.0));
        for (label, bit) in entries {
            let on = s.role_permissions & bit != 0;
            let bit = *bit;
            perms = perms.child(
                row()
                    .key(format!("role-perm-{bit}"))
                    .w_full()
                    .h(30.0)
                    .px(8.0)
                    .radius(4.0)
                    .gap(8.0)
                    .center_y()
                    .role("checkbox")
                    .label(*label)
                    .checked(on)
                    .hover(p.card_hover)
                    .on_press(move |s: &mut App| s.role_permissions ^= bit)
                    .child(
                        el().size(14.0, 14.0)
                            .radius(3.0)
                            .fill(if on { p.green } else { p.input })
                            .border(1.0, p.divider),
                    )
                    .child(text(*label).font(theme::SMALL).color(p.text)),
            );
        }
    }

    let display_tab = s.role_tab == 0;
    let tab = |label: &str, selected: bool, key: &'static str, index: u8| {
        let caption = label.to_string();
        col()
            .key(key)
            .h(36.0)
            .gap(5.0)
            .justify_end()
            .role("tab")
            .label(caption.clone())
            .checked(selected)
            .on_press(move |s: &mut App| s.role_tab = index)
            .child(text(caption).font(theme::SMALL_MED).color(if selected {
                p.text_bright
            } else {
                p.text_muted
            }))
            .child(el().h(2.0).w_full().fill(if selected {
                p.accent
            } else {
                Color::TRANSPARENT
            }))
    };

    let display = col()
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .gap(12.0)
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .child(text("ROLE NAME").font(theme::SECTION).color(p.text_muted))
        .child(
            text_input(&s.role_name, theme::SMALL)
                .key("role-name")
                .w_full()
                .h(38.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Role name")
                .placeholder("Role name", p.text_faint)
                .padding(10.0)
                .disabled(everyone)
                .on_edit(|s: &mut App| &mut s.role_name),
        )
        .child(
            text(if everyone {
                "EVERYONE MENTION"
            } else {
                "ROLE ID"
            })
            .font(theme::SECTION)
            .color(p.text_muted),
        )
        .child(if everyone {
            // `@everyone` is the server-wide ping, not a role you address by
            // ID, so show the word itself instead of an ID to copy.
            row()
                .w_full()
                .h(34.0)
                .gap(8.0)
                .center_y()
                .child(
                    el().grow(1.0)
                        .min_w(0.0)
                        .h(34.0)
                        .radius(4.0)
                        .fill(p.input)
                        .px(10.0)
                        .center_y()
                        .child(text("@everyone").font(theme::MONO).color(p.text).truncate()),
                )
                .child(
                    el().key("role-copy-mention")
                        .h(34.0)
                        .px(12.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Copy mention")
                        .hover(p.card_hover)
                        .on_press_tx(move |s: &mut App, tx: &mut rune::Tx| {
                            tx.copy("@everyone".to_string());
                            s.toast("Mention copied");
                        })
                        .child(text("Copy Mention").font(theme::LABEL).color(p.text_muted)),
                )
        } else {
            row()
                .w_full()
                .h(34.0)
                .gap(8.0)
                .center_y()
                .child(
                    el().grow(1.0)
                        .min_w(0.0)
                        .h(34.0)
                        .radius(4.0)
                        .fill(p.input)
                        .px(10.0)
                        .center_y()
                        .child(
                            text(role.id.to_string())
                                .font(theme::MONO)
                                .color(p.text)
                                .truncate(),
                        ),
                )
                .child(
                    el().key("role-copy-id")
                        .h(34.0)
                        .px(12.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Copy role ID")
                        .hover(p.card_hover)
                        .on_press_tx(move |s: &mut App, tx: &mut rune::Tx| {
                            tx.copy(role.id.to_string());
                            s.toast("Role ID copied");
                        })
                        .child(text("Copy ID").font(theme::LABEL).color(p.text_muted)),
                )
                .child(
                    el().key("role-copy-mention")
                        .h(34.0)
                        .px(12.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Copy role mention")
                        .hover(p.card_hover)
                        .on_press_tx(move |s: &mut App, tx: &mut rune::Tx| {
                            tx.copy(format!("<@&{}>", role.id));
                            s.toast("Role mention copied");
                        })
                        .child(text("Copy Mention").font(theme::LABEL).color(p.text_muted)),
                )
        })
        .child(text("ROLE COLOR").font(theme::SECTION).color(p.text_muted))
        .child(
            text("Members use the color of the highest role they have on the roles list.")
                .font(theme::TINY)
                .color(p.text_muted),
        )
        .child(role_swatches(s.role_color, p))
        .child(
            row()
                .w(140.0)
                .h(32.0)
                .radius(4.0)
                .fill(p.input)
                .center_y()
                .padding(8.0)
                .gap(8.0)
                .child(
                    el().size(14.0, 14.0)
                        .radius(7.0)
                        .fill(if s.role_color == 0 {
                            p.text_faint
                        } else {
                            theme::color(&format!("#{:06x}", s.role_color))
                        }),
                )
                .child(
                    text(if s.role_color == 0 {
                        "None".to_string()
                    } else {
                        format!("#{:06X}", s.role_color)
                    })
                    .font(theme::SMALL)
                    .color(p.text),
                ),
        )
        .child(text("ROLE ICON").font(theme::SECTION).color(p.text_muted))
        .child(role_icon_row(s, &role, p))
        .child(
            row()
                .w_full()
                .gap(16.0)
                .child(toggle(
                    "role-hoist",
                    "Show separately",
                    s.role_hoist,
                    p,
                    |s: &mut App| s.role_hoist = !s.role_hoist,
                ))
                .child(toggle(
                    "role-mention",
                    "Mentionable",
                    s.role_mentionable,
                    p,
                    |s: &mut App| s.role_mentionable = !s.role_mentionable,
                )),
        )
        .child(text("PREVIEW").font(theme::SECTION).color(p.text_muted))
        .child(role_preview(&s.role_name.text, s.role_color, p));

    let permissions_view = col().w_full().grow(1.0).min_h(0.0).gap(8.0).child(
        col()
            .w_full()
            .grow(1.0)
            .min_h(0.0)
            .scroll(ScrollSpec {
                stick_bottom: false,
            })
            .child(perms),
    );

    col()
        .key("role-editor")
        .grow(1.0)
        .max_w(740.0)
        .min_w(0.0)
        .h_full()
        .gap(10.0)
        .child(
            text(format!("EDIT ROLE — {}", role.name.to_uppercase()))
                .font(theme::TITLE)
                .color(p.text_bright),
        )
        .child(
            row()
                .w_full()
                .h(38.0)
                .gap(24.0)
                .child(tab("Display", display_tab, "role-page-display", 0))
                .child(tab(
                    "Permissions",
                    s.role_tab == 1,
                    "role-page-permissions",
                    1,
                ))
                .child(tab("Links", s.role_tab == 2, "role-page-links", 2))
                .child(tab(
                    &format!("Manage Members ({})", role_member_count),
                    s.role_tab == 3,
                    "role-page-members",
                    3,
                )),
        )
        .child(match s.role_tab {
            1 => permissions_view,
            2 => role_links_view(s, &role, p),
            3 => manage_members_view(s, &role, p),
            _ => display,
        })
}

pub(crate) fn role_icon_row(s: &App, role: &Role, p: Palette) -> El {
    // Role icons are a free cosmetic in this build.
    let unlocked = true;
    let role_id = role.id;
    let guild_id = s.active_guild;
    let icon_url = role.icon_url.clone();
    let preview = if let Some(pixels) = s.images.get(&icon_url).filter(|_| !icon_url.is_empty()) {
        el().size(44.0, 44.0)
            .radius(8.0)
            .clip_rounded(8.0)
            .pixels_fit(pixels, rune::SurfaceFit::Fill)
    } else {
        el().size(44.0, 44.0)
            .radius(8.0)
            .fill(if role.color != 0 {
                theme::color(&format!("#{:06x}", role.color))
            } else {
                p.input
            })
            .center()
            .child(icon(&icons::USERS, 20.0).color(p.text_bright))
    };
    let mut actions = row().gap(8.0).center_y();
    if unlocked {
        actions = actions
            .child(
                el().key("role-icon-upload")
                    .h(32.0)
                    .px(12.0)
                    .radius(4.0)
                    .center()
                    .role("button")
                    .label("Upload role icon")
                    .fill(p.accent)
                    .hover(p.accent_hover)
                    .on_press_tx(move |_s: &mut App, tx: &mut rune::Tx| {
                        let kind = crate::files::PickKind::RoleIcon {
                            guild: guild_id,
                            role: role_id,
                        };
                        crate::files::open(
                            tx,
                            kind.clone(),
                            "Upload role icon",
                            &["png", "jpg", "jpeg", "gif", "webp"],
                            move |s, path| s.accept_picked_path(kind.clone(), path),
                        );
                    })
                    .child(text("Upload Icon").font(theme::LABEL).color(Color::WHITE)),
            )
            .children((!icon_url.is_empty()).then(|| {
                el().key("role-icon-remove")
                    .h(30.0)
                    .px(10.0)
                    .radius(4.0)
                    .center()
                    .role("button")
                    .label("Remove role icon")
                    .hover(p.danger.alpha(0.2))
                    .on_press(move |s: &mut App| s.remove_role_icon(role_id))
                    .child(text("Remove").font(theme::LABEL).color(p.danger))
            }));
    } else {
        actions = actions.child(
            text("Unlocks at Boost Level 2 (7 Boosts)")
                .font(theme::TINY)
                .color(p.text_muted),
        );
    }
    row()
        .key("role-icon")
        .w_full()
        .gap(12.0)
        .center_y()
        .child(preview)
        .child(actions)
        .child(
            text("Shown next to the role in member lists and role chips.")
                .font(theme::TINY)
                .color(p.text_muted)
                .grow(1.0)
                .truncate(),
        )
}

pub(crate) fn role_links_view(s: &App, role: &Role, p: Palette) -> El {
    let role_id = role.id;
    let links = s.role_links.get(&role.id).cloned().unwrap_or_default();
    let mut list = col().w_full().gap(4.0);
    for link in &links {
        let link_id = link.id;
        list = list.child(
            row()
                .key(format!("role-link-{link_id}"))
                .w_full()
                .h(48.0)
                .px(8.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .hover(p.card_hover)
                .child(icon(&icons::LINK, 16.0).color(p.text_muted))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text(link.label.clone())
                                .font(theme::SMALL_MED)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(
                            text(link.url.clone())
                                .font(theme::TINY)
                                .color(p.text_link)
                                .truncate(),
                        ),
                )
                .child(
                    el().key(format!("role-link-remove-{link_id}"))
                        .size(28.0, 28.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Remove {}", link.label))
                        .hover(p.danger.alpha(0.2))
                        .on_press(move |s: &mut App| s.delete_role_link(role_id, link_id))
                        .child(icon(&icons::CLOSE, 14.0).color(p.danger)),
                ),
        );
    }
    if links.is_empty() {
        list = list.child(
            row().w_full().h(64.0).center().child(
                text("No links yet. Add the first link for this role.")
                    .font(theme::SMALL)
                    .color(p.text_muted),
            ),
        );
    }
    col()
        .key("role-links")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .gap(12.0)
        .child(
            text("LINKS")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(
            text("Attach up to five links to document this role. Links are stored with the role and shown to managers in this tab.")
                .font(theme::TINY)
                .color(p.text_muted),
        )
        .child(
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .child(
                    text_input(&s.role_link_label, theme::SMALL)
                        .key("role-link-label")
                        .h(34.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Link label")
                        .placeholder("Label", p.text_faint)
                        .padding(8.0)
                        .grow(1.0)
                        .on_edit(|s: &mut App| &mut s.role_link_label),
                )
                .child(
                    text_input(&s.role_link_url, theme::SMALL)
                        .key("role-link-url")
                        .h(34.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Link URL")
                        .placeholder("https://", p.text_faint)
                        .padding(8.0)
                        .grow(2.0)
                        .on_edit(|s: &mut App| &mut s.role_link_url)
                        .on_submit(move |s: &mut App, _: &mut rune::Tx| s.add_role_link(role_id)),
                )
                .child(
                    el().key("role-link-add")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Add role link")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(move |s: &mut App| s.add_role_link(role_id))
                        .child(
                            text("Add Link")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        ),
                ),
        )
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list),
        )
}

pub(crate) fn role_preview(name: &str, color: u32, p: Palette) -> El {
    let name = name.trim().to_string();
    let name = if name.is_empty() {
        "New Role".to_string()
    } else {
        name
    };
    let mention = name.to_lowercase().replace(' ', "");
    // The everyone role is already spelled as a mention (`@everyone`), so it
    // does not take another `@`.
    let handle = if name.starts_with('@') {
        name.clone()
    } else {
        format!("@{mention}")
    };
    let tint = if color == 0 {
        p.text_bright
    } else {
        theme::color(&format!("#{color:06x}"))
    };
    let message = |highlight: bool, body: &str, p: Palette| {
        let fill = if highlight {
            p.card_hover
        } else {
            Color::TRANSPARENT
        };
        row()
            .w_full()
            .px(10.0)
            .py(8.0)
            .gap(12.0)
            .fill(fill)
            .child(
                el().size(40.0, 40.0)
                    .radius(20.0)
                    .fill(tint.alpha(if highlight { 0.35 } else { 0.16 }))
                    .center()
                    .child(icon(&icons::USERS, 20.0).color(tint.alpha(if highlight {
                        1.0
                    } else {
                        0.5
                    }))),
            )
            .child(
                col()
                    .grow(1.0)
                    .min_w(0.0)
                    .gap(2.0)
                    .child(
                        row()
                            .gap(8.0)
                            .center_y()
                            .child(text(name.clone()).font(theme::NAME).color(if highlight {
                                tint
                            } else {
                                tint.alpha(0.5)
                            }))
                            .child(
                                text("Today at 12:34 PM")
                                    .font(theme::TINY)
                                    .color(p.text_muted),
                            ),
                    )
                    .child(
                        text(body.to_string())
                            .font(theme::BODY)
                            .color(if highlight { p.text } else { p.text_muted }),
                    ),
            )
    };
    col()
        .key("role-preview")
        .w_full()
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.chat)
        .border(1.0, p.divider)
        .child(message(
            true,
            &format!("This is how {handle} appears in chat."),
            p,
        ))
        .child(message(false, "Mention the role to notify its members.", p))
}

pub(crate) fn role_swatches(selected: u32, p: Palette) -> El {
    let mut row_el = row().w_full().gap(8.0).flex_wrap();
    row_el = row_el.child(
        el().size(26.0, 26.0)
            .radius(13.0)
            .border(1.0, p.divider)
            .center()
            .key("role-color-none")
            .role("button")
            .label("No role color")
            .on_press(|s: &mut App| s.role_color = 0)
            .child(icon(&icons::CLOSE, 12.0).color(p.text_muted)),
    );
    for seed in 0..10u32 {
        let value = theme::avatar_color_value(seed);
        row_el = row_el.child(
            el().size(26.0, 26.0)
                .radius(13.0)
                .fill(theme::avatar_color(seed))
                .border(if value == selected { 2.0 } else { 0.0 }, p.text_bright)
                .key(format!("role-color-{value}"))
                .role("button")
                .label(format!("Role color {:06x}", value))
                .on_press(move |s: &mut App| s.role_color = value),
        );
    }
    row_el
}
