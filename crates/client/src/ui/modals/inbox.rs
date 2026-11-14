//! Inbox: recent mentions and unread channels.

use super::super::prelude::*;
use super::super::*;
use runecord_protocol::{Id, InboxEntry};

pub(crate) fn inbox_sheet(s: &App, p: Palette) -> El {
    let inbox = s.inbox.clone().unwrap_or_default();
    let mentions = inbox.mentions.len();
    let unreads = inbox.unreads.len();
    let showing_mentions = s.inbox_tab == 0;
    let entries: Vec<InboxEntry> = if showing_mentions {
        inbox.mentions.clone()
    } else {
        inbox.unreads.clone()
    };

    let tab = |label: &str, count: usize, active: bool, key: &'static str, index: u8| {
        let caption = format!("{label}  {count}");
        el().key(key)
            .h(30.0)
            .px(12.0)
            .radius(15.0)
            .center()
            .role("tab")
            .label(caption.clone())
            .checked(active)
            .fill(if active {
                p.selected
            } else {
                Color::TRANSPARENT
            })
            .hover(p.card_hover)
            .on_press(move |s: &mut App| s.inbox_tab = index)
            .child(text(caption).font(theme::SMALL_MED).color(if active {
                p.text_bright
            } else {
                p.text_muted
            }))
    };

    let mut list = col().w_full().gap(8.0);
    if entries.is_empty() {
        list = list.child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .center()
                .gap(8.0)
                .pt(80.0)
                .child(icon(&icons::INBOX, 42.0).color(p.text_faint))
                .child(
                    text(if showing_mentions {
                        "No recent mentions"
                    } else {
                        "You're all caught up!"
                    })
                    .font(theme::BODY_BOLD)
                    .color(p.text_bright),
                )
                .child(
                    text(if showing_mentions {
                        "When someone @mentions you, it will show up here."
                    } else {
                        "New messages from servers and DMs will show up here."
                    })
                    .font(theme::SMALL)
                    .color(p.text_muted),
                ),
        );
    }
    for entry in &entries {
        let channel = entry.channel_id;
        let guild = entry.guild_id;
        let message = entry.message_id;
        let author = s.user(entry.author_id).cloned();
        let avatar_el = match author {
            Some(user) => avatar(&user, 32.0, p),
            None => el()
                .size(32.0, 32.0)
                .radius(16.0)
                .fill(p.accent)
                .center()
                .child(
                    text(
                        entry
                            .author_name
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .to_string(),
                    )
                    .font(theme::SMALL_MED)
                    .color(Color::WHITE),
                ),
        };
        let breadcrumb = if entry.guild_id.is_some() {
            format!("#{}  ·  {}", entry.channel_name, entry.guild_name)
        } else {
            format!("@{}", entry.channel_name)
        };
        let chip = if entry.mention {
            "mention".to_string()
        } else if entry.unread > 0 {
            format!("{} new", entry.unread)
        } else {
            String::new()
        };
        let snippet = entry.snippet.replace(
            &format!("<@{}>", s.me.id),
            &format!("@{}", s.me.display_name),
        );
        list = list.child(
            col()
                .key(format!("inbox-entry-{channel}-{message}"))
                .w_full()
                .radius(6.0)
                .fill(p.card)
                .padding(10.0)
                .gap(8.0)
                .role("button")
                .label(format!("Jump to {} in {}", entry.author_name, breadcrumb))
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.jump_to_inbox_entry(guild, channel, message))
                .child(
                    row()
                        .w_full()
                        .gap(8.0)
                        .center_y()
                        .child(
                            text(breadcrumb)
                                .font(theme::TINY)
                                .color(p.text_muted)
                                .grow(1.0)
                                .truncate(),
                        )
                        .children((!chip.is_empty()).then(|| {
                            el().h(18.0)
                                .px(6.0)
                                .radius(9.0)
                                .fill(if entry.mention {
                                    p.mention.alpha(0.35)
                                } else {
                                    p.accent.alpha(0.25)
                                })
                                .center()
                                .child(
                                    text(chip)
                                        .font(theme::TINY)
                                        .color(if entry.mention {
                                            p.text_bright
                                        } else {
                                            p.accent
                                        })
                                        .truncate(),
                                )
                        }))
                        .children((!showing_mentions).then(|| {
                            el().key(format!("inbox-read-{channel}"))
                                .size(24.0, 24.0)
                                .radius(12.0)
                                .center()
                                .role("button")
                                .label("Mark as read")
                                .help("Mark as read")
                                .hover(p.rail_hover)
                                .on_press(move |s: &mut App| s.mark_inbox_read(channel, message))
                                .child(icon(&icons::CHECK, 14.0).color(p.text_muted))
                        })),
                )
                .child(
                    row().w_full().gap(8.0).child(avatar_el).child(
                        col()
                            .grow(1.0)
                            .min_w(0.0)
                            .gap(2.0)
                            .child(
                                row()
                                    .gap(6.0)
                                    .center_y()
                                    .child(
                                        text(entry.author_name.clone())
                                            .font(theme::SMALL_MED)
                                            .color(p.text_bright)
                                            .truncate(),
                                    )
                                    .child(
                                        text(time_label(entry.timestamp))
                                            .font(theme::TINY)
                                            .color(p.text_muted),
                                    ),
                            )
                            .child(text(snippet).font(theme::SMALL).color(p.text).max_lines(2)),
                    ),
                ),
        );
    }

    col()
        .key("inbox-panel")
        .w(420.0)
        .h(760.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .shadow(16.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .h(52.0)
                .px(16.0)
                .gap(8.0)
                .center_y()
                .child(
                    text("Inbox")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .children((unreads > 0).then(|| {
                    el().key("inbox-mark-all")
                        .h(28.0)
                        .px(10.0)
                        .radius(4.0)
                        .center_y()
                        .gap(6.0)
                        .role("button")
                        .label("Mark all as read")
                        .hover(p.rail_hover)
                        .on_press(|s: &mut App| s.mark_all_inbox_read())
                        .child(icon(&icons::CHECK, 14.0).color(p.text_muted))
                        .child(text("Mark all read").font(theme::TINY).color(p.text_muted))
                }))
                .child(close_button(p)),
        )
        .child(
            row()
                .w_full()
                .h(42.0)
                .px(16.0)
                .gap(4.0)
                .center_y()
                .child(tab(
                    "Mentions",
                    mentions,
                    showing_mentions,
                    "inbox-tab-mentions",
                    0,
                ))
                .child(tab(
                    "Unreads",
                    unreads,
                    !showing_mentions,
                    "inbox-tab-unreads",
                    1,
                )),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .padding(12.0)
                .child(list),
        )
}
