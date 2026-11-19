//! Modal sheets: profile.

use super::*;

pub(crate) fn crop_image_sheet(s: &App, p: Palette, banner: bool, user: bool, group: bool) -> El {
    let (width, height) = if banner {
        (400.0, 225.0)
    } else {
        (320.0, 320.0)
    };
    let preview = if let Some(pixels) = &s.crop_preview {
        el().w_full()
            .h_full()
            .clip_rounded(6.0)
            .pixels_fit(pixels, rune::SurfaceFit::Fill)
    } else {
        el().w_full().h_full().fill(p.rail).center().child(
            text("Loading image…")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
    };
    let pan = |label: &'static str, key: &'static str, dx: f32, dy: f32, p: Palette| {
        el().key(key)
            .size(32.0, 32.0)
            .radius(4.0)
            .center()
            .role("button")
            .label(label)
            .fill(p.rail)
            .hover(p.rail_hover)
            .on_press(move |s: &mut App| s.crop_pan(dx, dy))
            .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
    };

    col()
        .key("crop-guild-image")
        .w(500.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(18.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.cancel_guild_image_crop())
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text(if group {
                        "Crop Group Icon"
                    } else {
                        match (user, banner) {
                            (true, true) => "Crop Profile Banner",
                            (true, false) => "Crop Profile Avatar",
                            (false, true) => "Crop Server Banner",
                            (false, false) => "Crop Server Icon",
                        }
                    })
                    .font(theme::TITLE)
                    .color(p.text_bright)
                    .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text(if banner {
                "Drag to reposition, then zoom until the 16:9 crop looks right."
            } else {
                "Drag to reposition, then zoom until the square crop looks right."
            })
            .font(theme::SMALL)
            .color(p.text_muted),
        )
        .child(
            el().w(width)
                .h(height)
                .radius(6.0)
                .clip_rounded(6.0)
                .fill(p.rail)
                .role("group")
                .label("Crop preview")
                .on_drag(|s: &mut App, drag, _| {
                    if drag.phase == rune::Phase::Down {
                        s.crop_drag_origin = s.crop_offset;
                    } else if drag.phase == rune::Phase::Move {
                        let origin = s.crop_drag_origin;
                        s.crop_drag_move(drag.dx, drag.dy, origin);
                    }
                })
                .child(preview),
        )
        .child(
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .child(
                    el().key("crop-zoom-out")
                        .size(32.0, 32.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Zoom out")
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .on_press(|s: &mut App| s.crop_zoom_by(-0.1))
                        .child(text("−").font(theme::TITLE).color(p.text_bright)),
                )
                .child(
                    text(format!("{:.0}%", s.crop_zoom * 100.0))
                        .font(theme::SMALL_MED)
                        .color(p.text_bright),
                )
                .child(
                    el().key("crop-zoom-in")
                        .size(32.0, 32.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Zoom in")
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .on_press(|s: &mut App| s.crop_zoom_by(0.1))
                        .child(text("+").font(theme::TITLE).color(p.text_bright)),
                )
                .child(
                    el().key("crop-reset")
                        .h(30.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Reset crop")
                        .hover(p.card_hover)
                        .on_press(|s: &mut App| s.crop_reset())
                        .child(text("Reset").font(theme::LABEL).color(p.text_muted)),
                )
                .child(el().grow(1.0))
                .child(pan("←", "crop-left", -20.0, 0.0, p))
                .child(pan("↑", "crop-up", 0.0, -20.0, p))
                .child(pan("↓", "crop-down", 0.0, 20.0, p))
                .child(pan("→", "crop-right", 20.0, 0.0, p)),
        )
        .child(
            row()
                .w_full()
                .justify_end()
                .gap(8.0)
                .child(
                    el().key("crop-cancel")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Cancel crop")
                        .hover(p.card_hover)
                        .on_press(|s: &mut App| s.cancel_guild_image_crop())
                        .child(text("Cancel").font(theme::SMALL_MED).color(p.text_muted)),
                )
                .child(
                    el().key("crop-apply")
                        .h(34.0)
                        .px(16.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Apply crop")
                        .fill(p.green)
                        .hover(p.green.alpha(0.85))
                        .on_press(|s: &mut App| s.confirm_guild_image_crop())
                        .child(text("Apply").font(theme::SMALL_MED).color(Color::WHITE)),
                ),
        )
}

pub(crate) fn profile_more_sheet(s: &App, p: Palette, id: Id) -> El {
    let relationship = s
        .relationships
        .iter()
        .find(|relationship| relationship.user.id == id)
        .map(|relationship| relationship.kind);
    let item = |label: &'static str, key: &'static str, red: bool, p: Palette| {
        el().key(key)
            .w_full()
            .h(34.0)
            .px(8.0)
            .radius(4.0)
            .center_y()
            .role("button")
            .label(label)
            .hover(if red {
                p.danger.alpha(0.18)
            } else {
                p.card_hover
            })
            .child(text(label).font(theme::SMALL_MED).color(if red {
                p.danger
            } else {
                p.text_bright
            }))
    };
    let mut list = col()
        .key("profile-more")
        .w(240.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(6.0)
        .gap(1.0)
        .shadow(8.0, 24.0, p.shadow)
        .modal(true, move |s: &mut App| s.modal = Modal::Profile(id))
        .child(
            item("Copy User ID", "profile-more-copy", false, p).on_press_tx(
                move |_: &mut App, tx: &mut rune::Tx| {
                    tx.copy(id.to_string());
                },
            ),
        );
    match relationship {
        Some(RelationshipKind::Friend) => {
            list = list.child(
                item("Remove Friend", "profile-more-remove", true, p).on_press(
                    move |s: &mut App| {
                        s.decline_friend(id);
                        s.modal = Modal::Profile(id);
                    },
                ),
            );
        }
        Some(RelationshipKind::Outgoing) => {
            list = list.child(
                item("Cancel Friend Request", "profile-more-cancel", false, p).on_press(
                    move |s: &mut App| {
                        s.decline_friend(id);
                        s.modal = Modal::Profile(id);
                    },
                ),
            );
        }
        Some(RelationshipKind::Incoming) => {
            list = list.child(
                item("Accept Friend Request", "profile-more-accept", false, p).on_press(
                    move |s: &mut App| {
                        s.accept_friend(id);
                        s.modal = Modal::Profile(id);
                    },
                ),
            );
        }
        _ => {}
    }
    list = list.child(item("Block User", "profile-more-block", true, p).on_press(
        move |s: &mut App| {
            s.block_user(id);
            s.modal = Modal::Profile(id);
        },
    ));
    list.child(el().w_full().h(1.0).fill(p.divider).my(4.0))
        .child(
            item("Close", "profile-more-close", false, p)
                .on_press(move |s: &mut App| s.modal = Modal::Profile(id)),
        )
}

pub(crate) fn video_quality_sheet(s: &App, p: Palette) -> El {
    let options = [
        (
            VideoQuality::Auto,
            "Auto",
            "Adapts to congestion, starts at 720p and steps down when frames back up.",
        ),
        (VideoQuality::P360, "360p", "640×360 · 15 fps · ~500 kbps"),
        (VideoQuality::P480, "480p", "854×480 · 24 fps · ~900 kbps"),
        (VideoQuality::P720, "720p", "1280×720 · 30 fps · ~1.8 Mbps"),
        (
            VideoQuality::P1080,
            "1080p",
            "1920×1080 · 30 fps · ~3.5 Mbps",
        ),
    ];
    let mut rows = col().w_full().gap(2.0);
    for (quality, label, description) in options {
        let selected = s.video_quality == quality;
        rows = rows.child(
            row()
                .key(format!("video-quality-{}", quality.key()))
                .w_full()
                .min_h(52.0)
                .px(10.0)
                .py(6.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .role("radio")
                .label(label)
                .checked(selected)
                .fill(if selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.set_video_quality(quality))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
                        .child(text(description).font(theme::TINY).color(p.text_muted)),
                )
                .children(selected.then(|| icon(&icons::CHECK, 18.0).color(p.green))),
        );
    }

    col()
        .key("video-quality")
        .w(440.0)
        .radius(8.0)
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
                    text("Video Quality")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Applies to your camera and screen share. The receiver shows the sender’s resolution and frame rate.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(rows)
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            text("RECEIVE QUALITY")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(
            row()
                .w_full()
                .gap(6.0)
                .children([ViewQuality::Auto, ViewQuality::High, ViewQuality::Low].map(
                    |quality| {
                        let selected = s.viewer_quality == quality;
                        el().key(format!("view-quality-{}", quality.key()))
                            .h(34.0)
                            .px(12.0)
                            .radius(4.0)
                            .center()
                            .role("radio")
                            .label(quality.label())
                            .checked(selected)
                            .fill(if selected { p.selected } else { p.rail })
                            .hover(p.rail_hover)
                            .on_press(move |s: &mut App| s.set_viewer_quality(quality))
                            .child(
                                text(quality.label())
                                    .font(theme::SMALL_MED)
                                    .color(if selected { p.text_bright } else { p.text_muted }),
                            )
                    },
                )),
        )
        .child(
            text("Auto uses the low simulcast layer when decoding falls behind.")
                .font(theme::TINY)
                .color(p.text_faint),
        )
}

/// Discord-style status picker opened from the user panel.
pub(crate) fn status_picker_sheet(s: &App, p: Palette) -> El {
    use runecord_protocol::Presence;
    let item =
        |key: &'static str,
         status: Presence,
         label: &'static str,
         detail: Option<&'static str>,
         dot: Color,
         p: Palette| {
            let selected = s.my_presence == status;
            row()
                .key(key)
                .w_full()
                .min_h(38.0)
                .px(10.0)
                .py(6.0)
                .gap(10.0)
                .center_y()
                .role("radio")
                .label(label)
                .checked(selected)
                .fill(if selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.rail_hover)
                .on_press(move |s: &mut App| {
                    s.set_my_presence(status);
                    s.modal = Modal::None;
                })
                .child(el().size(12.0, 12.0).radius(6.0).fill(dot))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(1.0)
                        .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
                        .children(detail.map(|detail| {
                            text(detail).wrap().font(theme::TINY).color(p.text_muted)
                        })),
                )
                .children(selected.then(|| icon(&icons::CHECK, 14.0).color(p.green)))
        };
    col()
        .key("status-picker")
        .w(260.0)
        .radius(8.0)
        .fill(theme::color("#111214"))
        .padding(6.0)
        .gap(1.0)
        .shadow(8.0, 24.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            text("SET STATUS")
                .font(theme::SECTION)
                .color(p.text_faint)
                .padding(8.0),
        )
        .child(item(
            "status-online",
            Presence::Online,
            "Online",
            None,
            p.green,
            p,
        ))
        .child(item(
            "status-idle",
            Presence::Idle,
            "Idle",
            None,
            p.yellow,
            p,
        ))
        .child(item(
            "status-dnd",
            Presence::Dnd,
            "Do Not Disturb",
            Some("You will not receive desktop notifications"),
            p.danger,
            p,
        ))
        .child(item(
            "status-invisible",
            Presence::Offline,
            "Invisible",
            Some("You will appear offline"),
            p.text_faint,
            p,
        ))
}
