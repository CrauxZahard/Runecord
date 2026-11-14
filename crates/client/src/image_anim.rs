//! Animated GIF and WebP decoding for the shared image cache.

use std::io::Cursor;

/// Largest width or height an animation is scaled to, to bound frame memory.
const MAX_DIMENSION: u32 = 512;
/// Most frames kept per animation.
const MAX_FRAMES: usize = 120;
/// Total decoded frame budget.
const MAX_BYTES: usize = 48 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct AnimFrame {
    pub delay_ms: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub width: u32,
    pub height: u32,
    pub frames: Vec<AnimFrame>,
}

pub fn is_animated(bytes: &[u8]) -> bool {
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return true;
    }
    bytes.len() > 12
        && &bytes[..4] == b"RIFF"
        && &bytes[8..12] == b"WEBP"
        && webp_is_animated(bytes)
}

/// Decodes an animated GIF or WebP. `None` means the caller should treat the bytes as a
/// single still image.
pub fn decode(bytes: &[u8]) -> Option<Animation> {
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        decode_gif(bytes)
    } else if bytes.len() > 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        decode_webp(bytes)
    } else {
        None
    }
}

fn bounded_size(width: u32, height: u32) -> (u32, u32) {
    let largest = width.max(height);
    if largest <= MAX_DIMENSION {
        return (width, height);
    }
    let scale = MAX_DIMENSION as f32 / largest as f32;
    (
        ((width as f32 * scale).round() as u32).max(1),
        ((height as f32 * scale).round() as u32).max(1),
    )
}

fn bounded(width: u32, height: u32, rgba: Vec<u8>) -> Option<(u32, u32, Vec<u8>)> {
    if width == 0 || height == 0 {
        return None;
    }
    let (next_width, next_height) = bounded_size(width, height);
    if next_width == width && next_height == height {
        return Some((width, height, rgba));
    }
    let image = image::RgbaImage::from_raw(width, height, rgba)?;
    let resized = image::imageops::resize(
        &image,
        next_width,
        next_height,
        image::imageops::FilterType::Triangle,
    );
    Some((next_width, next_height, resized.into_raw()))
}

fn decode_gif(bytes: &[u8]) -> Option<Animation> {
    use image::AnimationDecoder;
    let decoder = image::codecs::gif::GifDecoder::new(Cursor::new(bytes)).ok()?;
    let mut frames: Vec<AnimFrame> = Vec::new();
    let mut animation_size: Option<(u32, u32)> = None;
    let mut bytes_used = 0usize;
    for frame in decoder.into_frames().take(MAX_FRAMES) {
        let frame = frame.ok()?;
        let (numer, denom) = frame.delay().numer_denom_ms();
        let delay = numer.checked_div(denom).unwrap_or(100).clamp(20, 2000);
        let buffer = frame.buffer();
        let (width, height, rgba) =
            bounded(buffer.width(), buffer.height(), buffer.as_raw().clone())?;
        let size = animation_size.get_or_insert((width, height));
        if *size != (width, height) {
            return None;
        }
        bytes_used = bytes_used.saturating_add(rgba.len());
        if bytes_used > MAX_BYTES {
            return None;
        }
        frames.push(AnimFrame {
            delay_ms: delay,
            rgba,
        });
    }
    let (width, height) = animation_size?;
    if frames.len() < 2 {
        return None;
    }
    Some(Animation {
        width,
        height,
        frames,
    })
}

fn webp_is_animated(bytes: &[u8]) -> bool {
    image_webp::WebPDecoder::new(Cursor::new(bytes))
        .map(|decoder| decoder.is_animated())
        .unwrap_or(false)
}

fn decode_webp(bytes: &[u8]) -> Option<Animation> {
    let mut decoder = image_webp::WebPDecoder::new(Cursor::new(bytes)).ok()?;
    if !decoder.is_animated() {
        return None;
    }
    let (width, height) = decoder.dimensions();
    let buffer_size = decoder.output_buffer_size()?;
    let alpha = decoder.has_alpha();
    let (animation_width, animation_height) = bounded_size(width, height);
    let mut frames = Vec::new();
    let mut bytes_used = 0usize;
    for _ in 0..decoder.num_frames().min(MAX_FRAMES as u32) {
        let mut buffer = vec![0u8; buffer_size];
        let delay = decoder.read_frame(&mut buffer).ok()?;
        let rgba = if alpha {
            buffer
        } else {
            let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
            for pixel in buffer.as_chunks::<3>().0 {
                rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
            }
            rgba
        };
        let (frame_width, frame_height, rgba) = bounded(width, height, rgba)?;
        if (frame_width, frame_height) != (animation_width, animation_height) {
            return None;
        }
        bytes_used = bytes_used.saturating_add(rgba.len());
        if bytes_used > MAX_BYTES {
            return None;
        }
        frames.push(AnimFrame {
            delay_ms: delay.clamp(20, 2000),
            rgba,
        });
    }
    if frames.len() < 2 {
        return None;
    }
    Some(Animation {
        width: animation_width,
        height: animation_height,
        frames,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gif_bytes() -> Vec<u8> {
        use image::{Frame, Rgba, RgbaImage};
        let mut out = Vec::new();
        {
            let mut encoder = image::codecs::gif::GifEncoder::new(&mut out);
            for value in [40, 120, 200] {
                let frame = Frame::new(RgbaImage::from_pixel(8, 8, Rgba([value, 0, 0, 255])));
                encoder.encode_frame(frame).expect("gif frame");
            }
        }
        out
    }

    #[test]
    fn decodes_animated_gif_frames_and_delays() {
        let bytes = gif_bytes();
        assert!(is_animated(&bytes));
        let animation = decode(&bytes).expect("animated gif");
        assert_eq!(animation.frames.len(), 3);
        assert_eq!((animation.width, animation.height), (8, 8));
        assert!(
            animation
                .frames
                .iter()
                .all(|frame| frame.rgba.len() == 8 * 8 * 4)
        );
    }

    #[test]
    fn decodes_animated_webp_fixture() {
        let bytes = include_bytes!("../tests/fixtures/anim.webp");
        assert!(is_animated(bytes));
        let animation = decode(bytes).expect("animated webp");
        assert_eq!(animation.frames.len(), 2);
        assert!(animation.width > 0 && animation.height > 0);
    }

    #[test]
    fn static_png_is_not_animated() {
        let image = image::RgbaImage::from_pixel(4, 4, image::Rgba([1, 2, 3, 255]));
        let mut bytes = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .expect("png");
        assert!(!is_animated(&bytes));
        assert!(decode(&bytes).is_none());
    }
}
