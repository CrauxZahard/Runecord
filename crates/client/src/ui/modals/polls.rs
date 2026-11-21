//! Poll creation sheet.

use super::*;

pub(crate) fn create_poll_sheet(s: &App, p: Palette) -> El {
    let field = |label: &'static str,
                 key: &'static str,
                 edit: &rune::TextEdit,
                 access: fn(&mut App) -> &mut rune::TextEdit| {
        col()
            .w_full()
            .gap(5.0)
            .child(text(label).font(theme::SECTION).color(p.text_muted))
            .child(
                text_input(edit, theme::SMALL)
                    .key(key)
                    .w_full()
                    .h(36.0)
                    .radius(4.0)
                    .fill(p.input)
                    .color(p.text)
                    .role("textbox")
                    .label(label)
                    .placeholder(label, p.text_faint)
                    .padding(10.0)
                    .on_edit(access),
            )
    };
    let toggle = |label: &'static str, key: &'static str, on: bool, run: fn(&mut App)| {
        row()
            .key(key)
            .h(32.0)
            .radius(4.0)
            .px(8.0)
            .gap(8.0)
            .center_y()
            .role("checkbox")
            .label(label)
            .checked(on)
            .hover(p.card_hover)
            .on_press(run)
            .child(
                el().size(16.0, 16.0)
                    .radius(4.0)
                    .fill(if on { p.green } else { p.input })
                    .border(1.0, p.divider)
                    .center()
                    .children(on.then(|| icon(&icons::CHECK, 11.0).color(Color::WHITE))),
            )
            .child(text(label).font(theme::SMALL).color(p.text))
    };
    col()
        .key("poll-sheet")
        .w(480.0)
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
                    text("Create Poll")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Ask a question and give people 2-10 options.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(field(
            "QUESTION",
            "poll-question",
            &s.poll_question,
            |s: &mut App| &mut s.poll_question,
        ))
        .child(field(
            "OPTION 1",
            "poll-option-1",
            &s.poll_option_1,
            |s: &mut App| &mut s.poll_option_1,
        ))
        .child(field(
            "OPTION 2",
            "poll-option-2",
            &s.poll_option_2,
            |s: &mut App| &mut s.poll_option_2,
        ))
        .child(field(
            "OPTION 3 (OPTIONAL)",
            "poll-option-3",
            &s.poll_option_3,
            |s: &mut App| &mut s.poll_option_3,
        ))
        .child(field(
            "OPTION 4 (OPTIONAL)",
            "poll-option-4",
            &s.poll_option_4,
            |s: &mut App| &mut s.poll_option_4,
        ))
        .child(
            row()
                .w_full()
                .gap(12.0)
                .child(toggle(
                    "Allow multiple answers",
                    "poll-multiple",
                    s.poll_multiple,
                    |s: &mut App| s.poll_multiple = !s.poll_multiple,
                ))
                .child(toggle(
                    "Anonymous",
                    "poll-anonymous",
                    s.poll_anonymous,
                    |s: &mut App| s.poll_anonymous = !s.poll_anonymous,
                )),
        )
        .child(field(
            "DURATION IN HOURS",
            "poll-duration",
            &s.poll_duration,
            |s: &mut App| &mut s.poll_duration,
        ))
        .child(
            row()
                .w_full()
                .justify_end()
                .gap(8.0)
                .child(
                    el().key("poll-cancel")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Cancel")
                        .hover(p.card_hover)
                        .on_press(|s: &mut App| s.modal = Modal::None)
                        .child(text("Cancel").font(theme::SMALL).color(p.text_muted)),
                )
                .child(
                    el().key("poll-create")
                        .h(34.0)
                        .px(16.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Create poll")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(|s: &mut App| s.create_poll_now())
                        .child(
                            text("Create Poll")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        ),
                ),
        )
}
