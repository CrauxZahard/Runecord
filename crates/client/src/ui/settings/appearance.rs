use super::super::*;
use super::*;

pub(crate) fn appearance_tab(s: &App, p: Palette) -> El {
    let theme_card = |label: &'static str, dark: bool| {
        let selected = s.dark == dark;
        let preview = if dark {
            Palette::dark()
        } else {
            Palette::light()
        };
        col()
            .key(format!("theme-{label}"))
            .w(240.0)
            .radius(8.0)
            .padding(8.0)
            .gap(8.0)
            .fill(p.card)
            .border(
                if selected { 2.0 } else { 1.0 },
                if selected { p.accent } else { p.divider },
            )
            .on_press(move |s: &mut App| s.set_dark(dark))
            .child(
                row()
                    .w_full()
                    .h(96.0)
                    .radius(4.0)
                    .clip_rounded(4.0)
                    .fill(preview.chat)
                    .child(el().w(20.0).h_full().fill(preview.rail))
                    .child(el().w(52.0).h_full().fill(preview.sidebar))
                    .child(
                        col()
                            .grow(1.0)
                            .padding(8.0)
                            .gap(6.0)
                            .child(el().w_full().h(8.0).radius(4.0).fill(preview.input))
                            .child(el().w_frac(0.7).h(8.0).radius(4.0).fill(preview.input))
                            .child(el().w_frac(0.5).h(8.0).radius(4.0).fill(preview.input)),
                    ),
            )
            .child(
                row()
                    .center_y()
                    .gap(6.0)
                    .child(
                        el().size(18.0, 18.0)
                            .radius(9.0)
                            .border(2.0, if selected { p.accent } else { p.text_faint })
                            .center()
                            .child(el().size(8.0, 8.0).radius(4.0).fill(if selected {
                                p.accent
                            } else {
                                Color::TRANSPARENT
                            })),
                    )
                    .child(text(label).font(theme::SMALL_MED).color(p.text_bright)),
            )
    };

    col()
        .key("appearance")
        .w_full()
        .gap(16.0)
        .child(settings_title("Appearance", "Theme applies instantly and is remembered for this profile.", p))
        .child(text("THEME").font(theme::SECTION).color(p.text_muted))
        .child(row().gap(16.0).child(theme_card("Dark", true)).child(theme_card("Light", false)))
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(12.0)
                .child(text("MESSAGE DISPLAY").font(theme::SECTION).color(p.text_muted))
                .child(settings_switch(
                    "Compact message rows",
                    s.compact,
                    |s: &mut App| s.toggle_compact(!s.compact),
                    p,
                    &rune_kit::Kit::neutral(p.dark),
                ))
                .child(
                    text(if s.compact {
                        "Compact: tighter rows, smaller avatars."
                    } else {
                        "Cozy: grouped messages with full-size avatars."
                    })
                    .font(theme::SMALL)
                    .color(p.text_muted),
                ),
        )
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(12.0)
                .child(text("DEVELOPER").font(theme::SECTION).color(p.text_muted))
                .child(settings_switch(
                    "Developer mode",
                    s.developer,
                    |s: &mut App| s.toggle_developer(!s.developer),
                    p,
                    &rune_kit::Kit::neutral(p.dark),
                ))
                .child(
                    text(
                        "Adds a live status bar with gateway state, cache counts, image memory and snapshot age.",
                    )
                    .font(theme::SMALL)
                    .color(p.text_muted),
                ),
        )
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(12.0)
                .child(text("LOCAL DATA").font(theme::SECTION).color(p.text_muted))
                .child(
                    text("The warm-start snapshot only. Server messages, accounts and uploads stay untouched.")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                )
                .child(
                    el().h(32.0)
                        .px(16.0)
                        .radius(4.0)
                        .role("button")
                        .label("Clear warm-start cache")
                        .fill(p.danger)
                        .hover(p.danger_hover)
                        .center()
                        .on_press(|s: &mut App| {
                            crate::cache::clear(&s.cache_profile);
                            s.toast("Warm-start cache cleared");
                        })
                        .child(
                            text("Clear Warm-Start Cache")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        ),
                ),
        )
}
