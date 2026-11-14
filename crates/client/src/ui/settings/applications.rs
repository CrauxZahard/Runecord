use super::super::*;
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
                .child(
                    row()
                        .w_full()
                        .gap(8.0)
                        .center_y()
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
                        })),
                )
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
}
