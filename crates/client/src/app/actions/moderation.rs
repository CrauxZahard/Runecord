//! App actions split by feature.

use super::super::*;
use crate::net::Command;

impl App {
    pub fn prune_now(&mut self) {
        self.send(Command::PruneMembers {
            guild: self.active_guild,
            days: self.prune_days,
            roles: self.prune_roles.iter().copied().collect(),
        });
        self.modal = Modal::None;
    }

    pub fn kick_now(&mut self) {
        let Modal::ConfirmKick { guild, user } = self.modal.clone() else {
            return;
        };
        let name = self
            .user(user)
            .map(|user| user.display_name.clone())
            .unwrap_or_default();
        self.send(Command::KickMember { guild, user });
        self.modal = Modal::None;
        self.toast(format!("{name} was kicked"));
    }

    pub fn ban_now(&mut self) {
        let Modal::ConfirmBan { guild, user } = self.modal.clone() else {
            return;
        };
        let name = self
            .user(user)
            .map(|user| user.display_name.clone())
            .unwrap_or_default();
        let reason = self.ban_reason.text.trim().to_string();
        self.send(Command::BanMember {
            guild,
            user,
            reason,
        });
        self.modal = Modal::None;
        self.toast(format!("{name} was banned"));
    }

    pub fn timeout_now(&mut self, seconds: u64) {
        let Modal::TimeoutMember { guild, user } = self.modal.clone() else {
            return;
        };
        let name = self
            .user(user)
            .map(|user| user.display_name.clone())
            .unwrap_or_default();
        self.send(Command::TimeoutMember {
            guild,
            user,
            seconds,
        });
        self.modal = Modal::None;
        if seconds == 0 {
            self.toast(format!("Timeout removed for {name}"));
        } else {
            self.toast(format!("{name} was timed out"));
        }
    }

    pub fn leave_current_guild(&mut self) {
        let guild = self.active_guild;
        self.send(Command::LeaveGuild { guild });
        self.modal = Modal::None;
    }

    pub fn open_channel_notifications(&mut self, channel: Id) {
        self.modal = Modal::ChannelNotifications(channel);
    }

    pub fn set_channel_notify(&mut self, channel: Id, level: NotificationLevel) {
        self.channel_notify.insert(channel, level);
        let suppress = self.channel_suppress.contains(&channel);
        self.send(Command::SetChannelNotify {
            channel,
            level,
            suppress,
            clear: false,
        });
    }

    pub fn clear_channel_notify(&mut self, channel: Id) {
        self.channel_notify.remove(&channel);
        self.channel_suppress.remove(&channel);
        self.send(Command::SetChannelNotify {
            channel,
            level: NotificationLevel::All,
            suppress: false,
            clear: true,
        });
    }

    pub fn guild_notification(&self, guild: Id) -> GuildNotificationSettings {
        self.guild_notify.get(&guild).cloned().unwrap_or_default()
    }

    pub fn update_guild_notification(
        &mut self,
        guild: Id,
        change: impl FnOnce(&mut GuildNotificationSettings),
    ) {
        let mut settings = self.guild_notification(guild);
        change(&mut settings);
        self.guild_notify.insert(guild, settings.clone());
        self.guild_notify_local_at.insert(guild, self.clock);
        self.send(Command::SetGuildNotify { guild, settings });
        self.bootstrap_pending = true;
        self.mark_cache_dirty();
    }

    pub fn set_guild_notify_level(&mut self, guild: Id, level: NotificationLevel) {
        self.update_guild_notification(guild, |settings| settings.level = level);
    }

    pub fn set_active_guild_notify_level(&mut self, level: NotificationLevel) {
        let guild = self.active_guild;
        self.set_guild_notify_level(guild, level);
    }

    pub fn toggle_active_guild_mute(&mut self) {
        let guild = self.active_guild;
        self.toggle_guild_mute(guild);
    }

    pub fn toggle_active_guild_suppress(&mut self, field: &'static str) {
        let guild = self.active_guild;
        let settings = self.guild_notification(guild);
        let value = match field {
            "everyone" => !settings.suppress_everyone,
            "roles" => !settings.suppress_roles,
            "highlights" => !settings.suppress_highlights,
            _ => return,
        };
        self.set_guild_suppress(guild, field, value);
    }

    pub fn toggle_guild_suppress_everyone(&mut self) {
        self.toggle_active_guild_suppress("everyone");
    }

    pub fn toggle_guild_suppress_roles(&mut self) {
        self.toggle_active_guild_suppress("roles");
    }

    pub fn toggle_guild_suppress_highlights(&mut self) {
        self.toggle_active_guild_suppress("highlights");
    }

    pub fn toggle_guild_mute_new_events(&mut self) {
        let guild = self.active_guild;
        self.update_guild_notification(guild, |settings| {
            settings.mute_new_events = !settings.mute_new_events
        });
    }

    pub fn toggle_guild_mobile_push(&mut self) {
        let guild = self.active_guild;
        self.update_guild_notification(guild, |settings| {
            settings.mobile_push = !settings.mobile_push
        });
    }

    pub fn toggle_guild_mute(&mut self, guild: Id) {
        self.update_guild_notification(guild, |settings| settings.muted = !settings.muted);
    }

    pub fn set_guild_suppress(&mut self, guild: Id, field: &str, value: bool) {
        self.update_guild_notification(guild, |settings| match field {
            "everyone" => settings.suppress_everyone = value,
            "roles" => settings.suppress_roles = value,
            "highlights" => settings.suppress_highlights = value,
            _ => {}
        });
    }

    pub fn add_guild_notification_override(&mut self, guild: Id, channel: Id) {
        self.set_channel_notify(channel, NotificationLevel::All);
        self.modal = Modal::ServerNotifications;
        let _ = guild;
    }

    pub fn set_channel_suppress(&mut self, channel: Id, suppress: bool) {
        if suppress {
            self.channel_suppress.insert(channel);
        } else {
            self.channel_suppress.remove(&channel);
        }
        let level = self
            .channel_notify
            .get(&channel)
            .copied()
            .unwrap_or_default();
        self.send(Command::SetChannelNotify {
            channel,
            level,
            suppress,
            clear: false,
        });
    }

    pub fn mute_channel(&mut self, channel: Id) {
        self.set_channel_notify(channel, NotificationLevel::None);
        self.toast("Channel muted");
    }

    pub fn block_user(&mut self, user: Id) {
        self.send(Command::FriendBlock { user });
        self.toast("User blocked");
    }

    pub fn friend_user(&mut self, user: Id) {
        self.send(Command::FriendRequest { user });
        self.toast("Friend request sent");
    }

    pub fn open_explore(&mut self) {
        self.public_guilds.clear();
        self.explore_loading = true;
        self.modal = Modal::Explore;
        self.send(Command::LoadPublicGuilds);
    }

    pub fn join_public_guild(&mut self, guild: Id) {
        self.send(Command::JoinPublicGuild { guild });
    }

    pub fn delete_channel_now(&mut self) {
        self.send(Command::DeleteChannel {
            channel: self.editing_channel,
        });
        self.modal = Modal::None;
        self.toast("Channel deleted");
    }
}
