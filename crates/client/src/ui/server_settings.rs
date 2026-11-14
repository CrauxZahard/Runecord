//! Server settings: role CRUD/reordering and member role assignment.

use super::prelude::*;
use super::*;
use runecord_protocol::{AuditEntry, Role, RoleLink};

pub(crate) fn server_settings_page(s: &App, cx: &mut Cx<App>, p: Palette) -> El {
    let trailing = cx.chrome_insets().trailing;
    let content = match s.server_tab {
        crate::app::ServerTab::Overview => overview_view(s, p),
        crate::app::ServerTab::Bans => bans_view(s, p),
        crate::app::ServerTab::Roles => roles_view(s, p),
        crate::app::ServerTab::Members => members_view(s, p),
        crate::app::ServerTab::Emojis => emojis_view(s, p),
        crate::app::ServerTab::Audit => audit_view(s, p),
    };
    let title = match s.server_tab {
        crate::app::ServerTab::Overview => "Server Overview",
        crate::app::ServerTab::Bans => "Bans",
        crate::app::ServerTab::Roles => "Roles",
        crate::app::ServerTab::Members => "Members",
        crate::app::ServerTab::Emojis => "Emojis",
        crate::app::ServerTab::Audit => "Audit Log",
    };

    let nav = server_settings_nav(s, p);
    col()
        .key("server-settings-page")
        .w_full()
        .h_full()
        .fill(p.chat)
        .child(
            row().w_full().h_full().child(nav).child(
                col()
                    .grow(1.0)
                    .min_w(0.0)
                    .h_full()
                    .child(
                        row()
                            .key("server-settings-top")
                            .w_full()
                            .h(64.0)
                            .pl(32.0)
                            .pr(12.0 + trailing)
                            .gap(8.0)
                            .items_center()
                            .window_drag()
                            .child(
                                text(title)
                                    .font(theme::TITLE)
                                    .color(p.text_bright)
                                    .grow(1.0),
                            )
                            .children(
                                (s.server_tab == crate::app::ServerTab::Members
                                    && s.guild_can(perm::KICK_MEMBERS))
                                .then(|| {
                                    el().key("prune-open")
                                        .h(30.0)
                                        .px(12.0)
                                        .radius(4.0)
                                        .center()
                                        .role("button")
                                        .label("Prune Members")
                                        .fill(p.rail)
                                        .hover(p.rail_hover)
                                        .on_press(|s: &mut App| s.open_prune())
                                        .child(
                                            text("Prune Members")
                                                .font(theme::SMALL_MED)
                                                .color(p.text_bright),
                                        )
                                }),
                            )
                            .children((s.server_tab == crate::app::ServerTab::Members).then(|| {
                                el().key("bans-open")
                                    .h(30.0)
                                    .px(12.0)
                                    .radius(4.0)
                                    .center()
                                    .role("button")
                                    .label("Banned Users")
                                    .fill(p.rail)
                                    .hover(p.rail_hover)
                                    .on_press(|s: &mut App| s.open_bans())
                                    .child(
                                        text("Banned Users")
                                            .font(theme::SMALL_MED)
                                            .color(p.text_bright),
                                    )
                            }))
                            .child(close_button(p)),
                    )
                    .child(
                        col()
                            .w_full()
                            .grow(1.0)
                            .min_h(0.0)
                            .px(32.0)
                            .pb(32.0)
                            .scroll(ScrollSpec {
                                stick_bottom: false,
                            })
                            .child(content),
                    )
                    .children(
                        (s.server_tab == crate::app::ServerTab::Overview)
                            .then(|| overview_footer(s, p)),
                    ),
            ),
        )
}

fn overview_footer(s: &App, p: Palette) -> El {
    let dirty = s.overview_dirty();
    row()
        .key("overview-footer")
        .w_full()
        .h(64.0)
        .px(32.0)
        .gap(10.0)
        .center_y()
        .fill(p.modal_sidebar)
        .border_sides([1.0, 0.0, 0.0, 0.0], p.divider)
        .child(
            el().key("overview-reset")
                .h(34.0)
                .px(12.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Reset changes")
                .hover(p.card_hover)
                .on_press(|s: &mut App| s.sync_server_overview())
                .child(text("Reset").font(theme::SMALL_MED).color(p.text_muted)),
        )
        .child(el().grow(1.0))
        .children(dirty.then(|| {
            text("Careful — you have unsaved changes.")
                .font(theme::TINY)
                .color(p.text_faint)
        }))
        .child(
            el().key("overview-save")
                .h(34.0)
                .px(16.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Save Changes")
                .fill(if dirty { p.green } else { p.rail })
                .hover(if dirty { p.green.alpha(0.85) } else { p.rail })
                .on_press(|s: &mut App| {
                    if s.overview_dirty() {
                        s.save_server_overview();
                    }
                })
                .child(text("Save Changes").font(theme::SMALL_MED).color(if dirty {
                    Color::WHITE
                } else {
                    p.text_faint
                })),
        )
}

fn nav_group(label: &'static str, p: Palette) -> El {
    text(label)
        .font(theme::TINY)
        .color(p.text_faint)
        .pt(10.0)
        .pb(2.0)
        .pl(8.0)
}

fn server_settings_nav(s: &App, p: Palette) -> El {
    let guild = s.active_guild().cloned().unwrap_or_default();
    let nav_item = |label: &'static str, tab: crate::app::ServerTab, key: &'static str| {
        let selected = s.server_tab == tab;
        el().key(key)
            .w_full()
            .h(32.0)
            .px(8.0)
            .radius(4.0)
            .center_y()
            .role("tab")
            .label(label)
            .fill(if selected {
                p.selected
            } else {
                Color::TRANSPARENT
            })
            .hover(p.rail)
            .motion(motion::SNAPPY)
            .on_press(move |s: &mut App| s.select_server_tab(tab))
            .child(text(label).font(theme::SMALL_MED).color(if selected {
                p.text_bright
            } else {
                p.text_muted
            }))
    };

    col()
        .key("server-settings-nav")
        .w(232.0)
        .h_full()
        .fill(p.modal_sidebar)
        .px(8.0)
        .pt(16.0)
        .gap(2.0)
        .child(
            row()
                .w_full()
                .h(48.0)
                .px(8.0)
                .gap(10.0)
                .center_y()
                .child(guild_icon(s, &guild, 32.0, 10.0, p))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text(guild.name.clone())
                                .font(theme::SMALL_MED)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(
                            text("Server Settings")
                                .font(theme::TINY)
                                .color(p.text_muted)
                                .truncate(),
                        ),
                ),
        )
        .child(el().w_full().h(1.0).fill(p.divider).my(8.0))
        .child(nav_group("GENERAL", p))
        .child(nav_item(
            "Overview",
            crate::app::ServerTab::Overview,
            "role-tab-overview",
        ))
        .child(nav_item(
            "Roles",
            crate::app::ServerTab::Roles,
            "role-tab-roles",
        ))
        .child(nav_item(
            "Emojis",
            crate::app::ServerTab::Emojis,
            "role-tab-emojis",
        ))
        .child(nav_group("MODERATION", p))
        .child(nav_item(
            "Members",
            crate::app::ServerTab::Members,
            "server-nav-members",
        ))
        .child(nav_item(
            "Bans",
            crate::app::ServerTab::Bans,
            "role-tab-bans",
        ))
        .child(nav_item(
            "Audit Log",
            crate::app::ServerTab::Audit,
            "role-tab-audit",
        ))
        .child(el().grow(1.0))
        .child(el().h(8.0))
}

fn image_section(s: &App, guild: &GuildSummary, banner: bool, p: Palette) -> El {
    let url = if banner {
        guild.banner_url.clone()
    } else {
        guild.icon_url.clone()
    };
    let title = if banner {
        "SERVER BANNER"
    } else {
        "SERVER ICON"
    };
    let locked = banner && guild.boost_level < 2;
    let description = if locked {
        "Unlocks at Boost Level 2 (7 Boosts). Landscape, ideally 16:9."
    } else if banner {
        "Shown at the top of the channel list and on invites. Landscape, ideally 16:9."
    } else {
        "Shown in the server rail, invites and settings. Square, at least 128×128."
    };
    let key = if banner { "guild-banner" } else { "guild-icon" };
    let preview = if banner {
        let inner = if s.images.contains_key(&url) {
            guild_banner_el(s, guild, 225.0, p)
        } else {
            el().w_full()
                .h(225.0)
                .radius(6.0)
                .clip_rounded(6.0)
                .fill(theme::color(&format!("#{:06x}", guild.icon_color)).alpha(0.35))
                .center()
                .child(
                    text("No banner uploaded")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                )
        };
        el().w(400.0)
            .h(225.0)
            .radius(6.0)
            .clip_rounded(6.0)
            .child(inner)
    } else {
        el().size(128.0, 128.0)
            .radius(24.0)
            .clip_rounded(24.0)
            .child(guild_icon(s, guild, 128.0, 24.0, p))
    };

    let mut actions = col().gap(6.0).justify_center();
    let upload_label = if locked {
        "Level 2 Required"
    } else if banner {
        "Upload Banner"
    } else {
        "Upload Icon"
    };
    actions = actions.child(
        el().key(format!("{key}-upload"))
            .h(32.0)
            .px(12.0)
            .radius(4.0)
            .center()
            .role("button")
            .label(if locked {
                "Server banner locked until Level 2"
            } else if banner {
                "Upload banner"
            } else {
                "Upload icon"
            })
            .fill(if locked { p.rail } else { p.accent })
            .hover(if locked { p.rail } else { p.accent_hover })
            .on_press_tx(move |_s: &mut App, tx: &mut rune::Tx| {
                if locked {
                    return;
                }
                let options = rune::FileDialogOptions {
                    title: if banner {
                        "Upload server banner".to_string()
                    } else {
                        "Upload server icon".to_string()
                    },
                    directory: None,
                    file_name: None,
                    filters: Vec::new(),
                }
                .filter("Images", &["png", "jpg", "jpeg", "gif", "webp"]);
                tx.open_file(options, move |s: &mut App, result, _| {
                    if let Ok(Some(path)) = result {
                        s.begin_guild_image_crop(path, banner);
                    }
                });
            })
            .child(text(upload_label).font(theme::SMALL_MED).color(if locked {
                p.text_faint
            } else {
                Color::WHITE
            })),
    );
    if !url.is_empty() {
        actions = actions.child(
            el().key(format!("{key}-remove"))
                .h(30.0)
                .px(10.0)
                .radius(4.0)
                .center()
                .role("button")
                .label(if banner {
                    "Remove banner"
                } else {
                    "Remove icon"
                })
                .hover(p.danger.alpha(0.2))
                .on_press(move |s: &mut App| s.remove_guild_image(banner))
                .child(text("Remove").font(theme::LABEL).color(p.danger)),
        );
    }
    actions = actions.child(
        text(if locked {
            "Unlocks at Level 2"
        } else {
            "or drop an image here"
        })
        .font(theme::TINY)
        .color(p.text_faint),
    );

    col()
        .key(format!("{key}-section"))
        .w_full()
        .gap(8.0)
        .child(text(title).font(theme::SECTION).color(p.text_muted))
        .child(
            row()
                .w_full()
                .gap(14.0)
                .center_y()
                .role("group")
                .label(title)
                .on_drop_files(move |s: &mut App, files, _| {
                    if let Some(file) = files.first() {
                        if locked {
                            s.toast("Server banners unlock at Boost Level 2 (7 Boosts)");
                        } else {
                            s.begin_guild_image_crop(file.clone(), banner);
                        }
                    }
                })
                .accepts_extensions(&["png", "jpg", "jpeg", "gif", "webp"])
                .child(preview)
                .child(actions),
        )
        .child(text(description).font(theme::TINY).color(p.text_faint))
        .child(el().w_full().h(1.0).fill(p.divider).my(2.0))
}

fn overview_view(s: &App, p: Palette) -> El {
    let guild = s.active_guild().cloned().unwrap_or_default();
    let colors = [
        0x5865f2u32,
        0xeb459e,
        0x23a55a,
        0xf0b232,
        0xed4245,
        0x9b59b6,
        0x1abc9c,
        0x2f3136,
    ];
    let mut swatches = row().w_full().gap(8.0).flex_wrap();
    for color in colors {
        let selected = s.overview_color == color;
        swatches = swatches.child(
            el().size(30.0, 30.0)
                .radius(15.0)
                .key(format!("overview-color-{:06x}", color))
                .role("radio")
                .label(format!("Icon colour #{:06x}", color))
                .checked(selected)
                .fill(theme::color(&format!("#{:06x}", color)))
                .border(if selected { 2.0 } else { 0.0 }, p.text_bright)
                .hover(p.text_muted)
                .on_press(move |s: &mut App| s.overview_color = color),
        );
    }

    let field = |label: &'static str| text(label).font(theme::SECTION).color(p.text_muted).pt(6.0);

    let boosts = guild.boosts;
    let level = guild.boost_level;
    let boost_pink = theme::color("#ff73fa");
    let (prev_level, next_level) = match level {
        0 => (0, 2),
        1 => (2, 7),
        2 => (7, 14),
        _ => (14, 14),
    };
    let ratio = if level >= 3 || next_level <= prev_level {
        1.0
    } else {
        ((boosts.saturating_sub(prev_level)) as f32 / (next_level - prev_level) as f32)
            .clamp(0.0, 1.0)
    };
    let next_kbps = match level {
        0 => 128,
        1 => 256,
        2 => 384,
        _ => 384,
    };
    let next_mb = match level {
        0 => 64,
        1 => 96,
        2 => 128,
        _ => 128,
    };
    let current_mb = guild.upload_limit_bytes / (1024 * 1024);
    let remaining = next_level.saturating_sub(boosts);
    let perk_row = |locked: bool, label: String, detail: String, p: Palette| {
        row()
            .w_full()
            .min_h(26.0)
            .gap(8.0)
            .center_y()
            .child(if locked {
                icon(&icons::LOCK, 14.0).color(p.text_faint)
            } else {
                icon(&icons::CHECK, 14.0).color(p.green)
            })
            .child(text(label).font(theme::SMALL_MED).color(if locked {
                p.text_faint
            } else {
                p.text_bright
            }))
            .child(
                text(detail)
                    .font(theme::TINY)
                    .color(if locked { p.text_faint } else { p.text_muted })
                    .grow(1.0),
            )
    };
    let mut perk_rows = col()
        .w_full()
        .gap(2.0)
        .child(perk_row(
            false,
            format!("{} kbps voice", guild.voice_kbps),
            "Opus bitrate for everyone in this server".to_string(),
            p,
        ))
        .child(perk_row(
            false,
            format!("{current_mb} MB uploads"),
            "Per-file upload cap".to_string(),
            p,
        ));
    if level < 3 {
        perk_rows = perk_rows
            .child(perk_row(
                true,
                format!("{next_kbps} kbps voice"),
                format!("{remaining} more Boosts for Level {}", level + 1),
                p,
            ))
            .child(perk_row(
                true,
                format!("{next_mb} MB uploads"),
                format!("{remaining} more Boosts for Level {}", level + 1),
                p,
            ));
    }
    let badge = el()
        .h(22.0)
        .px(8.0)
        .radius(11.0)
        .center()
        .fill(if level > 0 { boost_pink } else { p.rail })
        .child(
            text(format!("LEVEL {level}"))
                .font(theme::TINY)
                .color(if level > 0 {
                    Color::WHITE
                } else {
                    p.text_muted
                }),
        );

    col()
        .key("server-overview")
        .w_full()
        .gap(12.0)
        .child(
            col()
                .w_full()
                .radius(8.0)
                .clip_rounded(8.0)
                .fill(p.card)
                .child(
                    row()
                        .w_full()
                        .gap(14.0)
                        .center_y()
                        .padding(14.0)
                        .child(guild_icon(
                            s,
                            &GuildSummary {
                                icon_color: s.overview_color,
                                ..guild.clone()
                            },
                            56.0,
                            16.0,
                            p,
                        ))
                        .child(
                            col()
                                .grow(1.0)
                                .min_w(0.0)
                                .gap(2.0)
                                .child(
                                    text(s.overview_name.text.clone())
                                        .font(theme::TITLE)
                                        .color(p.text_bright)
                                        .truncate(),
                                )
                                .child(
                                    text(if s.overview_tag.text.trim().is_empty() {
                                        "No tag".to_string()
                                    } else {
                                        format!(
                                            "[{}] · {} members",
                                            s.overview_tag.text.trim(),
                                            guild.member_count
                                        )
                                    })
                                    .font(theme::SMALL)
                                    .color(p.text_muted),
                                ),
                        ),
                ),
        )
        .child(image_section(s, &guild, false, p))
        .child(image_section(s, &guild, true, p))
        .child(
            col()
                .key("overview-boosts")
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(14.0)
                .gap(8.0)
                .child(
                    row()
                        .w_full()
                        .center_y()
                        .gap(8.0)
                        .child(icon(&icons::SPARKLES, 20.0).color(boost_pink))
                        .child(
                            text("Server Boosts")
                                .font(theme::TITLE)
                                .color(p.text_bright)
                                .grow(1.0),
                        )
                        .child(badge),
                )
                .child(
                    row()
                        .w_full()
                        .center_y()
                        .gap(8.0)
                        .child(
                            text(format!(
                                "{boosts} {}",
                                if boosts == 1 { "Boost" } else { "Boosts" }
                            ))
                            .font(theme::NAME)
                            .color(p.text_bright)
                            .grow(1.0),
                        )
                        .children((level < 3).then(|| {
                            text(format!("{next_level} for Level {}", level + 1))
                                .font(theme::TINY)
                                .color(p.text_muted)
                        })),
                )
                .child(
                    row()
                        .w_full()
                        .h(8.0)
                        .radius(4.0)
                        .clip_rounded(4.0)
                        .fill(p.input)
                        .child(el().h(8.0).grow(ratio.max(0.001)).fill(boost_pink))
                        .child(el().h(8.0).grow((1.0 - ratio).max(0.001))),
                )
                .child(
                    row()
                        .w_full()
                        .gap(8.0)
                        .children([(1u32, 2u32), (2, 7), (3, 14)].map(|(tier, at)| {
                            let reached = level >= tier;
                            text(format!("Level {tier} · {at}"))
                                .font(theme::TINY)
                                .color(if reached { boost_pink } else { p.text_faint })
                                .grow(1.0)
                        })),
                )
                .child(perk_rows)
                .child(row().w_full().justify_end().child(if guild.boosted_by_me {
                    el().key("overview-unboost")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Remove Boost")
                        .fill(p.rail)
                        .hover(p.danger.alpha(0.2))
                        .on_press(|s: &mut App| s.set_boost(false))
                        .child(text("Remove Boost").font(theme::SMALL_MED).color(p.danger))
                } else {
                    el().key("overview-boost")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Boost Server")
                        .fill(boost_pink)
                        .hover(boost_pink.alpha(0.85))
                        .on_press(|s: &mut App| s.set_boost(true))
                        .child(
                            text("Boost Server")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        )
                })),
        )
        .child(field("SERVER NAME"))
        .child(
            text_input(&s.overview_name, theme::BODY)
                .key("overview-name")
                .w_full()
                .h(40.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Server name")
                .placeholder("Server name", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.overview_name),
        )
        .child(
            text("This is the name people see in your server and invite links.")
                .font(theme::TINY)
                .color(p.text_faint),
        )
        .child(el().w_full().h(1.0).fill(p.divider).my(2.0))
        .child(field("ICON COLOUR"))
        .child(swatches)
        .child(
            text("Shown in the server rail, invites and settings.")
                .font(theme::TINY)
                .color(p.text_faint),
        )
        .child(el().w_full().h(1.0).fill(p.divider).my(2.0))
        .child(field("DESCRIPTION"))
        .child(
            text_input(&s.overview_description, theme::BODY)
                .key("overview-description")
                .w_full()
                .h(84.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Server description")
                .placeholder("Tell people what this server is about", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.overview_description),
        )
        .child(
            text("Tell people what your server is about. Up to 500 characters.")
                .font(theme::TINY)
                .color(p.text_faint),
        )
        .child(el().w_full().h(1.0).fill(p.divider).my(2.0))
        .child(field("TAG"))
        .child(
            row()
                .w_full()
                .gap(10.0)
                .center_y()
                .child(
                    text_input(&s.overview_tag, theme::BODY)
                        .key("overview-tag")
                        .w(140.0)
                        .h(38.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Server tag")
                        .placeholder("RUNE", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.overview_tag),
                )
                .child(
                    text("Up to 8 characters, shown beside member names.")
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        )
        .child(
            text("SERVER DISCOVERY")
                .font(theme::SECTION)
                .color(p.text_muted)
                .pt(6.0),
        )
        .child(
            row()
                .key("overview-discoverable")
                .w_full()
                .min_h(46.0)
                .px(10.0)
                .py(6.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .role("switch")
                .label("Show in Explore Public Servers")
                .checked(s.overview_discoverable)
                .hover(p.card_hover)
                .on_press(|s: &mut App| s.overview_discoverable = !s.overview_discoverable)
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
                        .fill(if s.overview_discoverable {
                            p.green
                        } else {
                            p.input
                        })
                        .child(
                            el().size(18.0, 18.0)
                                .radius(9.0)
                                .fill(Color::WHITE)
                                .ml(if s.overview_discoverable { 19.0 } else { 2.0 })
                                .mt(2.0),
                        ),
                ),
        )
}

fn bans_view(s: &App, p: Palette) -> El {
    col()
        .key("server-bans")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .gap(12.0)
        .child(
            row()
                .w_full()
                .center_y()
                .child(
                    text(format!(
                        "{} banned {}",
                        s.bans.len(),
                        if s.bans.len() == 1 { "user" } else { "users" }
                    ))
                    .font(theme::SECTION)
                    .color(p.text_muted)
                    .grow(1.0),
                )
                .child(
                    text("Banned members cannot rejoin until they are unbanned.")
                        .font(theme::TINY)
                        .color(p.text_faint),
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
                .child(appeals_list(s, p))
                .child(bans_list(s, p)),
        )
}

fn roles_view(s: &App, p: Palette) -> El {
    let mut roles: Vec<Role> = s.guild_roles(s.active_guild).to_vec();
    roles.sort_by_key(|role| std::cmp::Reverse(role.position));
    let mut list = col().key("role-list").w(240.0).gap(4.0);
    for role in &roles {
        let id = role.id;
        let selected = s.selected_role == id;
        list = list.child(
            row()
                .key(format!("role-select-{id}"))
                .w_full()
                .h(42.0)
                .px(10.0)
                .radius(5.0)
                .gap(10.0)
                .center_y()
                .role("button")
                .label(format!("{} role", role.name))
                .fill(if selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_drag(
                    move |s: &mut App, drag: &mut rune::Drag, _: &mut rune::Tx| match drag.phase {
                        rune::Phase::Down => {
                            s.select_role(id);
                            s.role_drag = Some((id, 0));
                        }
                        rune::Phase::Move => {
                            let Some((dragged, applied)) = s.role_drag else {
                                return;
                            };
                            if dragged != id {
                                return;
                            }
                            let target = (drag.dy / 42.0).round() as i32;
                            if target != applied {
                                let direction = if target > applied { 1 } else { -1 };
                                s.move_selected_role(direction);
                                s.role_drag = Some((id, applied + direction));
                            }
                        }
                        rune::Phase::Up => {
                            if drag.cancelled() {
                                if let Some((dragged, applied)) = s.role_drag {
                                    if dragged == id {
                                        let direction = if applied > 0 { -1 } else { 1 };
                                        for _ in 0..applied.abs() {
                                            s.move_selected_role(direction);
                                        }
                                    }
                                }
                            }
                            s.role_drag = None;
                        }
                    },
                )
                .on_press(move |s: &mut App| s.select_role(id))
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
                    el().h(30.0)
                        .px(12.0)
                        .radius(4.0)
                        .key("role-up")
                        .role("button")
                        .label("Move role up")
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .center()
                        .on_press(|s: &mut App| s.move_selected_role(-1))
                        .child(text("Up").font(theme::LABEL).color(p.text_bright)),
                )
                .child(
                    el().h(30.0)
                        .px(12.0)
                        .radius(4.0)
                        .key("role-down")
                        .role("button")
                        .label("Move role down")
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .center()
                        .on_press(|s: &mut App| s.move_selected_role(1))
                        .child(text("Down").font(theme::LABEL).color(p.text_bright)),
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

fn role_footer(s: &App, p: Palette) -> El {
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

fn manage_members_view(s: &App, role: &Role, p: Palette) -> El {
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

fn role_editor(s: &App, p: Palette) -> El {
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

fn role_icon_row(s: &App, role: &Role, p: Palette) -> El {
    let unlocked = s
        .active_guild()
        .map(|guild| guild.boost_level >= 2)
        .unwrap_or(false);
    let role_id = role.id;
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
                    .on_press_tx(move |s: &mut App, tx: &mut rune::Tx| {
                        let options = rune::FileDialogOptions {
                            title: "Upload role icon".to_string(),
                            directory: None,
                            file_name: None,
                            filters: Vec::new(),
                        }
                        .filter("Images", &["png", "jpg", "jpeg", "gif", "webp"]);
                        tx.open_file(options, move |s: &mut App, result, _| {
                            if let Ok(Some(path)) = result {
                                s.upload_role_icon(role_id, path);
                            }
                        });
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

fn role_links_view(s: &App, role: &Role, p: Palette) -> El {
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

fn role_preview(name: &str, color: u32, p: Palette) -> El {
    let name = name.trim().to_string();
    let name = if name.is_empty() {
        "New Role".to_string()
    } else {
        name
    };
    let mention = name.to_lowercase().replace(' ', "");
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
            &format!("This is how @{mention} appears in chat."),
            p,
        ))
        .child(message(false, "Mention the role to notify its members.", p))
}

fn role_swatches(selected: u32, p: Palette) -> El {
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

fn toggle(key: &'static str, label: &'static str, on: bool, p: Palette, run: fn(&mut App)) -> El {
    row()
        .key(key)
        .h(30.0)
        .px(8.0)
        .radius(4.0)
        .gap(8.0)
        .center_y()
        .role("checkbox")
        .label(label)
        .checked(on)
        .hover(p.card_hover)
        .on_press(run)
        .child(
            el().size(14.0, 14.0)
                .radius(3.0)
                .fill(if on { p.green } else { p.input })
                .border(1.0, p.divider),
        )
        .child(text(label).font(theme::SMALL).color(p.text))
}

fn audit_view(s: &App, p: Palette) -> El {
    let mut list = col().key("audit-list").w_full().gap(2.0);
    if s.audit.is_empty() {
        list = list.child(
            text(if s.audit_loading {
                "Loading audit log…"
            } else {
                "No audit entries yet."
            })
            .font(theme::SMALL)
            .color(p.text_faint)
            .padding(8.0),
        );
    }
    for entry in &s.audit {
        let (label, color) = audit_action(entry.action_type, p);
        let actor = s.user_name(entry.actor_id);
        let target = audit_target(s, entry);
        let changes = audit_changes(&entry.changes);
        list = list.child(
            row()
                .key(format!("audit-{}", entry.id))
                .w_full()
                .min_h(48.0)
                .px(10.0)
                .py(6.0)
                .radius(5.0)
                .gap(10.0)
                .center_y()
                .hover(p.card_hover)
                .child(el().w(4.0).h(30.0).radius(2.0).fill(color))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(
                            row()
                                .gap(6.0)
                                .center_y()
                                .child(text(label).font(theme::SMALL_MED).color(color))
                                .child(
                                    text(target)
                                        .font(theme::SMALL)
                                        .color(p.text_bright)
                                        .truncate(),
                                ),
                        )
                        .child(
                            text(format!("by {actor}"))
                                .font(theme::TINY)
                                .color(p.text_muted),
                        )
                        .children((!changes.is_empty()).then(|| {
                            text(changes)
                                .font(theme::TINY)
                                .color(p.text_faint)
                                .truncate()
                        })),
                )
                .child(
                    text(time_label(entry.created_at))
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        );
    }

    let mut footer = col().w_full().gap(8.0);
    if s.audit_loading {
        footer = footer.child(text("Loading…").font(theme::SMALL).color(p.text_faint));
    } else if !s.audit_complete && !s.audit.is_empty() {
        footer = footer.child(
            el().h(32.0)
                .px(16.0)
                .radius(4.0)
                .key("audit-more")
                .role("button")
                .label("Load older audit entries")
                .fill(p.rail)
                .hover(p.rail_hover)
                .center()
                .on_press(|s: &mut App| s.load_audit(true))
                .child(
                    text("Load Older")
                        .font(theme::SMALL_MED)
                        .color(p.text_bright),
                ),
        );
    }

    col()
        .key("audit")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .gap(10.0)
        .child(
            text(format!("{} entries loaded", s.audit.len()))
                .font(theme::TINY)
                .color(p.text_faint),
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
        .child(footer)
}

fn audit_action(action: i32, p: Palette) -> (String, Color) {
    let (label, color) = match action {
        10 => ("Created role", p.green),
        11 => ("Updated role", p.yellow),
        12 => ("Deleted role", p.danger),
        20 => ("Updated member roles", p.yellow),
        21 => ("Kicked member", p.danger),
        22 => ("Banned member", p.danger),
        23 => ("Timed out member", p.yellow),
        24 => ("Unbanned member", p.green),
        25 => ("Pruned members", p.yellow),
        26 => ("Accepted ban appeal", p.green),
        27 => ("Denied ban appeal", p.danger),
        28 => ("Submitted ban appeal", p.yellow),
        50 => ("Created webhook", p.green),
        51 => ("Deleted webhook", p.danger),
        60 => ("Created emoji", p.green),
        61 => ("Deleted emoji", p.danger),
        other => (return (format!("Action {other}"), p.text_muted)),
    };
    (label.to_string(), color)
}

fn audit_target(s: &App, entry: &AuditEntry) -> String {
    let Some(target) = entry.target_id else {
        return "server".to_string();
    };
    let name = audit_change_name(entry);
    match entry.action_type {
        10 | 11 | 12 => s
            .guild_roles(entry.guild_id)
            .iter()
            .find(|role| role.id == target)
            .map(|role| role.name.clone())
            .or(name)
            .unwrap_or_else(|| format!("Role #{target}")),
        20 | 21 | 22 | 23 | 24 | 26 | 27 => s
            .user(target)
            .map(|user| user.display_name.clone())
            .or(name)
            .unwrap_or_else(|| format!("#{target}")),
        60 | 61 => s
            .emojis
            .iter()
            .find(|emoji| emoji.id == target)
            .map(|emoji| format!(":{}:", emoji.name))
            .or(name.map(|name| format!(":{name}:")))
            .unwrap_or_else(|| format!("Emoji #{target}")),
        50 | 51 => name.unwrap_or_else(|| format!("Webhook #{target}")),
        _ => format!("#{target}"),
    }
}

fn audit_change_name(entry: &AuditEntry) -> Option<String> {
    match &entry.changes {
        serde_json::Value::Array(items) => items.iter().find_map(|item| {
            let object = item.as_object()?;
            if object.get("key")?.as_str()? != "name" {
                return None;
            }
            object
                .get("new_value")
                .or_else(|| object.get("old_value"))
                .and_then(|value| value.as_str())
                .map(str::to_string)
        }),
        serde_json::Value::Object(map) => map
            .get("name")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        _ => None,
    }
}

fn audit_changes(changes: &serde_json::Value) -> String {
    let pairs: Vec<String> = match changes {
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                let object = item.as_object()?;
                let key = object.get("key")?.as_str()?;
                let value = object
                    .get("new_value")
                    .or_else(|| object.get("old_value"))?;
                Some(format!("{key}: {}", audit_value(value)))
            })
            .collect(),
        serde_json::Value::Object(map) => map
            .iter()
            .filter(|(_, value)| !value.is_null())
            .map(|(key, value)| format!("{key}: {}", audit_value(value)))
            .collect(),
        _ => Vec::new(),
    };
    pairs.join(", ")
}

fn audit_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn emojis_view(s: &App, p: Palette) -> El {
    let mut grid = row().w_full().gap(8.0).flex_wrap();
    for emoji in &s.emojis {
        let id = emoji.id;
        let body = if let Some(pixels) = s.images.get(&emoji.url) {
            el().size(44.0, 44.0)
                .pixels_fit(pixels, rune::SurfaceFit::Fit)
        } else {
            text(emoji.name.clone())
                .font(theme::TINY)
                .color(p.text_faint)
        };
        grid = grid.child(
            col()
                .key(format!("emoji-card-{id}"))
                .w(112.0)
                .padding(8.0)
                .radius(6.0)
                .gap(6.0)
                .fill(p.rail)
                .center()
                .child(body)
                .child(
                    text(format!(":{}:", emoji.name))
                        .font(theme::TINY)
                        .color(p.text_bright)
                        .truncate(),
                )
                .child(
                    el().h(24.0)
                        .px(8.0)
                        .radius(4.0)
                        .key(format!("emoji-delete-{id}"))
                        .role("button")
                        .label(format!("Delete {}", emoji.name))
                        .fill(p.rail_hover)
                        .hover(p.danger.alpha(0.2))
                        .center()
                        .on_press(move |s: &mut App| s.delete_emoji(id))
                        .child(text("Delete").font(theme::TINY).color(p.danger)),
                ),
        );
    }

    col()
        .key("emoji-settings")
        .w_full()
        .grow(1.0)
        .gap(14.0)
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(10.0)
                .child(text("ADD EMOJI").font(theme::SECTION).color(p.text_muted))
                .child(
                    text("Name it, then drop a PNG onto the box. Names use letters, numbers and underscores.")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                )
                .child(
                    text_input(&s.emoji_name, theme::SMALL)
                        .key("emoji-name")
                        .w_full()
                        .h(38.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Emoji name")
                        .placeholder("Emoji name", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.emoji_name),
                )
                .child(
                    el()
                        .key("emoji-drop")
                        .w_full()
                        .h(88.0)
                        .radius(8.0)
                        .border(1.0, p.divider)
                        .fill(p.input)
                        .center()
                        .role("button")
                        .label("Drop a PNG here to add an emoji")
                        .on_drop_files(|s: &mut App, files, _| {
                            if let Some(file) = files.first() {
                                s.create_emoji_from_path(file.clone());
                            }
                        })
                        .accepts_extensions(&["png", "gif", "jpg", "jpeg", "webp"])
                        .child(
                            col()
                                .center()
                                .gap(4.0)
                                .child(
                                    text("Drop a PNG here")
                                        .font(theme::SMALL_MED)
                                        .color(p.text_bright),
                                )
                                .child(
                                    text("Up to 2 MB, square images look best")
                                        .font(theme::TINY)
                                        .color(p.text_faint),
                                ),
                        ),
                ),
        )
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(10.0)
                .child(
                    text(format!("SERVER EMOJIS — {}", s.emojis.len()))
                        .font(theme::SECTION)
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
                        .child(if s.emojis.is_empty() {
                            text("No emojis yet.")
                                .font(theme::SMALL)
                                .color(p.text_faint)
                        } else {
                            grid
                        }),
                ),
        )
}

fn members_view(s: &App, p: Palette) -> El {
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
