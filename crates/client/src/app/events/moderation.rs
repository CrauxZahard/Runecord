//! Network event handlers: moderation.

use super::super::*;
use crate::net::Command;

impl App {
    pub(crate) fn apply_bans(&mut self, bans: Vec<Ban>) {
        self.bans = bans;
    }
    pub(crate) fn apply_prune_preview(&mut self, days: u32, count: u32) {
        self.prune_loading = false;
        if days == self.prune_days {
            self.prune_preview = Some(count);
        }
    }
    pub(crate) fn apply_appeals(&mut self, appeals: Vec<BanAppeal>) {
        self.appeals = appeals;
    }
    pub(crate) fn apply_appeal_resolved(&mut self, appeal: Id, accepted: bool) {
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
    pub(crate) fn apply_appeal_submitted(&mut self) {
        self.modal = Modal::None;
        self.toast("Appeal submitted. A server manager will review it.");
        self.last_join_code.clear();
    }
    pub(crate) fn apply_pruned(&mut self, count: u32) {
        self.toast(if count == 1 {
            "1 member pruned".to_string()
        } else {
            format!("{count} members pruned")
        });
    }
    pub(crate) fn apply_unbanned(&mut self, guild: Id, user: Id) {
        if guild == self.active_guild {
            self.bans.retain(|ban| ban.user.id != user);
        }
        self.toast("User unbanned");
    }
}
