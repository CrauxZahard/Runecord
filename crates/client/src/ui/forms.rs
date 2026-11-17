//! Form and dialog primitives surface.

use super::*;

pub(crate) fn settings_title(title: &str, sub: &str, p: Palette) -> El {
    col()
        .gap(8.0)
        .child(
            text(title.to_string())
                .font(theme::TITLE)
                .color(p.text_bright),
        )
        .child(
            text(sub.to_string())
                .wrap()
                .font(theme::SMALL)
                .color(p.text_muted),
        )
}

pub(crate) fn setting_row(label: &str, value: &str, p: Palette) -> El {
    row()
        .w_full()
        .py(4.0)
        .center_y()
        .child(
            text(label.to_string())
                .font(theme::SMALL_MED)
                .color(p.text_muted)
                .grow(1.0),
        )
        .child(text(value.to_string()).font(theme::SMALL).color(p.text))
}

pub(crate) fn menu_item(
    label: &'static str,
    red: bool,
    key: &'static str,
    p: Palette,
    run: fn(&mut App),
) -> El {
    el().key(key)
        .w_full()
        .h(32.0)
        .px(8.0)
        .radius(4.0)
        .center_y()
        .role("menuitem")
        .label(label)
        .hover(if red { p.danger.alpha(0.12) } else { p.accent })
        .on_press(run)
        .child(
            text(label)
                .font(theme::SMALL_MED)
                .color(if red { p.danger } else { p.text_bright }),
        )
}

pub(crate) fn modal_input(
    edit: &rune::TextEdit,
    key: &'static str,
    placeholder: &'static str,
    p: Palette,
) -> El {
    text_input(edit, theme::BODY)
        .key(key)
        .w_full()
        .h(40.0)
        .radius(4.0)
        .fill(p.rail)
        .color(p.text)
        .role("textbox")
        .label(placeholder)
        .placeholder(placeholder, p.text_faint)
        .padding(10.0)
        .on_edit(|s: &mut App| &mut s.profile_edit)
        .on_submit(|s: &mut App, _: &mut rune::Tx| {
            if s.modal == Modal::CreateServer {
                s.create_guild_now();
            } else if s.modal == Modal::CreateChannel {
                // Honour the channel type the user selected in the picker.
                s.create_channel_now(s.create_channel_kind);
            }
        })
}

pub(crate) fn modal_frame(
    key: &'static str,
    title: &str,
    sub: &str,
    p: Palette,
    body: El,
    action: (&'static str, fn(&mut App)),
    p2: Palette,
) -> El {
    modal_frame_inner(key, title, sub, p, body, action, p2, true)
}

/// A modal whose footer offers only the primary action, as an invite sheet does.
pub(crate) fn modal_frame_done(
    key: &'static str,
    title: &str,
    sub: &str,
    p: Palette,
    body: El,
    action: (&'static str, fn(&mut App)),
    p2: Palette,
) -> El {
    modal_frame_inner(key, title, sub, p, body, action, p2, false)
}

#[allow(clippy::too_many_arguments)]
fn modal_frame_inner(
    key: &'static str,
    title: &str,
    sub: &str,
    p: Palette,
    body: El,
    action: (&'static str, fn(&mut App)),
    _p2: Palette,
    cancel: bool,
) -> El {
    let (label, run) = action;
    let mut footer = row().w_full().h(52.0).gap(8.0).items_center().justify_end();
    if cancel {
        footer = footer.child(
            text("Cancel")
                .font(theme::SMALL_MED)
                .color(p.text_muted)
                .padding(10.0)
                .on_press(|s: &mut App| s.modal = Modal::None),
        );
    }
    footer = footer.child(
        el().h(38.0)
            .px(18.0)
            .radius(4.0)
            .key("modal-action")
            .fill(p.accent)
            .hover(p.accent_hover)
            .center()
            .on_press(run)
            .child(text(label).font(theme::SMALL_MED).color(Color::WHITE)),
    );
    col()
        .key(key)
        .w(sheet_w(440.0))
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .backdrop_blur(4.0)
        .motion(motion::SNAPPY)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            col()
                .w_full()
                .px(20.0)
                .pt(20.0)
                .pb(12.0)
                .gap(6.0)
                .child(
                    row()
                        .w_full()
                        .items_center()
                        .child(
                            text(title.to_string())
                                .font(theme::TITLE)
                                .color(p.text_bright)
                                .grow(1.0),
                        )
                        .child(close_button(p)),
                )
                .child(
                    text(sub.to_string())
                        .w_full()
                        .wrap()
                        .font(theme::SMALL)
                        .color(p.text_muted),
                ),
        )
        .child(
            col()
                .w_full()
                .px(20.0)
                .gap(14.0)
                .child(body)
                .child(footer)
                .child(el().h(8.0)),
        )
}
