//! Modal sheets: server.

use super::super::prelude::*;
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

pub(crate) fn create_channel_sheet(s: &App, p: Palette) -> El {
    modal_frame(
        "create-channel",
        "Create Channel",
        "Channels are where conversations happen. Pick a name and start talking.",
        p,
        col()
            .w_full()
            .gap(8.0)
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
            ))
            .child(
                text("Text channels are the default in this local build.")
                    .font(theme::TINY)
                    .color(p.text_faint),
            ),
        ("Create Channel", |s: &mut App| {
            s.create_channel_now(ChannelKind::Text)
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
