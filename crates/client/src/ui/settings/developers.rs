use super::super::*;
use super::*;

pub(crate) fn bots_apps_tab(s: &App, p: Palette) -> El {
    let mut body = col()
        .key("bots-apps")
        .w_full()
        .gap(18.0)
        .child(settings_title(
            "Bots & Apps",
            "Owned bots, application tokens and server installs. Tokens are shown once.",
            p,
        ));

    if let Some((label, token)) = &s.token_reveal {
        let copy = token.clone();
        let secret = token.clone();
        body = body.child(
            col()
                .key("token-reveal")
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .border(1.0, p.yellow)
                .padding(18.0)
                .gap(10.0)
                .child(
                    text("COPY THIS TOKEN NOW")
                        .font(theme::SECTION)
                        .color(p.yellow),
                )
                .child(text(label.clone()).font(theme::SMALL).color(p.text_muted))
                .child(
                    text(secret)
                        .font(TextStyle::new(12.0, 400.0).mono())
                        .color(p.text_bright)
                        .selectable(true)
                        .wrap(),
                )
                .child(
                    row()
                        .gap(8.0)
                        .child(
                            el().h(30.0)
                                .px(14.0)
                                .radius(4.0)
                                .key("token-copy")
                                .role("button")
                                .label("Copy token")
                                .fill(p.accent)
                                .hover(p.accent_hover)
                                .center()
                                .on_press_tx(move |s: &mut App, tx: &mut rune::Tx| {
                                    tx.copy(copy.clone());
                                    s.token_reveal = None;
                                    s.toast("Token copied");
                                })
                                .child(
                                    text("Copy Token")
                                        .font(theme::SMALL_MED)
                                        .color(Color::WHITE),
                                ),
                        )
                        .child(
                            el().h(30.0)
                                .px(14.0)
                                .radius(4.0)
                                .key("token-dismiss")
                                .role("button")
                                .label("Dismiss token")
                                .fill(p.rail)
                                .hover(p.rail_hover)
                                .center()
                                .on_press(|s: &mut App| s.token_reveal = None)
                                .child(text("Dismiss").font(theme::SMALL_MED).color(p.text_bright)),
                        ),
                ),
        );
    }

    let guild_name = s
        .active_guild()
        .map(|guild| guild.name.clone())
        .unwrap_or_else(|| "this server".to_string());
    let add_to_guild = s.add_bot_to_guild;
    let mut bots = col()
        .w_full()
        .radius(8.0)
        .fill(p.card)
        .padding(18.0)
        .gap(12.0)
        .child(text("BOTS").font(theme::SECTION).color(p.text_muted))
        .child(
            row()
                .w_full()
                .gap(8.0)
                .child(col().grow(1.0).min_w(0.0).child(settings_input(
                    &s.bot_name,
                    "bot-name",
                    "Bot name",
                    p,
                    |s: &mut App| &mut s.bot_name,
                )))
                .child(
                    el().h(38.0)
                        .px(16.0)
                        .radius(4.0)
                        .key("bot-create")
                        .role("button")
                        .label("Create bot")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .center()
                        .on_press(|s: &mut App| s.create_bot())
                        .child(
                            text("Create Bot")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        ),
                ),
        )
        .child(
            row()
                .w_full()
                .h(30.0)
                .gap(8.0)
                .center_y()
                .key("bot-add-guild")
                .role("checkbox")
                .label(format!("Add new bots to {guild_name}"))
                .checked(add_to_guild)
                .on_press(|s: &mut App| s.add_bot_to_guild = !s.add_bot_to_guild)
                .child(
                    icon(
                        if add_to_guild {
                            &icons::CHECK
                        } else {
                            &icons::CLOSE
                        },
                        14.0,
                    )
                    .color(if add_to_guild { p.green } else { p.text_faint }),
                )
                .child(
                    text(format!("Add new bots to {guild_name}"))
                        .font(theme::SMALL)
                        .color(p.text_muted),
                ),
        );
    let attached_bots: Vec<Id> = s.applications.iter().map(|app| app.bot.id).collect();
    let visible_bots: Vec<&User> = s
        .bots
        .iter()
        .filter(|bot| !attached_bots.contains(&bot.id))
        .collect();
    if visible_bots.is_empty() {
        bots = bots.child(
            text("No bots yet.")
                .font(theme::SMALL)
                .color(p.text_faint)
                .padding(4.0),
        );
    }
    for bot in visible_bots {
        let id = bot.id;
        bots = bots.child(
            row()
                .key(format!("bot-{id}"))
                .w_full()
                .h(46.0)
                .gap(10.0)
                .center_y()
                .child(avatar(bot, 28.0, p))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text(bot.display_name.clone())
                                .font(theme::SMALL_MED)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(
                            text(bot.handle())
                                .font(theme::TINY)
                                .color(p.text_muted)
                                .truncate(),
                        ),
                )
                .child(bot_tag(p))
                .child(
                    el().h(28.0)
                        .px(12.0)
                        .radius(4.0)
                        .key(format!("bot-revoke-{id}"))
                        .role("button")
                        .label(format!("Revoke {}", bot.display_name))
                        .fill(p.rail)
                        .hover(p.danger.alpha(0.2))
                        .center()
                        .on_press(move |s: &mut App| s.revoke_bot(id))
                        .child(text("Revoke").font(theme::LABEL).color(p.danger)),
                ),
        );
    }
    body = body.child(bots);

    let mut apps = col()
        .w_full()
        .radius(8.0)
        .fill(p.card)
        .padding(18.0)
        .gap(12.0)
        .child(
            text("APPLICATIONS")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(settings_input(
            &s.app_name,
            "app-name",
            "Application name",
            p,
            |s: &mut App| &mut s.app_name,
        ))
        .child(settings_input(
            &s.app_description,
            "app-description",
            "Short description",
            p,
            |s: &mut App| &mut s.app_description,
        ))
        .child(
            row().w_full().justify_end().child(
                el().h(32.0)
                    .px(16.0)
                    .radius(4.0)
                    .key("app-create")
                    .role("button")
                    .label("Create application")
                    .fill(p.accent)
                    .hover(p.accent_hover)
                    .center()
                    .on_press(|s: &mut App| s.create_application())
                    .child(
                        text("Create Application")
                            .font(theme::SMALL_MED)
                            .color(Color::WHITE),
                    ),
            ),
        );
    if s.applications.is_empty() {
        apps = apps.child(
            text("No applications yet.")
                .font(theme::SMALL)
                .color(p.text_faint)
                .padding(4.0),
        );
    }
    for app in &s.applications {
        let id = app.id;
        let installable = s.active_guild != 0;
        apps = apps.child(
            col()
                .key(format!("app-{id}"))
                .w_full()
                .gap(8.0)
                .padding(10.0)
                .radius(6.0)
                .fill(p.rail)
                .child(
                    row()
                        .gap(10.0)
                        .center_y()
                        .child(
                            el().size(28.0, 28.0)
                                .radius(8.0)
                                .fill(theme::color(&format!("#{:06x}", app.icon_color))),
                        )
                        .child(
                            col()
                                .grow(1.0)
                                .min_w(0.0)
                                .child(
                                    text(app.name.clone())
                                        .font(theme::SMALL_MED)
                                        .color(p.text_bright)
                                        .truncate(),
                                )
                                .child(
                                    text(if app.description.is_empty() {
                                        "No description".to_string()
                                    } else {
                                        app.description.clone()
                                    })
                                    .font(theme::TINY)
                                    .color(p.text_muted)
                                    .truncate(),
                                ),
                        )
                        .child(
                            text(format!("bot {}", app.bot.display_name))
                                .font(theme::TINY)
                                .color(p.text_faint),
                        ),
                )
                .child(
                    row()
                        .gap(6.0)
                        .child(
                            el().h(28.0)
                                .px(10.0)
                                .radius(4.0)
                                .key(format!("app-install-{id}"))
                                .role("button")
                                .label(format!("Install {} in this server", app.name))
                                .fill(if installable { p.green } else { p.divider })
                                .hover(if installable {
                                    p.green.alpha(0.8)
                                } else {
                                    p.divider
                                })
                                .center()
                                .disabled(!installable)
                                .on_press(move |s: &mut App| s.create_install_link(id))
                                .child(
                                    text("Install in Server")
                                        .font(theme::LABEL)
                                        .color(Color::WHITE),
                                ),
                        )
                        .child(
                            el().h(28.0)
                                .px(10.0)
                                .radius(4.0)
                                .key(format!("app-token-{id}"))
                                .role("button")
                                .label(format!("New token for {}", app.name))
                                .fill(p.rail_hover)
                                .hover(p.input_hover)
                                .center()
                                .on_press(move |s: &mut App| s.reveal_application_token(id))
                                .child(text("New Token").font(theme::LABEL).color(p.text_bright)),
                        )
                        .child(
                            el().h(28.0)
                                .px(10.0)
                                .radius(4.0)
                                .key(format!("app-delete-{id}"))
                                .role("button")
                                .label(format!("Delete {}", app.name))
                                .fill(p.rail_hover)
                                .hover(p.danger.alpha(0.2))
                                .center()
                                .on_press(move |s: &mut App| s.delete_application(id))
                                .child(text("Delete").font(theme::LABEL).color(p.danger)),
                        ),
                ),
        );
    }
    body = body.child(apps);

    body.child(
        col()
            .w_full()
            .radius(8.0)
            .fill(p.card)
            .padding(18.0)
            .gap(12.0)
            .child(
                text("INSTALL BY CODE")
                    .font(theme::SECTION)
                    .color(p.text_muted),
            )
            .child(
                text(format!(
                    "Installs into {guild_name}. You need Manage Server there."
                ))
                .font(theme::SMALL)
                .color(p.text_muted),
            )
            .child(
                row()
                    .w_full()
                    .gap(8.0)
                    .child(col().grow(1.0).min_w(0.0).child(settings_input(
                        &s.install_code,
                        "install-code",
                        "Install code",
                        p,
                        |s: &mut App| &mut s.install_code,
                    )))
                    .child(
                        el().h(38.0)
                            .px(16.0)
                            .radius(4.0)
                            .key("install-apply")
                            .role("button")
                            .label("Install application")
                            .fill(p.accent)
                            .hover(p.accent_hover)
                            .center()
                            .on_press(|s: &mut App| s.install_by_code())
                            .child(text("Install").font(theme::SMALL_MED).color(Color::WHITE)),
                    ),
            ),
    )
}
