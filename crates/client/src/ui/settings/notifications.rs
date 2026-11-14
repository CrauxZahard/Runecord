use super::super::*;
use super::*;

pub(crate) fn notifications_tab(s: &App, p: Palette) -> El {
    let kit = rune_kit::Kit::neutral(p.dark);
    let switch_row =
        |label: &'static str, on: bool, p: Palette, kit: &rune_kit::Kit, toggle: fn(&mut App)| {
            row()
                .w_full()
                .h(40.0)
                .center_y()
                .gap(16.0)
                .child(text(label).font(theme::SMALL).color(p.text).grow(1.0))
                .child(
                    rune_kit::Switch::new(on)
                        .label(label)
                        .hide_label()
                        .on_toggle(toggle)
                        .build(kit),
                )
        };
    col()
        .key("notifications")
        .w_full()
        .gap(16.0)
        .child(settings_title(
            "Notifications",
            "Desktop toasts use Windows notifications while the window is unfocused.",
            p,
        ))
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(16.0)
                .gap(12.0)
                .child(switch_row(
                    "Notify for mentions",
                    s.notify_mentions,
                    p,
                    &kit,
                    |s: &mut App| {
                        s.notify_mentions = !s.notify_mentions;
                        s.mark_cache_dirty();
                    },
                ))
                .child(switch_row(
                    "Notify for direct messages",
                    s.notify_dm,
                    p,
                    &kit,
                    |s: &mut App| {
                        s.notify_dm = !s.notify_dm;
                        s.mark_cache_dirty();
                    },
                ))
                .child(
                    text("System notifications are silent in this build; there is no audio transport yet.")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                ),
        )
}
