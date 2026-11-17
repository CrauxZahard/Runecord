use super::*;

pub(crate) fn account_tab(s: &App, p: Palette) -> El {
    col()
        .key("account")
        .w_full()
        .gap(16.0)
        .child(settings_title(
            "My Account",
            "Local account on this machine.",
            p,
        ))
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(16.0)
                .gap(12.0)
                .child(
                    row()
                        .gap(16.0)
                        .center_y()
                        .child(avatar_status(&s.me, 64.0, p, p.card, true))
                        .child(
                            col()
                                .grow(1.0)
                                .child(
                                    text(s.me.display_name.clone())
                                        .font(theme::TITLE)
                                        .color(p.text_bright),
                                )
                                .child(text(s.me.handle()).font(theme::SMALL).color(p.text_muted)),
                        )
                        .child(
                            el().h(32.0)
                                .px(16.0)
                                .radius(4.0)
                                .role("button")
                                .label("Edit Profile")
                                .fill(p.accent)
                                .hover(p.accent_hover)
                                .center()
                                .on_press(|s: &mut App| s.open_settings(SettingsTab::Profile))
                                .child(
                                    text("Edit Profile")
                                        .font(theme::SMALL_MED)
                                        .color(Color::WHITE),
                                ),
                        ),
                )
                .child(el().w_full().h(1.0).fill(p.divider))
                .child(setting_row("Username", &s.me.username, p))
                .child(setting_row("Display Name", &s.me.display_name, p))
                .child(setting_row("Email", "not set (local account)", p)),
        )
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(16.0)
                .gap(12.0)
                .child(
                    text("Password and Authentication")
                        .font(theme::BODY_BOLD)
                        .color(p.text_bright),
                )
                .child(
                    text(
                        "Changing your password ends sessions on other profiles; this profile stays signed in."
                            .to_string(),
                    )
                    .wrap()
                    .font(theme::SMALL)
                    .color(p.text_muted),
                )
                .child(settings_password(
                    &s.password_current,
                    "password-current",
                    "Current password",
                    p,
                    |s: &mut App| &mut s.password_current,
                ))
                .child(settings_password(
                    &s.password_new,
                    "password-new",
                    "New password",
                    p,
                    |s: &mut App| &mut s.password_new,
                ))
                .child(
                    el().h(32.0)
                        .px(16.0)
                        .radius(4.0)
                        .role("button")
                        .label("Change Password")
                        .fill(p.danger)
                        .hover(p.danger_hover)
                        .center()
                        .on_press(|s: &mut App| s.change_password())
                        .child(
                            text("Change Password")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        ),
                ),
        )
}
