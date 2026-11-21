//! Modal sheets: server.

use super::*;

pub(crate) fn create_server_sheet(s: &App, p: Palette) -> El {
    modal_frame(
        "create-server",
        "Create a server",
        "Your server is where you and your people hang out. Make it yours.",
        p,
        col()
            .w_full()
            .gap(8.0)
            .child(text("SERVER NAME").font(theme::SECTION).color(p.text_muted))
            .child(modal_input(
                &s.profile_edit,
                "server-name",
                "My cool server",
                p,
            ))
            .child(
                text("You can always change it later.")
                    .font(theme::TINY)
                    .color(p.text_faint),
            )
            .child(
                text("TEMPLATE CODE (OPTIONAL)")
                    .font(theme::SECTION)
                    .color(p.text_muted)
                    .pt(6.0),
            )
            .child(modal_input(
                &s.template_code,
                "template-code",
                "Paste a template code",
                p,
            ))
            .child(
                row()
                    .pt(4.0)
                    .gap(4.0)
                    .child(
                        text("Have an invite?")
                            .font(theme::TINY)
                            .color(p.text_faint),
                    )
                    .child(
                        text("Join a Server")
                            .font(theme::TINY)
                            .color(p.text_link)
                            .on_press(|s: &mut App| s.open_invite_join()),
                    ),
            ),
        ("Create", |s: &mut App| s.create_guild_now()),
        p,
    )
}

pub(crate) fn confirm_leave_sheet(s: &App, p: Palette, id: Id) -> El {
    let name = s
        .guilds
        .iter()
        .find(|guild| guild.id == id)
        .map(|guild| guild.name.clone())
        .unwrap_or_else(|| "this server".to_string());
    modal_frame(
        "confirm-leave",
        "Leave Server",
        &format!(
            "Are you sure you want to leave {name}? You won't be able to rejoin unless you are invited again."
        ),
        p,
        col().w_full(),
        ("Leave Server", |s: &mut App| s.leave_current_guild()),
        p,
    )
}

pub(crate) fn edit_channel_sheet(s: &App, p: Palette) -> El {
    let channel = s.channel(s.editing_channel).cloned().unwrap_or_default();
    let channel_id = channel.id;
    col()
        .key("edit-channel")
        .w(460.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .backdrop_blur(4.0)
        .motion(motion::SNAPPY)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            col()
                .w_full()
                .px(20.0)
                .pt(20.0)
                .pb(16.0)
                .gap(14.0)
                .child(
                    row()
                        .w_full()
                        .items_center()
                        .gap(8.0)
                        .child(icon(&icons::SETTINGS, 20.0).color(p.text_muted))
                        .child(
                            text(format!("Edit #{}", channel.name))
                                .font(theme::TITLE)
                                .color(p.text_bright)
                                .grow(1.0),
                        )
                        .child(close_button(p)),
                )
                .child(
                    text("CHANNEL NAME")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(
                    text_input(&s.channel_name_edit, theme::BODY)
                        .key("channel-edit-name")
                        .w_full()
                        .h(40.0)
                        .radius(4.0)
                        .fill(p.rail)
                        .color(p.text)
                        .role("textbox")
                        .label("Channel name")
                        .placeholder("general", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.channel_name_edit),
                )
                .child(text("TOPIC").font(theme::SECTION).color(p.text_muted))
                .child(
                    text_input(&s.channel_topic_edit, theme::BODY)
                        .key("channel-edit-topic")
                        .w_full()
                        .h(40.0)
                        .radius(4.0)
                        .fill(p.rail)
                        .color(p.text)
                        .role("textbox")
                        .label("Channel topic")
                        .placeholder("What is this channel about?", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.channel_topic_edit),
                )
                .children((channel.kind == ChannelKind::Voice).then(|| {
                    col()
                        .w_full()
                        .gap(6.0)
                        .child(
                            text("CHANNEL STATUS")
                                .font(theme::SECTION)
                                .color(p.text_muted),
                        )
                        .child(
                            text_input(&s.channel_status_edit, theme::BODY)
                                .key("channel-edit-status")
                                .w_full()
                                .h(40.0)
                                .radius(4.0)
                                .fill(p.rail)
                                .color(p.text)
                                .role("textbox")
                                .label("Voice channel status")
                                .placeholder("Set a channel status", p.text_faint)
                                .padding(10.0)
                                .on_edit(|s: &mut App| &mut s.channel_status_edit),
                        )
                }))
                .children((channel.kind == ChannelKind::Forum).then(|| {
                    col()
                        .w_full()
                        .gap(6.0)
                        .child(
                            text("FORUM TAGS")
                                .font(theme::SECTION)
                                .color(p.text_muted),
                        )
                        .child(
                            text_input(&s.forum_tag_draft, theme::BODY)
                                .key("channel-edit-forum-tags")
                                .w_full()
                                .h(96.0)
                                .radius(4.0)
                                .fill(p.rail)
                                .color(p.text)
                                .role("textbox")
                                .label("Forum tags")
                                .placeholder("One tag per line, optionally with an emoji", p.text_faint)
                                .padding(10.0)
                                .on_edit(|s: &mut App| &mut s.forum_tag_draft),
                        )
                        .child(
                            row()
                                .w_full()
                                .gap(8.0)
                                .center_y()
                                .child(
                                    text("One tag per line. Start a line with an emoji to show it on the post.")
                                        .font(theme::TINY)
                                        .color(p.text_faint)
                                        .grow(1.0),
                                )
                                .child(
                                    el().key("channel-edit-save-tags")
                                        .h(30.0)
                                        .px(12.0)
                                        .radius(4.0)
                                        .center()
                                        .role("button")
                                        .label("Save forum tags")
                                        .fill(p.accent)
                                        .hover(p.accent_hover)
                                        .on_press(|s: &mut App| s.save_forum_tags())
                                        .child(
                                            text("Save Tags")
                                                .font(theme::TINY)
                                                .color(Color::WHITE),
                                        ),
                                ),
                        )
                }))
                .child(text("SLOWMODE").font(theme::SECTION).color(p.text_muted))
                .child(
                    row()
                        .w_full()
                        .gap(10.0)
                        .center_y()
                        .child(
                            text_input(&s.channel_slowmode_edit, theme::BODY)
                                .key("channel-edit-slowmode")
                                .w(140.0)
                                .h(38.0)
                                .radius(4.0)
                                .fill(p.rail)
                                .color(p.text)
                                .role("textbox")
                                .label("Slowmode seconds")
                                .placeholder("0", p.text_faint)
                                .padding(10.0)
                                .on_edit(|s: &mut App| &mut s.channel_slowmode_edit),
                        )
                        .child(
                            text("Seconds between messages (0-21600).")
                                .font(theme::TINY)
                                .color(p.text_faint)
                                .grow(1.0),
                        ),
                )
                .child(
                    row()
                        .key("channel-edit-nsfw")
                        .w_full()
                        .h(34.0)
                        .gap(10.0)
                        .center_y()
                        .role("checkbox")
                        .label("Age-restricted channel")
                        .checked(s.channel_nsfw_edit)
                        .on_press(|s: &mut App| s.channel_nsfw_edit = !s.channel_nsfw_edit)
                        .child(
                            el().size(16.0, 16.0)
                                .radius(3.0)
                                .fill(if s.channel_nsfw_edit {
                                    p.green
                                } else {
                                    p.input
                                })
                                .border(1.0, p.divider),
                        )
                        .child(
                            text("Age-restricted channel")
                                .font(theme::SMALL)
                                .color(p.text),
                        ),
                )
                .child(el().w_full().h(1.0).fill(p.divider))
                .child(
                    row()
                        .w_full()
                        .gap(8.0)
                        .child(
                            el().h(36.0)
                                .px(14.0)
                                .radius(4.0)
                                .key("channel-edit-delete")
                                .role("button")
                                .label("Delete channel")
                                .fill(p.rail)
                                .hover(p.danger.alpha(0.2))
                                .center()
                                .on_press(move |s: &mut App| s.confirm_delete_channel(channel_id))
                                .child(text("Delete").font(theme::SMALL_MED).color(p.danger)),
                        )
                        .child(el().grow(1.0))
                        .child(
                            el().h(36.0)
                                .px(16.0)
                                .radius(4.0)
                                .key("channel-edit-save")
                                .role("button")
                                .label("Save channel")
                                .fill(p.accent)
                                .hover(p.accent_hover)
                                .center()
                                .on_press(|s: &mut App| s.save_channel_edit())
                                .child(
                                    text("Save Changes")
                                        .font(theme::SMALL_MED)
                                        .color(Color::WHITE),
                                ),
                        ),
                ),
        )
}

pub(crate) fn onboarding_rules_sheet(s: &App, p: Palette, guild: Id) -> El {
    let onboarding = s
        .guild_detail(guild)
        .map(|detail| detail.onboarding.clone())
        .unwrap_or_default();
    let mut rules = col().w_full().gap(6.0);
    for (index, rule) in onboarding.rules.iter().enumerate() {
        rules = rules.child(
            row()
                .w_full()
                .gap(8.0)
                .items_start()
                .child(
                    text(format!("{}.", index + 1))
                        .font(theme::SMALL_MED)
                        .color(p.text_muted),
                )
                .child(
                    text(rule.clone())
                        .font(theme::SMALL)
                        .color(p.text)
                        .grow(1.0)
                        .wrap(),
                ),
        );
    }
    col()
        .key("onboarding-rules")
        .w(500.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .padding(20.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            text(if onboarding.welcome_title.is_empty() {
                "Welcome!".to_string()
            } else {
                onboarding.welcome_title.clone()
            })
            .font(theme::TITLE)
            .color(p.text_bright),
        )
        .children((!onboarding.welcome_description.is_empty()).then(|| {
            text(onboarding.welcome_description.clone())
                .font(theme::SMALL)
                .color(p.text_muted)
        }))
        .child(
            text("SERVER RULES")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(
            col()
                .w_full()
                .max_h(300.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(rules),
        )
        .child(
            el().key("onboarding-accept")
                .w_full()
                .h(38.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Accept the rules")
                .fill(p.green)
                .hover(p.green.alpha(0.85))
                .on_press(move |s: &mut App| s.accept_onboarding(guild))
                .child(
                    text("I have read and agree to the rules")
                        .font(theme::SMALL_MED)
                        .color(Color::WHITE),
                ),
        )
}

pub(crate) fn create_channel_sheet(s: &App, p: Palette) -> El {
    let choices: [(ChannelKind, &str, &str, &rune::Icon); 6] = [
        (
            ChannelKind::Text,
            "Text",
            "Send messages, images, GIFs, emoji and opinions.",
            &icons::HASH,
        ),
        (
            ChannelKind::Voice,
            "Voice",
            "Hang out together with voice, video and screen share.",
            &icons::VOICE,
        ),
        (
            ChannelKind::Stage,
            "Stage",
            "Host an audience with a handful of speakers.",
            &icons::VOICE,
        ),
        (
            ChannelKind::Announcement,
            "Announcement",
            "Post important updates for everyone.",
            &icons::MEGAPHONE,
        ),
        (
            ChannelKind::Forum,
            "Forum",
            "Organized, sortable conversations.",
            &icons::FORUM,
        ),
        (
            ChannelKind::Category,
            "Category",
            "Group channels together in the sidebar.",
            &icons::FOLDER,
        ),
    ];
    let mut types = col().w_full().gap(4.0);
    for (kind, label, detail, icon_id) in choices {
        let selected = s.create_channel_kind == kind;
        types = types.child(
            row()
                .key(match kind {
                    ChannelKind::Text => "channel-type-text",
                    ChannelKind::Voice => "channel-type-voice",
                    ChannelKind::Stage => "channel-type-stage",
                    ChannelKind::Announcement => "channel-type-announcement",
                    ChannelKind::Forum => "channel-type-forum",
                    ChannelKind::Category => "channel-type-category",
                    _ => "channel-type-other",
                })
                .w_full()
                .min_h(48.0)
                .px(10.0)
                .py(6.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .role("radio")
                .label(label)
                .checked(selected)
                .border(1.0, if selected { p.accent } else { p.divider })
                .fill(if selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.set_create_channel_kind(kind))
                .child(icon(icon_id, 20.0).color(p.text_muted))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(1.0)
                        .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
                        .child(
                            text(detail)
                                .font(theme::TINY)
                                .color(p.text_muted)
                                .truncate(),
                        ),
                )
                .children(selected.then(|| icon(&icons::CHECK, 16.0).color(p.accent))),
        );
    }
    modal_frame(
        "create-channel",
        "Create Channel",
        "Channels are where conversations happen. Pick a type and a name.",
        p,
        col()
            .w_full()
            .gap(10.0)
            .child(
                text("CHANNEL TYPE")
                    .font(theme::SECTION)
                    .color(p.text_muted),
            )
            .child(types)
            .child(el().w_full().h(1.0).fill(p.divider))
            .child(
                text("CHANNEL NAME")
                    .font(theme::SECTION)
                    .color(p.text_muted),
            )
            .child(modal_input(
                &s.profile_edit,
                "channel-name",
                "new-channel",
                p,
            )),
        ("Create Channel", |s: &mut App| {
            let kind = s.create_channel_kind;
            s.create_channel_now(kind)
        }),
        p,
    )
}

pub(crate) fn confirm_delete_channel_sheet(s: &App, p: Palette, id: Id) -> El {
    let name = s
        .channel(id)
        .map(|channel| channel.name.clone())
        .unwrap_or_default();
    modal_frame(
        "confirm-delete-channel",
        "Delete Channel",
        &format!("Delete #{name}? Its messages are removed for everyone. This cannot be undone.",),
        p,
        col().w_full().gap(10.0).child(
            text("This removes the channel and its messages for every member.")
                .font(theme::SMALL)
                .color(p.text_muted),
        ),
        ("Delete Channel", |s: &mut App| s.delete_channel_now()),
        p,
    )
}
