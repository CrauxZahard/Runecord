//! AutoMod rules settings.

use super::*;

const TRIGGERS: [(&str, &str); 4] = [
    ("keyword", "Keyword"),
    ("link", "Link"),
    ("mention_spam", "Mention spam"),
    ("caps", "Excessive caps"),
];

const ACTIONS: [(&str, &str); 3] = [
    ("block", "Block message"),
    ("timeout", "Timeout member"),
    ("alert", "Alert only"),
];

pub(crate) fn automod_view(s: &App, p: Palette) -> El {
    let mut list = col().w_full().gap(8.0);
    if s.automod_rules.is_empty() {
        list = list.child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(16.0)
                .gap(4.0)
                .child(
                    text("No AutoMod rules yet")
                        .font(theme::BODY_BOLD)
                        .color(p.text_bright),
                )
                .child(
                    text("Add a rule below and it applies to every new message in this server.")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                ),
        );
    }
    for (index, rule) in s.automod_rules.iter().enumerate() {
        let trigger_label = TRIGGERS
            .iter()
            .find(|(id, _)| *id == rule.trigger)
            .map(|(_, label)| *label)
            .unwrap_or("Keyword");
        let action_label = ACTIONS
            .iter()
            .find(|(id, _)| *id == rule.action)
            .map(|(_, label)| *label)
            .unwrap_or("Block message");
        let detail = match rule.trigger.as_str() {
            "keyword" => format!("keywords: {}", rule.pattern),
            "mention_spam" => format!(
                "more than {} mentions",
                if rule.pattern.is_empty() {
                    "5"
                } else {
                    &rule.pattern
                }
            ),
            "caps" => format!(
                "{}% or more capital letters",
                if rule.pattern.is_empty() {
                    "70"
                } else {
                    &rule.pattern
                }
            ),
            _ => "any http or https link".to_string(),
        };
        list = list.child(
            row()
                .key(format!("automod-rule-{index}"))
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(12.0)
                .gap(10.0)
                .center_y()
                .child(
                    col()
                        .gap(2.0)
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            row()
                                .gap(8.0)
                                .center_y()
                                .child(
                                    text(rule.name.clone())
                                        .font(theme::BODY_BOLD)
                                        .color(p.text_bright),
                                )
                                .child(
                                    el().px(6.0)
                                        .h(18.0)
                                        .radius(9.0)
                                        .center()
                                        .fill(p.input)
                                        .child(
                                            text(trigger_label)
                                                .font(theme::TINY)
                                                .color(p.text_muted),
                                        ),
                                ),
                        )
                        .child(
                            text(format!("{detail} - {action_label}"))
                                .font(theme::TINY)
                                .color(p.text_faint),
                        ),
                )
                .child(
                    el().key(format!("automod-toggle-{index}"))
                        .h(28.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("checkbox")
                        .label(format!("Toggle {}", rule.name))
                        .checked(rule.enabled)
                        .fill(if rule.enabled { p.green } else { p.input })
                        .on_press(move |s: &mut App| s.toggle_automod_rule(index))
                        .child(
                            text(if rule.enabled { "On" } else { "Off" })
                                .font(theme::TINY)
                                .color(if rule.enabled {
                                    Color::WHITE
                                } else {
                                    p.text_muted
                                }),
                        ),
                )
                .child(
                    el().key(format!("automod-delete-{index}"))
                        .size(28.0, 28.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Delete {}", rule.name))
                        .hover(p.danger)
                        .on_press(move |s: &mut App| s.delete_automod_rule(index))
                        .child(icon(&icons::TRASH, 15.0).color(p.danger)),
                ),
        );
    }

    let mut trigger_row = row().gap(6.0).flex_wrap();
    for (id, label) in TRIGGERS {
        let selected = s.automod_trigger == id;
        trigger_row = trigger_row.child(
            el().key(format!("automod-trigger-{id}"))
                .h(30.0)
                .px(10.0)
                .radius(4.0)
                .center()
                .role("radio")
                .label(label)
                .checked(selected)
                .fill(if selected { p.accent } else { p.input })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.automod_trigger = id.to_string())
                .child(text(label).font(theme::TINY).color(if selected {
                    Color::WHITE
                } else {
                    p.text_muted
                })),
        );
    }
    let mut action_row = row().gap(6.0).flex_wrap();
    for (id, label) in ACTIONS {
        let selected = s.automod_action == id;
        action_row = action_row.child(
            el().key(format!("automod-action-{id}"))
                .h(30.0)
                .px(10.0)
                .radius(4.0)
                .center()
                .role("radio")
                .label(label)
                .checked(selected)
                .fill(if selected { p.accent } else { p.input })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.automod_action = id.to_string())
                .child(text(label).font(theme::TINY).color(if selected {
                    Color::WHITE
                } else {
                    p.text_muted
                })),
        );
    }
    let pattern_hint = match s.automod_trigger.as_str() {
        "keyword" => "Comma-separated keywords, e.g. spoilers, leak",
        "mention_spam" => "Maximum mentions before the rule trips",
        "caps" => "Capital-letter percentage that trips the rule",
        _ => "Not used for link rules",
    };

    col()
        .key("automod-view")
        .w_full()
        .max_w(720.0)
        .gap(12.0)
        .child(settings_title(
            "AutoMod",
            "Block or time out messages that break your server rules, before anyone sees them.",
            p,
        ))
        .child(list)
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(16.0)
                .gap(10.0)
                .child(text("New rule").font(theme::BODY_BOLD).color(p.text_bright))
                .child(
                    text_input(&s.automod_name, theme::SMALL)
                        .key("automod-name")
                        .w_full()
                        .h(36.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Rule name")
                        .placeholder("No spoilers", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.automod_name),
                )
                .child(trigger_row)
                .child(
                    text_input(&s.automod_pattern, theme::SMALL)
                        .key("automod-pattern")
                        .w_full()
                        .h(36.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label(pattern_hint)
                        .placeholder(pattern_hint, p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.automod_pattern),
                )
                .child(action_row)
                .children((s.automod_action == "timeout").then(|| {
                    row()
                        .gap(8.0)
                        .center_y()
                        .child(
                            text("Timeout minutes")
                                .font(theme::TINY)
                                .color(p.text_muted),
                        )
                        .child(
                            text_input(&s.automod_timeout, theme::SMALL)
                                .key("automod-timeout")
                                .w(80.0)
                                .h(32.0)
                                .radius(4.0)
                                .fill(p.input)
                                .color(p.text)
                                .role("textbox")
                                .label("Timeout minutes")
                                .padding(8.0)
                                .on_edit(|s: &mut App| &mut s.automod_timeout),
                        )
                }))
                .child(
                    row().w_full().justify_end().child(
                        el().key("automod-add")
                            .h(34.0)
                            .px(16.0)
                            .radius(4.0)
                            .center()
                            .role("button")
                            .label("Add rule")
                            .fill(p.accent)
                            .hover(p.accent_hover)
                            .on_press(|s: &mut App| s.add_automod_rule())
                            .child(text("Add Rule").font(theme::SMALL_MED).color(Color::WHITE)),
                    ),
                ),
        )
}
