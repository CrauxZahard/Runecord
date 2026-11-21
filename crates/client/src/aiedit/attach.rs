//! Reference images the user attaches to an edit.
//!
//! The model never needs the original file: a reference is decoded once, shrunk
//! to a size a vision model accepts, re-encoded, and kept both as provider bytes
//! and as [`Pixels`] for the thumbnail in the settings card. Keeping the decoded
//! preview here (rather than in the shared image cache) means evicting chat
//! images cannot blank the attachment row.

use std::path::Path;

use rune::Pixels;

use super::provider;

/// More than a few references is more tokens than most providers accept.
pub const MAX_COUNT: usize = 4;
/// A reference larger than this is almost certainly a photo, not a UI sample.
const MAX_INPUT_BYTES: usize = 12 * 1024 * 1024;
/// Longest side sent to the provider. Enough to read a layout; small enough that
/// four images still fit in a chat request.
const MAX_SIDE: u32 = 1_400;

/// One attached reference image.
#[derive(Clone, Debug)]
pub struct Attachment {
    /// The file name as the user picked it, for the chip label.
    pub name: String,
    /// `image/png` or `image/jpeg`, matching [`bytes`](Self::bytes).
    pub media_type: String,
    /// The downscaled image the provider receives.
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// The same image as RGBA, for the preview tile.
    pub pixels: Pixels,
}

impl Attachment {
    /// The provider-facing form, without the preview buffer.
    pub fn image(&self) -> provider::Image {
        provider::Image {
            media_type: self.media_type.clone(),
            bytes: self.bytes.clone(),
        }
    }
}

/// Reads and normalises a file chosen in the native picker.
pub fn from_path(path: &Path) -> Result<Attachment, String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("reference image")
        .to_string();
    from_bytes(name, bytes)
}

/// Normalises bytes pasted from the clipboard or dropped on the prompt.
pub fn from_bytes(name: String, bytes: Vec<u8>) -> Result<Attachment, String> {
    if bytes.is_empty() {
        return Err("That reference image is empty.".into());
    }
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(format!(
            "That image is {:.1} MB; attach one under {} MB.",
            bytes.len() as f64 / (1024.0 * 1024.0),
            MAX_INPUT_BYTES / (1024 * 1024)
        ));
    }
    let decoded = image::load_from_memory(&bytes)
        .map_err(|error| format!("Could not read that image: {error}"))?;
    let (source_w, source_h) = (decoded.width(), decoded.height());
    if source_w < 8 || source_h < 8 {
        return Err("That image is too small to use as a reference.".into());
    }
    let scaled = if source_w.max(source_h) > MAX_SIDE {
        decoded.resize(MAX_SIDE, MAX_SIDE, image::imageops::FilterType::Lanczos3)
    } else {
        decoded
    };
    let has_alpha = scaled.color().has_alpha();
    let rgba = scaled.to_rgba8();
    let (width, height) = rgba.dimensions();
    // A decoder may hand back RGBA even when every pixel is opaque; only a
    // transparent pixel needs the larger PNG form.
    let has_alpha = has_alpha && rgba.pixels().any(|pixel| pixel[3] < 255);

    let mut encoded = std::io::Cursor::new(Vec::new());
    let (format, media_type) = if has_alpha {
        (image::ImageFormat::Png, "image/png")
    } else {
        (image::ImageFormat::Jpeg, "image/jpeg")
    };
    scaled
        .write_to(&mut encoded, format)
        .map_err(|error| format!("Could not prepare that image: {error}"))?;

    // The tile is 38px; keeping the 1400px provider copy in memory for all four
    // attachments would cost tens of megabytes for no visible gain.
    let preview_source = if scaled.width().max(scaled.height()) > 128 {
        scaled.resize(128, 128, image::imageops::FilterType::Triangle)
    } else {
        scaled.clone()
    };
    let preview = preview_source.to_rgba8();
    let (preview_w, preview_h) = preview.dimensions();
    let pixels = Pixels::new();
    pixels.set(preview_w, preview_h, preview.into_raw());

    Ok(Attachment {
        name,
        media_type: media_type.to_string(),
        bytes: encoded.into_inner(),
        width,
        height,
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use super::from_bytes;

    /// A 32x32 PNG, so the decoder path runs without a fixture file.
    fn tiny_png() -> Vec<u8> {
        let image = image::RgbaImage::from_pixel(32, 32, image::Rgba([200, 30, 60, 255]));
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut out, image::ImageFormat::Png)
            .expect("encode test png");
        out.into_inner()
    }

    #[test]
    fn an_opaque_image_is_reencoded_as_jpeg() {
        let attachment = from_bytes("reference.png".into(), tiny_png()).expect("reads image");
        assert_eq!(attachment.media_type, "image/jpeg");
        assert!(!attachment.bytes.is_empty());
        assert_eq!((attachment.width, attachment.height), (32, 32));
    }

    #[test]
    fn junk_is_rejected_with_a_readable_error() {
        let error = from_bytes("broken.png".into(), vec![1, 2, 3]).unwrap_err();
        assert!(error.contains("Could not read that image"), "{error}");
    }

    #[test]
    fn an_empty_attachment_is_rejected() {
        assert!(from_bytes("empty.png".into(), Vec::new()).is_err());
    }
}
