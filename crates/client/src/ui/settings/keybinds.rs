use super::super::*;
use super::*;

pub(crate) fn keybinds_tab(s: &App, p: Palette) -> El {
    let mut list = col().w_full().gap(4.0);
    for (action, label) in crate::app::KEYBIND_ACTIONS {
        let action = *action;
        let chord = s.keybinds.get(action).cloned().unwrap_or_default();
        let recording = s.recording_key.as_deref() == Some(action);
        list = list.child(
            row()
                .key(format!("keybind-row-{action}"))
                .w_full()
                .h(44.0)
                .px(10.0)
                .radius(5.0)
                .hover(p.card_hover)
                .center_y()
                .gap(12.0)
                .child(
                    text(label.to_string())
                        .font(theme::SMALL)
                        .color(p.text)
                        .grow(1.0),
                )
                .child(
                    text(if recording {
                        "Press keys…".to_string()
                    } else {
                        crate::app::display_chord(&chord)
                    })
                    .font(theme::LABEL)
                    .color(if recording { p.yellow } else { p.text_muted })
                    .border(1.0, if recording { p.yellow } else { p.divider })
                    .radius(4.0)
                    .px(8.0)
                    .py(4.0),
                )
                .child(
                    el().h(28.0)
                        .px(12.0)
                        .radius(4.0)
                        .key(format!("keybind-record-{action}"))
                        .role("button")
                        .label(format!("Record {label}"))
                        .center()
                        .fill(if recording { p.accent } else { p.rail })
                        .hover(p.rail_hover)
                        .on_press(move |s: &mut App| s.begin_keybind(action))
                        .child(
                            text(if recording { "Recording…" } else { "Record" })
                                .font(theme::LABEL)
                                .color(if recording {
                                    Color::WHITE
                                } else {
                                    p.text_bright
                                }),
                        ),
                ),
        );
    }
    col()
        .key("keybinds")
        .w_full()
        .gap(16.0)
        .child(settings_title(
            "Keybinds",
            "Click Record, then press a chord. Bindings save with the profile.",
            p,
        ))
        .child(list)
        .child(
            text("The composer keeps Ctrl+Enter to send and Enter for a newline.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(12.0)
                .child(text("PREFERENCES").font(theme::SECTION).color(p.text_muted))
                .child(
                    text("Restore the default theme, notification toggles, row density, developer mode and keybinds.")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                )
                .child(
                    row()
                        .w_full()
                        .gap(8.0)
                        .child(
                            el().h(32.0)
                                .px(16.0)
                                .radius(4.0)
                                .role("button")
                                .label("Reset keybinds")
                                .fill(p.rail)
                                .hover(p.rail_hover)
                                .center()
                                .on_press(|s: &mut App| s.reset_keybinds())
                                .child(
                                    text("Reset Keybinds")
                                        .font(theme::SMALL_MED)
                                        .color(p.text_bright),
                                ),
                        )
                        .child(
                            el().h(32.0)
                                .px(16.0)
                                .radius(4.0)
                                .role("button")
                                .label("Reset preferences")
                                .fill(p.rail)
                                .hover(p.rail_hover)
                                .center()
                                .on_press(|s: &mut App| s.reset_preferences())
                                .child(
                                    text("Reset Preferences")
                                        .font(theme::SMALL_MED)
                                        .color(p.text_bright),
                                ),
                        ),
                ),
        )
}
