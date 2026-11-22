# Architecture

Runecord is two native processes that talk HTTP and WebSocket:

```text
┌──────────────────────────────┐        HTTP /api/*         ┌──────────────────────────────┐
│ Rune client (runecord)       │ ─────────────────────────▶ │ Server (runecord-server)     │
│ UI + app state + audio       │ ◀───────────────────────── │ axum routes + gateway        │
│                              │      WebSocket /gateway     │ SQLite + encrypted uploads   │
└──────────────────────────────┘                             └──────────────────────────────┘
             │                                                            │
             └────────── runecord-protocol (shared wire types) ───────────┘
```

The client never touches SQLite; the server never renders UI. All shared
payloads live in `runecord-protocol`, which both repositories consume (the
client by path from `..\Runecord-backend\crates\protocol`).

## Workspaces

| Path | Role |
|---|---|
| `crates/client` | The Rune desktop application (`runecord` binary, `runecord_client` library) |
| `tools/cache-bench` | Snapshot cache benchmark; links the client library |
| `..\Runecord-backend\crates\protocol` | Wire and domain types |
| `..\Runecord-backend\crates\server` | HTTP API, gateway, storage |
| `..\rune` | Rune UI framework (path dependency) |

### Packaging mode

The default build is two processes: start `runecord-server`, then the client.
The opt-in `embedded-server` feature links the server crate and starts it on a
loopback thread inside the client process, so `dist\Runecord.exe` is a single
double-clickable application. Embedded mode only activates when the configured
URL is a literal loopback address without a running Runecord server; remote
URLs and `localhost` keep the normal two-process behaviour. See
`docs/DECISIONS.md` for the trade-off.

## Client layers

The client is one crate with a thin binary. `src/main.rs` calls
`runecord_client::run()`; `src/lib.rs` declares the modules and `bootstrap.rs`
owns process setup (window spec, scenes, waker, initial state).

```text
crates/client/src
  lib.rs, main.rs          launcher only
  bootstrap.rs             Rune app spec, scenes, startup
  config.rs                per-profile config and backend URL
  cache.rs                 warm-start snapshot cache (background writer)
  theme.rs, icons.rs       design tokens and SVG glyphs
  image_anim.rs            animated image decoding
  wasapi.rs                Windows audio output
  app/                     application state and orchestration
    mod.rs                 App struct, Screen/Modal/SettingsTab enums
    session.rs             login/session lifecycle
    seed.rs                demo/local state seeding
    media.rs               image request bookkeeping
    retention.rs           cache trimming
    actions/               user intents, one module per feature
    events/                NetEvent application, one module per feature
  net/                     transport, no UI access
    mod.rs                 Command, NetEvent, Net, Shared, worker threads
    http/                  blocking request handlers per feature
    gateway.rs             reconnecting WebSocket
  ui/                      render functions over &App
    mod.rs                 layout shell and shared widgets
    chat.rs, dms.rs, ...   screens
    settings/              user settings tabs
    server_settings/       guild settings views
    modals/                dialog sheets
  aiedit/                  AI UI editor: brief, allowlist, build, stage, supervisor
  voice/                   capture, playback, camera and video runtime
    mod.rs                 config/state statics and the voice loop
    mic.rs, camera.rs, devices.rs, soundboard.rs, video.rs
```

### Dependency direction

```text
bootstrap → app → net
     │        │
     └──────▶ ui ──▶ theme, icons, common widgets
app ──▶ voice (commands and state), cache
app ⇄ aiedit (UI self-edit; worker threads + events)
net ──▶ runecord-protocol
```

`ui` may read `&App` but never sends commands directly: widgets emit
`App` methods, and those methods create `Command`s. `net` knows nothing about
`App` or Rune scenes. `voice` exposes plain functions and shared state.

### Data flow

1. A widget callback calls an `App` action (`app/actions/*`).
2. The action mutates local state and/or pushes a `Command` onto the worker
   channel (`App::send`).
3. The HTTP worker (`net/http`) performs the blocking request and pushes a
   `NetEvent` into the global queue.
4. The gateway thread (`net/gateway`) pushes `NetEvent`s from server pushes.
5. `App::poll` drains events and dispatches to `App::apply`, which routes to
   `app/events/*` handlers.
6. Handlers update state and mark the cache dirty; the Rune runtime repaints.

The UI thread never blocks on network or disk. HTTP work happens on a worker
thread, cache writes on their own thread, and voice capture/playback on their
own threads.

### Self-editing UI

`aiedit/` is the only code path that writes UI source at runtime. A request is
expanded into a brief, then an orchestrator model plans, requests read-only
context and returns whole files. `patch.rs` rejects the reply unless every path
is under `crates/client/src/ui/` or `crates/ui/src/`; `build.rs` runs
`cargo build --release` as a child process; `install.rs` stages the binary and
supervises the restart. `net/`, `config.rs`, `paths.rs` and the session token
type are outside the writable tree, so an edit cannot reach them. Progress
crosses to the UI thread through an event queue; the UI thread never builds or
blocks. The installer bundles the build source under `<install>\src`; the user
supplies Rust. Without Cargo the card disables Generate, and each build records
the app version so the card can warn when an older UI is running against a
newer app.

### Persistent state

- Server: encrypted SQLite database plus encrypted CDN and thumbnail files
  under the server `data/` directory.
- Client: a warm-start snapshot in `%APPDATA%\Runecord` (`cache.rs`) holding
  bootstrap data, the last guild/channel, recent messages, and display
  preferences. It is disposable: the server remains the source of truth.

## Server layers

The server separates process wiring, transport, and persistence:

```text
crates/server/src
  lib.rs, main.rs          launcher and log setup
  bootstrap.rs             config → storage → state → routers → listener
  config.rs                CLI parsing (typed errors, unit tested)
  state.rs                 AppState: concrete services shared by handlers
  api/                     primary HTTP/WebSocket surface, one file per feature
  compat.rs                compatibility /api/v1 surface
  platform.rs              applications, webhooks, interactions, voice signaling
  gateway_v10.rs           versioned gateway sessions and event pump
  media.rs                 MediaStore: hashing, encryption, CDN and thumbs
  cache.rs                 bounded in-memory and on-disk caches
  crypto.rs                key loading, seal/open
  ratelimit.rs             token-bucket middleware
  store/
    mod.rs                 Database (connection, flush, durability)
    sqlite/                SQL by domain: guilds, messages, roles, …
```

Handlers translate HTTP into store calls and gateway events. SQL lives in
`store/sqlite/*`; a handler that needs new persistence adds a store function
instead of embedding a query. `MediaStore` owns the encrypt-on-write path so
uploads and generated images cannot drift apart.

### Request path

```text
request → ratelimit middleware → api/compat/platform router
        → auth helper → store call(s) → JSON response
        → optional broadcast::Sender<GatewayEvent>
```

The gateway pumps broadcast events to subscribed sockets, filtered per user
by shared-guild and relationship checks.

## Where to add things

| Change | Client | Server |
|---|---|---|
| New API call | `net/mod.rs` command variant, `net/http/<feature>.rs`, `app/actions/*` | `api/<feature>.rs`, route registration, `store/sqlite/<feature>.rs` |
| New server push | `net/mod.rs` event variant, `app/events/<feature>.rs` | broadcast `GatewayEvent` from the handler or gateway |
| New screen/dialog | `ui/*` render module, `Modal`/`Screen` variant | — |
| New settings tab | `ui/settings/*` and `SettingsTab` | — |
| New stored field | protocol type, cache snapshot | migration, row mapper, store function |

See `docs/DECISIONS.md` for the reasoning behind these boundaries and
`docs/CONTRIBUTING.md` for the verification commands.
