//! Sticker picker.

use super::*;

pub(crate) fn sticker_sheet(s: &App, p: Palette) -> El {
    let mut grid = row().w_full().gap(8.0).flex_wrap();
    let mut count = 0;
    for emoji in s.emojis.iter().filter(|emoji| emoji.sticker) {
        count += 1;
        let name = emoji.name.clone();
        let body = if let Some(pixels) = s.images.get(&emoji.url) {
            el().size(72.0, 72.0)
                .pixels_fit(pixels, rune::SurfaceFit::Fit)
        } else {
            text(format!(":{}:", emoji.name))
                .font(theme::TINY)
                .color(p.text_faint)
        };
        grid = grid.child(
            el().key(format!("sticker-{}", emoji.id))
                .size(88.0, 88.0)
                .radius(6.0)
                .fill(p.rail)
                .center()
                .role("button")
                .label(format!("Send sticker {}", emoji.name))
                .hover(p.card_hover)
                .on_press(move |s: &mut App| {
                    s.composer.insert(&format!(":{}: ", name));
                    s.modal = Modal::None;
                })
                .child(body),
        );
    }
    if count == 0 {
        grid = grid.child(
            text("No stickers yet. Upload one from Server Settings > Emojis.")
                .font(theme::SMALL)
                .color(p.text_muted)
                .padding(8.0),
        );
    }
    col()
        .key("sticker-sheet")
        .w(520.0)
        .h(420.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .padding(18.0)
        .gap(10.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text("Stickers")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(grid),
        )
}
