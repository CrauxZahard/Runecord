//! Network event handlers: roles.

use super::super::*;
use crate::net::Command;

impl App {
    pub(crate) fn apply_role_created(&mut self, role: Role) {
        if !role.icon_url.is_empty() {
            self.request_urls(vec![role.icon_url.clone()]);
        }
        self.apply_role(role.clone());
        self.select_role(role.id);
    }
    pub(crate) fn apply_role_links(&mut self, role: Id, links: Vec<RoleLink>) {
        self.role_links.insert(role, links);
    }
    pub(crate) fn apply_role_updated(&mut self, role: Role) {
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
    pub(crate) fn apply_role_deleted(&mut self, guild: Id, role: Id) {
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
    pub(crate) fn apply_emojis(&mut self, emojis: Vec<Emoji>) {
        self.emojis = emojis;
        let urls: Vec<String> = self
            .emojis
            .iter()
            .filter(|emoji| !self.images.contains_key(&emoji.url))
            .map(|emoji| emoji.url.clone())
            .collect();
        self.request_urls(urls);
    }
    pub(crate) fn apply_emoji_created(&mut self, emoji: Emoji) {
        self.emojis
            .retain(|item| item.id != emoji.id && item.name != emoji.name);
        let url = emoji.url.clone();
        self.emojis.push(emoji);
        self.request_urls(vec![url]);
        self.toast("Emoji added");
    }
    pub(crate) fn apply_emoji_deleted(&mut self, id: Id) {
        self.emojis.retain(|emoji| emoji.id != id);
        self.toast("Emoji deleted");
    }
    pub(crate) fn apply_audit_page(&mut self, entries: Vec<AuditEntry>, older: bool) {
        self.audit_loading = false;
        if older {
            self.audit.extend(entries.iter().cloned());
        } else {
            self.audit = entries.clone();
        }
        self.audit_complete = entries.len() < 50;
        self.audit_before = self.audit.last().map(|entry| entry.id);
    }
    pub(crate) fn apply_guild_commands(&mut self, commands: Vec<ApplicationCommand>) {
        self.commands = commands;
    }
}
