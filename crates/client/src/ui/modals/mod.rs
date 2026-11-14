//! Modals and sheets surface.

use super::*;
use crate::app::TimeoutUnit;

mod events;
mod inbox;
mod soundboard;
use crate::voice::{VideoQuality, ViewQuality};
use runecord_protocol::{PrivacyLevel, Role};

mod discovery;
mod moderation;
mod notifications;
mod profile;
mod profile_tools;
mod server;
mod social;

mod admin;
mod emoji;
mod search;
mod threads;

pub(crate) use admin::*;
pub(crate) use discovery::*;
pub(crate) use emoji::*;
pub(crate) use moderation::*;
pub(crate) use notifications::*;
pub(crate) use profile::*;
pub(crate) use profile_tools::*;
pub(crate) use search::*;
pub(crate) use server::*;
pub(crate) use social::*;
pub(crate) use threads::*;

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
        Modal::InviteJoin => "Join server",
        Modal::BanAppeal => "Ban appeal",
        Modal::ServerSettings | Modal::Roles => "Roles",
        Modal::ChannelPermissions(_) => "Channel permissions",
        Modal::None => "Dialog",
    }
}

pub(super) fn simple_sheet(s: &App, p: Palette, title: &str, body: &str) -> El {
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
