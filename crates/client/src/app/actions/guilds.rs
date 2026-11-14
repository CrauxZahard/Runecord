//! App actions split by feature.

use super::super::*;
use super::timeout_seconds;
use crate::net::Command;
use rune::TextEdit;

impl App {
    pub fn create_guild_now(&mut self) {
        let name = self.profile_edit.text.trim().to_string();
        if name.is_empty() {
            return;
        }
        let color = 0x5865f2;
        self.send(Command::CreateGuild { name, color });
        self.modal = Modal::None;
    }

    pub fn create_channel_now(&mut self, kind: ChannelKind) {
        let name = self.profile_edit.text.trim().to_string();
        if name.is_empty() {
            return;
        }
        let parent = self.create_parent.take();
        self.send(Command::CreateChannel {
            guild: self.active_guild,
            name,
            kind,
            parent,
            topic: String::new(),
        });
        self.modal = Modal::None;
    }

    pub fn open_channel_edit(&mut self, id: Id) {
        let Some(channel) = self.channel(id).cloned() else {
            return;
        };
        self.editing_channel = id;
        self.channel_name_edit = TextEdit::new(channel.name);
        self.channel_topic_edit = TextEdit::new(channel.topic);
        self.channel_status_edit = TextEdit::new(channel.status);
        self.channel_slowmode_edit = TextEdit::new(channel.slowmode.to_string());
        self.channel_nsfw_edit = channel.nsfw;
        self.modal = Modal::EditChannel(id);
    }

    pub fn save_channel_edit(&mut self) {
        let name = self
            .channel_name_edit
            .text
            .trim()
            .to_lowercase()
            .replace(' ', "-");
        if name.is_empty() {
            self.toast("Channel name cannot be empty");
            return;
        }
        let slowmode = self
            .channel_slowmode_edit
            .text
            .trim()
            .parse::<u32>()
            .unwrap_or(0)
            .min(21_600);
        self.send(Command::UpdateChannel {
            channel: self.editing_channel,
            name,
            topic: self.channel_topic_edit.text.trim().to_string(),
            status: self.channel_status_edit.text.trim().to_string(),
            slowmode,
            nsfw: self.channel_nsfw_edit,
        });
        self.modal = Modal::None;
        self.toast("Channel updated");
    }

    pub fn confirm_delete_channel(&mut self, id: Id) {
        self.editing_channel = id;
        self.modal = Modal::ConfirmDeleteChannel(id);
    }

    pub fn confirm_leave_guild(&mut self) {
        self.modal = Modal::ConfirmLeave(self.active_guild);
    }

    pub fn confirm_kick(&mut self, user: Id) {
        self.modal = Modal::ConfirmKick {
            guild: self.active_guild,
            user,
        };
    }

    pub fn confirm_ban(&mut self, user: Id) {
        self.ban_reason = TextEdit::new("");
        self.modal = Modal::ConfirmBan {
            guild: self.active_guild,
            user,
        };
    }

    pub fn open_timeout(&mut self, user: Id) {
        self.timeout_amount = TextEdit::new("10");
        self.timeout_unit = TimeoutUnit::Minutes;
        self.modal = Modal::TimeoutMember {
            guild: self.active_guild,
            user,
        };
    }

    pub fn set_timeout_unit(&mut self, unit: TimeoutUnit) {
        self.timeout_unit = unit;
    }

    pub fn apply_timeout(&mut self) {
        let Ok(value) = self.timeout_amount.text.trim().parse::<u64>() else {
            self.toast("Enter a whole number of minutes, hours or days");
            return;
        };
        if value == 0 {
            self.toast("Timeout must be at least one unit");
            return;
        }
        let unit = self.timeout_unit;
        let seconds = timeout_seconds(value, unit);
        self.timeout_now(seconds);
    }

    pub fn open_bans(&mut self) {
        self.bans.clear();
        self.appeals.clear();
        self.modal = Modal::Bans;
        self.send(Command::LoadBans {
            guild: self.active_guild,
        });
        self.send(Command::LoadAppeals {
            guild: self.active_guild,
        });
    }

    pub fn unban(&mut self, user: Id) {
        self.send(Command::UnbanMember {
            guild: self.active_guild,
            user,
        });
    }

    pub fn open_invite_join(&mut self) {
        self.join_code = TextEdit::new("");
        self.modal = Modal::InviteJoin;
    }

    pub fn join_invite_now(&mut self) {
        let raw = self.join_code.text.trim().trim_end_matches('/');
        let code = raw.rsplit('/').next().unwrap_or(raw).to_string();
        if code.is_empty() {
            self.toast("Paste an invite code or link");
            return;
        }
        self.last_join_code = code.clone();
        self.send(Command::JoinInvite { code });
    }

    pub fn open_appeal(&mut self, code: String) {
        self.appeal_code = code;
        self.appeal_message = TextEdit::new("");
        self.modal = Modal::BanAppeal;
    }

    pub fn submit_appeal(&mut self) {
        let message = self.appeal_message.text.trim().to_string();
        if message.is_empty() {
            self.toast("Explain why the ban should be lifted");
            return;
        }
        self.send(Command::SubmitAppeal {
            code: self.appeal_code.clone(),
            message,
        });
    }

    pub fn resolve_appeal(&mut self, appeal: Id, accept: bool) {
        self.send(Command::ResolveAppeal {
            guild: self.active_guild,
            appeal,
            accept,
        });
    }

    pub fn open_prune(&mut self) {
        self.prune_days = 7;
        self.prune_roles.clear();
        self.prune_preview = None;
        self.prune_loading = false;
        self.modal = Modal::PruneMembers;
    }

    pub fn prune_days_delta(&mut self, delta: i32) {
        let days = (self.prune_days as i32 + delta).clamp(1, 30) as u32;
        if days != self.prune_days {
            self.prune_days = days;
            self.prune_preview = None;
        }
    }

    pub fn toggle_prune_role(&mut self, role: Id) {
        if !self.prune_roles.insert(role) {
            self.prune_roles.remove(&role);
        }
        self.prune_preview = None;
    }

    pub fn preview_prune(&mut self) {
        if self.prune_loading {
            return;
        }
        self.prune_loading = true;
        self.prune_preview = None;
        self.send(Command::PrunePreview {
            guild: self.active_guild,
            days: self.prune_days,
            roles: self.prune_roles.iter().copied().collect(),
        });
    }

    pub fn open_video_quality(&mut self) {
        self.modal = Modal::VideoQuality;
    }
}
