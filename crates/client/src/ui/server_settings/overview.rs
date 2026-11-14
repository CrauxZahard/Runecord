use super::super::prelude::*;
use super::super::*;
use super::*;

pub(crate) fn overview_footer(s: &App, p: Palette) -> El {
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

pub(crate) fn image_section(s: &App, guild: &GuildSummary, banner: bool, p: Palette) -> El {
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

pub(crate) fn overview_view(s: &App, p: Palette) -> El {
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
