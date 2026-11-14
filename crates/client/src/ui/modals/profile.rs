//! Profile and message menus.

use super::super::prelude::*;
use super::super::*;

pub(crate) fn profile_sheet(s: &App, p: Palette, id: runecord_protocol::Id) -> El {
    profile_card(s, p, id, false)
}

pub(crate) fn full_profile_sheet(s: &App, p: Palette, id: runecord_protocol::Id) -> El {
    profile_card(s, p, id, true)
}

fn profile_card(s: &App, p: Palette, id: runecord_protocol::Id, full: bool) -> El {
    let user = s.user(id).cloned().unwrap_or_default();
    let member = s
        .members
        .get(&s.active_guild)
        .and_then(|members| members.iter().find(|member| member.user.id == id))
        .cloned();
    let relationship = s
        .relationships
        .iter()
        .find(|relationship| relationship.user.id == id)
        .map(|relationship| relationship.kind);
    let is_friend = relationship == Some(RelationshipKind::Friend);
    let pending = matches!(
        relationship,
        Some(RelationshipKind::Incoming) | Some(RelationshipKind::Outgoing)
    );
    let role = if s.screen == Screen::Guild {
        s.member_color(s.active_guild, id)
            .map(|c| theme::color(&format!("#{:06x}", c)))
    } else {
        None
    };
    let banner_top = if user.banner_color != 0 {
        theme::profile_banner_color(user.banner_color)
    } else {
        role.unwrap_or(p.accent)
    };
    let banner_bottom = role.unwrap_or_else(|| theme::avatar_color(user.avatar_color));
    let card_color = theme::color("#232428");
    let banner_pixel_height = if full { 140.0 } else { 110.0 };
    let banner_el = if let Some(pixels) = s
        .images
        .get(&user.banner_url)
        .filter(|_| !user.banner_url.is_empty())
    {
        el().w_full()
            .h(banner_pixel_height)
            .clip_rounded(0.0)
            .pixels_fit(pixels, rune::SurfaceFit::Fill)
    } else {
        el().w_full()
            .h(banner_pixel_height)
            .fill(banner_top)
            .gradient(banner_top, banner_bottom)
    };
    col()
        .key("profile-sheet")
        .w(if full { 560.0 } else { 340.0 })
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(card_color)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(banner_el)
        .child(
            col()
                .w_full()
                .px(16.0)
                .pb(12.0)
                .gap(6.0)
                .child(
                    row()
                        .w_full()
                        .px(2.0)
                        .mt(if full { -56.0 } else { -39.0 })
                        .gap(10.0)
                        .items_center()
                        .child(profile_avatar(
                            s,
                            &user,
                            if full { 96.0 } else { 80.0 },
                            p,
                            card_color,
                        ))
                        .children((!user.custom_status.is_empty()).then(|| {
                            el().h(26.0)
                                .px(10.0)
                                .radius(13.0)
                                .fill(p.rail)
                                .center_y()
                                .max_w(if full { 300.0 } else { 190.0 })
                                .child(
                                    text(user.custom_status.clone())
                                        .font(theme::TINY)
                                        .color(p.text_bright)
                                        .truncate(),
                                )
                        })),
                )
                .child(
                    row()
                        .gap(6.0)
                        .center_y()
                        .child(profile_name(s, &user, p))
                        .children(user.is_bot.then(|| bot_tag(p))),
                )
                .child(
                    row()
                        .w_full()
                        .gap(6.0)
                        .center_y()
                        .child(text(user.handle()).font(theme::SMALL).color(p.text_muted))
                        .children((!user.pronouns.is_empty()).then(|| {
                            text(format!("({})", user.pronouns))
                                .font(theme::TINY)
                                .color(p.text_faint)
                        }))
                        .child(badges_row(&user.badges, 16.0, p, 6)),
                )
                .children(
                    member
                        .as_ref()
                        .filter(|member| {
                            !member.nickname.is_empty() && member.nickname != user.display_name
                        })
                        .map(|member| {
                            text(format!("a.k.a. {}", member.nickname))
                                .font(theme::TINY)
                                .color(p.text_faint)
                        }),
                )
                .children(
                    (!user.bio.is_empty()).then(|| el().w_full().h(1.0).fill(p.divider).my(4.0)),
                )
                .children((!user.bio.is_empty()).then(|| {
                    col()
                        .w_full()
                        .gap(6.0)
                        .child(text("ABOUT ME").font(theme::SECTION).color(p.text_muted))
                        .child(text(user.bio.clone()).font(theme::SMALL).color(p.text))
                }))
                .children((!full && id != s.me.id).then(|| {
                    text("View Full Profile")
                        .key("profile-full")
                        .font(theme::TINY)
                        .color(p.text_link)
                        .on_press(move |s: &mut App| s.modal = Modal::FullProfile(id))
                }))
                .children(activity_card(s, &user, p))
                .children(connections_block(&user.connections, p))
                .children(
                    (full && !user.profile_widget.is_empty() && user.profile_widget != "none")
                        .then(|| profile_widget(s, &user, member.as_ref(), p)),
                )
                .child(el().w_full().h(1.0).fill(p.divider))
                .child(
                    text(if user.is_bot {
                        "CREATED"
                    } else {
                        "JOINED DISCORD"
                    })
                    .font(theme::SECTION)
                    .color(p.text_muted),
                )
                .child(
                    text(date_label(user.created_at))
                        .font(theme::SMALL)
                        .color(p.text),
                )
                .child(member_details(s, member.as_ref(), p))
                .child(mutual_servers(s, id, p))
                .child(
                    row()
                        .w_full()
                        .gap(8.0)
                        .child(
                            el().h(38.0)
                                .px(16.0)
                                .radius(4.0)
                                .grow(1.0)
                                .key("profile-message")
                                .role("button")
                                .label(if id == s.me.id {
                                    "Edit Profile"
                                } else {
                                    "Send Message"
                                })
                                .fill(p.accent)
                                .hover(p.accent_hover)
                                .center()
                                .on_press(move |s: &mut App| {
                                    s.modal = Modal::None;
                                    if id == s.me.id {
                                        s.open_settings(crate::app::SettingsTab::Profile);
                                    } else {
                                        s.start_dm(id);
                                    }
                                })
                                .child(
                                    text(if id == s.me.id {
                                        "Edit Profile"
                                    } else {
                                        "Message"
                                    })
                                    .font(theme::SMALL_MED)
                                    .color(Color::WHITE),
                                ),
                        )
                        .children((id != s.me.id && !is_friend && !pending).then(|| {
                            el().h(38.0)
                                .px(16.0)
                                .radius(4.0)
                                .grow(1.0)
                                .key("profile-add-friend")
                                .role("button")
                                .label("Add Friend")
                                .fill(p.green)
                                .hover(p.green.alpha(0.85))
                                .center()
                                .on_press(move |s: &mut App| s.friend_user(id))
                                .child(
                                    text("Add Friend")
                                        .font(theme::SMALL_MED)
                                        .color(Color::WHITE),
                                )
                        }))
                        .children(pending.then(|| {
                            text("Friend request pending")
                                .font(theme::TINY)
                                .color(p.text_muted)
                        }))
                        .children((id != s.me.id).then(|| {
                            el().size(38.0, 38.0)
                                .radius(4.0)
                                .fill(p.rail)
                                .hover(p.rail_hover)
                                .center()
                                .key("profile-more-open")
                                .role("button")
                                .label("More profile actions")
                                .on_press(move |s: &mut App| s.modal = Modal::ProfileMore(id))
                                .child(icon(&icons::MORE, 18.0).color(p.text_bright))
                        })),
                ),
        )
}

fn effect_colors(kind: &str) -> Option<(Color, Color)> {
    match kind {
        "aurora" => Some((theme::color("#57f287"), theme::color("#5865f2"))),
        "ember" => Some((theme::color("#faa61a"), theme::color("#ed4245"))),
        "neon" => Some((theme::color("#eb459e"), theme::color("#00e5ff"))),
        "snow" => Some((theme::color("#b9bbbe"), theme::color("#ffffff"))),
        _ => None,
    }
}

fn nameplate_colors(kind: &str) -> Option<(Color, Color)> {
    match kind {
        "midnight" => Some((theme::color("#232b52"), theme::color("#5865f2"))),
        "aurora" => Some((theme::color("#1db954"), theme::color("#5865f2"))),
        "sunset" => Some((theme::color("#eb459e"), theme::color("#faa61a"))),
        "candy" => Some((theme::color("#ff73fa"), theme::color("#ffb3fd"))),
        _ => None,
    }
}

fn profile_avatar(s: &App, user: &User, size: f32, p: Palette, ring: Color) -> El {
    let Some((first, second)) = effect_colors(&user.profile_effect) else {
        return avatar_status(user, size, p, ring, true);
    };
    let phase = s.clock as f32 * 1.8;
    let radius = size / 2.0 - 3.0;
    let dot = (size * 0.075).max(5.0);
    let mut stack = el()
        .size(size, size)
        .center()
        .child(
            el().absolute(-5.0, -5.0)
                .size(size + 10.0, size + 10.0)
                .radius((size + 10.0) / 2.0)
                .fill(first.alpha(0.22)),
        )
        .child(avatar_status(user, size, p, ring, true));
    for index in 0..3 {
        let angle = phase + index as f32 * std::f32::consts::TAU / 3.0;
        let x = size / 2.0 + angle.cos() * radius - dot / 2.0;
        let y = size / 2.0 + angle.sin() * radius - dot / 2.0;
        let tint = if index % 2 == 0 { first } else { second };
        let pulse = 0.55 + 0.35 * (phase * 1.3 + index as f32).sin().abs();
        stack = stack.child(
            el().absolute(x, y)
                .size(dot, dot)
                .radius(dot / 2.0)
                .fill(tint.alpha(pulse))
                .border(1.0, p.modal),
        );
    }
    stack
}

fn profile_name(s: &App, user: &User, p: Palette) -> El {
    let Some((first, second)) = nameplate_colors(&user.nameplate) else {
        return text(user.display_name.clone())
            .font(theme::TITLE)
            .color(p.text_bright);
    };
    let pulse = 0.82 + 0.18 * (s.clock as f32 * 1.2).sin().abs();
    row()
        .h(32.0)
        .px(10.0)
        .radius(6.0)
        .gap(6.0)
        .center_y()
        .gradient(first.alpha(pulse), second)
        .child(
            text(user.display_name.clone())
                .font(theme::TITLE)
                .color(Color::WHITE),
        )
}

fn profile_widget(
    s: &App,
    user: &User,
    member: Option<&runecord_protocol::Member>,
    p: Palette,
) -> El {
    let stats = user.profile_widget == "stats";
    let body = if stats {
        let roles = member
            .map(|member| member.roles.iter().filter(|id| **id != 0).count())
            .unwrap_or(0);
        let mutuals = s.mutuals.get(&user.id).map(Vec::len).unwrap_or(0);
        row().w_full().gap(18.0).children(
            [
                ("ROLES", roles.to_string()),
                ("MUTUAL SERVERS", mutuals.to_string()),
                ("MEMBER SINCE", date_label(user.created_at)),
            ]
            .into_iter()
            .map(|(label, value)| {
                col()
                    .gap(2.0)
                    .child(text(label).font(theme::TINY).color(p.text_muted))
                    .child(text(value).font(theme::SMALL_MED).color(p.text_bright))
            }),
        )
    } else {
        col()
            .w_full()
            .gap(2.0)
            .child(
                text(
                    user.activity
                        .as_ref()
                        .map(|activity| activity.title.clone())
                        .unwrap_or_else(|| "Nothing playing".into()),
                )
                .font(theme::SMALL_MED)
                .color(p.text_bright),
            )
            .child(
                text(
                    user.activity
                        .as_ref()
                        .map(|activity| activity.subtitle.clone())
                        .unwrap_or_else(|| "Set an activity in Profile settings".into()),
                )
                .font(theme::TINY)
                .color(p.text_muted),
            )
    };
    col().key("profile-widget").w_full().gap(0.0).child(
        col()
            .w_full()
            .radius(6.0)
            .fill(p.card)
            .padding(8.0)
            .gap(6.0)
            .child(
                text(if stats { "AT A GLANCE" } else { "NOW PLAYING" })
                    .font(theme::TINY)
                    .color(p.accent),
            )
            .child(body),
    )
}

fn activity_card(s: &App, user: &User, p: Palette) -> Option<El> {
    let activity = user.activity.as_ref()?;
    let spotify = activity.kind.eq_ignore_ascii_case("spotify");
    let accent = if spotify {
        theme::color("#1db954")
    } else {
        p.green
    };
    let label = if spotify {
        "LISTENING TO SPOTIFY"
    } else {
        "PLAYING A GAME"
    };
    let art = if let Some(pixels) = s
        .images
        .get(&activity.artwork_url)
        .filter(|_| !activity.artwork_url.is_empty())
    {
        el().size(58.0, 58.0)
            .radius(6.0)
            .clip_rounded(6.0)
            .pixels_fit(pixels, rune::SurfaceFit::Fill)
    } else {
        el().size(58.0, 58.0)
            .radius(6.0)
            .fill(p.accent)
            .gradient(accent, theme::avatar_color(user.avatar_color))
            .center()
            .child(icon(&icons::HEADPHONES, 24.0).color(Color::WHITE))
    };
    let mut body = col().grow(1.0).min_w(0.0).gap(1.0).child(
        text(activity.title.clone())
            .font(theme::SMALL_MED)
            .color(p.text_bright)
            .truncate(),
    );
    if !activity.subtitle.is_empty() {
        body = body.child(
            text(activity.subtitle.clone())
                .font(theme::TINY)
                .color(p.text)
                .truncate(),
        );
    }
    if !activity.state.is_empty() {
        body = body.child(
            text(activity.state.clone())
                .font(theme::TINY)
                .color(p.text_muted)
                .truncate(),
        );
    }
    if !activity.url.is_empty() {
        let url = activity.url.clone();
        body = body.child(
            el().key("profile-activity-open")
                .h(24.0)
                .px(8.0)
                .radius(12.0)
                .center()
                .role("button")
                .label(if spotify {
                    "Play on Spotify"
                } else {
                    "Open activity"
                })
                .fill(accent)
                .hover(accent.alpha(0.85))
                .on_press_tx(move |_: &mut App, tx: &mut rune::Tx| {
                    let _ = tx.open_url(&url);
                })
                .child(
                    text(if spotify { "Play on Spotify" } else { "Open" })
                        .font(theme::TINY)
                        .color(Color::WHITE),
                ),
        );
    }
    Some(
        col()
            .key("profile-activity")
            .w_full()
            .gap(4.0)
            .child(
                row()
                    .w_full()
                    .center_y()
                    .gap(6.0)
                    .child(text(label).font(theme::TINY).color(accent).grow(1.0))
                    .children(spotify.then(|| {
                        el().size(18.0, 18.0)
                            .radius(9.0)
                            .fill(accent)
                            .center()
                            .child(icon(&icons::HEADPHONES, 11.0).color(Color::WHITE))
                    })),
            )
            .child(
                row()
                    .w_full()
                    .radius(6.0)
                    .fill(p.card)
                    .padding(8.0)
                    .gap(10.0)
                    .center_y()
                    .child(art)
                    .child(body),
            ),
    )
}

fn connections_block(
    connections: &[runecord_protocol::ProfileConnection],
    p: Palette,
) -> Option<El> {
    if connections.is_empty() {
        return None;
    }
    let mut chips = row().w_full().gap(6.0).flex_wrap();
    for connection in connections {
        let tint = match connection.kind.as_str() {
            "github" => theme::color("#c9d1d9"),
            "steam" => theme::color("#66c0f4"),
            "spotify" => theme::color("#1db954"),
            _ => p.text_muted,
        };
        chips = chips.child(
            row()
                .h(26.0)
                .px(8.0)
                .radius(13.0)
                .gap(6.0)
                .center_y()
                .fill(p.rail)
                .child(
                    el().size(16.0, 16.0)
                        .radius(8.0)
                        .fill(tint.alpha(0.22))
                        .center()
                        .child(icon(&icons::LINK, 10.0).color(tint)),
                )
                .child(
                    text(connection.kind.to_uppercase())
                        .font(theme::TINY)
                        .color(tint),
                )
                .child(
                    text(connection.name.clone())
                        .font(theme::TINY)
                        .color(p.text_bright)
                        .truncate(),
                ),
        );
    }
    Some(
        col()
            .w_full()
            .gap(5.0)
            .child(text("CONNECTIONS").font(theme::SECTION).color(p.text_muted))
            .child(chips),
    )
}

fn mutual_servers(s: &App, user: Id, p: Palette) -> El {
    if user == s.me.id {
        return el();
    }
    let Some(guilds) = s.mutuals.get(&user) else {
        return el();
    };
    if guilds.is_empty() {
        return el();
    }
    let mut rows = col().w_full().gap(2.0);
    for guild in guilds.iter().take(3) {
        rows = rows.child(
            row()
                .w_full()
                .h(24.0)
                .gap(6.0)
                .center_y()
                .child(guild_icon(s, guild, 20.0, 6.0, p))
                .child(
                    text(guild.name.clone())
                        .font(theme::TINY)
                        .color(p.text_muted)
                        .truncate(),
                ),
        );
    }
    col()
        .w_full()
        .gap(4.0)
        .child(
            text("MUTUAL SERVERS")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(rows)
}

fn member_details(s: &App, member: Option<&runecord_protocol::Member>, p: Palette) -> El {
    let Some(member) = member else {
        return el();
    };
    let roles: Vec<&runecord_protocol::Role> = s
        .guild_roles(s.active_guild)
        .iter()
        .filter(|role| role.name != "@everyone" && member.roles.contains(&role.id))
        .collect();
    let mut details = col()
        .w_full()
        .gap(6.0)
        .child(el().w_full().h(1.0).fill(p.divider).my(4.0))
        .child(
            text("MEMBER SINCE")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(
            text(date_label(member.joined_at))
                .font(theme::SMALL)
                .color(p.text),
        );
    if member.timeout_until > now_ms() {
        details = details
            .child(text("TIMED OUT UNTIL").font(theme::SECTION).color(p.danger))
            .child(
                text(expiry_label(member.timeout_until))
                    .font(theme::SMALL)
                    .color(p.text),
            );
    }
    if !roles.is_empty() {
        let mut chips = row().w_full().gap(4.0).flex_wrap();
        for role in roles {
            chips =
                chips.child(
                    row()
                        .h(22.0)
                        .px(6.0)
                        .radius(4.0)
                        .gap(4.0)
                        .center_y()
                        .fill(if role.color != 0 {
                            theme::color(&format!("#{:06x}", role.color)).alpha(0.18)
                        } else {
                            p.rail
                        })
                        .child(
                            if let Some(pixels) = s
                                .images
                                .get(&role.icon_url)
                                .filter(|_| !role.icon_url.is_empty())
                            {
                                el().size(14.0, 14.0)
                                    .radius(3.0)
                                    .clip_rounded(3.0)
                                    .pixels_fit(pixels, rune::SurfaceFit::Fill)
                            } else {
                                el().size(8.0, 8.0)
                                    .radius(4.0)
                                    .fill(theme::color(&format!("#{:06x}", role.color)))
                            },
                        )
                        .child(text(role.name.clone()).font(theme::TINY).color(
                            if role.color != 0 {
                                theme::color(&format!("#{:06x}", role.color))
                            } else {
                                p.text_bright
                            },
                        )),
                );
        }
        details = details
            .child(text("ROLES").font(theme::SECTION).color(p.text_muted))
            .child(chips);
    }
    details
}

pub(crate) fn message_menu(s: &App, p: Palette, id: runecord_protocol::Id) -> El {
    let Some(msg) = s.messages.values().flatten().find(|m| m.id == id).cloned() else {
        return el();
    };
    let mine = msg.author_id == s.me.id;
    let perms = s
        .channel(msg.channel_id)
        .map(|c| c.my_permissions)
        .unwrap_or(0);
    let can_send = has(perms, perm::SEND_MESSAGES);
    let can_manage = has(perms, perm::MANAGE_MESSAGES);
    let can_react = has(perms, perm::ADD_REACTIONS);
    let mid = id;
    let author_id = msg.author_id;
    let channel_id = msg.channel_id;
    let content = msg.content.clone();
    let content_reply = content.clone();
    let content_copy = content.clone();
    let content_edit = content.clone();
    let thread_preview = content.clone();
    let pinned = msg.pinned;

    let item = |label: &'static str, red: bool, key: &'static str, p: Palette| {
        el().key(key)
            .w_full()
            .h(32.0)
            .px(8.0)
            .radius(4.0)
            .center_y()
            .hover(if red { p.danger.alpha(0.15) } else { p.accent })
            .child(text(label).font(theme::SMALL_MED).color(if red {
                p.danger
            } else {
                p.text_bright
            }))
    };

    let mut reactions = row().gap(2.0).px(4.0).py(2.0);
    for emoji in ["≡ƒöÑ", "≡ƒÿé", "≡ƒÆ£", "≡ƒæì", "≡ƒæÇ", "≡ƒÄë"] {
        reactions = reactions.child(
            text(emoji)
                .font(TextStyle::new(20.0, 400.0))
                .padding(4.0)
                .radius(4.0)
                .hover(p.selected)
                .on_press(move |s: &mut App| {
                    s.react(mid, emoji);
                    s.modal = Modal::None;
                }),
        );
    }

    el().key("message-menu")
        .w(220.0)
        .radius(8.0)
        .fill(theme::color("#111214"))
        .padding(6.0)
        .gap(2.0)
        .shadow(8.0, 24.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .children(can_react.then(|| reactions))
        .child(el().w_full().h(1.0).fill(p.divider).my(3.0))
        .children(can_send.then(|| {
            item("Reply", false, "menu-reply", p).on_press(move |s: &mut App| {
                s.set_reply(mid, author_id, &content_reply);
                s.modal = Modal::None;
            })
        }))
        .children(can_send.then(|| {
            item("Create Thread", false, "menu-thread", p).on_press(move |s: &mut App| {
                s.profile_edit = rune::TextEdit::new(plain_thread_name(&thread_preview));
                s.modal = Modal::ThreadCreate {
                    parent: channel_id,
                    message: Some(mid),
                };
            })
        }))
        .child(item("Copy Text", false, "menu-copy", p).on_press_tx(
            move |_: &mut App, tx: &mut rune::Tx| {
                tx.copy(content_copy.clone());
            },
        ))
        .children(can_manage.then(|| {
            item(
                if pinned {
                    "Unpin Message"
                } else {
                    "Pin Message"
                },
                false,
                "menu-pin",
                p,
            )
            .on_press(move |s: &mut App| {
                s.toggle_pin(mid);
                s.modal = Modal::None;
            })
        }))
        .children(mine.then(|| {
            item("Edit Message", false, "menu-edit", p)
                .on_press(move |s: &mut App| s.open_edit(mid, &content_edit))
        }))
        .children((mine || can_manage).then(|| {
            item("Delete Message", true, "menu-delete", p).on_press(move |s: &mut App| {
                s.delete_message(mid);
                s.modal = Modal::None;
            })
        }))
}

pub(crate) fn edit_sheet(s: &App, p: Palette, _id: runecord_protocol::Id) -> El {
    col()
        .key("edit-sheet")
        .w(520.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .h(56.0)
                .px(20.0)
                .items_center()
                .child(
                    text("Edit Message")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            col()
                .w_full()
                .px(20.0)
                .pb(20.0)
                .gap(12.0)
                .child(
                    text_input(&s.edit_text, theme::BODY)
                        .key("edit-input")
                        .role("textbox")
                        .label("Edit message")
                        .w_full()
                        .min_h(96.0)
                        .radius(4.0)
                        .fill(p.rail)
                        .color(p.text)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.edit_text)
                        .on_submit(|s: &mut App, _: &mut rune::Tx| s.save_edit()),
                )
                .child(
                    row()
                        .w_full()
                        .justify_end()
                        .gap(8.0)
                        .child(
                            text("Cancel")
                                .font(theme::SMALL_MED)
                                .color(p.text_muted)
                                .padding(10.0)
                                .on_press(|s: &mut App| s.modal = Modal::None),
                        )
                        .child(
                            el().h(38.0)
                                .px(18.0)
                                .radius(4.0)
                                .fill(p.accent)
                                .hover(p.accent_hover)
                                .center()
                                .on_press(|s: &mut App| s.save_edit())
                                .child(text("Save").font(theme::SMALL_MED).color(Color::WHITE)),
                        ),
                ),
        )
}
