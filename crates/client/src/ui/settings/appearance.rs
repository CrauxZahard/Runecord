use super::*;

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
        .child(text("LANGUAGE").font(theme::SECTION).color(p.text_muted))
        .child({
            let mut chips = row().w_full().gap(8.0).flex_wrap();
            for locale in crate::i18n::locales() {
                let selected = crate::i18n::locale() == locale;
                chips = chips.child(
                    el().key(format!("locale-{}", locale.tag()))
                        .h(32.0)
                        .px(12.0)
                        .radius(4.0)
                        .center()
                        .role("radio")
                        .label(locale.label())
                        .checked(selected)
                        .fill(if selected { p.accent } else { p.card })
                        .hover(p.card_hover)
                        .on_press(move |s: &mut App| s.set_locale(locale))
                        .child(
                            text(locale.label())
                                .font(theme::SMALL_MED)
                                .color(if selected { Color::WHITE } else { p.text_muted }),
                        ),
                );
            }
            chips
        })
        .child(
            text({
                let (done, total) = crate::i18n::coverage();
                if done == 0 {
                    "English is the source language; picking another translates the navigation and settings chrome.".to_string()
                } else {
                    format!("{done} of {total} navigation strings translated.")
                }
            })
            .font(theme::TINY)
            .color(p.text_faint),
        )
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
        .child(ai_edit_card(s, p))
}

/// Settings → Appearance → "Edit your own UI".
///
/// The model writes view code only; the app builds it into a separate binary and
/// keeps the previous one, so a bad edit is a restart away from being undone.
fn ai_edit_card(s: &App, p: Palette) -> El {
    let ui = &s.ui_edit;
    let running = crate::aiedit::busy();
    let available = crate::aiedit::source_available();
    let can_undo = ui.history.iter().any(|version| version.name != "base");

    // A quiet, bordered button: secondary actions should not compete with the
    // one accent-coloured primary action.
    let ghost = |key: &'static str, label: &'static str, height: f32| {
        el().key(key)
            .h(height)
            .px(14.0)
            .radius(6.0)
            .center()
            .role("button")
            .label(label)
            .fill(Color::TRANSPARENT)
            .border(1.0, p.divider)
            .hover(p.rail_hover)
            .child(text(label).font(theme::SMALL_MED).color(p.text_muted))
    };

    let pill = |ink: Color, label: &'static str| {
        row()
            .h(22.0)
            .px(9.0)
            .radius(11.0)
            .gap(6.0)
            .center_y()
            .fill(ink.alpha(0.12))
            .child(el().size(6.0, 6.0).radius(3.0).fill(ink))
            .child(text(label).font(theme::TINY).color(ink))
    };

    let input =
        |name: &'static str, key: &'static str, edit: &rune::TextEdit, hint: &'static str| {
            let mapped = match key {
                "ai-url" => "ai-url",
                "ai-model" => "ai-model",
                _ => "ai-key",
            };
            text_input(edit, theme::SMALL)
                .key(key)
                .w_full()
                .h(36.0)
                .radius(6.0)
                .fill(p.input)
                .border(1.0, p.divider_soft)
                .color(p.text)
                .role("textbox")
                .label(name)
                .placeholder(hint, p.text_faint)
                .padding(10.0)
                .single_line()
                .on_edit(move |s: &mut App| match mapped {
                    "ai-url" => &mut s.ui_edit.base_url,
                    "ai-model" => &mut s.ui_edit.model,
                    _ => &mut s.ui_edit.key,
                })
        };

    let field =
        |label: &'static str, key: &'static str, edit: &rune::TextEdit, hint: &'static str| {
            col()
                .grow(1.0)
                .min_w(0.0)
                .gap(6.0)
                .child(text(label).font(theme::LABEL).color(p.text_faint))
                .child(input(label, key, edit, hint))
        };

    let chip = |key: &'static str, label: &'static str, p: Palette| {
        el().key(key)
            .h(28.0)
            .px(10.0)
            .radius(14.0)
            .gap(6.0)
            .center_y()
            .role("button")
            .label(label)
            .fill(Color::TRANSPARENT)
            .border(1.0, p.divider)
            .hover(p.rail_hover)
            .on_press(move |s: &mut App| s.ui_edit.prompt.replace_text(label))
            .child(icon(&icons::PLUS, 11.0).color(p.text_faint).a11y_hidden())
            .child(text(label).font(theme::TINY).color(p.text_muted))
    };

    // The state is a badge in the header, not a paragraph in the middle of the
    // card, so the idle card stays quiet and a failure is hard to miss.
    let badge = match &ui.stage {
        crate::aiedit::Stage::Idle => None,
        crate::aiedit::Stage::Running(_) => Some(pill(p.yellow, "Working")),
        crate::aiedit::Stage::Staged(_) => Some(pill(p.green, "Build ready")),
        crate::aiedit::Stage::Checked(_) => Some(pill(p.green, "Key works")),
        crate::aiedit::Stage::Briefed(_) => Some(pill(p.green, "Brief ready")),
        crate::aiedit::Stage::Failed(_) => Some(pill(p.danger, "Needs attention")),
    };

    let mut header = row()
        .w_full()
        .gap(12.0)
        .center_y()
        .child(
            el().size(32.0, 32.0)
                .radius(8.0)
                .fill(p.accent.alpha(0.14))
                .center()
                .child(icon(&icons::SPARKLES, 17.0).color(p.accent).a11y_hidden()),
        )
        .child(
            text("EDIT YOUR OWN UI")
                .font(theme::SECTION)
                .color(p.text_muted)
                .grow(1.0),
        );
    header = header.child(pill(p.accent, "Preview"));
    if let Some(badge) = badge {
        header = header.child(badge);
    }

    let divider = || el().w_full().h(1.0).fill(p.divider_soft);

    if !available {
        let note = crate::aiedit::source_error()
            .unwrap_or_else(|| "The UI source tree is not installed.".to_string());
        return col()
            .key("ai-edit")
            .w_full()
            .radius(10.0)
            .clip_rounded(10.0)
            .fill(p.card)
            .child(
                col()
                    .w_full()
                    .padding(18.0)
                    .pb(16.0)
                    .gap(10.0)
                    .child(header.child(pill(p.yellow, "No source")))
                    .child(
                        text("This build cannot run the editor: the UI source tree is not installed. Nothing is sent to a provider.")
                            .wrap()
                            .font(theme::SMALL)
                            .color(p.text_muted),
                    ),
            )
            .child(divider())
            .child(
                col()
                    .w_full()
                    .padding(18.0)
                    .gap(8.0)
                    .child(
                        row()
                            .w_full()
                            .gap(8.0)
                            .center_y()
                            .child(icon(&icons::CLOCK, 14.0).color(p.yellow).a11y_hidden())
                            .child(
                                text("THE ONE-TIME SOURCE DOWNLOAD IS NOT PART OF THIS PREVIEW")
                                    .font(theme::LABEL)
                                    .color(p.yellow)
                                    .grow(1.0)
                                    .min_w(0.0)
                                    .wrap(),
                            ),
                    )
                    .child(text(note).wrap().font(theme::SMALL).color(p.text_muted))
                    .child(
                        text("Use a development checkout, or install the UI source and Rust toolchain bundle, to enable Generate. The rest of the app is unaffected.")
                            .wrap()
                            .font(theme::TINY)
                            .color(p.text_faint),
                    ),
            );
    }

    let mut card = col()
        .key("ai-edit")
        .w_full()
        .radius(10.0)
        .clip_rounded(10.0)
        .fill(p.card)
        .child(
            col()
                .w_full()
                .padding(18.0)
                .pb(16.0)
                .gap(10.0)
                .child(header)
                .child(
                    text("Describe a change in plain words — even something the app has never had. Attach reference images if a look matters. The app writes the UI source and builds a fresh copy beside the current one. Restart when it is ready.")
                        .wrap()
                        .font(theme::SMALL)
                        .color(p.text_muted),
                ),
        )
        .child(divider());

    let mut change = col()
        .w_full()
        .padding(18.0)
        .gap(10.0)
        .child(
            text("WHAT SHOULD CHANGE?")
                .font(theme::LABEL)
                .color(p.text_faint),
        )
        .child(
            text_input(&ui.prompt, theme::BODY)
                .key("ai-prompt")
                .w_full()
                .h(80.0)
                .radius(6.0)
                .fill(p.input)
                .border(1.0, p.divider_soft)
                .color(p.text)
                .role("textbox")
                .label("Describe the UI change")
                .placeholder(
                    "Move the channel sidebar to the right of the chat.",
                    p.text_faint,
                )
                .padding(11.0)
                .on_edit(|s: &mut App| &mut s.ui_edit.prompt)
                .on_input_paste(|s: &mut App, content, _: &mut rune::Tx| {
                    if let Some(image) = content.image() {
                        s.ui_edit_attach_bytes(
                            format!("pasted-{}.png", now_ms()),
                            image.png().to_vec(),
                        );
                        return true;
                    }
                    if let Some(path) = content.files().first() {
                        s.ui_edit_attach(path.clone());
                        return true;
                    }
                    false
                }),
        )
        .child(
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .child(
                    ghost("ai-attach", "Attach image", 30.0)
                        .help("Add a reference image the model can look at")
                        .on_press_tx(|_s: &mut App, tx: &mut rune::Tx| {
                            crate::files::open(
                                tx,
                                crate::files::PickKind::UiReference,
                                "Attach a reference image",
                                &["png", "jpg", "jpeg", "webp", "gif"],
                                |s, path| s.ui_edit_attach(path),
                            );
                        }),
                )
                .child(
                    text("or paste / drop an image on the box")
                        .font(theme::TINY)
                        .color(p.text_faint)
                        .grow(1.0)
                        .min_w(0.0),
                ),
        )
        .child(
            row()
                .w_full()
                .gap(6.0)
                .flex_wrap()
                .child(chip("ai-chip-right", "Sidebar on the right", p))
                .child(chip("ai-chip-compact", "Tighter message rows", p))
                .child(chip("ai-chip-bigger", "Bigger chat type", p))
                .child(chip("ai-chip-members", "Hide the member list", p))
                .child(chip("ai-chip-mascot", "Add a mascot in the corner", p))
                .child(chip("ai-chip-playful", "Playful, rounded icons", p)),
        )
        .child(
            row()
                .w_full()
                .gap(8.0)
                .padding(9.0)
                .radius(6.0)
                .fill(p.code_bg)
                .border(1.0, p.divider_soft)
                .child(icon(&icons::LOCK, 12.0).color(p.text_faint).a11y_hidden())
                .child(
                    text(format!(
                        "Your request, reference images and the UI files the editor reads go to the provider above. That provider bills your key. One edit is capped at {} model calls and {} repairs.",
                        crate::aiedit::MAX_STEPS,
                        crate::aiedit::MAX_REPAIRS
                    ))
                    .wrap()
                    .font(theme::TINY)
                    .color(p.text_faint)
                    .grow(1.0)
                    .min_w(0.0),
                ),
        )
        .child({
            let label = if running {
                "Building…"
            } else {
                "Generate & build"
            };
            let mut generate = el()
                .key("ai-generate")
                .h(36.0)
                .px(18.0)
                .radius(6.0)
                .gap(8.0)
                .center()
                .role("button")
                .label(label)
                .fill(if running { p.rail } else { p.accent })
                .disabled(running)
                .child(
                    icon(&icons::SPARKLES, 14.0)
                        .color(if running { p.text_faint } else { Color::WHITE })
                        .a11y_hidden(),
                )
                .child(text(label).font(theme::SMALL_MED).color(if running {
                    p.text_faint
                } else {
                    Color::WHITE
                }));
            if !running {
                generate = generate
                    .hover(p.accent_hover)
                    .on_press(|s: &mut App| s.ui_edit_begin());
            }
            let mut actions = row().w_full().gap(8.0).center_y();
            actions = actions.child(generate);
            if !running {
                actions = actions.child(
                    ghost("ai-refine", "Refine idea", 34.0)
                        .help("Turn the request into a detailed brief before building")
                        .on_press(|s: &mut App| s.ui_edit_refine()),
                );
            }
            // Undo and reset restart the app, so they stay out of reach while a
            // build is running.
            if !running {
                let mut secondary = Vec::new();
                if can_undo {
                    secondary.push(
                        ghost("ai-undo", "Undo last change", 34.0)
                            .help("Restart on the build from before the last change")
                            .on_press_tx(|s: &mut App, tx: &mut rune::Tx| match s.ui_edit_undo() {
                                Ok(()) => tx.quit(),
                                Err(error) => s.ui_edit.stage = crate::aiedit::Stage::Failed(error),
                            }),
                    );
                }
                secondary.push(
                    ghost("ai-reset", "Reset to default UI", 34.0)
                        .help("Restart on the UI that shipped with the app")
                        .on_press_tx(|s: &mut App, tx: &mut rune::Tx| match s.ui_edit_reset() {
                            Ok(()) => tx.quit(),
                            Err(error) => s.ui_edit.stage = crate::aiedit::Stage::Failed(error),
                        }),
                );
                actions = actions.child(el().grow(1.0)).children(secondary);
            }
            actions
        });

    if !ui.attachments.is_empty() {
        let mut strip = row().w_full().gap(8.0).flex_wrap();
        for (index, attachment) in ui.attachments.iter().enumerate() {
            let tile = row()
                .w(190.0)
                .h(50.0)
                .px(6.0)
                .radius(8.0)
                .gap(8.0)
                .center_y()
                .fill(p.input)
                .border(1.0, p.divider_soft)
                .child(
                    el().size(38.0, 38.0)
                        .radius(6.0)
                        .clip_rounded(6.0)
                        .pixels_fit(&attachment.pixels, rune::SurfaceFit::Fill),
                )
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(
                            text(attachment.name.clone())
                                .font(theme::TINY)
                                .color(p.text)
                                .truncate()
                                .help_when_truncated(attachment.name.clone()),
                        )
                        .child(
                            text(format!("{}×{}", attachment.width, attachment.height))
                                .font(theme::TINY)
                                .color(p.text_faint),
                        ),
                )
                .child(
                    el().key(format!("ai-attach-remove-{index}"))
                        .size(20.0, 20.0)
                        .radius(10.0)
                        .center()
                        .role("button")
                        .label("Remove reference image")
                        .hover(p.rail_hover)
                        .on_press(move |s: &mut App| {
                            if index < s.ui_edit.attachments.len() {
                                s.ui_edit.attachments.remove(index);
                            }
                        })
                        .child(icon(&icons::CLOSE, 11.0).color(p.text_muted).a11y_hidden()),
                );
            strip = strip.child(tile);
        }
        change = change.child(strip);
    }

    if let Some(brief) = &ui.brief {
        let shown: String = brief.lines().take(8).collect::<Vec<_>>().join("\n");
        let hidden = brief.lines().count().saturating_sub(8);
        let mut panel = col()
            .w_full()
            .radius(6.0)
            .fill(p.code_bg)
            .border(1.0, p.divider_soft)
            .padding(10.0)
            .gap(4.0)
            .child(
                row()
                    .w_full()
                    .gap(6.0)
                    .center_y()
                    .child(
                        text("GENERATED BRIEF")
                            .font(theme::TINY)
                            .color(p.text_link)
                            .grow(1.0),
                    )
                    .child(
                        el().key("ai-use-brief")
                            .h(20.0)
                            .px(8.0)
                            .radius(10.0)
                            .center()
                            .role("button")
                            .label("Use the brief as the prompt")
                            .fill(Color::TRANSPARENT)
                            .border(1.0, p.divider)
                            .hover(p.rail_hover)
                            .on_press(|s: &mut App| s.ui_edit_use_brief())
                            .child(text("Use as prompt").font(theme::TINY).color(p.text_muted)),
                    )
                    .child(
                        text(format!("{} lines", brief.lines().count()))
                            .font(theme::TINY)
                            .color(p.text_faint),
                    ),
            )
            .child(text(shown).wrap().font(theme::MONO).color(p.text_faint));
        if hidden > 0 {
            panel = panel.child(
                text(format!(
                    "… {hidden} more lines are included in the build request."
                ))
                .font(theme::TINY)
                .color(p.text_faint),
            );
        }
        change = change.child(panel);
    }

    if let crate::aiedit::Stage::Running(step) = &ui.stage {
        let mut log = col()
            .w_full()
            .radius(6.0)
            .fill(p.code_bg)
            .border(1.0, p.divider_soft)
            .padding(10.0)
            .gap(2.0)
            .child(
                text(step.clone())
                    .font(theme::MONO)
                    .color(p.text_link)
                    .truncate(),
            );
        for line in ui.detail.iter().skip(ui.detail.len().saturating_sub(6)) {
            log = log.child(
                text(line.clone())
                    .font(theme::MONO)
                    .color(p.text_faint)
                    .truncate(),
            );
        }
        change = change.child(log);
    }

    if let crate::aiedit::Stage::Staged(version) = &ui.stage {
        change = change.child(
            row()
                .w_full()
                .gap(12.0)
                .center_y()
                .padding(12.0)
                .radius(6.0)
                .fill(p.green.alpha(0.10))
                .border(1.0, p.green.alpha(0.30))
                .child(icon(&icons::CHECK, 16.0).color(p.green).a11y_hidden())
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(
                            text(format!("Build {version} is ready"))
                                .font(theme::SMALL_MED)
                                .color(p.text_bright),
                        )
                        .child(
                            text("Restart to switch. The current build keeps running until the new one starts.")
                                .wrap()
                                .font(theme::TINY)
                                .color(p.text_muted),
                        ),
                )
                .child(
                    el().key("ai-restart")
                        .h(32.0)
                        .px(14.0)
                        .radius(6.0)
                        .center()
                        .role("button")
                        .label("Restart the application")
                        .fill(Color::TRANSPARENT)
                        .border(1.0, p.green.alpha(0.5))
                        .hover(p.green.alpha(0.12))
                        .on_press_tx(|s: &mut App, tx: &mut rune::Tx| {
                            if s.ui_edit_restart().is_ok() {
                                tx.quit();
                            }
                        })
                        .child(
                            text("Restart now")
                                .font(theme::SMALL_MED)
                                .color(p.green),
                        ),
                ),
        );
    }

    if let crate::aiedit::Stage::Checked(message) = &ui.stage {
        change = change.child(
            row()
                .w_full()
                .gap(6.0)
                .center_y()
                .child(icon(&icons::CHECK, 13.0).color(p.green).a11y_hidden())
                .child(
                    text(message.clone())
                        .wrap()
                        .font(theme::TINY)
                        .color(p.green),
                ),
        );
    }

    if let crate::aiedit::Stage::Failed(error) = &ui.stage {
        change = change.child(
            row()
                .w_full()
                .gap(10.0)
                .padding(10.0)
                .radius(6.0)
                .fill(p.danger.alpha(0.10))
                .border(1.0, p.danger.alpha(0.30))
                .child(icon(&icons::CLOSE, 13.0).color(p.danger).a11y_hidden())
                .child(
                    text(error.clone())
                        .grow(1.0)
                        .min_w(0.0)
                        .wrap()
                        .font(theme::TINY)
                        .color(p.danger),
                ),
        );
    }

    if !running && !ui.key_saved {
        change = change.child(
            text("Add a provider key below before the first build.")
                .font(theme::TINY)
                .color(p.text_faint),
        );
    }

    card = card
        .child(change)
        .child(divider())
        .child({
            let builds = ui
                .history
                .iter()
                .filter(|version| version.name != "base")
                .count();
            let mut versions = col().w_full().padding(18.0).gap(8.0).child(
                row()
                    .w_full()
                    .center_y()
                    .child(
                        text("VERSIONS")
                            .font(theme::SECTION)
                            .color(p.text_muted)
                            .grow(1.0),
                    )
                    .child(
                        text(if builds == 0 {
                            "No builds yet".to_string()
                        } else if builds == 1 {
                            "1 build".to_string()
                        } else {
                            format!("{builds} builds")
                        })
                        .font(theme::TINY)
                        .color(p.text_faint),
                    ),
            );

            // The same files can be deleted by hand from this folder; showing it
            // here is what makes that possible without the docs.
            let store = crate::aiedit::store_dir();
            versions = versions.child(
                text(format!("Saved in {store}"))
                    .font(theme::MONO)
                    .color(p.text_faint)
                    .truncate()
                    .help_when_truncated(store),
            );

            let app = crate::aiedit::app_version();
            let outdated = if ui.kept_app_version == app {
                None
            } else {
                ui.history
                    .iter()
                    .find(|version| version.current)
                    .and_then(|version| {
                        let built = version.app_version.trim();
                        (!built.is_empty() && built != app).then(|| built.to_string())
                    })
            };
            if let Some(built) = outdated {
                let current = ui
                    .history
                    .iter()
                    .find(|version| version.current && version.source)
                    .map(|version| version.name.clone());
                let mut actions = row().w_full().gap(8.0).center_y();
                actions = actions.child(
                    ghost("ai-keep-old", "Keep for now", 30.0)
                        .on_press(|s: &mut App| s.ui_edit_keep_current()),
                );
                if let Some(name) = current {
                    actions = actions.child(
                        ghost("ai-rebuild-old", "Rebuild", 30.0)
                            .on_press(move |s: &mut App| s.ui_edit_rebuild(&name)),
                    );
                }
                actions = actions.child(
                    ghost("ai-reset-old", "Reset to default", 30.0).on_press_tx(
                        |s: &mut App, tx: &mut rune::Tx| match s.ui_edit_reset() {
                            Ok(()) => tx.quit(),
                            Err(error) => s.ui_edit.stage = crate::aiedit::Stage::Failed(error),
                        },
                    ),
                );
                versions = versions.child(
                    col()
                        .w_full()
                        .gap(8.0)
                        .padding(12.0)
                        .radius(6.0)
                        .fill(p.yellow.alpha(0.08))
                        .border(1.0, p.yellow.alpha(0.30))
                        .child(
                            row()
                                .w_full()
                                .gap(8.0)
                                .center_y()
                                .child(icon(&icons::CLOCK, 14.0).color(p.yellow).a11y_hidden())
                                .child(
                                    text(format!(
                                        "This UI was built for Runecord {built}; this app is {app}."
                                    ))
                                    .font(theme::SMALL_MED)
                                    .color(p.text_bright)
                                    .grow(1.0)
                                    .min_w(0.0)
                                    .wrap(),
                                ),
                        )
                        .child(
                            text("Rebuild it against the current app, or reset to the shipped UI. Keeping it leaves the old build running and stops this warning until the app changes again.")
                                .wrap()
                                .font(theme::TINY)
                                .color(p.text_muted),
                        )
                        .child(actions),
                );
            }

            if ui.history.iter().any(|version| version.name != "base") {
                let mut list = col().w_full().gap(2.0);
                for version in ui.history.iter() {
                    let mut entry = row()
                        .w_full()
                        .h(34.0)
                        .px(8.0)
                        .radius(6.0)
                        .gap(8.0)
                        .center_y();
                    if version.current {
                        entry = entry.fill(p.rail.alpha(0.55)).child(
                            el().size(6.0, 6.0).radius(3.0).fill(p.green),
                        );
                    } else if version.binary && version.source {
                        let name = version.name.clone();
                        entry = entry
                            .key(format!("ai-use-{}", version.name))
                            .role("button")
                            .label(format!("Use {}", version.goal))
                            .help("Switch to this build and restart the app")
                            .hover(p.rail_hover)
                            .on_press_tx(move |s: &mut App, tx: &mut rune::Tx| {
                                if s.ui_edit_use(&name).is_ok() {
                                    tx.quit();
                                }
                            })
                            .child(el().size(6.0, 6.0).radius(3.0).fill(if version.pending {
                                p.yellow
                            } else {
                                p.divider
                            }));
                    } else if version.source {
                        let name = version.name.clone();
                        entry = entry
                            .key(format!("ai-rebuild-{}", version.name))
                            .role("button")
                            .label(format!("Rebuild {}", version.goal))
                            .help("Rebuild this version from its saved source (1–3 minutes)")
                            .hover(p.rail_hover)
                            .on_press(move |s: &mut App| s.ui_edit_rebuild(&name))
                            .child(el().size(6.0, 6.0).radius(3.0).fill(p.text_link));
                    } else {
                        entry = entry.child(el().size(6.0, 6.0).radius(3.0).fill(p.divider));
                    }
                    let state = if version.current {
                        text("In use").font(theme::TINY).color(p.green)
                    } else if version.pending {
                        text("Staged").font(theme::TINY).color(p.yellow)
                    } else if version.binary && !version.source {
                        text("Fallback").font(theme::TINY).color(p.text_faint)
                    } else if !version.binary && version.source {
                        text("Rebuild").font(theme::TINY).color(p.text_link)
                    } else if !version.binary {
                        text("No source").font(theme::TINY).color(p.text_faint)
                    } else {
                        text("Use").font(theme::TINY).color(p.text_muted)
                    };
                    list = list.child(
                        entry
                            .child(
                                text(version.goal.clone())
                                    .font(theme::SMALL)
                                    .color(if version.current {
                                        p.text_bright
                                    } else {
                                        p.text_muted
                                    })
                                    .grow(1.0)
                                    .min_w(0.0)
                                    .truncate()
                                    .help_when_truncated(version.goal.clone()),
                            )
                            .child(
                                row()
                                    .w(100.0)
                                    .justify_end()
                                    .child(
                                        text(version.when.clone())
                                            .font(theme::TINY)
                                            .color(p.text_faint),
                                    )
                            )
                            .child(row().w(46.0).justify_end().child(state)),
                    );
                }
                if ui
                    .history
                    .iter()
                    .any(|version| !version.binary && version.source)
                {
                    list = list.child(
                        text("Source-only versions rebuild when selected (1–3 minutes).")
                            .font(theme::TINY)
                            .color(p.text_faint)
                            .padding(8.0),
                    );
                }
                versions = versions.child(list);
            } else {
                versions = versions.child(
                    col()
                        .w_full()
                        .padding(10.0)
                        .radius(6.0)
                        .fill(p.rail.alpha(0.35))
                        .child(
                            text("No custom builds yet. The first generated build appears here.")
                                .font(theme::TINY)
                                .color(p.text_faint),
                        ),
                );
            }
            // Deleting sits with the list it clears: the builds, their source
            // copies and the rollback files. The path above says where they are.
            let armed = ui.delete_armed;
            let label = if armed {
                "Press again to delete"
            } else {
                "Delete saved UI files"
            };
            versions = versions.child(
                row()
                    .w_full()
                    .gap(8.0)
                    .center_y()
                    .child(
                        text("This cannot be undone. Provider settings and the key stay.")
                            .font(theme::TINY)
                            .color(p.text_faint)
                            .grow(1.0)
                            .min_w(0.0)
                            .truncate(),
                    )
                    .child(
                        el().key("ai-delete-files")
                            .h(30.0)
                            .px(12.0)
                            .radius(6.0)
                            .center()
                            .role("button")
                            .label(label)
                            .help(
                                "Delete the custom UI builds, source copies and rollback files from disk",
                            )
                            .fill(if armed {
                                p.danger.alpha(0.12)
                            } else {
                                Color::TRANSPARENT
                            })
                            .border(1.0, if armed { p.danger.alpha(0.5) } else { p.divider })
                            .hover(p.danger.alpha(0.12))
                            .on_press(|s: &mut App| {
                                if s.ui_edit.delete_armed {
                                    s.ui_edit_delete_files();
                                } else {
                                    s.ui_edit.delete_armed = true;
                                    s.toast("Press the button again to delete the saved UI files.");
                                }
                            })
                            .child(
                                text(label).font(theme::SMALL_MED).color(if armed {
                                    p.danger
                                } else {
                                    p.text_muted
                                }),
                            ),
                    ),
            );
            versions
        })
        .child(divider())
        .child(
            col()
                .w_full()
                .padding(18.0)
                .gap(12.0)
                .child(text("PROVIDER").font(theme::SECTION).color(p.text_muted))
                .child(
                    row()
                        .w_full()
                        .gap(10.0)
                        .child(field(
                            "BASE URL",
                            "ai-url",
                            &ui.base_url,
                            "https://opencode.ai/inference/go/openai/v1",
                        ))
                        .child(field("MODEL", "ai-model", &ui.model, "deepseek-v4.1-flash"))
                )
                .child(
                    col()
                        .w_full()
                        .gap(6.0)
                        .child(
                            row()
                                .w_full()
                                .center_y()
                                .child(
                                    text("API KEY")
                                        .font(theme::LABEL)
                                        .color(p.text_faint)
                                        .grow(1.0),
                                )
                                .child(
                                    row()
                                        .gap(6.0)
                                        .center_y()
                                        .child(el().size(6.0, 6.0).radius(3.0).fill(
                                            if ui.key_saved { p.green } else { p.text_faint },
                                        ))
                                        .child(
                                            text(if ui.key_saved {
                                                "Saved for this account"
                                            } else {
                                                "No key saved"
                                            })
                                            .font(theme::TINY)
                                            .color(if ui.key_saved {
                                                p.green
                                            } else {
                                                p.text_faint
                                            }),
                                        ),
                                ),
                        )
                        .child(
                            row()
                                .w_full()
                                .gap(8.0)
                                .center_y()
                                .child(input("API key", "ai-key", &ui.key, "sk-…").grow(1.0))
                                .child(
                                    ghost("ai-save-key", "Save key", 36.0)
                                        .on_press(|s: &mut App| s.ui_edit_save_key()),
                                )
                                .child(
                                    ghost("ai-test-key", "Test connection", 36.0)
                                        .on_press(|s: &mut App| s.ui_edit_test_key()),
                                )
                        ),
                )
                .child(
                    row()
                        .w_full()
                        .gap(6.0)
                        .center_y()
                        .child(icon(&icons::LOCK, 12.0).color(p.text_faint).a11y_hidden())
                        .child(
                            text(if cfg!(windows) {
                                "Sealed with DPAPI for this Windows account; never written to provider.json."
                            } else {
                                "Stored in aiedit/.env with 0600 permissions; never written to provider.json."
                            })
                            .wrap()
                            .font(theme::TINY)
                            .color(p.text_faint),
                        ),
                ),
        );
    card
}
