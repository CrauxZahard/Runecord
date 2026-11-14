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
        if let Ok(meta) = std::fs::metadata(&path)
            && meta.len() > limit
        {
            self.toast(format!(
                "File is larger than {} MB. Boost the server to raise the limit.",
                limit / (1024 * 1024)
            ));
            return;
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
