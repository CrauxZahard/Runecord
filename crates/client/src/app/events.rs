//! Network event application.

use super::*;
use crate::net::{Command, NetEvent};
use rune::Tx;

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
            if let Some(activity) = self.user(user).and_then(|u| u.activity.as_ref()) {
                if !activity.artwork_url.is_empty()
                    && !self.images.contains_key(&activity.artwork_url)
                {
                    self.request_urls(vec![activity.artwork_url.clone()]);
                }
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
        if let Some(toast) = &self.toast {
            if toast.until < t {
                self.toast = None;
                changed = true;
            }
        }
        if self.toast.is_some() || self.typing.values().any(|v| !v.is_empty()) {
            tx.wake_after(std::time::Duration::from_millis(200));
        }
        let skeleton_loading = self.logged_in
            && self.connected
            && self.screen == Screen::Guild
            && ((self.active_channel != 0 && self.messages.get(&self.active_channel).is_none())
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
            NetEvent::Session(auth) => {
                self.me = auth.user;
                self.logged_in = true;
                self.auth_busy = false;
                self.auth_error = None;
                self.connected = true;
            }
            NetEvent::Bootstrap(b) => {
                self.apply_bootstrap(*b, true);
                self.mark_cache_dirty();
            }
            NetEvent::Users(users) => {
                let urls: Vec<String> = users
                    .iter()
                    .flat_map(|user| [user.avatar_url.clone(), user.banner_url.clone()])
                    .filter(|url| !url.is_empty())
                    .collect();
                for user in users {
                    self.user_requests.remove(&user.id);
                    self.presence.insert(user.id, user.status);
                    self.users.insert(user.id, user);
                }
                self.request_urls(urls);
            }
            NetEvent::Lookup(users) => {
                if let Some(first) = users.first().cloned() {
                    for user in users {
                        self.users.insert(user.id, user);
                    }
                    self.send(Command::FriendRequest { user: first.id });
                    self.toast(format!("Friend request sent to {}", first.display_name));
                } else {
                    self.toast("No local account with that username");
                }
            }
            NetEvent::Bots(bots) => {
                self.bots = bots;
            }
            NetEvent::BotCreated { bot, token } => {
                self.users.insert(bot.id, bot.clone());
                self.bots.insert(0, bot.clone());
                self.token_reveal = Some((format!("{} bot token", bot.display_name), token));
                self.toast("Bot created. Copy the token now; it is shown once.");
            }
            NetEvent::Applications(apps) => {
                self.applications = apps;
            }
            NetEvent::ApplicationCreated { application, token } => {
                self.applications.insert(0, application.clone());
                self.token_reveal = Some((format!("{} token", application.name), token));
                self.toast("Application created. Copy the token now; it is shown once.");
            }
            NetEvent::TokenRevealed { label, token } => {
                self.token_reveal = Some((label, token));
            }
            NetEvent::InstallLink(code) => {
                self.install_code = rune::TextEdit::new(code.clone());
                if let Some(app) = self.pending_install_app.take() {
                    if self.active_guild != 0 {
                        self.send(Command::InstallApplication {
                            code: code.clone(),
                            guild: self.active_guild,
                        });
                        self.toast("Installing application in this server…");
                    } else {
                        self.toast("Select a server before installing");
                    }
                } else {
                    self.toast("Install code ready");
                }
            }
            NetEvent::ApplicationInstalled(application) => {
                if let Some(existing) = self
                    .applications
                    .iter_mut()
                    .find(|item| item.id == application.id)
                {
                    *existing = application;
                } else {
                    self.applications.push(application);
                }
                self.mark_cache_dirty();
            }
            NetEvent::RoleCreated(role) => {
                if !role.icon_url.is_empty() {
                    self.request_urls(vec![role.icon_url.clone()]);
                }
                self.apply_role(role.clone());
                self.select_role(role.id);
            }
            NetEvent::RoleLinks { role, links } => {
                self.role_links.insert(role, links);
            }
            NetEvent::GuildEvents(events) => {
                self.guild_events = events;
            }
            NetEvent::GuildEventCreated(event) => {
                self.guild_events.retain(|item| item.id != event.id);
                self.guild_events.push(event);
                self.guild_events.sort_by_key(|item| item.start_at);
            }
            NetEvent::GuildEventDeleted { guild, event } => {
                let _ = guild;
                self.guild_events.retain(|item| item.id != event);
            }
            NetEvent::ServerChecked { url, ok, error } => {
                self.apply_checked_server(url, ok, error);
            }
            NetEvent::ConnectionFailed => {
                self.server_unreachable = true;
                self.connected = false;
            }
            NetEvent::Inbox(inbox) => {
                if self.modal == Modal::Inbox {
                    if inbox.mentions.is_empty() && !inbox.unreads.is_empty() {
                        self.inbox_tab = 1;
                    } else if !inbox.mentions.is_empty() && inbox.unreads.is_empty() {
                        self.inbox_tab = 0;
                    }
                }
                self.inbox = Some(*inbox);
            }
            NetEvent::RoleUpdated(role) => {
                let selected = self.selected_role == role.id;
                if !role.icon_url.is_empty() {
                    self.request_urls(vec![role.icon_url.clone()]);
                }
                self.apply_role(role.clone());
                if selected {
                    self.role_name = rune::TextEdit::new(role.name);
                    self.role_color = role.color;
                    self.role_permissions = role.permissions;
                    self.role_hoist = role.hoist;
                    self.role_mentionable = role.mentionable;
                }
            }
            NetEvent::RoleDeleted { guild, role } => {
                self.role_links.remove(&role);
                if let Some(detail) = self.guild_details.get_mut(&guild) {
                    detail.roles.retain(|item| item.id != role);
                }
                for list in self.members.values_mut() {
                    for member in list {
                        member.roles.retain(|id| *id != role);
                    }
                }
                if self.selected_role == role {
                    self.selected_role = self
                        .guild_roles(guild)
                        .first()
                        .map(|item| item.id)
                        .unwrap_or(0);
                    if self.selected_role != 0 {
                        let id = self.selected_role;
                        self.select_role(id);
                    }
                }
                self.mark_cache_dirty();
            }
            NetEvent::Emojis(emojis) => {
                self.emojis = emojis;
                let urls: Vec<String> = self
                    .emojis
                    .iter()
                    .filter(|emoji| !self.images.contains_key(&emoji.url))
                    .map(|emoji| emoji.url.clone())
                    .collect();
                self.request_urls(urls);
            }
            NetEvent::EmojiCreated(emoji) => {
                self.emojis
                    .retain(|item| item.id != emoji.id && item.name != emoji.name);
                let url = emoji.url.clone();
                self.emojis.push(emoji);
                self.request_urls(vec![url]);
                self.toast("Emoji added");
            }
            NetEvent::EmojiDeleted { id, .. } => {
                self.emojis.retain(|emoji| emoji.id != id);
                self.toast("Emoji deleted");
            }
            NetEvent::AuditPage { entries, older } => {
                self.audit_loading = false;
                if older {
                    self.audit.extend(entries.iter().cloned());
                } else {
                    self.audit = entries.clone();
                }
                self.audit_complete = entries.len() < 50;
                self.audit_before = self.audit.last().map(|entry| entry.id);
            }
            NetEvent::GuildCommands(commands) => {
                self.commands = commands;
            }
            NetEvent::UserUpdate(user) => {
                let user = *user;
                if user.avatar_url.is_empty() {
                    crate::ui::common::clear_avatar_pixels(user.id);
                }
                let mut urls: Vec<String> = [user.avatar_url.clone(), user.banner_url.clone()]
                    .into_iter()
                    .filter(|url| !url.is_empty())
                    .collect();
                if let Some(activity) = &user.activity {
                    if !activity.artwork_url.is_empty() {
                        urls.push(activity.artwork_url.clone());
                    }
                }
                self.request_urls(urls);
                self.presence.insert(user.id, user.status);
                self.users.insert(user.id, user.clone());
                for list in self.members.values_mut() {
                    if let Some(member) = list.iter_mut().find(|m| m.user.id == user.id) {
                        member.user = user.clone();
                    }
                }
                for relationship in self.relationships.iter_mut() {
                    if relationship.user.id == user.id {
                        relationship.user = user.clone();
                    }
                }
                for dm in self.dms.iter_mut() {
                    for recipient in dm.recipients.iter_mut() {
                        if recipient.id == user.id {
                            *recipient = user.clone();
                        }
                    }
                }
                if user.id == self.me.id {
                    self.me = user;
                    if matches!(self.modal, Modal::Settings(SettingsTab::Profile))
                        && self.profile_display.text.is_empty()
                    {
                        self.sync_profile_edits();
                    }
                    self.mark_cache_dirty();
                }
            }
            NetEvent::Notice(text) => self.toast(text),
            NetEvent::GuildDetail(d) => {
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
            NetEvent::MembersPage {
                guild,
                channel,
                page,
                replace,
            } => {
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
            NetEvent::MemberUpdate(member) => {
                let member = *member;
                let guild = member.guild_id;
                let member_user_is_self = member.user.id == self.me.id;
                self.users.insert(member.user.id, member.user.clone());
                self.presence.insert(member.user.id, member.user.status);
                let list = self.members.entry(guild).or_default();
                if let Some(existing) = list.iter_mut().find(|m| m.user.id == member.user.id) {
                    *existing = member;
                } else if guild == self.active_guild {
                    list.push(member);
                }
                if member_user_is_self {
                    if let Some(existing) = self
                        .members
                        .get(&guild)
                        .and_then(|list| list.iter().find(|m| m.user.id == self.me.id))
                    {
                        self.me = existing.user.clone();
                    }
                }
            }
            NetEvent::MemberRemove { guild, user } => {
                if let Some(list) = self.members.get_mut(&guild) {
                    list.retain(|m| m.user.id != user);
                }
                if let Some(summary) = self.guilds.iter_mut().find(|g| g.id == guild) {
                    summary.member_count = summary.member_count.saturating_sub(1);
                }
                if let Some(state) = self.member_state.get_mut(&guild) {
                    state.total = state.total.saturating_sub(1);
                }
            }
            NetEvent::MemberCountUpdate { guild, count } => {
                if let Some(summary) = self.guilds.iter_mut().find(|g| g.id == guild) {
                    summary.member_count = count;
                }
                if let Some(state) = self.member_state.get_mut(&guild) {
                    state.total = count;
                }
            }
            NetEvent::Connected(up) => {
                self.connected = up;
                self.connecting = !up;
                if up {
                    self.server_unreachable = false;
                }
                if self.me.id != 0 {
                    let status = if up {
                        Presence::Online
                    } else {
                        Presence::Offline
                    };
                    self.presence.insert(self.me.id, status);
                    self.me.status = status;
                    if let Some(user) = self.users.get_mut(&self.me.id) {
                        user.status = status;
                    }
                }
            }
            NetEvent::Messages {
                channel,
                messages,
                before,
            } => {
                let empty = messages.is_empty();
                if before {
                    self.loading_older.remove(&channel);
                    if empty {
                        self.older_complete.insert(channel);
                    }
                }
                let list = self.messages.entry(channel).or_default();
                if before {
                    let mut merged = messages;
                    merged.append(list);
                    *list = merged;
                } else {
                    let mut merged = messages;
                    for m in list.iter().filter(|m| !m.pending) {
                        if !merged.iter().any(|x| x.id == m.id) {
                            merged.push(m.clone());
                        }
                    }
                    // Keep optimistic messages unless the fetched page already
                    // contains their server-side echo.
                    for pending in list.iter().filter(|m| m.pending) {
                        let echoed = merged.iter().any(|x| {
                            !x.pending
                                && x.author_id == pending.author_id
                                && x.content == pending.content
                        });
                        if !echoed {
                            merged.push(pending.clone());
                        }
                    }
                    merged.sort_by_key(|m| m.timestamp);
                    *list = merged;
                }
                self.trim_channel(channel);
                self.request_channel_images(channel);
                self.request_channel_users(channel);
                if let Screen::Guild = self.screen {
                    if channel == self.active_channel {
                        self.mark_read(channel);
                    }
                } else if channel == self.active_dm {
                    self.mark_read(channel);
                }
                self.mark_cache_dirty();
            }
            NetEvent::MessageCreate(m) => {
                let msg = *m;
                let channel_id = msg.channel_id;
                let author_id = msg.author_id;
                let msg_id = msg.id;
                let preview = msg.preview();
                let mentioned = msg.mentions.contains(&self.me.id) || msg.mention_everyone;
                self.request_message_images(&msg);
                self.request_user(author_id);
                self.channel_seen.insert(channel_id, now_ms());
                let list = self.messages.entry(channel_id).or_default();
                list.retain(|x| {
                    !(x.pending && x.author_id == author_id && x.content == msg.content)
                });
                let is_new = !list.iter().any(|x| x.id == msg_id);
                if let Some(existing) = list.iter_mut().find(|x| x.id == msg_id) {
                    *existing = msg;
                } else {
                    list.push(msg);
                    list.sort_by_key(|x| x.timestamp);
                }
                let current = match self.screen {
                    Screen::Guild => self.active_channel,
                    Screen::Dms => self.active_dm,
                };
                self.typing
                    .entry(channel_id)
                    .or_default()
                    .retain(|(u, _)| *u != author_id);
                let mine = author_id == self.me.id;
                let channel = self.channel(channel_id).cloned();
                if is_new
                    && channel
                        .as_ref()
                        .is_some_and(|channel| channel.kind == ChannelKind::Thread)
                {
                    if let Some(list) = self.channels.get_mut(&self.active_guild) {
                        if let Some(thread) = list.iter_mut().find(|item| item.id == channel_id) {
                            if let Some(info) = thread.thread.as_mut() {
                                info.message_count += 1;
                            }
                        }
                    }
                    if let Some(detail) = self.guild_details.get_mut(&self.active_guild) {
                        if let Some(thread) = detail
                            .channels
                            .iter_mut()
                            .find(|item| item.id == channel_id)
                        {
                            if let Some(info) = thread.thread.as_mut() {
                                info.message_count += 1;
                            }
                        }
                    }
                }
                let is_dm = channel
                    .as_ref()
                    .map(|c| c.guild_id.is_none())
                    .unwrap_or(false);
                let level = self
                    .channel_notify
                    .get(&channel_id)
                    .copied()
                    .unwrap_or_default();
                if current == channel_id {
                    self.mark_read(channel_id);
                } else if !mine && level != NotificationLevel::None {
                    *self.unread.entry(channel_id).or_insert(0) += 1;
                    if let Some(guild) = channel.as_ref().and_then(|c| c.guild_id) {
                        if let Some(summary) = self.guilds.iter_mut().find(|g| g.id == guild) {
                            summary.unread_count = summary.unread_count.saturating_add(1);
                            if mentioned {
                                summary.mention_count = summary.mention_count.saturating_add(1);
                            }
                        }
                    }
                }
                let channel_allows = match level {
                    NotificationLevel::All => true,
                    NotificationLevel::Mentions => mentioned,
                    NotificationLevel::None => false,
                };
                let global_allows = if is_dm {
                    self.notify_dm
                } else if mentioned {
                    self.notify_mentions
                } else {
                    true
                };
                let wants_toast = channel_allows && global_allows;
                if wants_toast && !mine && !rune::is_headless() && !rune_notify::app_focused() {
                    let title = if is_dm {
                        self.user_name(author_id)
                    } else {
                        format!(
                            "#{} · {}",
                            channel.map(|c| c.name).unwrap_or_default(),
                            self.user_name(author_id)
                        )
                    };
                    let body = if preview.is_empty() {
                        "Sent an attachment".to_string()
                    } else {
                        preview
                    };
                    rune_notify::post(
                        &rune_notify::Notification::new(format!("msg-{msg_id}"), title)
                            .body(body)
                            .info("channel", channel_id.to_string()),
                    );
                }
                self.trim_channel(channel_id);
                self.mark_cache_dirty();
            }
            NetEvent::MessageUpdate(m) => {
                let msg = *m;
                self.request_message_images(&msg);
                self.request_user(msg.author_id);
                let channel = msg.channel_id;
                if let Some(list) = self.messages.get_mut(&channel) {
                    if let Some(existing) = list.iter_mut().find(|x| x.id == msg.id) {
                        *existing = msg.clone();
                    }
                }
                if let Some(existing) = self.pinned.iter_mut().find(|x| x.id == msg.id) {
                    *existing = msg.clone();
                }
                self.pinned.retain(|m| m.pinned);
                self.trim_channel(channel);
                self.mark_cache_dirty();
            }
            NetEvent::MessageDelete { channel, message } => {
                if let Some(list) = self.messages.get_mut(&channel) {
                    list.retain(|m| m.id != message);
                }
                self.pinned.retain(|m| m.id != message);
                self.mark_cache_dirty();
            }
            NetEvent::VoiceState(v) => {
                if v.channel_id.is_some() {
                    self.voice.insert(v.user_id, *v);
                } else {
                    self.voice.remove(&v.user_id);
                    self.voice_video.remove(&v.user_id);
                    if v.user_id == self.me.id {
                        self.voice_camera = false;
                        self.voice_screen = false;
                        self.voice_video.clear();
                        self.voice_video_info.clear();
                    }
                }
                self.mark_cache_dirty();
            }
            NetEvent::Presence { user, status } => {
                self.presence.insert(user, status);
                if let Some(u) = self.users.get_mut(&user) {
                    u.status = status;
                }
            }
            NetEvent::Typing {
                channel,
                user,
                until,
            } => {
                let list = self.typing.entry(channel).or_default();
                if !list.iter().any(|(u, _)| *u == user) {
                    list.push((user, until));
                } else if let Some(entry) = list.iter_mut().find(|(u, _)| *u == user) {
                    entry.1 = until;
                }
            }
            NetEvent::GuildDiscovery {
                guild,
                discoverable,
            } => {
                if let Some(summary) = self.guilds.iter_mut().find(|item| item.id == guild) {
                    summary.discoverable = discoverable;
                }
                if let Some(detail) = self.guild_details.get_mut(&guild) {
                    detail.guild.discoverable = discoverable;
                }
                self.mark_cache_dirty();
            }
            NetEvent::GuildLeft(guild) => {
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
            NetEvent::ChannelNotify {
                channel,
                level,
                suppress,
                clear,
            } => {
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
            NetEvent::GuildNotifyUpdated { guild, settings } => {
                self.guild_notify.insert(guild, settings);
                self.bootstrap_pending = true;
                self.mark_cache_dirty();
            }
            NetEvent::PublicGuilds(guilds) => {
                self.public_guilds = guilds;
                self.explore_loading = false;
                self.modal = Modal::Explore;
            }
            NetEvent::Pins(messages) => {
                self.pinned = messages;
            }
            NetEvent::Bans(bans) => {
                self.bans = bans;
            }
            NetEvent::Mutuals { user, guilds } => {
                self.mutuals.insert(user, guilds);
            }
            NetEvent::GuildUpdated(summary) => {
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
            NetEvent::PrunePreview { days, count } => {
                self.prune_loading = false;
                if days == self.prune_days {
                    self.prune_preview = Some(count);
                }
            }
            NetEvent::Appeals(appeals) => {
                self.appeals = appeals;
            }
            NetEvent::AppealResolved { appeal, accepted } => {
                self.appeals.retain(|item| item.id != appeal);
                self.toast(if accepted {
                    "Ban appeal accepted and ban lifted"
                } else {
                    "Ban appeal denied"
                });
                self.send(Command::LoadBans {
                    guild: self.active_guild,
                });
                self.send(Command::LoadAppeals {
                    guild: self.active_guild,
                });
            }
            NetEvent::AppealSubmitted => {
                self.modal = Modal::None;
                self.toast("Appeal submitted. A server manager will review it.");
                self.last_join_code.clear();
            }
            NetEvent::Pruned { count } => {
                self.toast(if count == 1 {
                    "1 member pruned".to_string()
                } else {
                    format!("{count} members pruned")
                });
            }
            NetEvent::Unbanned { guild, user } => {
                if guild == self.active_guild {
                    self.bans.retain(|ban| ban.user.id != user);
                }
                self.toast("User unbanned");
            }
            NetEvent::JoinedPublicGuild(guild) => {
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
            NetEvent::GuildChanged | NetEvent::RelationshipChanged => {
                // Coalesce bursts (member chunks, role edits, channel updates)
                // into at most one bootstrap per 400 ms. If another change lands
                // while a fetch is in flight, `bootstrap_pending` stays set.
                self.bootstrap_pending = true;
                self.bootstrap_changed_at = self.clock;
                let ids: Vec<Id> = self.guild_details.keys().copied().collect();
                self.stale_guilds.extend(ids);
            }
            NetEvent::ChannelCreate(c) => {
                let id = c.id;
                let forum_parent = c.thread.as_ref().map(|info| info.parent_id);
                if let Some(guild) = c.guild_id {
                    self.channels.entry(guild).or_default().push(*c.clone());
                    if let Some(detail) = self.guild_details.get_mut(&guild) {
                        detail.channels.push(*c);
                    }
                }
                if let (Some(forum), Some(parent)) = (self.pending_forum, forum_parent) {
                    if parent == forum {
                        self.pending_forum = None;
                        self.open_thread(id);
                    }
                }
                if let Some(guild) = self.channel(id).and_then(|c| c.guild_id) {
                    self.stale_guilds.insert(guild);
                    self.ensure_guild(guild);
                }
                self.mark_cache_dirty();
            }
            NetEvent::ChannelUpdated(c) => {
                if let Some(guild) = c.guild_id {
                    if let Some(list) = self.channels.get_mut(&guild) {
                        if let Some(existing) = list.iter_mut().find(|item| item.id == c.id) {
                            *existing = *c.clone();
                        } else {
                            list.push(*c.clone());
                        }
                    }
                    if let Some(detail) = self.guild_details.get_mut(&guild) {
                        if let Some(existing) =
                            detail.channels.iter_mut().find(|item| item.id == c.id)
                        {
                            *existing = *c.clone();
                        }
                    }
                }
                self.mark_cache_dirty();
            }
            NetEvent::ThreadCreated(thread) => {
                let id = thread.id;
                if let Some(guild) = thread.guild_id {
                    self.channels
                        .entry(guild)
                        .or_default()
                        .push(*thread.clone());
                    if let Some(detail) = self.guild_details.get_mut(&guild) {
                        detail.channels.push(*thread);
                    }
                }
                self.open_thread(id);
                if let Some(body) = self.pending_thread_body.take() {
                    let nonce = format!("thread-{id}-{}", now_ms());
                    self.send(Command::Send {
                        channel: id,
                        content: body,
                        reply_to: None,
                        attachments: Vec::new(),
                        nonce,
                    });
                }
                self.mark_cache_dirty();
            }
            NetEvent::ChannelDelete(id) => {
                for list in self.channels.values_mut() {
                    list.retain(|c| c.id != id);
                }
                for detail in self.guild_details.values_mut() {
                    detail.channels.retain(|c| c.id != id);
                    detail.overwrites.retain(|o| o.channel_id != id);
                }
                self.messages.remove(&id);
                self.channel_seen.remove(&id);
                if self.active_channel == id {
                    self.thread_parent = None;
                    let fallback = self.channels.get(&self.active_guild).and_then(|list| {
                        list.iter()
                            .find(|c| c.kind == ChannelKind::Text)
                            .or_else(|| list.iter().find(|c| c.kind == ChannelKind::Announcement))
                            .map(|c| c.id)
                    });
                    if let Some(fallback) = fallback {
                        self.switch_channel(fallback);
                    }
                }
                self.mark_cache_dirty();
            }
            NetEvent::Image {
                url,
                width,
                height,
                rgba,
                frames,
            } => {
                let pixels = self.insert_image(url.clone(), width, height, rgba);
                if frames.len() >= 2 {
                    let next_at = self.clock + frames[0].delay_ms.max(20) as f64 / 1000.0;
                    self.animations.insert(
                        url,
                        AnimationPlayer {
                            frames,
                            index: 0,
                            next_at,
                            width,
                            height,
                            pixels,
                        },
                    );
                } else {
                    self.animations.remove(&url);
                }
            }
            NetEvent::Uploaded(a) => {
                self.attachments.push(*a);
                self.toast("Attached");
            }
            NetEvent::Search(results) => {
                self.modal = Modal::Search;
                self.search_results = Some(*results);
            }
            NetEvent::Dm(dm) => {
                let id = dm.channel.id;
                let icon = dm.channel.icon_url.clone();
                if !self.dms.iter().any(|d| d.channel.id == id) {
                    self.dms.push(*dm);
                }
                if !icon.is_empty() {
                    self.request_urls(vec![icon]);
                }
                self.open_dm(id);
                self.mark_cache_dirty();
            }
            NetEvent::DmUpdated(dm) => {
                let mut dm = *dm;
                dm.recipients.retain(|user| user.id != self.me.id);
                let urls: Vec<String> = std::iter::once(dm.channel.icon_url.clone())
                    .chain(
                        dm.recipients
                            .iter()
                            .flat_map(|user| [user.avatar_url.clone(), user.banner_url.clone()]),
                    )
                    .filter(|url| !url.is_empty())
                    .collect();
                for user in &dm.recipients {
                    self.users.entry(user.id).or_insert_with(|| user.clone());
                }
                if let Some(existing) = self
                    .dms
                    .iter_mut()
                    .find(|existing| existing.channel.id == dm.channel.id)
                {
                    *existing = dm;
                } else {
                    self.dms.push(dm);
                }
                self.request_urls(urls);
                self.mark_cache_dirty();
            }
            NetEvent::DmRemoved { channel } => {
                self.dms.retain(|dm| dm.channel.id != channel);
                self.messages.remove(&channel);
                self.unread.remove(&channel);
                self.loading_older.remove(&channel);
                self.older_complete.remove(&channel);
                self.channel_seen.remove(&channel);
                if self.active_dm == channel {
                    self.active_dm = 0;
                    self.dm_home = true;
                    self.screen = Screen::Dms;
                }
                if matches!(self.modal, Modal::DmSettings { channel: open } if open == channel) {
                    self.modal = Modal::None;
                }
                self.mark_cache_dirty();
            }
            NetEvent::Error(msg) => {
                for state in self.member_state.values_mut() {
                    state.loading = false;
                }
                self.user_requests.clear();
                if msg.is_empty() {
                    self.logged_in = false;
                    return;
                }
                if self.logged_in && msg.contains("banned") && !self.last_join_code.is_empty() {
                    let code = self.last_join_code.clone();
                    self.last_join_code.clear();
                    self.open_appeal(code);
                    return;
                }
                if let Some(code) = msg.strip_prefix("INVITE:") {
                    self.invite_code = Some(code.to_string());
                    self.modal = Modal::Invite;
                    return;
                }
                if !self.logged_in {
                    self.auth_error = Some(msg);
                    self.auth_busy = false;
                } else if (self.stale_guilds.contains(&self.active_guild)
                    || self.clock < self.suppress_errors_until)
                    && (msg == "not found" || msg == "you don't have permission to do that")
                {
                    // A warm-cache channel can be deleted or newly restricted before
                    // guild detail revalidates; the refresh fixes the selection.
                } else {
                    self.toast(msg);
                }
            }
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
