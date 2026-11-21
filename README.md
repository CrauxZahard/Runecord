# Runecord

Runecord is a self-hosted desktop chat app. It uses Rune, a Rust UI framework. A separate Rust service runs the backend.

## Requirements

- Rune checkout at `..\rune`
- Backend checkout at `..\Runecord-backend`
- Rust toolchain 1.99.1
- Windows 10+ or Linux x86_64

Both sibling checkouts must exist. The manifests use relative paths only.

## Layout

```
Runecord\             Client repo. Holds crates\client, assets\, packaging\, and docs\.
..\Runecord-backend\  Backend repo. Holds crates\protocol and crates\server.
..\rune\              UI framework.
```

## Run

Run `run.bat`. The build embeds the backend and opens `127.0.0.1:47800`.

Debug builds seed demo accounts. Use password `runecord`. Release builds seed no accounts. The first account owns its server.

Run `run.bat 1`, `run.bat 2`, and so on. Each window then holds one account.

## Downloads

Prebuilt files are on [GitHub Releases](https://github.com/CrauxZahard/Runecord/releases).

To build locally, run `build.bat`. It writes these files to `dist\`:

| File | Use |
|---|---|
| `Runecord-Setup-0.1.0.exe` | Windows installer. No admin rights needed. |
| `Runecord.exe` | Portable Windows app. |
| `Runecord-linux.tar.gz` | Linux app. Run `./install.sh`. |

The Windows files use a self-signed certificate. SmartScreen may warn.

## Features

- Servers, channels, categories, roles, permissions, and invites.
- Messages, replies, polls, reactions, threads, search, and Markdown.
- Voice calls with Opus, WASAPI playback, push to talk, and screen share.
- Direct messages, group DMs, friends, and presence.
- Profiles with avatar, banner, badges, nameplate, and activity.
- Moderation: timeout, kick, ban, appeals, prune, and audit log.
- Events, AutoMod, onboarding, templates, and integrations.
- Inbox, notifications, and accessibility through AccessKit.
- Custom UI: describe a layout or theme change and the client rebuilds its own
  view layer.

## Custom UI

Settings > Appearance > Edit your own UI takes a plain-language request and
patches the view layer (`crates/client/src/ui`, `crates/ui/src`) before
rebuilding the client. Bring an OpenAI-compatible base URL, model and key; on
Windows the key is DPAPI-sealed in `aiedit\.env`, and elsewhere the file is 0600.

The editor is a preview. It needs the UI source tree and Rust toolchain; a
packaged install without the source bundle shows a disabled card instead of a
button that cannot work. Requests, reference images and the UI files it reads
go to the configured provider, and that provider bills the key. One edit is
capped at 12 model calls and 3 repairs.

The app builds beside the running binary and keeps the current and last-good
executables plus a source snapshot per version. A restart supervisor waits for
the new build's ready marker and reopens the previous build if it never appears.
See `docs/DECISIONS.md` (section 22) and `docs/plans/ai-ui-editor.md`.

## Limits

- No billing or Nitro.
- Video uses two layers. It does not use an SFU.
- Activities and connections are typed profile data. No OAuth.
- The Linux build is cross-compiled. It has not run on a Linux host here.
- Rune is pre-release. This repo does not build without `..\rune`.

## Settings and data

- Set the backend URL in Settings > Application. The client checks `/api/version` first.
- Data lives in `%APPDATA%\Runecord` on Windows. On Linux it lives in `~/.local/share/runecord`.
- Back up `server\key.bin` with the database. Backups run every 24 hours. The newest 7 remain.
- Custom UI builds and provider settings live in `%APPDATA%\Runecord\aiedit`.
- Reset a password with `Runecord.exe --reset-password USER`.

## Self-hosting

Start the standalone backend:

```
runecord-server --host 0.0.0.0 --data data --no-demo
```

Put a TLS reverse proxy in front of the port. Register the first account. Gate signup with `--signup code --signup-code CODE`, or use `--signup closed`.

## Develop

Run these commands in this workspace and in `..\Runecord-backend`:

```
cargo fmt --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace
```

Read `docs/ARCHITECTURE.md`, `docs/DECISIONS.md`, `docs/RELEASING.md` and `BENCHMARKS.md` for details.

## Build

```
build.bat                 :: Windows and Linux
build.bat --desktop-only  :: Windows only
build.bat --clean         :: Clean the build cache first
```

For headless screenshots, run:

```
runecord.exe --snap shot.png --size 1360x860
runecord.exe --scene light --snap light.png
```

## Keys

Ctrl+K opens the quick switcher. Ctrl+F opens search. Ctrl+, opens settings.
Esc closes a dialog. Ctrl+Enter sends a message.
