//! Server scheduled events.

use super::super::prelude::*;
use super::super::*;
use chrono::{Local, TimeZone};
use runecord_protocol::GuildEvent;

pub(crate) fn events_sheet(s: &App, p: Palette) -> El {
    let now = now_ms();
    let mut upcoming: Vec<&GuildEvent> = s
        .guild_events
        .iter()
        .filter(|event| event.start_at >= now)
        .collect();
    let mut past: Vec<&GuildEvent> = s
        .guild_events
        .iter()
        .filter(|event| event.start_at < now)
        .collect();
    upcoming.sort_by_key(|event| event.start_at);
    past.sort_by_key(|event| std::cmp::Reverse(event.start_at));

    let mut list = col().w_full().gap(6.0);
    if upcoming.is_empty() && past.is_empty() {
        list = list.child(
            col()
                .w_full()
                .h(180.0)
                .center()
                .gap(8.0)
                .child(icon(&icons::CLOCK, 40.0).color(p.text_faint))
                .child(
                    text("No events yet")
                        .font(theme::BODY_BOLD)
                        .color(p.text_bright),
                )
                .child(
                    text("Create an event to bring everyone together.")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                ),
        );
    }
    for (label, events) in [("UPCOMING EVENTS", upcoming), ("PAST EVENTS", past)] {
        if events.is_empty() {
            continue;
        }
        list = list.child(
            text(label)
                .font(theme::SECTION)
                .color(p.text_muted)
                .padding(6.0),
        );
        for event in events {
            list = list.child(event_row(s, event, p));
        }
    }

    col()
        .key("events-panel")
        .w(560.0)
        .h(660.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .shadow(16.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .h(58.0)
                .px(16.0)
                .gap(10.0)
                .center_y()
                .child(
                    text("Events")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(
                    el().key("event-create-open")
                        .h(32.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Create event")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(|s: &mut App| s.open_create_event())
                        .child(
                            text("Create Event")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        ),
                )
                .child(close_button(p)),
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
                .padding(12.0)
                .child(list),
        )
}

fn event_row(s: &App, event: &GuildEvent, p: Palette) -> El {
    let timestamp = Local
        .timestamp_millis_opt(event.start_at)
        .single()
        .map(|date| {
            (
                date.format("%b").to_string().to_uppercase(),
                date.format("%d").to_string(),
            )
        })
        .unwrap_or_else(|| ("EVENT".into(), "--".into()));
    let channel = event
        .channel_id
        .and_then(|id| s.channel(id))
        .map(|channel| format!("#{}", channel.name));
    let can_delete = event.creator_id == s.me.id || s.guild_can(perm::MANAGE_GUILD);
    let event_id = event.id;
    col()
        .key(format!("event-{event_id}"))
        .w_full()
        .radius(6.0)
        .fill(p.card)
        .padding(12.0)
        .gap(6.0)
        .child(
            row()
                .w_full()
                .gap(12.0)
                .child(
                    col()
                        .w(48.0)
                        .center()
                        .gap(0.0)
                        .child(text(timestamp.0).font(theme::TINY).color(p.danger))
                        .child(text(timestamp.1).font(theme::TITLE).color(p.text_bright)),
                )
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(
                            text(event.name.clone())
                                .font(theme::BODY_BOLD)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(
                            text(time_label(event.start_at))
                                .font(theme::TINY)
                                .color(p.text_muted),
                        )
                        .children(
                            channel
                                .map(|channel| text(channel).font(theme::TINY).color(p.text_link)),
                        ),
                )
                .children(can_delete.then(|| {
                    el().key(format!("event-delete-{event_id}"))
                        .size(28.0, 28.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Delete {}", event.name))
                        .hover(p.danger.alpha(0.2))
                        .on_press(move |s: &mut App| s.delete_event(event_id))
                        .child(icon(&icons::TRASH, 14.0).color(p.danger))
                })),
        )
        .children((!event.description.is_empty()).then(|| {
            text(event.description.clone())
                .font(theme::SMALL)
                .color(p.text)
                .max_lines(3)
        }))
}

pub(crate) fn create_event_sheet(s: &App, p: Palette) -> El {
    let channel = s
        .channel(s.current_channel_id())
        .map(|channel| format!("#{}", channel.name))
        .unwrap_or_else(|| "No channel".into());
    col()
        .key("create-event")
        .w(460.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(20.0)
        .gap(12.0)
        .shadow(16.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::Events)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text("Create Event")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(text("EVENT NAME").font(theme::SECTION).color(p.text_muted))
        .child(
            text_input(&s.event_name, theme::BODY)
                .key("event-name")
                .w_full()
                .h(38.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Event name")
                .placeholder("Design review", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.event_name),
        )
        .child(text("DESCRIPTION").font(theme::SECTION).color(p.text_muted))
        .child(
            text_input(&s.event_description, theme::BODY)
                .key("event-description")
                .w_full()
                .h(38.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Event description")
                .placeholder("What is this event about?", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.event_description),
        )
        .child(
            text("STARTS IN (MINUTES)")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(
            text_input(&s.event_start_minutes, theme::BODY)
                .key("event-minutes")
                .w(140.0)
                .h(38.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Starts in minutes")
                .placeholder("60", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.event_start_minutes),
        )
        .child(
            text(format!("Location: {channel}"))
                .font(theme::TINY)
                .color(p.text_muted),
        )
        .child(
            row()
                .w_full()
                .justify_end()
                .gap(8.0)
                .child(
                    el().key("event-cancel")
                        .h(34.0)
                        .px(12.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Cancel")
                        .hover(p.card_hover)
                        .on_press(|s: &mut App| s.modal = Modal::Events)
                        .child(text("Cancel").font(theme::LABEL).color(p.text_muted)),
                )
                .child(
                    el().key("event-create")
                        .h(34.0)
                        .px(16.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Create event")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(|s: &mut App| s.create_event_now())
                        .child(
                            text("Create Event")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        ),
                ),
        )
}
