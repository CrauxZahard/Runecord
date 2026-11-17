//! Server onboarding and rules settings.

use super::*;

pub(crate) fn onboarding_view(s: &App, p: Palette) -> El {
    let levels = [
        "None",
        "Low - 5 minutes",
        "Medium - 10 minutes",
        "High - 1 day",
    ];
    col()
        .key("onboarding-view")
        .w_full()
        .max_w(720.0)
        .gap(12.0)
        .child(settings_title(
            "Onboarding",
            "Welcome new members with a welcome screen, rules and a verification level.",
            p,
        ))
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(16.0)
                .gap(10.0)
                .child(
                    row()
                        .w_full()
                        .center_y()
                        .gap(12.0)
                        .child(
                            text("Welcome Screen")
                                .font(theme::BODY_BOLD)
                                .color(p.text_bright)
                                .grow(1.0),
                        )
                        .child(
                            el().key("onboarding-welcome-toggle")
                                .h(30.0)
                                .px(12.0)
                                .radius(4.0)
                                .center()
                                .role("checkbox")
                                .label("Enable welcome screen")
                                .checked(s.onboarding_welcome)
                                .fill(if s.onboarding_welcome {
                                    p.green
                                } else {
                                    p.input
                                })
                                .on_press(|s: &mut App| {
                                    s.onboarding_welcome = !s.onboarding_welcome
                                })
                                .child(
                                    text(if s.onboarding_welcome {
                                        "Enabled"
                                    } else {
                                        "Disabled"
                                    })
                                    .font(theme::TINY)
                                    .color(Color::WHITE),
                                ),
                        ),
                )
                .child(
                    text_input(&s.onboarding_title, theme::SMALL)
                        .key("onboarding-title")
                        .w_full()
                        .h(38.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Welcome title")
                        .placeholder("Welcome to the server!", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.onboarding_title),
                )
                .child(
                    text_input(&s.onboarding_description, theme::SMALL)
                        .key("onboarding-description")
                        .w_full()
                        .h(64.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Welcome description")
                        .placeholder("Tell people what this server is about.", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.onboarding_description),
                ),
        )
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(16.0)
                .gap(10.0)
                .child(
                    text("Rules Screening")
                        .font(theme::BODY_BOLD)
                        .color(p.text_bright),
                )
                .child(
                    text("One rule per line. New members must accept them before posting.")
                        .font(theme::TINY)
                        .color(p.text_muted),
                )
                .child(
                    text_input(&s.onboarding_rules, theme::SMALL)
                        .key("onboarding-rules")
                        .w_full()
                        .h(150.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Rules")
                        .placeholder("Be kind.\nNo spam.\nKeep it on topic.", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.onboarding_rules),
                ),
        )
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(16.0)
                .gap(10.0)
                .child(
                    text("Verification Level")
                        .font(theme::BODY_BOLD)
                        .color(p.text_bright),
                )
                .child(
                    text("In this local build, levels use account age.")
                        .font(theme::TINY)
                        .color(p.text_muted),
                )
                .child(
                    row()
                        .w_full()
                        .gap(6.0)
                        .children(levels.iter().enumerate().map(|(index, label)| {
                            let selected = s.onboarding_verification == index as u32;
                            el().key(format!("onboarding-level-{index}"))
                                .h(32.0)
                                .px(10.0)
                                .radius(4.0)
                                .center()
                                .role("radio")
                                .label(*label)
                                .checked(selected)
                                .fill(if selected { p.accent } else { p.input })
                                .hover(p.card_hover)
                                .on_press(move |s: &mut App| {
                                    s.onboarding_verification = index as u32
                                })
                                .child(text(*label).font(theme::TINY).color(if selected {
                                    Color::WHITE
                                } else {
                                    p.text_muted
                                }))
                        })),
                ),
        )
        .child(
            row().w_full().justify_end().child(
                el().key("onboarding-save")
                    .h(34.0)
                    .px(16.0)
                    .radius(4.0)
                    .center()
                    .role("button")
                    .label("Save onboarding")
                    .fill(p.green)
                    .hover(p.green.alpha(0.85))
                    .on_press(|s: &mut App| s.save_onboarding())
                    .child(
                        text("Save Changes")
                            .font(theme::SMALL_MED)
                            .color(Color::WHITE),
                    ),
            ),
        )
}
