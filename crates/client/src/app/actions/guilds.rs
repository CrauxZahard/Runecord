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
        let code = self.template_code.text.trim().to_string();
        if !code.is_empty() {
            self.send(Command::ApplyTemplate { code, name });
        } else {
            let color = 0x5865f2;
            self.send(Command::CreateGuild { name, color });
        }
        self.template_code = TextEdit::new("");
        self.modal = Modal::None;
    }

    pub fn open_create_channel(&mut self, parent: Option<Id>) {
        self.create_parent = parent;
        self.create_channel_kind = ChannelKind::Text;
        self.profile_edit.clear();
        self.modal = Modal::CreateChannel;
    }

    pub fn set_create_channel_kind(&mut self, kind: ChannelKind) {
        self.create_channel_kind = kind;
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

    pub fn create_webhook(&mut self) {
        let name = self.webhook_name.text.trim().to_string();
        if name.is_empty() {
            self.toast("Name the webhook");
            return;
        }
        let channel = if self.webhook_channel != 0 {
            self.webhook_channel
        } else {
            self.channels
                .get(&self.active_guild)
                .and_then(|list| {
                    list.iter()
                        .find(|channel| channel.kind == ChannelKind::Text)
                        .map(|channel| channel.id)
                })
                .unwrap_or(0)
        };
        if channel == 0 {
            self.toast("Pick a channel");
            return;
        }
        self.send(Command::CreateWebhook { channel, name });
        self.webhook_name = TextEdit::new("");
    }

    pub fn delete_webhook(&mut self, id: Id) {
        self.send(Command::DeleteWebhook {
            guild: self.active_guild,
            id,
        });
        self.webhook_token = None;
    }

    pub fn save_forum_tags(&mut self) {
        let channel = self.active_channel;
        let tags: Vec<ForumTag> = self
            .forum_tag_draft
            .text
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return None;
                }
                let (emoji, name) = match line.split_once(char::is_whitespace) {
                    Some((first, rest))
                        if first.chars().count() <= 2 && !rest.trim().is_empty() =>
                    {
                        (first.to_string(), rest.trim().to_string())
                    }
                    _ => (String::new(), line.to_string()),
                };
                Some(ForumTag {
                    id: 0,
                    channel_id: channel,
                    name,
                    emoji,
                    position: 0,
                })
            })
            .collect();
        self.send(Command::SetForumTags { channel, tags });
        self.toast("Forum tags saved");
    }

    pub fn toggle_thread_tag(&mut self, tag: Id) {
        if let Some(index) = self.thread_tag_ids.iter().position(|id| *id == tag) {
            self.thread_tag_ids.remove(index);
        } else if self.thread_tag_ids.len() >= 5 {
            self.toast("A post can have up to five tags");
        } else {
            self.thread_tag_ids.push(tag);
        }
    }

    pub fn create_template(&mut self) {
        let name = self.template_name.text.trim().to_string();
        if name.is_empty() {
            self.toast("Name the template");
            return;
        }
        self.send(Command::CreateTemplate {
            guild: self.active_guild,
            name,
            description: self.template_description.text.trim().to_string(),
        });
        self.template_name = TextEdit::new("");
        self.template_description = TextEdit::new("");
        self.toast("Template saved");
    }

    pub fn delete_template(&mut self, template: Id) {
        self.send(Command::DeleteTemplate {
            guild: self.active_guild,
            template,
        });
    }

    pub fn apply_template(&mut self, code: String) {
        self.template_code = TextEdit::new(code);
        self.profile_edit = TextEdit::new("");
        self.modal = Modal::CreateServer;
    }

    pub fn add_automod_rule(&mut self) {
        let name = self.automod_name.text.trim().to_string();
        if name.is_empty() {
            self.toast("Give the rule a name");
            return;
        }
        let pattern = self.automod_pattern.text.trim().to_string();
        if self.automod_trigger == "keyword" && pattern.is_empty() {
            self.toast("Add at least one keyword");
            return;
        }
        let timeout_minutes = self
            .automod_timeout
            .text
            .trim()
            .parse::<u32>()
            .unwrap_or(10)
            .clamp(1, 10_080);
        self.automod_rules.push(AutoModRule {
            id: 0,
            name,
            trigger: self.automod_trigger.clone(),
            pattern,
            action: self.automod_action.clone(),
            timeout_minutes,
            enabled: true,
        });
        self.automod_name = TextEdit::new("");
        self.automod_pattern = TextEdit::new("");
        self.save_automod_rules();
    }

    pub fn toggle_automod_rule(&mut self, index: usize) {
        if let Some(rule) = self.automod_rules.get_mut(index) {
            rule.enabled = !rule.enabled;
        }
        self.save_automod_rules();
    }

    pub fn delete_automod_rule(&mut self, index: usize) {
        if index < self.automod_rules.len() {
            self.automod_rules.remove(index);
        }
        self.save_automod_rules();
    }

    fn save_automod_rules(&mut self) {
        self.send(Command::SaveAutoMod {
            guild: self.active_guild,
            rules: self.automod_rules.clone(),
        });
    }

    pub fn open_user_volume(&mut self, user: Id) {
        self.modal = Modal::UserVolume(user);
    }

    pub fn set_user_volume(&mut self, user: Id, volume: f32) {
        let muted = crate::voice::user_muted(user);
        crate::voice::set_user_volume(user, volume, muted);
    }

    pub fn toggle_user_volume_mute(&mut self, user: Id) {
        let volume = crate::voice::user_volume(user);
        let muted = crate::voice::user_muted(user);
        crate::voice::set_user_volume(user, volume, !muted);
    }

    pub fn set_presence_auto(&mut self, enabled: bool) {
        self.presence_auto = enabled;
        self.mark_cache_dirty();
        if !enabled && !self.auto_presence.is_empty() {
            self.auto_presence.clear();
            self.clear_profile_activity();
        }
    }

    /// Publishes the detected game as the activity, but never overrides an
    /// activity the user set by hand.
    pub fn refresh_auto_presence(&mut self) {
        if !self.presence_auto || self.my_presence == Presence::Offline {
            return;
        }
        if let Some(activity) = &self.me.activity
            && (self.auto_presence.is_empty() || activity.title != self.auto_presence)
        {
            // A hand-written activity wins until it is cleared.
            return;
        }
        let detected = crate::presence_scan::detect_activity();
        let title = detected
            .map(|(_, title)| title.to_string())
            .unwrap_or_default();
        if title == self.auto_presence {
            return;
        }
        self.auto_presence = title.clone();
        if title.is_empty() {
            self.clear_profile_activity();
        } else {
            let kind = detected.map(|(kind, _)| kind).unwrap_or("playing");
            self.profile_activity_kind = TextEdit::new(kind);
            self.profile_activity_title = TextEdit::new(title);
            self.save_profile_activity();
        }
    }

    pub fn set_overlay_enabled(&mut self, enabled: bool) {
        self.overlay_enabled = enabled;
        self.mark_cache_dirty();
        self.sync_overlay();
    }

    pub fn save_onboarding(&mut self) {
        let rules = self
            .onboarding_rules
            .text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect();
        self.send(Command::UpdateOnboarding {
            guild: self.active_guild,
            update: UpdateOnboarding {
                welcome_enabled: self.onboarding_welcome,
                welcome_title: self.onboarding_title.text.trim().to_string(),
                welcome_description: self.onboarding_description.text.trim().to_string(),
                rules,
                verification_level: self.onboarding_verification,
            },
        });
        self.toast("Onboarding saved");
    }

    pub fn accept_onboarding(&mut self, guild: Id) {
        self.send(Command::AcceptOnboarding { guild });
        self.modal = Modal::None;
    }

    pub fn open_channel_permissions(&mut self, channel: Id) {
        self.perm_target = 0;
        self.perm_target_kind = "role".into();
        self.modal = Modal::ChannelPermissions(channel);
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
        let tag_lines: Vec<String> = self
            .forum_tags
            .iter()
            .filter(|tag| tag.channel_id == id)
            .map(|tag| {
                if tag.emoji.is_empty() {
                    tag.name.clone()
                } else {
                    format!("{} {}", tag.emoji, tag.name)
                }
            })
            .collect();
        self.forum_tag_draft = TextEdit::new(tag_lines.join("\n"));
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
