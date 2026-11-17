//! Bootstrap, snapshots and hydration.

use super::*;
use crate::net::Command;
use std::collections::HashSet;

impl App {
    pub(crate) fn apply_bootstrap(&mut self, b: Bootstrap, revalidate: bool) {
        let switched_account = self.me.id != 0 && self.me.id != b.user.id;
        if switched_account {
            // A profile token can point at a different account than the cached
            // snapshot. Never keep another account's messages or images.
            self.messages.clear();
            self.guild_details.clear();
            self.members.clear();
            self.member_state.clear();
            self.member_groups_collapsed.clear();
            self.images.clear();
            self.animations.clear();
            self.image_sizes.clear();
            self.image_order.clear();
            self.image_bytes = 0;
            self.image_requests.clear();
            self.user_requests.clear();
            self.visible_messages.clear();
            self.loading_older.clear();
            self.older_complete.clear();
            self.channel_seen.clear();
            self.guild_seen.clear();
            self.member_seen.clear();
            self.stale_guilds.clear();
            self.unread.clear();
            self.reads.clear();
            self.attachments.clear();
            self.guild_notify.clear();
            self.guild_notify_local_at.clear();
            self.role_links.clear();
        }
        self.me = b.user.clone();
        if matches!(self.modal, Modal::Settings(SettingsTab::Profile))
            && self.profile_display.text.is_empty()
        {
            self.sync_profile_edits();
        }
        self.logged_in = true;
        self.connecting = false;
        self.auth_busy = false;
        let user_image_urls: Vec<String> = b
            .users
            .iter()
            .flat_map(|user| [user.avatar_url.clone(), user.banner_url.clone()])
            .filter(|url| !url.is_empty())
            .collect();
        self.users = b.users.into_iter().map(|u| (u.id, u)).collect();
        self.users.insert(self.me.id, self.me.clone());
        self.request_urls(user_image_urls);
        self.guilds = b.guilds;
        self.emojis = b.emojis;
        let emoji_urls: Vec<String> = self
            .emojis
            .iter()
            .filter(|emoji| !self.images.contains_key(&emoji.url))
            .map(|emoji| emoji.url.clone())
            .collect();
        self.request_urls(emoji_urls);
        let guild_images: Vec<GuildSummary> = self.guilds.clone();
        self.request_guild_images(&guild_images);
        self.relationships = b.relationships;
        self.dms = b.dms;
        if let Modal::DmSettings { channel } = self.modal
            && let Some(dm) = self.dms.iter().find(|dm| dm.channel.id == channel)
        {
            self.dm_settings_name = TextEdit::new(dm.channel.name.clone());
        }
        let dm_icons: Vec<String> = self
            .dms
            .iter()
            .map(|dm| dm.channel.icon_url.clone())
            .filter(|url| !url.is_empty())
            .collect();
        self.request_urls(dm_icons);
        self.presence = b.presences.into_iter().collect();
        if self.me.id != 0 {
            self.presence.insert(self.me.id, Presence::Online);
            self.me.status = Presence::Online;
        }
        self.voice = b.voice_states.into_iter().map(|v| (v.user_id, v)).collect();
        self.unread = b.unread.into_iter().collect();
        self.channel_notify = b.channel_notifications;
        self.channel_suppress = b.channel_suppress.into_iter().collect();
        for (guild, settings) in b.guild_notifications {
            let fresh = self
                .guild_notify_local_at
                .get(&guild)
                .is_some_and(|at| self.clock - at < 2.0);
            if !fresh {
                self.guild_notify.insert(guild, settings);
            }
        }
        self.reads = b
            .reads
            .into_iter()
            .map(|r| (r.channel_id, r.last_read_id))
            .collect();

        let guild_ids: HashSet<Id> = self.guilds.iter().map(|g| g.id).collect();
        self.guild_details.retain(|id, _| guild_ids.contains(id));
        self.channels.retain(|id, _| guild_ids.contains(id));
        self.members.retain(|id, _| guild_ids.contains(id));
        self.member_state.retain(|id, _| guild_ids.contains(id));

        for dm in &self.dms {
            for u in &dm.recipients {
                self.users.entry(u.id).or_insert_with(|| u.clone());
            }
        }
        for rel in &self.relationships {
            self.users
                .entry(rel.user.id)
                .or_insert_with(|| rel.user.clone());
        }
        if !self.guilds.iter().any(|g| g.id == self.active_guild) {
            self.active_guild = self.guilds.first().map(|g| g.id).unwrap_or(0);
        }
        let channels: Vec<Channel> = self
            .channels
            .get(&self.active_guild)
            .cloned()
            .unwrap_or_default();
        if !channels.iter().any(|c| c.id == self.active_channel) {
            self.active_channel = self
                .last_guild_channel
                .get(&self.active_guild)
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
        if !self.dms.iter().any(|d| d.channel.id == self.active_dm) {
            self.active_dm = self.dms.first().map(|d| d.channel.id).unwrap_or(0);
        }
        if self.active_guild != 0 {
            if revalidate {
                let cached: Vec<Id> = self.guild_details.keys().copied().collect();
                self.stale_guilds.extend(cached);
                // The warm snapshot can hold messages deleted or edited since it
                // was written; reload the visible channels from the server.
                self.messages.clear();
                self.channel_seen.clear();
                self.visible_messages.clear();
                self.loading_older.clear();
                self.older_complete.clear();
                self.suppress_errors_until = self.clock + 4.0;
            }
            self.ensure_guild(self.active_guild);
        }
        if !revalidate {
            return;
        }
        let target = match self.screen {
            Screen::Guild => self.active_channel,
            Screen::Dms => self.active_dm,
        };
        if target != 0 {
            // Always refetch the newest page. Cached history can be older than
            // the last time the app ran, and READY does not replay messages.
            self.channel_seen.insert(target, now_ms());
            self.send(Command::LoadMessages {
                channel: target,
                before: None,
            });
            self.request_channel_images(target);
            self.request_channel_users(target);
            self.mark_read(target);
        }
        if self.bootstrap_changed_at <= self.last_bootstrap_request {
            self.bootstrap_pending = false;
        }
    }

    pub(crate) fn mark_cache_dirty(&mut self) {
        self.cache_dirty_at = Some(self.clock);
    }

    pub(crate) fn maybe_save_cache(&mut self) {
        let Some(dirty_at) = self.cache_dirty_at else {
            return;
        };
        // Save shortly after activity settles, but also force a write on a busy
        // channel so a stream of messages cannot postpone the snapshot forever.
        let quiet = self.clock - dirty_at >= 2.0;
        let overdue = self.clock - self.last_cache_save >= 30.0;
        if (!quiet && !overdue) || self.clock - self.last_cache_save < 10.0 {
            return;
        }
        self.save_cache();
    }

    pub(crate) fn save_cache(&mut self) {
        let snapshot = self.snapshot();
        crate::cache::save(&self.cache_profile, snapshot);
        self.cache_dirty_at = None;
        self.last_cache_save = self.clock;
    }

    pub(crate) fn snapshot(&self) -> crate::cache::Snapshot {
        let mut channel_ids: Vec<Id> = self
            .messages
            .iter()
            .filter(|(_, list)| !list.is_empty())
            .map(|(id, _)| *id)
            .collect();
        channel_ids
            .sort_by_key(|id| std::cmp::Reverse(self.channel_seen.get(id).copied().unwrap_or(0)));
        let messages = channel_ids
            .into_iter()
            .take(crate::cache::SNAPSHOT_CHANNELS)
            .map(|channel_id| {
                let mut list: Vec<Message> = self
                    .messages
                    .get(&channel_id)
                    .map(|list| list.iter().filter(|m| !m.pending).cloned().collect())
                    .unwrap_or_default();
                if list.len() > crate::cache::SNAPSHOT_MESSAGES_PER_CHANNEL {
                    let drop = list.len() - crate::cache::SNAPSHOT_MESSAGES_PER_CHANNEL;
                    list.drain(..drop);
                }
                crate::cache::CachedChannel {
                    channel_id,
                    messages: list,
                }
            })
            .collect();
        let mut details = Vec::new();
        if let Some(detail) = self.guild_details.get(&self.active_guild) {
            details.push(detail.clone());
        }
        for guild in &self.guilds {
            if guild.id == self.active_guild || details.len() >= 6 {
                continue;
            }
            if let Some(detail) = self.guild_details.get(&guild.id) {
                details.push(detail.clone());
            }
        }
        crate::cache::Snapshot {
            version: crate::cache::CACHE_VERSION,
            saved_at: now_ms(),
            bootstrap: Bootstrap {
                user: self.me.clone(),
                guilds: self.guilds.clone(),
                dms: self.dms.clone(),
                relationships: self.relationships.clone(),
                reads: self
                    .reads
                    .iter()
                    .map(|(channel_id, last_read_id)| ReadState {
                        channel_id: *channel_id,
                        last_read_id: *last_read_id,
                        mention_count: self.unread.get(channel_id).copied().unwrap_or(0),
                    })
                    .collect(),
                users: self.users.values().cloned().collect(),
                presences: self
                    .presence
                    .iter()
                    .map(|(id, status)| (*id, *status))
                    .collect(),
                unread: self
                    .unread
                    .iter()
                    .map(|(id, count)| (*id, *count))
                    .collect(),
                emojis: self.emojis.clone(),
                channel_notifications: self.channel_notify.clone(),
                channel_suppress: self.channel_suppress.iter().copied().collect(),
                guild_notifications: self.guild_notify.clone(),
                voice_states: self.voice.values().cloned().collect(),
            },
            details,
            messages,
            active_guild: self.active_guild,
            active_channel: self.active_channel,
            active_dm: self.active_dm,
            dm_home: self.dm_home,
            dm_screen: self.screen == Screen::Dms,
            last_guild_channel: self.last_guild_channel.clone(),
            notify_dm: self.notify_dm,
            notify_mentions: self.notify_mentions,
            compact: self.compact,
            developer: self.developer,
            keybinds: self.keybinds.clone(),
            video_quality: self.video_quality.key().to_string(),
            view_quality: self.viewer_quality.key().to_string(),
            voice_input_device: self.voice_input_device.clone(),
            voice_output_device: self.voice_output_device.clone(),
            voice_camera_device: self.voice_camera_device.clone(),
            voice_input_volume: self.voice_input_volume,
            voice_output_volume: self.voice_output_volume,
            voice_sensitivity: self.voice_sensitivity,
            voice_auto_sensitivity: self.voice_auto_sensitivity,
            voice_mode: self.voice_mode.clone(),
            voice_noise_suppression: self.voice_noise_suppression,
            ptt_key: self.ptt_key,
        }
    }

    pub(crate) fn hydrate(&mut self, snapshot: crate::cache::Snapshot) {
        self.active_guild = snapshot.active_guild;
        self.active_channel = snapshot.active_channel;
        self.active_dm = snapshot.active_dm;
        self.dm_home = snapshot.dm_home;
        self.notify_dm = snapshot.notify_dm;
        self.notify_mentions = snapshot.notify_mentions;
        self.compact = snapshot.compact;
        self.developer = snapshot.developer;
        self.video_quality = crate::voice::VideoQuality::parse(&snapshot.video_quality);
        crate::voice::set_quality(self.video_quality);
        self.viewer_quality = crate::voice::ViewQuality::parse(&snapshot.view_quality);
        crate::voice::set_viewer_quality(self.viewer_quality);
        self.voice_input_device = snapshot.voice_input_device;
        self.voice_output_device = snapshot.voice_output_device;
        self.voice_camera_device = snapshot.voice_camera_device;
        self.voice_input_volume = snapshot.voice_input_volume;
        self.voice_output_volume = snapshot.voice_output_volume;
        self.voice_sensitivity = snapshot.voice_sensitivity;
        self.voice_auto_sensitivity = snapshot.voice_auto_sensitivity;
        self.voice_mode = snapshot.voice_mode;
        self.voice_noise_suppression = snapshot.voice_noise_suppression;
        self.ptt_key = snapshot.ptt_key;
        self.apply_voice_settings();
        let keybinds =
            snapshot
                .keybinds
                .into_iter()
                .fold(default_keybinds(), |mut map, (action, chord)| {
                    map.insert(action, chord);
                    map
                });
        self.keybinds = keybinds;
        self.last_guild_channel = snapshot.last_guild_channel;
        self.screen = if snapshot.dm_screen {
            Screen::Dms
        } else {
            Screen::Guild
        };
        for detail in snapshot.details {
            self.channels
                .insert(detail.guild.id, detail.channels.clone());
            self.guild_details.insert(detail.guild.id, detail);
        }
        self.messages = snapshot
            .messages
            .into_iter()
            .map(|channel| {
                let list = channel
                    .messages
                    .into_iter()
                    .filter(|m| !m.pending)
                    .collect();
                (channel.channel_id, list)
            })
            .collect();
        for channel in self.messages.keys() {
            self.channel_seen.insert(*channel, 0);
        }
        self.apply_bootstrap(snapshot.bootstrap, false);
        self.logged_in = true;
        self.connected = false;
        self.connecting = true;
        let target = self.current_channel_id();
        if target != 0 {
            self.ensure_messages(target);
        }
    }

    // -----------------------------------------------------------------------
    // Sample state for headless snapshots
    // -----------------------------------------------------------------------
}
