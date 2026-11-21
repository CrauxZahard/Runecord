//! App actions split by feature.

use super::super::*;
use crate::net::Command;
use rune::TextEdit;

impl App {
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
        let Some(current_channel) = self.channel(channel).cloned() else {
            return;
        };
        if !self.can_send_in(&current_channel) {
            self.toast("You do not have permission to send messages here");
            return;
        }
        if current_channel.kind == ChannelKind::Thread {
            let archived = current_channel
                .thread
                .as_ref()
                .is_some_and(|info| info.archived);
            if archived {
                if !self.can_manage_thread(&current_channel) {
                    self.toast("This thread is archived. Ask a moderator to reopen it.");
                    return;
                }
                // Sending reopens a closed thread, the way Discord does.
                self.send(Command::ArchiveThread {
                    channel,
                    archived: false,
                });
            }
            self.joined_threads.insert(channel);
            self.left_threads.remove(&channel);
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
        // A reply pings the author by default. Encoding it as a leading mention
        // keeps every client in sync (including notification/unread counts) and
        // is exactly the mention chip Discord shows above a reply. "@ off"
        // suppresses it.
        let reply_mention = self
            .reply_notify
            .then(|| {
                self.reply_to
                    .as_ref()
                    .filter(|r| r.author_id != self.me.id)
                    .map(|r| r.author_id)
            })
            .flatten();
        let sent_text = match reply_mention {
            Some(author) => format!("<@{author}> {text}"),
            None => text.clone(),
        };
        if current_channel.kind == ChannelKind::Forum {
            self.pending_forum = Some(channel);
        } else {
            let temp = Message {
                id: -now,
                channel_id: channel,
                author_id: self.me.id,
                content: sent_text.clone(),
                timestamp: now,
                attachments: self.attachments.clone(),
                mentions: reply_mention.into_iter().collect(),
                pending: true,
                ..Message::default()
            };
            self.messages.entry(channel).or_default().push(temp);
        }
        self.send(Command::Send {
            channel,
            content: sent_text,
            reply_to: self.reply_to.as_ref().map(|r| r.message_id),
            attachments: self.attachments.iter().map(|a| a.id).collect(),
            nonce: format!("{now}"),
        });
        self.reply_to = None;
        self.reply_notify = true;
        self.composer.clear();
        self.attachments.clear();
        self.mark_read(channel);
    }

    pub fn open_scheduled(&mut self) {
        self.schedule_content = TextEdit::new(self.composer.text.clone());
        self.schedule_minutes = TextEdit::new("60");
        let channel = match self.screen {
            Screen::Guild => self.active_channel,
            Screen::Dms => self.active_dm,
        };
        if channel != 0 {
            self.send(Command::LoadScheduled { channel });
        }
        self.modal = Modal::ScheduledMessages;
    }

    pub fn schedule_now(&mut self) {
        let channel = match self.screen {
            Screen::Guild => self.active_channel,
            Screen::Dms => self.active_dm,
        };
        if channel == 0 {
            return;
        }
        let content = self.schedule_content.text.trim_end().to_string();
        if content.is_empty() {
            self.toast("Write a message to schedule");
            return;
        }
        let Ok(minutes) = self.schedule_minutes.text.trim().parse::<i64>() else {
            self.toast("Enter minutes from now");
            return;
        };
        if minutes < 1 {
            self.toast("Schedule at least one minute ahead");
            return;
        }
        let send_at = now_ms() + minutes.min(43_200) * 60 * 1000;
        self.send(Command::ScheduleMessage {
            channel,
            content,
            send_at,
        });
        self.toast("Message scheduled");
    }

    pub fn delete_scheduled(&mut self, id: Id) {
        self.send(Command::DeleteScheduled { id });
    }

    pub fn open_create_poll(&mut self) {
        self.poll_question = TextEdit::new("");
        self.poll_option_1 = TextEdit::new("");
        self.poll_option_2 = TextEdit::new("");
        self.poll_option_3 = TextEdit::new("");
        self.poll_option_4 = TextEdit::new("");
        self.poll_multiple = false;
        self.poll_anonymous = false;
        self.poll_duration = TextEdit::new("24");
        self.modal = Modal::CreatePoll;
    }

    pub fn create_poll_now(&mut self) {
        let channel = match self.screen {
            Screen::Guild => self.active_channel,
            Screen::Dms => self.active_dm,
        };
        if channel == 0 {
            return;
        }
        let question = self.poll_question.text.trim().to_string();
        let options: Vec<String> = [
            &self.poll_option_1,
            &self.poll_option_2,
            &self.poll_option_3,
            &self.poll_option_4,
        ]
        .into_iter()
        .map(|option| option.text.trim().to_string())
        .filter(|option| !option.is_empty())
        .collect();
        if question.is_empty() {
            self.toast("Add a poll question");
            return;
        }
        if options.len() < 2 {
            self.toast("Add at least two poll options");
            return;
        }
        let duration_hours = self
            .poll_duration
            .text
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|hours| *hours > 0)
            .map(|hours| hours.min(168));
        self.send(Command::CreatePoll {
            channel,
            question,
            options,
            multiple: self.poll_multiple,
            anonymous: self.poll_anonymous,
            duration_hours,
        });
        self.modal = Modal::None;
    }

    pub fn click_component(&mut self, message: Id, custom_id: String) {
        self.send(Command::ClickComponent { message, custom_id });
    }

    pub fn vote_poll(&mut self, message: Id, option: u32) {
        self.send(Command::VotePoll { message, option });
    }

    pub fn open_forward(&mut self, message: Id) {
        self.modal = Modal::ForwardMessage(message);
    }

    pub fn forward_to(&mut self, message: Id, channel: Id) {
        self.send(Command::ForwardMessage { message, channel });
        self.modal = Modal::None;
        self.toast("Message forwarded");
    }

    pub fn upload_file(&mut self, source: crate::net::UploadSource) {
        let channel = match self.screen {
            Screen::Guild => self.active_channel,
            Screen::Dms => self.active_dm,
        };
        let limit = match self.screen {
            Screen::Guild => self
                .active_guild()
                .map(|guild| guild.upload_limit_bytes)
                .filter(|limit| *limit > 0)
                .unwrap_or(128 * 1024 * 1024),
            Screen::Dms => 128 * 1024 * 1024,
        };
        let (name, size) = match &source {
            crate::net::UploadSource::Path(path) => (
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "file".into()),
                std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0),
            ),
            crate::net::UploadSource::Bytes { name, bytes } => (name.clone(), bytes.len() as u64),
        };
        if size > limit {
            self.toast(format!(
                "File is larger than the {} MB upload limit.",
                limit / (1024 * 1024)
            ));
            return;
        }
        if channel == 0 {
            return;
        }
        self.toast(format!("Uploading {name}"));
        self.send(Command::Upload { channel, source });
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

    /// Replace the `@query` the user typed with a mention token from the picker.
    pub fn insert_mention(&mut self, start: usize, end: usize, token: &str) {
        let end = end.max(start);
        self.composer.set_selection(start, end);
        self.composer.insert(token);
        self.composer.focused = true;
    }

    pub fn set_reply(&mut self, message: Id, author_id: Id, content: &str) {
        // Replying pings the author by default; the composer can turn it off.
        self.reply_notify = author_id != self.me.id;
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
        } else if let Some(list) = self.messages.get_mut(&self.active_channel)
            && let Some(m) = list.iter_mut().find(|m| m.id == message)
        {
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
}
