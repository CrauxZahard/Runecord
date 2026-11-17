use super::*;

pub(crate) fn emojis_view(s: &App, p: Palette) -> El {
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
                    row()
                        .key("emoji-sticker-toggle")
                        .h(30.0)
                        .gap(8.0)
                        .center_y()
                        .role("checkbox")
                        .label("Upload as sticker")
                        .checked(s.emoji_sticker)
                        .on_press(|s: &mut App| s.emoji_sticker = !s.emoji_sticker)
                        .child(
                            el().size(16.0, 16.0)
                                .radius(4.0)
                                .fill(if s.emoji_sticker { p.green } else { p.input })
                                .border(1.0, p.divider)
                                .center()
                                .children(s.emoji_sticker.then(|| {
                                    icon(&icons::CHECK, 11.0).color(Color::WHITE)
                                })),
                        )
                        .child(
                            text("Upload as sticker")
                                .font(theme::SMALL)
                                .color(p.text),
                        ),
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
                        .label("Choose a PNG to add an emoji")
                        .on_drop_files(|s: &mut App, files, _| {
                            if let Some(file) = files.first() {
                                s.create_emoji_from_path(crate::net::UploadSource::Path(
                                    file.clone(),
                                ));
                            }
                        })
                        .on_press_tx(|s: &mut App, tx: &mut rune::Tx| {
                            let kind = crate::files::PickKind::Emoji {
                                guild: s.active_guild,
                                sticker: s.emoji_sticker,
                                name: s.emoji_name.text.trim().to_string(),
                            };
                            crate::files::open(
                                tx,
                                kind.clone(),
                                "Choose an emoji image",
                                &["png", "gif", "jpg", "jpeg", "webp"],
                                move |s, path| s.accept_picked_path(kind.clone(), path),
                            );
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
