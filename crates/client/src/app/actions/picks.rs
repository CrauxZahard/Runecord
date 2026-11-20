//! Routing for files chosen in the native picker.

use super::super::*;
use crate::files::PickKind;
use crate::net::UploadSource;

impl App {
    pub(crate) fn accept_picked_path(&mut self, kind: PickKind, path: std::path::PathBuf) {
        self.accept_source(kind, UploadSource::Path(path));
    }

    fn accept_source(&mut self, kind: PickKind, source: UploadSource) {
        match kind {
            PickKind::Avatar => self.begin_user_image_crop(source, false),
            PickKind::Banner => self.begin_user_image_crop(source, true),
            PickKind::GuildIcon => self.begin_guild_image_crop(source, false),
            PickKind::GuildBanner => self.begin_guild_image_crop(source, true),
            PickKind::ActivityArtwork => self.upload_activity_artwork(source),
            PickKind::RoleIcon { role, .. } => self.upload_role_icon(role, source),
            PickKind::Emoji { sticker, name, .. } => {
                self.emoji_sticker = sticker;
                if !name.is_empty() {
                    self.emoji_name = rune::TextEdit::new(name);
                }
                self.create_emoji_from_path(source);
            }
            PickKind::DmIcon { channel } => self.begin_dm_icon_crop(channel, source),
            PickKind::ChannelUpload { .. } => self.upload_file(source),
            // The AI editor's picker handles its own path in the callback; this
            // arm exists only because the enum is shared.
            PickKind::UiReference => {}
        }
    }
}
