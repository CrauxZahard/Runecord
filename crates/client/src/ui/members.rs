//! Member list surface.

use super::*;
use runecord_protocol::Role;

#[allow(clippy::large_enum_variant)] // a row carries the full member snapshot
pub(crate) enum MemberListRow {
    Header { label: String, key: String },
    Member(Member),
}

pub(crate) fn member_list(s: &App, p: Palette, top_inset: f32) -> El {
    let members = s
        .members
        .get(&s.active_guild)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let state = s.member_state.get(&s.active_guild);
    let mut online: Vec<&Member> = members
        .iter()
        .filter(|m| {
            s.presence.get(&m.user.id).copied().unwrap_or(m.user.status) != Presence::Offline
        })
        .collect();
    let mut offline: Vec<&Member> = members
        .iter()
        .filter(|m| {
            s.presence.get(&m.user.id).copied().unwrap_or(m.user.status) == Presence::Offline
        })
        .collect();
    let rank = |m: &Member| -> i32 {
        s.guild_roles(m.guild_id)
            .iter()
            .filter(|role| m.roles.contains(&role.id))
            .map(|r| r.position)
            .max()
            .unwrap_or(0)
    };
    online.sort_by_key(|m| -rank(m));
    offline.sort_by_key(|m| -rank(m));

    let mut rows: Vec<MemberListRow> = Vec::new();
    let roles = s.guild_roles(s.active_guild);
    let hoisted: Vec<&Role> = roles
        .iter()
        .filter(|role| role.hoist && role.position > 0 && role.name != "@everyone")
        .collect();
    let mut groups: Vec<(&Role, Vec<&Member>)> =
        hoisted.iter().map(|role| (*role, Vec::new())).collect();
    let mut rest: Vec<&Member> = Vec::new();
    for member in online {
        let top = hoisted
            .iter()
            .filter(|role| member.roles.contains(&role.id))
            .max_by_key(|role| role.position);
        match top {
            Some(role) => {
                if let Some((_, list)) = groups.iter_mut().find(|(item, _)| item.id == role.id) {
                    list.push(member);
                } else {
                    rest.push(member);
                }
            }
            None => rest.push(member),
        }
    }
    for (role, members) in groups {
        if members.is_empty() {
            continue;
        }
        let key = format!("role-{}", role.id);
        rows.push(MemberListRow::Header {
            label: format!("{} — {}", role.name.to_uppercase(), members.len()),
            key: key.clone(),
        });
        if !s.member_groups_collapsed.contains(&key) {
            rows.extend(
                members
                    .into_iter()
                    .map(|m| MemberListRow::Member(m.clone())),
            );
        }
    }
    if !rest.is_empty() {
        rows.push(MemberListRow::Header {
            label: format!("ONLINE — {}", rest.len()),
            key: "online".into(),
        });
        if !s.member_groups_collapsed.contains("online") {
            rows.extend(rest.into_iter().map(|m| MemberListRow::Member(m.clone())));
        }
    }
    if !offline.is_empty() {
        rows.push(MemberListRow::Header {
            label: format!("OFFLINE — {}", offline.len()),
            key: "offline".into(),
        });
        if !s.member_groups_collapsed.contains("offline") {
            rows.extend(
                offline
                    .into_iter()
                    .map(|m| MemberListRow::Member(m.clone())),
            );
        }
    }
    let rows = Rc::new(rows);
    let row_count = rows.len();
    let rows_key = rows.clone();
    let rows_build = rows.clone();

    let mut list = col()
        .key("members-list")
        .w_full()
        .px(10.0)
        .pt(10.0 + top_inset)
        .pb(10.0)
        .gap(0.0);
    if let Some(state) = state {
        let loaded = members.len();
        if state.total > loaded as u32 || !state.query.is_empty() {
            list = list.child(
                el().key("member-count")
                    .w_full()
                    .h(24.0)
                    .px(8.0)
                    .center_y()
                    .child(
                        text(format!("{} of {} members", loaded, state.total))
                            .font(theme::TINY)
                            .color(p.text_faint),
                    ),
            );
        }
    }
    list = list.child(
        col()
            .key("member-rows")
            .w_full()
            .growth(GrowthSpec::default())
            .virtual_rows(
                row_count,
                48.0,
                move |_s: &App, i| {
                    rows_key
                        .get(i)
                        .map(|row| match row {
                            MemberListRow::Header { key, .. } => format!("mh-{key}").into(),
                            MemberListRow::Member(member) => {
                                format!("mm-{}", member.user.id).into()
                            }
                        })
                        .unwrap_or_else(|| format!("m-missing-{i}").into())
                },
                move |s: &App, i| match rows_build.get(i) {
                    Some(MemberListRow::Header { label, key }) => {
                        member_group_label(s, label, key, p)
                    }
                    Some(MemberListRow::Member(member)) => member_row(s, member, p),
                    None => el(),
                },
            )
            .on_visible_rows(move |s: &mut App, range| {
                if range.end.saturating_add(8) >= row_count {
                    s.load_more_members();
                }
                false
            }),
    );
    if let Some(state) = state {
        if state.loading {
            if members.is_empty() {
                list = list.child(skeleton_member_rows(8, p, s.clock));
            } else {
                let kit = rune_kit::Kit::neutral(p.dark);
                list = list.child(
                    row()
                        .key("members-loading")
                        .w_full()
                        .h(32.0)
                        .gap(8.0)
                        .center()
                        .child(rune_kit::Spinner::new().size(14.0).build(&kit))
                        .child(
                            text("Loading members…")
                                .font(theme::SMALL)
                                .color(p.text_faint),
                        ),
                );
            }
        } else if state.next_offset.is_some() {
            list = list.child(
                row()
                    .key("members-more")
                    .w_full()
                    .h(34.0)
                    .px(8.0)
                    .center_y()
                    .hover(p.sidebar_hover)
                    .on_press(|s: &mut App| s.load_more_members())
                    .child(
                        text("Load more members")
                            .font(theme::SMALL)
                            .color(p.text_link),
                    ),
            );
        }
    }

    col()
        .key("members-scroll")
        .w(s.members_width)
        .h_full()
        .fill(p.sidebar)
        .scroll(ScrollSpec {
            stick_bottom: false,
        })
        .child(list)
}

fn member_group_label(s: &App, label: &str, key: &str, p: Palette) -> El {
    let collapsed = s.member_groups_collapsed.contains(key);
    let group = key.to_string();
    row()
        .key(format!("member-group-{key}"))
        .w_full()
        .h(32.0)
        .px(8.0)
        .gap(4.0)
        .center_y()
        .role("button")
        .label(format!(
            "{label}, {}",
            if collapsed { "collapsed" } else { "expanded" }
        ))
        .expanded(!collapsed)
        .hover(p.sidebar_hover)
        .on_press(move |s: &mut App| {
            if !s.member_groups_collapsed.remove(&group) {
                s.member_groups_collapsed.insert(group.clone());
            }
        })
        .child(
            icon(&icons::CHEVRON_DOWN, 12.0)
                .color(p.text_muted)
                .icon_rotation(if collapsed {
                    -std::f32::consts::FRAC_PI_2
                } else {
                    0.0
                }),
        )
        .child(
            text(label.to_string())
                .font(theme::SECTION)
                .color(p.text_muted)
                .truncate(),
        )
}

pub(crate) fn member_row(s: &App, m: &Member, p: Palette) -> El {
    let uid = m.user.id;
    let status = s.presence.get(&m.user.id).copied().unwrap_or(m.user.status);
    let offline = status == Presence::Offline;
    let color = if m.user.id == s.me.id {
        p.text_bright
    } else {
        s.member_color(s.active_guild, m.user.id)
            .map(|c| theme::color(&format!("#{:06x}", c)))
            .unwrap_or(p.text_bright)
    };
    let mut body = col().grow(1.0).min_w(0.0).gap(0.0);
    let role_icon = s
        .guild_roles(s.active_guild)
        .iter()
        .filter(|role| m.roles.contains(&role.id) && !role.icon_url.is_empty())
        .max_by_key(|role| role.position)
        .and_then(|role| s.images.get(&role.icon_url))
        .map(|pixels| {
            el().size(14.0, 14.0)
                .radius(3.0)
                .clip_rounded(3.0)
                .pixels_fit(pixels, rune::SurfaceFit::Fill)
        });
    let timed_out = m.timeout_until > now_ms();
    body = body.child(
        row()
            .gap(4.0)
            .center_y()
            .child(
                text(m.user.display_name.clone())
                    .font(theme::SMALL_MED)
                    .color(if offline { p.text_faint } else { color })
                    .truncate(),
            )
            .children(role_icon)
            .children(m.user.is_bot.then(|| bot_tag(p)))
            .children(
                timed_out.then(|| icon(&icons::CLOCK, 14.0).color(p.danger).help("Timed out")),
            ),
    );
    if !m.user.custom_status.is_empty() {
        body = body.child(
            text(m.user.custom_status.clone())
                .font(theme::TINY)
                .color(p.text_muted)
                .truncate(),
        );
    }
    row()
        .key(format!("member-{}", m.user.id))
        .w_full()
        .h(46.0)
        .px(10.0)
        .radius(5.0)
        .gap(12.0)
        .center_y()
        .role("button")
        .label(format!("{} — {}", m.user.display_name, status.label()))
        .motion(motion::SNAPPY)
        .opacity(if offline { 0.55 } else { 1.0 })
        .hover(p.sidebar_hover)
        .on_secondary_press(move |s: &mut App, press, _: &mut rune::Tx| {
            s.context_menu = Some(crate::app::ContextMenu {
                kind: crate::app::ContextMenuKind::Member(uid),
                anchor: crate::app::ContextMenuAnchor::Pointer {
                    x: press.x,
                    y: press.y,
                },
            });
        })
        .on_press(move |s: &mut App| s.modal = Modal::Profile(uid))
        .children({
            let avatar = avatar_status(&m.user, 32.0, p, p.sidebar, true);
            [if crate::voice::user_speaking(uid) {
                avatar.border(2.0, p.green)
            } else {
                avatar
            }]
        })
        .child(body)
}

// ---------------------------------------------------------------------------
// DMs
// ---------------------------------------------------------------------------
