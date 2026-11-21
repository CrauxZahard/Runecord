//! Per-user voice volume.

use super::*;

pub(crate) fn user_volume_sheet(s: &App, p: Palette, user: Id) -> El {
    let name = s.user_name(user);
    let volume = crate::voice::user_volume(user);
    let muted = crate::voice::user_muted(user);
    col()
        .key("user-volume")
        .w(360.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .padding(18.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text(format!("{name} - Volume"))
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Only changes what you hear; the server and other members are unaffected.")
                .font(theme::TINY)
                .color(p.text_faint)
                .wrap(),
        )
        .child(
            row()
                .w_full()
                .gap(10.0)
                .center_y()
                .child(
                    el().key("user-volume-track")
                        .grow(1.0)
                        .h(20.0)
                        .role("slider")
                        .label(format!("{:.0}%", volume * 100.0))
                        .on_drag(move |s: &mut App, drag, _| {
                            let value = (drag.x / 200.0).clamp(0.0, 1.0) * 2.0;
                            s.set_user_volume(user, value);
                        })
                        .child(
                            el().w_full().h(8.0).radius(4.0).fill(p.input).child(
                                el().w((volume / 2.0).clamp(0.0, 1.0) * 200.0)
                                    .h(8.0)
                                    .radius(4.0)
                                    .fill(if muted { p.danger } else { p.accent }),
                            ),
                        ),
                )
                .child(
                    text(format!("{:.0}%", volume * 100.0))
                        .font(theme::SMALL_MED)
                        .color(p.text_bright),
                ),
        )
        .child(
            row()
                .w_full()
                .justify_end()
                .gap(8.0)
                .child(
                    el().key("user-volume-mute")
                        .h(32.0)
                        .px(12.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(if muted {
                            "Unmute for me"
                        } else {
                            "Mute for me"
                        })
                        .fill(if muted { p.green } else { p.input })
                        .hover(p.input_hover)
                        .on_press(move |s: &mut App| s.toggle_user_volume_mute(user))
                        .child(
                            text(if muted {
                                "Unmute for Me"
                            } else {
                                "Mute for Me"
                            })
                            .font(theme::TINY)
                            .color(if muted {
                                Color::WHITE
                            } else {
                                p.text_bright
                            }),
                        ),
                )
                .child(
                    el().key("user-volume-reset")
                        .h(32.0)
                        .px(12.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Reset volume to 100%")
                        .hover(p.card_hover)
                        .on_press(move |s: &mut App| s.set_user_volume(user, 1.0))
                        .child(text("Reset").font(theme::TINY).color(p.text_muted)),
                ),
        )
}
