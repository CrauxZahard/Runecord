//! Settings surface.

use super::*;

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

/// Settings as a full-window page, matching Discord's own settings screen rather
/// than a centred dialog.
pub(crate) fn settings_page(s: &App, cx: &mut Cx<App>, p: Palette, tab: SettingsTab) -> El {
    let trailing = cx.chrome_insets().trailing;
    col()
        .key("settings-page")
        .w_full()
        .h_full()
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
            .label(label)
            .fill(if selected {
                p.selected
            } else {
                Color::TRANSPARENT
            })
            .hover(p.rail)
            .motion(motion::SNAPPY)
            .on_press(move |s: &mut App| s.select_settings_tab(t))
            .child(text(label).font(theme::SMALL_MED).color(if selected {
                p.text_bright
            } else {
                p.text_muted
            }))
    };

    col()
        .key("settings-nav")
        .w(width)
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
        .child(nav_item("Bots & Apps", SettingsTab::Bots, "tab-bots"))
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
        SettingsTab::Bots => bots_apps_tab(s, p),
    };

    col()
        .key("settings-body")
        .grow(1.0)
        .min_w(0.0)
        .h_full()
        .child(
            row()
                .key("settings-top")
                .h(64.0)
                .px(40.0)
                .pr(12.0 + trailing)
                .items_center()
                .window_drag()
                .child(el().grow(1.0))
                .child(close_button(p)),
        )
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
                        .w_full()
                        .max_w(820.0)
                        .px(40.0)
                        .pb(48.0)
                        .gap(20.0)
                        .child(body),
                ),
        )
}

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

pub(crate) fn profile_tab(s: &App, p: Palette) -> El {
    col()
        .key("profile")
        .w_full()
        .gap(16.0)
        .child(settings_title(
            "Profile",
            "How you show up across Runecord. Changes save to the local server.",
            p,
        ))
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(14.0)
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
                        ),
                )
                .child(el().w_full().h(1.0).fill(p.divider))
                .child(
                    row()
                        .w_full()
                        .gap(18.0)
                        .child(
                            col()
                                .gap(6.0)
                                .child(avatar_status(&s.me, 96.0, p, p.card, true))
                                .child(
                                    row()
                                        .gap(6.0)
                                        .child(
                                            el().key("profile-avatar-upload")
                                                .h(30.0)
                                                .px(10.0)
                                                .radius(4.0)
                                                .center()
                                                .role("button")
                                                .label("Change avatar")
                                                .fill(p.accent)
                                                .hover(p.accent_hover)
                                                .on_press_tx(|_s: &mut App, tx: &mut rune::Tx| {
                                                    let options = rune::FileDialogOptions {
                                                        title: "Upload profile avatar".to_string(),
                                                        directory: None,
                                                        file_name: None,
                                                        filters: Vec::new(),
                                                    }
                                                    .filter(
                                                        "Images",
                                                        &["png", "jpg", "jpeg", "gif", "webp"],
                                                    );
                                                    tx.open_file(
                                                        options,
                                                        |s: &mut App, result, _| {
                                                            if let Ok(Some(path)) = result {
                                                                s.begin_user_image_crop(
                                                                    path, false,
                                                                );
                                                            }
                                                        },
                                                    );
                                                })
                                                .child(
                                                    text("Change Avatar")
                                                        .font(theme::LABEL)
                                                        .color(Color::WHITE),
                                                ),
                                        )
                                        .children((!s.me.avatar_url.is_empty()).then(|| {
                                            el().key("profile-avatar-remove")
                                                .h(30.0)
                                                .px(10.0)
                                                .radius(4.0)
                                                .center()
                                                .role("button")
                                                .label("Remove avatar")
                                                .hover(p.danger.alpha(0.2))
                                                .on_press(|s: &mut App| s.remove_user_image(false))
                                                .child(
                                                    text("Remove")
                                                        .font(theme::LABEL)
                                                        .color(p.danger),
                                                )
                                        })),
                                ),
                        )
                        .child(
                            col()
                                .gap(6.0)
                                .child(el().w(320.0).h(120.0).radius(6.0).clip_rounded(6.0).child(
                                    if let Some(pixels) = s.images.get(&s.me.banner_url) {
                                        el().w_full()
                                            .h(120.0)
                                            .pixels_fit(pixels, rune::SurfaceFit::Fill)
                                    } else {
                                        el().w_full()
                                            .h(120.0)
                                            .fill(if s.me.banner_color != 0 {
                                                theme::profile_banner_color(s.me.banner_color)
                                            } else {
                                                p.accent
                                            })
                                            .center()
                                            .child(
                                                text("No banner uploaded")
                                                    .font(theme::TINY)
                                                    .color(p.text_faint),
                                            )
                                    },
                                ))
                                .child(
                                    row()
                                        .gap(6.0)
                                        .child(
                                            el().key("profile-banner-upload")
                                                .h(30.0)
                                                .px(10.0)
                                                .radius(4.0)
                                                .center()
                                                .role("button")
                                                .label("Change banner")
                                                .fill(p.accent)
                                                .hover(p.accent_hover)
                                                .on_press_tx(|_s: &mut App, tx: &mut rune::Tx| {
                                                    let options = rune::FileDialogOptions {
                                                        title: "Upload profile banner".to_string(),
                                                        directory: None,
                                                        file_name: None,
                                                        filters: Vec::new(),
                                                    }
                                                    .filter(
                                                        "Images",
                                                        &["png", "jpg", "jpeg", "gif", "webp"],
                                                    );
                                                    tx.open_file(
                                                        options,
                                                        |s: &mut App, result, _| {
                                                            if let Ok(Some(path)) = result {
                                                                s.begin_user_image_crop(path, true);
                                                            }
                                                        },
                                                    );
                                                })
                                                .child(
                                                    text("Change Banner")
                                                        .font(theme::LABEL)
                                                        .color(Color::WHITE),
                                                ),
                                        )
                                        .children((!s.me.banner_url.is_empty()).then(|| {
                                            el().key("profile-banner-remove")
                                                .h(30.0)
                                                .px(10.0)
                                                .radius(4.0)
                                                .center()
                                                .role("button")
                                                .label("Remove banner")
                                                .hover(p.danger.alpha(0.2))
                                                .on_press(|s: &mut App| s.remove_user_image(true))
                                                .child(
                                                    text("Remove")
                                                        .font(theme::LABEL)
                                                        .color(p.danger),
                                                )
                                        })),
                                ),
                        ),
                )
                .child(
                    text("DISPLAY NAME")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(settings_input(
                    &s.profile_display,
                    "profile-display",
                    "Display name",
                    p,
                    |s: &mut App| &mut s.profile_display,
                ))
                .child(text("PRONOUNS").font(theme::SECTION).color(p.text_muted))
                .child(settings_input(
                    &s.profile_pronouns,
                    "profile-pronouns",
                    "Pronouns",
                    p,
                    |s: &mut App| &mut s.profile_pronouns,
                ))
                .child(text("ABOUT ME").font(theme::SECTION).color(p.text_muted))
                .child(settings_input(
                    &s.profile_bio,
                    "profile-bio",
                    "About me",
                    p,
                    |s: &mut App| &mut s.profile_bio,
                ))
                .child(
                    text("CUSTOM STATUS")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(settings_input(
                    &s.profile_status,
                    "profile-status",
                    "Custom status",
                    p,
                    |s: &mut App| &mut s.profile_status,
                ))
                .child(
                    text("PROFILE STYLE")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(
                    text("Effects animate around your avatar; nameplates sit behind your display name.")
                        .font(theme::TINY)
                        .color(p.text_muted),
                )
                .child({
                    let mut chips = row().w_full().gap(6.0).flex_wrap();
                    for (value, label) in [
                        ("", "None"),
                        ("aurora", "Aurora"),
                        ("ember", "Ember"),
                        ("neon", "Neon"),
                        ("snow", "Snow"),
                    ] {
                        let selected = s.me.profile_effect == value;
                        chips = chips.child(
                            el().key(format!("profile-effect-{value}"))
                                .h(30.0)
                                .px(10.0)
                                .radius(15.0)
                                .center()
                                .role("radio")
                                .label(format!("{label} effect"))
                                .checked(selected)
                                .fill(if selected { p.selected } else { p.rail })
                                .border(if selected { 2.0 } else { 0.0 }, p.text_bright)
                                .hover(p.card_hover)
                                .on_press(move |s: &mut App| s.set_profile_effect(value))
                                .child(
                                    text(label)
                                        .font(theme::LABEL)
                                        .color(if selected {
                                            p.text_bright
                                        } else {
                                            p.text_muted
                                        }),
                                ),
                        );
                    }
                    chips
                })
                .child(
                    text("NAMEPLATE")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child({
                    let mut chips = row().w_full().gap(6.0).flex_wrap();
                    for (value, label, top, bottom) in [
                        ("", "None", p.rail, p.rail),
                        (
                            "midnight",
                            "Midnight",
                            theme::color("#232b52"),
                            theme::color("#5865f2"),
                        ),
                        (
                            "aurora",
                            "Aurora",
                            theme::color("#1db954"),
                            theme::color("#5865f2"),
                        ),
                        (
                            "sunset",
                            "Sunset",
                            theme::color("#eb459e"),
                            theme::color("#faa61a"),
                        ),
                        (
                            "candy",
                            "Candy",
                            theme::color("#ff73fa"),
                            theme::color("#ffb3fd"),
                        ),
                    ] {
                        let selected = s.me.nameplate == value;
                        chips = chips.child(
                            el().key(format!("profile-nameplate-{value}"))
                                .h(30.0)
                                .px(10.0)
                                .radius(15.0)
                                .center()
                                .role("radio")
                                .label(format!("{label} nameplate"))
                                .checked(selected)
                                .gradient(top, bottom)
                                .border(if selected { 2.0 } else { 0.0 }, p.text_bright)
                                .hover(p.card_hover)
                                .on_press(move |s: &mut App| s.set_profile_nameplate(value))
                                .child(
                                    text(label)
                                        .font(theme::LABEL)
                                        .color(if selected {
                                            Color::WHITE
                                        } else {
                                            p.text_bright
                                        }),
                                ),
                        );
                    }
                    chips
                })
                .child(
                    text("PROFILE WIDGET")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child({
                    let mut chips = row().w_full().gap(6.0).flex_wrap();
                    for (value, label) in [
                        ("none", "None"),
                        ("stats", "At a glance"),
                        ("activity", "Now playing"),
                    ] {
                        let selected = s.me.profile_widget == value;
                        chips = chips.child(
                            el().key(format!("profile-widget-{value}"))
                                .h(30.0)
                                .px(10.0)
                                .radius(15.0)
                                .center()
                                .role("radio")
                                .label(format!("{label} widget"))
                                .checked(selected)
                                .fill(if selected { p.selected } else { p.rail })
                                .border(if selected { 2.0 } else { 0.0 }, p.text_bright)
                                .hover(p.card_hover)
                                .on_press(move |s: &mut App| s.set_profile_widget(value))
                                .child(
                                    text(label)
                                        .font(theme::LABEL)
                                        .color(if selected {
                                            p.text_bright
                                        } else {
                                            p.text_muted
                                        }),
                                ),
                        );
                    }
                    chips
                })
                .child(text("ACTIVITY").font(theme::SECTION).color(p.text_muted))
                .child(
                    text("Use \"spotify\" for a listening card or \"playing\" for a game card.")
                        .font(theme::TINY)
                        .color(p.text_muted),
                )
                .child(settings_input(
                    &s.profile_activity_kind,
                    "activity-kind",
                    "Activity kind (spotify or playing)",
                    p,
                    |s: &mut App| &mut s.profile_activity_kind,
                ))
                .child(settings_input(
                    &s.profile_activity_title,
                    "activity-title",
                    "Activity title",
                    p,
                    |s: &mut App| &mut s.profile_activity_title,
                ))
                .child(settings_input(
                    &s.profile_activity_subtitle,
                    "activity-subtitle",
                    "Activity subtitle",
                    p,
                    |s: &mut App| &mut s.profile_activity_subtitle,
                ))
                .child(settings_input(
                    &s.profile_activity_state,
                    "activity-state",
                    "Activity state (album or detail)",
                    p,
                    |s: &mut App| &mut s.profile_activity_state,
                ))
                .child(settings_input(
                    &s.profile_activity_url,
                    "activity-url",
                    "Activity link",
                    p,
                    |s: &mut App| &mut s.profile_activity_url,
                ))
                .child({
                    let mut actions = row().w_full().gap(8.0).flex_wrap();
                    actions = actions
                        .child(
                            el().key("activity-save")
                                .h(32.0)
                                .px(12.0)
                                .radius(4.0)
                                .center()
                                .role("button")
                                .label("Save activity")
                                .fill(p.accent)
                                .hover(p.accent_hover)
                                .on_press(|s: &mut App| s.save_profile_activity())
                                .child(
                                    text("Save Activity").font(theme::LABEL).color(Color::WHITE),
                                ),
                        )
                        .child(
                            el().key("activity-clear")
                                .h(32.0)
                                .px(12.0)
                                .radius(4.0)
                                .center()
                                .role("button")
                                .label("Clear activity")
                                .hover(p.danger.alpha(0.2))
                                .on_press(|s: &mut App| s.clear_profile_activity())
                                .child(text("Clear").font(theme::LABEL).color(p.danger)),
                        )
                        .child(
                            el().key("activity-artwork-upload")
                                .h(32.0)
                                .px(12.0)
                                .radius(4.0)
                                .center()
                                .role("button")
                                .label("Upload activity artwork")
                                .fill(p.rail)
                                .hover(p.rail_hover)
                                .on_press_tx(|_s: &mut App, tx: &mut rune::Tx| {
                                    let options = rune::FileDialogOptions {
                                        title: "Upload activity artwork".to_string(),
                                        directory: None,
                                        file_name: None,
                                        filters: Vec::new(),
                                    }
                                    .filter("Images", &["png", "jpg", "jpeg", "gif", "webp"]);
                                    tx.open_file(options, |s: &mut App, result, _| {
                                        if let Ok(Some(path)) = result {
                                            s.upload_activity_artwork(path);
                                        }
                                    });
                                })
                                .child(
                                    text("Upload Artwork")
                                        .font(theme::LABEL)
                                        .color(p.text_bright),
                                ),
                        );
                    if s.me
                        .activity
                        .as_ref()
                        .is_some_and(|activity| !activity.artwork_url.is_empty())
                    {
                        actions = actions.child(
                            el().key("activity-artwork-remove")
                                .h(32.0)
                                .px(10.0)
                                .radius(4.0)
                                .center()
                                .role("button")
                                .label("Remove activity artwork")
                                .hover(p.danger.alpha(0.2))
                                .on_press(|s: &mut App| s.remove_activity_artwork())
                                .child(text("Remove Art").font(theme::LABEL).color(p.danger)),
                        );
                    }
                    actions
                })
                .child(text("CONNECTIONS").font(theme::SECTION).color(p.text_muted))
                .child({
                    let mut chips = row().w_full().gap(6.0).flex_wrap();
                    for connection in &s.me.connections {
                        let kind = connection.kind.clone();
                        let name = connection.name.clone();
                        chips = chips.child(
                            row()
                                .key(format!("connection-{kind}-{name}"))
                                .h(28.0)
                                .px(8.0)
                                .radius(14.0)
                                .gap(6.0)
                                .center_y()
                                .fill(p.rail)
                                .child(
                                    text(kind.to_uppercase())
                                        .font(theme::TINY)
                                        .color(p.text_muted),
                                )
                                .child(text(name.clone()).font(theme::TINY).color(p.text_bright))
                                .child(
                                    el().key(format!("connection-remove-{kind}-{name}"))
                                        .size(18.0, 18.0)
                                        .radius(9.0)
                                        .center()
                                        .role("button")
                                        .label(format!("Remove {kind} connection"))
                                        .hover(p.danger.alpha(0.2))
                                        .on_press(move |s: &mut App| {
                                            s.remove_profile_connection(kind.clone(), name.clone())
                                        })
                                        .child(icon(&icons::CLOSE, 10.0).color(p.danger)),
                                ),
                        );
                    }
                    if s.me.connections.is_empty() {
                        chips = chips.child(
                            text("No connections yet.")
                                .font(theme::TINY)
                                .color(p.text_muted),
                        );
                    }
                    chips
                })
                .child(settings_input(
                    &s.profile_connection_kind,
                    "connection-kind",
                    "Connection kind (github, steam, spotify, website)",
                    p,
                    |s: &mut App| &mut s.profile_connection_kind,
                ))
                .child(settings_input(
                    &s.profile_connection_name,
                    "connection-name",
                    "Connection name",
                    p,
                    |s: &mut App| &mut s.profile_connection_name,
                ))
                .child(
                    el().key("connection-add")
                        .h(32.0)
                        .px(12.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Add connection")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(|s: &mut App| s.add_profile_connection())
                        .child(
                            text("Add Connection")
                                .font(theme::LABEL)
                                .color(Color::WHITE),
                        ),
                )
                .child(
                    text("PROFILE BADGES")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child({
                    let mut chips = row().w_full().gap(6.0).flex_wrap();
                    for (key, label) in crate::ui::common::BADGE_KEYS {
                        let selected = s.profile_badges.contains(key);
                        chips = chips.child(
                            el().key(format!("profile-badge-{key}"))
                                .h(28.0)
                                .px(8.0)
                                .radius(4.0)
                                .gap(5.0)
                                .center_y()
                                .role("checkbox")
                                .label(label)
                                .checked(selected)
                                .fill(if selected { p.selected } else { p.rail })
                                .border(if selected { 2.0 } else { 0.0 }, p.text_bright)
                                .hover(p.card_hover)
                                .on_press(move |s: &mut App| s.toggle_profile_badge(key))
                                .child(crate::ui::common::badge_chip(key, 16.0, p))
                                .child(text(label).font(theme::TINY).color(if selected {
                                    p.text_bright
                                } else {
                                    p.text_muted
                                })),
                        );
                    }
                    chips
                })
                .child(
                    text("AVATAR DECORATION")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child({
                    let mut swatches = row().w_full().gap(8.0).flex_wrap();
                    for (key, label) in crate::ui::common::DECORATIONS {
                        let selected = s.profile_decoration == key;
                        let (top, bottom) = crate::ui::common::decoration_colors(key);
                        swatches = swatches.child(
                            el().size(30.0, 30.0)
                                .radius(15.0)
                                .key(format!("profile-decoration-{key}"))
                                .role("radio")
                                .label(label)
                                .checked(selected)
                                .fill(if key.is_empty() { p.rail } else { top })
                                .gradient(
                                    if key.is_empty() { p.rail } else { top },
                                    if key.is_empty() { p.rail } else { bottom },
                                )
                                .border(if selected { 2.0 } else { 0.0 }, p.text_bright)
                                .help(label)
                                .on_press(move |s: &mut App| s.set_avatar_decoration(key)),
                        );
                    }
                    swatches
                })
                .child(
                    text("AVATAR COLOR")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(color_swatches(s.me.avatar_color, App::set_avatar_color, p))
                .child(
                    text("AVATAR BORDER")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(color_swatches(
                    s.me.avatar_border,
                    App::set_avatar_border,
                    p,
                ))
                .child(
                    text("PROFILE BANNER")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(color_swatches(s.me.banner_color, App::set_banner_color, p))
                .child(
                    row().w_full().justify_end().child(
                        el().h(34.0)
                            .px(18.0)
                            .radius(4.0)
                            .key("profile-save")
                            .role("button")
                            .label("Save Profile")
                            .fill(p.accent)
                            .hover(p.accent_hover)
                            .center()
                            .on_press(|s: &mut App| s.save_profile())
                            .child(
                                text("Save Changes")
                                    .font(theme::SMALL_MED)
                                    .color(Color::WHITE),
                            ),
                    ),
                ),
        )
}

fn volume_track(value: f32, set: fn(&mut App, f32), p: Palette) -> El {
    let fill = value.clamp(0.0, 1.0);
    el().w_full()
        .h(20.0)
        .role("slider")
        .label(format!("{:.0}%", fill * 100.0))
        .on_drag(move |s: &mut App, drag, _| {
            if drag.phase != rune::Phase::Up && drag.w > 0.0 {
                set(s, (drag.x / drag.w).clamp(0.0, 1.0));
            }
        })
        .child(
            row()
                .w_full()
                .h(8.0)
                .radius(4.0)
                .clip_rounded(4.0)
                .fill(p.rail)
                .child(el().h(8.0).grow(fill.max(0.001)).fill(p.accent))
                .child(el().h(8.0).grow((1.0 - fill).max(0.001))),
        )
}

fn voice_device_row(label: &'static str, value: &str, kind: &'static str, p: Palette) -> El {
    row()
        .w_full()
        .gap(10.0)
        .center_y()
        .child(text(label).font(theme::TINY).color(p.text_muted).w(140.0))
        .child(
            el().key(format!("voice-{kind}-device"))
                .h(38.0)
                .grow(1.0)
                .px(10.0)
                .radius(4.0)
                .fill(p.input)
                .center_y()
                .role("button")
                .label(format!("{label}: {value}"))
                .hover(p.card_hover)
                .on_press(move |s: &mut App| {
                    s.refresh_voice_devices();
                    s.modal = Modal::VoiceDevice {
                        kind: kind.to_string(),
                    };
                })
                .child(
                    text(if value.is_empty() { "Default" } else { value })
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .grow(1.0)
                        .truncate(),
                )
                .child(icon(&icons::CHEVRON_DOWN, 14.0).color(p.text_muted)),
        )
}

pub(crate) fn voice_tab(s: &App, p: Palette) -> El {
    let level = crate::voice::mic_level();
    let mic_running = crate::voice::mic_testing();
    let camera_running = crate::voice::camera_testing();
    let camera_frame = crate::voice::camera_test_frame();
    let auto = s.voice_auto_sensitivity;

    let section = |label: &'static str, p: Palette| {
        text(label).font(theme::SECTION).color(p.text_muted).pt(6.0)
    };

    col()
        .key("voice-settings")
        .w_full()
        .gap(14.0)
        .child(settings_title(
            "Voice & Video",
            "Choose devices, test your microphone and preview your camera.",
            p,
        ))
        .child(voice_device_row(
            "Input Device",
            &s.voice_input_device,
            "input",
            p,
        ))
        .child(voice_device_row(
            "Output Device",
            &s.voice_output_device,
            "output",
            p,
        ))
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            row()
                .w_full()
                .gap(16.0)
                .child(
                    col()
                        .grow(1.0)
                        .gap(6.0)
                        .child(text("INPUT VOLUME").font(theme::TINY).color(p.text_muted))
                        .child(volume_track(
                            s.voice_input_volume,
                            |s: &mut App, value| s.set_voice_volume(true, value),
                            p,
                        )),
                )
                .child(
                    col()
                        .grow(1.0)
                        .gap(6.0)
                        .child(text("OUTPUT VOLUME").font(theme::TINY).color(p.text_muted))
                        .child(volume_track(
                            s.voice_output_volume,
                            |s: &mut App, value| s.set_voice_volume(false, value),
                            p,
                        )),
                ),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(section("MIC TEST", p))
        .child(
            row()
                .w_full()
                .gap(12.0)
                .center_y()
                .child(
                    el().key("mic-test-toggle")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(if mic_running {
                            "Stop mic test"
                        } else {
                            "Let's Check"
                        })
                        .fill(if mic_running { p.rail } else { p.accent })
                        .hover(if mic_running {
                            p.rail_hover
                        } else {
                            p.accent_hover
                        })
                        .on_press(|s: &mut App| s.toggle_mic_test())
                        .child(
                            text(if mic_running {
                                "Stop Test"
                            } else {
                                "Let's Check"
                            })
                            .font(theme::SMALL_MED)
                            .color(Color::WHITE),
                        ),
                )
                .child(
                    el().grow(1.0)
                        .h(10.0)
                        .radius(5.0)
                        .clip_rounded(5.0)
                        .fill(p.rail)
                        .child(
                            row()
                                .h(10.0)
                                .w_full()
                                .child(el().h(10.0).grow(level.max(0.001)).fill(if level > 0.7 {
                                    p.yellow
                                } else {
                                    p.green
                                }))
                                .child(el().h(10.0).grow((1.0 - level).max(0.001))),
                        ),
                )
                .child(
                    text(if level > 0.02 {
                        "Hearing you"
                    } else {
                        "Say something"
                    })
                    .font(theme::TINY)
                    .color(p.text_faint),
                ),
        )
        .child(section("INPUT MODE", p))
        .child({
            let mode = |key: &'static str, label: &'static str, value: &'static str, p: Palette| {
                let selected = s.voice_mode == value;
                el().key(key)
                    .w_full()
                    .h(38.0)
                    .px(12.0)
                    .radius(4.0)
                    .gap(8.0)
                    .center_y()
                    .role("radio")
                    .label(label)
                    .checked(selected)
                    .fill(if selected {
                        p.selected
                    } else {
                        Color::TRANSPARENT
                    })
                    .hover(p.card_hover)
                    .on_press(move |s: &mut App| s.set_voice_mode(value))
                    .child(
                        el().size(14.0, 14.0)
                            .radius(7.0)
                            .border(2.0, if selected { p.accent } else { p.text_muted })
                            .child(if selected {
                                el().size(6.0, 6.0).radius(3.0).fill(p.accent)
                            } else {
                                el()
                            }),
                    )
                    .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
            };
            col()
                .w_full()
                .gap(4.0)
                .child(mode("voice-mode-voice", "Voice Activity", "voice", p))
                .child(mode("voice-mode-ptt", "Push to Talk", "ptt", p))
                .children((s.voice_mode == "ptt").then(|| {
                    row()
                        .w_full()
                        .h(30.0)
                        .px(12.0)
                        .gap(8.0)
                        .center_y()
                        .child(el().size(8.0, 8.0).radius(4.0).fill(
                            if crate::voice::push_to_talk_held() {
                                p.green
                            } else {
                                p.text_faint
                            },
                        ))
                        .child(
                            text(if crate::voice::push_to_talk_held() {
                                "Transmitting — F8 held"
                            } else {
                                "Hold F8 to talk"
                            })
                            .font(theme::TINY)
                            .color(p.text_muted),
                        )
                }))
        })
        .child(section("INPUT SENSITIVITY", p))
        .child(
            row()
                .key("voice-auto-sensitivity")
                .w_full()
                .min_h(40.0)
                .px(10.0)
                .radius(4.0)
                .center_y()
                .role("switch")
                .label("Automatically determine input sensitivity")
                .checked(auto)
                .hover(p.card_hover)
                .on_press(|s: &mut App| {
                    let next = !s.voice_auto_sensitivity;
                    s.set_voice_auto_sensitivity(next);
                })
                .child(
                    text("Automatically determine input sensitivity")
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(
                    el().size(40.0, 22.0)
                        .radius(11.0)
                        .fill(if auto { p.green } else { p.input })
                        .child(
                            el().size(18.0, 18.0)
                                .radius(9.0)
                                .fill(Color::WHITE)
                                .ml(if auto { 19.0 } else { 2.0 })
                                .mt(2.0),
                        ),
                ),
        )
        .child(if auto {
            el().w_full().h(20.0).child(
                row()
                    .w_full()
                    .h(8.0)
                    .radius(4.0)
                    .clip_rounded(4.0)
                    .fill(p.rail)
                    .child(el().h(8.0).grow(0.35).fill(p.yellow))
                    .child(el().h(8.0).grow(0.65).fill(p.green)),
            )
        } else {
            volume_track(
                (s.voice_sensitivity / 0.5).clamp(0.0, 1.0),
                |s: &mut App, value| s.set_voice_sensitivity(value * 0.5),
                p,
            )
        })
        .child(
            text(if auto {
                "Discord-style automatic sensitivity follows the noise floor."
            } else {
                "Drag the bar to set the cutoff for transmitting your microphone."
            })
            .font(theme::TINY)
            .color(p.text_faint),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(section("CAMERA", p))
        .child(voice_device_row(
            "Camera",
            &s.voice_camera_device,
            "camera",
            p,
        ))
        .child(
            row()
                .w_full()
                .gap(14.0)
                .child(
                    el().w(360.0)
                        .h(203.0)
                        .radius(8.0)
                        .clip_rounded(8.0)
                        .fill(p.rail)
                        .center()
                        .child(if let Some(pixels) = camera_frame {
                            el().w_full()
                                .h(203.0)
                                .pixels_fit(&pixels, rune::SurfaceFit::Fill)
                        } else {
                            col()
                                .center()
                                .gap(6.0)
                                .child(icon(&icons::VIDEO, 36.0).color(p.text_faint))
                                .child(
                                    text("Camera preview")
                                        .font(theme::SMALL)
                                        .color(p.text_muted),
                                )
                        }),
                )
                .child(
                    col()
                        .gap(8.0)
                        .child(
                            el().key("camera-test-toggle")
                                .h(34.0)
                                .px(14.0)
                                .radius(4.0)
                                .center()
                                .role("button")
                                .label(if camera_running {
                                    "Stop video test"
                                } else {
                                    "Test Video"
                                })
                                .fill(if camera_running { p.rail } else { p.accent })
                                .hover(if camera_running {
                                    p.rail_hover
                                } else {
                                    p.accent_hover
                                })
                                .on_press(|s: &mut App| s.toggle_camera_test())
                                .child(
                                    text(if camera_running {
                                        "Stop Video"
                                    } else {
                                        "Test Video"
                                    })
                                    .font(theme::SMALL_MED)
                                    .color(Color::WHITE),
                                ),
                        )
                        .child(
                            text("Live DirectShow preview at 640 px wide.")
                                .font(theme::TINY)
                                .color(p.text_faint),
                        ),
                ),
        )
}

pub(crate) fn voice_device_sheet(s: &App, p: Palette, kind: &str) -> El {
    let (devices, title) = match kind {
        "input" => (&s.audio_devices, "Input Device"),
        "output" => (&s.output_devices, "Output Device"),
        _ => (&s.video_devices, "Camera"),
    };
    let current = match kind {
        "input" => &s.voice_input_device,
        "output" => &s.voice_output_device,
        _ => &s.voice_camera_device,
    };
    let kind_owned = kind.to_string();
    let mut list = col().w_full().gap(2.0);
    let mut options = vec![String::new()];
    options.extend(devices.iter().cloned());
    for device in options {
        let selected = current == &device;
        let label = if device.is_empty() {
            "Default".to_string()
        } else {
            device.clone()
        };
        let kind = kind_owned.clone();
        list = list.child(
            el().key(format!(
                "voice-device-{}",
                if device.is_empty() {
                    "default"
                } else {
                    &device
                }
            ))
            .w_full()
            .min_h(38.0)
            .px(10.0)
            .radius(4.0)
            .center_y()
            .role("radio")
            .label(label.clone())
            .checked(selected)
            .fill(if selected {
                p.selected
            } else {
                Color::TRANSPARENT
            })
            .hover(p.card_hover)
            .on_press(move |s: &mut App| s.select_voice_device(&kind, device.clone()))
            .child(
                text(label)
                    .font(theme::SMALL)
                    .color(if selected {
                        p.text_bright
                    } else {
                        p.text_muted
                    })
                    .truncate(),
            ),
        );
    }
    col()
        .key("voice-devices")
        .w(460.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(18.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| {
            s.modal = Modal::Settings(SettingsTab::Voice)
        })
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text(title)
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            col()
                .w_full()
                .max_h(360.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list),
        )
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
                            "Save Changes",
                            if dirty { p.accent } else { p.rail },
                            if dirty { Color::WHITE } else { p.text_faint },
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

pub(crate) fn appearance_tab(s: &App, p: Palette) -> El {
    let theme_card = |label: &'static str, dark: bool| {
        let selected = s.dark == dark;
        let preview = if dark {
            Palette::dark()
        } else {
            Palette::light()
        };
        col()
            .key(format!("theme-{label}"))
            .w(240.0)
            .radius(8.0)
            .padding(8.0)
            .gap(8.0)
            .fill(p.card)
            .border(
                if selected { 2.0 } else { 1.0 },
                if selected { p.accent } else { p.divider },
            )
            .on_press(move |s: &mut App| s.set_dark(dark))
            .child(
                row()
                    .w_full()
                    .h(96.0)
                    .radius(4.0)
                    .clip_rounded(4.0)
                    .fill(preview.chat)
                    .child(el().w(20.0).h_full().fill(preview.rail))
                    .child(el().w(52.0).h_full().fill(preview.sidebar))
                    .child(
                        col()
                            .grow(1.0)
                            .padding(8.0)
                            .gap(6.0)
                            .child(el().w_full().h(8.0).radius(4.0).fill(preview.input))
                            .child(el().w_frac(0.7).h(8.0).radius(4.0).fill(preview.input))
                            .child(el().w_frac(0.5).h(8.0).radius(4.0).fill(preview.input)),
                    ),
            )
            .child(
                row()
                    .center_y()
                    .gap(6.0)
                    .child(
                        el().size(18.0, 18.0)
                            .radius(9.0)
                            .border(2.0, if selected { p.accent } else { p.text_faint })
                            .center()
                            .child(el().size(8.0, 8.0).radius(4.0).fill(if selected {
                                p.accent
                            } else {
                                Color::TRANSPARENT
                            })),
                    )
                    .child(text(label).font(theme::SMALL_MED).color(p.text_bright)),
            )
    };

    col()
        .key("appearance")
        .w_full()
        .gap(16.0)
        .child(settings_title("Appearance", "Theme applies instantly and is remembered for this profile.", p))
        .child(text("THEME").font(theme::SECTION).color(p.text_muted))
        .child(row().gap(16.0).child(theme_card("Dark", true)).child(theme_card("Light", false)))
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(12.0)
                .child(text("MESSAGE DISPLAY").font(theme::SECTION).color(p.text_muted))
                .child(settings_switch(
                    "Compact message rows",
                    s.compact,
                    |s: &mut App| s.toggle_compact(!s.compact),
                    p,
                    &rune_kit::Kit::neutral(p.dark),
                ))
                .child(
                    text(if s.compact {
                        "Compact: tighter rows, smaller avatars."
                    } else {
                        "Cozy: grouped messages with full-size avatars."
                    })
                    .font(theme::SMALL)
                    .color(p.text_muted),
                ),
        )
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(12.0)
                .child(text("DEVELOPER").font(theme::SECTION).color(p.text_muted))
                .child(settings_switch(
                    "Developer mode",
                    s.developer,
                    |s: &mut App| s.toggle_developer(!s.developer),
                    p,
                    &rune_kit::Kit::neutral(p.dark),
                ))
                .child(
                    text(
                        "Adds a live status bar with gateway state, cache counts, image memory and snapshot age.",
                    )
                    .font(theme::SMALL)
                    .color(p.text_muted),
                ),
        )
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(12.0)
                .child(text("LOCAL DATA").font(theme::SECTION).color(p.text_muted))
                .child(
                    text("The warm-start snapshot only. Server messages, accounts and uploads stay untouched.")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                )
                .child(
                    el().h(32.0)
                        .px(16.0)
                        .radius(4.0)
                        .role("button")
                        .label("Clear warm-start cache")
                        .fill(p.danger)
                        .hover(p.danger_hover)
                        .center()
                        .on_press(|s: &mut App| {
                            crate::cache::clear(&s.cache_profile);
                            s.toast("Warm-start cache cleared");
                        })
                        .child(
                            text("Clear Warm-Start Cache")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        ),
                ),
        )
}

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

pub(crate) fn keybinds_tab(s: &App, p: Palette) -> El {
    let mut list = col().w_full().gap(4.0);
    for (action, label) in crate::app::KEYBIND_ACTIONS {
        let action = *action;
        let chord = s.keybinds.get(action).cloned().unwrap_or_default();
        let recording = s.recording_key.as_deref() == Some(action);
        list = list.child(
            row()
                .key(format!("keybind-row-{action}"))
                .w_full()
                .h(44.0)
                .px(10.0)
                .radius(5.0)
                .hover(p.card_hover)
                .center_y()
                .gap(12.0)
                .child(
                    text(label.to_string())
                        .font(theme::SMALL)
                        .color(p.text)
                        .grow(1.0),
                )
                .child(
                    text(if recording {
                        "Press keys…".to_string()
                    } else {
                        crate::app::display_chord(&chord)
                    })
                    .font(theme::LABEL)
                    .color(if recording { p.yellow } else { p.text_muted })
                    .border(1.0, if recording { p.yellow } else { p.divider })
                    .radius(4.0)
                    .px(8.0)
                    .py(4.0),
                )
                .child(
                    el().h(28.0)
                        .px(12.0)
                        .radius(4.0)
                        .key(format!("keybind-record-{action}"))
                        .role("button")
                        .label(format!("Record {label}"))
                        .center()
                        .fill(if recording { p.accent } else { p.rail })
                        .hover(p.rail_hover)
                        .on_press(move |s: &mut App| s.begin_keybind(action))
                        .child(
                            text(if recording { "Recording…" } else { "Record" })
                                .font(theme::LABEL)
                                .color(if recording {
                                    Color::WHITE
                                } else {
                                    p.text_bright
                                }),
                        ),
                ),
        );
    }
    col()
        .key("keybinds")
        .w_full()
        .gap(16.0)
        .child(settings_title(
            "Keybinds",
            "Click Record, then press a chord. Bindings save with the profile.",
            p,
        ))
        .child(list)
        .child(
            text("The composer keeps Ctrl+Enter to send and Enter for a newline.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(18.0)
                .gap(12.0)
                .child(text("PREFERENCES").font(theme::SECTION).color(p.text_muted))
                .child(
                    text("Restore the default theme, notification toggles, row density, developer mode and keybinds.")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                )
                .child(
                    row()
                        .w_full()
                        .gap(8.0)
                        .child(
                            el().h(32.0)
                                .px(16.0)
                                .radius(4.0)
                                .role("button")
                                .label("Reset keybinds")
                                .fill(p.rail)
                                .hover(p.rail_hover)
                                .center()
                                .on_press(|s: &mut App| s.reset_keybinds())
                                .child(
                                    text("Reset Keybinds")
                                        .font(theme::SMALL_MED)
                                        .color(p.text_bright),
                                ),
                        )
                        .child(
                            el().h(32.0)
                                .px(16.0)
                                .radius(4.0)
                                .role("button")
                                .label("Reset preferences")
                                .fill(p.rail)
                                .hover(p.rail_hover)
                                .center()
                                .on_press(|s: &mut App| s.reset_preferences())
                                .child(
                                    text("Reset Preferences")
                                        .font(theme::SMALL_MED)
                                        .color(p.text_bright),
                                ),
                        ),
                ),
        )
}

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

pub(crate) fn server_menu(s: &App, p: Palette) -> El {
    let guild = s.active_guild().cloned().unwrap_or_default();
    let is_owner = guild.owner_id == s.me.id;
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
                .label("Server boost progress")
                .hover(p.rail_hover)
                .on_press(|s: &mut App| s.open_server_settings())
                .child(icon(&icons::SPARKLES, 14.0).color(boost_pink))
                .child(
                    text(if level > 0 {
                        format!("Level {level}")
                    } else {
                        "No boosts".to_string()
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
        .child(menu_item(
            "Server Settings",
            false,
            "menu-settings",
            p,
            |s: &mut App| s.open_server_settings(),
        ))
        .child(menu_item(
            "Create Channel",
            false,
            "menu-channel",
            p,
            |s: &mut App| {
                s.create_parent = None;
                s.profile_edit.clear();
                s.modal = Modal::CreateChannel;
            },
        ))
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
        .child(menu_item(
            "App Directory",
            false,
            "menu-apps",
            p,
            |s: &mut App| s.open_settings(SettingsTab::Bots),
        ))
        .children((!is_owner).then(|| el().w_full().h(1.0).fill(p.divider).my(4.0)))
        .children((!is_owner).then(|| {
            menu_item("Leave Server", true, "menu-leave", p, |s: &mut App| {
                s.confirm_leave_guild()
            })
        }))
}
