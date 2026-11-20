//! File picking for uploads.
//!
//! Rune's native dialog hands back a path, and the upload starts from it.

use runecord_protocol::Id;

/// What a picked file is for. The queue carries this so the poll loop can route
/// the bytes to the right upload without the UI having to remember it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PickKind {
    Avatar,
    Banner,
    GuildIcon,
    GuildBanner,
    ActivityArtwork,
    RoleIcon {
        guild: Id,
        role: Id,
    },
    Emoji {
        guild: Id,
        sticker: bool,
        name: String,
    },
    DmIcon {
        channel: Id,
    },
    ChannelUpload {
        channel: Id,
    },
    /// A reference image for the AI UI editor; nothing is uploaded to a server.
    UiReference,
}

/// Opens the native picker and calls `on_path` with the chosen path.
pub fn open(
    tx: &mut rune::Tx,
    _kind: PickKind,
    title: &str,
    extensions: &'static [&'static str],
    on_path: impl FnOnce(&mut crate::app::App, std::path::PathBuf) + 'static,
) {
    let options = rune::FileDialogOptions {
        title: title.to_string(),
        directory: None,
        file_name: None,
        filters: Vec::new(),
    }
    .filter("Files", extensions);
    tx.open_file(options, move |s: &mut crate::app::App, result, _| {
        if let Ok(Some(path)) = result {
            on_path(s, path);
        }
    });
}
