//! Search and quick switcher.

use super::super::prelude::*;
use super::super::*;

pub(crate) fn search_sheet(s: &App, p: Palette) -> El {
    let body = if let Some(results) = &s.search_results {
        if results.hits.is_empty() {
            col()
                .grow(1.0)
                .w_full()
                .center()
                .gap(8.0)
                .child(icon(&icons::SEARCH, 48.0).color(p.text_faint))
                .child(text("No results").font(theme::BODY).color(p.text_muted))
                .child(
                    text("Try a different word.")
                        .font(theme::SMALL)
                        .color(p.text_faint),
                )
        } else {
            let mut list = col().w_full().px(12.0).py(8.0).gap(4.0);
            for hit in results.hits.iter().take(40) {
                let guild = hit.guild_id;
                let channel = hit.channel_id;
                let name = hit.author.display_name.clone();
                let content = hit.message.content.replace('\n', " ");
                let channel_name = hit.channel_name.clone();
                let guild_name = hit.guild_name.clone().unwrap_or_default();
                let location = if guild_name.is_empty() {
                    "Direct Message".to_string()
                } else {
                    format!("#{channel_name} · {guild_name}")
                };
                let timestamp = hit.message.timestamp;
                list = list.child(
                    col()
                        .key(format!("hit-{}", hit.message.id))
                        .w_full()
                        .radius(4.0)
                        .padding(10.0)
                        .gap(4.0)
                        .hover(p.selected)
                        .on_press(move |s: &mut App| {
                            if let Some(guild) = guild {
                                s.screen = Screen::Guild;
                                s.active_guild = guild;
                                s.active_channel = channel;
                            } else {
                                s.screen = Screen::Dms;
                                s.dm_home = false;
                                s.active_dm = channel;
                            }
                            s.modal = Modal::None;
                            s.ensure_messages(channel);
                        })
                        .child(
                            row()
                                .gap(6.0)
                                .center_y()
                                .child(text(name).font(theme::SMALL_MED).color(p.text_bright))
                                .child(text(location).font(theme::TINY).color(p.text_faint))
                                .child(el().grow(1.0))
                                .child(
                                    text(time_label(timestamp))
                                        .font(theme::TINY)
                                        .color(p.text_faint),
                                ),
                        )
                        .child(
                            text(content)
                                .font(theme::SMALL)
                                .color(p.text_muted)
                                .truncate()
                                .max_lines(2),
                        ),
                );
            }
            col()
                .grow(1.0)
                .w_full()
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list)
        }
    } else {
        col()
            .grow(1.0)
            .w_full()
            .center()
            .gap(8.0)
            .child(icon(&icons::SEARCH, 48.0).color(p.text_faint))
            .child(
                text("Search your messages")
                    .font(theme::BODY)
                    .color(p.text_muted),
            )
            .child(
                text("Press Enter to search the whole local server.")
                    .font(theme::SMALL)
                    .color(p.text_faint),
            )
    };

    col()
        .key("search-sheet")
        .w(720.0)
        .h(520.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .h(56.0)
                .px(16.0)
                .gap(10.0)
                .items_center()
                .child(icon(&icons::SEARCH, 20.0).color(p.text_faint))
                .child(
                    text_input(&s.search, theme::BODY)
                        .key("search-input")
                        .grow(1.0)
                        .h(38.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Search messages")
                        .placeholder("Search messages", p.text_faint)
                        .padding(8.0)
                        .on_edit(|s: &mut App| &mut s.search)
                        .on_submit(|s: &mut App, _: &mut rune::Tx| s.run_search()),
                )
                .child(close_button(p)),
        )
        .child(
            row()
                .w_full()
                .h(38.0)
                .px(16.0)
                .gap(6.0)
                .center_y()
                .children(
                    [
                        ("from:", "From"),
                        ("in:", "In"),
                        ("mentions:", "Mentions"),
                        ("has:link", "Has Link"),
                        ("before:", "Before"),
                        ("after:", "After"),
                    ]
                    .into_iter()
                    .map(|(token, label)| {
                        el().key(format!(
                            "search-filter-{}",
                            token.trim_end_matches([':', ' '])
                        ))
                        .h(24.0)
                        .px(8.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Add {label} filter"))
                        .fill(p.input)
                        .hover(p.input_hover)
                        .on_press(move |s: &mut App| s.append_search_filter(token))
                        .child(text(label).font(theme::TINY).color(p.text_muted))
                    }),
                ),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(body)
}

pub(crate) fn quick_switcher(s: &App, p: Palette) -> El {
    let query = s.search.text.trim().to_lowercase();
    let mut channels = col().w_full().gap(1.0);
    let mut channel_count = 0;
    for guild in &s.guilds {
        for ch in s.channels.get(&guild.id).into_iter().flatten() {
            if ch.kind == ChannelKind::Category || ch.kind == ChannelKind::Thread {
                continue;
            }
            if !query.is_empty()
                && !ch.name.to_lowercase().contains(&query)
                && !guild.name.to_lowercase().contains(&query)
            {
                continue;
            }
            let id = ch.id;
            let gid = guild.id;
            let ic = match ch.kind {
                ChannelKind::Voice => &icons::VOICE,
                ChannelKind::Forum => &icons::FORUM,
                ChannelKind::Announcement => &icons::MEGAPHONE,
                _ => &icons::HASH,
            };
            channels = channels.child(
                row()
                    .key(format!("quick-{}", ch.id))
                    .w_full()
                    .h(40.0)
                    .px(12.0)
                    .gap(10.0)
                    .radius(4.0)
                    .center_y()
                    .role("button")
                    .label(format!("#{} in {}", ch.name, guild.name))
                    .hover(p.selected)
                    .motion(motion::SNAPPY)
                    .on_press(move |s: &mut App| {
                        s.screen = Screen::Guild;
                        s.active_guild = gid;
                        s.active_channel = id;
                        s.modal = Modal::None;
                    })
                    .child(icon(ic, 18.0).color(p.text_faint))
                    .child(
                        text(ch.name.clone())
                            .font(theme::SMALL_MED)
                            .color(p.text_bright)
                            .grow(1.0),
                    )
                    .child(
                        text(guild.name.clone())
                            .font(theme::TINY)
                            .color(p.text_faint),
                    ),
            );
            channel_count += 1;
        }
    }

    let mut dms = col().w_full().gap(1.0);
    let mut dm_count = 0;
    for dm in &s.dms {
        let Some(other) = dm.recipients.first() else {
            continue;
        };
        if !query.is_empty()
            && !other.display_name.to_lowercase().contains(&query)
            && !other.username.to_lowercase().contains(&query)
        {
            continue;
        }
        let id = dm.channel.id;
        let name = other.display_name.clone();
        dms = dms.child(
            row()
                .key(format!("quick-dm-{}", id))
                .w_full()
                .h(40.0)
                .px(12.0)
                .gap(10.0)
                .radius(4.0)
                .center_y()
                .role("button")
                .label(format!("Direct message {name}"))
                .hover(p.selected)
                .motion(motion::SNAPPY)
                .on_press(move |s: &mut App| {
                    s.screen = Screen::Dms;
                    s.dm_home = false;
                    s.add_friend_open = false;
                    s.active_dm = id;
                    s.modal = Modal::None;
                })
                .child(avatar_status(other, 24.0, p, p.modal, true))
                .child(
                    text(name)
                        .font(theme::SMALL_MED)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(text("Direct Message").font(theme::TINY).color(p.text_faint)),
        );
        dm_count += 1;
    }

    let mut results = col().w_full().pt(6.0);
    if channel_count > 0 {
        results = results
            .child(
                text("CHANNELS")
                    .font(theme::SECTION)
                    .color(p.text_muted)
                    .px(12.0)
                    .pb(4.0)
                    .pt(4.0),
            )
            .child(channels);
    }
    if dm_count > 0 {
        results = results
            .child(
                text("DIRECT MESSAGES")
                    .font(theme::SECTION)
                    .color(p.text_muted)
                    .px(12.0)
                    .pb(4.0)
                    .pt(10.0),
            )
            .child(dms);
    }
    if channel_count == 0 && dm_count == 0 {
        results = results.child(
            col()
                .w_full()
                .py(32.0)
                .items_center()
                .gap(4.0)
                .child(
                    text("No results found")
                        .font(theme::SMALL_MED)
                        .color(p.text_bright),
                )
                .child(
                    text("Try a channel, server or friend name.")
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        );
    }

    col()
        .key("quick-switcher")
        .w(640.0)
        .h(560.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .h(56.0)
                .px(16.0)
                .gap(10.0)
                .items_center()
                .child(icon(&icons::SEARCH, 20.0).color(p.text_faint))
                .child(
                    text_input(&s.search, theme::BODY)
                        .key("quick-input")
                        .grow(1.0)
                        .min_w(0.0)
                        .h(38.0)
                        .color(p.text)
                        .role("textbox")
                        .label("Where would you like to go?")
                        .placeholder("Where would you like to go?", p.text_faint)
                        .padding(0.0)
                        .on_edit(|s: &mut App| &mut s.search),
                )
                .child(close_button(p)),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            col()
                .grow(1.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(results),
        )
}
