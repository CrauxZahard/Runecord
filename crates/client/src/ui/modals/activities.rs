//! Embedded activities: launcher and the running-session card.

use super::*;

pub(crate) fn activities_sheet(s: &App, p: Palette, channel: Id) -> El {
    let mut list = col().w_full().gap(8.0);
    if s.activities.is_empty() {
        list = list.child(
            text("No activities are available in this server yet.")
                .font(theme::SMALL)
                .color(p.text_muted)
                .padding(8.0),
        );
    }
    for activity in &s.activities {
        let id = activity.id;
        let running = s
            .activity_sessions
            .get(&channel)
            .is_some_and(|session| session.activity.id == id);
        list = list.child(
            row()
                .key(format!("activity-{}", activity.id))
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(12.0)
                .gap(10.0)
                .center_y()
                .child(
                    el().size(36.0, 36.0)
                        .radius(18.0)
                        .fill(theme::color(&format!("#{:06x}", activity.color)))
                        .center()
                        .child(icon(&icons::SPARKLES, 18.0).color(Color::WHITE)),
                )
                .child(
                    col()
                        .gap(2.0)
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text(activity.name.clone())
                                .font(theme::BODY_BOLD)
                                .color(p.text_bright),
                        )
                        .children((!activity.description.is_empty()).then(|| {
                            text(activity.description.clone())
                                .font(theme::TINY)
                                .color(p.text_muted)
                                .wrap()
                        })),
                )
                .child(
                    el().key(format!("activity-start-{id}"))
                        .h(30.0)
                        .px(12.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(if running {
                            format!("Restart {}", activity.name)
                        } else {
                            format!("Start {}", activity.name)
                        })
                        .fill(if running { p.green } else { p.accent })
                        .hover(p.accent_hover)
                        .on_press(move |s: &mut App| s.start_activity(channel, id))
                        .child(
                            text(if running { "Running" } else { "Start" })
                                .font(theme::TINY)
                                .color(Color::WHITE),
                        ),
                ),
        );
    }

    col()
        .key("activities-sheet")
        .w(520.0)
        .h(520.0)
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
                    text("Activities")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Starting an activity shares it with everyone in the channel. It opens in your browser.")
                .font(theme::TINY)
                .color(p.text_muted)
                .wrap(),
        )
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

/// The card shown above a voice channel while an activity is running.
pub(crate) fn activity_card(s: &App, channel: Id, p: Palette) -> Option<El> {
    let session = s.activity_sessions.get(&channel)?.clone();
    let host = s.user_name(session.started_by);
    let can_stop = session.started_by == s.me.id || s.guild_can(perm::MANAGE_CHANNELS);
    let url = session.activity.url.clone();
    let mut actions = row().gap(6.0).center_y();
    if !url.is_empty() {
        actions = actions.child(
            el().key("activity-join")
                .h(28.0)
                .px(12.0)
                .radius(4.0)
                .center()
                .role("button")
                .label(format!("Open {}", session.activity.name))
                .fill(p.accent)
                .hover(p.accent_hover)
                .on_press_tx(move |_: &mut App, tx: &mut rune::Tx| {
                    let _ = tx.open_url(&url);
                })
                .child(text("Join").font(theme::TINY).color(Color::WHITE)),
        );
    }
    if can_stop {
        actions = actions.child(
            el().key("activity-stop")
                .h(28.0)
                .px(10.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Stop the activity")
                .fill(p.input)
                .hover(p.input_hover)
                .on_press(move |s: &mut App| s.stop_activity(channel))
                .child(text("Stop").font(theme::TINY).color(p.text_muted)),
        );
    }
    Some(
        row()
            .key("activity-card")
            .w_full()
            .px(20.0)
            .py(8.0)
            .gap(10.0)
            .center_y()
            .fill(p.sidebar)
            .border_sides([0.0, 0.0, 1.0, 0.0], p.divider_soft)
            .child(
                el().size(30.0, 30.0)
                    .radius(15.0)
                    .fill(theme::color(&format!("#{:06x}", session.activity.color)))
                    .center()
                    .child(icon(&icons::SPARKLES, 15.0).color(Color::WHITE)),
            )
            .child(
                col()
                    .gap(1.0)
                    .grow(1.0)
                    .min_w(0.0)
                    .child(
                        text(session.activity.name.clone())
                            .font(theme::SMALL_MED)
                            .color(p.text_bright),
                    )
                    .child(
                        text(format!("Started by {host}"))
                            .font(theme::TINY)
                            .color(p.text_faint),
                    ),
            )
            .child(actions),
    )
}
