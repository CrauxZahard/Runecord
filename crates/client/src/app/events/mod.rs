//! Network event application.

use super::*;
use crate::net::{Command, NetEvent};
use rune::Tx;

mod bots;
mod channels;
mod guilds;
mod members;
mod messages;
mod moderation;
mod roles;
mod session;
mod social;
mod voice;

impl App {
    pub fn setup(s: &mut App, waker: rune::Waker) {
        if let Some(net) = &s.net {
            net.set_waker(waker.clone());
        }
        crate::voice::set_waker(waker);
    }

    pub fn poll(&mut self, t: f64, tx: &mut Tx) -> bool {
        self.clock = t;
        let mut changed = self.tick_animations(tx);
        let mut events = Vec::new();
        let drained = crate::net::drain_events();
        events.extend(drained);
        for ev in events {
            self.apply(ev);
            changed = true;
        }
        if matches!(self.modal, Modal::Settings(SettingsTab::Voice)) {
            let version = crate::voice::camera_test_version();
            if version != self.last_camera_test_version {
                self.last_camera_test_version = version;
                changed = true;
            }
            if crate::voice::mic_testing()
                || crate::voice::camera_testing()
                || self.voice_mode == "ptt"
            {
                changed = true;
            }
            tx.wake_after(std::time::Duration::from_millis(90));
        }
        if let Modal::Profile(user) | Modal::FullProfile(user) = self.modal {
            if !self.mutuals.contains_key(&user) && self.mutual_requests.insert(user) {
                self.send(Command::LoadMutuals { user });
            }
            if let Some(activity) = self.user(user).and_then(|u| u.activity.as_ref())
                && !activity.artwork_url.is_empty()
                && !self.images.contains_key(&activity.artwork_url)
            {
                self.request_urls(vec![activity.artwork_url.clone()]);
            }
            if self
                .user(user)
                .is_some_and(|user| !user.profile_effect.is_empty() || !user.nameplate.is_empty())
            {
                changed = true;
                tx.wake_after(std::time::Duration::from_millis(33));
            }
        }
        if self.my_voice_channel().is_some() {
            let (frames, stopped) = crate::voice::drain_video();
            for (user, width, height, rgba) in frames {
                if let Some(pixels) = self.voice_video.get(&user) {
                    pixels.set(width, height, rgba);
                } else {
                    let pixels = rune::Pixels::new();
                    pixels.set(width, height, rgba);
                    self.voice_video.insert(user, pixels);
                }
                changed = true;
            }
            for user in stopped {
                if self.voice_video.remove(&user).is_some() {
                    changed = true;
                }
            }
            let stats = crate::voice::video_stats();
            if stats != self.voice_video_info {
                self.voice_video_info = stats;
                changed = true;
            }
            tx.wake_after(std::time::Duration::from_millis(50));
        }
        if self.zoom.gliding() {
            self.zoom.poll(t, tx);
            changed = true;
        }
        let typed = self.member_search.text.clone();
        if typed != self.member_search_live {
            self.member_search_live = typed;
            self.member_search_due = t + 0.28;
        }
        if self.member_search_due > 0.0 && t >= self.member_search_due {
            self.member_search_due = 0.0;
            let query = self.member_search_live.trim().to_string();
            if query != self.last_member_query {
                self.last_member_query = query.clone();
                if self.screen == Screen::Guild {
                    self.search_members(query);
                    changed = true;
                }
            }
        }
        if self.member_search_due > 0.0 {
            tx.wake_after(std::time::Duration::from_millis(140));
        }
        let now = now_ms();
        for list in self.typing.values_mut() {
            let before = list.len();
            list.retain(|(_, until)| *until > now);
            changed |= before != list.len();
        }
        if self.bootstrap_pending && self.clock - self.last_bootstrap_request >= 0.4 {
            self.last_bootstrap_request = self.clock;
            self.send(Command::Bootstrap);
        }
        if self.clock - self.last_maintenance >= 5.0 {
            changed |= self.maintenance();
        }
        self.maybe_save_cache();
        if let Some(toast) = &self.toast
            && toast.until < t
        {
            self.toast = None;
            changed = true;
        }
        if self.toast.is_some() || self.typing.values().any(|v| !v.is_empty()) {
            tx.wake_after(std::time::Duration::from_millis(200));
        }
        let skeleton_loading = self.logged_in
            && self.connected
            && self.screen == Screen::Guild
            && ((self.active_channel != 0 && !self.messages.contains_key(&self.active_channel))
                || self.guild_detail(self.active_guild).is_none()
                || self
                    .member_state
                    .get(&self.active_guild)
                    .is_some_and(|state| {
                        state.loading
                            && self
                                .members
                                .get(&self.active_guild)
                                .is_none_or(|members| members.is_empty())
                    }));
        if skeleton_loading {
            changed = true;
            tx.wake_after(std::time::Duration::from_millis(33));
        }
        changed
    }

    pub(crate) fn apply(&mut self, ev: NetEvent) {
        match ev {
            NetEvent::Session(auth) => self.apply_session(auth),
            NetEvent::Bootstrap(b) => self.apply_bootstrap_event(b),
            NetEvent::Users(users) => self.apply_users(users),
            NetEvent::Lookup(users) => self.apply_lookup(users),
            NetEvent::Bots(bots) => self.apply_bots(bots),
            NetEvent::BotCreated { bot, token } => self.apply_bot_created(bot, token),
            NetEvent::Applications(apps) => self.apply_applications(apps),
            NetEvent::ApplicationCreated { application, token } => {
                self.apply_application_created(application, token)
            }
            NetEvent::TokenRevealed { label, token } => self.apply_token_revealed(label, token),
            NetEvent::InstallLink(code) => self.apply_install_link(code),
            NetEvent::ApplicationInstalled(application) => {
                self.apply_application_installed(application)
            }
            NetEvent::RoleCreated(role) => self.apply_role_created(role),
            NetEvent::RoleLinks { role, links } => self.apply_role_links(role, links),
            NetEvent::GuildEvents(events) => self.apply_guild_events(events),
            NetEvent::GuildEventCreated(event) => self.apply_guild_event_created(event),
            NetEvent::GuildEventDeleted { guild, event } => {
                self.apply_guild_event_deleted(guild, event)
            }
            NetEvent::ServerChecked { url, ok, error } => self.apply_server_checked(url, ok, error),
            NetEvent::ConnectionFailed => self.apply_connection_failed(),
            NetEvent::Inbox(inbox) => self.apply_inbox(*inbox),
            NetEvent::RoleUpdated(role) => self.apply_role_updated(role),
            NetEvent::RoleDeleted { guild, role } => self.apply_role_deleted(guild, role),
            NetEvent::Emojis(emojis) => self.apply_emojis(emojis),
            NetEvent::EmojiCreated(emoji) => self.apply_emoji_created(emoji),
            NetEvent::EmojiDeleted { id, .. } => self.apply_emoji_deleted(id),
            NetEvent::AuditPage { entries, older } => self.apply_audit_page(entries, older),
            NetEvent::GuildCommands(commands) => self.apply_guild_commands(commands),
            NetEvent::UserUpdate(user) => self.apply_user_update(user),
            NetEvent::Notice(text) => self.apply_notice(text),
            NetEvent::GuildDetail(d) => self.apply_guild_detail(d),
            NetEvent::MembersPage {
                guild,
                channel,
                page,
                replace,
            } => self.apply_members_page(guild, channel, *page, replace),
            NetEvent::MemberUpdate(member) => self.apply_member_update(member),
            NetEvent::MemberRemove { guild, user } => self.apply_member_remove(guild, user),
            NetEvent::MemberCountUpdate { guild, count } => {
                self.apply_member_count_update(guild, count)
            }
            NetEvent::Connected(up) => self.apply_connected(up),
            NetEvent::Messages {
                channel,
                messages,
                before,
            } => self.apply_messages(channel, messages, before),
            NetEvent::MessageCreate(m) => self.apply_message_create(m),
            NetEvent::MessageUpdate(m) => self.apply_message_update(m),
            NetEvent::MessageDelete { channel, message } => {
                self.apply_message_delete(channel, message)
            }
            NetEvent::VoiceState(v) => self.apply_voice_state(*v),
            NetEvent::Presence { user, status } => self.apply_presence(user, status),
            NetEvent::Typing {
                channel,
                user,
                until,
            } => self.apply_typing(channel, user, until),
            NetEvent::GuildDiscovery {
                guild,
                discoverable,
            } => self.apply_guild_discovery(guild, discoverable),
            NetEvent::GuildLeft(guild) => self.apply_guild_left(guild),
            NetEvent::ChannelNotify {
                channel,
                level,
                suppress,
                clear,
            } => self.apply_channel_notify(channel, level, suppress, clear),
            NetEvent::GuildNotifyUpdated { guild, settings } => {
                self.apply_guild_notify_updated(guild, settings)
            }
            NetEvent::PublicGuilds(guilds) => self.apply_public_guilds(guilds),
            NetEvent::Pins(messages) => self.apply_pins(messages),
            NetEvent::Bans(bans) => self.apply_bans(bans),
            NetEvent::Mutuals { user, guilds } => self.apply_mutuals(user, guilds),
            NetEvent::GuildUpdated(summary) => self.apply_guild_updated(*summary),
            NetEvent::PrunePreview { days, count } => self.apply_prune_preview(days, count),
            NetEvent::Appeals(appeals) => self.apply_appeals(appeals),
            NetEvent::AppealResolved { appeal, accepted } => {
                self.apply_appeal_resolved(appeal, accepted)
            }
            NetEvent::AppealSubmitted => self.apply_appeal_submitted(),
            NetEvent::Pruned { count } => self.apply_pruned(count),
            NetEvent::Unbanned { guild, user } => self.apply_unbanned(guild, user),
            NetEvent::JoinedPublicGuild(guild) => self.apply_joined_public_guild(guild),
            NetEvent::GuildChanged | NetEvent::RelationshipChanged => self.apply_guild_changed(),
            NetEvent::ChannelCreate(c) => self.apply_channel_create(c),
            NetEvent::ChannelUpdated(c) => self.apply_channel_updated(c),
            NetEvent::ThreadCreated(thread) => self.apply_thread_created(thread),
            NetEvent::ChannelDelete(id) => self.apply_channel_delete(id),
            NetEvent::Image {
                url,
                width,
                height,
                rgba,
                frames,
            } => self.apply_image(url, width, height, rgba, frames),
            NetEvent::Uploaded(a) => self.apply_uploaded(*a),
            NetEvent::Search(results) => self.apply_search(*results),
            NetEvent::Dm(dm) => self.apply_dm(dm),
            NetEvent::DmUpdated(dm) => self.apply_dm_updated(dm),
            NetEvent::DmRemoved { channel } => self.apply_dm_removed(channel),
            NetEvent::Error(msg) => self.apply_error(msg),
        }
    }
}

impl App {
    pub(crate) fn tick_animations(&mut self, tx: &mut Tx) -> bool {
        if self.animations.is_empty()
            || std::env::var("RUNECORD_ANIM_FREEZE").ok().as_deref() == Some("1")
        {
            return false;
        }
        let now = self.clock;
        let mut changed = false;
        let mut wait = 0.5f64;
        for (url, player) in self.animations.iter_mut() {
            if player.frames.len() < 2 {
                continue;
            }
            let mut guard = 0;
            while now >= player.next_at && guard < 16 {
                player.index = (player.index + 1) % player.frames.len();
                let frame = &player.frames[player.index];
                player
                    .pixels
                    .set(player.width, player.height, frame.rgba.clone());
                player.next_at += frame.delay_ms.max(20) as f64 / 1000.0;
                if std::env::var("RUNECORD_ANIM_LOG").is_ok() {
                    eprintln!("animation {} frame {}", url, player.index);
                }
                changed = true;
                guard += 1;
            }
            let remaining = player.next_at - now;
            if remaining > 0.0 {
                wait = wait.min(remaining);
            }
        }
        tx.wake_after(std::time::Duration::from_secs_f64(wait.clamp(0.01, 0.5)));
        changed
    }
}
