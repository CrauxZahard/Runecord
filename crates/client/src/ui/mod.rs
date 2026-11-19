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
        Channel, ChannelKind, DmChannel, GuildSummary, Id, Member, Message, NotificationLevel,
        OverwriteKind, Presence, RelationshipKind, User, has, now_ms, perm,
    };
    pub(crate) use std::rc::Rc;
}

use self::prelude::*;

/// Height of the in-app titlebar the window's caption buttons sit over.
pub const TITLEBAR_HEIGHT: f32 = 36.0;

/// Height of the navigation dock that runs under the titlebar.
pub const DOCK_HEIGHT: f32 = 58.0;

pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette::dark()
    } else {
        Palette::light()
    }
}

pub fn view(s: &App, cx: &mut Cx<App>) -> El {
    let p = palette(s.dark);
    // Keyboard focus for every pressable control: Tab reaches the form fields
    // and buttons.
    cx.web_focus();
    cx.caption_color(if s.dark { Color::WHITE } else { Color::BLACK });
    let mut root = col()
        .key("app")
        .w_full()
        .h_full()
        .fill(p.chat)
        .child(title_bar(s, cx, p));
    if !s.logged_in {
        if let Modal::Settings(tab) = s.modal {
            root = root.child(settings_page(s, cx, p, tab));
        } else {
            root = root.child(auth_view(s, cx, p));
        }
        if let Some(t) = &s.toast {
            root = root.child(toast(t, p));
        }
        return root;
    }
    if let Modal::Settings(tab) = s.modal {
        root = root.child(settings_page(s, cx, p, tab));
        if let Some(t) = &s.toast {
            root = root.child(toast(t, p));
        }
        return root;
    }
    if matches!(s.modal, Modal::ServerSettings | Modal::Roles) && s.can_open_server_settings() {
        root = root.child(server_settings_page(s, cx, p));
        if let Some(t) = &s.toast {
            root = root.child(toast(t, p));
        }
        return root;
    }
    let shell = match s.screen {
        Screen::Guild => guild_shell(s, cx, p),
        Screen::Dms => dm_shell(s, cx, p),
    };
    if s.developer {
        root = root
            .child(col().grow(1.0).min_h(0.0).child(shell))
            .child(developer_bar(s, p));
    } else {
        root = root.child(col().grow(1.0).min_h(0.0).child(shell));
    }
    if s.modal == Modal::ServerMenu {
        root = root.child(
            el().key("server-menu-layer")
                .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .items_start()
                .justify_start()
                .child(
                    el().pl(16.0)
                        .pt(TITLEBAR_HEIGHT + DOCK_HEIGHT + 18.0)
                        .child(server_menu(s, p)),
                ),
        );
    } else if s.modal != Modal::None {
        root = root.child(modal(s, p));
    }
    // Context menus sit above modals so a post menu works inside the thread
    // browser.
    if let Some(menu) = &s.context_menu {
        root = root.child(context_menu_layer(s, cx, menu, p));
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
// Titlebar
// ---------------------------------------------------------------------------

fn title_bar(s: &App, cx: &Cx<App>, p: Palette) -> El {
    let insets = cx.chrome_insets();
    let section = if matches!(s.modal, Modal::Settings(_)) {
        "User Settings".to_string()
    } else if matches!(s.modal, Modal::ServerSettings | Modal::Roles) {
        "Server Settings".to_string()
    } else if !s.logged_in {
        "Welcome".to_string()
    } else if s.screen == Screen::Dms {
        "Direct Messages".to_string()
    } else {
        s.active_guild()
            .map(|guild| guild.name.clone())
            .unwrap_or_else(|| "Servers".into())
    };
    let (dot, status) = if s.connected {
        (p.green, "Connected")
    } else if s.connecting {
        (p.yellow, "Connecting")
    } else {
        (p.text_faint, "Offline")
    };
    // The bar itself is the dusk sky: violet into magenta, ruled along the
    // bottom by the amber head of the same sunset.
    let bar = row()
        .key("titlebar")
        .w_full()
        .h(TITLEBAR_HEIGHT)
        .pl(insets.leading + 12.0)
        .pr(insets.trailing + 12.0)
        .gap(9.0)
        .center_y()
        .fill(p.rail)
        .gradient(theme::sunset_violet(p.dark), theme::sunset_magenta(p.dark))
        .border_sides(
            [0.0, 0.0, 2.0, 0.0],
            theme::sunset_amber(p.dark).alpha(0.85),
        )
        .window_drag()
        .child(
            el().size(22.0, 22.0)
                .radius(11.0)
                .fill(theme::sunset_amber(p.dark))
                .gradient(theme::sunset_amber(p.dark), theme::sunset_magenta(p.dark))
                .border(1.0, Color::WHITE.alpha(0.6))
                .center()
                .child(
                    text("K")
                        .font(TextStyle::new(12.0, 800.0))
                        .color(p.accent_text),
                ),
        )
        .child(
            text("Kiln")
                .font(TextStyle::new(15.0, 800.0))
                .color(Color::WHITE),
        )
        .child(text("·").font(theme::TINY).color(Color::WHITE.alpha(0.55)));
    bar.child(
        text(section)
            .font(theme::SMALL)
            .color(Color::WHITE.alpha(0.82)),
    )
    .child(el().grow(1.0))
    .child(el().size(7.0, 7.0).radius(8.0).fill(dot))
    .child(
        text(status)
            .font(theme::TINY)
            .color(Color::WHITE.alpha(0.75)),
    )
}

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

fn auth_view(s: &App, cx: &mut Cx<App>, p: Palette) -> El {
    let register = s.auth_register && s.signup_mode != "closed";
    let title = if register {
        "Create an account"
    } else {
        "Welcome back!"
    };
    let sub = if register {
        "Pick a username and you're in. Your account lives on this server."
    } else {
        "We're so excited to see you again!"
    };

    let field = |label: &'static str, edit: &rune::TextEdit, key: &'static str| {
        let input = text_input(edit, theme::BODY)
            .key(key)
            .w_full()
            .h(42.0)
            .radius(14.0)
            .fill(p.input)
            .color(p.text)
            .border(1.0, theme::gloss_edge(p))
            .padding(11.0)
            .single_line()
            .on_edit(move |s: &mut App| match key {
                "auth-user" => &mut s.auth_user,
                "auth-display" => &mut s.auth_display,
                "auth-code" => &mut s.auth_code,
                _ => &mut s.auth_pass,
            })
            .on_submit(|s: &mut App, _: &mut rune::Tx| {
                if s.auth_register && s.signup_mode != "closed" {
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
            .child(text(label).font(theme::SECTION).color(p.accent))
            .child(input)
    };

    let mut body = col().w_full().gap(16.0);

    if register {
        body = body.child(field("DISPLAY NAME", &s.auth_display, "auth-display"));
        body = body.child(field("USERNAME", &s.auth_user, "auth-user"));
        body = body.child(field("PASSWORD", &s.auth_pass, "auth-pass"));
        if s.signup_mode == "code" {
            body = body.child(field("SIGNUP CODE", &s.auth_code, "auth-code"));
        }
    } else {
        body = body.child(field("USERNAME", &s.auth_user, "auth-user"));
        body = body.child(field("PASSWORD", &s.auth_pass, "auth-pass"));
    }

    if let Some(err) = &s.auth_error {
        body = body.child(text(err.clone()).font(theme::SMALL).color(p.danger));
    }

    body = body.child(
        el().key("auth-submit")
            .w_full()
            .h(46.0)
            .radius(23.0)
            .fill(p.accent)
            .gradient(theme::sunset_magenta(p.dark), theme::sunset_violet(p.dark))
            .border(
                1.0,
                theme::sunset_amber(p.dark).alpha(if s.auth_busy { 0.35 } else { 0.8 }),
            )
            .hover(p.accent_hover)
            .motion(motion::SNAPPY)
            .center()
            .on_press(|s: &mut App| {
                if s.auth_register && s.signup_mode != "closed" {
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

    if s.signup_mode == "closed" && !register {
        body = body.child(
            text("Registration is closed on this server.")
                .font(theme::SMALL)
                .color(p.text_muted),
        );
    } else {
        body = body.child(
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
    }

    // Demo accounts exist only in development builds; a release install starts
    // empty, so advertising them there was misleading.
    if !cfg!(release_build) {
        body = body.child(
            col()
                .w_full()
                .radius(14.0)
                .fill(p.card)
                .border(1.0, p.divider_soft)
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
                        .wrap()
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        );
    }

    let connection_failed = s.server_config_error.is_some() || s.server_unreachable;
    if connection_failed {
        body = body.child(
            col()
                .key("auth-server-error")
                .w_full()
                .gap(5.0)
                .child(
                    row()
                        .gap(6.0)
                        .center_y()
                        .child(el().size(8.0, 8.0).radius(8.0).fill(p.danger))
                        .child(
                            text("Kiln can’t connect to the server")
                                .font(theme::SMALL_MED)
                                .color(p.danger),
                        ),
                )
                .child(
                    text(s.server_base.clone())
                        .font(theme::TINY)
                        .color(p.text_muted)
                        .truncate(),
                )
                .child(
                    text(s.server_config_error.clone().unwrap_or_else(|| {
                        "Check that the server is running and that this address is correct.".into()
                    }))
                    .wrap()
                    .font(theme::TINY)
                    .color(p.text_muted),
                )
                .child(
                    el().key("auth-server-settings")
                        .h(30.0)
                        .px(12.0)
                        .radius(15.0)
                        .center()
                        .role("button")
                        .label("Open Application Settings")
                        .fill(p.input)
                        .border(1.0, p.divider_soft)
                        .hover(p.input_hover)
                        .on_press(|s: &mut App| s.open_settings(SettingsTab::Application))
                        .child(
                            text("Open Application Settings")
                                .font(theme::TINY)
                                .color(p.text_link),
                        ),
                ),
        );
    }
    // Before sign-in there is no session, so `connected` is false by design; the
    // old "waiting for the server" line read as a fault when nothing was wrong.
    // A real failure is the card above, and it names the address.

    // The card is the hero: a glossy aubergine panel with the sunset laid along
    // its top edge, the way a neon sign sits on a hotel facade.
    let card = col()
        .key("auth-card")
        .w_full()
        .max_w(440.0)
        .radius(24.0)
        .clip_rounded(24.0)
        .fill(p.modal)
        .gradient(theme::sunset_violet(p.dark).alpha(0.28), p.modal)
        .border(1.0, theme::gloss_edge(p))
        .shadow(18.0, 52.0, p.shadow)
        .child(
            row()
                .key("auth-sunset")
                .w_full()
                .h(6.0)
                .child(el().grow(1.0).h_full().fill(theme::sunset_amber(p.dark)))
                .child(el().grow(1.0).h_full().fill(theme::sunset_magenta(p.dark)))
                .child(el().grow(1.0).h_full().fill(theme::sunset_violet(p.dark))),
        )
        .child(
            col()
                .w_full()
                .padding(28.0)
                .gap(16.0)
                .child(text(title).font(theme::HERO).color(p.text_bright))
                .child(text(sub).font(theme::SMALL).color(p.text_muted))
                .child(body),
        );

    // The card takes the width it is given and the page scrolls when a short
    // screen cannot hold it; the desktop keeps the card centred.
    let viewport_h = cx.layout_size("auth").map(|size| size[1]).unwrap_or(0.0);
    let column = col()
        .w_full()
        .min_h(viewport_h)
        .justify_center()
        .items_center()
        .child(card);
    col()
        .key("auth")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .fill(p.rail)
        .gradient(theme::sunset_violet(p.dark).alpha(0.45), p.chat)
        .center()
        .child(column)
}

// ---------------------------------------------------------------------------
// Shell
// ---------------------------------------------------------------------------

/// The conversation as a floating card: rounded on every corner so the channel
/// header reads as the card's own top edge and the composer never touches the
/// window's bottom.
fn conversation_card(s: &App, cx: &mut Cx<App>, p: Palette) -> El {
    col()
        .grow(1.0)
        .min_w(0.0)
        .min_h(0.0)
        .radius(22.0)
        .clip_rounded(22.0)
        .fill(p.chat)
        .border(1.0, theme::gloss_edge(p))
        .child(chat(s, cx, p))
}

fn guild_shell(s: &App, cx: &mut Cx<App>, p: Palette) -> El {
    let in_voice = s
        .current_channel()
        .is_some_and(|channel| channel.kind == ChannelKind::Voice);
    // Navigation rides the top edge, the channel tree is a strip beneath it,
    // the conversation floats in the middle with members on the far right, and
    // the voice/account controls close the window along the bottom.
    let mut body = row()
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .fill(p.chat)
        .pt(8.0)
        .pb(8.0)
        .pl(8.0)
        .pr(8.0)
        .gap(6.0)
        .child(conversation_card(s, cx, p));
    if s.show_members && !in_voice {
        body = body
            .child(resize_handle(
                "members-resize",
                -1.0,
                180.0,
                420.0,
                |s: &mut App| &mut s.members_width,
                p,
            ))
            .child(member_list(s, p, 0.0));
    }
    col()
        .key("shell")
        .w_full()
        .h_full()
        .fill(p.chat)
        .child(rail(s, p))
        .child(sidebar(s, p))
        .child(body)
        .child(
            row()
                .key("status-rail")
                .w_full()
                .h(56.0)
                .px(12.0)
                .gap(8.0)
                .center_y()
                .fill(p.rail)
                .border_sides([1.0, 0.0, 0.0, 0.0], theme::gloss_edge(p))
                .children(voice_connected_panel(s, p))
                .child(el().grow(1.0))
                .child(user_panel(s, p)),
        )
}

/// A thin drag edge between two columns, the way resizable sidebars work.
fn resize_handle(
    key: &'static str,
    direction: f32,
    min: f32,
    max: f32,
    width: fn(&mut App) -> &mut f32,
    p: Palette,
) -> El {
    el().key(key)
        .w(5.0)
        .h_full()
        .fill(Color::TRANSPARENT)
        .hover(p.accent.alpha(0.45))
        .cursor(rune::Cursor::ResizeColumn)
        .role("separator")
        .label(if key == "sidebar-resize" {
            "Resize the channel list"
        } else {
            "Resize the member list"
        })
        .help("Drag to resize; double-click to reset")
        .on_drag(
            move |s: &mut App, drag: &mut rune::Drag, _: &mut rune::Tx| match drag.phase {
                rune::Phase::Down => drag.data = *width(s),
                rune::Phase::Move => {
                    *width(s) = (drag.data + direction * drag.dx).clamp(min, max).round();
                }
                rune::Phase::Up => {}
            },
        )
        .on_double_press_tx(move |s: &mut App, _: &mut rune::Tx| {
            *width(s) = 240.0;
        })
}

fn dm_shell(s: &App, cx: &mut Cx<App>, p: Palette) -> El {
    // Mirrors the guild shell: the dock runs along the top, conversations keep
    // their own column and the friends page fills what is left.
    let mut body = row().w_full().grow(1.0).min_h(0.0).fill(p.chat);
    if s.dm_home || s.active_dm == 0 {
        body = body.child(dm_sidebar(s, p));
    }
    body = body.child(dm_main(s, cx, p));
    col()
        .key("shell")
        .w_full()
        .h_full()
        .fill(p.chat)
        .child(rail(s, p))
        .child(body)
}

// ---------------------------------------------------------------------------
// Surfaces
// ---------------------------------------------------------------------------

mod chat;
pub(crate) mod common;
mod composer;
mod context_menu;
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
pub(crate) use context_menu::*;
pub(crate) use dms::*;
pub(crate) use forms::*;
pub(crate) use members::*;
pub(crate) use modals::*;
pub(crate) use rail::*;
pub(crate) use server_settings::*;
pub(crate) use settings::*;
pub(crate) use sidebar::*;
