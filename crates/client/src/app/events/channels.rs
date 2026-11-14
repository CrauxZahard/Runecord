//! Network event handlers: channels.

use super::super::*;
use crate::net::Command;

impl App {
    pub(crate) fn apply_pins(&mut self, messages: Vec<Message>) {
        self.pinned = messages;
    }
    pub(crate) fn apply_channel_create(&mut self, c: Box<Channel>) {
        let id = c.id;
        let forum_parent = c.thread.as_ref().map(|info| info.parent_id);
        if let Some(guild) = c.guild_id {
            self.channels.entry(guild).or_default().push(*c.clone());
            if let Some(detail) = self.guild_details.get_mut(&guild) {
                detail.channels.push(*c);
            }
        }
        if let (Some(forum), Some(parent)) = (self.pending_forum, forum_parent)
            && parent == forum
        {
            self.pending_forum = None;
            self.open_thread(id);
        }
        if let Some(guild) = self.channel(id).and_then(|c| c.guild_id) {
            self.stale_guilds.insert(guild);
            self.ensure_guild(guild);
        }
        self.mark_cache_dirty();
    }
    pub(crate) fn apply_channel_updated(&mut self, c: Box<Channel>) {
        if let Some(guild) = c.guild_id {
            if let Some(list) = self.channels.get_mut(&guild) {
                if let Some(existing) = list.iter_mut().find(|item| item.id == c.id) {
                    *existing = *c.clone();
                } else {
                    list.push(*c.clone());
                }
            }
            if let Some(detail) = self.guild_details.get_mut(&guild)
                && let Some(existing) = detail.channels.iter_mut().find(|item| item.id == c.id)
            {
                *existing = *c.clone();
            }
        }
        self.mark_cache_dirty();
    }
    pub(crate) fn apply_thread_created(&mut self, thread: Box<Channel>) {
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
    pub(crate) fn apply_channel_delete(&mut self, id: Id) {
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
}
