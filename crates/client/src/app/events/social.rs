//! Network event handlers: social.

use super::super::*;
use crate::net::Command;

impl App {
    pub(crate) fn apply_users(&mut self, users: Vec<User>) {
        let urls: Vec<String> = users
            .iter()
            .flat_map(|user| [user.avatar_url.clone(), user.banner_url.clone()])
            .filter(|url| !url.is_empty())
            .collect();
        for mut user in users {
            self.user_requests.remove(&user.id);
            if user.id == self.me.id {
                user.status = if self.connected {
                    self.my_presence
                } else {
                    Presence::Offline
                };
            }
            self.presence.insert(user.id, user.status);
            self.users.insert(user.id, user);
        }
        self.request_urls(urls);
    }
    pub(crate) fn apply_lookup(&mut self, users: Vec<User>) {
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
    pub(crate) fn apply_inbox(&mut self, inbox: Inbox) {
        if self.modal == Modal::Inbox {
            if inbox.mentions.is_empty() && !inbox.unreads.is_empty() {
                self.inbox_tab = 1;
            } else if !inbox.mentions.is_empty() && inbox.unreads.is_empty() {
                self.inbox_tab = 0;
            }
        }
        self.inbox = Some(inbox);
    }
    pub(crate) fn apply_mutuals(&mut self, user: Id, guilds: Vec<GuildSummary>) {
        self.mutuals.insert(user, guilds);
    }
    pub(crate) fn apply_dm(&mut self, dm: Box<DmChannel>) {
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
    pub(crate) fn apply_dm_updated(&mut self, dm: Box<DmChannel>) {
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
    pub(crate) fn apply_dm_removed(&mut self, channel: Id) {
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
}
