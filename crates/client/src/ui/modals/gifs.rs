//! GIF picker over the server's built-in catalogue.

use super::*;

pub(crate) fn gif_sheet(s: &App, p: Palette) -> El {
    let mut grid = row().w_full().gap(8.0).flex_wrap();
    for gif in &s.gifs {
        let slug = gif.slug.clone();
        let body = if let Some(pixels) = s.images.get(&gif.url) {
            el().size(104.0, 104.0)
                .pixels_fit(pixels, rune::SurfaceFit::Fit)
        } else {
            text(gif.title.clone())
                .font(theme::TINY)
                .color(p.text_muted)
        };
        grid = grid.child(
            el().key(format!("gif-{}", gif.slug))
                .size(116.0, 116.0)
                .radius(6.0)
                .fill(p.rail)
                .center()
                .role("button")
                .label(format!("Send the {} GIF", gif.title))
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.send_gif(&slug))
                .child(body),
        );
    }
    if s.gifs.is_empty() {
        grid = grid.child(
            text("No GIFs match that search.")
                .font(theme::SMALL)
                .color(p.text_muted)
                .padding(8.0),
        );
    }
    col()
        .key("gif-sheet")
        .w(sheet_w(560.0))
        .h(sheet_h(480.0))
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
                    text("GIFs")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(if mobile() {
                    // Focus a button, not the search field, so the keyboard
                    // does not open over the picker before a search is asked.
                    close_button(p).autofocus()
                } else {
                    close_button(p)
                }),
        )
        .child(
            text_input(&s.gif_query, theme::SMALL)
                .key("gif-search")
                .w_full()
                .h(36.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Search GIFs")
                .placeholder("Search the built-in catalogue", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.gif_query)
                .on_submit(|s: &mut App, _: &mut rune::Tx| s.search_gifs()),
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
        .child(
            text("The catalogue is rendered by your server, so search works without an API key.")
                .wrap()
                .font(theme::TINY)
                .color(p.text_faint),
        )
}
