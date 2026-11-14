//! View layer: one module per surface, plus shared primitives.

pub(crate) mod prelude {
    pub(crate) use crate::app::{App, Modal, Screen, SettingsTab, Toast};
    pub(crate) use crate::icons;
    pub(crate) use crate::theme::{self, Palette};
    pub(crate) use rune::tok::motion;
    pub(crate) use rune::{
        Color, Cx, El, GrowthSpec, Icon, ScrollSpec, TextStyle, col, el, icon, row, text,
        text_input,
    };
    pub(crate) use runecord_protocol::{
        Channel, ChannelKind, DmChannel, GuildNotificationSettings, GuildSummary, Id, Member,
        Message, NotificationLevel, OverwriteKind, Presence, RelationshipKind, User, has, now_ms,
        perm,
    };
    pub(crate) use std::rc::Rc;
}

use self::prelude::*;

pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette::dark()
    } else {
        Palette::light()
    }
}

pub fn view(s: &App, cx: &mut Cx<App>) -> El {
    let p = palette(s.dark);
    cx.caption_color(if s.dark { Color::WHITE } else { Color::BLACK });
    if !s.logged_in {
        return auth_view(s, p);
    }
    if let Modal::Settings(tab) = s.modal {
        return settings_page(s, cx, p, tab);
    }
    if matches!(s.modal, Modal::ServerSettings | Modal::Roles) {
        return server_settings_page(s, cx, p);
    }
    let shell = match s.screen {
        Screen::Guild => guild_shell(s, cx, p),
        Screen::Dms => dm_shell(s, cx, p),
    };
    let mut root = col().key("app").w_full().h_full().fill(p.chat);
    if s.developer {
        root = root
            .child(col().grow(1.0).min_h(0.0).child(shell))
            .child(developer_bar(s, p));
    } else {
        root = root.child(shell);
    }
    if s.modal == Modal::ServerMenu {
        root = root.child(
            el().key("server-menu-layer")
                .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .items_start()
                .justify_start()
                .child(el().pl(80.0).pt(52.0).child(server_menu(s, p))),
        );
    } else if s.modal != Modal::None {
        root = root.child(modal(s, p));
    }
    if let Some((url, width, height)) = &s.viewing_image {
        let image = if let Some(pixels) = s.images.get(url) {
            el().w_full()
                .h_full()
                .pixels_fit(pixels, rune::SurfaceFit::Fit)
        } else {
            col()
                .w_full()
                .h_full()
                .center()
                .child(icon(&icons::IMAGE, 48.0).color(p.text_faint))
        };
        let preview = rune_kit::image_zoom::Preview::new(
            |s: &mut App| &mut s.zoom,
            [width.max(1.0), height.max(1.0)],
            image,
        )
        .on_close(|s: &mut App, _| s.viewing_image = None);
        root = root.child(preview.build(
            &s.zoom,
            &rune_kit::Kit::neutral(s.dark),
            [1360.0, 860.0],
            cx.is_reduced_motion(),
        ));
    }
    if let Some(t) = &s.toast {
        root = root.child(toast(t, p));
    }
    root
}

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

fn auth_view(s: &App, p: Palette) -> El {
    let register = s.auth_register;
    let title = if register {
        "Create an account"
    } else {
        "Welcome back!"
    };
    let sub = if register {
        "Pick a username and you're in. Everything stays on this machine."
    } else {
        "We're so excited to see you again!"
    };

    let field = |label: &'static str, edit: &rune::TextEdit, key: &'static str| {
        let input = text_input(edit, theme::BODY)
            .key(key)
            .w_full()
            .h(40.0)
            .radius(3.0)
            .fill(p.rail)
            .color(p.text)
            .padding(10.0)
            .single_line()
            .on_edit(move |s: &mut App| match key {
                "auth-user" => &mut s.auth_user,
                "auth-display" => &mut s.auth_display,
                _ => &mut s.auth_pass,
            })
            .on_submit(|s: &mut App, _: &mut rune::Tx| {
                if s.auth_register {
                    s.register();
                } else {
                    s.login();
                }
            });
        let input = if key == "auth-pass" {
            input.secure()
        } else {
            input
        };
        col()
            .gap(8.0)
            .child(text(label).font(theme::SECTION).color(p.text_muted))
            .child(input)
    };

    let mut card = col()
        .key("auth-card")
        .w(480.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(32.0)
        .gap(20.0)
        .shadow(16.0, 48.0, p.shadow)
        .child(text(title).font(theme::HERO).color(p.text_bright))
        .child(text(sub).font(theme::SMALL).color(p.text_muted));

    if register {
        card = card.child(field("DISPLAY NAME", &s.auth_display, "auth-display"));
        card = card.child(field("USERNAME", &s.auth_user, "auth-user"));
        card = card.child(field("PASSWORD", &s.auth_pass, "auth-pass"));
    } else {
        card = card.child(field("USERNAME", &s.auth_user, "auth-user"));
        card = card.child(field("PASSWORD", &s.auth_pass, "auth-pass"));
    }

    if let Some(err) = &s.auth_error {
        card = card.child(text(err.clone()).font(theme::SMALL).color(p.danger));
    }

    card = card.child(
        el().key("auth-submit")
            .w_full()
            .h(44.0)
            .radius(3.0)
            .fill(if s.auth_busy {
                p.accent_hover
            } else {
                p.accent
            })
            .hover(p.accent_hover)
            .center()
            .on_press(|s: &mut App| {
                if s.auth_register {
                    s.register();
                } else {
                    s.login();
                }
            })
            .child(
                text(if s.auth_busy {
                    "Connecting…"
                } else if register {
                    "Create Account"
                } else {
                    "Log In"
                })
                .font(theme::BODY_MED)
                .color(Color::WHITE),
            ),
    );

    card = card.child(
        row()
            .gap(6.0)
            .center_y()
            .child(
                text(if register {
                    "Already have an account?"
                } else {
                    "Need an account?"
                })
                .font(theme::SMALL)
                .color(p.text_muted),
            )
            .child(
                text(if register { "Log In" } else { "Register" })
                    .font(theme::SMALL_MED)
                    .color(p.text_link)
                    .on_press(|s: &mut App| {
                        s.auth_register = !s.auth_register;
                        s.auth_error = None;
                    }),
            ),
    );

    card = card.child(
        col()
            .w_full()
            .radius(4.0)
            .fill(p.rail)
            .padding(12.0)
            .gap(4.0)
            .child(
                text("LOCAL DEMO ACCOUNT")
                    .font(theme::SECTION)
                    .color(p.text_faint),
            )
            .child(
                text("nova  ·  password: runecord")
                    .font(theme::SMALL)
                    .color(p.text_muted),
            )
            .child(
                text("Plus maya, leo, sam, ana, priya, tom and runebot — same password.")
                    .font(theme::TINY)
                    .color(p.text_faint),
            ),
    );

    if !s.connected {
        card = card.child(
            row()
                .gap(6.0)
                .center_y()
                .child(el().size(8.0, 8.0).radius(4.0).fill(p.yellow))
                .child(
                    text(format!("Waiting for the server at {}", s.server_base))
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        );
    }

    col()
        .key("auth")
        .w_full()
        .h_full()
        .fill(p.rail)
        .center()
        .child(card)
}

// ---------------------------------------------------------------------------
// Shell
// ---------------------------------------------------------------------------

fn guild_shell(s: &App, cx: &mut Cx<App>, p: Palette) -> El {
    let mut shell = row()
        .key("shell")
        .w_full()
        .h_full()
        .fill(p.chat)
        .child(rail(s, p))
        .child(sidebar(s, p))
        .child(chat(s, cx, p));
    let in_voice = s
        .current_channel()
        .is_some_and(|channel| channel.kind == ChannelKind::Voice);
    if s.show_members && !in_voice {
        shell = shell.child(member_list(
            s,
            p,
            if cx.chrome_insets().trailing > 0.0 {
                32.0
            } else {
                0.0
            },
        ));
    }
    shell
}

fn dm_shell(s: &App, cx: &mut Cx<App>, p: Palette) -> El {
    row()
        .key("shell")
        .w_full()
        .h_full()
        .fill(p.chat)
        .child(rail(s, p))
        .child(dm_sidebar(s, p))
        .child(dm_main(s, cx, p))
}

// ---------------------------------------------------------------------------
// Server rail
// ---------------------------------------------------------------------------

mod chat;
pub(crate) mod common;
mod composer;
mod dms;
mod forms;
mod members;
mod modals;
mod rail;
mod server_settings;
mod settings;
mod sidebar;

pub(crate) use chat::*;
pub(crate) use common::*;
pub(crate) use composer::*;
pub(crate) use dms::*;
pub(crate) use forms::*;
pub(crate) use members::*;
pub(crate) use modals::*;
pub(crate) use rail::*;
pub(crate) use server_settings::*;
pub(crate) use settings::*;
pub(crate) use sidebar::*;
