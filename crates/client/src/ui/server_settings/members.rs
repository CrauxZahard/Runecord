use super::*;

pub(crate) fn members_view(s: &App, p: Palette) -> El {
    let members = s
        .members
        .get(&s.active_guild)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let mut list = col().key("role-member-list").w(300.0).gap(4.0);
    for member in members {
        let id = member.user.id;
        let selected = s.selected_member == Some(id);
        list = list.child(
            row()
                .key(format!("role-member-{id}"))
                .w_full()
                .h(44.0)
                .px(10.0)
                .radius(5.0)
                .gap(10.0)
                .center_y()
                .role("button")
                .label(format!("{} member", member.user.display_name))
                .fill(if selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.select_member(id))
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
                            text(format!("{} roles", member.roles.len()))
                                .font(theme::TINY)
                                .color(p.text_faint),
                        ),
                ),
        );
    }

    let Some(member) = s
        .selected_member
        .and_then(|id| members.iter().find(|member| member.user.id == id).cloned())
    else {
        return row()
            .w_full()
            .grow(1.0)
            .gap(18.0)
            .child(
                col()
                    .w(300.0)
                    .grow(1.0)
                    .min_h(0.0)
                    .scroll(ScrollSpec {
                        stick_bottom: false,
                    })
                    .child(list),
            )
            .child(el().h_full().w(1.0).fill(p.divider))
            .child(
                col().grow(1.0).center().child(
                    text("Select a member.")
                        .font(theme::SMALL)
                        .color(p.text_faint),
                ),
            );
    };

    let mut roles = col().w_full().gap(3.0);
    for role in s.guild_roles(s.active_guild) {
        if role.name == "@everyone" {
            continue;
        }
        let id = role.id;
        let on = member.roles.contains(&id);
        roles = roles.child(
            row()
                .key(format!("member-role-{id}"))
                .w_full()
                .h(32.0)
                .px(8.0)
                .radius(4.0)
                .gap(8.0)
                .center_y()
                .role("checkbox")
                .label(role.name.clone())
                .checked(on)
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.toggle_member_role(id))
                .child(
                    el().size(14.0, 14.0)
                        .radius(3.0)
                        .fill(if on { p.green } else { p.input })
                        .border(1.0, p.divider),
                )
                .child(
                    el().size(10.0, 10.0)
                        .radius(5.0)
                        .fill(theme::color(&format!("#{:06x}", role.color))),
                )
                .child(
                    text(role.name.clone())
                        .font(theme::SMALL)
                        .color(p.text)
                        .grow(1.0),
                ),
        );
    }

    row()
        .w_full()
        .grow(1.0)
        .gap(18.0)
        .child(
            col()
                .w(300.0)
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list),
        )
        .child(el().h_full().w(1.0).fill(p.divider))
        .child(
            col()
                .key("member-role-editor")
                .grow(1.0)
                .max_w(740.0)
                .min_w(0.0)
                .h_full()
                .gap(12.0)
                .child(
                    row()
                        .gap(12.0)
                        .center_y()
                        .child(avatar(&member.user, 40.0, p))
                        .child(
                            col()
                                .child(
                                    text(member.user.display_name.clone())
                                        .font(theme::TITLE)
                                        .color(p.text_bright),
                                )
                                .child(
                                    text(member.user.handle())
                                        .font(theme::SMALL)
                                        .color(p.text_muted),
                                ),
                        ),
                )
                .child(text("ROLES").font(theme::SECTION).color(p.text_muted))
                .child(
                    col()
                        .w_full()
                        .grow(1.0)
                        .min_h(0.0)
                        .scroll(ScrollSpec {
                            stick_bottom: false,
                        })
                        .child(roles),
                ),
        )
}
