# Decisions

Short records of boundaries that are easy to undo by accident. Each entry
states the context, the decision, and what follows from it.

## 1. Keep `runecord-protocol` as a separate crate

**Context.** The client and server shared one crate before the split; wire
types and server-only logic were entangled, and the client depended on the
server binary crate.

**Decision.** `runecord-protocol` is a standalone crate owned by the backend
repository and consumed by the client through a sibling path dependency
(`..\Runecord-backend\crates\protocol`).

**Consequences.** Both processes compile against one definition of every
payload, and the client does not pull in axum, rusqlite or server code. The
cost is that the two repositories must be checked out side by side; this is
already how `run.bat` and the workspace manifests work.

## 2. Two server crates, no premature layer crates

**Context.** The server needed clearer internals, but splitting into
`domain`/`application`/`infrastructure` crates would have produced a large
dependency graph for a single-process local service.

**Decision.** Keep `runecord-protocol` plus `runecord-server`. Inside the
server, separate modules (`bootstrap`, `config`, `state`, `api`, `compat`,
`platform`, `store`, `media`) rather than crates.

**Consequences.** Refactoring stays cheap and compile times stay short. If a
second transport or a second storage backend appears, the store and media
boundaries already exist and can be promoted to crates then.

## 3. One client crate with a library and a thin binary

**Context.** The client was a binary-only crate, so the cache benchmark had to
`#[path]`-include source files, and nothing outside `main.rs` could be reused
or tested as a library.

**Decision.** `crates/client` exposes `runecord_client` (`src/lib.rs`) and a
`runecord` binary whose `main` calls `runecord_client::run()`.

**Consequences.** `tools/cache-bench` depends on the library normally, tests
run against the library target, and startup wiring lives in
`src/bootstrap.rs` instead of `main`. The binary stays a one-line launcher.

## 4. A concrete `Database`, not a storage trait

**Context.** An unused `Store` trait promised a future Postgres adapter but
had no second implementation and no callers outside a thin shim.

**Decision.** Remove the trait. `store::Database` is a concrete type that owns
the connection, key, dirty flag and flush loop. Domain-grouped functions in
`store/sqlite/*` take `&Connection` and contain the SQL.

**Consequences.** There is one obvious way to read or write data, and the
compiler enforces the real API instead of a speculative one. The boundary is
still explicit: handlers call store functions, and `Database::conn()` is only
used for connection-scoped work. Introduce a trait when a second backend
actually exists.

## 5. SQL belongs to the store, not to handlers

**Context.** API handlers embedded SQL directly, duplicating queries and
mixing permission checks with row mapping.

**Decision.** Move persistence into `store/sqlite/<domain>.rs`: channels,
messages, guilds, roles, members, invites, events, inbox, social, media,
users, platform and so on. Handlers keep HTTP concerns, permission policy and
gateway broadcasts.

**Consequences.** Query changes happen in one place, and the store suite can
exercise behaviour without an HTTP server. Handlers that need a new query add
a named store function rather than interpolating SQL.

## 6. `MediaStore` owns encrypted media

**Context.** Uploads, emoji, role icons, guild banners, avatars and activity
artwork each reimplemented hashing, extension handling, directory creation and
encryption.

**Decision.** `media::MediaStore` centralises `save`, `save_with_ext`, `read`,
`read_thumb` and `write_thumb`, using the database key. Handlers pass bytes and
get back a `StoredAsset`/URL.

**Consequences.** The encrypt-on-write invariant cannot drift between
features, and thumbnail paths and CDN URLs are consistent. Callers that need
extra metadata (for example `record_cdn`) still write it explicitly.

## 7. Feature modules instead of god files

**Context.** Several files concentrated unrelated behaviour: HTTP dispatch,
event application, app actions, settings UI and voice runtime.

**Decision.** Split by feature, consistent across layers:

- `app/actions/*` and `app/events/*`
- `net/http/*`
- `ui/settings/*`, `ui/server_settings/*`, `ui/modals/*`
- `voice/*`

`mod.rs` keeps shared types and dispatch; feature modules hold bodies.

**Consequences.** A change to messages no longer touches a 2,000-line file
shared with roles and moderation. Dispatch stays in one readable place, so the
set of commands and events remains easy to audit.

## 8. The UI thread never blocks

**Context.** Rune renders on the main thread; synchronous HTTP would freeze
frames.

**Decision.** Keep blocking HTTP on a worker thread, the gateway on its own
reconnecting thread, cache serialization on a writer thread, and voice
capture/playback on their own threads. The UI communicates through `Command`s
and drains `NetEvent`s in `App::poll`.

**Consequences.** Rendering stays responsive, and network code has no UI
dependencies. Ordering guarantees are explicit: events are applied in the
poll loop, not from worker threads.

## 9. The cache is disposable

**Context.** Starting cold means a blank window until the bootstrap request
returns.

**Decision.** Persist a snapshot (`cache::Snapshot`) of bootstrap data, the
active guild/channel, recent messages and display settings. Treat it purely as
a warm start; never trust it over the server.

**Consequences.** Startup paints immediately and offline restarts are usable.
Snapshot shape changes require a `CACHE_VERSION` bump, and stale guilds are
revalidated rather than assumed correct.

## 10. Typed errors where they carry information

**Context.** Configuration parsing returned `String` errors, while HTTP
handlers already had `ApiError` with a status code.

**Decision.** Add `thiserror` for `config::ConfigError`; keep `ApiError` for
the HTTP boundary. Do not force every layer into one global error enum.

**Consequences.** Callers can match on configuration failures and messages
stay identical. HTTP errors keep their status semantics; store functions keep
`rusqlite::Error` so callers decide how to translate it.

## 11. Structured logging at the process edge

**Context.** Startup and fatal errors used `println!`/`eprintln!` with no
level control.

**Decision.** Add `tracing` and `tracing-subscriber`. `lib.rs` installs one
subscriber; `RUST_LOG` selects the filter (default `info`). Bootstrap and
runtime failures log at `error`, the listening line at `info`.

**Consequences.** Operators can raise verbosity without rebuilding, and logs
carry module targets. Request-level tracing can be added incrementally without
changing call sites.

## 12. Clippy is part of the build contract

**Context.** Warnings accumulated until `-D warnings` was unusable, which hid
new problems.

**Decision.** Both workspaces must pass
`cargo clippy --workspace --all-targets --all-features -- -D warnings`.

**Consequences.** Fix lints rather than ignore them. Intentional dead code
(reserved modal variants, the icon catalogue, protocol fields kept for
compatibility) carries a targeted `#[allow(dead_code)]` with a comment. The
few data-mapping functions that legitimately take many arguments carry
`#[allow(clippy::too_many_arguments)]` instead of inventing one-off structs.

## 13. Box large event payloads

**Context.** `NetEvent` and `GatewayEvent` travel through channels and queues;
unboxed guild and message payloads made every variant large.

**Decision.** Box large variants (`Box<Message>`, `Box<Bootstrap>`,
`Box<GuildSummary>`, …). Handlers may take the boxed value and unbox it
locally; do not add boxing to values that live only inside one function.

**Consequences.** Channel traffic and match arms stay small. Unboxing at the
handler boundary keeps the rest of the code ergonomic.

## 14. Do not change behaviour while moving code

**Context.** The architecture work touched every large module.

**Decision.** Refactors preserve routes, payloads, schema, file formats and
user-visible behaviour. Fixes that would change behaviour are separate,
reviewable commits.

**Consequences.** Moves can be verified by `cargo test` plus diff review, and
regressions are attributable to a single behavioural change. Tests were kept
and expanded, never removed or relaxed.

## 15. Single-executable packaging is an opt-in feature

**Context.** The client and server are separate processes by design, but a
downloadable desktop app should open from one double-click without a launcher
or a visible console. Shipping a wrapper executable that spawns
`runecord-server.exe` was rejected: it is still a launcher, and a stray console
or orphaned child process is easy to hit.

**Decision.** Add the `embedded-server` feature. It links `runecord-server`
and starts it on a loopback thread inside the client process; the app uses the
same HTTP and WebSocket endpoints as before, just against a local server it
owns. The feature is off by default; `build.bat` enables it and stages
`dist\Runecord.exe`.

**Consequences.** One process, one window, no launcher. The binary is larger
and the packaging build pulls axum, rusqlite and image into the client graph,
so development builds and the default two-process release stay slim. Embedded
mode is limited to literal loopback URLs; remote servers still require an
external deployment, and a missing/invalid key falls back to the existing
startup errors. The server writes to `%APPDATA%\Runecord\server` instead of the
repository's `data/`.

## 16. Free reputation replaces paid boosts, and ids are snowflakes

**Context.** The product has no paid features, but the schema and UI carried
the familiar boost levels, 48-128 MB upload gates and role-icon/banner tiers from
the original clone. Separately, every entity used a small row counter, so ids
could collide across restarts and could not carry mention syntax.

**Decision.** Migration `0027_reputation` adds `guild_reputation` and
`reputation_cooldowns` and copies existing boosts across. An account older than
30 days may assign one reputation; moving it waits seven days. Reputation keeps
the familiar 2/7/14 level display, but every perk is unlocked: 128 MB uploads,
384 kbps voice, server banners and role icons. `next_id` now emits
snowflake ids (`(ms since 2015) << 22 | sequence`), so new users, guilds, channels,
roles and messages share one time-ordered id space. Existing demo rows keep
their historical ids.

**Consequences.** No billing or entitlement code is needed. Older `/boost`
routes stay as aliases for `/reputation`. Clients can render and resolve
`<@id>` and `<#id>` mentions, and their unread mention queries already match the
same snowflake ids.

## 17. Forwards are snapshots, not live links

**Context.** Forwarding copies a message into another channel while still
showing where it came from.

**Decision.** A forward is a new message row whose content and attachments are
copied from the source, plus a nullable `forwarded_from` pointer used only for
the "Forwarded from #channel by author" header. The source can be deleted
without breaking the forward. The client offers a channel/DM picker from the
message menu; the server enforces `View Channel` on the source and
`Send Messages` on the target.

**Consequences.** Forwarded attachments are duplicated, so deleting the
original media does not remove the forward's copy. Editing the original does
not update existing forwards, matching a snapshot model.

## 18. Polls are message attachments

**Context.** Polls belong to a message and update in place as votes arrive.

**Decision.** Store poll metadata, options and votes in `polls`,
`poll_options` and `poll_votes`, keyed by message id. `Message.poll` is
assembled by the message mapper, so every existing message path (list, search,
reply, pin, gateway) carries polls without a second endpoint shape. Votes
broadcast `MessageUpdate`. Single-choice voting replaces the voter's previous
option; multiple-choice toggles each option. Expiry is checked on read and on
vote.

**Consequences.** Message and channel deletion remove the poll, its options and
its votes with the message rows.

## 19. Scheduled messages are server-side timers

**Context.** A scheduled message must arrive even if the author closes the
client.

**Decision.** Store scheduled messages in `scheduled_messages`; the existing
two-second server maintenance loop delivers due rows through the normal
`create_message` path and broadcasts `MessageCreate`. The client only creates,
lists and cancels schedules. Delivery rechecks `Send Messages` at send time, so
a user who loses permission cannot post through a stale schedule.

**Consequences.** Delivery is within roughly two seconds of the chosen time.
The client must stay connected to see the list update, but the server owns the
timer and the message still lands when no client is open.

## 20. Link previews are server-fetched and bounded

**Context.** Rich previews need remote HTML metadata, but the server is
loopback-only and must not become an internal-network proxy.

**Decision.** On send, the server looks for the first public `http(s)` URL,
fetches at most 256 KB with a four-second timeout, and extracts OpenGraph or
Twitter card metadata plus `<title>`. Localhost, loopback, private, link-local
and `172.16/12` hosts are rejected. Embeds are stored with the message and
loaded by the normal message mapper. Image embeds are cached by the client
through the existing image pipeline.

**Consequences.** A slow or hostile page cannot hold the send request beyond
the timeout. Redirects to private hosts are not revalidated by the simple
resolver, so a hardened deployment should add redirect-host checks before
exposing the server beyond a trusted network.

## 21. Message components are a thin bot surface

**Context.** Buttons belong to bot/app messages and either open a URL or
send an interaction.

**Decision.** Store components in `message_components`, load them through the
message mapper, and render them under the message body. Link buttons open the
URL; interaction buttons POST to
`/api/messages/{message}/components/{custom_id}`, which broadcasts
`InteractionCreate` for bots. Only bot-authored messages may create components,
through `/api/channels/{id}/components`.

**Consequences.** The client has no component editor; bots own the component
payload. Component rows are removed with their message or channel.

## 22. The AI UI editor writes through an allowlist and stages binaries

**Context.** "Edit your own UI" lets a model rewrite the view layer and rebuild
the app. A prompt asking it not to touch logic is not a boundary, and a failed
build must not leave an install that cannot start.

**Decision.** Generated patches go through `crates/client/src/aiedit/patch.rs`,
which rejects the whole reply unless every path is under
`crates/client/src/ui/` or `crates/ui/src/`; new `.rs` files in those trees are
allowed, and the reply may `NEED` (read, never write) `app/mod.rs`,
`app/actions/**`, `app/events/**` and `cache.rs` for state context. Files are
backed up per edit. Builds run as a child process into the release profile. A
finished build is copied to `%APPDATA%\Runecord\aiedit\versions\ui-<stamp>` and
its full writable tree is recorded in `aiedit/source` as content-addressed
objects plus a manifest; the running binary is never overwritten. A restart
starts the staged build through a supervisor copy of the executable, which waits
for that build's ready marker (`App::poll` writes it) and reopens the previous
binary if it never appears. The installed app also acts as the launcher: it
reads `state.json`, starts the current version, and falls back to last-good or
its own built-in UI. A development checkout skips that dispatch so `run.bat`
runs the source it just built.

**Consequences.** "UI only" is enforced by the writer rather than by the system
prompt, and no edit can reach `net/`, `config.rs` or the session tokens. New
widgets, icons and whole modules are still expressible: the editor adds the file
and the parent that registers it. A failed build restores the previous files
automatically, so the tree still compiles. The cost is that an edit rebuilds the
whole `runecord` crate (1-3 minutes warm): the `crates/ui` crate exists for
tokens today, and moving screens into it is what will shorten that build, not
the allowlist. The key lives in `aiedit\.env`: a DPAPI-sealed blob on Windows,
bound to the signed-in account, and a 0600 file elsewhere. The app never writes
the key to a JSON file; saving verifies the file reads back before the card
reports success.

Requests are expanded before patching: `aiedit/brief.rs` turns the goal and any
reference images into a structured brief, and the orchestrator receives both. A
brief is advisory — the user's words stay authoritative — and a provider that
cannot write one leaves the original request in place. Images are optional,
downscaled once in `attach.rs`, capped at four, and retried text-only if the
provider rejects image parts.

The model runs as an orchestrator rather than one prompt: it briefs, plans,
requests the files it needs, patches, and gets compiler errors fed back for up to
three repairs. The caps live in `aiedit/mod.rs` (`MAX_STEPS`,
`MAX_FILE_ROUNDS`, `MAX_REPAIRS`, `MAX_IDLE`) so a looping model cannot spin
forever or exhaust the user's budget.

The card is explicit about availability and age. The installer bundles the
source tree, and the card disables Generate when Cargo is not on PATH. A
portable exe without a `src` folder shows a No source notice, so it cannot offer
a button that always fails. A current build whose recorded `app_version` differs
from the running app shows Keep / Rebuild / Reset; keeping silences it only for
that app version. The version list renders every source-backed version instead
of a fixed four, so a long history stays reachable. The prompt area discloses
that requests, reference images and the UI files the editor reads go to the
configured provider, that the key is billed there, and that an edit is capped at
12 model calls and 3 repairs.

Each staged build records the goal, its files and the source manifest, so the
card can list versions and switch between them; switching restores that build's
source first, keeping the tree and the running UI in step. Only the installed
app, `current`, `last_good` and a staged `pending` build stay on disk as
executables: a promoted build turns the old current into last-good and deletes
the old last-good. Older
versions keep their source manifest and show **Rebuild** instead of **Use**;
rebuilding takes the same 1-3 minute build but frees roughly 93 MB per retired
executable. Source snapshots are about 1 MB for the whole UI, and unchanged files
are stored once. The binaries a restart can use (`current`, `last_good`,
`pending`) are never pruned by the startup pass; `base` is used only when the
installed app path is unknown.

