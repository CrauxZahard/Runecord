//! Integrations: webhook management.

use super::*;

pub(crate) fn integrations_view(s: &App, p: Palette) -> El {
    let channels: Vec<(Id, String)> = s
        .channels
        .get(&s.active_guild)
        .map(|list| {
            list.iter()
                .filter(|channel| {
                    matches!(
                        channel.kind,
                        ChannelKind::Text | ChannelKind::Announcement | ChannelKind::Forum
                    )
                })
                .map(|channel| (channel.id, channel.name.clone()))
                .collect()
        })
        .unwrap_or_default();

    let mut list = col().w_full().gap(8.0);
    if s.webhooks.is_empty() {
        list = list.child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(16.0)
                .gap(4.0)
                .child(
                    text("No webhooks")
                        .font(theme::BODY_BOLD)
                        .color(p.text_bright),
                )
                .child(
                    text("Webhooks post into a channel from outside Runecord.")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                ),
        );
    }
    for hook in &s.webhooks {
        let id = hook.id;
        let channel_name = channels
            .iter()
            .find(|(channel, _)| *channel == hook.channel_id)
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| format!("#{}", hook.channel_id));
        list = list.child(
            row()
                .key(format!("webhook-{id}"))
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(12.0)
                .gap(10.0)
                .center_y()
                .child(
                    el().size(34.0, 34.0)
                        .radius(17.0)
                        .fill(theme::color(&format!("#{:06x}", hook.avatar_color)))
                        .center()
                        .child(icon(&icons::LINK, 18.0).color(Color::WHITE)),
                )
                .child(
                    col()
                        .gap(2.0)
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text(hook.name.clone())
                                .font(theme::BODY_BOLD)
                                .color(p.text_bright),
                        )
                        .child(
                            text(format!("posts in #{channel_name}"))
                                .font(theme::TINY)
                                .color(p.text_faint),
                        ),
                )
                .child(
                    el().key(format!("webhook-delete-{id}"))
                        .h(28.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Delete webhook {}", hook.name))
                        .hover(p.danger)
                        .on_press(move |s: &mut App| s.delete_webhook(id))
                        .child(text("Delete").font(theme::TINY).color(p.danger)),
                ),
        );
    }

    let mut channel_row = row().gap(6.0).flex_wrap();
    for (id, name) in &channels {
        let selected = s.webhook_channel == *id;
        let id = *id;
        channel_row = channel_row.child(
            el().key(format!("webhook-channel-{id}"))
                .h(30.0)
                .px(10.0)
                .radius(4.0)
                .center()
                .role("radio")
                .label(format!("Post to #{name}"))
                .checked(selected)
                .fill(if selected { p.accent } else { p.input })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.webhook_channel = id)
                .child(
                    text(format!("#{name}"))
                        .font(theme::TINY)
                        .color(if selected { Color::WHITE } else { p.text_muted }),
                ),
        );
    }

    col()
        .key("integrations-view")
        .w_full()
        .max_w(720.0)
        .gap(12.0)
        .child(settings_title(
            "Integrations",
            "Webhooks let other services post into your channels.",
            p,
        ))
        .child(list)
        .children(s.webhook_token.clone().map(|(id, token)| {
            row()
                .key("webhook-token")
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
                            text("Webhook token")
                                .font(theme::SECTION)
                                .color(p.text_muted),
                        )
                        .child(
                            text(format!("POST /api/webhooks/{id}/{token}"))
                                .font(theme::MONO)
                                .color(p.text_link)
                                .truncate(),
                        ),
                )
                .child(
                    el().key("webhook-token-copy")
                        .h(28.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Copy webhook token")
                        .fill(p.input)
                        .hover(p.input_hover)
                        .on_press_tx(move |_: &mut App, tx: &mut rune::Tx| {
                            tx.copy(token.clone());
                        })
                        .child(text("Copy").font(theme::TINY).color(p.text_bright)),
                )
        }))
        .child(
            col()
                .w_full()
                .radius(8.0)
                .fill(p.card)
                .padding(16.0)
                .gap(10.0)
                .child(
                    text("New webhook")
                        .font(theme::BODY_BOLD)
                        .color(p.text_bright),
                )
                .child(
                    text_input(&s.webhook_name, theme::SMALL)
                        .key("webhook-name")
                        .w_full()
                        .h(36.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Webhook name")
                        .placeholder("Build bot", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.webhook_name),
                )
                .child(channel_row)
                .child(
                    row().w_full().justify_end().child(
                        el().key("webhook-create")
                            .h(34.0)
                            .px(16.0)
                            .radius(4.0)
                            .center()
                            .role("button")
                            .label("Create webhook")
                            .fill(p.accent)
                            .hover(p.accent_hover)
                            .on_press(|s: &mut App| s.create_webhook())
                            .child(
                                text("Create Webhook")
                                    .font(theme::SMALL_MED)
                                    .color(Color::WHITE),
                            ),
                    ),
                ),
        )
}
