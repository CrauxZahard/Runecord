//! Composer surface.

use super::*;

pub(crate) fn composer(s: &App, channel: &Channel, p: Palette) -> El {
    composer_with(s, channel, p, false)
}

pub(crate) fn composer_with(s: &App, channel: &Channel, p: Palette, compact: bool) -> El {
    if !s.can_send_in(channel) {
        return col()
            .key("composer-wrap")
            .w_full()
            .px(24.0)
            .pt(8.0)
            .pb(20.0)
            .child(
                row().h(44.0).radius(22.0).fill(p.input).center().child(
                    text(if channel.kind == ChannelKind::Thread {
                        "You do not have permission to send messages in this thread."
                    } else {
                        "You do not have permission to send messages in this channel."
                    })
                    .font(theme::SMALL)
                    .color(p.text_muted),
                ),
            );
    }
    let placeholder = if channel.guild_id.is_none() {
        if let Some(dm) = s.dm_for(channel.id) {
            if channel.kind == ChannelKind::GroupDm {
                format!("Message {}", crate::ui::dms::dm_title(dm))
            } else {
                let name = dm
                    .recipients
                    .first()
                    .map(|u| u.display_name.clone())
                    .unwrap_or_else(|| channel.name.clone());
                format!("Message @{name}")
            }
        } else {
            format!("Message @{}", channel.name)
        }
    } else {
        format!("Message #{}", channel.name)
    };
    let mut wrap = col()
        .key("composer-wrap")
        .w_full()
        .px(24.0)
        .pt(8.0)
        .pb(20.0)
        .gap(8.0);
    if let Some(reply) = &s.reply_to {
        let reply_id = reply.message_id;
        wrap = wrap.child(
            row()
                .key("reply-chip")
                .pl(10.0)
                .pr(8.0)
                .py(5.0)
                .radius(14.0)
                .gap(6.0)
                .center_y()
                .fill(p.card)
                .border(1.0, theme::gloss_edge(p))
                .child(icon(&icons::REPLY, 14.0).color(p.text_link))
                .child(
                    text(format!("Replying to {}", reply.author_name))
                        .font(theme::SMALL_MED)
                        .color(p.text_bright),
                )
                .child(
                    text(reply.content.clone())
                        .font(theme::SMALL)
                        .color(p.text_muted)
                        .grow(1.0)
                        .min_w(0.0)
                        .truncate(),
                )
                .child(
                    el().key("toggle-reply-mention")
                        .h(20.0)
                        .px(8.0)
                        .radius(10.0)
                        .center()
                        .role("button")
                        .label(if s.reply_notify {
                            "Turn off reply mention"
                        } else {
                            "Turn on reply mention"
                        })
                        .help("@ off / @ on")
                        .fill(if s.reply_notify {
                            p.mention.alpha(0.25)
                        } else {
                            p.rail
                        })
                        .hover(p.input_hover)
                        .on_press(|s: &mut App| s.reply_notify = !s.reply_notify)
                        .child(
                            text(if s.reply_notify { "@ on" } else { "@ off" })
                                .font(theme::TINY)
                                .color(if s.reply_notify {
                                    p.text_link
                                } else {
                                    p.text_faint
                                }),
                        ),
                )
                .child(
                    el().key("cancel-reply")
                        .size(20.0, 20.0)
                        .radius(10.0)
                        .center()
                        .hover(p.input_hover)
                        .on_press(move |s: &mut App| {
                            let _ = reply_id;
                            s.reply_to = None;
                        })
                        .child(icon(&icons::CLOSE, 12.0).color(p.text_muted)),
                ),
        );
    }
    let typing: Vec<String> = s
        .typing
        .get(&channel.id)
        .map(|list| {
            list.iter()
                .filter(|(u, _)| *u != s.me.id)
                .map(|(u, _)| s.user_name(*u))
                .collect()
        })
        .unwrap_or_default();
    if !typing.is_empty() {
        let line = if typing.len() == 1 {
            format!("{} is typing…", typing[0])
        } else {
            format!("{} and {} are typing…", typing[0], typing.len() - 1)
        };
        wrap = wrap.child(
            row()
                .key("typing")
                .pl(8.0)
                .child(text(line).font(theme::SMALL).color(p.text_muted)),
        );
    }
    if !s.attachments.is_empty() {
        let mut chips = row().gap(8.0);
        for a in &s.attachments {
            let aid = a.id;
            chips = chips.child(
                row()
                    .h(40.0)
                    .radius(12.0)
                    .fill(p.rail)
                    .border(1.0, p.divider_soft)
                    .px(8.0)
                    .gap(8.0)
                    .center_y()
                    .child(icon(&icons::IMAGE, 20.0).color(p.text_muted))
                    .child(text(a.filename.clone()).font(theme::SMALL).color(p.text))
                    .child(
                        el().size(18.0, 18.0)
                            .radius(9.0)
                            .center()
                            .hover(p.input_hover)
                            .on_press(move |s: &mut App| s.attachments.retain(|x| x.id != aid))
                            .child(icon(&icons::CLOSE, 12.0).color(p.text_muted)),
                    ),
            );
        }
        wrap = wrap.child(chips);
    }
    let slash_query = s
        .composer
        .text
        .strip_prefix('/')
        .map(str::trim)
        .unwrap_or_default();
    if channel.guild_id.is_some()
        && s.composer.text.starts_with('/')
        && !slash_query.chars().any(char::is_whitespace)
    {
        let mut commands = col().w_full().gap(2.0);
        let mut count = 0;
        for command in s
            .commands
            .iter()
            .filter(|command| command.name.starts_with(&slash_query.to_lowercase()))
            .take(8)
        {
            let name = command.name.clone();
            commands = commands.child(
                row()
                    .key(format!("slash-{}", command.id))
                    .w_full()
                    .h(40.0)
                    .px(10.0)
                    .radius(10.0)
                    .gap(10.0)
                    .center_y()
                    .role("button")
                    .label(format!("/{}", command.name))
                    .hover(p.card_hover)
                    .on_press(move |s: &mut App| s.apply_slash_command(&name))
                    .child(
                        text(format!("/{}", command.name))
                            .font(theme::SMALL_MED)
                            .color(p.text_bright),
                    )
                    .child(
                        text(command.description.clone())
                            .font(theme::TINY)
                            .color(p.text_muted)
                            .grow(1.0)
                            .truncate(),
                    ),
            );
            count += 1;
        }
        if count > 0 {
            wrap = wrap.child(
                col()
                    .key("slash-picker")
                    .w_full()
                    .radius(16.0)
                    .clip_rounded(16.0)
                    .fill(p.card)
                    .border(1.0, theme::gloss_edge(p))
                    .padding(6.0)
                    .gap(2.0)
                    .shadow(8.0, 22.0, p.shadow)
                    .child(
                        text("COMMANDS")
                            .font(theme::SECTION)
                            .color(p.text_faint)
                            .padding(6.0),
                    )
                    .child(commands),
            );
        }
    }
    // Mention autocomplete sits inside the composer stack, above the field.
    if let Some((at, caret, query)) = mention_query(s) {
        wrap = wrap.child(mention_popup(s, at, caret, &query, p));
    }
    wrap.child(
        row()
            .key("composer")
            .w_full()
            .min_h(50.0)
            .radius(24.0)
            .fill(if s.drop_hover { p.input_hover } else { p.input })
            .border(
                1.0,
                if s.drop_hover {
                    theme::sunset_amber(p.dark)
                } else {
                    theme::gloss_edge(p)
                },
            )
            .shadow(10.0, 30.0, p.shadow)
            .backdrop_blur(10.0)
            .px(if compact { 10.0 } else { 14.0 })
            .gap(if compact { 8.0 } else { 12.0 })
            .center_y()
            .motion(motion::SNAPPY)
            .on_drop_files(|s: &mut App, files, _| {
                s.drop_hover = false;
                for f in files {
                    s.upload_file(crate::net::UploadSource::Path(f.to_path_buf()));
                }
            })
            .on_drop_hover(|s: &mut App, over, _| s.drop_hover = over)
            .on_drop_rejected(|s: &mut App, files, _| {
                s.drop_hover = false;
                let name = files
                    .first()
                    .and_then(|f| f.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("that file");
                s.toast(format!("{name} is not a supported file type"));
            })
            .accepts_extensions(&[
                "png", "jpg", "jpeg", "gif", "webp", "txt", "pdf", "zip", "mp4", "mp3",
            ])
            .child(
                el().key("attach")
                    .size(24.0, 24.0)
                    .radius(12.0)
                    .center()
                    .role("button")
                    .label("Upload a file")
                    .hover(p.text_bright)
                    .help("Upload a file")
                    .on_press_tx(|s: &mut App, tx: &mut rune::Tx| {
                        let channel = s.current_channel_id();
                        let kind = crate::files::PickKind::ChannelUpload { channel };
                        crate::files::open(
                            tx,
                            kind.clone(),
                            "Upload a file",
                            &[
                                "png", "jpg", "jpeg", "gif", "webp", "txt", "pdf", "zip", "mp4",
                                "mp3",
                            ],
                            move |s, path| s.accept_picked_path(kind.clone(), path),
                        );
                    })
                    .child(icon(&icons::PLUS, 22.0).color(p.interactive)),
            )
            .child(
                text_input(&s.composer, theme::BODY)
                    .key("composer-input")
                    .grow(1.0)
                    .min_w(0.0)
                    .h_full()
                    .color(p.text)
                    .role("textbox")
                    .label("Message")
                    .placeholder(placeholder, p.text_faint)
                    .padding(0.0)
                    .pt(11.0)
                    // Render markdown in place, inside the field itself: the
                    // preview is the text you are typing, styled as you type.
                    .rich_style(rune::RichStyle {
                        link: p.text_link,
                        code_fg: Some(p.text_link),
                        code_bg: p.code_bg,
                        code_block_bg: p.code_bg,
                        quote_bar: p.divider,
                        marker: Some(p.text_faint),
                    })
                    .spans(markdown_input_spans(&s.composer.text, p))
                    .on_edit(|s: &mut App| {
                        s.note_typing();
                        &mut s.composer
                    })
                    .on_submit(|s: &mut App, _: &mut rune::Tx| s.send_current())
                    .on_input_paste(|s: &mut App, content, _: &mut rune::Tx| {
                        if let Some(image) = content.image() {
                            s.upload_file(crate::net::UploadSource::Bytes {
                                name: format!("pasted-{}.png", now_ms()),
                                bytes: image.png().to_vec(),
                            });
                            return true;
                        }
                        if let Some(file) = content.files().first() {
                            s.upload_file(crate::net::UploadSource::Path(file.clone()));
                            return true;
                        }
                        false
                    })
                    .on_input_key(|s: &mut App, ev: &rune::KeyEvent, _: &mut rune::Tx| {
                        if ev.key == rune::Key::Enter && ev.mods.cmd {
                            s.send_current();
                            true
                        } else {
                            false
                        }
                    }),
            )
            .children((!compact).then(|| {
                let mut extras = row().gap(12.0).center_y().child(
                    el().key("composer-gif")
                        .gap(2.0)
                        .center_y()
                        .role("button")
                        .label("GIF picker")
                        .help("GIFs")
                        .on_press(|s: &mut App| s.open_gif_picker())
                        .child(icon(&icons::GIFT, 24.0).color(p.interactive))
                        .child(
                            text("GIF")
                                .font(TextStyle::new(10.0, 800.0))
                                .color(p.interactive)
                                .border(1.5, p.interactive)
                                .radius(8.0)
                                .px(4.0)
                                .py(1.0),
                        ),
                );
                extras = extras.child(
                    icon(&icons::STICKER, 24.0)
                        .key("composer-sticker")
                        .color(p.interactive)
                        .role("button")
                        .label("Stickers")
                        .help("Stickers")
                        .on_press(|s: &mut App| s.modal = Modal::StickerPicker),
                );
                extras = extras
                    .child(
                        icon(&icons::POLL, 22.0)
                            .key("composer-poll")
                            .color(p.interactive)
                            .role("button")
                            .label("Create poll")
                            .help("Create poll")
                            .on_press(|s: &mut App| s.open_create_poll()),
                    )
                    .child(
                        icon(&icons::CLOCK, 22.0)
                            .key("composer-schedule")
                            .color(p.interactive)
                            .role("button")
                            .label("Schedule message")
                            .help("Schedule message")
                            .on_press(|s: &mut App| s.open_scheduled()),
                    );
                extras
            }))
            .child(
                icon(&icons::SMILE, 24.0)
                    .key("composer-emoji")
                    .color(p.interactive)
                    .role("button")
                    .label("Emoji")
                    .help("Emoji")
                    .on_press(|s: &mut App| s.open_emoji()),
            )
            .children(
                (!s.composer.text.trim().is_empty() || !s.attachments.is_empty()).then(|| {
                    el().key("send")
                        .size(34.0, 34.0)
                        .radius(17.0)
                        .center()
                        .role("button")
                        .label("Send message")
                        .help("Send message")
                        .fill(p.accent)
                        .gradient(theme::sunset_magenta(p.dark), theme::sunset_violet(p.dark))
                        .border(1.0, theme::sunset_amber(p.dark).alpha(0.65))
                        .hover(p.accent_hover)
                        .motion(motion::SNAPPY)
                        .on_press(|s: &mut App| s.send_current())
                        .child(icon(&icons::SEND, 16.0).color(Color::WHITE))
                }),
            ),
    )
}

/// Byte ranges to style inside the composer's own text field, so markdown is
/// previewed in place (code, spoilers, bold, italics, quotes) instead of in a
/// separate box below it.
fn markdown_input_spans(content: &str, p: Palette) -> Vec<rune::Span> {
    let bytes = content.as_bytes();
    let mut spans = Vec::new();
    let mut open: Vec<(usize, u8)> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let rest = &content[i..];
        let (len, kind) = if rest.starts_with("```") {
            (3, 0)
        } else if rest.starts_with("||") {
            (2, 1)
        } else if rest.starts_with("**") || rest.starts_with("__") {
            (2, 2)
        } else if rest.starts_with("~~") {
            (2, 3)
        } else if rest.starts_with('`') {
            (1, 4)
        } else if rest.starts_with('*') || rest.starts_with('_') {
            (1, 5)
        } else {
            (0, 255)
        };
        if len > 0 {
            if let Some(pos) = open.iter().rposition(|(_, k)| *k == kind) {
                let (start, _) = open.remove(pos);
                if i > start + len {
                    let span = rune::Span::new(start + len, i);
                    spans.push(match kind {
                        0 => span.mono().color(p.text_link),
                        1 => span.bg(p.code_bg, 2.0),
                        2 => span.weight(700.0),
                        3 => span.color(p.text_faint),
                        4 => span.mono().bg(p.code_bg, 2.0).color(p.text_link),
                        5 => span.italic(),
                        _ => span,
                    });
                }
            } else {
                open.push((i, kind));
            }
            i += len;
            continue;
        }
        i += rest.chars().next().map(char::len_utf8).unwrap_or(1);
    }
    spans
}

/// The `@query` currently being typed, as byte offsets into `edit.text`.
fn mention_query(s: &App) -> Option<(usize, usize, String)> {
    if s.composer.text.is_empty() {
        return None;
    }
    let caret = s.composer.focus.min(s.composer.text.len());
    let before = &s.composer.text[..caret];
    let at = before.rfind('@')?;
    let query = &before[at + 1..caret];
    if query.contains(char::is_whitespace) || query.len() > 32 {
        return None;
    }
    Some((at, caret, query.to_lowercase()))
}

fn mention_popup(s: &App, at: usize, caret: usize, query: &str, p: Palette) -> El {
    let matches = |label: &str| query.is_empty() || label.to_lowercase().contains(query);
    let mut entries: Vec<El> = Vec::new();

    for (token, label, detail) in [
        ("@game ", "@game", "Mention a game"),
        (
            "@time ",
            "@time",
            "Refer to a time dynamically in the viewer's time zone",
        ),
    ] {
        if !matches(label) {
            continue;
        }
        let token = token.to_string();
        entries.push(
            row()
                .key(format!("mention-special-{label}"))
                .w_full()
                .h(34.0)
                .px(10.0)
                .gap(8.0)
                .center_y()
                .role("button")
                .hover(p.selected)
                .on_press(move |s: &mut App| s.insert_mention(at, caret, &token))
                .child(
                    text(label)
                        .font(theme::SMALL_MED)
                        .color(p.text_bright)
                        .w(96.0),
                )
                .child(
                    text(detail)
                        .font(theme::TINY)
                        .color(p.text_muted)
                        .grow(1.0)
                        .truncate(),
                ),
        );
    }

    let mut members: Vec<_> = s
        .members
        .get(&s.active_guild)
        .map(|list| list.iter().map(|m| m.user.clone()).collect::<Vec<_>>())
        .unwrap_or_else(|| s.users.values().cloned().collect());
    members.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    // Keep a few members, then roles, so role pings are visible without
    // scrolling through a long member list.
    for user in members.into_iter().take(4) {
        let label = format!("@{}", user.display_name);
        if !matches(&label) && !matches(&user.username) {
            continue;
        }
        let token = format!("<@{}> ", user.id);
        let handle = user.handle();
        let avatar_user = user.clone();
        entries.push(
            row()
                .key(format!("mention-member-{}", user.id))
                .w_full()
                .h(36.0)
                .px(10.0)
                .gap(8.0)
                .center_y()
                .role("button")
                .hover(p.selected)
                .on_press(move |s: &mut App| s.insert_mention(at, caret, &token))
                .child(crate::ui::common::avatar_status(
                    &avatar_user,
                    20.0,
                    p,
                    p.card,
                    true,
                ))
                .child(
                    text(label)
                        .font(theme::SMALL_MED)
                        .color(p.text_bright)
                        .w(130.0)
                        .truncate(),
                )
                .child(
                    text(handle)
                        .font(theme::TINY)
                        .color(p.text_faint)
                        .grow(1.0)
                        .truncate(),
                ),
        );
    }

    for role in s.guild_roles(s.active_guild) {
        let everyone = role.name == "@everyone";
        // `@everyone` already reads as a mention, so it needs no second `@`.
        let label = if everyone {
            role.name.clone()
        } else {
            format!("@{}", role.name)
        };
        if !matches(&label) {
            continue;
        }
        // `@everyone` is not a role ping. It is the plain-text word that pings
        // the whole server, so send it as text instead of a role token.
        let token = if everyone {
            "@everyone ".to_string()
        } else {
            format!("<@&{}> ", role.id)
        };
        let tint = if role.color == 0 {
            p.text_bright
        } else {
            theme::color(&format!("#{:06x}", role.color))
        };
        let detail = if everyone {
            "Notify everyone in the server."
        } else {
            "Notify users with this role who have permission to view this channel."
        };
        entries.push(
            row()
                .key(format!("mention-role-{}", role.id))
                .w_full()
                .h(36.0)
                .px(10.0)
                .gap(8.0)
                .center_y()
                .role("button")
                .hover(p.selected)
                .on_press(move |s: &mut App| s.insert_mention(at, caret, &token))
                .child(
                    text(label)
                        .font(theme::SMALL_MED)
                        .color(tint)
                        .w(160.0)
                        .truncate(),
                )
                .child(
                    text(detail)
                        .font(theme::TINY)
                        .color(p.text_muted)
                        .grow(1.0)
                        .truncate(),
                ),
        );
    }

    if entries.is_empty() {
        return el();
    }
    let entries = entries.into_iter().take(16).collect::<Vec<_>>();
    let mut body = col().w_full().gap(1.0);
    body = body.child(
        text("MEMBERS")
            .font(theme::SECTION)
            .color(p.text_link)
            .px(10.0)
            .py(4.0),
    );
    for entry in entries {
        body = body.child(entry);
    }
    col()
        .key("mention-popup")
        .w_full()
        .max_h(320.0)
        .radius(16.0)
        .fill(p.modal)
        .border(1.0, theme::gloss_edge(p))
        .clip_rounded(16.0)
        .scroll(rune::ScrollSpec {
            stick_bottom: false,
        })
        .padding(5.0)
        .shadow(8.0, 22.0, p.shadow)
        .child(body)
}
