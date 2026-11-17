# Runecord

A local self-hosted chat client built with [Rune](https://github.com/anomalyco/rune), the GPU-native Rust UI framework on this machine. The interface uses a familiar desktop chat layout, measurements and palettes; the backend is a standalone Rust service in the sibling repository `..\Runecord-backend`.

It is a self-hosted chat app with local accounts, servers, channels, DMs, roles, invites and real-time messaging.

```
Runecord/                 frontend repo (this one)
  crates/client           the Rune app
  assets/                 app icon (png for runtime, ico for Windows)
  packaging/              installer, Linux scripts, portable readme
  download/               the download chooser page copied into dist
  run.bat / build.bat     launchers

..\Runecord-backend\      backend repo (separate git repository)
  crates/protocol         wire types the client depends on by path
  crates/server           HTTP API, WebSocket gateway, SQLite store
  migrations/             numbered SQL schema
  data/                   runtime database + uploads

..\rune\                  Rune framework checkout (path dependency)
```

Both siblings are required: the manifests use only relative paths, so the framework can live anywhere as long as `..\rune` resolves to it. On Windows a directory junction is enough if your Rune checkout is elsewhere.

## Platforms and downloads

Two desktop targets. `build.bat` builds, signs and packages both into `dist\`:

| Platform | Artifact | How it runs |
|---|---|---|
| Windows 10+ | `dist\Runecord-Setup-0.1.0.exe` | Per-user installer (no admin), Start-menu shortcut and uninstaller. `dist\Runecord.exe` is the portable copy. |
| Linux x86_64 | `dist\Runecord-linux.tar.gz` | Unpack and run `./install.sh` (user-local, no root) or `./Runecord`. |

Windows binaries are signed with a self-signed certificate: the signature is
verifiable, but the certificate is not from a public authority, so SmartScreen
may still warn. Set `-Pfx`/`-PfxPassword` on `scripts\build-installer.ps1` to
use a purchased certificate.

`dist\index.html` is the download chooser: it names each file, its size and
requirements, and highlights the card for the visitor's platform. The page
source lives in `download/index.html`; `build.bat` copies it next to the
artifacts. `--desktop-only` skips the Linux build.

There is no browser or mobile client in this repository. Android is a separate
project; nothing here builds a wasm bundle or an APK.

A narrow window (below 720 points wide, or below 520 points tall) uses a
one-pane layout with a back button, a member drawer, larger controls and a
settings list that drills into one section. Modal sheets cap to the window
through `sheet_w`/`sheet_h`.


## Run it

```bat
run.bat
```

The first launch builds the client with the embedded backend and opens one window on `127.0.0.1:47800`. It uses the debug profile so every run shows the current source.

Development builds seed demo accounts (password `runecord`) when the database is empty. **Release builds seed nothing**: a fresh install starts at registration and the first account you create owns that server.

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

### Point it at another server

Runecord is self-hosted. Open **User Settings → Application → Backend Server**, enter the deployment's `http://` or `https://` URL, choose **Save Changes**, then **Restart Runecord**. Saving validates the endpoint by requesting `/api/version`; an unreachable address or a server that does not identify as Runecord is rejected and left unsaved. The URL is stored in `%APPDATA%\Runecord\config.json` (`~/.local/share/runecord/config.json` on Linux); `RUNECORD_URL` overrides it for scripted runs. Changing it signs out saved profiles because their tokens belong to the previous backend.

A deployment beyond localhost should run the standalone server with `--host 0.0.0.0` and put TLS in front of it: the server has no TLS of its own, and the startup log warns when it binds a non-loopback address. Browser origins are denied by default; a web front end must be listed with `--allow-origin`. WebSocket authentication is header-only (there is no token in the query string).

Registration is open by default. Before exposing the server, gate it:

```bat
runecord-server --host 0.0.0.0 --signup code --signup-code "a-long-shared-code"
runecord-server --host 0.0.0.0 --signup closed
```

`/api/version` reports the mode, so the client shows the signup-code field or hides registration. Login and registration are limited per client IP (5 per minute); behind a TLS reverse proxy add `--trust-proxy` so the proxy's `X-Forwarded-For` is used instead of the proxy's own address. Never enable it when clients connect directly, or they can spoof the header.

The desktop build's embedded server reads the same gate from `RUNECORD_SIGNUP` (`open`, `code`, `closed`) and `RUNECORD_SIGNUP_CODE`, for the case where the app itself sits behind a tunnel.

### Passwords, backups and updates

- **Forgot a password:** `Runecord.exe --reset-password USER` (or `runecord-server --reset-password USER`) sets a random password, ends that account's sessions and shows or prints the result. There is no email reset.
- **Backups:** the server writes an encrypted snapshot into `server/backups/` at startup and every 24 hours, keeping the newest 7. Set `RUNECORD_BACKUPS` to change the count. The database key is `server/key.bin`; back it up with the snapshots or they cannot be opened.
- **Updates:** Settings → Application checks GitHub releases and shows a Download link when a newer version exists. Installing is manual; the app never replaces itself.

### Reset

```bat
reset.bat
```

Deletes the development backend data and the saved sessions. The next debug launch seeds the demo server again.

## What works

- **Servers and channels** - create servers, create text channels, categories, invites, join by invite code, per-server roles with colours and hoisting, permission checks. Channel settings edit the name, topic, voice status, slowmode and age restriction, and deleting a channel removes its messages, overwrites and threads.
- **Chat** - send, edit, delete, reply, forward to a channel or DM, polls with single/multiple choice, anonymous mode, timed close and live vote counts, scheduled messages with a cancel list, pin, reactions (searchable picker with categories, server emoji and clickable chips), a pinned-messages sheet with author, channel, time, jump and unpin, message grouping within seven minutes, centered date dividers, full date-and-time timestamps, typing indicators, unread/mention badges, message search with `from:`, `in:`, `mentions:`, `has:link`, `before:` and `after:` filters, a quick switcher for channels and DMs, selectable Markdown with code blocks (via `rune-kit`), Unix timestamp tags (`<t:unix:style>`) and bot message buttons with link and interaction actions, OpenGraph link previews with title, description, site and image, clickable user, channel and role mention pills (`<@id>`, `<#id>`, `<@&id>`), image attachments, channel and member context menus, and an image viewer with zoom/pan. Empty channels bottom-align the welcome intro above the composer and expose Edit Channel to managers.
- **Animated media** - animated GIF and WebP play anywhere the shared image cache draws: avatars, profile banners, server icons and banners, custom emojis, inline attachments and the image viewer. The decoder composites each frame, keeps per-frame delays, scales frames to at most 512 px and caps each animation at 120 frames / 48 MB. Animated avatars, banners and group icons skip the static crop editor and upload the original file, while static images keep the crop/zoom flow. Uploads and CDN bytes persist, so animations replay after a restart.
- **Notifications** - The server dropdown Notification Settings opens a server-level modal with Mute Server, All Messages / Only @mentions / Nothing, Suppress @everyone and @here, Suppress All Role @mentions, Suppress Highlights, Mute New Events, Mobile Push Notifications and a per-channel override list with a channel picker and remove action. Settings persist in SQLite and drive unread and mention counts; channel overrides take precedence.
- **Server events** - the sidebar Events shortcut opens a server event page with upcoming and past events, month/day blocks, start time, location channel, description and delete for the creator or managers. Create Event captures a name, description, start offset and the current channel, persists to SQLite and lists immediately for the acting client; other open clients refresh when they reopen Events because there is no gateway event yet. The API is `GET/POST /api/guilds/{id}/events` and `DELETE /api/guilds/{id}/events/{event}`.
- **Inbox** - the channel-header envelope opens a right-docked panel with Mentions and Unreads tabs. `GET /api/inbox` aggregates direct, @everyone/@here and role mentions plus unread guild and DM channels; each entry shows the server/channel breadcrumb, author, snippet, timestamp and unread count. Clicking an entry jumps to the channel, highlights the message and marks it read; each row and the header also offer mark-as-read. Mentions and unread counts come from the same read cursors and notification levels as the sidebar, and empty tabs have their own caught-up state.
- **DMs and friends** - a Friends page with a Home and All Unlocked sidebar, Online/All/Pending/Blocked tabs, live friend search, Add Friend, a friend-row message action and an Active Now rail. A group-DM sheet picks 2–9 friends, optionally names the group, and creates a real `group_dm` channel; the sidebar and header render the group icon (or a stacked avatar) and either the name or the member list, and membership scopes messages, permissions and gateway delivery. Group settings let the owner rename the group and upload or remove a cropped icon, any member add friends up to 10 people, and the owner remove members; anyone can leave, the owner transfers automatically, and live `DmUpdate`/`DmRemove` events update every open client without a restart. Friend requests use username lookup (not just people already loaded); DM conversations carry online/idle/DND/offline presence, custom statuses, and send permissions from the server.
- **Server menu and notifications** - the server dropdown opens under the header with Invite People, Server Settings, Create Channel, Notification Settings, Privacy, App Directory and Leave Server. Leaving removes the membership, roles, reads and notification overrides and switches to the next server. Every channel has persisted All Messages / Only @mentions / Nothing levels plus a Suppress @everyone and @here switch, and muted channels show a crossed-bell icon in the sidebar. These drive unread counts and desktop toasts.
- **Moderation** - right-click a member for Timeout, Kick or Ban with role-hierarchy and permission checks. Dialogs confirm each action; timeouts take a custom number of minutes, hours or days up to 28 days, persist the exact expiry, show a clock in the member list and the profile popout with the expiry date, and block messages and voice until cleared. Bans store a reason and appear in Server Settings → Bans with Unban. A banned user can submit an appeal from the Join Server flow; managers see pending appeals in Server Settings → Bans and accept or deny them, with the outcome stored and audited. Server Settings → Members → Prune Members previews inactive members by inactivity window and role protection, then prunes on confirmation. Every action writes an audit-log entry.
- **Server discovery** - Explore Public Servers lists discoverable local servers you have not joined, with member counts and descriptions, and joins one directly into the guild rail. A seeded `Rune Community` server gives the directory something to show on first run. Managers can hide or show a server in Explore from Privacy Settings.
- **Server reputation** - paid boosts are replaced by free reputation. An account older than one month can give one reputation to a server; moving it to another server waits seven days. Reputation persists and counts toward the same 2/7/14 cosmetic levels. Every perk is unlocked: 128 MB uploads, 384 kbps voice, server banners and role icons never require a level. Overview shows the banner, level badge, reputation count and progress, plus Give Reputation / Remove Reputation with the cooldown and account-age state. The server dropdown shows the same Level and x/y progress. There is no billing or Nitro.
- **Files** - drag a file onto the composer or press the plus button to choose one with the native file picker; images render inline, other files show as attachments. The cap is 128 MB everywhere, and the client rejects oversized files before upload while the server enforces the same limit.
- **UI** - the 72 px server rail with circular server icons, 232 px settings nav, a resizable channel sidebar (240 px default), 48 px channel header, a resizable member list (240 px default) grouped by hoisted role with collapsible groups, virtualized message rows with pulsing skeleton placeholders while channels, messages or members load, frosted composer and dialogs, full-window user and server settings with grouped GENERAL/MODERATION nav, scrollable content and a sticky Save/Reset footer that only activates when something changed, quick switcher, search, working emoji/reaction picker, uploaded profile avatars and banners (cropped, stored encrypted, shown in messages, member lists and profiles), profile badges and avatar decoration rings, profile views with a taller banner, overlapping avatar with presence dot, custom-status pill, badges beside the handle, a Spotify/playing activity card with artwork, a platform badge and a green Play on Spotify action, animated profile effects with orbiting highlights, gradient nameplates behind the display name, an At a glance/Now playing widget on the full profile, connected-account chips, joined dates, role-coloured chips, mutual servers, View Full Profile, and a self view that swaps Message/Add Friend for Edit Profile, inline Add Friend and a More menu (copy ID, remove friend or cancel a request, block), light/dark themes and a compact-row mode. Member lists are scoped to the selected channel, so members denied `View Channel` are omitted. Voice channels show a status subtitle (managers see Set a channel status until one is written). The voice view splits into the participant stage and an optional right-side text chat panel using the channel's own messages and composer, with circular camera, screen-share, quality, mute, deafen and leave controls, plus a Soundboard panel that broadcasts six built-in effects into the call; connected voice users appear beneath their voice channel, and a Voice Connected card with the channel, server, camera, screen-share, voice-chat and disconnect controls sits above the account panel.
- **Settings** - an **Application** tab with the backend server URL (defaults to `http://127.0.0.1:47800`, saved to `%APPDATA%\Runecord\config.json`, with Save, Use Local Default and Restart actions). **Save** probes `GET /api/version` first and refuses to write an unreachable endpoint or one that is not a Runecord server; a hand-edited invalid URL shows **Runecord can’t connect to the server** on the sign-in screen with a link back to Application. Also profile display name, bio, custom status, avatar color/border/banner, effect/nameplate/widget pickers, password change, notification toggles, theme, row density, developer mode, editable keybinds, privacy controls for new DMs and friend requests, bot/application management (create, one-time token reveal, revoke/delete, install links), preference reset and warm-cache clearing all write real state.
- **Server identity** - server icons and banners upload as PNG, JPEG, GIF or WebP. Selecting a static image opens a crop editor with drag-to-reposition, zoom from 100% to 300%, reset, arrow-key nudges and a live crop preview; Apply writes the cropped PNG. Animated GIF/WebP files bypass the crop and keep their frames. Overview shows large previews with Upload/Remove and drag-and-drop, the rail renders the icon, and servers with a banner show it at the top of the channel sidebar and in the server dropdown. Banners are free at every reputation level. Images are stored encrypted under `data/cdn/` and served from `/cdn/`.
- **Channel permissions** - a two-column editor picks a role or member and cycles every permission through Inherit, Allow and Deny, including channel-specific member overwrites.
- **Server settings** - a full-window page with Overview, Onboarding, AutoMod, Templates, Integrations, Roles, Members, Bans, Emojis and Audit Log in the left nav. Overview includes a server banner and reputation-level badge, edits the server name, icon colour, description, tag and Explore discoverability through `PATCH /api/guilds/{id}`, and updates the rail, header and settings identity immediately. Roles creates, renames, colours, reorders and deletes roles through a tabbed editor (Display, Permissions, Links, Manage Members) with a live chat preview, a snowflake role ID with Copy ID / Copy Mention actions, a role-colour hex readout, an unlocked role icon shown in role chips and member lists, drag-to-reorder rows (Up/Down buttons remain as the keyboard path), grouped permission checkboxes, persisted role links, per-member role toggles and a full-width unsaved-changes footer with Reset and Save Changes that only activates when something changed. Members assigns and removes roles and opens Prune Members (inactivity days, role inclusion, preview count, confirmation); Bans lists banned users with reason, actor, date and Unban; Emojis adds by naming and dropping a PNG; Audit Log pages through role, member-role, member moderation, prune, emoji and webhook actions with actor, target and change summary. Role changes broadcast to connected clients.
- **Threads and forums** - create a thread from a message's menu or from a channel's thread list, open it with a parent-channel context bar and message count, archive/unarchive it, and use forum cards with previews. New posts take a title and body, open as their own thread and keep the parent context. Threads inherit the parent channel's permissions.
- **Custom emoji and stickers** - guild emojis load through bootstrap, `:name:` renders inline at message size, stickers upload from Server Settings, and the composer sticker picker sends them at sticker size. The reaction picker shows a This Server section with the image entries.
- **Slash commands** - the composer `/` picker lists commands from applications installed in the guild. Selecting one fills the composer; sending posts `/api/interactions`, and the local app runtime returns a callback that appears as a bot message. The seeded Rune Tools app provides `/ping` and `/echo`.
- **Developer mode** - a live bottom bar with gateway state, guild/channel/message/member counts, guild-detail LRU usage, image cache size, snapshot age and profile id.
- **Accessibility** - controls expose roles, names and state through Rune's AccessKit tree; toasts are polite live regions; icon buttons carry labels and help text.
- **Profiles** — avatar borders, profile banners, guild/server tags.
- **Desktop integration** - Windows toast notifications for DMs and mentions while the window is unfocused, plus a tray icon with Open/Quit.
- **Voice & Video settings** - a settings tab with a DirectShow microphone picker and a native WASAPI render-endpoint picker, input/output volume, a live mic test, Voice Activity / Push to Talk modes, automatic or manual input sensitivity, and a real DirectShow camera preview. Push to Talk is a global **F8 hold** checked with `GetAsyncKeyState`, so releasing it silences the outgoing PCM. Input sensitivity gates the captured PCM before Opus encoding and volume scales it. Output-device selection runs through an in-process shared-mode WASAPI sink with automatic sample-rate and channel conversion; there is no helper executable, and FFplay remains only as a fallback if WASAPI initialization fails. Settings persist in the warm cache and are applied on the next join.
- **Voice** - joining a voice channel opens a local audio relay: microphone capture through FFmpeg DirectShow, Opus 48 kHz mono encoding, WebSocket fan-out, Opus decoding and native shared-mode WASAPI playback to the selected endpoint. Late joiners request the Ogg headers from the relay, senders resend them on demand, and the 15-second fallback refresh is ignored by existing decoders, so playback runs uninterrupted. The Opus bitrate is always the top 384 kbps tier; voice perks are unlocked. Input mute and output deafen are live in the user panel. Set `RUNECORD_FFMPEG`/`RUNECORD_FFPLAY` if the binaries are not on `PATH`.
- **Video and screen share** - the voice stage with a tile per participant. Camera capture uses FFmpeg DirectShow, screen capture uses `gdigrab`; frames are JPEG relayed over the same voice WebSocket. A quality selector offers Auto, 360p (15 fps), 480p (24 fps), 720p and 1080p (30 fps); the choice persists across restarts, and Auto steps between 360p and 720p when the outbound frame queue backs up. Every participant can publish at once: the relay fans each user's stream out to all peers, and the stage lays the tiles out as a responsive 16:9 grid (one large tile, then two-by-two, then three columns). The sender publishes two simulcast layers (the selected quality plus a 360p/15 layer); each viewer picks Auto, High or Data saver, and Auto drops to the low layer if its decoder falls behind. The sender decodes its own main layer into a local `(You)` self-preview tile, so the badge matches the capture preset instead of always reading 360p. The receiver decodes with FFmpeg at the active layer’s native resolution (854×480, 1280×720 or 1920×1080), always taking the newest frame so stale ones are dropped, and shows the active layer’s resolution and measured frame rate on each tile. Camera and screen share are mutually exclusive, stopping a share clears the remote tile, and local audio keeps flowing.
- **Bots** — backend bot accounts with `rcbot.…` tokens (`Authorization: Bot`), owned by human accounts; bot messages carry a BOT tag in the client.
- **API compatibility** — native REST/gateway plus a versioned `/api/v10` surface with rate-limit headers and 429s.
- **Keys** — `Ctrl+K` quick switcher, `Ctrl+F` search, `Ctrl+,` settings, `Esc` closes modals, `Ctrl+Enter` or the send button sends (Enter is a newline). The global shortcuts are rebindable in Settings → Keybinds and persist with the profile.

## Architecture

- **Backend** (`..\Runecord-backend`): axum + `rusqlite` (WAL), Argon2 password hashes, opaque bearer sessions, channel permission overwrites, `broadcast`-based WebSocket fan-out, and encrypted media under `data/cdn`. Persistence is a concrete `store::Database` with SQL grouped by domain in `store/sqlite/*`; `media::MediaStore` owns hashing, encryption and thumbnails. Startup and failures log through `tracing` (`RUST_LOG`, default `info`). See `docs/ARCHITECTURE.md` in the backend repository.
- **Client** (`crates/client`): one crate with a `runecord_client` library and a thin `runecord` binary. Rune views are plain Rust. Network I/O runs on worker threads - one blocking HTTP command loop and one reconnecting gateway socket - and the UI drains a shared event queue with a `Waker` plus a low-frequency `wake_after`. No network or disk work happens inside layout.
- **Source layout**: `lib.rs`/`main.rs` launch, `bootstrap.rs` wires the app; `app/` owns state and splits into feature-grouped `actions/`, `events/`, plus `session`, `media`, `retention` and `seed`; `net/` splits into `http/`, `gateway` and the shared `Command`/`NetEvent` types; `ui/` has one module per surface (`rail`, `sidebar`, `chat`, `composer`, `members`, `dms`) with `settings/`, `server_settings/` and `modals/` grouped by feature; `voice/` splits capture, devices, camera, soundboard and video helpers; `cache.rs` owns the snapshot format.
- **Server cache**: the backend keeps a 256-entry guild-detail LRU keyed by SQLite's write generation, a 32 MB decoded-thumbnail LRU, and a 512 MB on-disk thumbnail budget. No Redis: it is one local process, and writes invalidate by construction.
- **Rendering**: Rune's own wgpu renderer, text layout and springs - no Electron, no web view, no JavaScript.
- **Client cache**: a warm-start snapshot lives at `%APPDATA%\Runecord\cache-<profile>.json`. It is painted before the socket answers, then revalidated by the first bootstrap/READY. Retention is bounded: 500 messages per channel, 5,000 messages total, 8 channels per snapshot, 300 decoded images or 64 MB. Guilds are summaries in memory; guild details are LRU (6), members are paginated 100 at a time and capped at 300 per guild, and images are fetched only for the visible row range plus prefetch. Messages, members and channels use Rune's `virtual_rows`. The cache is plaintext, same trust boundary as the saved session token, and is deleted on logout/reset.
- **Data**: `..\Runecord-backend\data\runecord.db.enc`, `data/cdn/` and `data/thumbs/`. Sessions live in `%APPDATA%\Runecord\session-<profile>.txt`.

## UI parity audit

The high-visibility Server Settings → Overview surface was compared against a public
server-invite reference. The audit fixed the page title (`Server Overview`), grouped the left nav into
`GENERAL` and `MODERATION`, made the content column scroll, added field helper
text and dividers, marked the 2/7/14 boost thresholds under the progress bar, and replaced the
below-the-fold save row with a sticky footer whose `Save Changes` button stays disabled until a
field changes. Evidence: `shots/ui-overview-side-by-side.png`.

## Honest limits

Remaining gaps after the full-surface audit: there is no Nitro or billing at all; badges, avatar decorations, profile effects and names are free local presets, and server reputation replaces boosts; video has two simulcast layers rather than a full SFU; activities are not implemented; the soundboard has six built-in synthesized effects but no uploaded per-server catalog, per-sound volume or Boost-gated slots; profile effects, nameplates and widgets are local presets rather than Nitro purchases. The comparison sheets cover the guild view, member list, composer, voice split, profile, Inbox, Events, Server Settings, role editor, Friends, group DMs and user settings rather than every dialog.

- **Voice output is shared-mode WASAPI.** The client enumerates render endpoints and opens the selected friendly name directly, converting the decoded 48 kHz mono PCM to the endpoint mix format. It does not request exclusive mode or expose per-application volume, and falls back to FFplay on the default endpoint only if WASAPI initialization fails. The Voice Connected camera, screen-share and voice-chat buttons work, and the Soundboard button opens a six-effect panel whose PCM is mixed into the outgoing capture stream, so the whole channel hears it even while the microphone is muted. Activities are not offered rather than shown as a dead placeholder. Server events are real data but have no recurring events, interested counts, reminders or external integrations. Push to Talk is fixed to F8 and is not rebindable. Existing listeners ignore duplicate header pages; a late joiner asks the relay for headers instead of waiting for the fallback refresh.
- **Video is multi-party but best-effort.** Every participant publishes a main layer (up to 1080p/30) and a 360p/15 low layer; viewers choose Auto, High or Data saver, and Auto falls back to the low layer when decoding cannot keep up. There are never more than two layers, and each peer runs its own FFmpeg capture and decode subprocesses, so a larger call scales with local CPU rather than a simulated SFU. The receiver decodes at the published layer resolution through FFmpeg, so 720p and 1080p tiles carry real detail; CPU and memory scale with that resolution. Frames reuse one Pixels surface per participant. A GPU capture path would avoid the subprocess; this one shells out to FFmpeg.
- **Highlights and mobile push are stored for parity.** The switches persist, but this build has no Highlights feed and no mobile client; the user-level Notifications tab remains separate from the server modal.
- **Activity and connections are manual profile data.** The Profile tab can set a Spotify or playing card with title, subtitle, state, link and uploaded square artwork, plus up to six typed connection chips. Nothing is auto-detected from Spotify, Steam or a game process, connections are labels rather than OAuth-linked accounts, and the card has no synchronized progress bar or synced playback. Effects, nameplates and widgets are free local presets: four animated effects, four gradient nameplates and two widget variants, with no store or purchased catalog. The custom status remains a rounded pill rather than a speech bubble with a tail, and the voice chat panel opens and closes per session rather than remembering its previous state.
- **Badges and decorations are local presets.** Profile badges and the Aurora/Sakura/Flame/Smoke avatar rings are self-service free data, not linked to billing or account age. The profile effects and nameplates are local animation/gradient presets, and there are no profile decorations beyond the avatar rings.
- **Inbox groups unreads per channel.** Each unread row shows the channel's latest message, not every message, and the panel caps mentions at 20 and unread channels at 30. Jumping loads the newest 50 messages, so an older message highlights only when it falls in that page. There is no For You tab, reaction/bookmark list or followed-thread section.
- **Role links are local references, not linked accounts.** The Links tab stores up to five label/URL pairs per role and they persist and audit like the rest of the editor, but they do not feed linked-role OAuth metadata. Role icons are free, accept PNG/JPEG/GIF/WebP up to 4 MB and must be square; there is no in-app crop or rotation, and the Display preview is a two-row message mock rather than live channel content.
- **Group DMs have a single owner.** There are no moderators or role badges inside a group, ownership only transfers when the owner leaves, animated group icons play but cannot be cropped, and a group with no members is deleted with its messages.
- **Image cropping is static-only.** The editor supports drag, zoom and reset but not rotation or aspect presets, and animated files skip it entirely; icons must end up square (up to 4 MB) and banners landscape (up to 10 MB). Animations are GIF/WebP only (no APNG), are downscaled to 512 px, and cap at 120 frames / 48 MB per image with no pause control. Banners are available without a reputation level.
- **Windows uses native context menus.** Linux falls back to the in-app message menu. Pasting an image or file from the clipboard into the composer is supported alongside file drops.
- **Third-party assets are not bundled.** The look uses the app's own metrics and palette with Inter (Rune's bundled font) and redrawn SVG icons. No third-party trademarks or fonts are included.
- **The embedded server binds loopback only.** Sharing with another machine means running the standalone backend with `--host 0.0.0.0`; it has no TLS of its own, so a reverse proxy is required for anything beyond a trusted LAN. Multi-node/Postgres deployment is Phase 2/3 of the backend plan.
- **Packaging limits.** Windows builds are self-signed (SmartScreen warns until a purchased certificate is supplied), there is no auto-updater (the app only links to the release page), and the Linux binary is cross-built and structurally checked but has not been run on a Linux host in this repository.
- **No browser or mobile client.** The wasm client and its build scripts were removed; Android is a separate project. Mobile push remains a stored preference with nothing behind it.
- **Rune is pre-release.** The client path-depends on the Rune checkout at `..\rune`; Rune's `vendor/winit` and `vendor/wgpu-hal` patches are applied in this workspace `Cargo.toml`. The repository therefore does not build on its own, and there is no CI yet.
- **Storage boundary.** Handlers no longer embed SQL: queries live in `store/sqlite/*` and `MediaStore` owns encrypted file writes. `Database::conn()` is still exposed inside the crate for connection-scoped work; a storage trait will only return if a second backend is actually implemented.

## Development

Both Rust workspaces pin toolchain `1.99.1` and must pass:

```bat
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace
```

Run the same set in `..\Runecord-backend`. See `docs/ARCHITECTURE.md` for the module map, `docs/CONTRIBUTING.md` for workflow and test expectations, and `docs/DECISIONS.md` for boundary rationale.

## Build

```bat
build.bat                 :: Windows installer + Linux tarball
build.bat --desktop-only  :: skip Linux
build.bat --clean         :: wipe the build cache first
```

`build.bat` produces, in `dist\`:

- `Runecord-Setup-0.1.0.exe` — the per-user Windows installer (Inno Setup),
  signed with the local self-signed certificate. Installs under
  `%LOCALAPPDATA%\Programs\Runecord`; no admin prompt.
- `Runecord.exe` — the same client as a portable single file, with the backend
  embedded in the process: no launcher, no console window, no child process.
- `Runecord-linux.tar.gz` — the Linux build with `install.sh`,
  `uninstall.sh` and a desktop entry.
- `index.html`, `README.txt`, `LICENSE` and `assets\`.

The embedded server keeps its encrypted database, key, media, log and backups
under `%APPDATA%\Runecord\server` (`~/.local/share/runecord/server` on Linux),
so replacing the program never touches user data.

The Windows step is `scripts\build-installer.ps1` (release build → sign →
ISCC). It needs Inno Setup 6 (`ISCC.exe`) and `windres` from MinGW. The Linux
step is `scripts\build-linux.ps1` (cargo-zigbuild plus the vendored
libxkbcommon). A separate two-process build still exists for development:

```bat
cargo build --release -p runecord            :: client only, expects a server
..\Runecord-backend\run-server.bat           :: standalone backend (dev)
cargo build --release -p runecord --features embedded-server   :: single file
```

For headless UI checks the client supports Rune's tooling:

```bat
target\release\runecord.exe --snap shot.png --size 1360x860
target\release\runecord.exe --scene light --snap light.png
```

The live end-to-end run uses `RUNECORD_LIVE=1` with a Rune script (see `shots/*.json`).

Benchmarks are in [`BENCHMARKS.md`](BENCHMARKS.md).


## Phase 2 additions

- **Onboarding and rules screening** - Server Settings > Onboarding sets a welcome title and
  description, a one-rule-per-line rules list and a verification level. Guild detail carries the
  onboarding block, a member who has not accepted sees the rules sheet on open, and posting is
  rejected until they accept. Verification levels gate posting on account age (5 min, 10 min,
  1 day). API: `GET/PATCH /api/guilds/{id}/onboarding` and
  `POST /api/guilds/{id}/onboarding/accept`.
- **AutoMod** - keyword, link, mention-spam and excessive-caps rules with block, timeout or
  alert actions. Managers bypass rules; every trip writes an audit entry. API:
  `GET/PUT /api/guilds/{id}/automod`.
- **Server templates** - snapshot a server's channels and roles, get a short code, then create a
  new server from it (the Create Server dialog takes an optional template code, and Templates has
  a Use button). Applying a template replaces the default channels with the template layout.
  API: `GET/POST /api/guilds/{id}/templates`, `DELETE /api/guilds/{id}/templates/{template}`,
  `GET/POST /api/templates/{code}`.
- **Integrations** - a webhook manager listing every guild webhook with its channel, creating new
  webhooks with a one-time token and endpoint, and deleting them. API:
  `GET /api/guilds/{id}/webhooks` plus the existing per-channel webhook routes.
- **Audit filters and export** - filter the audit log by action and by moderator, then export the
  loaded page as CSV to `%APPDATA%\Runecord\exports\audit-log.csv`. API filters:
  `?actor=` and `?action=`.
- **Forum tags** - per-forum tag definitions with optional emoji, editable from the channel editor,
  shown on post cards, filterable in the toolbar, and toggled per post from the post header. The
  forum also sorts by latest activity or newest and switches between grid and list. API:
  `PUT /api/channels/{id}/forum-tags` and `PUT /api/channels/{id}/thread-tags`.
- **Recurring events, RSVP and reminders** - events take a recurrence (daily, weekly, monthly),
  a length and a reminder lead time. Recurring events roll forward to the next occurrence in
  listings, members can mark Interested or Going with a live count, and the server tick posts a
  reminder message in the event channel when the lead time elapses. API:
  `PUT /api/guilds/{id}/events/{event}/rsvp`.

## Phase 3 additions

- **DM and group calls** - the DM header call and video buttons join the DM channel on the
  same voice relay as guild channels. `POST /api/voice` accepts `guild_id: 0` for a DM or
  group-DM channel, gated on channel membership, and voice state for DM calls is delivered to
  every member of that DM. A call strip above the messages shows participants and gives mute,
  deafen, camera, screen share and leave controls.
- **Per-user volume and local mute** - right-click a member for Mute for Me or User Volume.
  Volume runs 0-200% per speaker, is applied while each speaker's decoded stream is mixed, and
  local mute silences that speaker only. Gains reset when you leave the channel.
- **Rebindable Push to Talk** - Voice & Video > Push to Talk shows the current key, captures a
  new one when you press Change, and offers Reset to F8. The binding persists with the rest of
  the voice settings.
- **Noise suppression** - an optional suppressor tracks the room's noise floor and soft-expands
  quiet tails, removing steady fan and hum before the frame is encoded.

### Stage channels and screen-share audio

- **Stage channels** - the channel type picker creates a `stage` channel. Joining puts you in the
  audience; Raise Hand asks to speak, and the stage bar shows speakers, raised hands and the
  audience count. Members with Mute Members or Manage Channels can invite a raised hand to speak
  or move a speaker back to the audience. API: `GET/PUT /api/channels/{id}/stage`, plus a
  `STAGE_INSTANCE_UPDATE` gateway event for compatible clients.
- **Screen-share audio** - while screen sharing, a Share Audio control starts a WASAPI loopback
  capture on the default output endpoint, resamples it to 48 kHz mono and mixes it into the
  outgoing Opus stream through the same queue the soundboard uses, so it keeps working while the
  microphone is muted. The control reports whether loopback is live or unavailable.

## Phase 4 additions

- **Verified connections** - Profile > Connections lists GitHub, Steam, Twitch, YouTube, Spotify
  and X. Request a verification code, paste it into that service's public profile, then verify:
  the server fetches the profile URL with a 10 second timeout and only stores the connection when
  the code is present, marked verified with the URL kept. API:
  `POST /api/connections/challenge` and `POST /api/connections/verify`. Codes expire after 30
  minutes and are cleared once used.
- **Automatic Rich Presence** - Profile has an Automatic Rich Presence toggle. Every 15 seconds the
  client scans running processes against a small catalog (Rune, Counter-Strike 2, VALORANT,
  Minecraft, Factorio, Hades, Stardew Valley, Visual Studio Code, Spotify) and publishes the match
  as the activity. A hand-written activity always wins, and clearing the game clears the activity.

### Activities and the game overlay

- **Embedded activities** - every guild gets four built-in activities (Watch Together, Chess in the
  Park, Sketch Heads, Word Snacks). A voice channel's Activities button lists them; starting one
  stores a session, broadcasts it to the channel and shows a card with Join (opens the activity)
  and Stop (host or Manage Channels). API: `GET /api/guilds/{id}/activities`,
  `POST /api/channels/{id}/activities/{activity}`, `DELETE /api/channels/{id}/activities`, plus an
  `ACTIVITY_SESSION_UPDATE` gateway event.
- **Game overlay** - a Rune overlay window sits top-right with the channel name and up to four
  participants, showing who is speaking and who is muted. It is non-activating and click-through,
  follows the voice state, and has a toggle plus a live status line in Application settings.

### Localization and accessibility

- **Localization** - every navigation string goes through `i18n::t`, which falls back to English
  and then to the key. English, German, Spanish and French catalogues cover the 31 navigation and
  settings strings, the picker lives in Appearance, and the choice persists to
  `%APPDATA%\Runecord\locale.txt`.
- **Accessibility audit** - Rune ships AccessKit (UI Automation, NSAccessibility, AT-SPI), so the
  audit ran against the real accessibility tree with `--tree`. Before the pass the guild scene had
  41 unnamed interactive elements (39 message-action buttons plus both resize handles) and the DM
  scene had one; every audited scene now reports zero. The audit script is
  `shots/a11y-audit.ps1` and the dumps are `shots/a11y-*.json`.

### GIF search (no API key) and Postgres interop

- **Built-in GIF catalogue** - the server renders twelve animated GIFs itself on first start
  (pulse, bounce, wave, spin, sparkle and bar patterns in the app palette) and records them in
  a `gifs` table. `GET /api/gifs?q=` searches titles and tags, and
  `POST /api/channels/{id}/gifs` posts one as a real `image/gif` attachment. The composer's GIF
  button opens a picker with search. This replaces the Tenor/Giphy dependency, so GIF search works
  offline and needs no credentials.
- **Postgres schema and dump** - `runecord-server --export-postgres out.sql` writes a Postgres
  schema translated from the same migration list SQLite applies (INTEGER to BIGINT, REAL to DOUBLE
  PRECISION, BLOB to BYTEA, SQLite-only PRAGMA/COLLATE dropped) followed by `INSERT` statements for
  every row, ready to load with `psql`. The translator is unit-tested so the two schemas cannot
  drift.

### One command dispatcher

`net/http/mod.rs` exposes `dispatch(&Arc<Shared>, Command)`, used by the desktop worker thread.
Handlers are written once, in terms of three helpers:

- `get_then(shared, path, then)` - GET and decode.
- `send_then_json(shared, method, path, body, then)` - any verb with a JSON body.
- `fire_then(shared, method, path, body, then)` - any verb, response ignored.

### Uploads and the Postgres driver

- **One upload path** - `net::UploadSource` carries either a `Path` from the native dialog or
  `Bytes { name, bytes }` for a pasted image or a cropped avatar. `upload_then` reads the source
  and POSTs the raw body, so every upload handler is written once.
- **Cropping without a temp file** - the avatar/banner/icon crop modal encodes the cropped pixels
  as PNG in memory and uploads `UploadSource::Bytes`.
- **Native picker** - `files::open` shows Rune's dialog; picked paths are queued and routed in
  `App::poll` through `PickKind` (avatar, banner, guild icon/banner, activity artwork, role icon,
  emoji, DM icon, channel upload).
- **Live Postgres** - `--features postgres` adds a `tokio-postgres` driver:
  `runecord-server --migrate-postgres URL` applies the translated schema, and
  `--push-postgres URL` migrates then loads the local dump inside one transaction. The schema is
  the same text SQLite applies, so the two cannot drift. A UTF-8 BOM in a migration file is
  stripped before the statement is sent.
