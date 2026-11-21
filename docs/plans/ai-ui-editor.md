# AI UI editor: "edit your own UI"

Status: phase 0 and the core loop are implemented; the toolchain download and
the full source bundle for packaged installs are not.

## What exists today

| Path | Role |
|---|---|
| `crates/ui` | `runecord-ui`: palette, type scale, icon catalogue |
| `crates/client/src/aiedit/mod.rs` | prompt, worker thread, progress events, `App` actions |
| `crates/client/src/aiedit/brief.rs` | turns a short request and its references into an implementation brief |
| `crates/client/src/aiedit/attach.rs` | decodes, downscales and previews reference images |
| `crates/client/src/aiedit/source.rs` | content-addressed UI source snapshots and restore |
| `crates/client/src/aiedit/provider.rs` | OpenAI-compatible chat completions, including image parts |
| `crates/client/src/aiedit/patch.rs` | file allowlist, reply parser, backup and write |
| `crates/client/src/aiedit/build.rs` | `cargo build --release` child process, streamed lines |
| `crates/client/src/aiedit/install.rs` | versions, `state.json`, restart supervisor, restore |
| `crates/client/src/aiedit/secret.rs` | key storage: `aiedit\.env`, DPAPI-sealed |
| `crates/client/src/ui/settings/appearance.rs` | the settings card |
| `crates/client/src/main.rs` | supervisor dispatch before the app starts |

Trying it in a checkout: `run.bat`, Settings → Appearance → **Edit your own UI**,
save an OpenAI-compatible key, ask for "move the channel sidebar to the right of
the chat", wait for the build, press **Restart the application**. Builds land in
`%APPDATA%\Runecord\aiedit\`.

The card keeps the full source-backed history in a list: each build stores its
goal, the files it changed and the source snapshot, and **Use** switches back to
it (restoring that source first). **Undo last change** reverts only the newest
edit; **Reset to default UI** replays every backup and reopens the original
build. Only the current and last-good binaries stay on disk (plus a pending one
during a restart); older versions keep a source snapshot and rebuild on demand.
When a build's recorded `app_version` differs from the running app, the card
warns and offers **Keep for now**, **Rebuild**, or **Reset**; keeping silences
that warning until the next app update.

The editor is labelled Preview. Without a UI source tree the Appearance card
shows a **No source** notice and does not offer Generate; nothing is sent to a
provider. The prompt area also states that the request, reference images and
the UI files the editor reads go to the configured provider, that the provider
bills the user's key, and that one edit is capped at 12 model calls and 3
repairs.

The provider URL and model persist in `aiedit\provider.json`; on Windows the key
is a DPAPI-sealed blob in `aiedit\.env`, elsewhere a 0600 file.

Cartridge-free defaults: the card now ships pointing at opencode's Go gateway
(`https://opencode.ai/inference/go/openai/v1`) with `deepseek-v4.1-flash`, and
sends the `x-opencode-session` header that gateway requires when it sees an
`opencode.ai` host. **Test** sends one tiny request and reports the provider's
answer, so a key can be checked without paying for a build.

Six prompt chips sit above the buttons ("Sidebar on the right", "Tighter message
rows", "Bigger chat type", "Hide the member list", "Add a mascot in the corner",
"Playful, rounded icons") and fill the box rather than replacing it. The last two
show that the editor is not limited to rearranging what already exists.

The card also accepts **reference images** (picker, paste or drop on the prompt).
Each one is decoded, shrunk to a 1400px long side, re-encoded and shown as a
thumbnail; Generate sends them to the model with the request. A provider that
rejects image parts gets one text-only retry instead of failing the edit.

**Refine idea** turns the request into a structured implementation brief
(intent, reference, placement, new modules, steps, look, limits, acceptance)
before any build. Generate runs the same step automatically when no brief exists
for the current request, shows it under the prompt, and reuses it while the text
is unchanged. A failed brief is not fatal: the original request is used, and the
reference images still reach the first planning turn. **Use as prompt** copies
the brief into the editable prompt so it can be tuned before building.

The writable tree now includes files that do not exist yet. A new icon in
`crates/ui/src/icons.rs`, a new widget module under `crates/client/src/ui/`, and
the parent file that registers it are a valid patch. Read-only context from
`crates/client/src/app/**` and `cache.rs` can be requested with `NEED` so a new
view can call real state and actions without guessing.

## How a session runs

The model is an orchestrator, not a one-shot patcher. `aiedit::run` keeps a
message history and accepts one action per turn:

| Marker | Meaning |
|---|---|
| `<<<PLAN … >>>` | What moves, which files, what stays untouched. Logged in the card. |
| `<<<NEED path>>>` | A file to read before editing. The host sends it. |
| `<<<FILE path … >>>` | Complete replacement files. Host writes, backs up, builds. |
| `<<<NOTE … >>>` | The goal needs logic the UI layer cannot add; shown to the user as-is. |

Round 1 gets the goal, the generated brief, a manifest of every writable file
with line counts, four grounding files (`ui/mod.rs`, `ui/common.rs`, `theme.rs`,
`icons.rs`) and the reference images on the first turn, so it plans instead of
guessing. Caps: 12 model calls, 5 file rounds, 2 plan-only turns, 3 repairs. A
failed build rolls the files back, then hands the compiler output back to the
model with the list of files in the failed patch; after the third repair the
error is shown and the tree is left compiling.

### Storage: runnable binaries, full source history

Only the installed app, `current`, `last_good` and (between staging and restart)
`pending` stay as executables. Every version records a full-tree source snapshot in
`aiedit\source`:

```text
source\objects\<sha256>          one copy of each distinct UI file
source\versions\<name>\manifest.json
```

A manifest lists every writable file and its object hash, so it can reconstruct
the exact tree; unchanged files are stored once. When a build is promoted, the
old current becomes last-good and the old last-good executable is deleted. An
older version without a binary shows **Rebuild** in the card: it restores the
manifest, builds, stages the result, and leaves the running UI alone until
**Restart**. Source snapshots are roughly 1 MB for the whole UI, versus 93 MB
per executable.

### Launcher

The installed app also dispatches on startup: it reads `state.json`, starts
`current` through the supervisor, and exits. If current never writes its ready
marker, it tries `last_good`, then its own built-in UI. A development checkout
skips this dispatch so `run.bat` still runs the source that was just built.
This is what makes the desktop shortcut follow the edited version instead of
always opening the original binary.

Startup also prunes any executable that is not current, last-good or pending.
The source manifests are never pruned by that pass.

Not built yet: the one-time toolchain and source download for packaged installs
(`source_root()` currently finds a sibling checkout or `%APPDATA%\runecord\aiedit\src`);
the card detects a missing source tree and disables Generate. A diff preview
before writing is also still open.


## What the feature is

Settings → Appearance gains an **Edit your own UI** card. The user types what
they want, brings their own OpenAI-compatible key, and the client:

1. asks the model to change the UI source,
2. builds the result next to the running version (never in place),
3. shows build progress,
4. offers **Restart** once a binary is staged,
5. keeps the previous binary so a broken build can fall back.

The user-facing payoff is "move the channel sidebar to the right", "rail at the
bottom", "bigger avatars, tighter rows", "no member list on this server".

## Decisions

| Question | Answer |
|---|---|
| Who can use it | Shipped installer users, not just dev checkouts |
| Engine | The UI source is Rust; the model edits it and it is rebuilt |
| Scope | Anything in the UI, including layout of every screen |
| Provider | OpenAI-compatible chat completions, custom base URL, BYO key |
| Toolchain | Downloaded once, on first use of the feature |
| Failure handling | Keep last-good binary, automatic fallback, reset to default |
| Boundary | Path allowlist in the patch writer; `crates/ui` holds the tokens |

## Why the split is the first phase, not a detail

Moving `ui/` into its own crate is not a folder move. The view layer is bound
to `App` in three ways:

| Coupling | Count |
|---|---|
| Files under `crates/client/src/ui` | 52 |
| Lines under `crates/client/src/ui` | ~22,500 |
| Callbacks typed `&mut App` | 595 |
| Calls to `App` methods (`s.something()`) | 765 |
| Direct reads of `App` fields (`s.field`) | 1,463 |
| `pub` fields on `App` | 317 |

A `UiState` + `Action` boundary therefore rewrites every callback signature and
every field access. That is a rewrite of the interaction model, not a move, and
it is the largest single risk in this project. Doing it in one commit would
leave the app uncompilable for days and make regressions impossible to bisect.

The plan below migrates in slices that each compile and ship.

## Target shape

```text
crates/ui          package runecord-ui
  tokens/          theme (palette, type scale), icons        <- App-free today
  state.rs         UiState: the fields the view reads, owned by the client
  act.rs           Act: the intents the view emits
  screens/         one module per surface, moved from crates/client/src/ui
crates/client
  app/mod.rs       App { ui: UiState, core: Core }
  app/act.rs       impl: Act -> App methods (the only place view meets logic)
```

Two properties matter more than the exact file layout:

- **`runecord-ui` cannot name `net`, `config`, `paths`, `session` or the
  session token type.** A model edit cannot reach credentials or the network,
  and "UI only" stops being a prompt request. Add `reqwest` to the forbidden
  list, or the crate can exfiltrate anything it can see.
- **Field names are preserved** (`s.compact`, `s.channels`, `s.sidebar_width`),
  so call sites stay readable and the mechanical diff is a signature change,
  not a rename. Intents become `s.act(Act::SetDark(dark))`.

Editing `runecord-ui` recompiles that crate and `runecord`, not `rune`, `wgpu`
or the dependency tree. That is the difference between a one-minute build and
a twenty-minute one.

## Install layout and launch lifecycle

```text
%LOCALAPPDATA%\Programs\Runecord\
  Runecord.exe             launcher: state.json -> spawn a version, verify, fall back
  versions\base\runecord.exe   shipped, signed
  versions\ui-3\runecord.exe   an AI build
  src\                     writable copy of crates/ui (+ crates/ui/tokens)
  src.base\                pristine copy, source of "reset"
  state.json               current, last_good, pending
  toolchain\, vendor\, cache\target\    downloaded once, see below
```

The running process cannot replace its own exe on Windows. So:

1. Build into a scratch target dir, copy the binary to a new `versions\ui-N`.
2. Set `pending` in `state.json`, leave `current` alone.
3. **Restart** stops the app; the launcher starts `pending`, which writes a
   ready marker after the first frame.
4. Marker seen → `current = pending`, old version deleted. Marker missing or
   the process exits → start `last_good`, record the failure for the settings
   card.

Step 4 is why "reset to default" alone is not enough: the reset button lives
inside the app that failed to start. Reset remains the only *user-facing*
control; fallback is internal and invisible unless it fires.

## Toolchain delivery

The base installer stays as it is (4 files, 88 MB exe). On first use of the
feature the client downloads:

- rustup-distributed 1.99.1 toolchain with std (~1 GB installed)
- the vendored workspace: `crates/ui`, `crates/client`'s manifests, and copies
  of the two sibling trees the manifest path-links (`rune`'s vendored
  `wgpu-hal`/`winit`, `Runecord-backend`'s `protocol`) — the published source
  cannot use `../rune/...` paths
- a prebuilt dependency `target/` for the release profile (multi-GB, the part
  that makes the first user build minutes instead of an hour)

Show a single progress screen with bytes and a cancel button; resume by range.
Checksum every archive against a manifest signed into the binary. Offline
users get a clear "this feature needs a one-time download" state, not a hang.

Consequence to accept: an AI-edited UI is pinned to the base version it was
built from. When the app updates, show "your custom UI was built for 0.1.0 —
keep or reset". Never silently rebase a diff.

## The AI loop

1. **Retrieve.** The UI crate is small enough to send whole files. Rank by the
   files a prompt plausibly touches (`rail`, `sidebar`, `chat`, `settings`),
   always include `tokens/`, cap the prompt at a budget and say what was left
   out.
2. **Patch.** Ask for whole-file replacements of named files only. Reject a
   response that names a path outside `src\`.
3. **Build.** `cargo build --release -p runecord --features embedded-server`
   in a builder subprocess with `CARGO_TARGET_DIR` pointed at the scratch tree.
   Stream stderr into the progress card as normalised steps (compiling,
   linking, done) rather than a wall of text.
4. **Repair.** On failure, send the compiler errors plus the original diff back
   for up to two more attempts, showing "attempt 2 of 3". A model that sees
   `rustc` output converges far more often than a one-shot patch.
5. **Stage.** Copy the binary, mark pending, show **Restart**.

The system prompt saying "do not change app logic" is useful but not the
boundary; the writable tree is the boundary. Keep the sentence anyway, because
it improves output.

Key handling: a `.env` beside the editor's state (`aiedit\.env`). On Windows it
holds a DPAPI-sealed blob under `RUNECORD_AI_KEY_DPAPI`; elsewhere it is a 0600
file. The app never writes the key to a JSON file. A hand-written plaintext
`RUNECORD_AI_KEY=` line is accepted when reading; the app only writes the sealed
form on Windows. Saving reads the file back and compares before the card says
"Saved".

## Phases

| Phase | Work | Exit criteria | Estimate |
|---|---|---|---|
| 0 | `crates/ui` with `theme` + `icons`; client re-exports them | `cargo check` green, app unchanged, AI could restyle colours and type | 1-2 days |
| 1 | `UiState`/`Act` boundary for one leaf surface (Appearance tab) | two crates, one screen proves the pattern, no behaviour change | 3-5 days |
| 2 | Shell: rail, sidebar, chat header, member list, resize handles | sidebar can move right by editing the crate | 1-2 weeks |
| 3 | Settings, modals, server settings, chat rows | full `ui/` lives in `runecord-ui` | 2-3 weeks |
| 4 | Launcher stub, versioned installs, ready marker, fallback | a deliberately broken build falls back unattended | 1-2 weeks |
| 5 | Builder subprocess, toolchain download, progress streaming | first-use download and a full edit build end to end | 1-2 weeks |
| 6 | Prompt card, diff summary, retries, key storage, reset, update conflict | usable by someone who is not the author | 1-2 weeks |
| 7 | Hardening: AV checks, disk-full, offline, cancelled builds, crash loops | no install can be left unusable | 1-2 weeks |

Phases 4-6 can start as soon as phase 2 lands, in parallel with 3.

## Known unknowns

- **Callback typing (resolved by inspection).** `El::on_press` is generic over
  the state type (`on_press<S: 'static>(impl Fn(&mut S))`) and stores the
  closure by `TypeId` of the concrete app state; `states::downcast` panics with
  "handler registered for a different state type than the app's". So a closure
  typed `&mut UiState` inside a separate crate would panic at the first click
  unless the app's state type really is `UiState`. Two shapes follow, and phase
  1 has to pick one:
  - **`trait UiApp`, generic screens.** `fn view<A: UiApp>(s: &A, cx: &mut Cx<A>)`
    monomorphised at `A = App`; closures stay `&mut A` and downcast correctly.
    Strongest boundary, but 1,463 field reads become accessor calls and the
    trait grows to ~150-300 methods.
  - **`App` moves to a shared state crate.** `runecord-ui` takes `&mut App`
    unchanged, so almost no call sites change, but the boundary becomes "the
    view crate only depends on the state crate" rather than "the view cannot
    name the network" — review-enforced, not compiler-enforced.
- **Handle-like state.** `UiState` holds `Pixels` for avatars and images, the
  zoom preview, and `Rc` caches. Decide whether those are fields of `UiState`
  or opaque handles the client owns.
- **Memo keys.** Memoised subtrees key off `&App` identity; a state split must
  keep keys stable or every edit repaints the world.
- **AV and enterprise policy.** Shipping or downloading `rustc` is a common
  heuristic trigger; the installer is self-signed. Test against Defender and
  at least one enterprise AV before shipping.
- **Build cost on user hardware.** The client crate is one compilation unit;
  until phase 3 the AI edits still rebuild `app/` and `net/` too.
- **Disk.** Toolchain + vendor + prebuilt target is several GB. Check free
  space before download and uninstall it when the feature is reset.

## Non-goals

- Running AI-authored code as a plugin. This edits the app's own UI source.
- Server-side builds. Everything compiles on the user's machine.
- Edits to networking, auth, storage or voice. Structurally excluded by the
  crate boundary, not by instruction.
