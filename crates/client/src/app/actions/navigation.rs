//! App actions split by feature.

use super::super::*;
use crate::net::Command;
use rune::TextEdit;

impl App {
    pub fn switch_guild(&mut self, id: Id) {
        self.screen = Screen::Guild;
        self.active_guild = id;
        self.member_search.clear();
        self.member_search_live.clear();
        self.member_search_due = 0.0;
        self.last_member_query.clear();
        if self
            .member_state
            .get(&id)
            .is_some_and(|state| !state.query.is_empty())
        {
            self.members.remove(&id);
            self.member_state.remove(&id);
        }
        self.active_channel = self
            .last_guild_channel
            .get(&id)
            .copied()
            .or_else(|| {
                self.channels
                    .get(&id)
                    .and_then(|c| c.iter().find(|c| c.kind == ChannelKind::Text))
                    .map(|c| c.id)
            })
            .unwrap_or(0);
        self.ensure_guild(id);
    }

    pub fn ensure_guild(&mut self, id: Id) {
        if id == 0 {
            return;
        }
        self.guild_seen.insert(id, now_ms());
        if self.stale_guilds.contains(&id) || !self.guild_details.contains_key(&id) {
            self.send(Command::LoadGuildDetail { guild: id });
        }
        self.send(Command::LoadGuildCommands { guild: id });
        self.ensure_members(id);
    }

    pub fn ensure_members(&mut self, guild: Id) {
        self.member_seen.insert(guild, now_ms());
        let channel = if self.screen == Screen::Guild {
            self.active_channel
        } else {
            0
        };
        let state = self.member_state.entry(guild).or_default();
        if state.channel != channel {
            state.channel = channel;
            state.total = 0;
            state.next_offset = None;
            state.complete = false;
            state.loading = false;
            state.query.clear();
            self.members.remove(&guild);
        }
        if state.loading {
            return;
        }
        if self.members.get(&guild).is_none_or(Vec::is_empty) {
            state.loading = true;
            state.query.clear();
            self.send(Command::LoadMembers {
                guild,
                channel,
                offset: 0,
                query: String::new(),
                replace: true,
            });
        }
    }

    pub fn load_more_members(&mut self) {
        let guild = self.active_guild;
        let Some(state) = self.member_state.get(&guild).cloned() else {
            return;
        };
        if state.loading {
            return;
        }
        let Some(offset) = state.next_offset else {
            return;
        };
        if let Some(state) = self.member_state.get_mut(&guild) {
            state.loading = true;
        }
        self.send(Command::LoadMembers {
            guild,
            channel: state.channel,
            offset,
            query: state.query.clone(),
            replace: false,
        });
    }

    pub fn search_members(&mut self, query: String) {
        let guild = self.active_guild;
        let state = self.member_state.entry(guild).or_default();
        state.loading = true;
        state.query = query.clone();
        state.next_offset = None;
        state.complete = false;
        self.last_member_query = query.clone();
        self.send(Command::LoadMembers {
            guild,
            channel: self.active_channel,
            offset: 0,
            query,
            replace: true,
        });
    }

    pub fn switch_channel(&mut self, id: Id) {
        self.pinned_target = None;
        let current = match self.screen {
            Screen::Guild => self.active_channel,
            Screen::Dms => self.active_dm,
        };
        if current == id && !self.messages.get(&id).is_none_or(Vec::is_empty) {
            self.mark_read(id);
            return;
        }
        self.screen = Screen::Guild;
        self.active_channel = id;
        if self
            .channel(id)
            .is_some_and(|channel| channel.kind != ChannelKind::Thread)
        {
            self.thread_parent = None;
        }
        self.last_guild_channel.insert(self.active_guild, id);
        self.channel_seen.insert(id, now_ms());
        self.ensure_members(self.active_guild);
        self.ensure_messages(id);
        self.mark_read(id);
        self.trim_images();
    }

    pub fn open_dm(&mut self, id: Id) {
        self.screen = Screen::Dms;
        self.dm_home = false;
        self.active_dm = id;
        self.channel_seen.insert(id, now_ms());
        self.ensure_messages(id);
        self.mark_read(id);
        self.trim_images();
    }

    pub fn current_channel_id(&self) -> Id {
        match self.screen {
            Screen::Guild => self.active_channel,
            Screen::Dms => self.active_dm,
        }
    }

    pub fn ensure_messages(&mut self, id: Id) {
        self.channel_seen.insert(id, now_ms());
        if self.messages.get(&id).is_none_or(Vec::is_empty) {
            self.send(Command::LoadMessages {
                channel: id,
                before: None,
            });
        }
        self.request_channel_images(id);
        self.request_channel_users(id);
    }

    pub fn load_older(&mut self) {
        let id = self.current_channel_id();
        if id == 0 || self.loading_older.contains(&id) || self.older_complete.contains(&id) {
            return;
        }
        let before = self.messages.get(&id).and_then(|m| m.first()).map(|m| m.id);
        if let Some(before) = before.filter(|b| *b > 0) {
            self.loading_older.insert(id);
            self.send(Command::LoadMessages {
                channel: id,
                before: Some(before),
            });
        }
    }

    pub fn open_events(&mut self) {
        self.modal = Modal::Events;
        self.send(Command::LoadGuildEvents {
            guild: self.active_guild,
        });
    }

    pub fn open_create_event(&mut self) {
        self.event_name.clear();
        self.event_description.clear();
        self.event_start_minutes = TextEdit::new("60");
        self.modal = Modal::CreateEvent;
    }

    pub fn create_event_now(&mut self) {
        let name = self.event_name.text.trim().to_string();
        if name.is_empty() {
            self.toast("Name the event");
            return;
        }
        let minutes = self
            .event_start_minutes
            .text
            .trim()
            .parse::<i64>()
            .unwrap_or(60)
            .clamp(1, 60 * 24 * 30);
        let description = self.event_description.text.trim().to_string();
        self.send(Command::CreateGuildEvent {
            guild: self.active_guild,
            name,
            description,
            start_at: now_ms() + minutes * 60_000,
            channel: Some(self.current_channel_id()).filter(|id| *id != 0),
        });
        self.modal = Modal::Events;
    }

    pub fn delete_event(&mut self, event: Id) {
        self.send(Command::DeleteGuildEvent {
            guild: self.active_guild,
            event,
        });
    }

    pub fn open_inbox(&mut self) {
        self.modal = Modal::Inbox;
        self.send(Command::LoadInbox);
    }

    #[allow(dead_code)]
    pub fn refresh_inbox(&mut self) {
        self.send(Command::LoadInbox);
    }

    pub fn mark_inbox_read(&mut self, channel: Id, message: Id) {
        let removed = self.unread.remove(&channel).unwrap_or(0);
        if removed > 0
            && let Some(guild) = self.channel(channel).and_then(|c| c.guild_id)
            && let Some(summary) = self.guilds.iter_mut().find(|g| g.id == guild)
        {
            summary.unread_count = summary.unread_count.saturating_sub(removed);
        }
        self.reads.insert(channel, Some(message));
        self.send(Command::MarkRead { channel, message });
        if let Some(inbox) = &mut self.inbox {
            inbox.unreads.retain(|entry| entry.channel_id != channel);
            inbox.mentions.retain(|entry| entry.channel_id != channel);
        }
        self.mark_cache_dirty();
    }

    pub fn mark_all_inbox_read(&mut self) {
        let entries: Vec<(Id, Id)> = self
            .inbox
            .as_ref()
            .map(|inbox| {
                inbox
                    .unreads
                    .iter()
                    .map(|entry| (entry.channel_id, entry.message_id))
                    .collect()
            })
            .unwrap_or_default();
        for (channel, message) in entries {
            self.mark_inbox_read(channel, message);
        }
        self.toast("Inbox marked as read");
    }

    pub fn jump_to_inbox_entry(&mut self, guild: Option<Id>, channel: Id, message: Id) {
        self.modal = Modal::None;
        self.pinned_target = Some(message);
        if let Some(guild) = guild {
            self.screen = Screen::Guild;
            self.active_guild = guild;
            self.active_channel = channel;
            self.ensure_guild(guild);
        } else {
            self.screen = Screen::Dms;
            self.dm_home = false;
            self.active_dm = channel;
        }
        self.ensure_messages(channel);
        self.send(Command::LoadMessages {
            channel,
            before: None,
        });
        self.mark_inbox_read(channel, message);
    }

    pub fn mark_read(&mut self, channel: Id) {
        let removed = self.unread.remove(&channel).unwrap_or(0);
        if removed > 0
            && let Some(guild) = self.channel(channel).and_then(|c| c.guild_id)
            && let Some(summary) = self.guilds.iter_mut().find(|g| g.id == guild)
        {
            summary.unread_count = summary.unread_count.saturating_sub(removed);
        }
        let last = self
            .messages
            .get(&channel)
            .and_then(|m| m.last())
            .map(|m| m.id)
            .filter(|id| *id > 0);
        if let Some(last) = last {
            self.reads.insert(channel, Some(last));
            self.send(Command::MarkRead {
                channel,
                message: last,
            });
        }
        self.mark_cache_dirty();
    }
}
