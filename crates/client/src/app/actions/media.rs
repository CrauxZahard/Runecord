//! App actions split by feature.

use super::super::*;
use crate::net::Command;

impl App {
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

    pub(crate) fn crop_window(&self) -> Option<(f32, f32, f32, f32)> {
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

    pub(crate) fn clamp_crop_offset(&mut self) {
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

    pub(crate) fn render_crop(&self, out_width: u32, out_height: u32) -> Option<Vec<u8>> {
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

    pub(crate) fn refresh_crop_preview(&mut self) {
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
}
