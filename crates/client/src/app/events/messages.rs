//! Network event handlers: messages.

use super::super::*;

impl App {
    pub(crate) fn apply_messages(&mut self, channel: Id, messages: Vec<Message>, before: bool) {
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
                    !x.pending && x.author_id == pending.author_id && x.content == pending.content
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
    pub(crate) fn apply_message_create(&mut self, m: Box<Message>) {
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
        list.retain(|x| !(x.pending && x.author_id == author_id && x.content == msg.content));
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
            if let Some(list) = self.channels.get_mut(&self.active_guild)
                && let Some(thread) = list.iter_mut().find(|item| item.id == channel_id)
                && let Some(info) = thread.thread.as_mut()
            {
                info.message_count += 1;
            }
            if let Some(detail) = self.guild_details.get_mut(&self.active_guild)
                && let Some(thread) = detail
                    .channels
                    .iter_mut()
                    .find(|item| item.id == channel_id)
                && let Some(info) = thread.thread.as_mut()
            {
                info.message_count += 1;
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
            if let Some(guild) = channel.as_ref().and_then(|c| c.guild_id)
                && let Some(summary) = self.guilds.iter_mut().find(|g| g.id == guild)
            {
                summary.unread_count = summary.unread_count.saturating_add(1);
                if mentioned {
                    summary.mention_count = summary.mention_count.saturating_add(1);
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
    pub(crate) fn apply_message_update(&mut self, m: Box<Message>) {
        let msg = *m;
        self.request_message_images(&msg);
        self.request_user(msg.author_id);
        let channel = msg.channel_id;
        if let Some(list) = self.messages.get_mut(&channel)
            && let Some(existing) = list.iter_mut().find(|x| x.id == msg.id)
        {
            *existing = msg.clone();
        }
        if let Some(existing) = self.pinned.iter_mut().find(|x| x.id == msg.id) {
            *existing = msg.clone();
        }
        self.pinned.retain(|m| m.pinned);
        self.trim_channel(channel);
        self.mark_cache_dirty();
    }
    pub(crate) fn apply_message_delete(&mut self, channel: Id, message: Id) {
        if let Some(list) = self.messages.get_mut(&channel) {
            list.retain(|m| m.id != message);
        }
        self.pinned.retain(|m| m.id != message);
        self.mark_cache_dirty();
    }
    pub(crate) fn apply_presence(&mut self, user: Id, status: Presence) {
        self.presence.insert(user, status);
        if let Some(u) = self.users.get_mut(&user) {
            u.status = status;
        }
    }
    pub(crate) fn apply_typing(&mut self, channel: Id, user: Id, until: i64) {
        let list = self.typing.entry(channel).or_default();
        if !list.iter().any(|(u, _)| *u == user) {
            list.push((user, until));
        } else if let Some(entry) = list.iter_mut().find(|(u, _)| *u == user) {
            entry.1 = until;
        }
    }
    pub(crate) fn apply_image(
        &mut self,
        url: String,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        frames: Vec<crate::image_anim::AnimFrame>,
    ) {
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
    pub(crate) fn apply_uploaded(&mut self, a: Attachment) {
        self.attachments.push(a);
        self.toast("Attached");
    }
    pub(crate) fn apply_search(&mut self, results: SearchResults) {
        self.modal = Modal::Search;
        self.search_results = Some(results);
    }
}
