//! Modals and sheets surface.

use super::*;
use crate::app::TimeoutUnit;

mod events;
mod inbox;
mod soundboard;
use crate::voice::{VideoQuality, ViewQuality};
use runecord_protocol::{NotificationLevel, PrivacyLevel, Role};

pub(crate) fn modal(s: &App, p: Palette) -> El {
    let sheet = match &s.modal {
        Modal::Settings(tab) => settings_sheet(s, p, *tab),
        Modal::ServerMenu => server_menu(s, p),
        Modal::CreateServer => create_server_sheet(s, p),
        Modal::CreateChannel => create_channel_sheet(s, p),
        Modal::EditChannel(_) => edit_channel_sheet(s, p),
        Modal::ConfirmDeleteChannel(id) => confirm_delete_channel_sheet(s, p, *id),
        Modal::ConfirmLeave(id) => confirm_leave_sheet(s, p, *id),
        Modal::ChannelNotifications(id) => channel_notifications_sheet(s, p, *id),
        Modal::Explore => explore_sheet(s, p),
        Modal::Privacy => privacy_sheet(s, p),
        Modal::ConfirmKick { guild, user } => confirm_kick_sheet(s, p, *guild, *user),
        Modal::ConfirmBan { guild, user } => confirm_ban_sheet(s, p, *guild, *user),
        Modal::TimeoutMember { guild, user } => timeout_sheet(s, p, *guild, *user),
        Modal::Bans => bans_sheet(s, p),
        Modal::PruneMembers => prune_sheet(s, p),
        Modal::VideoQuality => video_quality_sheet(s, p),
        Modal::CropGuildImage { banner } => crop_image_sheet(s, p, *banner, false, false),
        Modal::CropUserImage { banner } => crop_image_sheet(s, p, *banner, true, false),
        Modal::VoiceDevice { kind } => crate::ui::settings::voice_device_sheet(s, p, kind),
        Modal::Soundboard => soundboard::soundboard_sheet(s, p),
        Modal::CropDmIcon { channel } => {
            let _ = channel;
            crop_image_sheet(s, p, false, false, true)
        }
        Modal::DmSettings { channel } => dm_settings_sheet(s, p, *channel),
        Modal::GroupDm => group_dm_sheet(s, p),
        Modal::ProfileMore(id) => profile_more_sheet(s, p, *id),
        Modal::Invite => invite_sheet(s, p),
        Modal::AddFriend => add_friend_sheet(s, p),
        Modal::Search => search_sheet(s, p),
        Modal::Pins => pins_sheet(s, p),
        Modal::QuickSwitcher => quick_switcher(s, p),
        Modal::Profile(id) => profile_sheet(s, p, *id),
        Modal::FullProfile(id) => full_profile_sheet(s, p, *id),
        Modal::EmojiPicker => emoji_sheet(s, p, None),
        Modal::Reaction(id) => emoji_sheet(s, p, Some(*id)),
        Modal::Threads => threads_sheet(s, p),
        Modal::ThreadCreate { parent, message } => thread_create_sheet(s, p, *parent, *message),
        Modal::EditMessage(id) => edit_sheet(s, p, *id),
        Modal::MessageMenu(id) => message_menu(s, p, *id),
        Modal::ConfirmDelete(id) => simple_sheet(
            s,
            p,
            "Delete Message",
            &format!("Are you sure you want to delete message {id}?"),
        ),
        Modal::ConfirmLeave(_) => simple_sheet(
            s,
            p,
            "Leave Server",
            "Are you sure you want to leave this server?",
        ),
        Modal::InviteJoin => invite_join_sheet(s, p),
        Modal::BanAppeal => ban_appeal_sheet(s, p),
        Modal::ServerSettings => el(),
        Modal::Roles => el(),
        Modal::ServerNotifications => server_notifications_sheet(s, p),
        Modal::NotifyChannelPicker => notify_channel_picker_sheet(s, p),
        Modal::Inbox => inbox::inbox_sheet(s, p),
        Modal::Events => events::events_sheet(s, p),
        Modal::CreateEvent => events::create_event_sheet(s, p),
        Modal::ChannelPermissions(id) => channel_permissions_sheet(s, p, *id),
        Modal::None => return el(),
    };

    let overlay = el()
        .key("scrim")
        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
        .fill(p.scrim)
        .backdrop_blur(6.0);
    let overlay = if matches!(s.modal, Modal::Inbox) {
        overlay
            .justify_end()
            .items_start()
            .pt(56.0)
            .pr(16.0)
            .fill(Color::TRANSPARENT)
            .backdrop_blur(0.0)
    } else {
        overlay.center()
    };
    overlay
        .role("dialog")
        .label(modal_title(&s.modal))
        .child(sheet)
}

fn modal_title(modal: &Modal) -> &'static str {
    match modal {
        Modal::Settings(_) => "Settings",
        Modal::ServerMenu => "Server menu",
        Modal::CreateServer => "Create server",
        Modal::CreateChannel => "Create channel",
        Modal::EditChannel(_) => "Edit channel",
        Modal::ConfirmDeleteChannel(_) => "Delete channel",
        Modal::ConfirmLeave(_) => "Leave server",
        Modal::ChannelNotifications(_) => "Notification settings",
        Modal::ServerNotifications => "Server notification settings",
        Modal::NotifyChannelPicker => "Add notification override",
        Modal::Inbox => "Inbox",
        Modal::Events => "Events",
        Modal::CreateEvent => "Create event",
        Modal::Explore => "Explore public servers",
        Modal::Privacy => "Privacy settings",
        Modal::ConfirmKick { .. } => "Kick member",
        Modal::ConfirmBan { .. } => "Ban member",
        Modal::TimeoutMember { .. } => "Timeout member",
        Modal::Bans => "Banned users",
        Modal::PruneMembers => "Prune members",
        Modal::VideoQuality => "Video quality",
        Modal::CropGuildImage { .. } => "Crop server image",
        Modal::CropUserImage { .. } => "Crop profile image",
        Modal::VoiceDevice { .. } => "Voice device",
        Modal::Soundboard => "Soundboard",
        Modal::CropDmIcon { .. } => "Crop group icon",
        Modal::DmSettings { .. } => "Group DM settings",
        Modal::GroupDm => "Create group DM",
        Modal::ProfileMore(_) => "Profile actions",
        Modal::Invite => "Invite people",
        Modal::AddFriend => "Add friend",
        Modal::Search => "Search",
        Modal::Pins => "Pinned messages",
        Modal::QuickSwitcher => "Quick switcher",
        Modal::Profile(_) => "Profile",
        Modal::FullProfile(_) => "Full profile",
        Modal::EmojiPicker => "Emoji picker",
        Modal::Reaction(_) => "Add reaction",
        Modal::Threads => "Threads",
        Modal::ThreadCreate { .. } => "Create thread",
        Modal::EditMessage(_) => "Edit message",
        Modal::MessageMenu(_) => "Message actions",
        Modal::ConfirmDelete(_) => "Delete message",
        Modal::ConfirmLeave(_) => "Leave server",
        Modal::InviteJoin => "Join server",
        Modal::BanAppeal => "Ban appeal",
        Modal::ServerSettings | Modal::Roles => "Roles",
        Modal::ChannelPermissions(_) => "Channel permissions",
        Modal::None => "Dialog",
    }
}

pub(crate) fn simple_sheet(s: &App, p: Palette, title: &str, body: &str) -> El {
    let _ = s;
    col()
        .key("simple-sheet")
        .w(440.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .h(56.0)
                .px(20.0)
                .items_center()
                .child(
                    text(title.to_string())
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            col()
                .w_full()
                .px(20.0)
                .pb(20.0)
                .gap(12.0)
                .child(
                    text(body.to_string())
                        .font(theme::SMALL)
                        .color(p.text_muted),
                )
                .child(
                    el().h(38.0)
                        .px(16.0)
                        .radius(4.0)
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .center()
                        .on_press(|s: &mut App| s.modal = Modal::None)
                        .child(text("Done").font(theme::SMALL_MED).color(Color::WHITE)),
                ),
        )
}

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

pub(crate) fn server_notifications_sheet(s: &App, p: Palette) -> El {
    let guild = s.active_guild().cloned().unwrap_or_default();
    let settings = s.guild_notification(guild.id);
    let switch_row = |label: &'static str,
                      detail: Option<&'static str>,
                      on: bool,
                      key: &'static str,
                      toggle: fn(&mut App),
                      p: Palette| {
        row()
            .key(key)
            .w_full()
            .min_h(if detail.is_some() { 46.0 } else { 36.0 })
            .py(2.0)
            .gap(12.0)
            .center_y()
            .role("switch")
            .label(label)
            .checked(on)
            .hover(p.card_hover)
            .on_press(toggle)
            .child(
                col()
                    .grow(1.0)
                    .min_w(0.0)
                    .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
                    .children(detail.map(|detail| {
                        text(detail)
                            .font(theme::TINY)
                            .color(p.text_muted)
                            .max_lines(2)
                    })),
            )
            .child(
                el().size(42.0, 24.0)
                    .radius(12.0)
                    .fill(if on { p.green } else { p.text_faint })
                    .child(
                        el().size(18.0, 18.0)
                            .radius(9.0)
                            .fill(Color::WHITE)
                            .ml(if on { 21.0 } else { 3.0 })
                            .mt(3.0)
                            .center()
                            .children(on.then(|| icon(&icons::CHECK, 12.0).color(p.green))),
                    ),
            )
    };
    let radio = |level: NotificationLevel, label: &'static str, key: &'static str| {
        let selected = settings.level == level;
        row()
            .key(key)
            .w_full()
            .h(36.0)
            .px(8.0)
            .radius(4.0)
            .gap(10.0)
            .center_y()
            .role("radio")
            .label(label)
            .checked(selected)
            .hover(p.card_hover)
            .on_press(move |s: &mut App| s.set_active_guild_notify_level(level))
            .child(
                el().size(18.0, 18.0)
                    .radius(9.0)
                    .border(
                        2.0,
                        if selected {
                            p.text_bright
                        } else {
                            p.text_muted
                        },
                    )
                    .center()
                    .children(
                        selected.then(|| el().size(8.0, 8.0).radius(4.0).fill(p.text_bright)),
                    ),
            )
            .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
    };

    let mut overrides = col().w_full().gap(2.0);
    let mut override_count = 0;
    for channel in s.channels.get(&guild.id).cloned().unwrap_or_default() {
        if !matches!(
            channel.kind,
            ChannelKind::Text | ChannelKind::Voice | ChannelKind::Announcement | ChannelKind::Forum
        ) {
            continue;
        }
        if !s.channel_notify.contains_key(&channel.id) && !s.channel_suppress.contains(&channel.id)
        {
            continue;
        }
        override_count += 1;
        let level = s
            .channel_notify
            .get(&channel.id)
            .copied()
            .unwrap_or(NotificationLevel::All);
        let channel_id = channel.id;
        let level_label = match level {
            NotificationLevel::All => "All Messages",
            NotificationLevel::Mentions => "Only @mentions",
            NotificationLevel::None => "Nothing",
        };
        let channel_icon = match channel.kind {
            ChannelKind::Voice => &icons::VOICE,
            ChannelKind::Forum => &icons::FORUM,
            ChannelKind::Announcement => &icons::MEGAPHONE,
            _ => &icons::HASH,
        };
        overrides = overrides.child(
            row()
                .key(format!("guild-notify-override-{channel_id}"))
                .w_full()
                .h(34.0)
                .px(6.0)
                .radius(4.0)
                .gap(8.0)
                .center_y()
                .hover(p.card_hover)
                .child(icon(channel_icon, 16.0).color(p.text_muted))
                .child(
                    text(channel.name.clone())
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .grow(1.0)
                        .truncate(),
                )
                .child(text(level_label).font(theme::TINY).color(p.text_muted))
                .child(
                    el().key(format!("guild-notify-remove-{channel_id}"))
                        .size(26.0, 26.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Remove override for {}", channel.name))
                        .hover(p.danger.alpha(0.2))
                        .on_press(move |s: &mut App| s.clear_channel_notify(channel_id))
                        .child(icon(&icons::CLOSE, 12.0).color(p.danger)),
                ),
        );
    }
    if override_count == 0 {
        overrides = overrides.child(
            row().w_full().h(52.0).center().child(
                text("Add a channel to override its default notification settings")
                    .font(theme::TINY)
                    .color(p.text_muted),
            ),
        );
    }

    col()
        .key("server-notifications")
        .w(480.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(16.0)
        .gap(4.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text("Notification Settings")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text(guild.name.clone())
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(switch_row(
            "Mute Server",
            Some("Muting a server prevents unread indicators and notifications from appearing unless you're mentioned."),
            settings.muted,
            "guild-notify-mute",
            App::toggle_active_guild_mute,
            p,
        ))
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            text("SERVER NOTIFICATION SETTINGS")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(radio(NotificationLevel::All, "All Messages", "guild-notify-all"))
        .child(radio(
            NotificationLevel::Mentions,
            "Only @mentions",
            "guild-notify-mentions",
        ))
        .child(radio(
            NotificationLevel::None,
            "Nothing",
            "guild-notify-nothing",
        ))
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(switch_row(
            "Suppress @everyone and @here",
            None,
            settings.suppress_everyone,
            "guild-notify-suppress-everyone",
            App::toggle_guild_suppress_everyone,
            p,
        ))
        .child(switch_row(
            "Suppress All Role @mentions",
            None,
            settings.suppress_roles,
            "guild-notify-suppress-roles",
            App::toggle_guild_suppress_roles,
            p,
        ))
        .child(switch_row(
            "Suppress Highlights",
            Some("Highlights provide occasional updates when your friends are chatting in busy servers, and more."),
            settings.suppress_highlights,
            "guild-notify-suppress-highlights",
            App::toggle_guild_suppress_highlights,
            p,
        ))
        .child(
            text("Learn more about Highlights")
                .font(theme::TINY)
                .color(p.text_link)
                .on_press(|s: &mut App| s.toast("Highlights are not available in this build")),
        )
        .child(switch_row(
            "Mute New Events",
            None,
            settings.mute_new_events,
            "guild-notify-mute-events",
            App::toggle_guild_mute_new_events,
            p,
        ))
        .child(switch_row(
            "Mobile Push Notifications",
            None,
            settings.mobile_push,
            "guild-notify-mobile-push",
            App::toggle_guild_mobile_push,
            p,
        ))
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            text("NOTIFICATION OVERRIDES")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(
            text("Add a channel to override its default notification settings")
                .font(theme::TINY)
                .color(p.text_muted),
        )
        .child(
            row()
                .w_full()
                .h(24.0)
                .px(6.0)
                .gap(8.0)
                .center_y()
                .child(
                    text("CHANNEL OR CATEGORY")
                        .font(theme::TINY)
                        .color(p.text_faint)
                        .grow(1.0),
                )
                .child(text("OVERRIDE").font(theme::TINY).color(p.text_faint)),
        )
        .child(
            el().key("guild-notify-add")
                .w_full()
                .h(34.0)
                .px(10.0)
                .radius(4.0)
                .center_y()
                .role("button")
                .label("Select a channel or category")
                .fill(p.input)
                .hover(p.card_hover)
                .on_press(|s: &mut App| s.modal = Modal::NotifyChannelPicker)
                .child(
                    text("Select a channel or category...")
                        .font(theme::SMALL)
                        .color(p.text_muted)
                        .grow(1.0),
                )
                .child(icon(&icons::CHEVRON_DOWN, 14.0).color(p.text_muted)),
        )
        .child(
            col()
                .w_full()
                .max_h(140.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(overrides),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            row()
                .w_full()
                .justify_end()
                .child(
                    el().key("guild-notify-done")
                        .h(34.0)
                        .px(18.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Done")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(|s: &mut App| s.modal = Modal::None)
                        .child(text("Done").font(theme::SMALL_MED).color(Color::WHITE)),
                ),
        )
}

pub(crate) fn notify_channel_picker_sheet(s: &App, p: Palette) -> El {
    let guild = s.active_guild().cloned().unwrap_or_default();
    let mut rows = col().w_full().gap(2.0);
    let mut count = 0;
    for channel in s.channels.get(&guild.id).cloned().unwrap_or_default() {
        if !matches!(
            channel.kind,
            ChannelKind::Text | ChannelKind::Voice | ChannelKind::Announcement | ChannelKind::Forum
        ) {
            continue;
        }
        if s.channel_notify.contains_key(&channel.id) || s.channel_suppress.contains(&channel.id) {
            continue;
        }
        count += 1;
        let channel_id = channel.id;
        let channel_icon = match channel.kind {
            ChannelKind::Voice => &icons::VOICE,
            ChannelKind::Forum => &icons::FORUM,
            ChannelKind::Announcement => &icons::MEGAPHONE,
            _ => &icons::HASH,
        };
        rows = rows.child(
            row()
                .key(format!("guild-notify-pick-{channel_id}"))
                .w_full()
                .h(40.0)
                .px(8.0)
                .radius(4.0)
                .gap(8.0)
                .center_y()
                .role("button")
                .label(format!("Add override for {}", channel.name))
                .hover(p.card_hover)
                .on_press(move |s: &mut App| {
                    s.add_guild_notification_override(guild.id, channel_id)
                })
                .child(icon(channel_icon, 16.0).color(p.text_muted))
                .child(
                    text(channel.name.clone())
                        .font(theme::SMALL)
                        .color(p.text_bright),
                ),
        );
    }
    if count == 0 {
        rows = rows.child(
            text("Every channel already has an override.")
                .font(theme::SMALL)
                .color(p.text_muted)
                .padding(8.0),
        );
    }
    col()
        .key("notify-channel-picker")
        .w(420.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(18.0)
        .gap(10.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::ServerNotifications)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text("Add Notification Override")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            col()
                .w_full()
                .max_h(420.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(rows),
        )
}

pub(crate) fn channel_notifications_sheet(s: &App, p: Palette, channel: Id) -> El {
    let name = s
        .channel(channel)
        .map(|channel| channel.name.clone())
        .unwrap_or_default();
    let current = s.channel_notify.get(&channel).copied().unwrap_or_default();
    let suppress = s.channel_suppress.contains(&channel);
    let options = [
        (
            NotificationLevel::All,
            "All Messages",
            "Get notified for every message in this channel.",
        ),
        (
            NotificationLevel::Mentions,
            "Only @mentions",
            "Only notify me when someone mentions me.",
        ),
        (
            NotificationLevel::None,
            "Nothing",
            "Mute this channel completely.",
        ),
    ];
    let mut rows = col().w_full().gap(2.0);
    for (level, label, description) in options {
        let selected = current == level;
        rows = rows.child(
            row()
                .key(format!(
                    "notify-{}",
                    match level {
                        NotificationLevel::All => "all",
                        NotificationLevel::Mentions => "mentions",
                        NotificationLevel::None => "none",
                    }
                ))
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
                .fill(if selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.set_channel_notify(channel, level))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
                        .child(text(description).font(theme::TINY).color(p.text_muted)),
                )
                .children(selected.then(|| icon(&icons::CHECK, 18.0).color(p.green))),
        );
    }
    col()
        .key("channel-notifications")
        .w(400.0)
        .radius(8.0)
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
                    text("Notification Settings")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text(format!("#{name}"))
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(rows)
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            row()
                .key("notify-suppress")
                .w_full()
                .min_h(48.0)
                .px(10.0)
                .py(6.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .role("switch")
                .label("Suppress @everyone and @here")
                .checked(suppress)
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.set_channel_suppress(channel, !suppress))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(
                            text("Suppress @everyone and @here")
                                .font(theme::SMALL_MED)
                                .color(p.text_bright),
                        )
                        .child(
                            text("You will only be notified for direct mentions.")
                                .font(theme::TINY)
                                .color(p.text_muted),
                        ),
                )
                .child(
                    el().size(40.0, 22.0)
                        .radius(11.0)
                        .fill(if suppress { p.green } else { p.input })
                        .child(
                            el().size(18.0, 18.0)
                                .radius(9.0)
                                .fill(Color::WHITE)
                                .ml(if suppress { 19.0 } else { 2.0 })
                                .mt(2.0),
                        ),
                ),
        )
        .child(
            row().w_full().justify_end().child(
                el().key("notify-done")
                    .h(34.0)
                    .px(16.0)
                    .radius(4.0)
                    .role("button")
                    .label("Done")
                    .fill(p.accent)
                    .hover(p.accent_hover)
                    .center()
                    .on_press(|s: &mut App| s.modal = Modal::None)
                    .child(text("Done").font(theme::SMALL_MED).color(Color::WHITE)),
            ),
        )
}

pub(crate) fn explore_sheet(s: &App, p: Palette) -> El {
    let mut list = col().w_full().gap(4.0);
    for guild in &s.public_guilds {
        let id = guild.id;
        let initials: String = guild
            .name
            .split_whitespace()
            .filter_map(|word| word.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase();
        list = list.child(
            row()
                .key(format!("public-{}", id))
                .w_full()
                .min_h(72.0)
                .px(12.0)
                .py(10.0)
                .radius(6.0)
                .gap(12.0)
                .center_y()
                .hover(p.card_hover)
                .child(
                    el().size(44.0, 44.0)
                        .radius(14.0)
                        .fill(theme::color(&format!("#{:06x}", guild.icon_color)))
                        .center()
                        .child(text(initials).font(theme::BODY_BOLD).color(Color::WHITE)),
                )
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(
                            text(guild.name.clone())
                                .font(theme::BODY_BOLD)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(
                            text(if guild.description.is_empty() {
                                "A public Rune server.".to_string()
                            } else {
                                guild.description.clone()
                            })
                            .font(theme::SMALL)
                            .color(p.text_muted)
                            .max_lines(2),
                        )
                        .child(
                            text(format!(
                                "{} member{}",
                                guild.member_count,
                                if guild.member_count == 1 { "" } else { "s" }
                            ))
                            .font(theme::TINY)
                            .color(p.text_faint),
                        ),
                )
                .child(
                    el().key(format!("join-{}", id))
                        .h(32.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Join {}", guild.name))
                        .fill(p.green)
                        .hover(p.green.alpha(0.85))
                        .on_press(move |s: &mut App| s.join_public_guild(id))
                        .child(
                            text("Join Server")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        ),
                ),
        );
    }
    if s.explore_loading {
        list = list.child(
            col().w_full().py(48.0).items_center().child(
                text("Loading public servers…")
                    .font(theme::SMALL)
                    .color(p.text_faint),
            ),
        );
    } else if s.public_guilds.is_empty() {
        list = list.child(
            col()
                .w_full()
                .py(48.0)
                .items_center()
                .gap(8.0)
                .child(icon(&icons::COMPASS, 48.0).color(p.text_faint))
                .child(
                    text("No public servers to join")
                        .font(theme::BODY)
                        .color(p.text_muted),
                )
                .child(
                    text("Create a server, or ask a friend for an invite.")
                        .font(theme::SMALL)
                        .color(p.text_faint),
                ),
        );
    }
    col()
        .key("explore")
        .w(680.0)
        .h(540.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .px(20.0)
                .pt(20.0)
                .pb(12.0)
                .items_center()
                .child(
                    text("Explore Public Servers")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Find a community to join. Every server here is local to this machine.")
                .font(theme::SMALL)
                .color(p.text_muted)
                .px(20.0)
                .pb(12.0),
        )
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .px(8.0)
                .pb(12.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list),
        )
}

pub(crate) fn privacy_sheet(s: &App, p: Palette) -> El {
    let mut dm_rows = col().w_full().gap(2.0);
    let mut friend_rows = col().w_full().gap(2.0);
    let options = [
        (
            PrivacyLevel::Everyone,
            "everyone",
            "Everyone",
            "Anyone can send you a direct message.",
        ),
        (
            PrivacyLevel::ServerMembers,
            "server-members",
            "Server Members",
            "Only people who share a server with you.",
        ),
        (
            PrivacyLevel::Nobody,
            "nobody",
            "Nobody",
            "No one can start a new conversation.",
        ),
    ];
    for (level, key, label, description) in options {
        let dm_selected = s.me.dm_privacy == level;
        dm_rows = dm_rows.child(
            row()
                .key(format!("privacy-dm-{key}"))
                .w_full()
                .min_h(48.0)
                .px(10.0)
                .py(6.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .role("radio")
                .label(label)
                .checked(dm_selected)
                .fill(if dm_selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.set_privacy(level, s.me.friend_privacy))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
                        .child(text(description).font(theme::TINY).color(p.text_muted)),
                )
                .children(dm_selected.then(|| icon(&icons::CHECK, 18.0).color(p.green))),
        );

        let friend_selected = s.me.friend_privacy == level;
        let friend_description = match level {
            PrivacyLevel::Everyone => "Anyone can send you a friend request.",
            PrivacyLevel::ServerMembers => "Only people who share a server with you.",
            PrivacyLevel::Nobody => "No one can send you a friend request.",
        };
        friend_rows = friend_rows.child(
            row()
                .key(format!("privacy-friend-{key}"))
                .w_full()
                .min_h(48.0)
                .px(10.0)
                .py(6.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .role("radio")
                .label(label)
                .checked(friend_selected)
                .fill(if friend_selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.set_privacy(s.me.dm_privacy, level))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
                        .child(
                            text(friend_description)
                                .font(theme::TINY)
                                .color(p.text_muted),
                        ),
                )
                .children(friend_selected.then(|| icon(&icons::CHECK, 18.0).color(p.green))),
        );
    }

    let mut body = col()
        .w_full()
        .gap(10.0)
        .child(
            text("DIRECT MESSAGES")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(dm_rows)
        .child(
            text("FRIEND REQUESTS")
                .font(theme::SECTION)
                .color(p.text_muted)
                .pt(10.0),
        )
        .child(friend_rows);

    if s.guild_can(perm::MANAGE_GUILD) {
        let discoverable = s
            .active_guild()
            .map(|guild| guild.discoverable)
            .unwrap_or(false);
        body = body
            .child(
                text("SERVER DISCOVERY")
                    .font(theme::SECTION)
                    .color(p.text_muted)
                    .pt(10.0),
            )
            .child(
                row()
                    .key("privacy-discovery")
                    .w_full()
                    .min_h(48.0)
                    .px(10.0)
                    .py(6.0)
                    .radius(4.0)
                    .gap(10.0)
                    .center_y()
                    .role("switch")
                    .label("Show server in Explore")
                    .checked(discoverable)
                    .hover(p.card_hover)
                    .on_press(|s: &mut App| s.toggle_guild_discovery())
                    .child(
                        col()
                            .grow(1.0)
                            .min_w(0.0)
                            .child(
                                text("Show in Explore Public Servers")
                                    .font(theme::SMALL_MED)
                                    .color(p.text_bright),
                            )
                            .child(
                                text("People who are not members can find and join this server.")
                                    .font(theme::TINY)
                                    .color(p.text_muted),
                            ),
                    )
                    .child(
                        el().size(40.0, 22.0)
                            .radius(11.0)
                            .fill(if discoverable { p.green } else { p.input })
                            .child(
                                el().size(18.0, 18.0)
                                    .radius(9.0)
                                    .fill(Color::WHITE)
                                    .ml(if discoverable { 19.0 } else { 2.0 })
                                    .mt(2.0),
                            ),
                    ),
            );
    }

    col()
        .key("privacy")
        .w(520.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(18.0)
        .gap(10.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text("Privacy Settings")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Choose who can reach you and whether this server appears in Explore.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            col()
                .w_full()
                .max_h(560.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(body),
        )
        .child(
            row().w_full().justify_end().pt(6.0).child(
                el().key("privacy-done")
                    .h(34.0)
                    .px(16.0)
                    .radius(4.0)
                    .role("button")
                    .label("Done")
                    .fill(p.accent)
                    .hover(p.accent_hover)
                    .center()
                    .on_press(|s: &mut App| s.modal = Modal::None)
                    .child(text("Done").font(theme::SMALL_MED).color(Color::WHITE)),
            ),
        )
}

pub(crate) fn pins_sheet(s: &App, p: Palette) -> El {
    let channel = match s.screen {
        Screen::Guild => s.active_channel,
        Screen::Dms => s.active_dm,
    };
    let channel_name = s
        .channel(channel)
        .map(|channel| channel.name.clone())
        .unwrap_or_default();
    let mut list = col().w_full().gap(4.0);
    for message in &s.pinned {
        let id = message.id;
        let author = s.user(message.author_id).cloned().unwrap_or_default();
        let author_name = author.display_name.clone();
        let body = message.preview();
        let time = time_label(message.timestamp);
        list = list.child(
            col()
                .key(format!("pinned-{}", id))
                .w_full()
                .radius(6.0)
                .padding(10.0)
                .gap(6.0)
                .fill(p.card)
                .child(
                    row()
                        .gap(8.0)
                        .center_y()
                        .child(avatar(&author, 24.0, p))
                        .child(
                            text(author_name)
                                .font(theme::SMALL_MED)
                                .color(p.text_bright),
                        )
                        .child(
                            text(format!("#{channel_name} · {time}"))
                                .font(theme::TINY)
                                .color(p.text_faint)
                                .grow(1.0)
                                .truncate(),
                        ),
                )
                .child(text(body).font(theme::SMALL).color(p.text).max_lines(3))
                .child(
                    row()
                        .w_full()
                        .gap(6.0)
                        .justify_end()
                        .child(
                            el().key(format!("pin-jump-{}", id))
                                .h(26.0)
                                .px(10.0)
                                .radius(4.0)
                                .center()
                                .role("button")
                                .label("Jump to message")
                                .fill(p.rail)
                                .hover(p.rail_hover)
                                .on_press_tx(move |s: &mut App, tx: &mut rune::Tx| {
                                    s.jump_to_pinned(id, channel);
                                    tx.scroll_into_view(format!("msg-{}", id));
                                })
                                .child(text("Jump").font(theme::LABEL).color(p.text_bright)),
                        )
                        .child(
                            el().key(format!("pin-remove-{}", id))
                                .h(26.0)
                                .px(10.0)
                                .radius(4.0)
                                .center()
                                .role("button")
                                .label("Unpin message")
                                .fill(p.rail)
                                .hover(p.danger.alpha(0.2))
                                .on_press(move |s: &mut App| s.toggle_pin(id))
                                .child(text("Unpin").font(theme::LABEL).color(p.danger)),
                        ),
                ),
        );
    }
    if s.pinned.is_empty() {
        list = list.child(
            col()
                .w_full()
                .py(48.0)
                .items_center()
                .gap(8.0)
                .child(icon(&icons::PIN, 40.0).color(p.text_faint))
                .child(
                    text("No pinned messages in this channel yet.")
                        .font(theme::SMALL)
                        .color(p.text_muted),
                )
                .child(
                    text("Right-click a message and choose Pin Message.")
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        );
    }
    col()
        .key("pins")
        .w(540.0)
        .h(560.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .px(20.0)
                .pt(20.0)
                .pb(12.0)
                .items_center()
                .child(
                    text("Pinned Messages")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text(format!("#{channel_name}"))
                .font(theme::SMALL)
                .color(p.text_muted)
                .px(20.0)
                .pb(12.0),
        )
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .px(12.0)
                .pb(12.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list),
        )
}

pub(crate) fn confirm_kick_sheet(s: &App, p: Palette, _guild: Id, user: Id) -> El {
    let name = s
        .user(user)
        .map(|user| user.display_name.clone())
        .unwrap_or_default();
    modal_frame(
        "confirm-kick",
        &format!("Kick {name}"),
        "Are you sure you want to kick this member? They can be invited back later.",
        p,
        col().w_full().gap(8.0).child(
            text(format!(
                "{name} will lose access to this server and its channels."
            ))
            .font(theme::SMALL)
            .color(p.text_muted),
        ),
        ("Kick", |s: &mut App| s.kick_now()),
        p,
    )
}

pub(crate) fn confirm_ban_sheet(s: &App, p: Palette, _guild: Id, user: Id) -> El {
    let name = s
        .user(user)
        .map(|user| user.display_name.clone())
        .unwrap_or_default();
    modal_frame(
        "confirm-ban",
        &format!("Ban {name}"),
        "Banning removes the member and blocks them from rejoining until they are unbanned.",
        p,
        col()
            .w_full()
            .gap(8.0)
            .child(text("BAN REASON").font(theme::SECTION).color(p.text_muted))
            .child(
                text_input(&s.ban_reason, theme::SMALL)
                    .key("ban-reason")
                    .w_full()
                    .h(38.0)
                    .radius(4.0)
                    .fill(p.rail)
                    .color(p.text)
                    .role("textbox")
                    .label("Reason (optional)")
                    .placeholder("Reason (optional)", p.text_faint)
                    .padding(10.0)
                    .on_edit(|s: &mut App| &mut s.ban_reason),
            )
            .child(
                text(format!(
                    "{name} will be removed from the server immediately."
                ))
                .font(theme::TINY)
                .color(p.text_faint),
            ),
        ("Ban", |s: &mut App| s.ban_now()),
        p,
    )
}

pub(crate) fn timeout_sheet(s: &App, p: Palette, guild: Id, user: Id) -> El {
    let name = s
        .user(user)
        .map(|user| user.display_name.clone())
        .unwrap_or_default();
    let until = s
        .member(guild, user)
        .map(|member| member.timeout_until)
        .unwrap_or(0);
    let active = until > now_ms();
    let entered = s.timeout_amount.text.trim().parse::<u64>().ok();
    let seconds =
        entered.map(|value| value.min(s.timeout_unit.max_value()) * s.timeout_unit.seconds());
    let expiry = seconds
        .filter(|seconds| *seconds > 0)
        .map(|seconds| expiry_label(now_ms() + seconds as i64 * 1000));
    let hint = match (entered, expiry) {
        (None, _) => "Enter a whole number.".to_string(),
        (Some(0), _) => "Use at least one unit.".to_string(),
        (Some(value), Some(expiry)) => {
            let max = s.timeout_unit.max_value();
            if value > max {
                format!(
                    "Clamped to {max} {}. Expires {expiry}.",
                    s.timeout_unit.label().to_lowercase()
                )
            } else {
                format!("Expires {expiry}.")
            }
        }
        _ => String::new(),
    };

    let mut units = row().w_full().gap(6.0);
    for unit in [TimeoutUnit::Minutes, TimeoutUnit::Hours, TimeoutUnit::Days] {
        let selected = s.timeout_unit == unit;
        units = units.child(
            el().key(format!("timeout-unit-{}", unit.label().to_lowercase()))
                .h(34.0)
                .px(12.0)
                .radius(4.0)
                .center()
                .role("radio")
                .label(unit.label())
                .checked(selected)
                .fill(if selected { p.selected } else { p.rail })
                .hover(p.rail_hover)
                .on_press(move |s: &mut App| s.set_timeout_unit(unit))
                .child(
                    text(unit.label())
                        .font(theme::SMALL_MED)
                        .color(if selected {
                            p.text_bright
                        } else {
                            p.text_muted
                        }),
                ),
        );
    }

    let mut footer = row().w_full().gap(8.0).justify_end().pt(4.0);
    if active {
        footer = footer.child(
            el().key("timeout-clear")
                .h(36.0)
                .px(14.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Remove timeout")
                .fill(p.rail)
                .hover(p.danger.alpha(0.2))
                .on_press(|s: &mut App| s.timeout_now(0))
                .child(
                    text("Remove Timeout")
                        .font(theme::SMALL_MED)
                        .color(p.danger),
                ),
        );
    }
    footer = footer.child(
        el().key("timeout-apply")
            .h(36.0)
            .px(16.0)
            .radius(4.0)
            .center()
            .role("button")
            .label("Apply timeout")
            .fill(p.accent)
            .hover(p.accent_hover)
            .on_press(|s: &mut App| s.apply_timeout())
            .child(
                text("Apply Timeout")
                    .font(theme::SMALL_MED)
                    .color(Color::WHITE),
            ),
    );

    col()
        .key("timeout-sheet")
        .w(430.0)
        .radius(8.0)
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
                    text(format!("Timeout {name}"))
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text(if active {
                format!(
                    "Currently timed out until {}. Choose a new duration to replace it.",
                    expiry_label(until)
                )
            } else {
                "Timed-out members cannot send messages or join voice.".to_string()
            })
            .font(theme::SMALL)
            .color(p.text_muted),
        )
        .child(text("DURATION").font(theme::SECTION).color(p.text_muted))
        .child(
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .child(
                    text_input(&s.timeout_amount, theme::BODY)
                        .key("timeout-amount")
                        .w(96.0)
                        .h(38.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Timeout amount")
                        .placeholder("10", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.timeout_amount),
                )
                .child(units),
        )
        .child(
            row()
                .w_full()
                .justify_between()
                .child(text(hint).font(theme::TINY).color(if entered.is_none() {
                    p.danger
                } else {
                    p.text_faint
                }))
                .child(
                    text("Maximum 28 days")
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        )
        .child(footer)
}

pub(crate) fn bans_list(s: &App, p: Palette) -> El {
    let mut list = col().w_full().gap(4.0);
    for ban in &s.bans {
        let user = &ban.user;
        let id = user.id;
        let name = user.display_name.clone();
        let reason = if ban.reason.trim().is_empty() {
            "No reason given".to_string()
        } else {
            ban.reason.clone()
        };
        let actor = s.user_name(ban.actor_id);
        list = list.child(
            row()
                .key(format!("ban-{}", id))
                .w_full()
                .min_h(64.0)
                .px(12.0)
                .py(8.0)
                .radius(6.0)
                .gap(10.0)
                .center_y()
                .fill(p.card)
                .child(avatar(user, 36.0, p))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(
                            text(name.clone())
                                .font(theme::SMALL_MED)
                                .color(p.text_bright)
                                .truncate(),
                        )
                        .child(
                            text(reason)
                                .font(theme::TINY)
                                .color(p.text_muted)
                                .max_lines(2),
                        )
                        .child(
                            text(format!(
                                "Banned by {actor} · {}",
                                date_label(ban.created_at)
                            ))
                            .font(theme::TINY)
                            .color(p.text_faint),
                        ),
                )
                .child(
                    el().key(format!("unban-{}", id))
                        .h(30.0)
                        .px(12.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Unban {name}"))
                        .fill(p.rail)
                        .hover(p.danger.alpha(0.2))
                        .on_press(move |s: &mut App| s.unban(id))
                        .child(text("Unban").font(theme::LABEL).color(p.danger)),
                ),
        );
    }
    if s.bans.is_empty() {
        list = list.child(
            col()
                .w_full()
                .py(48.0)
                .items_center()
                .gap(8.0)
                .child(icon(&icons::SHIELD, 40.0).color(p.text_faint))
                .child(
                    text("No banned users")
                        .font(theme::BODY)
                        .color(p.text_muted),
                )
                .child(
                    text("Ban a member from their right-click menu.")
                        .font(theme::SMALL)
                        .color(p.text_faint),
                ),
        );
    }
    list
}

pub(crate) fn bans_sheet(s: &App, p: Palette) -> El {
    col()
        .key("bans")
        .w(520.0)
        .h(520.0)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.modal)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .px(20.0)
                .pt(20.0)
                .pb(12.0)
                .items_center()
                .child(
                    text("Banned Users")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Banned members cannot rejoin until they are unbanned.")
                .font(theme::SMALL)
                .color(p.text_muted)
                .px(20.0)
                .pb(12.0),
        )
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .px(12.0)
                .pb(12.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(appeals_list(s, p))
                .child(bans_list(s, p)),
        )
}

pub(crate) fn profile_more_sheet(s: &App, p: Palette, id: Id) -> El {
    let relationship = s
        .relationships
        .iter()
        .find(|relationship| relationship.user.id == id)
        .map(|relationship| relationship.kind);
    let item = |label: &'static str, key: &'static str, red: bool, p: Palette| {
        el().key(key)
            .w_full()
            .h(34.0)
            .px(8.0)
            .radius(4.0)
            .center_y()
            .role("button")
            .label(label)
            .hover(if red {
                p.danger.alpha(0.18)
            } else {
                p.card_hover
            })
            .child(text(label).font(theme::SMALL_MED).color(if red {
                p.danger
            } else {
                p.text_bright
            }))
    };
    let mut list = col()
        .key("profile-more")
        .w(240.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(6.0)
        .gap(1.0)
        .shadow(8.0, 24.0, p.shadow)
        .modal(true, move |s: &mut App| s.modal = Modal::Profile(id))
        .child(
            item("Copy User ID", "profile-more-copy", false, p).on_press_tx(
                move |_: &mut App, tx: &mut rune::Tx| {
                    tx.copy(id.to_string());
                },
            ),
        );
    match relationship {
        Some(RelationshipKind::Friend) => {
            list = list.child(
                item("Remove Friend", "profile-more-remove", true, p).on_press(
                    move |s: &mut App| {
                        s.decline_friend(id);
                        s.modal = Modal::Profile(id);
                    },
                ),
            );
        }
        Some(RelationshipKind::Outgoing) => {
            list = list.child(
                item("Cancel Friend Request", "profile-more-cancel", false, p).on_press(
                    move |s: &mut App| {
                        s.decline_friend(id);
                        s.modal = Modal::Profile(id);
                    },
                ),
            );
        }
        Some(RelationshipKind::Incoming) => {
            list = list.child(
                item("Accept Friend Request", "profile-more-accept", false, p).on_press(
                    move |s: &mut App| {
                        s.accept_friend(id);
                        s.modal = Modal::Profile(id);
                    },
                ),
            );
        }
        _ => {}
    }
    list = list.child(item("Block User", "profile-more-block", true, p).on_press(
        move |s: &mut App| {
            s.block_user(id);
            s.modal = Modal::Profile(id);
        },
    ));
    list.child(el().w_full().h(1.0).fill(p.divider).my(4.0))
        .child(
            item("Close", "profile-more-close", false, p)
                .on_press(move |s: &mut App| s.modal = Modal::Profile(id)),
        )
}

pub(crate) fn crop_image_sheet(s: &App, p: Palette, banner: bool, user: bool, group: bool) -> El {
    let (width, height) = if banner {
        (400.0, 225.0)
    } else {
        (320.0, 320.0)
    };
    let preview = if let Some(pixels) = &s.crop_preview {
        el().w_full()
            .h_full()
            .clip_rounded(6.0)
            .pixels_fit(pixels, rune::SurfaceFit::Fill)
    } else {
        el().w_full().h_full().fill(p.rail).center().child(
            text("Loading image…")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
    };
    let pan = |label: &'static str, key: &'static str, dx: f32, dy: f32, p: Palette| {
        el().key(key)
            .size(32.0, 32.0)
            .radius(4.0)
            .center()
            .role("button")
            .label(label)
            .fill(p.rail)
            .hover(p.rail_hover)
            .on_press(move |s: &mut App| s.crop_pan(dx, dy))
            .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
    };

    col()
        .key("crop-guild-image")
        .w(500.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(18.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.cancel_guild_image_crop())
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text(if group {
                        "Crop Group Icon"
                    } else {
                        match (user, banner) {
                            (true, true) => "Crop Profile Banner",
                            (true, false) => "Crop Profile Avatar",
                            (false, true) => "Crop Server Banner",
                            (false, false) => "Crop Server Icon",
                        }
                    })
                    .font(theme::TITLE)
                    .color(p.text_bright)
                    .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text(if banner {
                "Drag to reposition, then zoom until the 16:9 crop looks right."
            } else {
                "Drag to reposition, then zoom until the square crop looks right."
            })
            .font(theme::SMALL)
            .color(p.text_muted),
        )
        .child(
            el().w(width)
                .h(height)
                .radius(6.0)
                .clip_rounded(6.0)
                .fill(p.rail)
                .role("group")
                .label("Crop preview")
                .on_drag(|s: &mut App, drag, _| {
                    if drag.phase == rune::Phase::Down {
                        s.crop_drag_origin = s.crop_offset;
                    } else if drag.phase == rune::Phase::Move {
                        let origin = s.crop_drag_origin;
                        s.crop_drag_move(drag.dx, drag.dy, origin);
                    }
                })
                .child(preview),
        )
        .child(
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .child(
                    el().key("crop-zoom-out")
                        .size(32.0, 32.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Zoom out")
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .on_press(|s: &mut App| s.crop_zoom_by(-0.1))
                        .child(text("−").font(theme::TITLE).color(p.text_bright)),
                )
                .child(
                    text(format!("{:.0}%", s.crop_zoom * 100.0))
                        .font(theme::SMALL_MED)
                        .color(p.text_bright),
                )
                .child(
                    el().key("crop-zoom-in")
                        .size(32.0, 32.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Zoom in")
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .on_press(|s: &mut App| s.crop_zoom_by(0.1))
                        .child(text("+").font(theme::TITLE).color(p.text_bright)),
                )
                .child(
                    el().key("crop-reset")
                        .h(30.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Reset crop")
                        .hover(p.card_hover)
                        .on_press(|s: &mut App| s.crop_reset())
                        .child(text("Reset").font(theme::LABEL).color(p.text_muted)),
                )
                .child(el().grow(1.0))
                .child(pan("←", "crop-left", -20.0, 0.0, p))
                .child(pan("↑", "crop-up", 0.0, -20.0, p))
                .child(pan("↓", "crop-down", 0.0, 20.0, p))
                .child(pan("→", "crop-right", 20.0, 0.0, p)),
        )
        .child(
            row()
                .w_full()
                .justify_end()
                .gap(8.0)
                .child(
                    el().key("crop-cancel")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Cancel crop")
                        .hover(p.card_hover)
                        .on_press(|s: &mut App| s.cancel_guild_image_crop())
                        .child(text("Cancel").font(theme::SMALL_MED).color(p.text_muted)),
                )
                .child(
                    el().key("crop-apply")
                        .h(34.0)
                        .px(16.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Apply crop")
                        .fill(p.green)
                        .hover(p.green.alpha(0.85))
                        .on_press(|s: &mut App| s.confirm_guild_image_crop())
                        .child(text("Apply").font(theme::SMALL_MED).color(Color::WHITE)),
                ),
        )
}

pub(crate) fn video_quality_sheet(s: &App, p: Palette) -> El {
    let options = [
        (
            VideoQuality::Auto,
            "Auto",
            "Adapts to congestion, starts at 720p and steps down when frames back up.",
        ),
        (VideoQuality::P360, "360p", "640×360 · 15 fps · ~500 kbps"),
        (VideoQuality::P480, "480p", "854×480 · 24 fps · ~900 kbps"),
        (VideoQuality::P720, "720p", "1280×720 · 30 fps · ~1.8 Mbps"),
        (
            VideoQuality::P1080,
            "1080p",
            "1920×1080 · 30 fps · ~3.5 Mbps",
        ),
    ];
    let mut rows = col().w_full().gap(2.0);
    for (quality, label, description) in options {
        let selected = s.video_quality == quality;
        rows = rows.child(
            row()
                .key(format!("video-quality-{}", quality.key()))
                .w_full()
                .min_h(52.0)
                .px(10.0)
                .py(6.0)
                .radius(4.0)
                .gap(10.0)
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
                .on_press(move |s: &mut App| s.set_video_quality(quality))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .child(text(label).font(theme::SMALL_MED).color(p.text_bright))
                        .child(text(description).font(theme::TINY).color(p.text_muted)),
                )
                .children(selected.then(|| icon(&icons::CHECK, 18.0).color(p.green))),
        );
    }

    col()
        .key("video-quality")
        .w(440.0)
        .radius(8.0)
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
                    text("Video Quality")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Applies to your camera and screen share. The receiver shows the sender’s resolution and frame rate.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(rows)
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            text("RECEIVE QUALITY")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(
            row()
                .w_full()
                .gap(6.0)
                .children([ViewQuality::Auto, ViewQuality::High, ViewQuality::Low].map(
                    |quality| {
                        let selected = s.viewer_quality == quality;
                        el().key(format!("view-quality-{}", quality.key()))
                            .h(34.0)
                            .px(12.0)
                            .radius(4.0)
                            .center()
                            .role("radio")
                            .label(quality.label())
                            .checked(selected)
                            .fill(if selected { p.selected } else { p.rail })
                            .hover(p.rail_hover)
                            .on_press(move |s: &mut App| s.set_viewer_quality(quality))
                            .child(
                                text(quality.label())
                                    .font(theme::SMALL_MED)
                                    .color(if selected { p.text_bright } else { p.text_muted }),
                            )
                    },
                )),
        )
        .child(
            text("Auto uses the low simulcast layer when decoding falls behind.")
                .font(theme::TINY)
                .color(p.text_faint),
        )
}

pub(crate) fn prune_sheet(s: &App, p: Palette) -> El {
    let roles: Vec<&Role> = s
        .guild_roles(s.active_guild)
        .iter()
        .filter(|role| role.name != "@everyone")
        .collect();
    let mut role_rows = col().w_full().gap(2.0);
    for role in &roles {
        let id = role.id;
        let checked = s.prune_roles.contains(&id);
        let label = role.name.clone();
        role_rows = role_rows.child(
            row()
                .key(format!("prune-role-{}", id))
                .w_full()
                .h(32.0)
                .px(8.0)
                .radius(4.0)
                .gap(8.0)
                .center_y()
                .role("checkbox")
                .label(format!("Include {label}"))
                .checked(checked)
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.toggle_prune_role(id))
                .child(
                    el().size(16.0, 16.0)
                        .radius(3.0)
                        .fill(if checked { p.green } else { p.input })
                        .border(1.0, p.divider),
                )
                .child(
                    el().size(10.0, 10.0)
                        .radius(5.0)
                        .fill(theme::color(&format!("#{:06x}", role.color))),
                )
                .child(
                    text(label)
                        .font(theme::SMALL)
                        .color(p.text)
                        .grow(1.0)
                        .truncate(),
                ),
        );
    }

    let preview = match (s.prune_loading, s.prune_preview) {
        (true, _) => "Checking inactive members…".to_string(),
        (false, Some(count)) if count == 1 => "1 member would be pruned.".to_string(),
        (false, Some(count)) => format!("{count} members would be pruned."),
        _ => "Preview to count the members this would remove.".to_string(),
    };
    let ready = s.prune_preview.is_some() && !s.prune_loading;
    let count = s.prune_preview.unwrap_or(0);

    col()
        .key("prune-members")
        .w(460.0)
        .radius(8.0)
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
                    text("Prune Members")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Remove members who have not been active recently.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(text("INACTIVITY").font(theme::SECTION).color(p.text_muted))
        .child(
            row()
                .w_full()
                .h(40.0)
                .gap(10.0)
                .center_y()
                .child(
                    el().key("prune-days-minus")
                        .size(30.0, 30.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("One day less")
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .on_press(|s: &mut App| s.prune_days_delta(-1))
                        .child(text("−").font(theme::TITLE).color(p.text_bright)),
                )
                .child(
                    el().key("prune-days")
                        .w(74.0)
                        .h(32.0)
                        .radius(4.0)
                        .center()
                        .fill(p.input)
                        .child(
                            text(format!("{} days", s.prune_days))
                                .font(theme::SMALL_MED)
                                .color(p.text_bright),
                        ),
                )
                .child(
                    el().key("prune-days-plus")
                        .size(30.0, 30.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("One day more")
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .on_press(|s: &mut App| s.prune_days_delta(1))
                        .child(text("+").font(theme::TITLE).color(p.text_bright)),
                )
                .child(
                    text("1–30 days. Bots, the owner and you are never pruned.")
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        )
        .child(
            text("INCLUDE ROLES")
                .font(theme::SECTION)
                .color(p.text_muted)
                .pt(4.0),
        )
        .child(
            text("Members with roles are protected unless you include them here.")
                .font(theme::TINY)
                .color(p.text_faint),
        )
        .child(
            col()
                .w_full()
                .max_h(170.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(role_rows),
        )
        .child(
            row()
                .w_full()
                .min_h(36.0)
                .px(10.0)
                .radius(4.0)
                .center_y()
                .fill(p.input)
                .child(text(preview).font(theme::SMALL).color(if ready {
                    p.text_bright
                } else {
                    p.text_faint
                })),
        )
        .child(
            row()
                .w_full()
                .justify_end()
                .gap(8.0)
                .child(
                    el().key("prune-preview")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Preview members")
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .on_press(|s: &mut App| s.preview_prune())
                        .child(text("Preview").font(theme::SMALL_MED).color(p.text_bright)),
                )
                .child(
                    el().key("prune-confirm")
                        .h(34.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Prune {count} members"))
                        .fill(if ready { p.danger } else { p.rail })
                        .hover(if ready { p.danger_hover } else { p.rail })
                        .on_press(|s: &mut App| {
                            if s.prune_preview.is_some() {
                                s.prune_now();
                            }
                        })
                        .child(
                            text(if ready && count > 0 {
                                format!("Prune {count}")
                            } else {
                                "Prune".to_string()
                            })
                            .font(theme::SMALL_MED)
                            .color(if ready {
                                Color::WHITE
                            } else {
                                p.text_faint
                            }),
                        ),
                ),
        )
}

pub(crate) fn invite_join_sheet(s: &App, p: Palette) -> El {
    col()
        .key("invite-join")
        .w(440.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(20.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text("Join a Server")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Paste an invite code or link. If you are banned, you can appeal from here.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            text_input(&s.join_code, theme::BODY)
                .key("join-code")
                .w_full()
                .h(40.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Invite code or link")
                .placeholder("https://…/invite/abc123 or abc123", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.join_code)
                .on_submit(|s: &mut App, _: &mut rune::Tx| s.join_invite_now()),
        )
        .child(
            row().w_full().justify_end().child(
                el().key("join-submit")
                    .h(36.0)
                    .px(16.0)
                    .radius(4.0)
                    .center()
                    .role("button")
                    .label("Join server")
                    .fill(p.accent)
                    .hover(p.accent_hover)
                    .on_press(|s: &mut App| s.join_invite_now())
                    .child(
                        text("Join Server")
                            .font(theme::SMALL_MED)
                            .color(Color::WHITE),
                    ),
            ),
        )
}

pub(crate) fn ban_appeal_sheet(s: &App, p: Palette) -> El {
    col()
        .key("ban-appeal")
        .w(470.0)
        .radius(8.0)
        .fill(p.modal)
        .padding(20.0)
        .gap(12.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .items_center()
                .child(
                    text("Submit an Appeal")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("You are banned from this server. Explain why the ban should be lifted; server managers see this in Server Settings → Bans.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            text(format!("Invite: {}", s.appeal_code))
                .font(theme::TINY)
                .color(p.text_faint),
        )
        .child(
            text_input(&s.appeal_message, theme::BODY)
                .key("appeal-message")
                .w_full()
                .h(110.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Appeal message")
                .placeholder("What should the moderators know?", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.appeal_message),
        )
        .child(
            row()
                .w_full()
                .justify_end()
                .child(
                    el().key("appeal-submit")
                        .h(36.0)
                        .px(16.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Submit appeal")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(|s: &mut App| s.submit_appeal())
                        .child(
                            text("Submit Appeal")
                                .font(theme::SMALL_MED)
                                .color(Color::WHITE),
                        ),
                ),
        )
}

pub(crate) fn appeals_list(s: &App, p: Palette) -> El {
    let mut list = col().w_full().gap(4.0);
    for appeal in &s.appeals {
        let id = appeal.id;
        let name = appeal.user.display_name.clone();
        list = list.child(
            row()
                .key(format!("appeal-{}", id))
                .w_full()
                .min_h(72.0)
                .px(12.0)
                .py(8.0)
                .radius(6.0)
                .gap(10.0)
                .center_y()
                .fill(p.card)
                .child(avatar(&appeal.user, 36.0, p))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(
                            row()
                                .gap(8.0)
                                .center_y()
                                .child(text(name).font(theme::SMALL_MED).color(p.text_bright))
                                .child(
                                    text(format!("Submitted {}", date_label(appeal.created_at)))
                                        .font(theme::TINY)
                                        .color(p.text_faint),
                                ),
                        )
                        .child(
                            text(appeal.message.clone())
                                .font(theme::SMALL)
                                .color(p.text)
                                .max_lines(3),
                        ),
                )
                .child(
                    el().key(format!("appeal-deny-{}", id))
                        .h(30.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Deny appeal")
                        .fill(p.rail)
                        .hover(p.danger.alpha(0.2))
                        .on_press(move |s: &mut App| s.resolve_appeal(id, false))
                        .child(text("Deny").font(theme::LABEL).color(p.danger)),
                )
                .child(
                    el().key(format!("appeal-accept-{}", id))
                        .h(30.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Accept appeal")
                        .fill(p.green)
                        .hover(p.green.alpha(0.85))
                        .on_press(move |s: &mut App| s.resolve_appeal(id, true))
                        .child(text("Accept").font(theme::LABEL).color(Color::WHITE)),
                ),
        );
    }
    if s.appeals.is_empty() {
        return el();
    }
    col()
        .w_full()
        .gap(8.0)
        .child(
            text(format!("PENDING APPEALS — {}", s.appeals.len()))
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(list)
}

pub(crate) fn dm_settings_sheet(s: &App, p: Palette, channel: Id) -> El {
    let Some(dm) = s.dm_for(channel).cloned() else {
        return el();
    };
    let group = dm.channel.kind == ChannelKind::GroupDm;
    let owner = dm.channel.owner_id == Some(s.me.id);
    let icon_url = dm.channel.icon_url.clone();
    let total = dm.recipients.len() + 1;

    let icon_preview =
        if let Some(pixels) = s.images.get(&icon_url).filter(|_| !icon_url.is_empty()) {
            el().size(80.0, 80.0)
                .radius(40.0)
                .clip_rounded(40.0)
                .pixels_fit(pixels, rune::SurfaceFit::Fill)
        } else {
            dm_avatar(s, &dm, 80.0, p, p.modal, false)
        };

    let mut icon_actions = col().gap(6.0).justify_center();
    if owner {
        icon_actions = icon_actions.child(
            el().key("dm-icon-upload")
                .w(150.0)
                .h(32.0)
                .px(12.0)
                .radius(4.0)
                .center()
                .role("button")
                .label("Change group icon")
                .fill(p.accent)
                .hover(p.accent_hover)
                .on_press_tx(move |_s: &mut App, tx: &mut rune::Tx| {
                    let options = rune::FileDialogOptions {
                        title: "Upload group icon".to_string(),
                        directory: None,
                        file_name: None,
                        filters: Vec::new(),
                    }
                    .filter("Images", &["png", "jpg", "jpeg", "gif", "webp"]);
                    tx.open_file(options, move |s: &mut App, result, _| {
                        if let Ok(Some(path)) = result {
                            s.begin_dm_icon_crop(channel, path);
                        }
                    });
                })
                .child(text("Change Icon").font(theme::LABEL).color(Color::WHITE)),
        );
        if !icon_url.is_empty() {
            icon_actions = icon_actions.child(
                el().key("dm-icon-remove")
                    .w(150.0)
                    .h(30.0)
                    .px(10.0)
                    .radius(4.0)
                    .center()
                    .role("button")
                    .label("Remove group icon")
                    .hover(p.danger.alpha(0.2))
                    .on_press(move |s: &mut App| s.remove_dm_icon(channel))
                    .child(text("Remove").font(theme::LABEL).color(p.danger)),
            );
        }
    }

    let mut members = col().w_full().gap(2.0);
    members = members.child(
        row()
            .w_full()
            .h(40.0)
            .px(8.0)
            .gap(10.0)
            .center_y()
            .child(avatar_status(&s.me, 28.0, p, p.modal, true))
            .child(
                text(format!("{} (you)", s.me.display_name))
                    .font(theme::SMALL)
                    .color(p.text_bright)
                    .grow(1.0),
            )
            .children(
                (owner && group).then(|| text("Owner").font(theme::TINY).color(p.text_faint)),
            ),
    );
    for user in &dm.recipients {
        let user_id = user.id;
        members = members.child(
            row()
                .w_full()
                .h(40.0)
                .px(8.0)
                .gap(10.0)
                .center_y()
                .child(avatar_status(user, 28.0, p, p.modal, true))
                .child(
                    text(user.display_name.clone())
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .grow(1.0)
                        .truncate(),
                )
                .children((owner && group).then(|| {
                    el().key(format!("dm-remove-{user_id}"))
                        .size(28.0, 28.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Remove {}", user.display_name))
                        .help("Remove from group")
                        .hover(p.danger.alpha(0.2))
                        .on_press(move |s: &mut App| s.remove_dm_member(channel, user_id))
                        .child(icon(&icons::CLOSE, 14.0).color(p.danger))
                })),
        );
    }

    let mut addable = col().w_full().gap(2.0);
    let mut candidates = 0;
    for rel in &s.relationships {
        if rel.kind != RelationshipKind::Friend {
            continue;
        }
        if rel.user.id == s.me.id
            || dm.recipients.iter().any(|user| user.id == rel.user.id)
            || !group
        {
            continue;
        }
        candidates += 1;
        let user_id = rel.user.id;
        addable = addable.child(
            row()
                .w_full()
                .h(36.0)
                .px(8.0)
                .gap(10.0)
                .center_y()
                .child(avatar_status(&rel.user, 24.0, p, p.modal, true))
                .child(
                    text(rel.user.display_name.clone())
                        .font(theme::SMALL)
                        .color(p.text_muted)
                        .grow(1.0)
                        .truncate(),
                )
                .child(
                    el().key(format!("dm-add-{user_id}"))
                        .h(28.0)
                        .px(10.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label(format!("Add {}", rel.user.display_name))
                        .fill(p.rail)
                        .hover(p.rail_hover)
                        .on_press(move |s: &mut App| s.add_dm_member(channel, user_id))
                        .child(text("Add").font(theme::TINY).color(p.text_bright)),
                ),
        );
    }

    col()
        .key("dm-settings")
        .w(500.0)
        .radius(8.0)
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
                    text(if group {
                        "Group DM Settings"
                    } else {
                        "Direct Message Settings"
                    })
                    .font(theme::TITLE)
                    .color(p.text_bright)
                    .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            row()
                .w_full()
                .gap(14.0)
                .center_y()
                .child(icon_preview)
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(6.0)
                        .child(
                            text(format!(
                                "{total} {}",
                                if total == 1 { "member" } else { "members" }
                            ))
                            .font(theme::SMALL)
                            .color(p.text_muted),
                        )
                        .child(icon_actions),
                ),
        )
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(text("GROUP NAME").font(theme::SECTION).color(p.text_muted))
        .child(if owner {
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .child(
                    text_input(&s.dm_settings_name, theme::SMALL)
                        .key("dm-settings-name")
                        .grow(1.0)
                        .h(36.0)
                        .radius(4.0)
                        .fill(p.input)
                        .color(p.text)
                        .role("textbox")
                        .label("Group name")
                        .placeholder("Group name", p.text_faint)
                        .padding(10.0)
                        .on_edit(|s: &mut App| &mut s.dm_settings_name),
                )
                .child(
                    el().key("dm-settings-save")
                        .h(36.0)
                        .px(14.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Save group name")
                        .fill(p.green)
                        .hover(p.green.alpha(0.85))
                        .on_press(|s: &mut App| s.save_dm_name())
                        .child(text("Save").font(theme::SMALL_MED).color(Color::WHITE)),
                )
        } else {
            el().w_full()
                .h(36.0)
                .px(10.0)
                .radius(4.0)
                .fill(p.input)
                .center_y()
                .child(
                    text(dm.channel.name.clone())
                        .font(theme::SMALL)
                        .color(p.text_bright)
                        .truncate(),
                )
        })
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(text("MEMBERS").font(theme::SECTION).color(p.text_muted))
        .child(
            col()
                .w_full()
                .max_h(180.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(members),
        )
        .children((group && candidates > 0).then(|| {
            col()
                .w_full()
                .gap(4.0)
                .child(
                    text("ADD A FRIEND")
                        .font(theme::SECTION)
                        .color(p.text_muted),
                )
                .child(
                    col()
                        .w_full()
                        .max_h(150.0)
                        .scroll(ScrollSpec {
                            stick_bottom: false,
                        })
                        .child(addable),
                )
        }))
        .child(el().w_full().h(1.0).fill(p.divider))
        .child(
            row()
                .w_full()
                .gap(8.0)
                .center_y()
                .children((group).then(|| {
                    el().key("dm-settings-leave")
                        .h(34.0)
                        .px(12.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Leave group")
                        .hover(p.danger.alpha(0.2))
                        .on_press(move |s: &mut App| s.leave_dm(channel))
                        .child(text("Leave Group").font(theme::SMALL).color(p.danger))
                }))
                .child(el().grow(1.0))
                .child(
                    el().key("dm-settings-close")
                        .h(34.0)
                        .px(16.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Close group settings")
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .on_press(|s: &mut App| s.modal = Modal::None)
                        .child(text("Done").font(theme::SMALL_MED).color(Color::WHITE)),
                ),
        )
}

pub(crate) fn group_dm_sheet(s: &App, p: Palette) -> El {
    let mut rows = col().w_full().gap(2.0);
    let mut friends = 0;
    for rel in &s.relationships {
        if rel.kind != RelationshipKind::Friend {
            continue;
        }
        friends += 1;
        let id = rel.user.id;
        let selected = s.group_dm_selected.contains(&id);
        rows = rows.child(
            row()
                .key(format!("group-dm-user-{id}"))
                .w_full()
                .h(44.0)
                .px(8.0)
                .radius(4.0)
                .gap(10.0)
                .center_y()
                .role("checkbox")
                .label(format!("Add {}", rel.user.display_name))
                .checked(selected)
                .fill(if selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.card_hover)
                .on_press(move |s: &mut App| s.toggle_group_dm(id))
                .child(
                    el().size(18.0, 18.0)
                        .radius(4.0)
                        .fill(if selected { p.green } else { p.input })
                        .border(1.0, p.divider)
                        .center()
                        .children(selected.then(|| icon(&icons::CHECK, 12.0).color(Color::WHITE))),
                )
                .child(avatar_status(&rel.user, 28.0, p, p.card, false))
                .child(
                    text(rel.user.display_name.clone())
                        .font(theme::SMALL_MED)
                        .color(p.text_bright)
                        .grow(1.0)
                        .truncate(),
                ),
        );
    }
    if friends == 0 {
        rows = rows.child(
            text("Add a friend first.")
                .font(theme::SMALL)
                .color(p.text_muted)
                .padding(8.0),
        );
    }
    let count = s.group_dm_selected.len();
    col()
        .key("group-dm")
        .w(480.0)
        .radius(8.0)
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
                    text("Create Group DM")
                        .font(theme::TITLE)
                        .color(p.text_bright)
                        .grow(1.0),
                )
                .child(close_button(p)),
        )
        .child(
            text("Pick at least two friends. You can name the group now or leave it as the member list.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            text("GROUP NAME (OPTIONAL)")
                .font(theme::SECTION)
                .color(p.text_muted),
        )
        .child(
            text_input(&s.group_dm_name, theme::SMALL)
                .key("group-dm-name")
                .w_full()
                .h(36.0)
                .radius(4.0)
                .fill(p.input)
                .color(p.text)
                .role("textbox")
                .label("Group name")
                .placeholder("Weekend crew", p.text_faint)
                .padding(10.0)
                .on_edit(|s: &mut App| &mut s.group_dm_name),
        )
        .child(
            col()
                .w_full()
                .max_h(300.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(rows),
        )
        .child(
            row()
                .w_full()
                .gap(10.0)
                .center_y()
                .child(
                    text(format!("{count} selected"))
                        .font(theme::TINY)
                        .color(p.text_muted)
                        .grow(1.0),
                )
                .child(
                    el().key("group-dm-cancel")
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
                    el().key("group-dm-create")
                        .h(34.0)
                        .px(16.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Create group DM")
                        .fill(if count >= 2 { p.green } else { p.rail })
                        .hover(if count >= 2 { p.green.alpha(0.85) } else { p.rail })
                        .on_press(|s: &mut App| s.create_group_dm())
                        .child(
                            text("Create")
                                .font(theme::SMALL_MED)
                                .color(if count >= 2 { Color::WHITE } else { p.text_faint }),
                        ),
                ),
        )
}

pub(crate) fn add_friend_sheet(s: &App, p: Palette) -> El {
    modal_frame(
        "add-friend",
        "Add Friend",
        "Add friends by their username. This is a local account on your machine.",
        p,
        col()
            .w_full()
            .gap(8.0)
            .child(text("USERNAME").font(theme::SECTION).color(p.text_muted))
            .child(
                text_input(&s.search, theme::BODY)
                    .key("friend-username")
                    .w_full()
                    .h(40.0)
                    .radius(4.0)
                    .fill(p.rail)
                    .color(p.text)
                    .role("textbox")
                    .label("Username")
                    .placeholder("maya", p.text_faint)
                    .padding(10.0)
                    .on_edit(|s: &mut App| &mut s.search)
                    .on_submit(|s: &mut App, _: &mut rune::Tx| s.add_friend()),
            )
            .child(
                text("Try maya, leo, ana or sam.")
                    .font(theme::TINY)
                    .color(p.text_faint),
            ),
        ("Send Friend Request", |s: &mut App| s.add_friend()),
        p,
    )
}

pub(crate) fn invite_sheet(s: &App, p: Palette) -> El {
    let code = s.invite_code.clone();
    let url = code
        .as_ref()
        .map(|c| format!("http://127.0.0.1:47800/invite/{c}"));
    let body = col()
        .w_full()
        .gap(12.0)
        .child(
            text("Send a server invite link to a friend.")
                .font(theme::SMALL)
                .color(p.text_muted),
        )
        .child(
            row()
                .w_full()
                .h(44.0)
                .radius(4.0)
                .fill(p.rail)
                .px(12.0)
                .gap(10.0)
                .center_y()
                .child(
                    text(url.clone().unwrap_or_else(|| "Generating invite…".into()))
                        .font(theme::MONO)
                        .color(p.text)
                        .grow(1.0)
                        .min_w(0.0)
                        .truncate(),
                )
                .child(
                    el().h(30.0)
                        .px(12.0)
                        .radius(4.0)
                        .fill(p.accent)
                        .hover(p.accent_hover)
                        .center()
                        .on_press_tx(move |_: &mut App, tx: &mut rune::Tx| {
                            if let Some(url) = &url {
                                tx.copy(url.clone());
                            }
                        })
                        .child(text("Copy").font(theme::SMALL_MED).color(Color::WHITE)),
                ),
        )
        .child(
            text("Invite codes can be joined from any local Runecord account.")
                .font(theme::TINY)
                .color(p.text_faint)
                .wrap(),
        );
    modal_frame_done(
        "invite",
        "Invite friends",
        "Let’s get some people in here.",
        p,
        body,
        ("Done", |s: &mut App| s.modal = Modal::None),
        p,
    )
}

mod admin;
mod emoji;
mod profile;
mod search;
mod threads;

pub(crate) use admin::*;
pub(crate) use emoji::*;
pub(crate) use profile::*;
pub(crate) use search::*;
pub(crate) use threads::*;
