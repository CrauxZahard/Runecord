//! Application state, live server data, and the poll loop that lands network events.

use crate::net::{self, Command, Net};
use rune::{Key, KeyEvent, Pixels, TextEdit, Tx};
use runecord_protocol::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Guild,
    Dms,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Modal {
    None,
    Settings(SettingsTab),
    CreateServer,
    CreateChannel,
    EditChannel(Id),
    ConfirmDeleteChannel(Id),
    Invite,
    AddFriend,
    Search,
    Pins,
    QuickSwitcher,
    ServerMenu,
    Profile(Id),
    FullProfile(Id),
    EmojiPicker,
    Reaction(Id),
    ThreadCreate { parent: Id, message: Option<Id> },
    Threads,
    EditMessage(Id),
    MessageMenu(Id),
    ConfirmDelete(Id),
    ConfirmLeave(Id),
    ChannelNotifications(Id),
    ServerNotifications,
    NotifyChannelPicker,
    Inbox,
    Events,
    CreateEvent,
    Explore,
    Privacy,
    ConfirmKick { guild: Id, user: Id },
    ConfirmBan { guild: Id, user: Id },
    TimeoutMember { guild: Id, user: Id },
    Bans,
    PruneMembers,
    VideoQuality,
    CropGuildImage { banner: bool },
    CropUserImage { banner: bool },
    CropDmIcon { channel: Id },
    DmSettings { channel: Id },
    GroupDm,
    VoiceDevice { kind: String },
    Soundboard,
    ProfileMore(Id),
    BanAppeal,
    InviteJoin,
    ServerSettings,
    Roles,
    ChannelPermissions(Id),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    Account,
    Profile,
    Application,
    Appearance,
    Notifications,
    Voice,
    Keybinds,
    Bots,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeoutUnit {
    Minutes,
    Hours,
    Days,
}

impl TimeoutUnit {
    pub fn label(self) -> &'static str {
        match self {
            TimeoutUnit::Minutes => "Minutes",
            TimeoutUnit::Hours => "Hours",
            TimeoutUnit::Days => "Days",
        }
    }

    pub fn seconds(self) -> u64 {
        match self {
            TimeoutUnit::Minutes => 60,
            TimeoutUnit::Hours => 60 * 60,
            TimeoutUnit::Days => 24 * 60 * 60,
        }
    }

    pub fn max_value(self) -> u64 {
        (28 * 24 * 60 * 60) / self.seconds()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerTab {
    Overview,
    Roles,
    Members,
    Bans,
    Emojis,
    Audit,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub text: String,
    pub until: f64,
}

#[derive(Clone, Debug, Default)]
pub struct MemberState {
    pub total: u32,
    pub next_offset: Option<u32>,
    pub loading: bool,
    pub complete: bool,
    pub query: String,
    pub channel: Id,
}

pub struct App {
    pub dark: bool,
    pub screen: Screen,
    pub dm_home: bool,
    pub friend_tab: u8,
    pub friend_filter: TextEdit,
    pub group_dm_selected: std::collections::BTreeSet<Id>,
    pub group_dm_name: TextEdit,
    pub dm_settings_name: TextEdit,
    pub add_friend_open: bool,
    pub me: User,
    pub guilds: Vec<GuildSummary>,
    pub guild_details: HashMap<Id, GuildDetail>,
    pub channels: HashMap<Id, Vec<Channel>>,
    pub messages: HashMap<Id, Vec<Message>>,
    pub members: HashMap<Id, Vec<Member>>,
    pub member_state: HashMap<Id, MemberState>,
    pub member_groups_collapsed: HashSet<String>,
    pub users: HashMap<Id, User>,
    pub dms: Vec<DmChannel>,
    pub relationships: Vec<Relationship>,
    pub unread: HashMap<Id, u32>,
    pub reads: HashMap<Id, Option<Id>>,
    pub presence: HashMap<Id, Presence>,
    pub voice: HashMap<Id, VoiceState>,
    pub active_guild: Id,
    pub active_channel: Id,
    pub active_dm: Id,
    pub last_guild_channel: HashMap<Id, Id>,
    pub collapsed: HashSet<Id>,
    pub show_members: bool,
    pub composer: TextEdit,
    pub search: TextEdit,
    pub member_search: TextEdit,
    pub modal: Modal,
    pub settings_tab: SettingsTab,
    pub server_base: String,
    pub server_url: TextEdit,
    pub server_url_saved: String,
    pub server_config_error: Option<String>,
    pub server_url_error: Option<String>,
    pub server_checking: bool,
    pub pending_server_url: Option<String>,
    pub server_unreachable: bool,
    pub toast: Option<Toast>,
    pub attachments: Vec<Attachment>,
    pub clock: f64,
    pub net: Option<Net>,
    pub logged_in: bool,
    pub connected: bool,
    pub connecting: bool,
    pub auth_error: Option<String>,
    pub auth_register: bool,
    pub auth_busy: bool,
    pub auth_user: TextEdit,
    pub auth_pass: TextEdit,
    pub auth_display: TextEdit,
    pub typing: HashMap<Id, Vec<(Id, i64)>>,
    pub images: HashMap<String, Pixels>,
    pub animations: HashMap<String, AnimationPlayer>,
    pub image_requests: HashSet<String>,
    pub user_requests: HashSet<Id>,
    pub visible_messages: HashMap<Id, (usize, usize)>,
    pub loading_older: HashSet<Id>,
    pub older_complete: HashSet<Id>,
    pub image_sizes: HashMap<String, usize>,
    pub image_order: VecDeque<String>,
    pub image_bytes: usize,
    pub search_results: Option<SearchResults>,
    pub last_typing: f64,
    pub invite_code: Option<String>,
    pub profile_edit: TextEdit,
    pub edit_text: TextEdit,
    pub reply_to: Option<MessageReply>,
    pub cache_profile: String,
    pub cache_dirty_at: Option<f64>,
    pub last_cache_save: f64,
    pub last_maintenance: f64,
    pub bootstrap_pending: bool,
    pub bootstrap_changed_at: f64,
    pub last_bootstrap_request: f64,
    pub channel_seen: HashMap<Id, i64>,
    pub guild_seen: HashMap<Id, i64>,
    pub member_seen: HashMap<Id, i64>,
    pub stale_guilds: HashSet<Id>,
    pub zoom: rune_kit::image_zoom::ImageZoom,
    pub viewing_image: Option<(String, f32, f32)>,
    pub drop_hover: bool,
    pub notify_dm: bool,
    pub notify_mentions: bool,
    pub member_search_live: String,
    pub member_search_due: f64,
    pub last_member_query: String,
    pub profile_display: TextEdit,
    pub profile_pronouns: TextEdit,
    pub profile_badges: std::collections::BTreeSet<String>,
    pub profile_decoration: String,
    pub voice_input_device: String,
    pub voice_output_device: String,
    pub voice_camera_device: String,
    pub voice_input_volume: f32,
    pub voice_output_volume: f32,
    pub voice_sensitivity: f32,
    pub voice_auto_sensitivity: bool,
    pub voice_mode: String,
    pub voice_chat: bool,
    pub audio_devices: Vec<String>,
    pub output_devices: Vec<String>,
    pub video_devices: Vec<String>,
    pub last_camera_test_version: u64,
    pub profile_bio: TextEdit,
    pub profile_status: TextEdit,
    pub password_current: TextEdit,
    pub password_new: TextEdit,
    pub compact: bool,
    pub developer: bool,
    pub keybinds: BTreeMap<String, String>,
    pub recording_key: Option<String>,
    pub bots: Vec<User>,
    pub applications: Vec<Application>,
    pub token_reveal: Option<(String, String)>,
    pub bot_name: TextEdit,
    pub app_name: TextEdit,
    pub app_description: TextEdit,
    pub install_code: TextEdit,
    pub add_bot_to_guild: bool,
    pub pending_install_app: Option<Id>,
    pub server_tab: ServerTab,
    pub selected_role: Id,
    pub role_name: TextEdit,
    pub role_color: u32,
    pub role_permissions: u64,
    pub role_hoist: bool,
    pub role_mentionable: bool,
    pub role_tab: u8,
    pub role_links: HashMap<Id, Vec<RoleLink>>,
    pub role_link_label: TextEdit,
    pub role_link_url: TextEdit,
    pub role_drag: Option<(Id, i32)>,
    pub inbox: Option<Inbox>,
    pub inbox_tab: u8,
    pub guild_events: Vec<GuildEvent>,
    pub event_name: TextEdit,
    pub event_description: TextEdit,
    pub event_start_minutes: TextEdit,
    pub profile_activity_kind: TextEdit,
    pub profile_activity_title: TextEdit,
    pub profile_activity_subtitle: TextEdit,
    pub profile_activity_state: TextEdit,
    pub profile_activity_url: TextEdit,
    pub profile_connection_kind: TextEdit,
    pub profile_connection_name: TextEdit,
    pub selected_member: Option<Id>,
    pub overview_name: TextEdit,
    pub overview_description: TextEdit,
    pub overview_tag: TextEdit,
    pub overview_color: u32,
    pub overview_discoverable: bool,
    pub emojis: Vec<Emoji>,
    pub emoji_name: TextEdit,
    pub audit: Vec<AuditEntry>,
    pub audit_before: Option<Id>,
    pub audit_loading: bool,
    pub audit_complete: bool,
    pub commands: Vec<ApplicationCommand>,
    pub thread_parent: Option<Id>,
    pub pending_forum: Option<Id>,
    pub editing_channel: Id,
    pub channel_name_edit: TextEdit,
    pub channel_topic_edit: TextEdit,
    pub channel_status_edit: TextEdit,
    pub channel_slowmode_edit: TextEdit,
    pub channel_nsfw_edit: bool,
    pub create_parent: Option<Id>,
    pub suppress_errors_until: f64,
    pub emoji_search: TextEdit,
    pub thread_body: TextEdit,
    pub pending_thread_body: Option<String>,
    pub emoji_category: usize,
    pub channel_notify: BTreeMap<Id, NotificationLevel>,
    pub guild_notify: BTreeMap<Id, GuildNotificationSettings>,
    pub guild_notify_local_at: BTreeMap<Id, f64>,
    pub channel_suppress: BTreeSet<Id>,
    pub public_guilds: Vec<PublicGuild>,
    pub explore_loading: bool,
    pub pinned: Vec<Message>,
    pub pinned_target: Option<Id>,
    pub voice_mute: bool,
    pub voice_deaf: bool,
    pub voice_camera: bool,
    pub voice_screen: bool,
    pub voice_video: HashMap<Id, rune::Pixels>,
    pub voice_video_info: HashMap<Id, (u32, u32, u32)>,
    pub video_quality: crate::voice::VideoQuality,
    pub viewer_quality: crate::voice::ViewQuality,
    pub timeout_amount: TextEdit,
    pub timeout_unit: TimeoutUnit,
    pub join_code: TextEdit,
    pub last_join_code: String,
    pub appeal_code: String,
    pub appeal_message: TextEdit,
    pub appeals: Vec<BanAppeal>,
    pub ban_reason: TextEdit,
    pub bans: Vec<Ban>,
    pub prune_days: u32,
    pub prune_roles: BTreeSet<Id>,
    pub prune_preview: Option<u32>,
    pub prune_loading: bool,
    pub crop_source: Option<CropSource>,
    pub crop_banner: bool,
    pub crop_zoom: f32,
    pub crop_offset: (f32, f32),
    pub crop_drag_origin: (f32, f32),
    pub crop_preview: Option<rune::Pixels>,
    pub crop_user: bool,
    pub crop_dm: Id,
    pub mutuals: HashMap<Id, Vec<GuildSummary>>,
    pub mutual_requests: HashSet<Id>,
}

pub struct CropSource {
    pub image: image::RgbaImage,
}

pub struct AnimationPlayer {
    pub frames: Vec<crate::image_anim::AnimFrame>,
    pub index: usize,
    pub next_at: f64,
    pub width: u32,
    pub height: u32,
    pub pixels: rune::Pixels,
}

impl App {
    pub fn new() -> Self {
        let headless = rune::is_headless() && std::env::var_os("RUNECORD_LIVE").is_none();
        let startup = crate::config::startup_effective();
        let server_base = startup.url;
        let server_config_error = startup.error;
        let server_url_saved = crate::config::stored_server_url();
        if !headless && crate::config::uses_bundled_server(&server_base) {
            ensure_server();
        }
        let server_unreachable = !headless
            && server_config_error.is_none()
            && !crate::config::server_reachable(&server_base);
        let profile_name = profile().unwrap_or_else(|| "1".to_string());
        // Only paint a snapshot when a session token exists. A logged-out
        // profile must never look logged in just because an old cache exists.
        let snapshot = if headless || !net::has_token(&profile_name) {
            None
        } else {
            crate::cache::load(&profile_name)
        };
        let net = if headless {
            None
        } else {
            Some(net::start(Some(profile_name.clone()), snapshot.is_some()))
        };
        let mut app = App {
            dark: if headless { true } else { load_theme() },
            screen: Screen::Guild,
            dm_home: true,
            friend_tab: 0,
            friend_filter: TextEdit::new(""),
            group_dm_selected: std::collections::BTreeSet::new(),
            group_dm_name: TextEdit::new(""),
            dm_settings_name: TextEdit::new(""),
            add_friend_open: false,
            me: User::default(),
            guilds: Vec::new(),
            guild_details: HashMap::new(),
            channels: HashMap::new(),
            messages: HashMap::new(),
            members: HashMap::new(),
            member_state: HashMap::new(),
            member_groups_collapsed: HashSet::new(),
            users: HashMap::new(),
            dms: Vec::new(),
            relationships: Vec::new(),
            unread: HashMap::new(),
            reads: HashMap::new(),
            presence: HashMap::new(),
            voice: HashMap::new(),
            active_guild: 0,
            active_channel: 0,
            active_dm: 0,
            last_guild_channel: HashMap::new(),
            collapsed: HashSet::new(),
            show_members: true,
            composer: TextEdit::new(""),
            search: TextEdit::new(""),
            member_search: TextEdit::new(""),
            modal: Modal::None,
            settings_tab: SettingsTab::Account,
            server_base,
            server_url: TextEdit::new(server_url_saved.clone()),
            server_url_saved,
            server_config_error,
            server_url_error: None,
            server_checking: false,
            pending_server_url: None,
            server_unreachable,
            toast: None,
            attachments: Vec::new(),
            clock: 0.0,
            net,
            logged_in: headless,
            connected: false,
            connecting: !headless,
            auth_error: None,
            auth_register: false,
            auth_busy: false,
            auth_user: TextEdit::new(""),
            auth_pass: TextEdit::new(""),
            auth_display: TextEdit::new(""),
            typing: HashMap::new(),
            images: HashMap::new(),
            animations: HashMap::new(),
            image_requests: HashSet::new(),
            user_requests: HashSet::new(),
            visible_messages: HashMap::new(),
            loading_older: HashSet::new(),
            older_complete: HashSet::new(),
            image_sizes: HashMap::new(),
            image_order: VecDeque::new(),
            image_bytes: 0,
            search_results: None,
            last_typing: -100.0,
            invite_code: None,
            profile_edit: TextEdit::new(""),
            edit_text: TextEdit::new(""),
            reply_to: None,
            cache_profile: profile_name,
            cache_dirty_at: None,
            last_cache_save: -100.0,
            last_maintenance: 0.0,
            bootstrap_pending: false,
            bootstrap_changed_at: 0.0,
            last_bootstrap_request: -10.0,
            channel_seen: HashMap::new(),
            guild_seen: HashMap::new(),
            member_seen: HashMap::new(),
            stale_guilds: HashSet::new(),
            zoom: rune_kit::image_zoom::ImageZoom::new(),
            viewing_image: None,
            drop_hover: false,
            notify_dm: true,
            notify_mentions: true,
            member_search_live: String::new(),
            member_search_due: 0.0,
            last_member_query: String::new(),
            profile_display: TextEdit::new(""),
            profile_pronouns: TextEdit::new(""),
            profile_badges: std::collections::BTreeSet::new(),
            profile_decoration: String::new(),
            voice_input_device: String::new(),
            voice_output_device: String::new(),
            voice_camera_device: String::new(),
            voice_input_volume: 1.0,
            voice_output_volume: 1.0,
            voice_sensitivity: 0.02,
            voice_auto_sensitivity: true,
            voice_mode: "voice".to_string(),
            voice_chat: false,
            audio_devices: Vec::new(),
            output_devices: Vec::new(),
            video_devices: Vec::new(),
            last_camera_test_version: 0,
            profile_bio: TextEdit::new(""),
            profile_status: TextEdit::new(""),
            password_current: TextEdit::new(""),
            password_new: TextEdit::new(""),
            compact: false,
            developer: false,
            keybinds: default_keybinds(),
            recording_key: None,
            bots: Vec::new(),
            applications: Vec::new(),
            token_reveal: None,
            bot_name: TextEdit::new(""),
            app_name: TextEdit::new(""),
            app_description: TextEdit::new(""),
            install_code: TextEdit::new(""),
            add_bot_to_guild: true,
            pending_install_app: None,
            server_tab: ServerTab::Roles,
            selected_role: 0,
            role_name: TextEdit::new(""),
            role_color: 0,
            role_permissions: 0,
            role_hoist: false,
            role_mentionable: false,
            role_tab: 0,
            role_links: HashMap::new(),
            role_link_label: TextEdit::new(""),
            role_link_url: TextEdit::new(""),
            role_drag: None,
            inbox: None,
            inbox_tab: 1,
            guild_events: Vec::new(),
            event_name: TextEdit::new(""),
            event_description: TextEdit::new(""),
            event_start_minutes: TextEdit::new("60"),
            profile_activity_kind: TextEdit::new(""),
            profile_activity_title: TextEdit::new(""),
            profile_activity_subtitle: TextEdit::new(""),
            profile_activity_state: TextEdit::new(""),
            profile_activity_url: TextEdit::new(""),
            profile_connection_kind: TextEdit::new(""),
            profile_connection_name: TextEdit::new(""),
            selected_member: None,
            overview_name: TextEdit::new(""),
            overview_description: TextEdit::new(""),
            overview_tag: TextEdit::new(""),
            overview_color: 0x5865f2,
            overview_discoverable: true,
            emojis: Vec::new(),
            emoji_name: TextEdit::new(""),
            audit: Vec::new(),
            audit_before: None,
            audit_loading: false,
            audit_complete: false,
            commands: Vec::new(),
            thread_parent: None,
            pending_forum: None,
            editing_channel: 0,
            channel_name_edit: TextEdit::new(""),
            channel_topic_edit: TextEdit::new(""),
            channel_status_edit: TextEdit::new(""),
            channel_slowmode_edit: TextEdit::new("0"),
            channel_nsfw_edit: false,
            create_parent: None,
            suppress_errors_until: 0.0,
            emoji_search: TextEdit::new(""),
            thread_body: TextEdit::new(""),
            pending_thread_body: None,
            emoji_category: 0,
            channel_notify: BTreeMap::new(),
            guild_notify: BTreeMap::new(),
            guild_notify_local_at: BTreeMap::new(),
            channel_suppress: BTreeSet::new(),
            public_guilds: Vec::new(),
            explore_loading: false,
            pinned: Vec::new(),
            pinned_target: None,
            voice_mute: false,
            voice_deaf: false,
            voice_camera: false,
            voice_screen: false,
            voice_video: HashMap::new(),
            voice_video_info: HashMap::new(),
            video_quality: crate::voice::VideoQuality::Auto,
            viewer_quality: crate::voice::ViewQuality::Auto,
            timeout_amount: TextEdit::new("10"),
            timeout_unit: TimeoutUnit::Minutes,
            join_code: TextEdit::new(""),
            last_join_code: String::new(),
            appeal_code: String::new(),
            appeal_message: TextEdit::new(""),
            appeals: Vec::new(),
            ban_reason: TextEdit::new(""),
            bans: Vec::new(),
            prune_days: 7,
            prune_roles: BTreeSet::new(),
            prune_preview: None,
            prune_loading: false,
            crop_source: None,
            crop_banner: false,
            crop_zoom: 1.0,
            crop_offset: (0.0, 0.0),
            crop_drag_origin: (0.0, 0.0),
            crop_preview: None,
            crop_user: false,
            crop_dm: 0,
            mutuals: HashMap::new(),
            mutual_requests: HashSet::new(),
        };
        if headless {
            app.seed();
        } else if let Some(snapshot) = snapshot {
            app.hydrate(snapshot);
        }
        if let Ok(mode) = std::env::var("RUNECORD_VOICE_MODE") {
            app.voice_mode = mode;
        }
        if let Ok(device) = std::env::var("RUNECORD_OUTPUT_DEVICE") {
            app.voice_output_device = device;
        }
        if let Ok(quality) = std::env::var("RUNECORD_VIDEO_QUALITY") {
            app.video_quality = crate::voice::VideoQuality::parse(&quality);
            crate::voice::set_quality(app.video_quality);
        }
        if let Ok(quality) = std::env::var("RUNECORD_VIEW_QUALITY") {
            app.viewer_quality = crate::voice::ViewQuality::parse(&quality);
            crate::voice::set_viewer_quality(app.viewer_quality);
        }
        app.apply_voice_settings();
        app
    }

    pub fn demo() -> Self {
        let mut app = App::new();
        app.logged_in = true;
        app
    }

    // -----------------------------------------------------------------------
    // Lookups
    // -----------------------------------------------------------------------

    pub fn active_guild(&self) -> Option<&GuildSummary> {
        self.guilds
            .iter()
            .find(|g| g.id == self.active_guild)
            .or_else(|| self.guilds.first())
    }

    pub fn guild_detail(&self, guild: Id) -> Option<&GuildDetail> {
        self.guild_details.get(&guild)
    }

    pub fn guild_roles(&self, guild: Id) -> &[Role] {
        self.guild_details
            .get(&guild)
            .map(|d| d.roles.as_slice())
            .unwrap_or(&[])
    }

    pub fn guild_channels(&self, guild: Id) -> &[Channel] {
        self.channels.get(&guild).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn channel(&self, id: Id) -> Option<&Channel> {
        self.channels
            .values()
            .flatten()
            .chain(self.dms.iter().map(|d| &d.channel))
            .find(|c| c.id == id)
    }

    pub fn current_channel(&self) -> Option<&Channel> {
        match self.screen {
            Screen::Guild => self.channel(self.active_channel),
            Screen::Dms => self.channel(self.active_dm),
        }
    }

    pub fn user(&self, id: Id) -> Option<&User> {
        self.users.get(&id)
    }

    pub fn user_name(&self, id: Id) -> String {
        self.users
            .get(&id)
            .map(|u| u.display_name.clone())
            .unwrap_or_else(|| "Unknown".into())
    }

    pub fn member(&self, guild: Id, user: Id) -> Option<&Member> {
        self.members.get(&guild)?.iter().find(|m| m.user.id == user)
    }

    pub fn member_color(&self, guild: Id, user: Id) -> Option<u32> {
        let member = self.member(guild, user)?;
        let roles = self.guild_roles(guild);
        let mut best: Option<&Role> = None;
        for rid in &member.roles {
            if let Some(role) = roles.iter().find(|r| r.id == *rid) {
                if role.color != 0 && best.is_none_or(|b| role.position > b.position) {
                    best = Some(role);
                }
            }
        }
        best.map(|r| r.color)
    }

    pub fn dm_for(&self, channel: Id) -> Option<&DmChannel> {
        self.dms.iter().find(|d| d.channel.id == channel)
    }

    pub fn presence_of(&self, id: Id) -> Presence {
        self.presence
            .get(&id)
            .copied()
            .or_else(|| self.users.get(&id).map(|u| u.status))
            .unwrap_or(Presence::Offline)
    }

    pub fn guild_can(&self, bit: u64) -> bool {
        has(self.active_guild().map(|g| g.permissions).unwrap_or(0), bit)
    }

    /// The highest role position a member holds, matching the server's own
    /// hierarchy check. The owner outranks everyone.
    pub fn member_rank(&self, guild: Id, user: Id) -> i32 {
        if self
            .guilds
            .iter()
            .find(|item| item.id == guild)
            .is_some_and(|summary| summary.owner_id == user)
        {
            return i32::MAX;
        }
        let Some(member) = self.member(guild, user) else {
            return 0;
        };
        self.guild_roles(guild)
            .iter()
            .filter(|role| member.roles.contains(&role.id))
            .map(|role| role.position)
            .max()
            .unwrap_or(0)
    }

    pub fn channel_can(&self, channel: Id, bit: u64) -> bool {
        self.channel(channel)
            .map(|c| has(c.my_permissions, bit))
            .unwrap_or(false)
    }

    // -----------------------------------------------------------------------
    // Actions
    // -----------------------------------------------------------------------

    pub fn send(&self, cmd: Command) {
        if let Some(net) = &self.net {
            net.send(cmd);
        }
    }

    pub fn keybind_action(&self, ev: &KeyEvent) -> Option<&'static str> {
        KEYBIND_ACTIONS.iter().find_map(|(action, _)| {
            let spec = self.keybinds.get(*action)?;
            let chord = spec.parse::<rune::Chord>().ok()?;
            chord.matches(ev).then_some(*action)
        })
    }
}

pub const KEYBIND_ACTIONS: &[(&str, &str)] = &[
    ("quick_switcher", "Quick switcher"),
    ("search", "Search"),
    ("settings", "Open settings"),
    ("mute", "Toggle mute"),
    ("deafen", "Toggle deafen"),
];

pub fn default_keybinds() -> BTreeMap<String, String> {
    [
        ("quick_switcher", "mod+k"),
        ("search", "mod+f"),
        ("settings", "mod+,"),
        ("mute", "mod+shift+m"),
        ("deafen", "mod+shift+d"),
    ]
    .into_iter()
    .map(|(action, chord)| (action.to_string(), chord.to_string()))
    .collect()
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

// ---------------------------------------------------------------------------
// Image helpers
// ---------------------------------------------------------------------------

fn collect_image_urls(message: &Message, out: &mut Vec<String>) {
    for attachment in &message.attachments {
        if let Some(url) = attachment_image_url(attachment) {
            out.push(url);
        }
    }
}

pub fn attachment_image_url(attachment: &Attachment) -> Option<String> {
    if attachment.content_type.starts_with("image/") && !attachment.url.is_empty() {
        Some(format!("{}?w=480", attachment.url))
    } else {
        None
    }
}

fn bench_messages(count: usize, now: i64) -> Vec<Message> {
    (0..count)
        .map(|i| {
            let author = (i % 8 + 1) as Id;
            let body = if i % 17 == 0 {
                format!("bench row {i}\n```rust\nlet value = {i};\n```")
            } else {
                format!(
                    "benchmark message {i} — plain text row for layout and shaping measurements"
                )
            };
            Message {
                id: 100_000 + i as Id,
                channel_id: 211,
                author_id: author,
                content: body,
                timestamp: now - ((count - i) as i64 * 1_000),
                ..Message::default()
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Process environment
// ---------------------------------------------------------------------------

fn prefs_path() -> Option<std::path::PathBuf> {
    let base = std::env::var("APPDATA")
        .ok()
        .map(std::path::PathBuf::from)?;
    Some(base.join("Runecord").join("prefs.txt"))
}

fn load_theme() -> bool {
    prefs_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| !s.trim().eq_ignore_ascii_case("light"))
        .unwrap_or(true)
}

fn save_theme(dark: bool) {
    if let Some(path) = prefs_path() {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, if dark { "dark" } else { "light" });
    }
}

fn profile() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--profile" {
            return args.next();
        }
        if let Some(v) = arg.strip_prefix("--profile=") {
            return Some(v.to_string());
        }
    }
    std::env::var("RUNECORD_PROFILE").ok()
}

fn ensure_server() {
    if std::net::TcpStream::connect_timeout(
        &"127.0.0.1:47800".parse().unwrap(),
        std::time::Duration::from_millis(250),
    )
    .is_ok()
    {
        return;
    }
    let Some(exe) = std::env::current_exe().ok() else {
        return;
    };
    let dir = exe.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let front_root = project_root(&dir).unwrap_or_else(|| dir.clone());
    let backend = front_root.parent().map(|p| p.join("Runecord-backend"));
    let mut candidates = vec![dir.join("runecord-server.exe")];
    if let Some(backend) = &backend {
        let debug = backend
            .join("target")
            .join("debug")
            .join("runecord-server.exe");
        let release = backend
            .join("target")
            .join("release")
            .join("runecord-server.exe");
        if cfg!(debug_assertions) {
            candidates.push(debug);
            candidates.push(release);
        } else {
            candidates.push(release);
            candidates.push(debug);
        }
    }
    let Some(server) = candidates.into_iter().find(|p| p.exists()) else {
        return;
    };
    let (cwd, data) = backend
        .filter(|b| b.join("crates").join("server").is_dir())
        .map(|b| (b.clone(), b.join("data")))
        .unwrap_or_else(|| (dir.clone(), dir.join("data")));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new(server)
            .arg("--data")
            .arg(&data)
            .current_dir(&cwd)
            .creation_flags(0x08000000)
            .spawn();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new(server)
            .arg("--data")
            .arg(&data)
            .current_dir(&cwd)
            .spawn();
    }
    std::thread::sleep(std::time::Duration::from_millis(350));
}

fn project_root(mut dir: &std::path::Path) -> Option<std::path::PathBuf> {
    for _ in 0..6 {
        if dir.join("crates").join("client").is_dir() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
    None
}

// ---------------------------------------------------------------------------
// Keys
// ---------------------------------------------------------------------------

pub fn on_key(s: &mut App, ev: &KeyEvent, tx: &mut Tx) -> bool {
    if let Some(action) = s.recording_key.clone() {
        if matches!(ev.key, Key::Escape) && !ev.mods.cmd && !ev.mods.ctrl && !ev.mods.alt {
            s.recording_key = None;
            return true;
        }
        if let Some(chord) = event_chord(ev) {
            s.set_keybind(&action, &chord);
            s.recording_key = None;
            s.toast(format!(
                "{} is now {}",
                action_label(&action),
                display_chord(&chord)
            ));
        }
        return true;
    }
    if matches!(ev.key, Key::Escape) {
        if s.viewing_image.is_some() {
            if s.zoom.owns_escape() {
                s.zoom.fit(tx);
            } else {
                s.viewing_image = None;
            }
            return true;
        }
        if s.modal != Modal::None {
            s.modal = Modal::None;
            return true;
        }
        return false;
    }
    if let Some(action) = s.keybind_action(ev) {
        match action {
            "quick_switcher" => {
                s.search.clear();
                s.modal = Modal::QuickSwitcher;
            }
            "search" => {
                s.search.clear();
                s.modal = Modal::Search;
            }
            "settings" => s.open_settings(SettingsTab::Account),
            "mute" => s.toast("Mute toggled (no audio in this build)"),
            "deafen" => s.toast("Deafen toggled (no audio in this build)"),
            _ => return false,
        }
        return true;
    }
    false
}

pub fn action_label(action: &str) -> &'static str {
    KEYBIND_ACTIONS
        .iter()
        .find(|(id, _)| *id == action)
        .map(|(_, label)| *label)
        .unwrap_or("Shortcut")
}

pub fn display_chord(spec: &str) -> String {
    if spec.is_empty() {
        return "Unbound".to_string();
    }
    spec.split('+')
        .map(|part| match part {
            "mod" => {
                if cfg!(target_os = "macos") {
                    "Cmd".to_string()
                } else {
                    "Ctrl".to_string()
                }
            }
            "ctrl" => "Ctrl".to_string(),
            "shift" => "Shift".to_string(),
            "alt" => "Alt".to_string(),
            "space" => "Space".to_string(),
            "enter" => "Enter".to_string(),
            "escape" => "Esc".to_string(),
            other => other.to_uppercase(),
        })
        .collect::<Vec<_>>()
        .join("+")
}

fn event_chord(ev: &KeyEvent) -> Option<String> {
    let mut parts = Vec::new();
    if ev.mods.cmd {
        parts.push("mod");
    } else if ev.mods.ctrl {
        parts.push("ctrl");
    }
    if ev.mods.alt {
        parts.push("alt");
    }
    if ev.mods.shift {
        parts.push("shift");
    }
    let token = match &ev.key {
        Key::Command(c) => c.to_ascii_lowercase().to_string(),
        Key::Text(text) => {
            let mut chars = text.chars();
            match (chars.next(), chars.next()) {
                (Some(' '), None) => "space".to_string(),
                (Some(c), None) => c.to_ascii_lowercase().to_string(),
                _ => return None,
            }
        }
        Key::Function(n) => format!("f{n}"),
        Key::Enter => "enter".to_string(),
        Key::Backspace => "backspace".to_string(),
        Key::Delete => "delete".to_string(),
        Key::Left => "left".to_string(),
        Key::Right => "right".to_string(),
        Key::Up => "up".to_string(),
        Key::Down => "down".to_string(),
        Key::Home => "home".to_string(),
        Key::End => "end".to_string(),
        Key::PageUp => "pageup".to_string(),
        Key::PageDown => "pagedown".to_string(),
        Key::Insert => "insert".to_string(),
        Key::Tab => "tab".to_string(),
        Key::Escape => "escape".to_string(),
        _ => return None,
    };
    if parts.is_empty() && matches!(ev.key, Key::Text(_) | Key::Command(_)) {
        return None;
    }
    parts.push(token.as_str());
    Some(parts.join("+"))
}

mod actions;
mod events;
mod media;
mod retention;
mod seed;
mod session;
