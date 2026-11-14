# Contributing

This repository is the Rune client. It depends on the sibling checkout
`..\Runecord-backend` for the shared protocol crate and the server, and on
`..\rune` for the UI framework. Keep all three directories as siblings, or use
a directory junction on Windows.

## Toolchain

Both Rust workspaces pin `1.99.1` in `rust-toolchain.toml` and request
`rustfmt` and `clippy`. Install it once:

```bat
rustup toolchain install 1.99.1 --component rustfmt clippy
```

`rustup` switches automatically inside either repository.

## Build and run

```bat
run.bat             :: build backend + client, start both
run.bat 2           :: second profile
build.bat           :: release build of the client
cargo run -p runecord
```

The client reads the backend URL from `%APPDATA%\Runecord\config.json`;
`RUNECORD_URL` overrides it. Start the backend separately when iterating:

```bat
cd ..\Runecord-backend
cargo run -p runecord-server -- --data ..\devdata
```

## Verify before committing

Run the same checks in both repositories:

```bat
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace
```

The client suite covers cache round-trips, config parsing and protocol
helpers. The server suite (`crates/server/src/store/sqlite/tests.rs`) exercises
storage behaviour against in-memory databases: permissions, threads,
moderation, notifications, discovery, privacy and profile data. Add a store
test when you change SQL; add a unit test when you change parsing or pure
logic. Do not delete or weaken existing tests to make a change pass.

## Style

- Run `cargo fmt` rather than hand-formatting.
- Keep code warning-free; `-D warnings` is the bar, so fix lints instead of
  suppressing them. If an item is intentionally unused (a reserved modal, a
  design token, a protocol field kept for compatibility), add a short,
  specific `#[allow(...)]` with a comment explaining why.
- Prefer explicit types and small functions. A module should have one reason
  to change; split by feature, not by file size alone.
- UI functions take `&App` and return `El`. They must not perform I/O or send
  commands directly; route intents through `App` actions.
- Network modules must not reference UI or `App` types.
- Preserve payloads, routes, database schema and on-disk formats. Changing any
  of them needs a deliberate migration and a note in `docs/DECISIONS.md`.

## Adding a feature

1. Add or reuse protocol types in `..\Runecord-backend\crates\protocol\src`.
2. Server: add the route in `crates/server/src/api/<feature>.rs` (or
   `compat.rs`/`platform.rs`), put SQL in `store/sqlite/<feature>.rs`, and
   broadcast a gateway event if other clients must update live.
3. Client transport: add a `Command` variant, a handler in
   `net/http/<feature>.rs`, and a `NetEvent` if the result is asynchronous.
4. Client state: add the action in `app/actions/<feature>.rs` and the event
   handler in `app/events/<feature>.rs`.
5. UI: render from `ui/`; keep new dialogs in `ui/modals/` and settings in
   `ui/settings/` or `ui/server_settings/`.
6. Cache only what is needed for a warm start; bump `CACHE_VERSION` when the
   snapshot shape changes incompatibly.

## Commits

Write one focused commit per logical change with an imperative subject
("Split the HTTP command worker into feature handlers"). Include the
verification you ran in the body when it is not the standard set. Refactor
commits should not mix in behaviour changes; behaviour changes should not hide
inside moves.

## Documentation

- `docs/ARCHITECTURE.md` — module map and data flow.
- `docs/DECISIONS.md` — why the boundaries are where they are.
- `docs/CONTRIBUTING.md` — this file.
- `BENCHMARKS.md` — cache benchmark methodology.

`docs/` also hosts the published marketing site; Markdown files there are
project documentation and are not linked from the site.
