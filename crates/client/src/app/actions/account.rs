//! App actions split by feature.

use super::super::*;
use crate::net::Command;
use rune::TextEdit;

impl App {
    pub fn login(&mut self) {
        let username = self.auth_user.text.trim().to_string();
        let password = self.auth_pass.text.clone();
        if username.is_empty() || password.is_empty() {
            self.auth_error = Some("Enter a username and password.".into());
            return;
        }
        self.auth_busy = true;
        self.auth_error = None;
        self.send(Command::Login { username, password });
    }

    pub fn register(&mut self) {
        let username = self.auth_user.text.trim().to_lowercase();
        let password = self.auth_pass.text.clone();
        let display = self.auth_display.text.trim().to_string();
        if username.len() < 2 || password.len() < 4 {
            self.auth_error = Some("Username needs 2+ characters and password 4+.".into());
            return;
        }
        self.auth_busy = true;
        self.auth_error = None;
        let display_name = if display.is_empty() {
            username.clone()
        } else {
            display
        };
        self.send(Command::Register {
            username,
            display_name,
            password,
        });
    }

    pub fn set_dark(&mut self, dark: bool) {
        self.dark = dark;
        save_theme(dark);
    }

    pub fn open_settings(&mut self, tab: SettingsTab) {
        self.settings_tab = tab;
        self.sync_profile_edits();
        self.password_current.clear();
        self.password_new.clear();
        if tab == SettingsTab::Application {
            self.server_url_saved = crate::config::stored_server_url();
            self.server_url = TextEdit::new(self.server_url_saved.clone());
        }
        self.modal = Modal::Settings(tab);
        if tab == SettingsTab::Bots {
            self.send(Command::LoadBots);
            self.send(Command::LoadApplications);
        }
        if tab == SettingsTab::Voice {
            self.refresh_voice_devices();
            self.apply_voice_settings();
        }
    }

    pub fn save_server_url(&mut self) {
        let url = match crate::config::normalize(&self.server_url.text) {
            Ok(url) => url,
            Err(message) => {
                self.server_url_error = Some(message.clone());
                self.toast(message);
                return;
            }
        };
        if self.server_checking {
            return;
        }
        if self.net.is_none() {
            let message = "Runecord can't validate a server without a network session.";
            self.server_url_error = Some(message.into());
            self.toast(message);
            return;
        }
        self.server_url_error = None;
        self.server_checking = true;
        self.pending_server_url = Some(url.clone());
        self.send(Command::CheckServer { url });
    }

    pub fn apply_checked_server(&mut self, url: String, ok: bool, error: Option<String>) {
        if self.pending_server_url.as_deref() != Some(url.as_str()) {
            return;
        }
        self.server_checking = false;
        self.pending_server_url = None;
        if !ok {
            let reason = error.unwrap_or_else(|| "The server did not answer.".into());
            self.server_url_error = Some(format!("Can’t connect to {url}: {reason}"));
            self.toast("Can’t connect to that server");
            return;
        }
        match crate::config::save_server_url(&url) {
            Ok((saved, changed)) => {
                self.server_url = TextEdit::new(saved.clone());
                self.server_url_saved = saved;
                self.server_config_error = None;
                self.server_url_error = None;
                if changed {
                    self.toast("Server saved. Restart Runecord to connect to it.");
                } else {
                    self.toast("That server URL is already saved.");
                }
            }
            Err(message) => {
                self.server_url_error = Some(message.clone());
                self.toast(message);
            }
        }
    }

    pub fn reset_server_url(&mut self) {
        self.server_url = TextEdit::new(crate::config::DEFAULT_SERVER_URL.to_string());
        self.save_server_url();
    }

    pub fn restart_app(&mut self) {
        if rune::is_headless() {
            self.toast("Restart Runecord from the app window.");
            return;
        }
        let Ok(exe) = std::env::current_exe() else {
            self.toast("Could not find the Runecord executable.");
            return;
        };
        let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
        match std::process::Command::new(exe).args(args).spawn() {
            Ok(_) => std::process::exit(0),
            Err(error) => self.toast(format!("Could not restart: {error}")),
        }
    }

    pub fn sync_profile_edits(&mut self) {
        self.profile_display = TextEdit::new(self.me.display_name.clone());
        self.profile_pronouns = TextEdit::new(self.me.pronouns.clone());
        self.profile_badges = self.me.badges.iter().cloned().collect();
        self.profile_decoration = self.me.avatar_decoration.clone();
        self.profile_bio = TextEdit::new(self.me.bio.clone());
        self.profile_status = TextEdit::new(self.me.custom_status.clone());
        let activity = self.me.activity.clone().unwrap_or_default();
        self.profile_activity_kind = TextEdit::new(activity.kind);
        self.profile_activity_title = TextEdit::new(activity.title);
        self.profile_activity_subtitle = TextEdit::new(activity.subtitle);
        self.profile_activity_state = TextEdit::new(activity.state);
        self.profile_activity_url = TextEdit::new(activity.url);
        self.profile_connection_kind.clear();
        self.profile_connection_name.clear();
    }

    pub fn set_profile_effect(&mut self, effect: &str) {
        self.send(Command::UpdateProfileStyle {
            effect: effect.to_string(),
            nameplate: self.me.nameplate.clone(),
            widget: self.me.profile_widget.clone(),
        });
    }

    pub fn set_profile_nameplate(&mut self, nameplate: &str) {
        self.send(Command::UpdateProfileStyle {
            effect: self.me.profile_effect.clone(),
            nameplate: nameplate.to_string(),
            widget: self.me.profile_widget.clone(),
        });
    }

    pub fn set_profile_widget(&mut self, widget: &str) {
        self.send(Command::UpdateProfileStyle {
            effect: self.me.profile_effect.clone(),
            nameplate: self.me.nameplate.clone(),
            widget: widget.to_string(),
        });
    }

    pub fn save_profile_activity(&mut self) {
        let kind = self.profile_activity_kind.text.trim().to_lowercase();
        let title = self.profile_activity_title.text.trim().to_string();
        if kind.is_empty() || title.is_empty() {
            self.toast("Set an activity kind and title");
            return;
        }
        let current = self.me.activity.clone().unwrap_or_default();
        let activity = ProfileActivity {
            kind,
            title,
            subtitle: self.profile_activity_subtitle.text.trim().to_string(),
            state: self.profile_activity_state.text.trim().to_string(),
            url: self.profile_activity_url.text.trim().to_string(),
            artwork_url: current.artwork_url,
            started_at: if current.started_at == 0 {
                now_ms()
            } else {
                current.started_at
            },
        };
        self.send(Command::UpdateProfileExtras {
            activity: Some(activity),
            connections: self.me.connections.clone(),
        });
        self.toast("Activity saved");
    }

    pub fn clear_profile_activity(&mut self) {
        self.send(Command::UpdateProfileExtras {
            activity: None,
            connections: self.me.connections.clone(),
        });
        self.profile_activity_kind.clear();
        self.profile_activity_title.clear();
        self.profile_activity_subtitle.clear();
        self.profile_activity_state.clear();
        self.profile_activity_url.clear();
        self.toast("Activity cleared");
    }

    pub fn upload_activity_artwork(&mut self, path: std::path::PathBuf) {
        self.send(Command::UploadActivityArtwork { path });
    }

    pub fn remove_activity_artwork(&mut self) {
        self.send(Command::RemoveActivityArtwork);
    }

    pub fn add_profile_connection(&mut self) {
        let kind = self.profile_connection_kind.text.trim().to_lowercase();
        let name = self.profile_connection_name.text.trim().to_string();
        if kind.is_empty() || name.is_empty() {
            self.toast("Add a connection kind and name");
            return;
        }
        let mut connections = self.me.connections.clone();
        if connections
            .iter()
            .any(|connection| connection.kind == kind && connection.name == name)
        {
            self.toast("That connection is already listed");
            return;
        }
        connections.push(ProfileConnection { kind, name });
        connections.truncate(6);
        self.send(Command::UpdateProfileExtras {
            activity: self.me.activity.clone(),
            connections,
        });
        self.profile_connection_kind.clear();
        self.profile_connection_name.clear();
    }

    pub fn remove_profile_connection(&mut self, kind: String, name: String) {
        let connections: Vec<ProfileConnection> = self
            .me
            .connections
            .iter()
            .filter(|connection| !(connection.kind == kind && connection.name == name))
            .cloned()
            .collect();
        self.send(Command::UpdateProfileExtras {
            activity: self.me.activity.clone(),
            connections,
        });
    }

    pub fn toggle_profile_badge(&mut self, badge: &str) {
        if !self.profile_badges.remove(badge) {
            self.profile_badges.insert(badge.to_string());
        }
        self.send(Command::UpdateProfile {
            display_name: None,
            pronouns: None,
            badges: Some(self.profile_badges.iter().cloned().collect()),
            avatar_decoration: None,
            custom_status: None,
            bio: None,
            color: None,
            avatar_border: None,
            banner_color: None,
        });
    }

    pub fn set_avatar_decoration(&mut self, decoration: &str) {
        self.profile_decoration = decoration.to_string();
        self.send(Command::UpdateProfile {
            display_name: None,
            pronouns: None,
            badges: None,
            avatar_decoration: Some(decoration.to_string()),
            custom_status: None,
            bio: None,
            color: None,
            avatar_border: None,
            banner_color: None,
        });
    }

    pub fn open_group_dm(&mut self) {
        self.group_dm_selected.clear();
        self.group_dm_name = TextEdit::new("");
        self.modal = Modal::GroupDm;
    }

    pub fn open_dm_settings(&mut self, channel: Id) {
        let Some(dm) = self.dm_for(channel) else {
            return;
        };
        self.dm_settings_name = TextEdit::new(dm.channel.name.clone());
        self.modal = Modal::DmSettings { channel };
    }

    pub fn save_dm_name(&mut self) {
        let Modal::DmSettings { channel } = self.modal else {
            return;
        };
        let name = self.dm_settings_name.text.trim().to_string();
        if name.is_empty() {
            self.toast("Group name can't be empty");
            return;
        }
        self.send(Command::RenameDm { channel, name });
    }

    pub fn begin_dm_icon_crop(&mut self, channel: Id, path: std::path::PathBuf) {
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                self.toast(format!("Could not read image: {error}"));
                return;
            }
        };
        if crate::image_anim::is_animated(&bytes) {
            self.send(Command::UploadDmIcon { channel, path });
            self.toast("Uploading animated group icon.");
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
        self.crop_banner = false;
        self.crop_user = false;
        self.crop_dm = channel;
        self.crop_zoom = 1.0;
        self.crop_offset = (0.0, 0.0);
        self.crop_drag_origin = (0.0, 0.0);
        self.modal = Modal::CropDmIcon { channel };
        self.refresh_crop_preview();
    }

    pub fn remove_dm_icon(&mut self, channel: Id) {
        self.send(Command::RemoveDmIcon { channel });
    }

    pub fn add_dm_member(&mut self, channel: Id, user: Id) {
        self.send(Command::AddDmMember { channel, user });
    }

    pub fn remove_dm_member(&mut self, channel: Id, user: Id) {
        self.send(Command::RemoveDmMember { channel, user });
    }

    pub fn leave_dm(&mut self, channel: Id) {
        self.send(Command::LeaveDm { channel });
        self.modal = Modal::None;
        self.toast("You left the group");
    }

    pub fn toggle_group_dm(&mut self, user: Id) {
        if !self.group_dm_selected.remove(&user) {
            self.group_dm_selected.insert(user);
        }
    }

    pub fn create_group_dm(&mut self) {
        if self.group_dm_selected.len() < 2 {
            self.toast("Pick at least two friends");
            return;
        }
        if self.group_dm_selected.len() > 9 {
            self.toast("Group DMs support up to 10 people");
            return;
        }
        self.send(Command::CreateGroupDm {
            users: self.group_dm_selected.iter().copied().collect(),
            name: self.group_dm_name.text.trim().to_string(),
        });
        self.modal = Modal::None;
    }
}
