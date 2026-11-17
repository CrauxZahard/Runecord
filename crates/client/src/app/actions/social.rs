//! App actions split by feature.

use super::super::*;
use crate::net::Command;

impl App {
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

    pub fn open_gif_picker(&mut self) {
        self.gif_query.clear();
        self.send(Command::LoadGifs {
            query: String::new(),
        });
        self.modal = Modal::GifPicker;
    }

    pub fn search_gifs(&mut self) {
        self.send(Command::LoadGifs {
            query: self.gif_query.text.clone(),
        });
    }

    pub fn send_gif(&mut self, slug: &str) {
        let channel = self.current_channel_id();
        if channel == 0 {
            return;
        }
        self.send(Command::SendGif {
            channel,
            slug: slug.to_string(),
        });
        self.modal = Modal::None;
    }

    pub fn load_activities(&mut self) {
        self.send(Command::LoadActivities {
            guild: self.active_guild,
        });
    }

    pub fn open_activities(&mut self, channel: Id) {
        self.load_activities();
        self.modal = Modal::Activities(channel);
    }

    pub fn start_activity(&mut self, channel: Id, activity: Id) {
        self.send(Command::StartActivity { channel, activity });
        self.modal = Modal::None;
    }

    pub fn stop_activity(&mut self, channel: Id) {
        self.send(Command::StopActivity { channel });
    }

    pub fn stage_action(&mut self, channel: Id, action: &str, user: Option<Id>) {
        self.send(Command::StageAction {
            channel,
            action: action.to_string(),
            user,
        });
    }

    pub fn load_stage(&mut self, channel: Id) {
        self.send(Command::LoadStage { channel });
    }

    pub fn join_stage(&mut self, channel: Id) {
        self.set_voice(Some(channel));
        self.load_stage(channel);
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
        let guild = channel
            .and_then(|id| self.channel(id))
            .and_then(|channel| channel.guild_id)
            .or_else(|| self.active_guild().map(|guild| guild.id))
            .unwrap_or(0);
        if channel.is_none() {
            self.voice_camera = false;
            self.voice_screen = false;
            self.voice_video.clear();
            self.voice_video_info.clear();
            // Update locally right away: otherwise the "Voice Connected"
            // panel lingers until a gateway event arrives. With a guild id the
            // backend's disconnect event is visible to everyone in the server.
            self.voice.remove(&self.me.id);
            self.sync_overlay();
            self.mark_cache_dirty();
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

    pub(crate) fn push_video(&mut self, channel: Id) {
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
        let tag_ids = std::mem::take(&mut self.thread_tag_ids);
        self.send(Command::CreateThread {
            channel: parent,
            message,
            name,
            tag_ids,
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
}
