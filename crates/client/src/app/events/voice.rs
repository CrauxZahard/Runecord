//! Network event handlers: voice.

use super::super::*;

impl App {
    pub(crate) fn apply_voice_state(&mut self, v: VoiceState) {
        if v.channel_id.is_some() {
            self.voice.insert(v.user_id, v);
        } else {
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
    }
}
