//! Transport: blocking HTTP commands on a worker thread plus a reconnecting gateway socket.
//! Nothing here touches the UI thread.

use runecord_protocol::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Percent-encodes one URL component (query value, path segment).
///
/// RFC 3986 unreserved bytes pass through; everything else, including every
/// UTF-8 continuation byte, becomes `%XX`. This is the only encoding the HTTP
/// handlers need, so it replaces a dependency with a few lines.
pub(crate) fn encode_component(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(value.len());
    for &byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push(HEX[(byte >> 4) as usize] as char);
                out.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
    out
}

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
        signup_code: String,
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
    ForwardMessage {
        message: Id,
        channel: Id,
    },
    CreatePoll {
        channel: Id,
        question: String,
        options: Vec<String>,
        multiple: bool,
        anonymous: bool,
        duration_hours: Option<u32>,
    },
    ScheduleMessage {
        channel: Id,
        content: String,
        send_at: i64,
    },
    LoadScheduled {
        channel: Id,
    },
    DeleteScheduled {
        id: Id,
    },
    VotePoll {
        message: Id,
        option: u32,
    },
    ClickComponent {
        message: Id,
        custom_id: String,
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
        recurrence: String,
        duration_minutes: u32,
        reminder_minutes: u32,
    },
    RsvpEvent {
        guild: Id,
        event: Id,
        status: String,
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
    ConnectionChallenge,
    VerifyConnection {
        kind: String,
        handle: String,
        url: String,
    },
    UploadActivityArtwork {
        source: UploadSource,
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
        source: UploadSource,
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
        source: UploadSource,
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
        source: UploadSource,
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
    UpdateOnboarding {
        guild: Id,
        update: UpdateOnboarding,
    },
    LoadAutoMod {
        guild: Id,
    },
    SaveAutoMod {
        guild: Id,
        rules: Vec<AutoModRule>,
    },
    LoadTemplates {
        guild: Id,
    },
    CreateTemplate {
        guild: Id,
        name: String,
        description: String,
    },
    DeleteTemplate {
        guild: Id,
        template: Id,
    },
    ApplyTemplate {
        code: String,
        name: String,
    },
    LoadWebhooks {
        guild: Id,
    },
    CreateWebhook {
        channel: Id,
        name: String,
    },
    DeleteWebhook {
        guild: Id,
        id: Id,
    },
    AcceptOnboarding {
        guild: Id,
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
        tag_ids: Vec<Id>,
    },
    SetForumTags {
        channel: Id,
        tags: Vec<ForumTag>,
    },
    SetThreadTags {
        thread: Id,
        tags: Vec<Id>,
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
        source: UploadSource,
    },
    Voice {
        guild: Id,
        channel: Option<Id>,
        mute: bool,
        deaf: bool,
    },
    StageAction {
        channel: Id,
        action: String,
        user: Option<Id>,
    },
    LoadStage {
        channel: Id,
    },
    LoadActivities {
        guild: Id,
    },
    StartActivity {
        channel: Id,
        activity: Id,
    },
    StopActivity {
        channel: Id,
    },
    LoadGifs {
        query: String,
    },
    SendGif {
        channel: Id,
        slug: String,
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
        source: UploadSource,
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
        actor: Option<Id>,
        action: Option<i32>,
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
        sticker: bool,
        source: UploadSource,
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
    ConnectionCode(String),
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
    Scheduled(Vec<ScheduledMessage>),
    ScheduledCreated(ScheduledMessage),
    ScheduledDeleted(Id),
    Bans(Vec<Ban>),
    Unbanned {
        guild: Id,
        user: Id,
    },
    GuildUpdated(Box<GuildSummary>),
    OnboardingUpdated(Box<GuildOnboarding>),
    AutoModConfig(Box<AutoModConfig>),
    Templates(Vec<GuildTemplate>),
    Webhooks(Vec<WebhookInfo>),
    WebhookCreated(Box<WebhookInfo>),
    ForumTags(Vec<ForumTag>),
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
    GuildEventUpdated(GuildEvent),
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
    StageUpdated(Box<StageState>),
    Activities(Vec<Activity>),
    ActivityUpdated(Box<ActivityUpdate>),
    Gifs(Vec<GifItem>),
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

#[cfg(target_arch = "wasm32")]
const NOT_PORTED: &str = "That action is not available in the web client yet.";

pub(crate) struct Shared {
    base: String,
    wake: Arc<Mutex<Option<rune::Waker>>>,
    token: Mutex<Option<String>>,
    profile: String,
    #[cfg(not(target_arch = "wasm32"))]
    client: reqwest::blocking::Client,
    bootstrap_seen: AtomicBool,
}

#[cfg(target_arch = "wasm32")]
fn session_path(_profile: &str) -> Option<std::path::PathBuf> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn session_path(profile: &str) -> Option<std::path::PathBuf> {
    Some(crate::paths::data_dir()?.join(format!("session-{profile}.txt")))
}

#[cfg(target_arch = "wasm32")]
fn read_token(_profile: &str) -> Option<String> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item("runecord-token").ok().flatten())
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
fn write_token(_profile: &str, token: Option<&str>) {
    let Some(storage) = web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    else {
        return;
    };
    match token {
        Some(token) => {
            let _ = storage.set_item("runecord-token", token);
        }
        None => {
            let _ = storage.remove_item("runecord-token");
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
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
    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(not(target_arch = "wasm32"))]
    fn put<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<serde_json::Value, String> {
        let token = self.token.lock().map_err(|_| "lock")?.clone();
        let mut req = self.client.put(format!("{}{}", self.base, path)).json(body);
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

    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(not(target_arch = "wasm32"))]
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

    /// One call for every verb, used by the transport-neutral helpers.
    #[cfg(not(target_arch = "wasm32"))]
    fn send(
        &self,
        method: HttpMethod,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match method {
            HttpMethod::Get => self.get(path),
            HttpMethod::Post => self.post(path, body),
            HttpMethod::Put => self.put(path, body),
            HttpMethod::Patch => self.patch(path, body),
            HttpMethod::Delete => self.delete_json(path),
        }
    }

    /// POST a raw byte body, used by uploads.
    #[cfg(not(target_arch = "wasm32"))]
    fn post_bytes(
        &self,
        path: &str,
        bytes: &[u8],
        content_type: &str,
    ) -> Result<serde_json::Value, String> {
        let token = self.token.lock().map_err(|_| "lock")?.clone();
        let mut req = self
            .client
            .post(format!("{}{}", self.base, path))
            .header("Content-Type", content_type)
            .body(bytes.to_vec());
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

    // Browser builds keep the same call sites for handlers that have not been
    // ported to the callback form yet; they report a clear error instead of
    // blocking the single UI thread.
    #[cfg(target_arch = "wasm32")]
    fn get(&self, _path: &str) -> Result<serde_json::Value, String> {
        Err(NOT_PORTED.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    fn post<B: serde::Serialize>(
        &self,
        _path: &str,
        _body: &B,
    ) -> Result<serde_json::Value, String> {
        Err(NOT_PORTED.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    fn put<B: serde::Serialize>(
        &self,
        _path: &str,
        _body: &B,
    ) -> Result<serde_json::Value, String> {
        Err(NOT_PORTED.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    fn patch<B: serde::Serialize>(
        &self,
        _path: &str,
        _body: &B,
    ) -> Result<serde_json::Value, String> {
        Err(NOT_PORTED.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    fn delete_json(&self, _path: &str) -> Result<serde_json::Value, String> {
        Err(NOT_PORTED.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    fn delete(&self, _path: &str) -> Result<(), String> {
        Err(NOT_PORTED.to_string())
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

// ---------------------------------------------------------------------------
// Transport-neutral request helpers
//
// Desktop runs each request on the worker thread and calls back at once; the
// browser spawns a fetch and calls back when it resolves. Handler code is the
// same either way, so `net/http` compiles for both targets.
// ---------------------------------------------------------------------------

/// GET a path and decode it, handing the outcome to `then`.
pub(crate) fn get_then<T, F>(shared: &Arc<Shared>, path: String, then: F)
where
    T: serde::de::DeserializeOwned + 'static,
    F: FnOnce(Result<T, String>) + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        then(
            shared
                .get(&path)
                .and_then(|value| serde_json::from_value::<T>(value).map_err(|e| e.to_string())),
        );
    }
    #[cfg(target_arch = "wasm32")]
    web::get_result(shared, path, then);
}

/// Any method with a JSON body, handing the raw response to `then`.
pub(crate) fn send_then<B, F>(
    shared: &Arc<Shared>,
    method: HttpMethod,
    path: String,
    body: B,
    then: F,
) where
    B: serde::Serialize,
    F: FnOnce(Result<serde_json::Value, String>) + 'static,
{
    let body = serde_json::to_value(&body).unwrap_or(serde_json::Value::Null);
    #[cfg(not(target_arch = "wasm32"))]
    {
        then(shared.send(method, &path, &body));
    }
    #[cfg(target_arch = "wasm32")]
    web::send_json(shared, method, path, body, then);
}

/// A request with no response body, reporting only success or failure.
pub(crate) fn fire_then<B, F>(
    shared: &Arc<Shared>,
    method: HttpMethod,
    path: String,
    body: B,
    then: F,
) where
    B: serde::Serialize,
    F: FnOnce(Result<(), String>) + 'static,
{
    send_then(shared, method, path, body, move |result| {
        then(result.map(|_| ()))
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)] // `Get` travels through `get_then`, which does not name it
pub(crate) enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// Where an uploaded file's bytes come from.
///
/// Desktop reads a path returned by the native file dialog; the browser holds
/// the bytes from an `<input type="file">` instead. Handlers only see the
/// resolved `(name, bytes)`, so an upload is written once for both targets.
#[derive(Clone, Debug)]
pub enum UploadSource {
    Path(std::path::PathBuf),
    Bytes { name: String, bytes: Vec<u8> },
}

impl UploadSource {
    /// Resolves the source into a filename and its bytes.
    pub fn read(&self) -> Result<(String, Vec<u8>), String> {
        match self {
            UploadSource::Path(path) => {
                let bytes = std::fs::read(path).map_err(|e| format!("Could not read file: {e}"))?;
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "file".into());
                Ok((name, bytes))
            }
            UploadSource::Bytes { name, bytes } => Ok((name.clone(), bytes.clone())),
        }
    }
}

/// Guesses a content type from a filename.
pub(crate) fn content_type_for(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    let ext = lower.rsplit('.').next().unwrap_or_default();
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "mp4" => "video/mp4",
        "mp3" => "audio/mpeg",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    }
}

/// POST raw bytes to a path and hand the decoded response to `then`.
pub(crate) fn upload_then<T, F>(
    shared: &Arc<Shared>,
    path: String,
    source: UploadSource,
    content_type: String,
    then: F,
) where
    T: serde::de::DeserializeOwned + 'static,
    F: FnOnce(Result<T, String>) + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        let result = source
            .read()
            .and_then(|(_, bytes)| shared.post_bytes(&path, &bytes, &content_type))
            .and_then(|value| serde_json::from_value::<T>(value).map_err(|e| e.to_string()));
        then(result);
    }
    #[cfg(target_arch = "wasm32")]
    {
        match source.read() {
            Ok((_, bytes)) => web::upload_json(shared, path, bytes, content_type, then),
            Err(e) => {
                let _ = then;
                web::report_error(shared, e);
            }
        }
    }
}

/// GET an image and decode it into the client's image event.
pub(crate) fn fetch_image_then(shared: &Arc<Shared>, url: String) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(res) = shared.client.get(format!("{}{}", shared.base, url)).send()
            && let Ok(bytes) = res.bytes()
        {
            emit_image(shared, url, &bytes);
        }
    }
    #[cfg(target_arch = "wasm32")]
    web::fetch_bytes(shared, url);
}

/// Decodes image bytes into a `NetEvent::Image`, honouring animation.
pub(crate) fn emit_image(shared: &Arc<Shared>, url: String, bytes: &[u8]) {
    if let Some(animation) = crate::image_anim::decode(bytes) {
        let first = animation.frames[0].rgba.clone();
        shared.emit(NetEvent::Image {
            url,
            width: animation.width,
            height: animation.height,
            rgba: first,
            frames: animation.frames,
        });
        return;
    }
    match image::load_from_memory(bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            shared.emit(NetEvent::Image {
                url,
                width: w,
                height: h,
                rgba: rgba.into_raw(),
                frames: Vec::new(),
            });
        }
        Err(e) => shared.emit(NetEvent::Error(format!("Image: {e}"))),
    }
}

pub fn drain_events() -> Vec<NetEvent> {
    let mut out = Vec::new();
    if let Ok(mut v) = EVENTS.lock() {
        std::mem::swap(&mut out, &mut *v);
    }
    out
}

#[cfg(target_arch = "wasm32")]
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
        bootstrap_seen: AtomicBool::new(false),
    });

    if shared.token.lock().map(|t| t.is_some()).unwrap_or(false) {
        let fetch_shared = shared.clone();
        wasm_bindgen_futures::spawn_local(async move {
            crate::net::web::wait(500).await;
            if !fetch_shared.bootstrap_seen.load(Ordering::SeqCst) {
                web::fetch_bootstrap(&fetch_shared);
            }
        });
    }

    {
        let shared = shared.clone();
        wasm_bindgen_futures::spawn_local(async move {
            loop {
                while let Ok(cmd) = cmd_rx.try_recv() {
                    http::dispatch(&shared, cmd);
                }
                crate::net::web::wait(16).await;
            }
        });
    }
    {
        let shared = shared.clone();
        wasm_bindgen_futures::spawn_local(async move {
            ws_worker(shared);
        });
    }

    Net { tx: cmd_tx, wake }
}

#[cfg(not(target_arch = "wasm32"))]
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

/// Send a body and decode the response, handing the outcome to `then`.
pub(crate) fn send_then_json<T, B, F>(
    shared: &Arc<Shared>,
    method: HttpMethod,
    path: String,
    body: B,
    then: F,
) where
    T: serde::de::DeserializeOwned + 'static,
    B: serde::Serialize,
    F: FnOnce(Result<T, String>) + 'static,
{
    send_then(shared, method, path, body, move |result| {
        then(result.and_then(|value| {
            serde_json::from_value::<T>(value).map_err(|error| error.to_string())
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::encode_component;

    #[test]
    fn query_values_are_percent_encoded() {
        assert_eq!(encode_component("hello world"), "hello%20world");
        assert_eq!(encode_component("a&b=c?d/e"), "a%26b%3Dc%3Fd%2Fe");
        assert_eq!(encode_component("Runecord_-.~"), "Runecord_-.~");
        assert_eq!(encode_component("café"), "caf%C3%A9");
        assert_eq!(encode_component("100%"), "100%25");
    }
}
