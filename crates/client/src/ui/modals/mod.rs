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
mod polls;
mod profile;
mod profile_tools;
mod scheduled;
mod server;
mod social;
mod stickers;

mod activities;
mod admin;
mod emoji;
mod gifs;
mod search;
mod slash;
mod threads;
mod volume;

pub(crate) use activities::*;
pub(crate) use admin::*;
pub(crate) use discovery::*;
pub(crate) use emoji::*;
pub(crate) use gifs::*;
pub(crate) use moderation::*;
pub(crate) use notifications::*;
pub(crate) use polls::*;
pub(crate) use profile::*;
pub(crate) use profile_tools::*;
pub(crate) use scheduled::*;
pub(crate) use search::*;
pub(crate) use server::*;
pub(crate) use slash::*;
pub(crate) use social::*;
pub(crate) use stickers::*;
pub(crate) use threads::*;
pub(crate) use volume::*;

pub(crate) fn modal(s: &App, p: Palette) -> El {
    let sheet = match &s.modal {
        Modal::Settings(tab) => settings_sheet(s, p, *tab),
        Modal::StatusPicker => status_picker_sheet(s, p),
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
        Modal::ThreadTags(id) => thread_tags_sheet(s, p, *id),
        Modal::EditMessage(id) => edit_sheet(s, p, *id),
        Modal::ForwardMessage(id) => forward_sheet(s, p, *id),
        Modal::CreatePoll => create_poll_sheet(s, p),
        Modal::StickerPicker => sticker_sheet(s, p),
        Modal::SlashCommand => slash_sheet(s, p),
        Modal::UserVolume(user) => user_volume_sheet(s, p, *user),
        Modal::Activities(channel) => activities_sheet(s, p, *channel),
        Modal::GifPicker => gif_sheet(s, p),
        Modal::ScheduledMessages => scheduled_sheet(s, p),
        Modal::InviteJoin => invite_join_sheet(s, p),
        Modal::BanAppeal => ban_appeal_sheet(s, p),
        Modal::OnboardingRules(guild) => onboarding_rules_sheet(s, p, *guild),
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
    } else if matches!(s.modal, Modal::Profile(_)) {
        // A member's profile appears as a card beside the member list, not as
        // a centred dialog.
        overlay
            .justify_end()
            .items_end()
            .pb(76.0)
            .pr(if s.show_members {
                s.members_width + 14.0
            } else {
                14.0
            })
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
        Modal::StatusPicker => "Set status",
        Modal::Profile(_) => "Profile",
        Modal::FullProfile(_) => "Full profile",
        Modal::EmojiPicker => "Emoji picker",
        Modal::Reaction(_) => "Add reaction",
        Modal::Threads => "Threads",
        Modal::ThreadCreate { .. } => "Create thread",
        Modal::ThreadTags(_) => "Edit tags",
        Modal::EditMessage(_) => "Edit message",
        Modal::ForwardMessage(_) => "Forward message",
        Modal::CreatePoll => "Create poll",
        Modal::StickerPicker => "Stickers",
        Modal::SlashCommand => "Slash command",
        Modal::UserVolume(_) => "User volume",
        Modal::Activities(_) => "Activities",
        Modal::GifPicker => "GIFs",
        Modal::ScheduledMessages => "Scheduled messages",
        Modal::InviteJoin => "Join server",
        Modal::BanAppeal => "Ban appeal",
        Modal::OnboardingRules(_) => "Server rules",
        Modal::ServerSettings | Modal::Roles => "Roles",
        Modal::ChannelPermissions(_) => "Channel permissions",
        Modal::None => "Dialog",
    }
}
