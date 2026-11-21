//! Settings surface.

use super::*;

mod account;
mod appearance;
mod applications;
mod keybinds;
mod notifications;
mod profile;
mod voice;
pub(crate) use account::*;
pub(crate) use appearance::*;
pub(crate) use applications::*;
pub(crate) use keybinds::*;
pub(crate) use notifications::*;
pub(crate) use profile::*;
pub(crate) use voice::*;

pub(crate) fn settings_sheet(s: &App, p: Palette, tab: SettingsTab) -> El {
    row()
        .key("settings-sheet")
        .w(800.0)
        .h(600.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(settings_nav(s, p, tab, 218.0))
        .child(settings_body(s, p, tab, true, 0.0))
}

/// Settings as a full-window page rather than a centred dialog.
pub(crate) fn settings_page(s: &App, cx: &mut Cx<App>, p: Palette, tab: SettingsTab) -> El {
    let trailing = cx.chrome_insets().trailing;
    col()
        .key("settings-page")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .fill(p.chat)
        .child(
            row()
                .key("settings-main")
                .w_full()
                .h_full()
                .child(settings_nav(s, p, tab, 232.0))
                .child(settings_body(s, p, tab, false, trailing)),
        )
}

fn settings_nav(s: &App, p: Palette, tab: SettingsTab, width: f32) -> El {
    let nav_item = |label: &'static str, t: SettingsTab, key: &'static str| {
        let selected = tab == t;
        el().key(key)
            .w_full()
            .h(32.0)
            .px(8.0)
            .radius(4.0)
            .center_y()
            .role("tab")
            .label(crate::i18n::t(label))
            .fill(if selected {
                p.selected
            } else {
                Color::TRANSPARENT
            })
            .hover(p.rail)
            .motion(motion::SNAPPY)
            .on_press(move |s: &mut App| s.select_settings_tab(t))
            .child(
                text(crate::i18n::t(label))
                    .font(theme::SMALL_MED)
                    .color(if selected {
                        p.text_bright
                    } else {
                        p.text_muted
                    }),
            )
    };

    col()
        .key("settings-nav")
        .w_full()
        .max_w(if width > 0.0 { width } else { f32::INFINITY })
        .h_full()
        .fill(p.modal_sidebar)
        .px(8.0)
        .pt(16.0)
        .gap(2.0)
        .child(
            row()
                .key("settings-me")
                .w_full()
                .h(48.0)
                .px(8.0)
                .gap(10.0)
                .center_y()
                .child(avatar_status(&s.me, 32.0, p, p.modal_sidebar, false))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text(s.me.display_name.clone())
                                .font(theme::SMALL_MED)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(
                            text(s.me.handle())
                                .font(theme::TINY)
                                .color(p.text_muted)
                                .truncate(),
                        ),
                ),
        )
        .child(el().w_full().h(1.0).fill(p.divider).my(8.0))
        .child(
            text("USER SETTINGS")
                .font(theme::SECTION)
                .color(p.text_muted)
                .padding(8.0),
        )
        .child(nav_item("My Account", SettingsTab::Account, "tab-account"))
        .child(nav_item("Profile", SettingsTab::Profile, "tab-profile"))
        .child(
            text("APP SETTINGS")
                .font(theme::SECTION)
                .color(p.text_muted)
                .padding(8.0)
                .pt(16.0),
        )
        .child(nav_item(
            "Application",
            SettingsTab::Application,
            "tab-application",
        ))
        .child(nav_item(
            "Appearance",
            SettingsTab::Appearance,
            "tab-appearance",
        ))
        .child(nav_item(
            "Notifications",
            SettingsTab::Notifications,
            "tab-notifications",
        ))
        .child(nav_item("Voice & Video", SettingsTab::Voice, "tab-voice"))
        .child(nav_item("Keybinds", SettingsTab::Keybinds, "tab-keybinds"))
        .child(el().grow(1.0))
        .child(
            row()
                .key("logout")
                .w_full()
                .h(32.0)
                .px(8.0)
                .radius(4.0)
                .center_y()
                .role("button")
                .label("Log Out")
                .hover(p.rail)
                .on_press(|s: &mut App| s.logout())
                .child(text("Log Out").font(theme::SMALL_MED).color(p.danger)),
        )
        .child(el().h(8.0))
}

type EditAccess = for<'a> fn(&'a mut App) -> &'a mut rune::TextEdit;

fn settings_input(
    edit: &rune::TextEdit,
    key: &'static str,
    label: &'static str,
    p: Palette,
    access: EditAccess,
) -> El {
    text_input(edit, theme::SMALL)
        .key(key)
        .w_full()
        .h(38.0)
        .radius(4.0)
        .fill(p.input)
        .color(p.text)
        .role("textbox")
        .label(label)
        .placeholder(label, p.text_faint)
        .padding(10.0)
        .on_edit(access)
}

fn settings_switch(
    label: &'static str,
    on: bool,
    toggle: fn(&mut App),
    p: Palette,
    kit: &rune_kit::Kit,
) -> El {
    row()
        .w_full()
        .min_h(40.0)
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
}

fn settings_password(
    edit: &rune::TextEdit,
    key: &'static str,
    label: &'static str,
    p: Palette,
    access: EditAccess,
) -> El {
    settings_input(edit, key, label, p, access).secure()
}

fn settings_body(s: &App, p: Palette, tab: SettingsTab, _sheet: bool, trailing: f32) -> El {
    let body = match tab {
        SettingsTab::Account => account_tab(s, p),
        SettingsTab::Profile => profile_tab(s, p),
        SettingsTab::Application => application_tab(s, p),
        SettingsTab::Appearance => appearance_tab(s, p),
        SettingsTab::Notifications => notifications_tab(s, p),
        SettingsTab::Voice => voice_tab(s, p),
        SettingsTab::Keybinds => keybinds_tab(s, p),
        // Bots & apps are made in the developer portal (soon); the old tab now
        // simply shows the Application settings instead.
        SettingsTab::Bots => application_tab(s, p),
    };

    let mut shell = col().key("settings-body").grow(1.0).min_w(0.0).h_full();
    shell = shell.child(
        row()
            .key("settings-top")
            .h(64.0)
            .px(40.0)
            .pr(12.0 + trailing)
            .items_center()
            .window_drag()
            .child(el().grow(1.0))
            .child(close_button(p)),
    );
    shell
        .child(
            col()
                .key("settings-scroll")
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(
                    col()
                        .key("settings-content")
                        .w(820.0)
                        .px(40.0)
                        .pb(48.0)
                        .gap(20.0)
                        .child(body),
                ),
        )
        // Sticky footer: unlike the old button at the end of the profile section,
        // this stays visible while scrolling through the long profile form.
        .children((tab == SettingsTab::Profile).then(|| profile_save_bar(s, p)))
}

fn color_swatches(selected: u32, set: fn(&mut App, u32), p: Palette) -> El {
    let mut row_el = row().w_full().gap(8.0).flex_wrap();
    row_el = row_el.child(
        el().size(26.0, 26.0)
            .radius(13.0)
            .border(1.0, p.divider)
            .center()
            .role("button")
            .label("No color")
            .help("No color")
            .on_press(move |s: &mut App| set(s, 0))
            .child(icon(&icons::CLOSE, 12.0).color(p.text_muted)),
    );
    for seed in 0..10u32 {
        let value = theme::avatar_color_value(seed);
        row_el = row_el.child(
            el().size(26.0, 26.0)
                .radius(13.0)
                .fill(theme::avatar_color(seed))
                .border(if value == selected { 2.0 } else { 0.0 }, p.text_bright)
                .role("button")
                .label(format!("Color {:06x}", value))
                .help(format!("#{:06x}", value))
                .on_press(move |s: &mut App| set(s, value)),
        );
    }
    row_el
}

pub(crate) fn server_menu(s: &App, p: Palette) -> El {
    let guild = s.active_guild().cloned().unwrap_or_default();
    let is_owner = guild.owner_id == s.me.id;
    let can_manage = s.can_manage_server();
    let can_channels = s.guild_can(perm::MANAGE_CHANNELS);
    let boost_pink = theme::color("#ff73fa");
    let boosts = guild.boosts;
    let level = guild.boost_level;
    let next_level = match level {
        0 => 2,
        1 => 7,
        2 => 14,
        _ => 14,
    };
    let prev_level = match level {
        0 => 0,
        1 => 2,
        2 => 7,
        _ => 14,
    };
    let ratio = if level >= 3 || next_level <= prev_level {
        1.0
    } else {
        ((boosts.saturating_sub(prev_level)) as f32 / (next_level - prev_level) as f32)
            .clamp(0.0, 1.0)
    };
    col()
        .key("server-menu")
        .w(220.0)
        .radius(8.0)
        .fill(theme::color("#111214"))
        .padding(8.0)
        .gap(2.0)
        .shadow(8.0, 24.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .children((!guild.banner_url.is_empty()).then(|| {
            el().w_full()
                .h(64.0)
                .radius(6.0)
                .clip_rounded(6.0)
                .child(guild_banner_el(s, &guild, 64.0, p))
        }))
        .child(
            text(guild.name.clone())
                .font(theme::SECTION)
                .color(p.text_faint)
                .padding(6.0),
        )
        .child(
            row()
                .key("menu-boost")
                .w_full()
                .h(30.0)
                .px(6.0)
                .radius(4.0)
                .gap(6.0)
                .center_y()
                .role("button")
                .label("Server reputation progress")
                .hover(p.rail_hover)
                .on_press(|s: &mut App| s.open_server_settings())
                .child(icon(&icons::SPARKLES, 14.0).color(boost_pink))
                .child(
                    text(if level > 0 {
                        format!("Reputation Lv {level}")
                    } else {
                        "No reputation".to_string()
                    })
                    .font(theme::TINY)
                    .color(p.text_bright),
                )
                .child(
                    text(format!("{boosts}/{next_level}"))
                        .font(theme::TINY)
                        .color(p.text_faint)
                        .grow(1.0),
                )
                .child(
                    el().w(54.0)
                        .h(6.0)
                        .radius(3.0)
                        .clip_rounded(3.0)
                        .fill(p.rail)
                        .child(
                            row()
                                .h(6.0)
                                .child(el().w(54.0 * ratio).h(6.0).fill(boost_pink)),
                        ),
                ),
        )
        .child(menu_item(
            "Invite People",
            false,
            "menu-invite",
            p,
            |s: &mut App| s.open_invite(),
        ))
        .children(can_manage.then(|| {
            menu_item(
                "Server Settings",
                false,
                "menu-settings",
                p,
                |s: &mut App| s.open_server_settings(),
            )
        }))
        .children(can_channels.then(|| {
            menu_item("Create Channel", false, "menu-channel", p, |s: &mut App| {
                s.open_create_channel(None)
            })
        }))
        .child(menu_item(
            "Notification Settings",
            false,
            "menu-notify",
            p,
            |s: &mut App| s.modal = Modal::ServerNotifications,
        ))
        .child(menu_item(
            "Privacy Settings",
            false,
            "menu-privacy",
            p,
            |s: &mut App| s.open_privacy(),
        ))
        .children((!is_owner).then(|| el().w_full().h(1.0).fill(p.divider).my(4.0)))
        .children((!is_owner).then(|| {
            menu_item("Leave Server", true, "menu-leave", p, |s: &mut App| {
                s.confirm_leave_guild()
            })
        }))
}
