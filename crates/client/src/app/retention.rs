//! Cache retention.

use super::*;

impl App {
    pub(crate) fn trim_channel(&mut self, channel: Id) -> bool {
        if let Some(list) = self.messages.get_mut(&channel)
            && list.len() > crate::cache::MAX_MESSAGES_PER_CHANNEL
        {
            let drop = list.len() - crate::cache::MAX_MESSAGES_PER_CHANNEL;
            list.drain(..drop);
            return true;
        }
        false
    }

    pub(crate) fn trim_guild_caches(&mut self) {
        let active = self.active_guild;
        while self.guild_details.len() > crate::cache::MAX_GUILD_DETAILS {
            let Some(id) = self
                .guild_details
                .keys()
                .copied()
                .filter(|id| *id != active)
                .min_by_key(|id| self.guild_seen.get(id).copied().unwrap_or(0))
            else {
                break;
            };
            self.guild_details.remove(&id);
            self.channels.remove(&id);
            self.guild_seen.remove(&id);
        }
    }

    pub(crate) fn trim_member_caches(&mut self) {
        let active = self.active_guild;
        while self.member_state.len() > crate::cache::MAX_MEMBER_GUILDS {
            let Some(id) = self
                .member_state
                .keys()
                .copied()
                .filter(|id| *id != active)
                .min_by_key(|id| self.member_seen.get(id).copied().unwrap_or(0))
            else {
                break;
            };
            self.members.remove(&id);
            self.member_state.remove(&id);
            self.member_seen.remove(&id);
        }
    }

    pub(crate) fn maintenance(&mut self) -> bool {
        self.last_maintenance = self.clock;
        let mut changed = false;
        let channels: Vec<Id> = self.messages.keys().copied().collect();
        for channel in channels {
            changed |= self.trim_channel(channel);
        }

        let mut total: usize = self.messages.values().map(Vec::len).sum();
        if total > crate::cache::MAX_TOTAL_MESSAGES {
            let current = self.current_channel_id();
            let mut ids: Vec<Id> = self.messages.keys().copied().collect();
            ids.sort_by_key(|id| self.channel_seen.get(id).copied().unwrap_or(0));
            for id in ids {
                if total <= crate::cache::MAX_TOTAL_MESSAGES {
                    break;
                }
                if id == current {
                    continue;
                }
                if let Some(list) = self.messages.remove(&id) {
                    total = total.saturating_sub(list.len());
                    changed = true;
                }
            }
        }

        changed |= self.trim_images();
        self.trim_guild_caches();
        self.trim_member_caches();
        let current = self.current_channel_id();
        if current != 0 {
            self.request_channel_images(current);
        }
        changed
    }
}
