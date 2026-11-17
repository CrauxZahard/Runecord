//! Scheduled message sheet.

use super::*;

pub(crate) fn scheduled_sheet(s: &App, p: Palette) -> El {
    let mut list = col().w_full().gap(2.0);
    if s.scheduled.is_empty() {
        list = list.child(
            text("Nothing scheduled in this channel.")
                .font(theme::SMALL)
                .color(p.text_faint)
                .padding(8.0),
        );
    }
    for item in &s.scheduled {
        let id = item.id;
        let when = date_label(item.send_at);
        list = list.child(
            row()
                .key(format!("scheduled-{}", item.id))
                .w_full()
                .min_h(44.0)
                .px(8.0)
                .py(6.0)
                .radius(4.0)
                .gap(8.0)
                .center_y()
                .hover(p.card_hover)
                .child(icon(&icons::CLOCK, 16.0).color(p.text_muted))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text(item.content.clone())
                                .font(theme::SMALL)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(text(when).font(theme::TINY).color(p.text_faint)),
                )
                .child(
                    el().key(format!("scheduled-delete-{}", item.id))
                        .size(28.0, 28.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Cancel scheduled message")
                        .hover(p.danger.alpha(0.2))
                        .on_press(move |s: &mut App| s.delete_scheduled(id))
                        .child(icon(&icons::TRASH, 15.0).color(p.danger)),
                ),
        );
    }
    col()
        .key("scheduled-sheet")
        .w(sheet_w(500.0))
        .h(sheet_h(560.0))
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
                    text("Scheduled Messages")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Write now, send later. Times are shown in your local timezone.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            text_input(&s.schedule_content, theme::SMALL)
                .key("schedule-content")
                .w_full()
                .h(64.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Scheduled message")
                .placeholder("Message", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.schedule_content),
        )
        .child(
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .child(text("SEND IN").font(theme::SECTION).color(p.text_muted))
                .child(
                    text_input(&s.schedule_minutes, theme::SMALL)
                        .key("schedule-minutes")
                        .w(90.0)
                        .h(34.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Minutes")
                        .placeholder("60", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.schedule_minutes),
                )
                .child(text("minutes").font(theme::TINY).color(p.text_faint))
                .child(el().grow(1.0))
                .child(
                    el().key("schedule-create")
                        .h(34.0)
                        .px(16.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Schedule message")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(|s: &mut App| s.schedule_now())
                        .child(text("Schedule").font(theme::SMALL_MED).color(Color::WHITE)),
                ),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list),
        )
}
