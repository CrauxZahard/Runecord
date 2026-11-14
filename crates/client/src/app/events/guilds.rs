//! Network event handlers: guilds.

use super::super::*;
use crate::net::Command;

impl App {
    pub(crate) fn apply_guild_events(&mut self, events: Vec<GuildEvent>) {
        self.guild_events = events;
    }
    pub(crate) fn apply_guild_event_created(&mut self, event: GuildEvent) {
        self.guild_events.retain(|item| item.id != event.id);
        self.guild_events.push(event);
        self.guild_events.sort_by_key(|item| item.start_at);
    }
    pub(crate) fn apply_guild_event_deleted(&mut self, guild: Id, event: Id) {
        let _ = guild;
        self.guild_events.retain(|item| item.id != event);
    }
    pub(crate) fn apply_guild_detail(&mut self, d: Box<GuildDetail>) {
        let detail = *d;
        let guild = detail.guild.id;
        let summary = detail.guild.clone();
        if let Some(existing) = self.guilds.iter_mut().find(|g| g.id == guild) {
            *existing = summary;
        }
        self.channels.insert(guild, detail.channels.clone());
        let role_icons: Vec<String> = detail
            .roles
            .iter()
            .map(|role| role.icon_url.clone())
            .filter(|url| !url.is_empty())
            .collect();
        self.guild_details.insert(guild, detail);
        self.request_urls(role_icons);
        self.stale_guilds.remove(&guild);
        self.trim_guild_caches();
        if guild == self.active_guild {
            let channels = self.channels.get(&guild).cloned().unwrap_or_default();
            if !channels.iter().any(|c| c.id == self.active_channel) {
                self.active_channel = self
                    .last_guild_channel
                    .get(&guild)
                    .copied()
                    .filter(|id| channels.iter().any(|c| c.id == *id))
                    .or_else(|| {
                        channels
                            .iter()
                            .find(|c| c.kind == ChannelKind::Text)
                            .map(|c| c.id)
                    })
                    .or_else(|| {
                        channels
                            .iter()
                            .find(|c| c.kind == ChannelKind::Announcement)
                            .map(|c| c.id)
                    })
                    .unwrap_or(0);
            }
            if self.active_channel != 0 {
                self.last_guild_channel.insert(guild, self.active_channel);
            }
            self.ensure_members(guild);
            if self.active_channel != 0 {
                self.ensure_messages(self.active_channel);
            }
        }
        self.mark_cache_dirty();
    }
    pub(crate) fn apply_members_page(
        &mut self,
        guild: Id,
        channel: Id,
        page: Box<MemberPage>,
        replace: bool,
    ) {
        if self
            .member_state
            .get(&guild)
            .is_some_and(|state| state.channel != channel)
        {
            return;
        }
        let page = *page;
        let mut next_offset = page.next_offset;
        if let Some(state) = self.member_state.get_mut(&guild) {
            state.loading = false;
            state.total = page.total;
            state.complete = page.next_offset.is_none();
        }
        let members = page.members;
        for member in &members {
            self.users
                .entry(member.user.id)
                .or_insert_with(|| member.user.clone());
        }
        let list = self.members.entry(guild).or_default();
        if replace {
            list.clear();
        }
        for member in members {
            if !list.iter().any(|m| m.user.id == member.user.id) {
                list.push(member);
            }
        }
        if list.len() >= crate::cache::MAX_MEMBERS_PER_GUILD {
            list.truncate(crate::cache::MAX_MEMBERS_PER_GUILD);
            next_offset = None;
        }
        if let Some(state) = self.member_state.get_mut(&guild) {
            state.next_offset = next_offset;
        }
        for (user, status) in page.presences {
            if user == self.me.id {
                continue;
            }
            self.presence.insert(user, status);
            if let Some(u) = self.users.get_mut(&user) {
                u.status = status;
            }
        }
        self.trim_member_caches();
    }
    pub(crate) fn apply_guild_discovery(&mut self, guild: Id, discoverable: bool) {
        if let Some(summary) = self.guilds.iter_mut().find(|item| item.id == guild) {
            summary.discoverable = discoverable;
        }
        if let Some(detail) = self.guild_details.get_mut(&guild) {
            detail.guild.discoverable = discoverable;
        }
        self.mark_cache_dirty();
    }
    pub(crate) fn apply_guild_left(&mut self, guild: Id) {
        let mut removed: Vec<Id> = self
            .channels
            .get(&guild)
            .map(|list| list.iter().map(|channel| channel.id).collect())
            .unwrap_or_default();
        if let Some(detail) = self.guild_details.get(&guild) {
            removed.extend(detail.channels.iter().map(|channel| channel.id));
        }
        for channel in &removed {
            self.messages.remove(channel);
            self.channel_seen.remove(channel);
            self.unread.remove(channel);
            self.reads.remove(channel);
            self.channel_notify.remove(channel);
        }
        self.channels.remove(&guild);
        self.guild_details.remove(&guild);
        self.members.remove(&guild);
        self.member_state.remove(&guild);
        self.last_guild_channel.remove(&guild);
        self.guilds.retain(|item| item.id != guild);
        self.stale_guilds.remove(&guild);
        if self.active_guild == guild {
            if let Some(next) = self.guilds.first().map(|item| item.id) {
                self.switch_guild(next);
            } else {
                self.screen = Screen::Dms;
                self.dm_home = true;
                self.active_guild = 0;
                self.active_channel = 0;
            }
        }
        self.modal = Modal::None;
        self.toast("Left the server");
        self.mark_cache_dirty();
    }
    pub(crate) fn apply_channel_notify(
        &mut self,
        channel: Id,
        level: NotificationLevel,
        suppress: bool,
        clear: bool,
    ) {
        if clear {
            self.channel_notify.remove(&channel);
            self.channel_suppress.remove(&channel);
        } else {
            self.channel_notify.insert(channel, level);
            if suppress {
                self.channel_suppress.insert(channel);
            } else {
                self.channel_suppress.remove(&channel);
            }
        }
        self.toast(if clear {
            "Channel notifications reset".to_string()
        } else {
            match level {
                NotificationLevel::All => "Notifications: all messages".to_string(),
                NotificationLevel::Mentions => "Notifications: only @mentions".to_string(),
                NotificationLevel::None => "Notifications: nothing".to_string(),
            }
        });
    }
    pub(crate) fn apply_guild_notify_updated(
        &mut self,
        guild: Id,
        settings: GuildNotificationSettings,
    ) {
        self.guild_notify.insert(guild, settings);
        self.bootstrap_pending = true;
        self.mark_cache_dirty();
    }
    pub(crate) fn apply_public_guilds(&mut self, guilds: Vec<PublicGuild>) {
        self.public_guilds = guilds;
        self.explore_loading = false;
        self.modal = Modal::Explore;
    }
    pub(crate) fn apply_guild_updated(&mut self, summary: Box<GuildSummary>) {
        let summary = *summary;
        if let Some(existing) = self.guilds.iter_mut().find(|item| item.id == summary.id) {
            *existing = summary.clone();
        }
        if let Some(detail) = self.guild_details.get_mut(&summary.id) {
            detail.guild = summary.clone();
        }
        self.request_guild_images(std::slice::from_ref(&summary));
        self.mark_cache_dirty();
    }
    pub(crate) fn apply_joined_public_guild(&mut self, guild: Id) {
        self.public_guilds.retain(|item| item.id != guild);
        self.screen = Screen::Guild;
        self.dm_home = false;
        self.active_guild = guild;
        self.active_channel = 0;
        self.modal = Modal::None;
        self.bootstrap_pending = true;
        self.ensure_guild(guild);
        self.toast("Joined the server");
    }
    pub(crate) fn apply_guild_changed(&mut self) {
        // Coalesce bursts (member chunks, role edits, channel updates)
        // into at most one bootstrap per 400 ms. If another change lands
        // while a fetch is in flight, `bootstrap_pending` stays set.
        self.bootstrap_pending = true;
        self.bootstrap_changed_at = self.clock;
        let ids: Vec<Id> = self.guild_details.keys().copied().collect();
        self.stale_guilds.extend(ids);
    }
}
