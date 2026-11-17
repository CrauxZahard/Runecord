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

thread_local! {
    /// Whether this view runs the phone layout: the window is narrower than
    /// [`MOBILE_BREAKPOINT`] points. Set by [`view`] from the root's layout.
    static MOBILE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// The root's size in points, so modal sheets can fit the screen.
    static MOBILE_SIZE: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((1360.0, 860.0)) };
}

/// The width at which the desktop columns stop fitting side by side.
pub(crate) const MOBILE_BREAKPOINT: f32 = 720.0;

/// A phone held sideways is wider than [`MOBILE_BREAKPOINT`] but too short for
/// the desktop columns; below this height the phone layout still applies.
pub(crate) const MOBILE_LANDSCAPE_HEIGHT: f32 = 520.0;

/// Whether the current view runs the phone layout.
pub(crate) fn mobile() -> bool {
    MOBILE.get()
}

/// The root's size in points, as laid out.
pub(crate) fn mobile_size() -> (f32, f32) {
    MOBILE_SIZE.get()
}

/// A modal sheet's width: its desktop width, shrunk to the phone with a margin.
pub(crate) fn sheet_w(desktop: f32) -> f32 {
    if mobile() {
        (mobile_size().0 - 24.0).clamp(280.0, desktop)
    } else {
        desktop
    }
}

/// A modal sheet's height: its desktop height, capped to the phone. A phone
/// keyboard takes roughly half the screen and WebView keeps the layout
/// viewport full height, so a sheet taller than about half would slide under
/// the keyboard; its body scrolls instead.
pub(crate) fn sheet_h(desktop: f32) -> f32 {
    if mobile() {
        let (_, height) = mobile_size();
        (height * 0.46).clamp(240.0, desktop).min(height - 24.0)
    } else {
        desktop
    }
}

pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette::dark()
    } else {
        Palette::light()
    }
}

pub fn view(s: &App, cx: &mut Cx<App>) -> El {
    let p = palette(s.dark);
    let root = cx
        .layout_with("app", |rect| (rect[2], rect[3]))
        .unwrap_or((1360.0, 860.0));
    let root = if root.0 <= 0.0 || root.1 <= 0.0 {
        (1360.0, 860.0)
    } else {
        root
    };
    MOBILE_SIZE.set(root);
    let narrow = root.0 < MOBILE_BREAKPOINT || root.1 < MOBILE_LANDSCAPE_HEIGHT;
    MOBILE.set(narrow);
    // Browser safe-area insets (status bar, cutout, gesture bar). Zero on
    // desktop; a browser reports them through `viewport-fit=cover`.
    let safe = cx.safe_area();
    // Keyboard focus for every pressable control, as a web page has it: Tab
    // reaches the form fields and buttons, which also lets the phone shell
    // drive the login without pointing at canvas coordinates.
    cx.web_focus();
    cx.caption_color(if s.dark { Color::WHITE } else { Color::BLACK });
    let mut root = col()
        .key("app")
        .w_full()
        .h_full()
        .fill(p.chat)
        .pl(safe[3])
        .pr(safe[1])
        .pt(safe[0])
        .pb(safe[2])
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
    if matches!(s.modal, Modal::ServerSettings | Modal::Roles)
        && s.can_open_server_settings()
    {
        root = root.child(server_settings_page(s, cx, p));
        if let Some(t) = &s.toast {
            root = root.child(toast(t, p));
        }
        return root;
    }
    let shell = match s.screen {
        Screen::Guild if narrow => guild_shell_mobile(s, cx, p),
        Screen::Guild => guild_shell(s, cx, p),
        Screen::Dms if narrow => dm_shell_mobile(s, cx, p),
        Screen::Dms => dm_shell(s, cx, p),
    };
    if s.developer {
        root = root
            .child(col().grow(1.0).min_h(0.0).child(shell))
            .child(developer_bar(s, p));
    } else {
        root = root.child(col().grow(1.0).min_h(0.0).child(shell));
    }
    if let Some(menu) = &s.context_menu {
        root = root.child(context_menu_layer(s, cx, menu, p));
    }
    if narrow && s.members_drawer && s.screen == Screen::Guild && s.mobile_chat {
        root = root.child(member_drawer(s, p));
    }
    if s.modal == Modal::ServerMenu {
        root = root.child(
            el().key("server-menu-layer")
                .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .items_start()
                .justify_start()
                .child(
                    el().pl(80.0)
                        .pt(TITLEBAR_HEIGHT + 20.0)
                        .child(server_menu(s, p)),
                ),
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
    let bar = row()
        .key("titlebar")
        .w_full()
        .h(if mobile() { 30.0 } else { TITLEBAR_HEIGHT })
        .pl(insets.leading + 12.0)
        .pr(insets.trailing + 12.0)
        .gap(9.0)
        .center_y()
        .fill(p.rail)
        .border_sides([0.0, 0.0, 1.0, 0.0], p.divider_soft)
        .window_drag()
        .child(
            el().size(21.0, 21.0)
                .radius(6.0)
                .fill(p.accent)
                .center()
                .child(
                    text("R")
                        .font(TextStyle::new(12.0, 800.0))
                        .color(Color::WHITE),
                ),
        )
        .child(
            text("Runecord")
                .font(TextStyle::new(14.0, 700.0))
                .color(p.text_bright),
        );
    if mobile() {
        // The chat header already names the channel; a phone bar only carries
        // the app and its connection.
        return bar
            .child(el().grow(1.0))
            .child(el().size(7.0, 7.0).radius(4.0).fill(dot))
            .child(text(status).font(theme::TINY).color(p.text_faint));
    }
    bar.child(el().w(1.0).h(14.0).fill(p.divider).mx(2.0))
        .child(text(section).font(theme::SMALL).color(p.text_muted))
        .child(el().grow(1.0))
        .child(el().size(7.0, 7.0).radius(4.0).fill(dot))
        .child(text(status).font(theme::TINY).color(p.text_faint))
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
            .h(40.0)
            .radius(3.0)
            .fill(p.rail)
            .color(p.text)
            .padding(10.0)
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
            .child(text(label).font(theme::SECTION).color(p.text_muted))
            .child(input)
    };

    let mut card = col()
        .key("auth-card")
        .w_full()
        .max_w(480.0)
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
        if s.signup_mode == "code" {
            card = card.child(field("SIGNUP CODE", &s.auth_code, "auth-code"));
        }
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
        card = card.child(
            text("Registration is closed on this server.")
                .font(theme::SMALL)
                .color(p.text_muted),
        );
    } else {
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
    }

    // Demo accounts exist only in development builds; a release install starts
    // empty, so advertising them there was misleading.
    if !cfg!(release_build) {
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
                        .wrap()
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        );
    }

    let connection_failed = s.server_config_error.is_some() || s.server_unreachable;
    if connection_failed {
        card = card.child(
            col()
                .key("auth-server-error")
                .w_full()
                .gap(5.0)
                .child(
                    row()
                        .gap(6.0)
                        .center_y()
                        .child(el().size(8.0, 8.0).radius(4.0).fill(p.danger))
                        .child(
                            text("Runecord can’t connect to the server")
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
                        .h(28.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Open Application Settings")
                        .fill(p.rail_hover)
                        .hover(p.card_hover)
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

    // On a phone the card takes the width it is given and the page scrolls
    // when a short screen cannot hold it; the desktop keeps the card centred.
    let viewport_h = cx.layout_size("auth").map(|size| size[1]).unwrap_or(0.0);
    let column = col()
        .w_full()
        .min_h(viewport_h)
        .justify_center()
        .px(if mobile() { 20.0 } else { 0.0 })
        .py(if mobile() { 20.0 } else { 0.0 })
        .items_center()
        .child(card);
    let mut page = col().key("auth").w_full().grow(1.0).min_h(0.0).fill(p.rail);
    if mobile() {
        page = page.scroll(ScrollSpec {
            stick_bottom: false,
        });
    } else {
        page = page.center();
    }
    page.child(column)
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
        .child(resize_handle(
            "sidebar-resize",
            1.0,
            180.0,
            460.0,
            |s: &mut App| &mut s.sidebar_width,
            p,
        ))
        .child(chat(s, cx, p));
    let in_voice = s
        .current_channel()
        .is_some_and(|channel| channel.kind == ChannelKind::Voice);
    if s.show_members && !in_voice {
        shell = shell
            .child(resize_handle(
                "members-resize",
                -1.0,
                180.0,
                420.0,
                |s: &mut App| &mut s.members_width,
                p,
            ))
            .child(member_list(
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

/// The phone layout. The rail and its list fill the screen until a channel is
/// chosen, then the chat takes over; the titlebar's back button returns.
fn guild_shell_mobile(s: &App, cx: &mut Cx<App>, p: Palette) -> El {
    if s.mobile_chat && s.active_channel != 0 {
        return chat(s, cx, p);
    }
    row()
        .key("shell")
        .w_full()
        .h_full()
        .fill(p.chat)
        .child(rail(s, p))
        .child(sidebar(s, p).w_full().grow(1.0).min_w(0.0))
}

fn dm_shell_mobile(s: &App, cx: &mut Cx<App>, p: Palette) -> El {
    if s.mobile_chat && (s.dm_home || s.active_dm != 0) {
        return dm_main(s, cx, p);
    }
    row()
        .key("shell")
        .w_full()
        .h_full()
        .fill(p.chat)
        .child(rail(s, p))
        .child(dm_sidebar(s, p).w_full().grow(1.0).min_w(0.0))
}

/// The member list as a drawer over the chat, dimmed behind it.
fn member_drawer(s: &App, p: Palette) -> El {
    el().key("member-drawer")
        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
        .child(
            el().key("member-drawer-scrim")
                .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .fill(Color::BLACK.alpha(0.45))
                .on_press(|s: &mut App| s.members_drawer = false),
        )
        .child(
            row()
                .key("member-drawer-panel")
                .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                .justify_end()
                .child(member_list(s, p, 0.0).w(300.0).max_w(320.0)),
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
