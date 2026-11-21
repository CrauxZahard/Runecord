//! Slash command option sheet.

use super::*;

pub(crate) fn slash_sheet(s: &App, p: Palette) -> El {
    let Some(command) = &s.slash_command else {
        return el();
    };
    let schema = command
        .options
        .get("options")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let mut fields = col().w_full().gap(10.0);
    for (index, option) in schema.iter().enumerate() {
        let name = option
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("value")
            .to_string();
        let description = option
            .get("description")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        let required = option
            .get("required")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        fields = fields.child(
            col()
                .w_full()
                .gap(4.0)
                .child(
                    text(format!(
                        "{}{}",
                        name.to_uppercase(),
                        if required { " *" } else { "" }
                    ))
                    .font(theme::SECTION)
                    .color(p.text_muted),
                )
                .children(
                    (!description.is_empty())
                        .then(|| text(description).font(theme::TINY).color(p.text_faint)),
                )
                .child(
                    text_input(&s.slash_values[index], theme::SMALL)
                        .key(format!("slash-option-{index}"))
                        .w_full()
                        .h(36.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label(name.clone())
                        .placeholder(name, p.text_faint)
                        .padding(10.0)
                        .on_edit(move |s: &mut App| &mut s.slash_values[index]),
                ),
        );
    }
    if schema.is_empty() {
        fields = fields.child(
            text("This command takes no options.")
                .font(theme::SMALL)
                .color(p.text_faint),
        );
    }
    col()
        .key("slash-sheet")
        .w(460.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .padding(18.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text(format!("/{}", command.name))
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text(command.description.clone())
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(fields)
        .child(
            row()
                .w_full()
                .justify_end()
                .gap(8.0)
                .child(
                    el().key("slash-cancel")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Cancel")
                        .hover(p.card_hover)
                        .on_press(|s: &mut App| s.modal = Modal::None)
                        .child(text("Cancel").font(theme::SMALL).color(p.text_muted)),
                )
                .child(
                    el().key("slash-run")
                        .h(34.0)
                        .px(16.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Run command")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(|s: &mut App| s.run_slash_command())
                        .child(text("Run").font(theme::SMALL_MED).color(Color::WHITE)),
                ),
        )
}
