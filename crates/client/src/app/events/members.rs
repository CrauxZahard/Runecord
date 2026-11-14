//! Network event handlers: members.

use super::super::*;

impl App {
    pub(crate) fn apply_member_update(&mut self, member: Box<Member>) {
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
        if member_user_is_self
            && let Some(existing) = self
                .members
                .get(&guild)
                .and_then(|list| list.iter().find(|m| m.user.id == self.me.id))
        {
            self.me = existing.user.clone();
        }
    }
    pub(crate) fn apply_member_remove(&mut self, guild: Id, user: Id) {
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
    pub(crate) fn apply_member_count_update(&mut self, guild: Id, count: u32) {
        if let Some(summary) = self.guilds.iter_mut().find(|g| g.id == guild) {
            summary.member_count = count;
        }
        if let Some(state) = self.member_state.get_mut(&guild) {
            state.total = count;
        }
    }
}
