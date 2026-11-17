//! Server templates settings.

use super::*;

pub(crate) fn templates_view(s: &App, p: Palette) -> El {
    let mut list = col().w_full().gap(8.0);
    if s.templates.is_empty() {
        list = list.child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(16.0)
                .gap(4.0)
                .child(
                    text("No templates yet")
                        .font(theme::BODY_BOLD)
                        .color(p.text_bright),
                )
                .child(
                    text(
                        "A template saves this server's channels and roles so you can reuse them.",
                    )
                    .font(theme::SMALL)
                    .color(p.text_muted),
                ),
        );
    }
    for template in &s.templates {
        let id = template.id;
        let code = template.code.clone();
        let use_code = template.code.clone();
        list = list.child(
            row()
                .key(format!("template-{id}"))
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
                            text(template.name.clone())
                                .font(theme::BODY_BOLD)
                                .color(p.text_bright),
                        )
                        .children((!template.description.is_empty()).then(|| {
                            text(template.description.clone())
                                .font(theme::TINY)
                                .color(p.text_muted)
                        }))
                        .child(
                            text(format!(
                                "{} channels - {} roles - used {} times",
                                template.channel_count, template.role_count, template.uses
                            ))
                            .font(theme::TINY)
                            .color(p.text_faint),
                        ),
                )
                .child(
                    el().key(format!("template-code-{id}"))
                        .h(28.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Copy template code {}", template.code))
                        .fill(p.input)
                        .hover(p.input_hover)
                        .on_press_tx(move |_: &mut App, tx: &mut rune::Tx| {
                            tx.copy(code.clone());
                        })
                        .child(
                            text(template.code.clone())
                                .font(theme::MONO)
                                .color(p.text_link),
                        ),
                )
                .child(
                    el().key(format!("template-use-{id}"))
                        .h(28.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Use template {}", template.name))
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(move |s: &mut App| s.apply_template(use_code.clone()))
                        .child(text("Use").font(theme::TINY).color(Color::WHITE)),
                )
                .child(
                    el().key(format!("template-delete-{id}"))
                        .size(28.0, 28.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Delete {}", template.name))
                        .hover(p.danger)
                        .on_press(move |s: &mut App| s.delete_template(id))
                        .child(icon(&icons::TRASH, 15.0).color(p.danger)),
                ),
        );
    }

    col()
        .key("templates-view")
        .w_full()
        .max_w(720.0)
        .gap(12.0)
        .child(settings_title(
            "Templates",
            "Snapshot this server's channels and roles, then start a new server from it.",
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
                .child(
                    text("Create template")
                        .font(theme::BODY_BOLD)
                        .color(p.text_bright),
                )
                .child(
                    text_input(&s.template_name, theme::SMALL)
                        .key("template-name")
                        .w_full()
                        .h(36.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Template name")
                        .placeholder("Community starter", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.template_name),
                )
                .child(
                    text_input(&s.template_description, theme::SMALL)
                        .key("template-description")
                        .w_full()
                        .h(36.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Template description")
                        .placeholder("What is this template for?", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.template_description),
                )
                .child(
                    row().w_full().justify_end().child(
                        el().key("template-create")
                            .h(34.0)
                            .px(16.0)
                            .radius(4.0)
                            .center()
                            .role("button")
                            .label("Create template")
                            .fill(p.accent)
                            .hover(p.accent_hover)
                            .on_press(|s: &mut App| s.create_template())
                            .child(
                                text("Create Template")
                                    .font(theme::SMALL_MED)
                                    .color(Color::WHITE),
                            ),
                    ),
                ),
        )
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(16.0)
                .gap(6.0)
                .child(
                    text("Using a template")
                        .font(theme::BODY_BOLD)
                        .color(p.text_bright),
                )
                .child(
                    text(
                        "Copy a template code and paste it into the Create Server dialog to start a new server from it.",
                    )
                    .font(theme::TINY)
                    .color(p.text_muted),
                ),
        )
}
