use super::*;

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
