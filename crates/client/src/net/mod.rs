//! Transport: blocking HTTP commands on a worker thread plus a reconnecting gateway socket.
//! Nothing here touches the UI thread.

use runecord_protocol::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Command {
    Login {
        username: String,
        password: String,
    },
    Register {
        username: String,
        display_name: String,
        password: String,
    },
    Logout,
    Bootstrap,
    LoadGuildDetail {
        guild: Id,
    },
    LoadMembers {
        guild: Id,
        channel: Id,
        offset: u32,
        query: String,
        replace: bool,
    },
    LoadUsers {
        users: Vec<Id>,
    },
    LookupUser {
        query: String,
    },
    LoadMessages {
        channel: Id,
        before: Option<Id>,
    },
    Send {
        channel: Id,
        content: String,
        reply_to: Option<Id>,
        attachments: Vec<Id>,
        #[allow(dead_code)]
        nonce: String,
    },
    Edit {
        message: Id,
        content: String,
    },
    Delete {
        message: Id,
    },
    React {
        message: Id,
        emoji: String,
    },
    Pin {
        message: Id,
    },
    Typing {
        channel: Id,
    },
    MarkRead {
        channel: Id,
        message: Id,
    },
    Search {
        q: String,
    },
    LoadInbox,
    UpdateProfileStyle {
        effect: String,
        nameplate: String,
        widget: String,
    },
    LoadGuildEvents {
        guild: Id,
    },
    CreateGuildEvent {
        guild: Id,
        name: String,
        description: String,
        start_at: i64,
        channel: Option<Id>,
    },
    DeleteGuildEvent {
        guild: Id,
        event: Id,
    },
    CheckServer {
        url: String,
    },
    UpdateProfileExtras {
        activity: Option<ProfileActivity>,
        connections: Vec<ProfileConnection>,
    },
    UploadActivityArtwork {
        path: std::path::PathBuf,
    },
    RemoveActivityArtwork,
    CreateGuild {
        name: String,
        color: u32,
    },
    LeaveGuild {
        guild: Id,
    },
    SetChannelNotify {
        channel: Id,
        level: NotificationLevel,
        suppress: bool,
        clear: bool,
    },
    SetGuildNotify {
        guild: Id,
        settings: GuildNotificationSettings,
    },
    LoadPublicGuilds,
    JoinPublicGuild {
        guild: Id,
    },
    LoadPins {
        channel: Id,
    },
    SetPrivacy {
        dm: PrivacyLevel,
        friend: PrivacyLevel,
    },
    SetGuildDiscovery {
        guild: Id,
        discoverable: bool,
    },
    BoostGuild {
        guild: Id,
        on: bool,
    },
    UploadGuildImage {
        guild: Id,
        banner: bool,
        path: std::path::PathBuf,
    },
    RemoveGuildImage {
        guild: Id,
        banner: bool,
    },
    LoadMutuals {
        user: Id,
    },
    CreateGroupDm {
        users: Vec<Id>,
        name: String,
    },
    RenameDm {
        channel: Id,
        name: String,
    },
    UploadDmIcon {
        channel: Id,
        path: std::path::PathBuf,
    },
    RemoveDmIcon {
        channel: Id,
    },
    AddDmMember {
        channel: Id,
        user: Id,
    },
    RemoveDmMember {
        channel: Id,
        user: Id,
    },
    LeaveDm {
        channel: Id,
    },
    UploadUserImage {
        banner: bool,
        path: std::path::PathBuf,
    },
    RemoveUserImage {
        banner: bool,
    },
    Video {
        channel: Option<Id>,
        kind: String,
    },
    CreateChannel {
        guild: Id,
        name: String,
        kind: ChannelKind,
        parent: Option<Id>,
        topic: String,
    },
    UpdateChannel {
        channel: Id,
        name: String,
        topic: String,
        status: String,
        slowmode: u32,
        nsfw: bool,
    },
    KickMember {
        guild: Id,
        user: Id,
    },
    BanMember {
        guild: Id,
        user: Id,
        reason: String,
    },
    TimeoutMember {
        guild: Id,
        user: Id,
        seconds: u64,
    },
    LoadBans {
        guild: Id,
    },
    UnbanMember {
        guild: Id,
        user: Id,
    },
    UpdateGuildSettings {
        guild: Id,
        name: String,
        description: String,
        color: u32,
        tag: String,
        discoverable: bool,
    },
    PrunePreview {
        guild: Id,
        days: u32,
        roles: Vec<Id>,
    },
    PruneMembers {
        guild: Id,
        days: u32,
        roles: Vec<Id>,
    },
    LoadAppeals {
        guild: Id,
    },
    ResolveAppeal {
        guild: Id,
        appeal: Id,
        accept: bool,
    },
    SubmitAppeal {
        code: String,
        message: String,
    },
    DeleteChannel {
        channel: Id,
    },
    CreateInvite {
        guild: Id,
    },
    JoinInvite {
        code: String,
    },
    OpenDm {
        user: Id,
    },
    FriendRequest {
        user: Id,
    },
    FriendAccept {
        user: Id,
    },
    FriendRemove {
        user: Id,
    },
    FriendBlock {
        user: Id,
    },
    CreateThread {
        channel: Id,
        message: Option<Id>,
        name: String,
    },
    ArchiveThread {
        channel: Id,
        archived: bool,
    },
    UpdateProfile {
        display_name: Option<String>,
        pronouns: Option<String>,
        badges: Option<Vec<String>>,
        avatar_decoration: Option<String>,
        custom_status: Option<String>,
        bio: Option<String>,
        color: Option<u32>,
        avatar_border: Option<u32>,
        banner_color: Option<u32>,
    },
    LoadBots,
    CreateBot {
        name: String,
        guild: Option<Id>,
    },
    DeleteBot {
        id: Id,
    },
    LoadApplications,
    CreateApplication {
        name: String,
        description: String,
    },
    DeleteApplication {
        id: Id,
    },
    ApplicationToken {
        id: Id,
        name: String,
    },
    ApplicationInstallLink {
        id: Id,
    },
    InstallApplication {
        code: String,
        guild: Id,
    },
    ChangePassword {
        current: String,
        new: String,
    },
    FetchImage {
        url: String,
    },
    Upload {
        channel: Id,
        path: std::path::PathBuf,
    },
    Voice {
        guild: Id,
        channel: Option<Id>,
        mute: bool,
        deaf: bool,
    },
    UpdateRole {
        guild: Id,
        role: Id,
        name: Option<String>,
        color: Option<u32>,
        permissions: Option<u64>,
        hoist: Option<bool>,
        mentionable: Option<bool>,
        position: Option<i32>,
    },
    CreateRole {
        guild: Id,
        name: String,
        color: u32,
        permissions: u64,
    },
    DeleteRole {
        guild: Id,
        role: Id,
    },
    UpdateMemberRoles {
        guild: Id,
        user: Id,
        roles: Vec<Id>,
    },
    UploadRoleIcon {
        guild: Id,
        role: Id,
        path: std::path::PathBuf,
    },
    RemoveRoleIcon {
        guild: Id,
        role: Id,
    },
    LoadRoleLinks {
        guild: Id,
        role: Id,
    },
    AddRoleLink {
        guild: Id,
        role: Id,
        label: String,
        url: String,
    },
    DeleteRoleLink {
        guild: Id,
        role: Id,
        link: Id,
    },
    LoadEmojis {
        guild: Id,
    },
    LoadAudit {
        guild: Id,
        before: Option<Id>,
    },
    LoadGuildCommands {
        guild: Id,
    },
    InvokeCommand {
        channel: Id,
        command: String,
        options: serde_json::Value,
        response: String,
    },
    CreateEmoji {
        guild: Id,
        name: String,
        path: std::path::PathBuf,
    },
    DeleteEmoji {
        guild: Id,
        id: Id,
    },
    SetOverwrite {
        channel: Id,
        target: Id,
        kind: String,
        allow: u64,
        deny: u64,
    },
    #[allow(dead_code)]
    ClearOverwrite {
        channel: Id,
        target: Id,
        kind: String,
    },
    #[allow(dead_code)]
    Ping,
}

#[derive(Clone, Debug)]
pub enum NetEvent {
    Session(Box<AuthResponse>),
    Bootstrap(Box<Bootstrap>),
    GuildDetail(Box<GuildDetail>),
    MembersPage {
        guild: Id,
        channel: Id,
        page: Box<MemberPage>,
        replace: bool,
    },
    Users(Vec<User>),
    Lookup(Vec<User>),
    Bots(Vec<User>),
    BotCreated {
        bot: User,
        token: String,
    },
    Applications(Vec<Application>),
    ApplicationCreated {
        application: Application,
        token: String,
    },
    TokenRevealed {
        label: String,
        token: String,
    },
    InstallLink(String),
    ApplicationInstalled(Application),
    RoleCreated(Role),
    RoleUpdated(Role),
    RoleLinks {
        role: Id,
        links: Vec<RoleLink>,
    },
    RoleDeleted {
        guild: Id,
        role: Id,
    },
    Emojis(Vec<Emoji>),
    EmojiCreated(Emoji),
    EmojiDeleted {
        #[allow(dead_code)]
        guild: Id,
        id: Id,
    },
    AuditPage {
        entries: Vec<AuditEntry>,
        older: bool,
    },
    GuildCommands(Vec<ApplicationCommand>),
    UserUpdate(Box<User>),
    Notice(String),
    MemberUpdate(Box<Member>),
    MemberRemove {
        guild: Id,
        user: Id,
    },
    MemberCountUpdate {
        guild: Id,
        count: u32,
    },
    Messages {
        channel: Id,
        messages: Vec<Message>,
        before: bool,
    },
    MessageCreate(Box<Message>),
    MessageUpdate(Box<Message>),
    MessageDelete {
        channel: Id,
        message: Id,
    },
    Presence {
        user: Id,
        status: Presence,
    },
    Typing {
        channel: Id,
        user: Id,
        until: i64,
    },
    GuildChanged,
    GuildLeft(Id),
    ChannelNotify {
        channel: Id,
        level: NotificationLevel,
        suppress: bool,
        clear: bool,
    },
    GuildNotifyUpdated {
        guild: Id,
        settings: GuildNotificationSettings,
    },
    PublicGuilds(Vec<PublicGuild>),
    JoinedPublicGuild(Id),
    Pins(Vec<Message>),
    Bans(Vec<Ban>),
    Unbanned {
        guild: Id,
        user: Id,
    },
    GuildUpdated(Box<GuildSummary>),
    Mutuals {
        user: Id,
        guilds: Vec<GuildSummary>,
    },
    DmUpdated(Box<DmChannel>),
    DmRemoved {
        channel: Id,
    },
    PrunePreview {
        days: u32,
        count: u32,
    },
    Pruned {
        count: u32,
    },
    Appeals(Vec<BanAppeal>),
    AppealResolved {
        appeal: Id,
        accepted: bool,
    },
    AppealSubmitted,
    GuildDiscovery {
        guild: Id,
        discoverable: bool,
    },
    ChannelCreate(Box<Channel>),
    ChannelUpdated(Box<Channel>),
    ThreadCreated(Box<Channel>),
    ChannelDelete(Id),
    RelationshipChanged,
    Image {
        url: String,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        frames: Vec<crate::image_anim::AnimFrame>,
    },
    Uploaded(Box<Attachment>),
    Search(Box<SearchResults>),
    Inbox(Box<Inbox>),
    GuildEvents(Vec<GuildEvent>),
    GuildEventCreated(GuildEvent),
    GuildEventDeleted {
        guild: Id,
        event: Id,
    },
    ServerChecked {
        url: String,
        ok: bool,
        error: Option<String>,
    },
    ConnectionFailed,
    Dm(Box<DmChannel>),
    VoiceState(Box<VoiceState>),
    Error(String),
    Connected(bool),
}

pub struct Net {
    tx: Sender<Command>,
    wake: Arc<Mutex<Option<rune::Waker>>>,
}

impl Net {
    pub fn send(&self, cmd: Command) {
        let _ = self.tx.send(cmd);
    }

    pub fn set_waker(&self, waker: rune::Waker) {
        if let Ok(mut w) = self.wake.lock() {
            *w = Some(waker.clone());
        }
        // A worker can enqueue an event just before the app installs its waker.
        // Wake once so that event is drained rather than waiting for the next one.
        if EVENTS
            .lock()
            .map(|queue| !queue.is_empty())
            .unwrap_or(false)
        {
            waker();
        }
    }
}

struct Shared {
    base: String,
    wake: Arc<Mutex<Option<rune::Waker>>>,
    token: Mutex<Option<String>>,
    profile: String,
    client: reqwest::blocking::Client,
    bootstrap_seen: AtomicBool,
}

fn session_path(profile: &str) -> Option<std::path::PathBuf> {
    let base = std::env::var("APPDATA")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok())?;
    Some(base.join("Runecord").join(format!("session-{profile}.txt")))
}

fn read_token(profile: &str) -> Option<String> {
    let path = session_path(profile)?;
    std::fs::read_to_string(path)
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

/// Whether this profile has a persisted session token.
pub fn has_token(profile: &str) -> bool {
    read_token(profile).is_some()
}

fn write_token(profile: &str, token: Option<&str>) {
    let Some(path) = session_path(profile) else {
        return;
    };
    match token {
        Some(t) => {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(path, t);
        }
        None => {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn error_text(response: reqwest::blocking::Response) -> String {
    let status = response.status();
    let text = response.text().unwrap_or_default();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text)
        && let Some(e) = v.get("error").and_then(|e| e.as_str())
    {
        return e.to_string();
    }
    if text.is_empty() {
        format!("Request failed ({status})")
    } else {
        text
    }
}

impl Shared {
    fn get(&self, path: &str) -> Result<serde_json::Value, String> {
        let token = self.token.lock().map_err(|_| "lock")?.clone();
        let mut req = self.client.get(format!("{}{}", self.base, path));
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        let res = req
            .send()
            .map_err(|e| format!("Can't reach the local server: {e}"))?;
        if !res.status().is_success() {
            return Err(error_text(res));
        }
        res.json::<serde_json::Value>().map_err(|e| e.to_string())
    }

    fn post<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<serde_json::Value, String> {
        let token = self.token.lock().map_err(|_| "lock")?.clone();
        let mut req = self
            .client
            .post(format!("{}{}", self.base, path))
            .json(body);
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        let res = req
            .send()
            .map_err(|e| format!("Can't reach the local server: {e}"))?;
        if !res.status().is_success() {
            return Err(error_text(res));
        }
        if res.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(serde_json::Value::Null);
        }
        res.json::<serde_json::Value>().map_err(|e| e.to_string())
    }

    fn patch<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<serde_json::Value, String> {
        let token = self.token.lock().map_err(|_| "lock")?.clone();
        let mut req = self
            .client
            .patch(format!("{}{}", self.base, path))
            .json(body);
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        let res = req
            .send()
            .map_err(|e| format!("Can't reach the local server: {e}"))?;
        if !res.status().is_success() {
            return Err(error_text(res));
        }
        res.json::<serde_json::Value>().map_err(|e| e.to_string())
    }

    fn delete_json(&self, path: &str) -> Result<serde_json::Value, String> {
        let token = self.token.lock().map_err(|_| "lock")?.clone();
        let mut req = self.client.delete(format!("{}{}", self.base, path));
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        let res = req
            .send()
            .map_err(|e| format!("Can't reach the local server: {e}"))?;
        if !res.status().is_success() {
            return Err(error_text(res));
        }
        res.json::<serde_json::Value>().map_err(|e| e.to_string())
    }

    fn delete(&self, path: &str) -> Result<(), String> {
        let token = self.token.lock().map_err(|_| "lock")?.clone();
        let mut req = self.client.delete(format!("{}{}", self.base, path));
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        let res = req
            .send()
            .map_err(|e| format!("Can't reach the local server: {e}"))?;
        if !res.status().is_success() {
            return Err(error_text(res));
        }
        Ok(())
    }

    fn emit(&self, ev: NetEvent) {
        if let Ok(mut q) = EVENTS.lock() {
            q.push(ev);
        }
        if let Ok(w) = self.wake.lock()
            && let Some(waker) = w.as_ref()
        {
            waker();
        }
    }
}

pub static EVENTS: Mutex<Vec<NetEvent>> = Mutex::new(Vec::new());

pub fn drain_events() -> Vec<NetEvent> {
    let mut out = Vec::new();
    if let Ok(mut v) = EVENTS.lock() {
        std::mem::swap(&mut out, &mut *v);
    }
    out
}

pub fn start(fast_profile: Option<String>, _hydrated: bool) -> Net {
    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
    let base = crate::config::effective_server_url();
    let profile = fast_profile
        .or_else(|| std::env::var("RUNECORD_PROFILE").ok())
        .unwrap_or_else(|| "1".to_string());
    let wake = Arc::new(Mutex::new(None));
    let shared = Arc::new(Shared {
        base,
        wake: wake.clone(),
        token: Mutex::new(read_token(&profile)),
        profile,
        client: reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new()),
        bootstrap_seen: AtomicBool::new(false),
    });

    if shared.token.lock().map(|t| t.is_some()).unwrap_or(false) {
        // Never block app construction on a network round trip. A warm cache
        // paints immediately; the gateway READY normally revalidates it. If the
        // socket is slow, a delayed bootstrap acts as the fallback.
        let fetch_shared = shared.clone();
        std::thread::spawn(move || {
            // Give the gateway a chance to deliver READY first, so a normal
            // launch performs one bootstrap instead of two. The HTTP fetch is
            // the fallback when the socket is slow or unavailable.
            std::thread::sleep(Duration::from_millis(500));
            if fetch_shared.bootstrap_seen.load(Ordering::SeqCst) {
                return;
            }
            fetch_bootstrap(&fetch_shared);
        });
    }

    {
        let shared = shared.clone();
        std::thread::spawn(move || http_worker(shared, cmd_rx));
    }
    {
        let shared = shared.clone();
        std::thread::spawn(move || ws_worker(shared));
    }

    Net { tx: cmd_tx, wake }
}

mod gateway;
mod http;

use self::gateway::*;
use self::http::*;
