//! Image and user fetching.

use super::*;
use crate::net::Command;
use rune::Pixels;
use std::collections::HashSet;

impl App {
    pub(crate) fn request_channel_images(&mut self, channel: Id) {
        let mut urls = Vec::new();
        if let Some(list) = self.messages.get(&channel) {
            let start = list.len().saturating_sub(40);
            for message in &list[start..] {
                collect_image_urls(message, &mut urls);
            }
        }
        self.request_urls(urls);
    }

    pub fn request_images_for_range(&mut self, channel: Id, start: usize, end: usize) {
        let mut urls = Vec::new();
        if let Some(list) = self.messages.get(&channel) {
            let lo = start.saturating_sub(20);
            let hi = end.saturating_add(20).min(list.len());
            for message in &list[lo.min(list.len())..hi] {
                collect_image_urls(message, &mut urls);
            }
        }
        self.request_urls(urls);
    }

    pub(crate) fn request_message_images(&mut self, message: &Message) {
        let mut urls = Vec::new();
        collect_image_urls(message, &mut urls);
        self.request_urls(urls);
    }

    pub(crate) fn request_urls(&mut self, urls: Vec<String>) {
        for url in urls {
            if self.images.contains_key(&url) || self.image_requests.contains(&url) {
                continue;
            }
            self.image_requests.insert(url.clone());
            self.send(Command::FetchImage { url });
        }
    }

    pub(crate) fn request_guild_images(&mut self, guilds: &[GuildSummary]) {
        let urls: Vec<String> = guilds
            .iter()
            .flat_map(|guild| [guild.icon_url.clone(), guild.banner_url.clone()])
            .filter(|url| !url.is_empty())
            .collect();
        self.request_urls(urls);
    }

    pub(crate) fn request_user(&mut self, user: Id) {
        self.request_users(vec![user]);
    }

    pub(crate) fn request_users(&mut self, users: Vec<Id>) {
        let mut ids = Vec::new();
        for user in users {
            if user == 0 || self.users.contains_key(&user) || !self.user_requests.insert(user) {
                continue;
            }
            ids.push(user);
        }
        for chunk in ids.chunks(100) {
            self.send(Command::LoadUsers {
                users: chunk.to_vec(),
            });
        }
    }

    pub fn request_channel_users(&mut self, channel: Id) {
        let ids: Vec<Id> = self
            .messages
            .get(&channel)
            .map(|list| {
                list.iter()
                    .map(|message| message.author_id)
                    .filter(|id| !self.users.contains_key(id))
                    .collect()
            })
            .unwrap_or_default();
        self.request_users(ids);
    }

    pub(crate) fn insert_image(
        &mut self,
        url: String,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Pixels {
        let pixels = Pixels::new();
        pixels.set(width, height, rgba);
        let bytes = width as usize * height as usize * 4;
        if let Some(old) = self.image_sizes.remove(&url) {
            self.image_bytes = self.image_bytes.saturating_sub(old);
        }
        if self.images.contains_key(&url) {
            self.image_order.retain(|u| u != &url);
        }
        let avatar_owner = self
            .users
            .values()
            .find(|user| !user.avatar_url.is_empty() && user.avatar_url == url)
            .map(|user| user.id);
        self.images.insert(url.clone(), pixels.clone());
        if let Some(user) = avatar_owner {
            crate::ui::common::set_avatar_pixels(user, pixels.clone());
        }
        self.image_sizes.insert(url.clone(), bytes);
        self.image_order.push_back(url.clone());
        self.image_bytes = self.image_bytes.saturating_add(bytes);
        self.image_requests.remove(&url);
        self.trim_images();
        pixels
    }

    pub(crate) fn drop_image(&mut self, url: &str) {
        if let Some(size) = self.image_sizes.remove(url) {
            self.image_bytes = self.image_bytes.saturating_sub(size);
        }
        self.images.remove(url);
        self.animations.remove(url);
        self.image_requests.remove(url);
    }

    pub(crate) fn pinned_images(&self) -> HashSet<String> {
        let mut pinned = HashSet::new();
        if let Some(list) = self.messages.get(&self.current_channel_id()) {
            for message in list {
                for attachment in &message.attachments {
                    if let Some(url) = attachment_image_url(attachment) {
                        pinned.insert(url);
                    }
                }
            }
        }
        pinned
    }

    pub(crate) fn evict_image(&mut self, pinned: &HashSet<String>) -> bool {
        let mut scanned = 0usize;
        loop {
            let Some(url) = self.image_order.pop_front() else {
                return false;
            };
            if !self.images.contains_key(&url) {
                continue;
            }
            if pinned.contains(&url) {
                self.image_order.push_back(url);
                scanned += 1;
                if scanned >= self.image_order.len() {
                    return false;
                }
                continue;
            }
            self.drop_image(&url);
            return true;
        }
    }

    pub(crate) fn trim_images(&mut self) -> bool {
        const MAX_IMAGES: usize = 300;
        const MAX_IMAGE_BYTES: usize = 64 * 1024 * 1024;
        if self.images.len() <= MAX_IMAGES && self.image_bytes <= MAX_IMAGE_BYTES {
            return false;
        }
        let pinned = self.pinned_images();
        let mut changed = false;
        while self.images.len() > MAX_IMAGES || self.image_bytes > MAX_IMAGE_BYTES {
            // Never evict images the active channel is displaying: dropping and
            // immediately refetching them would thrash. They become evictable
            // as soon as the user switches channels.
            if !self.evict_image(&pinned) {
                break;
            }
            changed = true;
        }
        changed
    }
}
