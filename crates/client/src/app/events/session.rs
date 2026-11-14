//! Network event handlers: session.

use super::super::*;
use crate::net::Command;

impl App {
    pub(crate) fn apply_session(&mut self, auth: Box<AuthResponse>) {
        self.me = auth.user;
        self.logged_in = true;
        self.auth_busy = false;
        self.auth_error = None;
        self.connected = true;
    }
    pub(crate) fn apply_bootstrap_event(&mut self, b: Box<Bootstrap>) {
        self.apply_bootstrap(*b, true);
        self.mark_cache_dirty();
    }
    pub(crate) fn apply_token_revealed(&mut self, label: String, token: String) {
        self.token_reveal = Some((label, token));
    }
    pub(crate) fn apply_server_checked(&mut self, url: String, ok: bool, error: Option<String>) {
        self.apply_checked_server(url, ok, error);
    }
    pub(crate) fn apply_connection_failed(&mut self) {
        self.server_unreachable = true;
        self.connected = false;
    }
    pub(crate) fn apply_user_update(&mut self, user: Box<User>) {
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
    pub(crate) fn apply_notice(&mut self, text: String) {
        self.toast(text)
    }
    pub(crate) fn apply_connected(&mut self, up: bool) {
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
    pub(crate) fn apply_error(&mut self, msg: String) {
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
