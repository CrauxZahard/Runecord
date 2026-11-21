//! App actions split by feature.

use super::super::*;
use crate::app::save_export;
use crate::net::Command;
use rune::TextEdit;

impl App {
    pub fn apply_voice_settings(&self) {
        crate::voice::set_input_device(self.voice_input_device.clone());
        crate::voice::set_camera_device(self.voice_camera_device.clone());
        crate::voice::set_output_device(self.voice_output_device.clone());
        crate::voice::set_input_sensitivity(self.voice_sensitivity);
        crate::voice::set_auto_sensitivity(self.voice_auto_sensitivity);
        crate::voice::set_input_volume(self.voice_input_volume);
        crate::voice::set_output_volume(self.voice_output_volume);
        crate::voice::set_voice_activity(self.voice_mode == "voice");
        crate::voice::set_push_to_talk(self.voice_mode == "ptt");
        crate::voice::set_noise_suppression(self.voice_noise_suppression);
        crate::voice::set_push_to_talk_key(self.ptt_key);
    }

    pub fn set_noise_suppression(&mut self, enabled: bool) {
        self.voice_noise_suppression = enabled;
        crate::voice::set_noise_suppression(enabled);
        self.mark_cache_dirty();
    }

    pub fn set_push_to_talk_key(&mut self, code: u32) {
        self.ptt_key = code;
        self.ptt_capture = false;
        crate::voice::set_push_to_talk_key(code);
        self.mark_cache_dirty();
    }

    pub fn refresh_voice_devices(&mut self) {
        let (audio, video) = crate::voice::list_devices();
        self.audio_devices = audio;
        self.video_devices = video;
        self.output_devices = crate::voice::list_output_devices();
    }

    pub fn select_voice_device(&mut self, kind: &str, device: String) {
        match kind {
            "input" => self.voice_input_device = device,
            "output" => self.voice_output_device = device,
            "camera" => self.voice_camera_device = device,
            _ => {}
        }
        self.apply_voice_settings();
        self.mark_cache_dirty();
        self.modal = Modal::Settings(SettingsTab::Voice);
    }

    pub fn set_voice_sensitivity(&mut self, value: f32) {
        self.voice_sensitivity = value.clamp(0.001, 0.5);
        crate::voice::set_input_sensitivity(self.voice_sensitivity);
        self.mark_cache_dirty();
    }

    pub fn set_voice_auto_sensitivity(&mut self, enabled: bool) {
        self.voice_auto_sensitivity = enabled;
        crate::voice::set_auto_sensitivity(enabled);
        self.mark_cache_dirty();
    }

    pub fn set_voice_volume(&mut self, input: bool, value: f32) {
        if input {
            self.voice_input_volume = value.clamp(0.0, 1.0);
            crate::voice::set_input_volume(self.voice_input_volume);
        } else {
            self.voice_output_volume = value.clamp(0.0, 1.0);
            crate::voice::set_output_volume(self.voice_output_volume);
        }
        self.mark_cache_dirty();
    }

    pub fn set_voice_mode(&mut self, mode: &str) {
        self.voice_mode = mode.to_string();
        crate::voice::set_voice_activity(mode == "voice");
        crate::voice::set_push_to_talk(mode == "ptt");
        self.mark_cache_dirty();
    }

    pub fn toggle_mic_test(&mut self) {
        if crate::voice::mic_testing() {
            crate::voice::stop_mic_test();
        } else {
            crate::voice::start_mic_test();
        }
    }

    pub fn toggle_camera_test(&mut self) {
        if crate::voice::camera_testing() {
            crate::voice::stop_camera_test();
        } else {
            crate::voice::start_camera_test();
        }
    }

    pub fn save_profile(&mut self) {
        let display_name = self.profile_display.text.trim().to_string();
        let pronouns = self.profile_pronouns.text.trim().to_string();
        let bio = self.profile_bio.text.trim().to_string();
        let custom_status = self.profile_status.text.trim().to_string();
        if display_name.is_empty() {
            self.toast("Display name cannot be empty");
            return;
        }
        self.send(Command::UpdateProfile {
            display_name: Some(display_name),
            pronouns: Some(pronouns),
            badges: Some(self.profile_badges.iter().cloned().collect()),
            avatar_decoration: Some(self.profile_decoration.clone()),
            custom_status: Some(custom_status),
            bio: Some(bio),
            color: Some(self.profile_color),
            avatar_border: Some(self.profile_border),
            banner_color: Some(self.profile_banner_color),
        });
        self.send(Command::UpdateProfileStyle {
            effect: self.profile_effect.clone(),
            nameplate: self.profile_nameplate.clone(),
            widget: self.profile_widget.clone(),
        });
        self.toast("Profile saved");
    }

    /// Whether the profile drafts differ from the saved account.
    pub fn profile_dirty(&self) -> bool {
        self.profile_display.text.trim() != self.me.display_name
            || self.profile_pronouns.text.trim() != self.me.pronouns
            || self.profile_bio.text.trim() != self.me.bio
            || self.profile_status.text.trim() != self.me.custom_status
            || self.profile_badges.len() != self.me.badges.len()
            || self
                .me
                .badges
                .iter()
                .any(|badge| !self.profile_badges.contains(badge))
            || self.profile_decoration != self.me.avatar_decoration
            || self.profile_effect != self.me.profile_effect
            || self.profile_nameplate != self.me.nameplate
            || self.profile_widget != self.me.profile_widget
            || self.profile_color != self.me.avatar_color
            || self.profile_border != self.me.avatar_border
            || self.profile_banner_color != self.me.banner_color
    }

    pub fn set_avatar_color(&mut self, color: u32) {
        self.profile_color = color;
    }

    pub fn set_avatar_border(&mut self, color: u32) {
        self.profile_border = color;
    }

    pub fn set_banner_color(&mut self, color: u32) {
        self.profile_banner_color = color;
    }

    pub fn select_settings_tab(&mut self, tab: SettingsTab) {
        self.settings_tab = tab;
        // The view renders the tab stored in the modal; without this the nav
        // click only moved a field nobody read, so settings felt unclickable.
        self.modal = Modal::Settings(tab);
        if tab == SettingsTab::Bots {
            self.send(Command::LoadBots);
            self.send(Command::LoadApplications);
        }
    }

    pub fn change_password(&mut self) {
        let current = self.password_current.text.clone();
        let new = self.password_new.text.clone();
        if current.is_empty() || new.is_empty() {
            self.toast("Enter your current and new password");
            return;
        }
        self.send(Command::ChangePassword { current, new });
        self.password_current.clear();
        self.password_new.clear();
    }

    pub fn toggle_compact(&mut self, compact: bool) {
        self.compact = compact;
        self.mark_cache_dirty();
    }

    pub fn toggle_developer(&mut self, developer: bool) {
        self.developer = developer;
        self.mark_cache_dirty();
    }

    pub fn reset_preferences(&mut self) {
        self.set_dark(true);
        self.notify_dm = true;
        self.notify_mentions = true;
        self.compact = false;
        self.developer = false;
        self.keybinds = default_keybinds();
        self.recording_key = None;
        self.mark_cache_dirty();
        self.toast("Preferences reset");
    }

    pub fn begin_keybind(&mut self, action: &str) {
        self.recording_key = Some(action.to_string());
    }

    pub fn set_keybind(&mut self, action: &str, chord: &str) {
        for (other, value) in self.keybinds.iter_mut() {
            if other != action && value == chord {
                value.clear();
            }
        }
        self.keybinds.insert(action.to_string(), chord.to_string());
        self.mark_cache_dirty();
    }

    pub fn reset_keybinds(&mut self) {
        self.keybinds = default_keybinds();
        self.recording_key = None;
        self.mark_cache_dirty();
        self.toast("Keybinds reset");
    }

    pub fn open_server_settings(&mut self) {
        // Discord hides server settings from members without any management
        // permission; don't open (and don't let them edit) when unauthorized.
        if !self.can_open_server_settings() {
            self.toast("You don't have permission to manage this server.");
            return;
        }
        self.modal = Modal::ServerSettings;
        self.server_tab = ServerTab::Overview;
        if !self.can_manage_tab(self.server_tab) {
            self.server_tab = [
                ServerTab::Roles,
                ServerTab::Members,
                ServerTab::Bans,
                ServerTab::Audit,
            ]
            .into_iter()
            .find(|tab| self.can_manage_tab(*tab))
            .unwrap_or(ServerTab::Overview);
        }
        self.sync_server_overview();
        self.ensure_guild(self.active_guild);
        if self.active_guild != 0 {
            self.send(Command::LoadEmojis {
                guild: self.active_guild,
            });
        }
        let first = self.guild_roles(self.active_guild).first().map(|r| r.id);
        self.selected_role = first.unwrap_or(0);
        if let Some(id) = first {
            self.select_role(id);
        }
        if let Some(member) = self.members.get(&self.active_guild).and_then(|m| m.first()) {
            self.selected_member = Some(member.user.id);
        }
    }

    pub fn sync_server_overview(&mut self) {
        let Some(guild) = self.active_guild().cloned() else {
            return;
        };
        self.overview_name = TextEdit::new(guild.name);
        self.overview_description = TextEdit::new(guild.description);
        self.overview_tag = TextEdit::new(guild.tag);
        self.overview_color = guild.icon_color;
        self.overview_discoverable = guild.discoverable;
    }

    pub fn overview_dirty(&self) -> bool {
        let Some(guild) = self.active_guild() else {
            return false;
        };
        self.overview_name.text.trim() != guild.name
            || self.overview_description.text.trim() != guild.description
            || self.overview_tag.text.trim() != guild.tag
            || self.overview_color != guild.icon_color
            || self.overview_discoverable != guild.discoverable
    }

    pub fn save_server_overview(&mut self) {
        let name = self.overview_name.text.trim().to_string();
        if name.is_empty() {
            self.toast("Name the server first");
            return;
        }
        self.send(Command::UpdateGuildSettings {
            guild: self.active_guild,
            name,
            description: self.overview_description.text.trim().to_string(),
            color: self.overview_color,
            tag: self.overview_tag.text.trim().to_string(),
            discoverable: self.overview_discoverable,
        });
        self.toast("Server updated");
    }

    pub fn select_server_tab(&mut self, tab: ServerTab) {
        if !self.can_manage_tab(tab) {
            self.toast("You don't have permission to view that section.");
            return;
        }
        self.server_tab = tab;
        match tab {
            ServerTab::Audit => self.load_audit(false),
            ServerTab::Bans => {
                self.bans.clear();
                self.appeals.clear();
                self.send(Command::LoadBans {
                    guild: self.active_guild,
                });
                self.send(Command::LoadAppeals {
                    guild: self.active_guild,
                });
            }
            ServerTab::Overview => self.sync_server_overview(),
            ServerTab::AutoMod => self.send(Command::LoadAutoMod {
                guild: self.active_guild,
            }),
            ServerTab::Templates => self.send(Command::LoadTemplates {
                guild: self.active_guild,
            }),
            ServerTab::Integrations => {
                self.webhook_token = None;
                self.send(Command::LoadWebhooks {
                    guild: self.active_guild,
                });
            }
            _ => {}
        }
    }

    pub fn load_audit(&mut self, older: bool) {
        if self.active_guild == 0 || self.audit_loading {
            return;
        }
        let before = if older {
            self.audit.last().map(|entry| entry.id)
        } else {
            None
        };
        if older && self.audit_complete {
            return;
        }
        self.audit_loading = true;
        if !older {
            self.audit.clear();
            self.audit_complete = false;
        }
        self.send(Command::LoadAudit {
            guild: self.active_guild,
            before,
            actor: self.audit_actor,
            action: self.audit_action,
        });
    }

    pub fn set_audit_actor(&mut self, actor: Option<Id>) {
        self.audit_actor = actor;
        self.load_audit(false);
    }

    pub fn set_audit_action(&mut self, action: Option<i32>) {
        self.audit_action = action;
        self.load_audit(false);
    }

    pub fn export_audit(&mut self) {
        let mut csv = String::from("id,action,actor,target,created_at\n");
        for entry in &self.audit {
            let target = entry.target_id.map(|id| id.to_string()).unwrap_or_default();
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                entry.id, entry.action_type, entry.actor_id, target, entry.created_at
            ));
        }
        match save_export("audit-log.csv", &csv) {
            Ok(path) => self.toast(format!("Saved {path}")),
            Err(e) => self.toast(format!("Export failed: {e}")),
        }
    }

    pub fn select_role(&mut self, id: Id) {
        let Some(role) = self
            .guild_roles(self.active_guild)
            .iter()
            .find(|role| role.id == id)
            .cloned()
        else {
            return;
        };
        self.selected_role = id;
        self.role_name = TextEdit::new(role.name);
        self.role_color = role.color;
        self.role_permissions = role.permissions;
        self.role_hoist = role.hoist;
        self.role_mentionable = role.mentionable;
        self.role_link_label.clear();
        self.role_link_url.clear();
        self.send(Command::LoadRoleLinks {
            guild: self.active_guild,
            role: id,
        });
    }

    pub fn upload_role_icon(&mut self, role: Id, source: crate::net::UploadSource) {
        self.send(Command::UploadRoleIcon {
            guild: self.active_guild,
            role,
            source,
        });
    }

    pub fn remove_role_icon(&mut self, role: Id) {
        self.send(Command::RemoveRoleIcon {
            guild: self.active_guild,
            role,
        });
    }

    pub fn add_role_link(&mut self, role: Id) {
        let label = self.role_link_label.text.trim().to_string();
        let url = self.role_link_url.text.trim().to_string();
        if label.is_empty() || url.is_empty() {
            self.toast("Add a label and an https:// link");
            return;
        }
        self.send(Command::AddRoleLink {
            guild: self.active_guild,
            role,
            label,
            url,
        });
        self.role_link_label.clear();
        self.role_link_url.clear();
    }

    pub fn delete_role_link(&mut self, role: Id, link: Id) {
        self.send(Command::DeleteRoleLink {
            guild: self.active_guild,
            role,
            link,
        });
    }

    pub fn create_role_now(&mut self) {
        let name = format!("new role {}", self.guild_roles(self.active_guild).len());
        self.send(Command::CreateRole {
            guild: self.active_guild,
            name,
            color: 0x99aab5,
            permissions: 0,
        });
    }

    pub fn save_role(&mut self) {
        if self.selected_role == 0 {
            self.toast("Select a role first");
            return;
        }
        let name = self.role_name.text.trim().to_string();
        if name.is_empty() {
            self.toast("Role name cannot be empty");
            return;
        }
        let is_everyone = self
            .guild_roles(self.active_guild)
            .iter()
            .any(|role| role.id == self.selected_role && role.name == "@everyone");
        self.send(Command::UpdateRole {
            guild: self.active_guild,
            role: self.selected_role,
            name: (!is_everyone).then_some(name),
            color: Some(self.role_color),
            permissions: Some(self.role_permissions),
            hoist: Some(self.role_hoist),
            mentionable: Some(self.role_mentionable),
            position: None,
        });
    }

    pub fn delete_selected_role(&mut self) {
        let role = self.selected_role;
        if role == 0 {
            return;
        }
        if self
            .guild_roles(self.active_guild)
            .iter()
            .any(|item| item.id == role && item.name == "@everyone")
        {
            self.toast("@everyone cannot be deleted");
            return;
        }
        self.send(Command::DeleteRole {
            guild: self.active_guild,
            role,
        });
    }

    pub fn move_selected_role(&mut self, delta: i32) {
        let guild = self.active_guild;
        let mut roles: Vec<Role> = self.guild_roles(guild).to_vec();
        roles.sort_by_key(|role| std::cmp::Reverse(role.position));
        let Some(index) = roles.iter().position(|role| role.id == self.selected_role) else {
            return;
        };
        let target = index as i32 + delta;
        if target < 0 || target >= roles.len() as i32 {
            return;
        }
        let target = target as usize;
        if roles[target].name == "@everyone" || roles[index].name == "@everyone" {
            return;
        }
        let a = roles[index].clone();
        let b = roles[target].clone();
        let (a_id, b_id) = (a.id, b.id);
        let (a_pos, b_pos) = (a.position, b.position);
        for (role, position) in [(a_id, b_pos), (b_id, a_pos)] {
            self.send(Command::UpdateRole {
                guild,
                role,
                name: None,
                color: None,
                permissions: None,
                hoist: None,
                mentionable: None,
                position: Some(position),
            });
        }
    }

    pub fn select_member(&mut self, id: Id) {
        self.selected_member = Some(id);
    }

    pub fn toggle_member_role(&mut self, role: Id) {
        let Some(user) = self.selected_member else {
            self.toast("Select a member first");
            return;
        };
        self.toggle_role_for_member(role, user);
    }

    pub fn toggle_role_for_member(&mut self, role: Id, user: Id) {
        let guild = self.active_guild;
        let Some(member) = self
            .members
            .get(&guild)
            .and_then(|list| list.iter().find(|member| member.user.id == user))
            .cloned()
        else {
            return;
        };
        let mut roles = member.roles;
        if let Some(index) = roles.iter().position(|id| *id == role) {
            roles.remove(index);
        } else {
            roles.push(role);
        }
        self.send(Command::UpdateMemberRoles { guild, user, roles });
    }

    pub fn apply_slash_command(&mut self, name: &str) {
        let Some(command) = self
            .commands
            .iter()
            .find(|command| command.name.eq_ignore_ascii_case(name))
            .cloned()
        else {
            return;
        };
        let options = command
            .options
            .get("options")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        self.slash_values = options.iter().map(|_| TextEdit::new("")).collect();
        self.slash_command = Some(command);
        self.modal = Modal::SlashCommand;
    }

    pub fn run_slash_command(&mut self) {
        let Some(command) = self.slash_command.clone() else {
            return;
        };
        let schema = command
            .options
            .get("options")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        let mut values = serde_json::Map::new();
        for (index, option) in schema.iter().enumerate() {
            let name = option
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or("value");
            let value = self
                .slash_values
                .get(index)
                .map(|edit| edit.text.trim().to_string())
                .unwrap_or_default();
            if value.is_empty()
                && option
                    .get("required")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false)
            {
                self.toast(format!("Fill in {name}"));
                return;
            }
            if !value.is_empty() {
                values.insert(name.to_string(), serde_json::Value::String(value));
            }
        }
        let channel = match self.screen {
            Screen::Guild => self.active_channel,
            Screen::Dms => self.active_dm,
        };
        if channel == 0 {
            return;
        }
        let response = command
            .options
            .get("response")
            .and_then(|value| value.as_str())
            .unwrap_or("Command ran.")
            .to_string();
        self.send(Command::InvokeCommand {
            channel,
            command: command.name,
            options: serde_json::Value::Object(values),
            response,
        });
        self.modal = Modal::None;
        self.composer.clear();
        self.slash_command = None;
        self.slash_values.clear();
    }

    pub fn create_emoji_from_path(&mut self, source: crate::net::UploadSource) {
        let stem = match &source {
            crate::net::UploadSource::Path(path) => path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("emoji")
                .to_string(),
            crate::net::UploadSource::Bytes { name, .. } => name
                .rsplit_once('.')
                .map(|(stem, _)| stem)
                .unwrap_or("emoji")
                .to_string(),
        };
        let name = if self.emoji_name.text.trim().is_empty() {
            stem.to_lowercase()
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect::<String>()
        } else {
            self.emoji_name.text.trim().to_lowercase()
        };
        if name.chars().count() < 2 {
            self.toast("Emoji name needs at least two characters");
            return;
        }
        self.send(Command::CreateEmoji {
            guild: self.active_guild,
            name,
            sticker: self.emoji_sticker,
            source,
        });
        self.emoji_name.clear();
    }

    pub fn delete_emoji(&mut self, id: Id) {
        self.send(Command::DeleteEmoji {
            guild: self.active_guild,
            id,
        });
    }

    pub fn apply_role(&mut self, role: Role) {
        if let Some(detail) = self.guild_details.get_mut(&role.guild_id) {
            if let Some(existing) = detail.roles.iter_mut().find(|item| item.id == role.id) {
                *existing = role;
            } else {
                detail.roles.push(role);
            }
            detail
                .roles
                .sort_by_key(|item| std::cmp::Reverse(item.position));
        }
        self.mark_cache_dirty();
    }

    pub fn create_bot(&mut self) {
        let name = self.bot_name.text.trim().to_string();
        if name.chars().count() < 2 {
            self.toast("Bot name needs at least two characters");
            return;
        }
        let guild = if self.add_bot_to_guild && self.active_guild != 0 {
            Some(self.active_guild)
        } else {
            None
        };
        self.send(Command::CreateBot { name, guild });
        self.bot_name.clear();
    }

    pub fn revoke_bot(&mut self, id: Id) {
        self.send(Command::DeleteBot { id });
    }

    pub fn create_application(&mut self) {
        let name = self.app_name.text.trim().to_string();
        if name.chars().count() < 2 {
            self.toast("Application name needs at least two characters");
            return;
        }
        let description = self.app_description.text.trim().to_string();
        self.send(Command::CreateApplication { name, description });
        self.app_name.clear();
        self.app_description.clear();
    }

    pub fn delete_application(&mut self, id: Id) {
        self.send(Command::DeleteApplication { id });
    }

    pub fn reveal_application_token(&mut self, id: Id) {
        let name = self
            .applications
            .iter()
            .find(|app| app.id == id)
            .map(|app| app.name.clone())
            .unwrap_or_else(|| "Application".to_string());
        self.send(Command::ApplicationToken { id, name });
    }

    pub fn create_install_link(&mut self, id: Id) {
        self.pending_install_app = Some(id);
        self.send(Command::ApplicationInstallLink { id });
    }

    pub fn install_by_code(&mut self) {
        let code = self.install_code.text.trim().to_string();
        if code.is_empty() {
            self.toast("Paste an install code first");
            return;
        }
        if self.active_guild == 0 {
            self.toast("Select a server before installing");
            return;
        }
        self.send(Command::InstallApplication {
            code,
            guild: self.active_guild,
        });
        self.install_code.clear();
    }

    pub fn logout(&mut self) {
        crate::voice::shutdown();
        self.send(Command::Logout);
        crate::cache::clear(&self.cache_profile);
        self.logged_in = false;
        self.connecting = false;
        self.auth_busy = false;
        self.auth_pass.clear();
        self.me = User::default();
        self.messages.clear();
        self.guilds.clear();
        self.guild_details.clear();
        self.channels.clear();
        self.members.clear();
        self.member_state.clear();
        self.dms.clear();
        self.relationships.clear();
        self.images.clear();
        self.image_sizes.clear();
        self.image_order.clear();
        self.image_bytes = 0;
        self.image_requests.clear();
        self.user_requests.clear();
        self.visible_messages.clear();
        self.loading_older.clear();
        self.older_complete.clear();
        self.channel_seen.clear();
        self.guild_seen.clear();
        self.member_seen.clear();
        self.stale_guilds.clear();
        self.viewing_image = None;
        self.drop_hover = false;
        self.member_search_due = 0.0;
        self.member_search_live.clear();
        self.last_member_query.clear();
        self.unread.clear();
        self.reads.clear();
        self.presence.clear();
        self.voice.clear();
        self.typing.clear();
        self.cache_dirty_at = None;
        self.bootstrap_pending = false;
        self.modal = Modal::None;
    }

    // -----------------------------------------------------------------------
    // Poll
    // -----------------------------------------------------------------------
}
