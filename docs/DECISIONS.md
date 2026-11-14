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
