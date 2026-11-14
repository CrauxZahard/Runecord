//! User actions.

use super::*;
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

    pub fn refresh_inbox(&mut self) {
        self.send(Command::LoadInbox);
    }

    pub fn mark_inbox_read(&mut self, channel: Id, message: Id) {
        let removed = self.unread.remove(&channel).unwrap_or(0);
        if removed > 0 {
            if let Some(guild) = self.channel(channel).and_then(|c| c.guild_id) {
                if let Some(summary) = self.guilds.iter_mut().find(|g| g.id == guild) {
                    summary.unread_count = summary.unread_count.saturating_sub(removed);
                }
            }
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
        if removed > 0 {
            if let Some(guild) = self.channel(channel).and_then(|c| c.guild_id) {
                if let Some(summary) = self.guilds.iter_mut().find(|g| g.id == guild) {
                    summary.unread_count = summary.unread_count.saturating_sub(removed);
                }
            }
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

    pub fn send_current(&mut self) {
        let text = self.composer.text.trim().to_string();
        if text.is_empty() && self.attachments.is_empty() {
            return;
        }
        let channel = match self.screen {
            Screen::Guild => self.active_channel,
            Screen::Dms => self.active_dm,
        };
        if channel == 0 {
            return;
        }
        if let Some(command_text) = text.strip_prefix('/') {
            let mut parts = command_text.splitn(2, char::is_whitespace);
            let name = parts.next().unwrap_or_default();
            let args = parts.next().unwrap_or_default().trim().to_string();
            let Some(command) = self
                .commands
                .iter()
                .find(|command| command.name.eq_ignore_ascii_case(name))
                .cloned()
            else {
                self.toast(format!("Unknown command /{name}"));
                return;
            };
            let mut response = command
                .options
                .get("response")
                .and_then(|value| value.as_str())
                .unwrap_or("Command ran.")
                .to_string();
            if command.name == "echo" && !args.is_empty() {
                response = args.clone();
            }
            self.send(Command::InvokeCommand {
                channel,
                command: command.name.clone(),
                options: serde_json::json!({ "text": args }),
                response,
            });
            self.composer.clear();
            self.attachments.clear();
            return;
        }
        let now = now_ms();
        if self
            .channel(channel)
            .is_some_and(|channel| channel.kind == ChannelKind::Forum)
        {
            self.pending_forum = Some(channel);
        } else {
            let temp = Message {
                id: -now,
                channel_id: channel,
                author_id: self.me.id,
                content: text.clone(),
                timestamp: now,
                attachments: self.attachments.clone(),
                mentions: Vec::new(),
                pending: true,
                ..Message::default()
            };
            self.messages.entry(channel).or_default().push(temp);
        }
        self.send(Command::Send {
            channel,
            content: text,
            reply_to: self.reply_to.as_ref().map(|r| r.message_id),
            attachments: self.attachments.iter().map(|a| a.id).collect(),
            nonce: format!("{now}"),
        });
        self.reply_to = None;
        self.composer.clear();
        self.attachments.clear();
        self.mark_read(channel);
    }

    pub fn upload_file(&mut self, path: std::path::PathBuf) {
        let channel = match self.screen {
            Screen::Guild => self.active_channel,
            Screen::Dms => self.active_dm,
        };
        let limit = match self.screen {
            Screen::Guild => self
                .active_guild()
                .map(|guild| guild.upload_limit_bytes)
                .filter(|limit| *limit > 0)
                .unwrap_or(48 * 1024 * 1024),
            Screen::Dms => 48 * 1024 * 1024,
        };
        if let Ok(meta) = std::fs::metadata(&path) {
            if meta.len() > limit {
                self.toast(format!(
                    "File is larger than {} MB. Boost the server to raise the limit.",
                    limit / (1024 * 1024)
                ));
                return;
            }
        }
        if channel == 0 {
            return;
        }
        if let Some(name) = path.file_name() {
            self.toast(format!("Uploading {}", name.to_string_lossy()));
        }
        self.send(Command::Upload { channel, path });
    }

    pub fn note_typing(&mut self) {
        if self.clock - self.last_typing < 4.0 {
            return;
        }
        self.last_typing = self.clock;
        let channel = match self.screen {
            Screen::Guild => self.active_channel,
            Screen::Dms => self.active_dm,
        };
        if channel != 0 {
            self.send(Command::Typing { channel });
        }
    }

    pub fn toast(&mut self, text: impl Into<String>) {
        self.toast = Some(Toast {
            text: text.into(),
            until: self.clock + 4.0,
        });
    }

    pub fn set_reply(&mut self, message: Id, author_id: Id, content: &str) {
        self.reply_to = Some(MessageReply {
            message_id: message,
            author_id,
            author_name: self.user_name(author_id),
            content: content.to_string(),
        });
    }

    pub fn open_edit(&mut self, message: Id, content: &str) {
        self.edit_text = TextEdit::new(content.to_string());
        self.modal = Modal::EditMessage(message);
    }

    pub fn save_edit(&mut self) {
        if let Modal::EditMessage(id) = self.modal {
            let content = self.edit_text.text.clone();
            self.send(Command::Edit {
                message: id,
                content,
            });
            self.modal = Modal::None;
            self.mark_cache_dirty();
        }
    }

    pub fn toggle_pin(&mut self, id: Id) {
        if let Some(net) = &self.net {
            net.send(Command::Pin { message: id });
        }
        self.mark_cache_dirty();
    }

    pub fn open_pins(&mut self) {
        let channel = match self.screen {
            Screen::Guild => self.active_channel,
            Screen::Dms => self.active_dm,
        };
        if channel == 0 {
            return;
        }
        self.pinned.clear();
        self.modal = Modal::Pins;
        self.send(Command::LoadPins { channel });
    }

    pub fn open_privacy(&mut self) {
        self.modal = Modal::Privacy;
    }

    pub fn set_privacy(&mut self, dm: PrivacyLevel, friend: PrivacyLevel) {
        self.me.dm_privacy = dm;
        self.me.friend_privacy = friend;
        self.send(Command::SetPrivacy { dm, friend });
    }

    pub fn toggle_guild_discovery(&mut self) {
        let guild = self.active_guild;
        let current = self
            .active_guild()
            .map(|summary| summary.discoverable)
            .unwrap_or(true);
        let next = !current;
        if let Some(summary) = self.guilds.iter_mut().find(|item| item.id == guild) {
            summary.discoverable = next;
        }
        if let Some(detail) = self.guild_details.get_mut(&guild) {
            detail.guild.discoverable = next;
        }
        self.send(Command::SetGuildDiscovery {
            guild,
            discoverable: next,
        });
    }

    pub fn jump_to_pinned(&mut self, message: Id, channel: Id) {
        self.pinned_target = Some(message);
        self.modal = Modal::None;
        if let Some(target) = self.channel(channel).cloned() {
            if let Some(guild) = target.guild_id {
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
        }
    }

    pub fn delete_message(&mut self, id: Id) {
        if let Some(net) = &self.net {
            net.send(Command::Delete { message: id });
        } else if let Some(list) = self.messages.get_mut(&self.active_channel) {
            list.retain(|m| m.id != id);
        }
        self.mark_cache_dirty();
    }

    pub fn react(&mut self, message: Id, emoji: &str) {
        if let Some(net) = &self.net {
            net.send(Command::React {
                message,
                emoji: emoji.to_string(),
            });
        } else if let Some(list) = self.messages.get_mut(&self.active_channel) {
            if let Some(m) = list.iter_mut().find(|m| m.id == message) {
                if let Some(r) = m.reactions.iter_mut().find(|r| r.emoji == emoji) {
                    r.me = !r.me;
                    r.count = if r.me {
                        r.count + 1
                    } else {
                        r.count.saturating_sub(1)
                    };
                } else {
                    m.reactions.push(Reaction {
                        emoji: emoji.to_string(),
                        count: 1,
                        me: true,
                        user_ids: vec![self.me.id],
                    });
                }
            }
        }
        self.mark_cache_dirty();
    }

    pub fn run_search(&mut self) {
        let q = self.search.text.trim().to_string();
        if q.is_empty() {
            return;
        }
        self.send(Command::Search { q });
    }

    pub fn append_search_filter(&mut self, token: &str) {
        let current = self.search.text.trim_end();
        let next = if current.is_empty() {
            token.to_string()
        } else {
            format!("{current} {token}")
        };
        self.search = TextEdit::new(next);
    }

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

    pub fn begin_user_image_crop(&mut self, path: std::path::PathBuf, banner: bool) {
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                self.toast(format!("Could not read image: {error}"));
                return;
            }
        };
        if crate::image_anim::is_animated(&bytes) {
            self.send(Command::UploadUserImage { banner, path });
            self.toast(if banner {
                "Uploading animated banner."
            } else {
                "Uploading animated avatar."
            });
            return;
        }
        let decoded = match image::load_from_memory(&bytes) {
            Ok(image) => image,
            Err(error) => {
                self.toast(format!("Could not decode image: {error}"));
                return;
            }
        };
        let rgba = decoded.to_rgba8();
        let (width, height) = rgba.dimensions();
        if width < 32 || height < 32 {
            self.toast("Image is too small");
            return;
        }
        self.crop_source = Some(CropSource { image: rgba });
        self.crop_banner = banner;
        self.crop_user = true;
        self.crop_dm = 0;
        self.crop_zoom = 1.0;
        self.crop_offset = (0.0, 0.0);
        self.crop_drag_origin = (0.0, 0.0);
        self.modal = Modal::CropUserImage { banner };
        self.refresh_crop_preview();
    }

    pub fn remove_user_image(&mut self, banner: bool) {
        self.send(Command::RemoveUserImage { banner });
        self.toast(if banner {
            "Profile banner removed"
        } else {
            "Profile avatar removed"
        });
    }

    pub fn begin_guild_image_crop(&mut self, path: std::path::PathBuf, banner: bool) {
        self.crop_dm = 0;
        if banner
            && self
                .active_guild()
                .map(|guild| guild.boost_level)
                .unwrap_or(0)
                < 2
        {
            self.toast("Server banners unlock at Boost Level 2 (7 Boosts)");
            return;
        }
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                self.toast(format!("Could not read image: {error}"));
                return;
            }
        };
        if crate::image_anim::is_animated(&bytes) {
            self.send(Command::UploadGuildImage {
                guild: self.active_guild,
                banner,
                path,
            });
            self.toast("Uploading animated server image.");
            return;
        }
        let decoded = match image::load_from_memory(&bytes) {
            Ok(image) => image,
            Err(error) => {
                self.toast(format!("Could not decode image: {error}"));
                return;
            }
        };
        let rgba = decoded.to_rgba8();
        let (width, height) = rgba.dimensions();
        if width < 32 || height < 32 {
            self.toast("Image is too small");
            return;
        }
        self.crop_source = Some(CropSource { image: rgba });
        self.crop_banner = banner;
        self.crop_zoom = 1.0;
        self.crop_offset = (0.0, 0.0);
        self.crop_drag_origin = (0.0, 0.0);
        self.modal = Modal::CropGuildImage { banner };
        self.refresh_crop_preview();
    }

    fn crop_window(&self) -> Option<(f32, f32, f32, f32)> {
        let source = self.crop_source.as_ref()?;
        let width = source.image.width() as f32;
        let height = source.image.height() as f32;
        let aspect = if self.crop_banner { 16.0 / 9.0 } else { 1.0 };
        let crop_width = width.min(height * aspect) / self.crop_zoom.clamp(1.0, 3.0);
        let crop_height = crop_width / aspect;
        let max_x = ((width - crop_width) / 2.0).max(0.0);
        let max_y = ((height - crop_height) / 2.0).max(0.0);
        let center_x =
            (width / 2.0 + self.crop_offset.0).clamp(crop_width / 2.0, width - crop_width / 2.0);
        let center_y = (height / 2.0 + self.crop_offset.1)
            .clamp(crop_height / 2.0, height - crop_height / 2.0);
        let _ = (max_x, max_y);
        Some((
            center_x - crop_width / 2.0,
            center_y - crop_height / 2.0,
            crop_width,
            crop_height,
        ))
    }

    fn clamp_crop_offset(&mut self) {
        let Some(source) = self.crop_source.as_ref() else {
            return;
        };
        let width = source.image.width() as f32;
        let height = source.image.height() as f32;
        let Some((_, _, crop_width, crop_height)) = self.crop_window() else {
            return;
        };
        let max_x = ((width - crop_width) / 2.0).max(0.0);
        let max_y = ((height - crop_height) / 2.0).max(0.0);
        self.crop_offset.0 = self.crop_offset.0.clamp(-max_x, max_x);
        self.crop_offset.1 = self.crop_offset.1.clamp(-max_y, max_y);
    }

    fn render_crop(&self, out_width: u32, out_height: u32) -> Option<Vec<u8>> {
        let source = self.crop_source.as_ref()?;
        let (x, y, width, height) = self.crop_window()?;
        let image_width = source.image.width();
        let image_height = source.image.height();
        let x = x.round().clamp(0.0, (image_width - 1) as f32) as u32;
        let y = y.round().clamp(0.0, (image_height - 1) as f32) as u32;
        let width = (width.round() as u32).clamp(1, image_width - x);
        let height = (height.round() as u32).clamp(1, image_height - y);
        let cropped = image::imageops::crop_imm(&source.image, x, y, width, height).to_image();
        let resized = image::imageops::resize(
            &cropped,
            out_width,
            out_height,
            image::imageops::FilterType::Triangle,
        );
        Some(resized.into_raw())
    }

    fn refresh_crop_preview(&mut self) {
        let (width, height) = if self.crop_banner {
            (400u32, 225u32)
        } else {
            (320u32, 320u32)
        };
        if let Some(rgba) = self.render_crop(width, height) {
            let pixels = rune::Pixels::new();
            pixels.set(width, height, rgba);
            self.crop_preview = Some(pixels);
        }
    }

    pub fn crop_zoom_by(&mut self, delta: f32) {
        self.crop_zoom = (self.crop_zoom + delta).clamp(1.0, 3.0);
        self.clamp_crop_offset();
        self.refresh_crop_preview();
    }

    pub fn crop_pan(&mut self, dx: f32, dy: f32) {
        self.clamp_crop_offset();
        self.crop_offset.0 += dx;
        self.crop_offset.1 += dy;
        self.clamp_crop_offset();
        self.refresh_crop_preview();
    }

    pub fn crop_reset(&mut self) {
        self.crop_zoom = 1.0;
        self.crop_offset = (0.0, 0.0);
        self.refresh_crop_preview();
    }

    pub fn crop_drag_scale(&self) -> f32 {
        let Some((_, _, width, _)) = self.crop_window() else {
            return 1.0;
        };
        width / if self.crop_banner { 400.0 } else { 320.0 }
    }

    pub fn crop_drag_move(&mut self, dx: f32, dy: f32, origin: (f32, f32)) {
        let scale = self.crop_drag_scale();
        self.crop_offset = (origin.0 - dx * scale, origin.1 - dy * scale);
        self.clamp_crop_offset();
        self.refresh_crop_preview();
    }

    pub fn confirm_guild_image_crop(&mut self) {
        let (width, height) = if self.crop_banner {
            (960u32, 540u32)
        } else {
            (512u32, 512u32)
        };
        let Some(rgba) = self.render_crop(width, height) else {
            self.toast("Nothing to crop");
            return;
        };
        let path =
            std::env::temp_dir().join(format!("runecord-crop-{}.png", runecord_protocol::now_ms()));
        if image::save_buffer(&path, &rgba, width, height, image::ColorType::Rgba8).is_err() {
            self.toast("Could not prepare the image");
            return;
        }
        let banner = self.crop_banner;
        let user = self.crop_user;
        let dm = self.crop_dm;
        self.crop_source = None;
        self.crop_preview = None;
        self.crop_user = false;
        self.crop_dm = 0;
        if dm != 0 {
            self.modal = Modal::DmSettings { channel: dm };
            self.send(Command::UploadDmIcon { channel: dm, path });
            self.toast("Uploading group icon.");
        } else if user {
            self.modal = Modal::Settings(SettingsTab::Profile);
            self.sync_profile_edits();
            self.send(Command::UploadUserImage { banner, path });
            self.toast(if banner {
                "Uploading profile banner."
            } else {
                "Uploading profile avatar."
            });
        } else {
            self.server_tab = ServerTab::Overview;
            self.modal = Modal::ServerSettings;
            self.upload_guild_image(path, banner);
        }
    }

    pub fn cancel_guild_image_crop(&mut self) {
        let user = self.crop_user;
        let dm = self.crop_dm;
        self.crop_source = None;
        self.crop_preview = None;
        self.crop_user = false;
        self.crop_dm = 0;
        if dm != 0 {
            self.modal = Modal::DmSettings { channel: dm };
        } else if user {
            self.modal = Modal::Settings(SettingsTab::Profile);
        } else {
            self.server_tab = ServerTab::Overview;
            self.modal = Modal::ServerSettings;
        }
    }

    pub fn upload_guild_image(&mut self, path: std::path::PathBuf, banner: bool) {
        self.send(Command::UploadGuildImage {
            guild: self.active_guild,
            banner,
            path,
        });
        self.toast(if banner {
            "Uploading server banner…"
        } else {
            "Uploading server icon…"
        });
    }

    pub fn remove_guild_image(&mut self, banner: bool) {
        self.send(Command::RemoveGuildImage {
            guild: self.active_guild,
            banner,
        });
        self.toast(if banner {
            "Server banner removed"
        } else {
            "Server icon removed"
        });
    }

    pub fn set_boost(&mut self, on: bool) {
        self.send(Command::BoostGuild {
            guild: self.active_guild,
            on,
        });
        self.toast(if on {
            "Boosted the server"
        } else {
            "Boost removed"
        });
    }

    pub fn set_viewer_quality(&mut self, quality: crate::voice::ViewQuality) {
        self.viewer_quality = quality;
        crate::voice::set_viewer_quality(quality);
        self.mark_cache_dirty();
        self.toast(format!("Receive quality: {}", quality.label()));
    }

    pub fn set_video_quality(&mut self, quality: crate::voice::VideoQuality) {
        self.video_quality = quality;
        crate::voice::set_quality(quality);
        self.mark_cache_dirty();
        self.toast(format!("Video quality: {}", quality.label()));
        self.modal = Modal::None;
    }

    pub fn prune_now(&mut self) {
        self.send(Command::PruneMembers {
            guild: self.active_guild,
            days: self.prune_days,
            roles: self.prune_roles.iter().copied().collect(),
        });
        self.modal = Modal::None;
    }

    pub fn kick_now(&mut self) {
        let Modal::ConfirmKick { guild, user } = self.modal.clone() else {
            return;
        };
        let name = self
            .user(user)
            .map(|user| user.display_name.clone())
            .unwrap_or_default();
        self.send(Command::KickMember { guild, user });
        self.modal = Modal::None;
        self.toast(format!("{name} was kicked"));
    }

    pub fn ban_now(&mut self) {
        let Modal::ConfirmBan { guild, user } = self.modal.clone() else {
            return;
        };
        let name = self
            .user(user)
            .map(|user| user.display_name.clone())
            .unwrap_or_default();
        let reason = self.ban_reason.text.trim().to_string();
        self.send(Command::BanMember {
            guild,
            user,
            reason,
        });
        self.modal = Modal::None;
        self.toast(format!("{name} was banned"));
    }

    pub fn timeout_now(&mut self, seconds: u64) {
        let Modal::TimeoutMember { guild, user } = self.modal.clone() else {
            return;
        };
        let name = self
            .user(user)
            .map(|user| user.display_name.clone())
            .unwrap_or_default();
        self.send(Command::TimeoutMember {
            guild,
            user,
            seconds,
        });
        self.modal = Modal::None;
        if seconds == 0 {
            self.toast(format!("Timeout removed for {name}"));
        } else {
            self.toast(format!("{name} was timed out"));
        }
    }

    pub fn leave_current_guild(&mut self) {
        let guild = self.active_guild;
        self.send(Command::LeaveGuild { guild });
        self.modal = Modal::None;
    }

    pub fn open_channel_notifications(&mut self, channel: Id) {
        self.modal = Modal::ChannelNotifications(channel);
    }

    pub fn set_channel_notify(&mut self, channel: Id, level: NotificationLevel) {
        self.channel_notify.insert(channel, level);
        let suppress = self.channel_suppress.contains(&channel);
        self.send(Command::SetChannelNotify {
            channel,
            level,
            suppress,
            clear: false,
        });
    }

    pub fn clear_channel_notify(&mut self, channel: Id) {
        self.channel_notify.remove(&channel);
        self.channel_suppress.remove(&channel);
        self.send(Command::SetChannelNotify {
            channel,
            level: NotificationLevel::All,
            suppress: false,
            clear: true,
        });
    }

    pub fn guild_notification(&self, guild: Id) -> GuildNotificationSettings {
        self.guild_notify.get(&guild).cloned().unwrap_or_default()
    }

    pub fn update_guild_notification(
        &mut self,
        guild: Id,
        change: impl FnOnce(&mut GuildNotificationSettings),
    ) {
        let mut settings = self.guild_notification(guild);
        change(&mut settings);
        self.guild_notify.insert(guild, settings.clone());
        self.guild_notify_local_at.insert(guild, self.clock);
        self.send(Command::SetGuildNotify { guild, settings });
        self.bootstrap_pending = true;
        self.mark_cache_dirty();
    }

    pub fn set_guild_notify_level(&mut self, guild: Id, level: NotificationLevel) {
        self.update_guild_notification(guild, |settings| settings.level = level);
    }

    pub fn set_active_guild_notify_level(&mut self, level: NotificationLevel) {
        let guild = self.active_guild;
        self.set_guild_notify_level(guild, level);
    }

    pub fn toggle_active_guild_mute(&mut self) {
        let guild = self.active_guild;
        self.toggle_guild_mute(guild);
    }

    pub fn toggle_active_guild_suppress(&mut self, field: &'static str) {
        let guild = self.active_guild;
        let settings = self.guild_notification(guild);
        let value = match field {
            "everyone" => !settings.suppress_everyone,
            "roles" => !settings.suppress_roles,
            "highlights" => !settings.suppress_highlights,
            _ => return,
        };
        self.set_guild_suppress(guild, field, value);
    }

    pub fn toggle_guild_suppress_everyone(&mut self) {
        self.toggle_active_guild_suppress("everyone");
    }

    pub fn toggle_guild_suppress_roles(&mut self) {
        self.toggle_active_guild_suppress("roles");
    }

    pub fn toggle_guild_suppress_highlights(&mut self) {
        self.toggle_active_guild_suppress("highlights");
    }

    pub fn toggle_guild_mute_new_events(&mut self) {
        let guild = self.active_guild;
        self.update_guild_notification(guild, |settings| {
            settings.mute_new_events = !settings.mute_new_events
        });
    }

    pub fn toggle_guild_mobile_push(&mut self) {
        let guild = self.active_guild;
        self.update_guild_notification(guild, |settings| {
            settings.mobile_push = !settings.mobile_push
        });
    }

    pub fn toggle_guild_mute(&mut self, guild: Id) {
        self.update_guild_notification(guild, |settings| settings.muted = !settings.muted);
    }

    pub fn set_guild_suppress(&mut self, guild: Id, field: &str, value: bool) {
        self.update_guild_notification(guild, |settings| match field {
            "everyone" => settings.suppress_everyone = value,
            "roles" => settings.suppress_roles = value,
            "highlights" => settings.suppress_highlights = value,
            _ => {}
        });
    }

    pub fn add_guild_notification_override(&mut self, guild: Id, channel: Id) {
        self.set_channel_notify(channel, NotificationLevel::All);
        self.modal = Modal::ServerNotifications;
        let _ = guild;
    }

    pub fn set_channel_suppress(&mut self, channel: Id, suppress: bool) {
        if suppress {
            self.channel_suppress.insert(channel);
        } else {
            self.channel_suppress.remove(&channel);
        }
        let level = self
            .channel_notify
            .get(&channel)
            .copied()
            .unwrap_or_default();
        self.send(Command::SetChannelNotify {
            channel,
            level,
            suppress,
            clear: false,
        });
    }

    pub fn mute_channel(&mut self, channel: Id) {
        self.set_channel_notify(channel, NotificationLevel::None);
        self.toast("Channel muted");
    }

    pub fn block_user(&mut self, user: Id) {
        self.send(Command::FriendBlock { user });
        self.toast("User blocked");
    }

    pub fn friend_user(&mut self, user: Id) {
        self.send(Command::FriendRequest { user });
        self.toast("Friend request sent");
    }

    pub fn open_explore(&mut self) {
        self.public_guilds.clear();
        self.explore_loading = true;
        self.modal = Modal::Explore;
        self.send(Command::LoadPublicGuilds);
    }

    pub fn join_public_guild(&mut self, guild: Id) {
        self.send(Command::JoinPublicGuild { guild });
    }

    pub fn delete_channel_now(&mut self) {
        self.send(Command::DeleteChannel {
            channel: self.editing_channel,
        });
        self.modal = Modal::None;
        self.toast("Channel deleted");
    }

    pub fn open_invite(&mut self) {
        self.invite_code = None;
        self.modal = Modal::Invite;
        self.send(Command::CreateInvite {
            guild: self.active_guild,
        });
    }

    pub fn add_friend(&mut self) {
        let handle = self
            .search
            .text
            .trim()
            .trim_start_matches('@')
            .to_lowercase();
        if let Some(user) = self
            .users
            .values()
            .find(|u| {
                u.id != self.me.id
                    && (u.username == handle || u.display_name.to_lowercase() == handle)
            })
            .cloned()
        {
            self.send(Command::FriendRequest { user: user.id });
            self.toast(format!("Friend request sent to {}", user.display_name));
        } else if handle.len() >= 2 {
            self.send(Command::LookupUser { query: handle });
        } else {
            self.toast("Enter at least two characters of the username");
        }
    }

    pub fn accept_friend(&mut self, id: Id) {
        self.send(Command::FriendAccept { user: id });
    }

    pub fn open_emoji(&mut self) {
        self.emoji_search.clear();
        self.emoji_category = 0;
        self.modal = Modal::EmojiPicker;
    }

    pub fn open_reaction(&mut self, message: Id) {
        self.emoji_search.clear();
        self.emoji_category = 0;
        self.modal = Modal::Reaction(message);
    }

    pub fn decline_friend(&mut self, id: Id) {
        self.send(Command::FriendRemove { user: id });
    }

    pub fn set_voice(&mut self, channel: Option<Id>) {
        crate::voice::set_local_user(self.me.id);
        crate::voice::set_quality(self.video_quality);
        let bitrate = self
            .active_guild()
            .map(|guild| guild.voice_kbps)
            .filter(|kbps| *kbps > 0)
            .unwrap_or(64);
        crate::voice::set_audio_bitrate(bitrate);
        let guild = self.active_guild;
        if channel.is_none() {
            self.voice_camera = false;
            self.voice_screen = false;
            self.voice_video.clear();
            self.send(Command::Video {
                channel: None,
                kind: "none".into(),
            });
        }
        self.send(Command::Voice {
            guild,
            channel,
            mute: self.voice_mute,
            deaf: self.voice_deaf,
        });
    }

    pub fn toggle_voice_camera(&mut self) {
        let Some(channel) = self.my_voice_channel() else {
            self.toast("Join a voice channel first");
            return;
        };
        self.voice_camera = !self.voice_camera;
        if self.voice_camera {
            self.voice_screen = false;
        }
        self.push_video(channel);
    }

    pub fn toggle_voice_screen(&mut self) {
        let Some(channel) = self.my_voice_channel() else {
            self.toast("Join a voice channel first");
            return;
        };
        self.voice_screen = !self.voice_screen;
        if self.voice_screen {
            self.voice_camera = false;
        }
        self.push_video(channel);
    }

    fn push_video(&mut self, channel: Id) {
        let kind = if self.voice_camera {
            "camera"
        } else if self.voice_screen {
            "screen"
        } else {
            "none"
        };
        self.send(Command::Video {
            channel: Some(channel),
            kind: kind.into(),
        });
    }

    pub fn my_voice_channel(&self) -> Option<Id> {
        self.voice
            .get(&self.me.id)
            .and_then(|state| state.channel_id)
    }

    pub fn toggle_voice_mute(&mut self) {
        self.voice_mute = !self.voice_mute;
        let channel = self.my_voice_channel();
        self.set_voice(channel);
    }

    pub fn toggle_voice_deafen(&mut self) {
        self.voice_deaf = !self.voice_deaf;
        let channel = self.my_voice_channel();
        self.set_voice(channel);
    }

    pub fn create_thread_named(&mut self, parent: Id, message: Option<Id>) {
        let name = self.profile_edit.text.trim().to_string();
        if name.is_empty() {
            return;
        }
        let body = self.thread_body.text.trim().to_string();
        self.pending_thread_body = (!body.is_empty()).then_some(body);
        self.send(Command::CreateThread {
            channel: parent,
            message,
            name,
        });
        self.modal = Modal::None;
    }

    pub fn open_thread(&mut self, id: Id) {
        let Some(thread) = self.channel(id).cloned() else {
            return;
        };
        let Some(info) = &thread.thread else {
            return;
        };
        self.screen = Screen::Guild;
        self.thread_parent = Some(info.parent_id);
        self.active_channel = id;
        self.channel_seen.insert(id, now_ms());
        self.ensure_messages(id);
        self.ensure_members(self.active_guild);
        self.mark_read(id);
    }

    pub fn close_thread(&mut self) {
        let Some(parent) = self.thread_parent.take() else {
            return;
        };
        self.switch_channel(parent);
    }

    pub fn archive_thread(&mut self, archived: bool) {
        self.send(Command::ArchiveThread {
            channel: self.active_channel,
            archived,
        });
    }

    pub fn start_dm(&mut self, id: Id) {
        if let Some(dm) = self
            .dms
            .iter()
            .find(|d| d.recipients.iter().any(|u| u.id == id))
        {
            let channel = dm.channel.id;
            self.open_dm(channel);
        } else {
            self.send(Command::OpenDm { user: id });
        }
    }

    pub fn login(&mut self) {
        let username = self.auth_user.text.trim().to_string();
        let password = self.auth_pass.text.clone();
        if username.is_empty() || password.is_empty() {
            self.auth_error = Some("Enter a username and password.".into());
            return;
        }
        self.auth_busy = true;
        self.auth_error = None;
        self.send(Command::Login { username, password });
    }

    pub fn register(&mut self) {
        let username = self.auth_user.text.trim().to_lowercase();
        let password = self.auth_pass.text.clone();
        let display = self.auth_display.text.trim().to_string();
        if username.len() < 2 || password.len() < 4 {
            self.auth_error = Some("Username needs 2+ characters and password 4+.".into());
            return;
        }
        self.auth_busy = true;
        self.auth_error = None;
        let display_name = if display.is_empty() {
            username.clone()
        } else {
            display
        };
        self.send(Command::Register {
            username,
            display_name,
            password,
        });
    }

    pub fn set_dark(&mut self, dark: bool) {
        self.dark = dark;
        save_theme(dark);
    }

    pub fn open_settings(&mut self, tab: SettingsTab) {
        self.settings_tab = tab;
        self.sync_profile_edits();
        self.password_current.clear();
        self.password_new.clear();
        if tab == SettingsTab::Application {
            self.server_url_saved = crate::config::stored_server_url();
            self.server_url = TextEdit::new(self.server_url_saved.clone());
        }
        self.modal = Modal::Settings(tab);
        if tab == SettingsTab::Bots {
            self.send(Command::LoadBots);
            self.send(Command::LoadApplications);
        }
        if tab == SettingsTab::Voice {
            self.refresh_voice_devices();
            self.apply_voice_settings();
        }
    }

    pub fn save_server_url(&mut self) {
        let url = match crate::config::normalize(&self.server_url.text) {
            Ok(url) => url,
            Err(message) => {
                self.server_url_error = Some(message.clone());
                self.toast(message);
                return;
            }
        };
        if self.server_checking {
            return;
        }
        if self.net.is_none() {
            let message = "Runecord can't validate a server without a network session.";
            self.server_url_error = Some(message.into());
            self.toast(message);
            return;
        }
        self.server_url_error = None;
        self.server_checking = true;
        self.pending_server_url = Some(url.clone());
        self.send(Command::CheckServer { url });
    }

    pub fn apply_checked_server(&mut self, url: String, ok: bool, error: Option<String>) {
        if self.pending_server_url.as_deref() != Some(url.as_str()) {
            return;
        }
        self.server_checking = false;
        self.pending_server_url = None;
        if !ok {
            let reason = error.unwrap_or_else(|| "The server did not answer.".into());
            self.server_url_error = Some(format!("Can’t connect to {url}: {reason}"));
            self.toast("Can’t connect to that server");
            return;
        }
        match crate::config::save_server_url(&url) {
            Ok((saved, changed)) => {
                self.server_url = TextEdit::new(saved.clone());
                self.server_url_saved = saved;
                self.server_config_error = None;
                self.server_url_error = None;
                if changed {
                    self.toast("Server saved. Restart Runecord to connect to it.");
                } else {
                    self.toast("That server URL is already saved.");
                }
            }
            Err(message) => {
                self.server_url_error = Some(message.clone());
                self.toast(message);
            }
        }
    }

    pub fn reset_server_url(&mut self) {
        self.server_url = TextEdit::new(crate::config::DEFAULT_SERVER_URL.to_string());
        self.save_server_url();
    }

    pub fn restart_app(&mut self) {
        if rune::is_headless() {
            self.toast("Restart Runecord from the app window.");
            return;
        }
        let Ok(exe) = std::env::current_exe() else {
            self.toast("Could not find the Runecord executable.");
            return;
        };
        let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
        match std::process::Command::new(exe).args(args).spawn() {
            Ok(_) => std::process::exit(0),
            Err(error) => self.toast(format!("Could not restart: {error}")),
        }
    }

    pub fn sync_profile_edits(&mut self) {
        self.profile_display = TextEdit::new(self.me.display_name.clone());
        self.profile_pronouns = TextEdit::new(self.me.pronouns.clone());
        self.profile_badges = self.me.badges.iter().cloned().collect();
        self.profile_decoration = self.me.avatar_decoration.clone();
        self.profile_bio = TextEdit::new(self.me.bio.clone());
        self.profile_status = TextEdit::new(self.me.custom_status.clone());
        let activity = self.me.activity.clone().unwrap_or_default();
        self.profile_activity_kind = TextEdit::new(activity.kind);
        self.profile_activity_title = TextEdit::new(activity.title);
        self.profile_activity_subtitle = TextEdit::new(activity.subtitle);
        self.profile_activity_state = TextEdit::new(activity.state);
        self.profile_activity_url = TextEdit::new(activity.url);
        self.profile_connection_kind.clear();
        self.profile_connection_name.clear();
    }

    pub fn set_profile_effect(&mut self, effect: &str) {
        self.send(Command::UpdateProfileStyle {
            effect: effect.to_string(),
            nameplate: self.me.nameplate.clone(),
            widget: self.me.profile_widget.clone(),
        });
    }

    pub fn set_profile_nameplate(&mut self, nameplate: &str) {
        self.send(Command::UpdateProfileStyle {
            effect: self.me.profile_effect.clone(),
            nameplate: nameplate.to_string(),
            widget: self.me.profile_widget.clone(),
        });
    }

    pub fn set_profile_widget(&mut self, widget: &str) {
        self.send(Command::UpdateProfileStyle {
            effect: self.me.profile_effect.clone(),
            nameplate: self.me.nameplate.clone(),
            widget: widget.to_string(),
        });
    }

    pub fn save_profile_activity(&mut self) {
        let kind = self.profile_activity_kind.text.trim().to_lowercase();
        let title = self.profile_activity_title.text.trim().to_string();
        if kind.is_empty() || title.is_empty() {
            self.toast("Set an activity kind and title");
            return;
        }
        let current = self.me.activity.clone().unwrap_or_default();
        let activity = ProfileActivity {
            kind,
            title,
            subtitle: self.profile_activity_subtitle.text.trim().to_string(),
            state: self.profile_activity_state.text.trim().to_string(),
            url: self.profile_activity_url.text.trim().to_string(),
            artwork_url: current.artwork_url,
            started_at: if current.started_at == 0 {
                now_ms()
            } else {
                current.started_at
            },
        };
        self.send(Command::UpdateProfileExtras {
            activity: Some(activity),
            connections: self.me.connections.clone(),
        });
        self.toast("Activity saved");
    }

    pub fn clear_profile_activity(&mut self) {
        self.send(Command::UpdateProfileExtras {
            activity: None,
            connections: self.me.connections.clone(),
        });
        self.profile_activity_kind.clear();
        self.profile_activity_title.clear();
        self.profile_activity_subtitle.clear();
        self.profile_activity_state.clear();
        self.profile_activity_url.clear();
        self.toast("Activity cleared");
    }

    pub fn upload_activity_artwork(&mut self, path: std::path::PathBuf) {
        self.send(Command::UploadActivityArtwork { path });
    }

    pub fn remove_activity_artwork(&mut self) {
        self.send(Command::RemoveActivityArtwork);
    }

    pub fn add_profile_connection(&mut self) {
        let kind = self.profile_connection_kind.text.trim().to_lowercase();
        let name = self.profile_connection_name.text.trim().to_string();
        if kind.is_empty() || name.is_empty() {
            self.toast("Add a connection kind and name");
            return;
        }
        let mut connections = self.me.connections.clone();
        if connections
            .iter()
            .any(|connection| connection.kind == kind && connection.name == name)
        {
            self.toast("That connection is already listed");
            return;
        }
        connections.push(ProfileConnection { kind, name });
        connections.truncate(6);
        self.send(Command::UpdateProfileExtras {
            activity: self.me.activity.clone(),
            connections,
        });
        self.profile_connection_kind.clear();
        self.profile_connection_name.clear();
    }

    pub fn remove_profile_connection(&mut self, kind: String, name: String) {
        let connections: Vec<ProfileConnection> = self
            .me
            .connections
            .iter()
            .filter(|connection| !(connection.kind == kind && connection.name == name))
            .cloned()
            .collect();
        self.send(Command::UpdateProfileExtras {
            activity: self.me.activity.clone(),
            connections,
        });
    }

    pub fn toggle_profile_badge(&mut self, badge: &str) {
        if !self.profile_badges.remove(badge) {
            self.profile_badges.insert(badge.to_string());
        }
        self.send(Command::UpdateProfile {
            display_name: None,
            pronouns: None,
            badges: Some(self.profile_badges.iter().cloned().collect()),
            avatar_decoration: None,
            custom_status: None,
            bio: None,
            color: None,
            avatar_border: None,
            banner_color: None,
        });
    }

    pub fn set_avatar_decoration(&mut self, decoration: &str) {
        self.profile_decoration = decoration.to_string();
        self.send(Command::UpdateProfile {
            display_name: None,
            pronouns: None,
            badges: None,
            avatar_decoration: Some(decoration.to_string()),
            custom_status: None,
            bio: None,
            color: None,
            avatar_border: None,
            banner_color: None,
        });
    }

    pub fn open_group_dm(&mut self) {
        self.group_dm_selected.clear();
        self.group_dm_name = TextEdit::new("");
        self.modal = Modal::GroupDm;
    }

    pub fn open_dm_settings(&mut self, channel: Id) {
        let Some(dm) = self.dm_for(channel) else {
            return;
        };
        self.dm_settings_name = TextEdit::new(dm.channel.name.clone());
        self.modal = Modal::DmSettings { channel };
    }

    pub fn save_dm_name(&mut self) {
        let Modal::DmSettings { channel } = self.modal else {
            return;
        };
        let name = self.dm_settings_name.text.trim().to_string();
        if name.is_empty() {
            self.toast("Group name can't be empty");
            return;
        }
        self.send(Command::RenameDm { channel, name });
    }

    pub fn begin_dm_icon_crop(&mut self, channel: Id, path: std::path::PathBuf) {
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                self.toast(format!("Could not read image: {error}"));
                return;
            }
        };
        if crate::image_anim::is_animated(&bytes) {
            self.send(Command::UploadDmIcon { channel, path });
            self.toast("Uploading animated group icon.");
            return;
        }
        let decoded = match image::load_from_memory(&bytes) {
            Ok(image) => image,
            Err(error) => {
                self.toast(format!("Could not decode image: {error}"));
                return;
            }
        };
        let rgba = decoded.to_rgba8();
        let (width, height) = rgba.dimensions();
        if width < 32 || height < 32 {
            self.toast("Image is too small");
            return;
        }
        self.crop_source = Some(CropSource { image: rgba });
        self.crop_banner = false;
        self.crop_user = false;
        self.crop_dm = channel;
        self.crop_zoom = 1.0;
        self.crop_offset = (0.0, 0.0);
        self.crop_drag_origin = (0.0, 0.0);
        self.modal = Modal::CropDmIcon { channel };
        self.refresh_crop_preview();
    }

    pub fn remove_dm_icon(&mut self, channel: Id) {
        self.send(Command::RemoveDmIcon { channel });
    }

    pub fn add_dm_member(&mut self, channel: Id, user: Id) {
        self.send(Command::AddDmMember { channel, user });
    }

    pub fn remove_dm_member(&mut self, channel: Id, user: Id) {
        self.send(Command::RemoveDmMember { channel, user });
    }

    pub fn leave_dm(&mut self, channel: Id) {
        self.send(Command::LeaveDm { channel });
        self.modal = Modal::None;
        self.toast("You left the group");
    }

    pub fn toggle_group_dm(&mut self, user: Id) {
        if !self.group_dm_selected.remove(&user) {
            self.group_dm_selected.insert(user);
        }
    }

    pub fn create_group_dm(&mut self) {
        if self.group_dm_selected.len() < 2 {
            self.toast("Pick at least two friends");
            return;
        }
        if self.group_dm_selected.len() > 9 {
            self.toast("Group DMs support up to 10 people");
            return;
        }
        self.send(Command::CreateGroupDm {
            users: self.group_dm_selected.iter().copied().collect(),
            name: self.group_dm_name.text.trim().to_string(),
        });
        self.modal = Modal::None;
    }

    pub fn apply_voice_settings(&self) {
        crate::voice::set_input_device(self.voice_input_device.clone());
        crate::voice::set_camera_device(self.voice_camera_device.clone());
        crate::voice::set_output_device(self.voice_output_device.clone());
        crate::voice::set_input_sensitivity(self.voice_sensitivity);
        crate::voice::set_auto_sensitivity(self.voice_auto_sensitivity);
        crate::voice::set_input_volume(self.voice_input_volume);
        crate::voice::set_output_volume(self.voice_output_volume);
        crate::voice::set_voice_activity(self.voice_mode == "voice");
        crate::voice::set_push_to_talk(self.voice_mode == "ptt");
    }

    pub fn refresh_voice_devices(&mut self) {
        let (audio, video) = crate::voice::list_devices();
        self.audio_devices = audio;
        self.video_devices = video;
        self.output_devices = crate::voice::list_output_devices();
    }

    pub fn select_voice_device(&mut self, kind: &str, device: String) {
        match kind {
            "input" => self.voice_input_device = device,
            "output" => self.voice_output_device = device,
            "camera" => self.voice_camera_device = device,
            _ => {}
        }
        self.apply_voice_settings();
        self.mark_cache_dirty();
        self.modal = Modal::Settings(SettingsTab::Voice);
    }

    pub fn set_voice_sensitivity(&mut self, value: f32) {
        self.voice_sensitivity = value.clamp(0.001, 0.5);
        crate::voice::set_input_sensitivity(self.voice_sensitivity);
        self.mark_cache_dirty();
    }

    pub fn set_voice_auto_sensitivity(&mut self, enabled: bool) {
        self.voice_auto_sensitivity = enabled;
        crate::voice::set_auto_sensitivity(enabled);
        self.mark_cache_dirty();
    }

    pub fn set_voice_volume(&mut self, input: bool, value: f32) {
        if input {
            self.voice_input_volume = value.clamp(0.0, 1.0);
            crate::voice::set_input_volume(self.voice_input_volume);
        } else {
            self.voice_output_volume = value.clamp(0.0, 1.0);
            crate::voice::set_output_volume(self.voice_output_volume);
        }
        self.mark_cache_dirty();
    }

    pub fn set_voice_mode(&mut self, mode: &str) {
        self.voice_mode = mode.to_string();
        crate::voice::set_voice_activity(mode == "voice");
        crate::voice::set_push_to_talk(mode == "ptt");
        self.mark_cache_dirty();
    }

    pub fn toggle_mic_test(&mut self) {
        if crate::voice::mic_testing() {
            crate::voice::stop_mic_test();
        } else {
            crate::voice::start_mic_test();
        }
    }

    pub fn toggle_camera_test(&mut self) {
        if crate::voice::camera_testing() {
            crate::voice::stop_camera_test();
        } else {
            crate::voice::start_camera_test();
        }
    }

    pub fn save_profile(&mut self) {
        let display_name = self.profile_display.text.trim().to_string();
        let pronouns = self.profile_pronouns.text.trim().to_string();
        let bio = self.profile_bio.text.trim().to_string();
        let custom_status = self.profile_status.text.trim().to_string();
        if display_name.is_empty() {
            self.toast("Display name cannot be empty");
            return;
        }
        self.send(Command::UpdateProfile {
            display_name: Some(display_name),
            pronouns: Some(pronouns),
            badges: Some(self.profile_badges.iter().cloned().collect()),
            avatar_decoration: Some(self.profile_decoration.clone()),
            custom_status: Some(custom_status),
            bio: Some(bio),
            color: None,
            avatar_border: None,
            banner_color: None,
        });
        self.toast("Profile saved");
    }

    pub fn set_avatar_color(&mut self, color: u32) {
        self.send(Command::UpdateProfile {
            display_name: None,
            pronouns: None,
            badges: None,
            avatar_decoration: None,
            custom_status: None,
            bio: None,
            color: Some(color),
            avatar_border: None,
            banner_color: None,
        });
    }

    pub fn set_avatar_border(&mut self, color: u32) {
        self.send(Command::UpdateProfile {
            display_name: None,
            pronouns: None,
            badges: None,
            avatar_decoration: None,
            custom_status: None,
            bio: None,
            color: None,
            avatar_border: Some(color),
            banner_color: None,
        });
    }

    pub fn set_banner_color(&mut self, color: u32) {
        self.send(Command::UpdateProfile {
            display_name: None,
            pronouns: None,
            badges: None,
            avatar_decoration: None,
            custom_status: None,
            bio: None,
            color: None,
            avatar_border: None,
            banner_color: Some(color),
        });
    }

    pub fn select_settings_tab(&mut self, tab: SettingsTab) {
        self.settings_tab = tab;
        if tab == SettingsTab::Bots {
            self.send(Command::LoadBots);
            self.send(Command::LoadApplications);
        }
    }

    pub fn change_password(&mut self) {
        let current = self.password_current.text.clone();
        let new = self.password_new.text.clone();
        if current.is_empty() || new.is_empty() {
            self.toast("Enter your current and new password");
            return;
        }
        self.send(Command::ChangePassword { current, new });
        self.password_current.clear();
        self.password_new.clear();
    }

    pub fn toggle_compact(&mut self, compact: bool) {
        self.compact = compact;
        self.mark_cache_dirty();
    }

    pub fn toggle_developer(&mut self, developer: bool) {
        self.developer = developer;
        self.mark_cache_dirty();
    }

    pub fn reset_preferences(&mut self) {
        self.set_dark(true);
        self.notify_dm = true;
        self.notify_mentions = true;
        self.compact = false;
        self.developer = false;
        self.keybinds = default_keybinds();
        self.recording_key = None;
        self.mark_cache_dirty();
        self.toast("Preferences reset");
    }

    pub fn begin_keybind(&mut self, action: &str) {
        self.recording_key = Some(action.to_string());
    }

    pub fn set_keybind(&mut self, action: &str, chord: &str) {
        for (other, value) in self.keybinds.iter_mut() {
            if other != action && value == chord {
                value.clear();
            }
        }
        self.keybinds.insert(action.to_string(), chord.to_string());
        self.mark_cache_dirty();
    }

    pub fn reset_keybinds(&mut self) {
        self.keybinds = default_keybinds();
        self.recording_key = None;
        self.mark_cache_dirty();
        self.toast("Keybinds reset");
    }

    pub fn open_server_settings(&mut self) {
        self.modal = Modal::ServerSettings;
        self.server_tab = ServerTab::Overview;
        self.sync_server_overview();
        self.ensure_guild(self.active_guild);
        if self.active_guild != 0 {
            self.send(Command::LoadEmojis {
                guild: self.active_guild,
            });
        }
        let first = self.guild_roles(self.active_guild).first().map(|r| r.id);
        self.selected_role = first.unwrap_or(0);
        if let Some(id) = first {
            self.select_role(id);
        }
        if let Some(member) = self.members.get(&self.active_guild).and_then(|m| m.first()) {
            self.selected_member = Some(member.user.id);
        }
    }

    pub fn sync_server_overview(&mut self) {
        let Some(guild) = self.active_guild().cloned() else {
            return;
        };
        self.overview_name = TextEdit::new(guild.name);
        self.overview_description = TextEdit::new(guild.description);
        self.overview_tag = TextEdit::new(guild.tag);
        self.overview_color = guild.icon_color;
        self.overview_discoverable = guild.discoverable;
    }

    pub fn overview_dirty(&self) -> bool {
        let Some(guild) = self.active_guild() else {
            return false;
        };
        self.overview_name.text.trim() != guild.name
            || self.overview_description.text.trim() != guild.description
            || self.overview_tag.text.trim() != guild.tag
            || self.overview_color != guild.icon_color
            || self.overview_discoverable != guild.discoverable
    }

    pub fn save_server_overview(&mut self) {
        let name = self.overview_name.text.trim().to_string();
        if name.is_empty() {
            self.toast("Name the server first");
            return;
        }
        self.send(Command::UpdateGuildSettings {
            guild: self.active_guild,
            name,
            description: self.overview_description.text.trim().to_string(),
            color: self.overview_color,
            tag: self.overview_tag.text.trim().to_string(),
            discoverable: self.overview_discoverable,
        });
        self.toast("Server updated");
    }

    pub fn select_server_tab(&mut self, tab: ServerTab) {
        self.server_tab = tab;
        match tab {
            ServerTab::Audit => self.load_audit(false),
            ServerTab::Bans => {
                self.bans.clear();
                self.appeals.clear();
                self.send(Command::LoadBans {
                    guild: self.active_guild,
                });
                self.send(Command::LoadAppeals {
                    guild: self.active_guild,
                });
            }
            ServerTab::Overview => self.sync_server_overview(),
            _ => {}
        }
    }

    pub fn load_audit(&mut self, older: bool) {
        if self.active_guild == 0 || self.audit_loading {
            return;
        }
        let before = if older {
            self.audit.last().map(|entry| entry.id)
        } else {
            None
        };
        if older && self.audit_complete {
            return;
        }
        self.audit_loading = true;
        if !older {
            self.audit.clear();
            self.audit_complete = false;
        }
        self.send(Command::LoadAudit {
            guild: self.active_guild,
            before,
        });
    }

    pub fn select_role(&mut self, id: Id) {
        let Some(role) = self
            .guild_roles(self.active_guild)
            .iter()
            .find(|role| role.id == id)
            .cloned()
        else {
            return;
        };
        self.selected_role = id;
        self.role_name = TextEdit::new(role.name);
        self.role_color = role.color;
        self.role_permissions = role.permissions;
        self.role_hoist = role.hoist;
        self.role_mentionable = role.mentionable;
        self.role_link_label.clear();
        self.role_link_url.clear();
        self.send(Command::LoadRoleLinks {
            guild: self.active_guild,
            role: id,
        });
    }

    pub fn upload_role_icon(&mut self, role: Id, path: std::path::PathBuf) {
        self.send(Command::UploadRoleIcon {
            guild: self.active_guild,
            role,
            path,
        });
    }

    pub fn remove_role_icon(&mut self, role: Id) {
        self.send(Command::RemoveRoleIcon {
            guild: self.active_guild,
            role,
        });
    }

    pub fn add_role_link(&mut self, role: Id) {
        let label = self.role_link_label.text.trim().to_string();
        let url = self.role_link_url.text.trim().to_string();
        if label.is_empty() || url.is_empty() {
            self.toast("Add a label and an https:// link");
            return;
        }
        self.send(Command::AddRoleLink {
            guild: self.active_guild,
            role,
            label,
            url,
        });
        self.role_link_label.clear();
        self.role_link_url.clear();
    }

    pub fn delete_role_link(&mut self, role: Id, link: Id) {
        self.send(Command::DeleteRoleLink {
            guild: self.active_guild,
            role,
            link,
        });
    }

    pub fn create_role_now(&mut self) {
        let name = format!("new role {}", self.guild_roles(self.active_guild).len());
        self.send(Command::CreateRole {
            guild: self.active_guild,
            name,
            color: 0x99aab5,
            permissions: 0,
        });
    }

    pub fn save_role(&mut self) {
        if self.selected_role == 0 {
            self.toast("Select a role first");
            return;
        }
        let name = self.role_name.text.trim().to_string();
        if name.is_empty() {
            self.toast("Role name cannot be empty");
            return;
        }
        let is_everyone = self
            .guild_roles(self.active_guild)
            .iter()
            .any(|role| role.id == self.selected_role && role.name == "@everyone");
        self.send(Command::UpdateRole {
            guild: self.active_guild,
            role: self.selected_role,
            name: (!is_everyone).then_some(name),
            color: Some(self.role_color),
            permissions: Some(self.role_permissions),
            hoist: Some(self.role_hoist),
            mentionable: Some(self.role_mentionable),
            position: None,
        });
    }

    pub fn delete_selected_role(&mut self) {
        let role = self.selected_role;
        if role == 0 {
            return;
        }
        if self
            .guild_roles(self.active_guild)
            .iter()
            .any(|item| item.id == role && item.name == "@everyone")
        {
            self.toast("@everyone cannot be deleted");
            return;
        }
        self.send(Command::DeleteRole {
            guild: self.active_guild,
            role,
        });
    }

    pub fn move_selected_role(&mut self, delta: i32) {
        let guild = self.active_guild;
        let mut roles: Vec<Role> = self.guild_roles(guild).to_vec();
        roles.sort_by_key(|role| std::cmp::Reverse(role.position));
        let Some(index) = roles.iter().position(|role| role.id == self.selected_role) else {
            return;
        };
        let target = index as i32 + delta;
        if target < 0 || target >= roles.len() as i32 {
            return;
        }
        let target = target as usize;
        if roles[target].name == "@everyone" || roles[index].name == "@everyone" {
            return;
        }
        let a = roles[index].clone();
        let b = roles[target].clone();
        let (a_id, b_id) = (a.id, b.id);
        let (a_pos, b_pos) = (a.position, b.position);
        for (role, position) in [(a_id, b_pos), (b_id, a_pos)] {
            self.send(Command::UpdateRole {
                guild,
                role,
                name: None,
                color: None,
                permissions: None,
                hoist: None,
                mentionable: None,
                position: Some(position),
            });
        }
    }

    pub fn select_member(&mut self, id: Id) {
        self.selected_member = Some(id);
    }

    pub fn toggle_member_role(&mut self, role: Id) {
        let Some(user) = self.selected_member else {
            self.toast("Select a member first");
            return;
        };
        self.toggle_role_for_member(role, user);
    }

    pub fn toggle_role_for_member(&mut self, role: Id, user: Id) {
        let guild = self.active_guild;
        let Some(member) = self
            .members
            .get(&guild)
            .and_then(|list| list.iter().find(|member| member.user.id == user))
            .cloned()
        else {
            return;
        };
        let mut roles = member.roles;
        if let Some(index) = roles.iter().position(|id| *id == role) {
            roles.remove(index);
        } else {
            roles.push(role);
        }
        self.send(Command::UpdateMemberRoles { guild, user, roles });
    }

    pub fn apply_slash_command(&mut self, name: &str) {
        self.composer.clear();
        self.composer.insert(&format!("/{name} "));
    }

    pub fn create_emoji_from_path(&mut self, path: std::path::PathBuf) {
        let name = if self.emoji_name.text.trim().is_empty() {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("emoji")
                .to_lowercase()
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect::<String>()
        } else {
            self.emoji_name.text.trim().to_lowercase()
        };
        if name.chars().count() < 2 {
            self.toast("Emoji name needs at least two characters");
            return;
        }
        self.send(Command::CreateEmoji {
            guild: self.active_guild,
            name,
            path,
        });
        self.emoji_name.clear();
    }

    pub fn delete_emoji(&mut self, id: Id) {
        self.send(Command::DeleteEmoji {
            guild: self.active_guild,
            id,
        });
    }

    pub fn apply_role(&mut self, role: Role) {
        if let Some(detail) = self.guild_details.get_mut(&role.guild_id) {
            if let Some(existing) = detail.roles.iter_mut().find(|item| item.id == role.id) {
                *existing = role;
            } else {
                detail.roles.push(role);
            }
            detail
                .roles
                .sort_by_key(|item| std::cmp::Reverse(item.position));
        }
        self.mark_cache_dirty();
    }

    pub fn create_bot(&mut self) {
        let name = self.bot_name.text.trim().to_string();
        if name.chars().count() < 2 {
            self.toast("Bot name needs at least two characters");
            return;
        }
        let guild = if self.add_bot_to_guild && self.active_guild != 0 {
            Some(self.active_guild)
        } else {
            None
        };
        self.send(Command::CreateBot { name, guild });
        self.bot_name.clear();
    }

    pub fn revoke_bot(&mut self, id: Id) {
        self.send(Command::DeleteBot { id });
    }

    pub fn create_application(&mut self) {
        let name = self.app_name.text.trim().to_string();
        if name.chars().count() < 2 {
            self.toast("Application name needs at least two characters");
            return;
        }
        let description = self.app_description.text.trim().to_string();
        self.send(Command::CreateApplication { name, description });
        self.app_name.clear();
        self.app_description.clear();
    }

    pub fn delete_application(&mut self, id: Id) {
        self.send(Command::DeleteApplication { id });
    }

    pub fn reveal_application_token(&mut self, id: Id) {
        let name = self
            .applications
            .iter()
            .find(|app| app.id == id)
            .map(|app| app.name.clone())
            .unwrap_or_else(|| "Application".to_string());
        self.send(Command::ApplicationToken { id, name });
    }

    pub fn create_install_link(&mut self, id: Id) {
        self.pending_install_app = Some(id);
        self.send(Command::ApplicationInstallLink { id });
    }

    pub fn install_by_code(&mut self) {
        let code = self.install_code.text.trim().to_string();
        if code.is_empty() {
            self.toast("Paste an install code first");
            return;
        }
        if self.active_guild == 0 {
            self.toast("Select a server before installing");
            return;
        }
        self.send(Command::InstallApplication {
            code,
            guild: self.active_guild,
        });
        self.install_code.clear();
    }

    pub fn logout(&mut self) {
        crate::voice::shutdown();
        self.send(Command::Logout);
        crate::cache::clear(&self.cache_profile);
        self.logged_in = false;
        self.connecting = false;
        self.auth_busy = false;
        self.auth_pass.clear();
        self.me = User::default();
        self.messages.clear();
        self.guilds.clear();
        self.guild_details.clear();
        self.channels.clear();
        self.members.clear();
        self.member_state.clear();
        self.dms.clear();
        self.relationships.clear();
        self.images.clear();
        self.image_sizes.clear();
        self.image_order.clear();
        self.image_bytes = 0;
        self.image_requests.clear();
        self.user_requests.clear();
        self.visible_messages.clear();
        self.loading_older.clear();
        self.older_complete.clear();
        self.channel_seen.clear();
        self.guild_seen.clear();
        self.member_seen.clear();
        self.stale_guilds.clear();
        self.viewing_image = None;
        self.drop_hover = false;
        self.member_search_due = 0.0;
        self.member_search_live.clear();
        self.last_member_query.clear();
        self.unread.clear();
        self.reads.clear();
        self.presence.clear();
        self.voice.clear();
        self.typing.clear();
        self.cache_dirty_at = None;
        self.bootstrap_pending = false;
        self.modal = Modal::None;
    }

    // -----------------------------------------------------------------------
    // Poll
    // -----------------------------------------------------------------------
}

fn timeout_seconds(value: u64, unit: TimeoutUnit) -> u64 {
    value.min(unit.max_value()) * unit.seconds()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_timeout_converts_and_clamps() {
        assert_eq!(timeout_seconds(10, TimeoutUnit::Minutes), 600);
        assert_eq!(timeout_seconds(3, TimeoutUnit::Hours), 10_800);
        assert_eq!(timeout_seconds(3, TimeoutUnit::Days), 259_200);
        assert_eq!(timeout_seconds(28, TimeoutUnit::Days), 2_419_200);
        assert_eq!(timeout_seconds(29, TimeoutUnit::Days), 2_419_200);
        assert_eq!(timeout_seconds(99_999, TimeoutUnit::Minutes), 2_419_200);
    }
}
