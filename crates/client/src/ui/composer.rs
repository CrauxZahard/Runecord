//! Composer surface.

use super::*;

pub(crate) fn composer(s: &App, channel: &Channel, p: Palette) -> El {
    composer_with(s, channel, p, false)
}

pub(crate) fn composer_with(s: &App, channel: &Channel, p: Palette, compact: bool) -> El {
    if !has(channel.my_permissions, perm::SEND_MESSAGES) {
        return col().key("composer-wrap").w_full().px(16.0).pb(16.0).child(
            row().h(44.0).radius(8.0).fill(p.input).center().child(
                text("You do not have permission to send messages in this channel.")
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
        .px(20.0)
        .pb(18.0)
        .gap(8.0);
    if let Some(reply) = &s.reply_to {
        let reply_id = reply.message_id;
        wrap = wrap.child(
            row()
                .key("reply-chip")
                .pl(8.0)
                .pr(8.0)
                .py(4.0)
                .gap(6.0)
                .center_y()
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
                    .radius(4.0)
                    .fill(p.rail)
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
                    .radius(4.0)
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
                    .radius(8.0)
                    .fill(p.card)
                    .border(1.0, p.divider)
                    .padding(6.0)
                    .gap(2.0)
                    .shadow(8.0, 20.0, p.shadow)
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
    wrap.child(
        row()
            .key("composer")
            .w_full()
            .min_h(44.0)
            .radius(10.0)
            .fill(if s.drop_hover { p.input_hover } else { p.input })
            .border(if s.drop_hover { 1.0 } else { 0.0 }, p.accent)
            .backdrop_blur(8.0)
            .px(if compact { 10.0 } else { 16.0 })
            .gap(if compact { 8.0 } else { 12.0 })
            .center_y()
            .motion(motion::SNAPPY)
            .on_drop_files(|s: &mut App, files, _| {
                s.drop_hover = false;
                for f in files {
                    s.upload_file(f.to_path_buf());
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
                    .on_press(|s: &mut App| s.toast("Drag a file into the window to attach it"))
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
                    .on_edit(|s: &mut App| {
                        s.note_typing();
                        &mut s.composer
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
                row()
                    .gap(12.0)
                    .center_y()
                    .child(icon(&icons::GIFT, 24.0).color(p.interactive))
                    .child(
                        text("GIF")
                            .font(TextStyle::new(11.0, 700.0))
                            .color(p.interactive)
                            .border(1.5, p.interactive)
                            .radius(4.0)
                            .px(3.0)
                            .py(1.0),
                    )
                    .child(icon(&icons::STICKER, 24.0).color(p.interactive))
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
                        .size(24.0, 24.0)
                        .radius(12.0)
                        .center()
                        .role("button")
                        .label("Send message")
                        .help("Send message")
                        .hover(p.input_hover)
                        .on_press(|s: &mut App| s.send_current())
                        .child(icon(&icons::SEND, 18.0).color(p.interactive))
                }),
            ),
    )
}

// ---------------------------------------------------------------------------
// Members
// ---------------------------------------------------------------------------
