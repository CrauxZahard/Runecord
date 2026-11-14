# Runecord

A local Discord-style chat client built with [Rune](https://github.com/anomalyco/rune), the GPU-native Rust UI framework on this machine. The interface copies Discord's desktop layout, measurements and palettes; the backend is a standalone Rust service in the sibling repository `..\Runecord-backend`.

This is **not** a Discord client: it never talks to Discord. It is a self-hosted chat app that looks and behaves like Discord, with local accounts, servers, channels, DMs, roles, invites and real-time messaging.

```
Runecord/                 frontend repo (this one)
  crates/client           the Rune app
  assets/                 app icon
  run.bat / build.bat     launchers

..\Runecord-backend\      backend repo (separate git repository)
  crates/protocol         wire types the client depends on by path
  crates/server           HTTP API, WebSocket gateway, SQLite store
  migrations/             numbered SQL schema
  data/                   runtime database + uploads

..\rune\                  Rune framework checkout (path dependency)
```

Both siblings are required: the manifests use only relative paths, so the framework can live anywhere as long as `..\rune` resolves to it. On Windows a directory junction is enough if your Rune checkout is elsewhere.

## Run it

```bat
run.bat
```

The first launch builds the backend and client if needed, starts the backend in the background on `127.0.0.1:47800`, and opens the client. Local launches use the debug binaries so the window always shows the current source.

Demo accounts (seeded on first run, password `runecord`):

| username | display name |
|---|---|
| `nova` | Nova (server owner) |
| `maya` | Maya Chen |
| `leo` | Leo Park |
| `sam` | Sam Rivera |
| `ana` | Ana Souza |
| `priya` | Priya Nair |
| `tom` | Tom Vale |
| `runebot` | Runebot |

### Multiple accounts on one laptop

Each window is one account, and each profile stores its own session:

```bat
run.bat 1     :: Nova
run.bat 2     :: log in as maya/runecord
run.bat 3     :: sam/runecord, and so on
```

Messages, reactions, edits, deletes, typing indicators, presence and unread badges update live between the windows through the gateway.

### Reset

```bat
reset.bat
```

Deletes `..\Runecord-backend\data` and the saved sessions. The next launch seeds the demo server again.

## What works

- **Servers and channels** - create servers, create text channels, categories, invites, join by invite code, per-server roles with colours and hoisting, permission checks. Channel settings edit the name, topic, voice status, slowmode and age restriction, and deleting a channel removes its messages, overwrites and threads.
- **Chat** - send, edit, delete, reply, pin, reactions (searchable picker with categories, server emoji and clickable chips), a pinned-messages sheet with author, channel, time, jump and unpin, message grouping within seven minutes, centered date dividers, full date-and-time timestamps, typing indicators, unread/mention badges, message search with `from:`, `in:`, `mentions:`, `has:link`, `before:` and `after:` filters, a quick switcher for channels and DMs, selectable Markdown with code blocks (via `rune-kit`), rendered user and role mention pills, image attachments, native right-click menus, and an image viewer with zoom/pan. Empty channels bottom-align the welcome intro above the composer and expose Edit Channel to managers.
- **Animated media** - animated GIF and WebP play anywhere the shared image cache draws: avatars, profile banners, server icons and banners, custom emojis, inline attachments and the image viewer. The decoder composites each frame, keeps per-frame delays, scales frames to at most 512 px and caps each animation at 120 frames / 48 MB. Animated avatars, banners and group icons skip the static crop editor and upload the original file, while static images keep the crop/zoom flow. Uploads and CDN bytes persist, so animations replay after a restart.
- **Notifications** - Discord's server dropdown Notification Settings opens a server-level modal with Mute Server, All Messages / Only @mentions / Nothing, Suppress @everyone and @here, Suppress All Role @mentions, Suppress Highlights, Mute New Events, Mobile Push Notifications and a per-channel override list with a channel picker and remove action. Settings persist in SQLite and drive unread and mention counts; channel overrides take precedence.
- **Server events** - the sidebar Events shortcut opens a server event page with upcoming and past events, month/day blocks, start time, location channel, description and delete for the creator or managers. Create Event captures a name, description, start offset and the current channel, persists to SQLite and lists immediately for the acting client; other open clients refresh when they reopen Events because there is no gateway event yet. The API is `GET/POST /api/guilds/{id}/events` and `DELETE /api/guilds/{id}/events/{event}`.
- **Inbox** - the channel-header envelope opens a right-docked panel with Mentions and Unreads tabs. `GET /api/inbox` aggregates direct, @everyone/@here and role mentions plus unread guild and DM channels; each entry shows the server/channel breadcrumb, author, snippet, timestamp and unread count. Clicking an entry jumps to the channel, highlights the message and marks it read; each row and the header also offer mark-as-read. Mentions and unread counts come from the same read cursors and notification levels as the sidebar, and empty tabs have their own caught-up state.
- **DMs and friends** - a Discord-style Friends page with a Home/Nitro/Shop sidebar, Online/All/Pending/Blocked tabs, live friend search, Add Friend, a friend-row message action and an Active Now rail. A group-DM sheet picks 2–9 friends, optionally names the group, and creates a real `group_dm` channel; the sidebar and header render the group icon (or a stacked avatar) and either the name or the member list, and membership scopes messages, permissions and gateway delivery. Group settings let the owner rename the group and upload or remove a cropped icon, any member add friends up to 10 people, and the owner remove members; anyone can leave, the owner transfers automatically, and live `DmUpdate`/`DmRemove` events update every open client without a restart. Friend requests use username lookup (not just people already loaded); DM conversations carry online/idle/DND/offline presence, custom statuses, and send permissions from the server.
- **Server menu and notifications** - the server dropdown opens under the header with Invite People, Server Settings, Create Channel, Notification Settings, Privacy, App Directory and Leave Server. Leaving removes the membership, roles, reads and notification overrides and switches to the next server. Every channel has persisted All Messages / Only @mentions / Nothing levels plus a Suppress @everyone and @here switch, and muted channels show a crossed-bell icon in the sidebar. These drive unread counts and desktop toasts.
- **Moderation** - right-click a member for Timeout, Kick or Ban with role-hierarchy and permission checks. Dialogs confirm each action; timeouts take a custom number of minutes, hours or days up to 28 days, persist the exact expiry, show a clock in the member list and the profile popout with the expiry date, and block messages and voice until cleared. Bans store a reason and appear in Server Settings → Bans with Unban. A banned user can submit an appeal from the Join Server flow; managers see pending appeals in Server Settings → Bans and accept or deny them, with the outcome stored and audited. Server Settings → Members → Prune Members previews inactive members by inactivity window and role protection, then prunes on confirmation. Every action writes an audit-log entry.
- **Server discovery** - Explore Public Servers lists discoverable local servers you have not joined, with member counts and descriptions, and joins one directly into the guild rail. A seeded `Rune Community` server gives the directory something to show on first run. Managers can hide or show a server in Explore from Privacy Settings.
- **Server boosts** - each member can boost a server once; boosts persist and count toward Discord's 2/7/14 levels. The perks are enforced, not labels: upload caps become 48/64/96/128 MB and voice Opus bitrate becomes 64/128/256/384 kbps at Levels 0/1/2/3. Overview shows a server banner, level badge, boost count and progress to the next level with active and locked perk rows, plus Boost Server / Remove Boost. The server dropdown shows the same Level and x/y progress. Boosting is local; no billing or Nitro is involved.
- **Files** - drag a file onto the composer to upload it, with hover and rejected-file feedback; images render inline, other files show as attachments. The cap is 48 MB, rising to 64/96/128 MB with server Levels 1/2/3; the client rejects oversized files before upload and the server enforces the same limit.
- **UI** - Discord's 72 px server rail with circular server icons, 232 px settings nav, 240 px channel sidebar, 48 px channel header, 240 px member list grouped by hoisted role with collapsible groups, virtualized message rows, frosted composer and dialogs, full-window user and server settings with grouped GENERAL/MODERATION nav, scrollable content and a sticky Save/Reset footer that only activates when something changed, quick switcher, search, working emoji/reaction picker, uploaded profile avatars and banners (cropped, stored encrypted, shown in messages, member lists and profiles), profile badges and avatar decoration rings, profile views with a taller banner, overlapping avatar with presence dot, custom-status pill, badges beside the handle, a Spotify/playing activity card with artwork, a platform badge and a green Play on Spotify action, animated profile effects with orbiting highlights, gradient nameplates behind the display name, an At a glance/Now playing widget on the full profile, connected-account chips, joined dates, role-coloured chips, mutual servers, View Full Profile, and a self view that swaps Message/Add Friend for Edit Profile, inline Add Friend and a More menu (copy ID, remove friend or cancel a request, block), light/dark themes and a compact-row mode. Member lists are scoped to the selected channel, so members denied `View Channel` are omitted. Voice channels show a status subtitle (managers see Set a channel status until one is written). The voice view splits into the participant stage and an optional right-side text chat panel using the channel's own messages and composer, with circular camera, screen-share, quality, mute, deafen and leave controls, plus a Soundboard panel that broadcasts six built-in effects into the call; connected voice users appear beneath their voice channel, and a Voice Connected card with the channel, server, camera, screen-share, voice-chat and disconnect controls sits above the account panel.
- **Settings** - profile display name, bio, custom status, avatar color/border/banner, effect/nameplate/widget pickers, password change, notification toggles, theme, row density, developer mode, editable keybinds, privacy controls for new DMs and friend requests, bot/application management (create, one-time token reveal, revoke/delete, install links), preference reset and warm-cache clearing all write real state.
- **Server identity** - server icons and banners upload as PNG, JPEG, GIF or WebP. Selecting a static image opens a crop editor with drag-to-reposition, zoom from 100% to 300%, reset, arrow-key nudges and a live crop preview; Apply writes the cropped PNG. Animated GIF/WebP files bypass the crop and keep their frames. Overview shows large previews with Upload/Remove and drag-and-drop, the rail renders the icon, and servers with a banner show it at the top of the channel sidebar and in the server dropdown. Banners unlock at Boost Level 2 (7 Boosts); the server enforces the gate. Images are stored encrypted under `data/cdn/` and served from `/cdn/`.
- **Server settings** - a full-window page with Overview, Roles, Members, Bans, Emojis and Audit Log in the left nav. Overview includes a server banner and boost-level badge, edits the server name, icon colour, description, tag and Explore discoverability through `PATCH /api/guilds/{id}`, and updates the rail, header and settings identity immediately. Roles creates, renames, colours, reorders and deletes roles through a tabbed editor (Display, Permissions, Links, Manage Members) with a live chat preview, a role-colour hex readout, a Boost Level 2-gated role icon shown in role chips and member lists, drag-to-reorder rows (Up/Down buttons remain as the keyboard path), grouped permission checkboxes, persisted role links, per-member role toggles and a full-width unsaved-changes footer with Reset and Save Changes that only activates when something changed. Members assigns and removes roles and opens Prune Members (inactivity days, role inclusion, preview count, confirmation); Bans lists banned users with reason, actor, date and Unban; Emojis adds by naming and dropping a PNG; Audit Log pages through role, member-role, member moderation, prune, emoji and webhook actions with actor, target and change summary. Role changes broadcast to connected clients.
- **Threads and forums** - create a thread from a message's menu or from a channel's thread list, open it with a parent-channel context bar and message count, archive/unarchive it, and use forum cards with previews. New posts take a title and body, open as their own thread and keep the parent context. Threads inherit the parent channel's permissions.
- **Custom emoji** - guild emojis load through bootstrap, `:name:` renders inline at message size, and the composer/reaction picker shows a This Server section with the image entries.
- **Slash commands** - the composer `/` picker lists commands from applications installed in the guild. Selecting one fills the composer; sending posts `/api/interactions`, and the local app runtime returns a callback that appears as a bot message. The seeded Rune Tools app provides `/ping` and `/echo`.
- **Developer mode** - a live bottom bar with gateway state, guild/channel/message/member counts, guild-detail LRU usage, image cache size, snapshot age and profile id.
- **Accessibility** - controls expose roles, names and state through Rune's AccessKit tree; toasts are polite live regions; icon buttons carry labels and help text.
- **Profiles** — avatar borders, profile banners, guild/server tags.
- **Desktop integration** - Windows toast notifications for DMs and mentions while the window is unfocused, plus a tray icon with Open/Quit.
- **Voice & Video settings** - a Discord-style settings tab with a DirectShow microphone picker and a native WASAPI render-endpoint picker, input/output volume, a live mic test, Voice Activity / Push to Talk modes, automatic or manual input sensitivity, and a real DirectShow camera preview. Push to Talk is a global **F8 hold** checked with `GetAsyncKeyState`, so releasing it silences the outgoing PCM. Input sensitivity gates the captured PCM before Opus encoding and volume scales it. Output-device selection runs through an in-process shared-mode WASAPI sink with automatic sample-rate and channel conversion; there is no helper executable, and FFplay remains only as a fallback if WASAPI initialization fails. Settings persist in the warm cache and are applied on the next join.
- **Voice** - joining a voice channel opens a local audio relay: microphone capture through FFmpeg DirectShow, Opus 48 kHz mono encoding, WebSocket fan-out, Opus decoding and native shared-mode WASAPI playback to the selected endpoint. Late joiners request the Ogg headers from the relay, senders resend them on demand, and the 15-second fallback refresh is ignored by existing decoders, so playback runs uninterrupted. The Opus bitrate follows the server's boost level (64/128/256/384 kbps). Input mute and output deafen are live in the user panel. Set `RUNECORD_FFMPEG`/`RUNECORD_FFPLAY` if the binaries are not on `PATH`.
- **Video and screen share** - Discord's voice stage with a tile per participant. Camera capture uses FFmpeg DirectShow, screen capture uses `gdigrab`; frames are JPEG relayed over the same voice WebSocket. A quality selector offers Auto, 360p (15 fps), 480p (24 fps), 720p and 1080p (30 fps); the choice persists across restarts, and Auto steps between 360p and 720p when the outbound frame queue backs up. Every participant can publish at once: the relay fans each user's stream out to all peers, and the stage lays the tiles out as a responsive 16:9 grid (one large tile, then two-by-two, then three columns). The sender publishes two simulcast layers (the selected quality plus a 360p/15 layer); each viewer picks Auto, High or Data saver, and Auto drops to the low layer if its decoder falls behind. The sender decodes its own main layer into a local `(You)` self-preview tile, so the badge matches the capture preset instead of always reading 360p. The receiver decodes with FFmpeg at the active layer’s native resolution (854×480, 1280×720 or 1920×1080), always taking the newest frame so stale ones are dropped, and shows the active layer’s resolution and measured frame rate on each tile. Camera and screen share are mutually exclusive, stopping a share clears the remote tile, and local audio keeps flowing.
- **Bots** — backend bot accounts with `rcbot.…` tokens (`Authorization: Bot`), owned by human accounts; bot messages carry a BOT tag in the client.
- **API compatibility** — native REST/gateway plus a Discord-shaped `/api/v10` surface with rate-limit headers and 429s.
- **Keys** — `Ctrl+K` quick switcher, `Ctrl+F` search, `Ctrl+,` settings, `Esc` closes modals, `Ctrl+Enter` or the send button sends (Enter is a newline). The global shortcuts are rebindable in Settings → Keybinds and persist with the profile.

## Architecture

- **Backend** (`..\Runecord-backend`): axum + `rusqlite` (WAL), Argon2 password hashes, opaque bearer sessions, channel permission overwrites, `broadcast`-based WebSocket fan-out, uploads under `data/uploads`. Storage sits behind `store::Store`; Phase 2 adds a Postgres adapter and gateway RESUME.
- **Client** (`crates/client`): Rune views are plain Rust. Network I/O runs on worker threads - one blocking HTTP command loop and one reconnecting gateway socket - and the UI drains a shared event queue with a `Waker` plus a low-frequency `wake_after`. No network or disk work happens inside layout.
- **Source layout**: `main.rs` wires the app; `app/` owns state and splits into `actions`, `events`, `session`, `media`, `retention` and `seed`; `net/` splits into the HTTP and gateway workers; `ui/` has one module per surface (`rail`, `sidebar`, `chat`, `composer`, `members`, `dms`, `settings`, `modals`) plus shared `common` and `forms` primitives; `cache.rs` owns the snapshot format.
- **Server cache**: the backend keeps a 256-entry guild-detail LRU keyed by SQLite's write generation, a 32 MB decoded-thumbnail LRU, and a 512 MB on-disk thumbnail budget. No Redis: it is one local process, and writes invalidate by construction.
- **Rendering**: Rune's own wgpu renderer, text layout and springs - no Electron, no web view, no JavaScript.
- **Client cache**: a warm-start snapshot lives at `%APPDATA%\Runecord\cache-<profile>.json`. It is painted before the socket answers, then revalidated by the first bootstrap/READY. Retention is bounded: 500 messages per channel, 5,000 messages total, 8 channels per snapshot, 300 decoded images or 64 MB. Guilds are summaries in memory; guild details are LRU (6), members are paginated 100 at a time and capped at 300 per guild, and images are fetched only for the visible row range plus prefetch. Messages, members and channels use Rune's `virtual_rows`. The cache is plaintext, same trust boundary as the saved session token, and is deleted on logout/reset.
- **Data**: `..\Runecord-backend\data\runecord.db` and `data/uploads/`. Sessions live in `%APPDATA%\Runecord\session-<profile>.txt`.

## UI parity audit

The high-visibility Server Settings → Overview surface was compared against Discord's
[Server Invite Background](https://support.discord.com/hc/en-us/articles/4415841146391-Server-Invite-Background)
reference. The audit fixed the page title (`Server Overview`), grouped the left nav into
`GENERAL` and `MODERATION`, made the content column scroll, added Discord-style field helper
text and dividers, marked the 2/7/14 boost thresholds under the progress bar, and replaced the
below-the-fold save row with a sticky footer whose `Save Changes` button stays disabled until a
field changes. Evidence: `shots/ui-overview-side-by-side.png`.

## Honest limits

Remaining Discord gaps after the full-surface audit: Nitro, boosts, badges and avatar decorations are local demo data with no billing or entitlements; video has two simulcast layers rather than a full SFU; activities are not implemented; the soundboard has six built-in synthesized effects but no uploaded per-server catalog, per-sound volume or Boost-gated slots; profile effects, nameplates and widgets are local presets rather than Nitro purchases. The comparison sheets cover the guild view, member list, composer, voice split, profile, Inbox, Events, Server Settings, role editor, Friends, group DMs and user settings rather than every Discord dialog.

- **Voice output is shared-mode WASAPI.** The client enumerates render endpoints and opens the selected friendly name directly, converting the decoded 48 kHz mono PCM to the endpoint mix format. It does not request exclusive mode or expose per-application volume, and falls back to FFplay on the default endpoint only if WASAPI initialization fails. The Voice Connected camera, screen-share and voice-chat buttons work, and the Soundboard button opens a six-effect panel whose PCM is mixed into the outgoing capture stream, so the whole channel hears it even while the microphone is muted. Activities are not offered rather than shown as a dead placeholder. Server events are real data but have no recurring events, interested counts, reminders or external integrations. Push to Talk is fixed to F8 and is not rebindable. Existing listeners ignore duplicate header pages; a late joiner asks the relay for headers instead of waiting for the fallback refresh.
- **Video is multi-party but best-effort.** Every participant publishes a main layer (up to 1080p/30) and a 360p/15 low layer; viewers choose Auto, High or Data saver, and Auto falls back to the low layer when decoding cannot keep up. There are never more than two layers, and each peer runs its own FFmpeg capture and decode subprocesses, so a larger call scales with local CPU rather than a simulated SFU. The receiver decodes at the published layer resolution through FFmpeg, so 720p and 1080p tiles carry real detail; CPU and memory scale with that resolution. Frames reuse one Pixels surface per participant. Discord’s own capture runs on the GPU; this path shells out to FFmpeg.
- **Highlights and mobile push are stored for parity.** The switches persist, but this build has no Highlights feed and no mobile client; the user-level Notifications tab remains separate from the server modal.
- **Activity and connections are manual profile data.** The Profile tab can set a Spotify or playing card with title, subtitle, state, link and uploaded square artwork, plus up to six typed connection chips. Nothing is auto-detected from Spotify, Steam or a game process, connections are labels rather than OAuth-linked accounts, and the card has no synchronized progress bar or synced playback. Effects, nameplates and widgets are local presets: four animated effects, four gradient nameplates and two widget variants, with no Nitro store or purchased catalog. The custom status remains a rounded pill rather than Discord's speech bubble with a tail, and the voice chat panel opens and closes per session rather than remembering its previous state.
- **Badges and decorations are local presets.** Profile badges and the Aurora/Sakura/Flame/Smoke avatar rings are self-service demo data, not linked to Nitro, billing or account age. The profile effects and nameplates are local animation/gradient presets rather than Discord's purchased catalog, and there are no profile decorations beyond the avatar rings.
- **Inbox groups unreads per channel.** Each unread row shows the channel's latest message, not every message, and the panel caps mentions at 20 and unread channels at 30. Jumping loads the newest 50 messages, so an older message highlights only when it falls in that page. There is no For You tab, reaction/bookmark list or followed-thread section.
- **Role links are local references, not Discord connections.** The Links tab stores up to five label/URL pairs per role and they persist and audit like the rest of the editor, but they do not feed Discord's linked-role OAuth metadata. Role icons require Boost Level 2, accept PNG/JPEG/GIF/WebP up to 4 MB and must be square; there is no in-app crop or rotation, and the Display preview is a two-row message mock rather than live channel content.
- **Group DMs have a single owner.** There are no moderators or role badges inside a group, ownership only transfers when the owner leaves, animated group icons play but cannot be cropped, and a group with no members is deleted with its messages.
- **Image cropping is static-only.** The editor supports drag, zoom and reset but not rotation or aspect presets, and animated files skip it entirely; icons must end up square (up to 4 MB) and banners landscape (up to 10 MB). Animations are GIF/WebP only (no APNG), are downscaled to 512 px, and cap at 120 frames / 48 MB per image with no pause control. Banners are gated behind Boost Level 2 like Discord.
- **Context menus are native on Windows and macOS.** Other platforms fall back to the in-app message menu. Pasting an image from the clipboard into the composer is not wired yet; file drops are.
- **Discord assets are not bundled.** The look uses Discord's metrics and palette with Inter (Rune's bundled font) and redrawn SVG icons. No Discord trademarks or `gg sans` files are included.
- **Local-only by design.** The server binds loopback only. Multi-node/Postgres deployment is Phase 2/3 of the backend plan.
- **Rune is pre-release.** The client path-depends on the local Rune checkout at `Documents\Default Project\rune`; Rune's `vendor/winit` and `vendor/wgpu-hal` patches are applied in this workspace `Cargo.toml`.
- **Storage boundary is partial.** Phase 1 routes sessions/bootstrap through `Store`, but many handlers still use the SQLite connection directly; Phase 2 completes the migration.

## Build

```bat
build.bat
```

builds both repositories. `run.bat` builds and launches the debug binaries, so it always shows the current source; use the commands below for release builds.

```bat
cargo build --release -p runecord            :: frontend
..\Runecord-backend\run-server.bat           :: backend (dev)
```

For headless UI checks the client supports Rune's tooling:

```bat
target\release\runecord.exe --snap shot.png --size 1360x860
target\release\runecord.exe --scene light --snap light.png
```

The live end-to-end run uses `RUNECORD_LIVE=1` with a Rune script (see `shots/*.json`).

Benchmarks and the Discord comparison are in [`BENCHMARKS.md`](BENCHMARKS.md).
