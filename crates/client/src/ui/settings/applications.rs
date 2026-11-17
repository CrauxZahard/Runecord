use super::*;

pub(crate) fn application_tab(s: &App, p: Palette) -> El {
    let draft = s.server_url.text.trim().trim_end_matches('/');
    let dirty = draft != s.server_url_saved.trim_end_matches('/');
    let note = if draft == s.server_base.trim_end_matches('/') {
        "This client is using this URL for the current session."
    } else {
        "Saved URLs take effect after Runecord restarts."
    };
    let button = |key: &'static str,
                  label: &'static str,
                  fill: Color,
                  text_color: Color,
                  run: fn(&mut App)| {
        el().key(key)
            .h(34.0)
            .px(14.0)
            .radius(4.0)
            .center()
            .role("button")
            .label(label)
            .fill(fill)
            .hover(fill.alpha(0.85))
            .on_press(run)
            .child(text(label).font(theme::SMALL_MED).color(text_color))
    };
    col()
        .key("application")
        .w_full()
        .gap(16.0)
        .child(settings_title(
            "Application",
            "Where this client connects. Runecord is self-hosted, so you can point it at any compatible server.",
            p,
        ))
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(6.0)
                .child(
                    text("BOTS & APPS")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(
                    text("Bots and applications are created in the Runecord Developer Portal (Soon), just like Discord. They are no longer managed inside the app.")
                        .wrap()
                        .font(theme::SMALL)
                        .color(p.text),
                ),
        )
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(12.0)
                .child(
                    text("GAME OVERLAY")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(
                    text("A small always-on-top card lists who is in your voice channel while you play. It never takes keyboard focus.")
                        .wrap()
                        .font(theme::TINY)
                        .color(p.text_muted),
                )
                .child(
                    row()
                        .key("overlay-toggle")
                        .w_full()
                        .h(36.0)
                        .gap(10.0)
                        .center_y()
                        .role("checkbox")
                        .label("Enable the in-game overlay")
                        .checked(s.overlay_enabled)
                        .hover(p.card_hover)
                        .on_press(|s: &mut App| {
                            let next = !s.overlay_enabled;
                            s.set_overlay_enabled(next)
                        })
                        .child(
                            el().size(16.0, 16.0)
                                .radius(4.0)
                                .border(1.0, p.divider)
                                .fill(if s.overlay_enabled { p.green } else { p.input })
                                .center()
                                .children(s.overlay_enabled.then(|| {
                                    icon(&icons::CHECK, 11.0).color(Color::WHITE)
                                })),
                        )
                        .child(
                            col()
                                .gap(1.0)
                                .child(
                                    text("Show the voice overlay")
                                        .font(theme::SMALL_MED)
                                        .color(p.text_bright),
                                )
                                .child(
                                    text(format!(
                                        "Overlay window: {}",
                                        crate::overlay::availability()
                                    ))
                                    .font(theme::TINY)
                                    .color(p.text_faint),
                                ),
                        ),
                ),
        )
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(12.0)
                .child(
                    text("BACKEND SERVER")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(
                    text("The default is the local server that run.bat starts. A self-hosted deployment can use any http:// or https:// URL.")
                        .wrap()
                        .font(theme::TINY)
                        .color(p.text_muted),
                )
                .child(settings_input(
                    &s.server_url,
                    "server-url",
                    "Backend server URL",
                    p,
                    |s: &mut App| &mut s.server_url,
                ))
                .child({
                    let mut actions = row().w_full().gap(8.0).center_y();
                    if mobile() {
                        // Three buttons exceed a phone's card; let them wrap.
                        actions = actions.flex_wrap();
                    }
                    actions
                        .child(button(
                            "server-url-save",
                            if s.server_checking {
                                "Checking…"
                            } else {
                                "Save Changes"
                            },
                            if dirty && !s.server_checking {
                                p.accent
                            } else {
                                p.rail
                            },
                            if dirty && !s.server_checking {
                                Color::WHITE
                            } else {
                                p.text_faint
                            },
                            |s: &mut App| s.save_server_url(),
                        ))
                        .child(button(
                            "server-url-reset",
                            "Use Local Default",
                            p.rail,
                            p.text,
                            |s: &mut App| s.reset_server_url(),
                        ))
                        .child(button(
                            "app-restart",
                            "Restart Runecord",
                            p.rail,
                            p.text_bright,
                            |s: &mut App| s.restart_app(),
                        ))
                        .child(el().grow(1.0))
                        .children(dirty.then(|| {
                            text("Unsaved changes")
                                .font(theme::TINY)
                                .color(p.yellow)
                        }))
                })
                .children(s.server_config_error.clone().map(|message| {
                    text(format!("Saved setting: {message}"))
                        .wrap()
                        .font(theme::TINY)
                        .color(p.danger)
                }))
                .children(s.server_url_error.clone().map(|message| {
                    text(message)
                        .wrap()
                        .font(theme::TINY)
                        .color(p.danger)
                }))
                .child(el().w_full().h(1.0).fill(p.divider))
                .child(
                    text("CURRENT SESSION")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(
                    text(s.server_base.clone())
                        .font(theme::SMALL)
                        .color(p.text_bright),
                )
                .child(
                    text(note)
                        .font(theme::TINY)
                        .color(p.text_muted),
                ),
        )
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.rail)
                .padding(14.0)
                .gap(6.0)
                .child(
                    text("SELF-HOSTING")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(
                    text("Run the backend on the machine that will host it, expose its HTTP and WebSocket port, then put this URL here. Changing the server signs out every saved profile because session tokens belong to the previous backend.")
                        .wrap()
                        .font(theme::TINY)
                        .color(p.text_muted),
                ),
        )
        .child(updates_card(s, p))
}

/// The notify-only update check: the app never installs anything, it points at
/// the release page.
fn updates_card(s: &App, p: Palette) -> El {
    let current = env!("CARGO_PKG_VERSION");
    let (status, color) = match &s.update_status {
        crate::update::Status::Idle => (format!("Runecord {current} is installed."), p.text_muted),
        crate::update::Status::Checking => ("Checking for updates…".to_string(), p.text_muted),
        crate::update::Status::Latest => (
            format!("Runecord {current} is the newest version."),
            p.green,
        ),
        crate::update::Status::Available(release) => (
            format!("Runecord {} is available.", release.version),
            p.yellow,
        ),
        crate::update::Status::Failed(error) => {
            (format!("Could not check for updates: {error}"), p.danger)
        }
    };
    let mut card = col()
        .key("updates-card")
        .w_full()
        .radius(8.0)
        .fill(p.card)
        .padding(18.0)
        .gap(10.0)
        .child(text("UPDATES").font(theme::SECTION).color(p.text_muted))
        .child(text(status).wrap().font(theme::SMALL).color(color))
        .child(
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .child(
                    el().key("check-updates")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Check for updates")
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .on_press(|s: &mut App| s.check_updates())
                        .child(
                            text(
                                if matches!(&s.update_status, crate::update::Status::Checking) {
                                    "Checking…"
                                } else {
                                    "Check for updates"
                                },
                            )
                            .font(theme::SMALL_MED)
                            .color(p.text_bright),
                        ),
                )
                .children(match &s.update_status {
                    crate::update::Status::Available(release) => Some(
                        el().key("open-release")
                            .h(34.0)
                            .px(14.0)
                            .radius(4.0)
                            .center()
                            .role("link")
                            .label("Open the release page")
                            .fill(p.accent)
                            .hover(p.accent_hover)
                            .on_press_tx({
                                let url = release.url.clone();
                                move |_: &mut App, tx: &mut rune::Tx| {
                                    let _ = tx.open_url(&url);
                                }
                            })
                            .child(text("Download").font(theme::SMALL_MED).color(Color::WHITE)),
                    ),
                    _ => None,
                }),
        );
    if matches!(&s.update_status, crate::update::Status::Available(_)) {
        card = card.child(
            text("Downloading is manual: the installer on the release page replaces this copy and keeps your data.")
                .wrap()
                .font(theme::TINY)
                .color(p.text_muted),
        );
    }
    card
}
