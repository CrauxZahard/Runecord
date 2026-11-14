use super::*;

pub(crate) fn bans_view(s: &App, p: Palette) -> El {
    col()
        .key("server-bans")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .gap(12.0)
        .child(
            row()
                .w_full()
                .center_y()
                .child(
                    text(format!(
                        "{} banned {}",
                        s.bans.len(),
                        if s.bans.len() == 1 { "user" } else { "users" }
                    ))
                    .font(theme::SECTION)
                    .color(p.text_muted)
                    .grow(1.0),
                )
                .child(
                    text("Banned members cannot rejoin until they are unbanned.")
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        )
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(appeals_list(s, p))
                .child(bans_list(s, p)),
        )
}
