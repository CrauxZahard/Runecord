//! Network event handlers: voice.

use super::super::*;

impl App {
    pub(crate) fn apply_voice_state(&mut self, v: VoiceState) {
        if v.channel_id.is_some() {
            self.voice.insert(v.user_id, v);
        } else {
            crate::voice::clear_speaking(v.user_id);
            self.voice.remove(&v.user_id);
            self.voice_video.remove(&v.user_id);
            if v.user_id == self.me.id {
                self.voice_camera = false;
                self.voice_screen = false;
                self.voice_video.clear();
                self.voice_video_info.clear();
            }
        }
        self.mark_cache_dirty();
        self.sync_overlay();
    }

    /// Pushes the current voice channel into the in-game overlay.
    pub(crate) fn sync_overlay(&self) {
        let Some(channel_id) = self.my_voice_channel() else {
            crate::overlay::sync(crate::overlay::OverlayModel::default(), false);
            return;
        };
        let channel = self
            .channel(channel_id)
            .map(|channel| channel.name.clone())
            .unwrap_or_default();
        let mut users: Vec<crate::overlay::OverlayUser> = self
            .voice
            .values()
            .filter(|state| state.channel_id == Some(channel_id))
            .map(|state| crate::overlay::OverlayUser {
                name: self.user_name(state.user_id),
                speaking: state.user_id == self.me.id && !state.self_mute,
                muted: state.self_mute,
            })
            .collect();
        users.sort_by(|a, b| a.name.cmp(&b.name));
        crate::overlay::sync(
            crate::overlay::OverlayModel {
                channel,
                users,
                muted: self.voice_mute,
            },
            self.overlay_enabled,
        );
    }
}
