//! Network event handlers: bots.

use super::super::*;
use crate::net::Command;

impl App {
    pub(crate) fn apply_bots(&mut self, bots: Vec<User>) {
        self.bots = bots;
    }
    pub(crate) fn apply_bot_created(&mut self, bot: User, token: String) {
        self.users.insert(bot.id, bot.clone());
        self.bots.insert(0, bot.clone());
        self.token_reveal = Some((format!("{} bot token", bot.display_name), token));
        self.toast("Bot created. Copy the token now; it is shown once.");
    }
    pub(crate) fn apply_applications(&mut self, apps: Vec<Application>) {
        self.applications = apps;
    }
    pub(crate) fn apply_application_created(&mut self, application: Application, token: String) {
        self.applications.insert(0, application.clone());
        self.token_reveal = Some((format!("{} token", application.name), token));
        self.toast("Application created. Copy the token now; it is shown once.");
    }
    pub(crate) fn apply_install_link(&mut self, code: String) {
        self.install_code = rune::TextEdit::new(code.clone());
        if let Some(app) = self.pending_install_app.take() {
            if self.active_guild != 0 {
                self.send(Command::InstallApplication {
                    code: code.clone(),
                    guild: self.active_guild,
                });
                self.toast("Installing application in this server…");
            } else {
                self.toast("Select a server before installing");
            }
        } else {
            self.toast("Install code ready");
        }
    }
    pub(crate) fn apply_application_installed(&mut self, application: Application) {
        if let Some(existing) = self
            .applications
            .iter_mut()
            .find(|item| item.id == application.id)
        {
            *existing = application;
        } else {
            self.applications.push(application);
        }
        self.mark_cache_dirty();
    }
}
