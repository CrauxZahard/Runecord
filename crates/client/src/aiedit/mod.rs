//! "Edit your own UI": an AI-authored change to the view layer, built into a
//! separate executable and staged for a restart.
//!
//! A short request is first expanded into an implementation brief, with any
//! attached reference images in front of the model. The orchestrator then reads,
//! plans and patches; it may create new modules under the same UI trees.
//!
//! The model may only write files under [`patch::WRITABLE`]. The writer rejects
//! any other path, so "UI only, never logic" is enforced here rather than by the
//! system prompt. The build is a child process, the result is a new binary under
//! `versions/`, and the previous one stays runnable.

mod attach;
mod brief;
mod build;
pub mod install;
mod patch;
mod provider;
mod secret;
mod source;
#[cfg(test)]
mod tests;

pub use attach::Attachment;
pub use provider::Image;

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use rune::TextEdit;

/// The orchestrator contract. Sent once, ahead of every session.
const SYSTEM_PROMPT: &str = r#"You are the UI orchestrator for Runecord, a desktop chat client written in Rust
with the Rune UI toolkit. A session has one goal, written by the user in plain language and possibly
expanded into an implementation brief. You work in rounds: read, plan, patch, repair. You cannot ask
the user questions; when part of a goal cannot be met inside the view layer, build the closest
UI-only version and say what was left out in a NOTE.

## Scope

You may read and write only:
  crates/client/src/ui/**   screens, shell, settings, modals
  crates/ui/src/**          palette, type scale, icon catalogue

You may create new .rs files inside those two trees. A new file joins the build only when a file in
the same tree declares it (`mod mascot;`) and a mounted view calls it, so include that parent file in
the same reply even when nothing else in it changes.

You may also request these read-only files when you need to know what the view can reach:
  crates/client/src/app/mod.rs
  crates/client/src/app/actions/**
  crates/client/src/app/events/**
  crates/client/src/cache.rs
Never write them. The host rejects the whole patch if it names any other path. Networking (net/),
secrets, configuration (config.rs, paths.rs), startup, the binary, manifests, dependencies and the
App fields themselves are permanently out of reach.

## Build what does not exist yet

The icon catalogue and the current screens are a starting point, not a limit. If the user asks for
something the app has never had — a mascot, sticker, decorative panel, badge, background pattern, a
new card, an alternate navigation — make it from the primitives the UI already has. Do not refuse
because there is no existing field, icon or widget for it.

- A new icon: add an entry to `crates/ui/src/icons.rs` with `stroke_icon!` for a line glyph or
  `fill_icon!` for a solid mascot shape (or an inline `Icon { name, view, body }`), then draw it with
  `icon(&icons::YOUR_NAME, size)`.
- A new widget or panel: add `crates/client/src/ui/<name>.rs`, declare `mod <name>;` in
  `ui/mod.rs` (or the owning module), expose `pub(crate) fn <name>(s: &App, p: Palette) -> El`, and
  mount it from the shell or the screen it belongs to.
- A visual motif: gradients, rounded shapes, borders, inset shadows and type are available on `El`.
  Build the motif; recolouring an existing card is not the same thing.
- A reference image: keep its palette, proportions and mood, but draw an original glyph. Do not
  reproduce a trademarked character exactly.
- If the idea needs data the view cannot see (a new server field, a new persisted setting), build
  the part that is visible, use existing state where possible, and name the missing piece in PLAN.

## Rounds

1. You receive the goal, the brief when one was generated, the file list, and four grounding files.
   Reply with a PLAN and NEED lines for the files you must read. Do not write code for a file you
   have not seen.
2. You receive the files you requested. When you have everything, return the complete replacement
   files.
3. The host builds. If it fails you are shown the compiler errors and your patch has been rolled
   back, so the files on disk are the originals again; return the complete corrected files. At most
   four file requests and three repairs. Do not loop.

## Reply format

Exactly one action per message, with these markers:

<<<PLAN
Which elements move, resize, restyle, appear or disappear; which files; what stays untouched.
Three to six lines.
>>>
<<<NEED crates/client/src/ui/sidebar.rs>>>
<<<NOTE This part needs a stored preference, which the view layer cannot add.>>>
<<<FILE crates/client/src/ui/sidebar.rs
<the complete file>
>>>

## Patch rules

- Whole files, not diffs, and only files you actually change. A new file is a complete file.
- Register every new module in a parent file included in the same reply.
- It must compile on the first try. Use only `s.<field>` reads and `s.<method>()` calls that appear
  in the files you were given or requested; inventing one fails the build. Import only what the file
  already imports or what `rune`/`rune_kit` offers there. No new crates.
- Keep every existing `key`, `role`, `label`, `help` and handler unless the goal is about them.
- Do not reformat code you are not changing. A reviewable diff is part of the deliverable.
- Both themes: take colours from `Palette` (`p.*`) or `theme::*`; never hardcode a light-only value.
- Layout must not overflow: `.wrap()` or `.truncate()` text, `.min_w(0.0)` on flexible columns, and
  leave room for the member list, resize handles and the title bar.
- "Whole new UI", "redesign", "different app", "reorganize": these are structural briefs. Change
  where things are — column order, navigation model, header and composer placement, density, and the
  product name on screen — not only colours. Repainting the tokens while keeping the same columns
  does not satisfy that goal, and the user will reject it.
- Position lives in the shell: `crates/client/src/ui/mod.rs` holds `guild_shell` and `dm_shell`
  (column order is the order of their `.child(...)` calls) and the title bar; `rail.rs` is the
  server column, `sidebar.rs` the channel list, `members.rs` the member column, `dms.rs` the DM list
  and DM view, `composer.rs` the message box. Request these with NEED before a structural edit
  instead of patching only the tokens.
- For a structural goal, the smallest change that satisfies it is a layout change; do not fall back
  to a recolour. For a narrow goal, still change as little as possible and say so in PLAN.
- If the goal is already satisfied by the code, return a NOTE saying so; do not rewrite files for no
  reason."#;

/// One AI edit attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stage {
    Idle,
    /// A step label, shown under the spinner.
    Running(String),
    /// A build finished and is waiting for the restart button.
    Staged(String),
    /// The provider answered a test request.
    Checked(String),
    /// A short request was expanded into an implementation brief.
    Briefed(String),
    Failed(String),
}

/// Progress messages the poll loop drains.
pub enum Event {
    Stage(Stage),
    Line(String),
    /// The full brief, and the request it was written from.
    Brief {
        text: String,
        goal: String,
    },
}

/// The settings card's state. One of these lives on `App`.
pub struct UiEdit {
    pub prompt: TextEdit,
    pub base_url: TextEdit,
    pub model: TextEdit,
    pub key: TextEdit,
    pub key_saved: bool,
    pub stage: Stage,
    pub detail: Vec<String>,
    /// Staged builds, newest first, for switching between looks.
    pub history: Vec<install::Version>,
    /// Reference images attached to the next edit.
    pub attachments: Vec<Attachment>,
    /// The implementation brief for [`brief_for`](Self::brief_for), when one was
    /// generated. Reused only while the request is unchanged.
    pub brief: Option<String>,
    pub brief_for: String,
}

impl Default for UiEdit {
    fn default() -> Self {
        let provider = install::load_provider();
        let base_url = provider
            .as_ref()
            .map(|provider| provider.base_url.clone())
            .filter(|value| !value.trim().is_empty())
            .or_else(|| std::env::var("RUNECORD_AI_URL").ok())
            .unwrap_or_else(|| "https://opencode.ai/inference/go/openai/v1".into());
        let model = provider
            .as_ref()
            .map(|provider| provider.model.clone())
            .filter(|value| !value.trim().is_empty())
            .or_else(|| std::env::var("RUNECORD_AI_MODEL").ok())
            .unwrap_or_else(|| "deepseek-v4.1-flash".into());
        UiEdit {
            prompt: TextEdit::new(""),
            base_url: TextEdit::new(base_url),
            model: TextEdit::new(model),
            key: TextEdit::new(""),
            key_saved: secret::exists(),
            stage: Stage::Idle,
            detail: Vec::new(),
            history: install::versions(),
            attachments: Vec::new(),
            brief: None,
            brief_for: String::new(),
        }
    }
}

static BUSY: AtomicBool = AtomicBool::new(false);
static EVENTS: Mutex<Vec<Event>> = Mutex::new(Vec::new());

/// Model calls per session, across briefing, planning, file requests and repairs.
const MAX_STEPS: usize = 12;
/// Rounds of "send me these files" before the session is abandoned. A
/// structural redesign needs the shell modules too, not just the tokens.
const MAX_FILE_ROUNDS: usize = 5;
/// Compiler-error repairs after the first failed build.
const MAX_REPAIRS: usize = 3;
/// Plan-only turns before the session is abandoned.
const MAX_IDLE: usize = 2;

/// Whether an edit is running, so the card can say so instead of looking idle.
pub fn busy() -> bool {
    BUSY.load(Ordering::SeqCst)
}

/// Staged builds, newest first.
pub fn versions() -> Vec<install::Version> {
    install::versions()
}

/// Drains the progress queue. Called from the poll loop, on the UI thread.
pub fn take() -> Vec<Event> {
    let mut events = EVENTS.lock().unwrap_or_else(|poison| poison.into_inner());
    std::mem::take(&mut *events)
}

fn push(event: Event) {
    let mut events = EVENTS.lock().unwrap_or_else(|poison| poison.into_inner());
    if events.len() < 2_000 {
        events.push(event);
    }
}

fn set_stage(stage: Stage) {
    push(Event::Stage(stage));
}

/// A build or request line. Called from worker threads.
pub fn log(line: &str) {
    push(Event::Line(line.to_string()));
}

/// Sends one tiny request so the card can say whether the key, URL and model work
/// together, without paying for a build.
pub fn test_key(base_url: &str, model: &str) -> Result<String, String> {
    let config = provider::Config {
        base_url: base_url.trim().to_string(),
        model: model.trim().to_string(),
        key: secret::load().ok_or_else(|| "Save an API key first.".to_string())?,
    };
    let messages = vec![(
        "user".to_string(),
        "Reply with the single word OK.".to_string(),
    )];
    let reply = provider::ask(&config, &messages)?;
    let reply = reply.trim();
    if reply.is_empty() {
        return Err("The provider answered with an empty message.".into());
    }
    Ok(reply.chars().take(60).collect())
}

/// Starts an edit on a worker thread.
///
/// `brief` is a brief the user generated ahead of time for this exact request;
/// when it is absent the worker writes one before orchestrating. `images` are
/// the provider-facing reference images, not the preview buffers.
pub fn start(
    prompt: &str,
    base_url: &str,
    model: &str,
    images: Vec<Image>,
    brief: Option<String>,
) -> Result<(), String> {
    let goal = prompt.trim().to_string();
    if goal.is_empty() {
        return Err("Describe the change you want first.".into());
    }
    if BUSY.swap(true, Ordering::SeqCst) {
        return Err("An edit is already running.".into());
    }
    let config = provider::Config {
        base_url: base_url.trim().to_string(),
        model: model.trim().to_string(),
        key: secret::load().unwrap_or_default(),
    };
    // Remember the endpoint and model; only the key needs sealing.
    let _ = install::save_provider(&config.base_url, &config.model);
    set_stage(Stage::Running("Reading the UI source".into()));
    std::thread::Builder::new()
        .name("runecord-ui-edit".into())
        .spawn(move || {
            let result = run(&goal, &config, &images, brief.as_deref());
            BUSY.store(false, Ordering::SeqCst);
            match result {
                Ok(version) => set_stage(Stage::Staged(version)),
                Err(error) => set_stage(Stage::Failed(error)),
            }
        })
        .map_err(|error| {
            BUSY.store(false, Ordering::SeqCst);
            format!("Could not start the editor: {error}")
        })?;
    Ok(())
}

/// Writes a brief for a request without building anything, so the user can see
/// the plan before spending a build on it.
pub fn refine(prompt: &str, base_url: &str, model: &str, images: Vec<Image>) -> Result<(), String> {
    let goal = prompt.trim().to_string();
    if goal.is_empty() {
        return Err("Describe the change you want first.".into());
    }
    if BUSY.swap(true, Ordering::SeqCst) {
        return Err("An edit is already running.".into());
    }
    let config = provider::Config {
        base_url: base_url.trim().to_string(),
        model: model.trim().to_string(),
        key: secret::load().unwrap_or_default(),
    };
    let _ = install::save_provider(&config.base_url, &config.model);
    set_stage(Stage::Running("Refining the idea".into()));
    std::thread::Builder::new()
        .name("runecord-ui-refine".into())
        .spawn(move || {
            let text = brief::expand(&goal, &config, &images, &ask_with_images);
            BUSY.store(false, Ordering::SeqCst);
            if text.trim() == goal.trim() {
                set_stage(Stage::Failed(
                    "The provider couldn't write a brief. The request is unchanged; Generate still works."
                        .into(),
                ));
                return;
            }
            for line in text.lines().take(12) {
                log(&format!("Brief: {}", line.trim()));
            }
            push(Event::Brief {
                text,
                goal: goal.clone(),
            });
            set_stage(Stage::Briefed("Brief ready".into()));
        })
        .map_err(|error| {
            BUSY.store(false, Ordering::SeqCst);
            format!("Could not start the brief: {error}")
        })?;
    Ok(())
}

/// A model call. The orchestrator gets replies through this so a test can drive
/// the whole loop without a network.
type Ask<'a> = dyn Fn(&provider::Config, &[(String, String)]) -> Result<String, String> + 'a;

fn ask_with_images(
    cfg: &provider::Config,
    messages: &[(String, String)],
    images: &[Image],
) -> Result<String, String> {
    provider::ask_images(cfg, messages, images)
}

fn run(
    goal: &str,
    config: &provider::Config,
    images: &[provider::Image],
    brief: Option<&str>,
) -> Result<String, String> {
    let mut generated = false;
    let brief = match brief {
        Some(text) if !text.trim().is_empty() => text.trim().to_string(),
        _ => {
            generated = true;
            set_stage(Stage::Running("Briefing the idea".into()));
            brief::expand(goal, config, images, &ask_with_images)
        }
    };
    if brief.trim() != goal.trim() {
        log(&format!(
            "Brief prepared ({} lines); handing it to the orchestrator.",
            brief.lines().count()
        ));
        if generated {
            // Show the auto-generated brief and remember it, so pressing
            // Generate again on the same request does not pay for it twice.
            push(Event::Brief {
                text: brief.clone(),
                goal: goal.to_string(),
            });
        }
    }
    let goal = brief::orchestration_goal(goal, &brief);
    // The first turn is the only one that needs the pictures: the brief and the
    // plan carry the decisions into later rounds. A provider that rejects
    // images falls back to text inside `ask_images`.
    let first = AtomicBool::new(true);
    let ask = move |cfg: &provider::Config, messages: &[(String, String)]| {
        if first.swap(false, Ordering::SeqCst) {
            provider::ask_images(cfg, messages, images)
        } else {
            provider::ask(cfg, messages)
        }
    };
    run_with(&goal, config, &ask)
}

/// Rebuilds a source-only version without calling a model.
pub fn rebuild(name: &str) -> Result<(), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Pick a version to rebuild first.".into());
    }
    if !install::has_source(&name) {
        return Err(format!(
            "{name} has no source snapshot, so it cannot be rebuilt. Use Reset to default UI."
        ));
    }
    if BUSY.swap(true, Ordering::SeqCst) {
        return Err("An edit is already running.".into());
    }
    set_stage(Stage::Running(format!("Restoring the source for {name}")));
    std::thread::Builder::new()
        .name("runecord-ui-rebuild".into())
        .spawn(move || {
            let result = rebuild_run(&name);
            BUSY.store(false, Ordering::SeqCst);
            match result {
                Ok(version) => set_stage(Stage::Staged(version)),
                Err(error) => set_stage(Stage::Failed(error)),
            }
        })
        .map_err(|error| {
            BUSY.store(false, Ordering::SeqCst);
            format!("Could not start the rebuild: {error}")
        })?;
    Ok(())
}

fn rebuild_run(name: &str) -> Result<String, String> {
    let previous = install::state().current;
    install::restore_source(name)?;
    set_stage(Stage::Running(format!("Building {name}")));
    let root = patch::source_root()?;
    let target = install::target_dir()?;
    match build::release(&root, &target) {
        Ok(exe) => {
            set_stage(Stage::Running(format!("Staging {name}")));
            install::adopt_build(name, &exe)?;
            // The running UI is still the old one until Restart. Keep the tree
            // on its source so an edit started now changes what is on screen.
            if !previous.is_empty() && previous != name && install::has_source(&previous) {
                let _ = install::restore_source(&previous);
            }
            log(&format!(
                "{name} was rebuilt from its source snapshot. Restart to use it."
            ));
            Ok(name.to_string())
        }
        Err(error) => {
            if !previous.is_empty() && previous != name && install::has_source(&previous) {
                let _ = install::restore_source(&previous);
            }
            Err(format!("{error}\n\nThe previous UI source was restored."))
        }
    }
}

/// The orchestrator, with the model call passed in so a test can drive the whole
/// loop — planning, file requests, patch, build, staging — without a network.
fn run_with(goal: &str, config: &provider::Config, ask: &Ask<'_>) -> Result<String, String> {
    let manifest = patch::manifest()?;
    let grounding = patch::grounding()?;
    log(&format!(
        "Sending the file list and {} grounding file(s) to {}.",
        grounding.len(),
        config.model
    ));
    let mut messages = vec![
        ("system".to_string(), SYSTEM_PROMPT.to_string()),
        ("user".to_string(), opening(goal, &manifest, &grounding)),
    ];
    let mut file_rounds = 0usize;
    let mut repairs = 0usize;
    let mut idle = 0usize;

    for step in 1..=MAX_STEPS {
        set_stage(Stage::Running(if step == 1 {
            "Planning".to_string()
        } else {
            format!("Working (step {step} of {MAX_STEPS})")
        }));
        let reply = ask(config, &messages)?;
        let parsed = patch::parse(&reply)?;
        if let Some(plan) = &parsed.plan {
            for line in plan.lines().take(8) {
                log(&format!("Plan: {}", line.trim()));
            }
        }
        for note in &parsed.notes {
            log(&format!("Note: {note}"));
        }

        if !parsed.files.is_empty() {
            let changed: Vec<String> = parsed.files.iter().map(|(name, _)| name.clone()).collect();
            log(&format!("Writing {}.", changed.join(", ")));
            set_stage(Stage::Running("Writing the new UI".into()));
            let (backup, summary) = patch::snapshot_and_write(&parsed.files)?;
            for change in &summary {
                log(&format!(
                    "{}: {} → {} lines",
                    change.name, change.before, change.after
                ));
            }
            set_stage(Stage::Running(format!(
                "Building (attempt {} of {})",
                repairs + 1,
                MAX_REPAIRS + 1
            )));
            let root = patch::source_root()?;
            let target = install::target_dir()?;
            match build::release(&root, &target) {
                Ok(exe) => {
                    set_stage(Stage::Running("Staging the new build".into()));
                    let version = install::stage(&exe, stage_label(goal), &changed, &backup)?;
                    log(&format!("Build {version} is ready. Restart to try it."));
                    return Ok(version);
                }
                Err(error) => {
                    // A failed build must not leave the tree broken: the next
                    // attempt and the app itself both need it to compile.
                    let _ = install::restore_backup(&backup);
                    if repairs >= MAX_REPAIRS {
                        return Err(format!(
                            "{error}\n\nThe previous UI files were restored, so the app still builds."
                        ));
                    }
                    repairs += 1;
                    log(&format!(
                        "The build failed. Rolled back; asking for fix {repairs} of {MAX_REPAIRS}."
                    ));
                    messages.push((
                        "assistant".to_string(),
                        format!("Returned a patch for {}.", changed.join(", ")),
                    ));
                    messages.push((
                        "user".to_string(),
                        repair_request(&error, &changed, repairs),
                    ));
                    continue;
                }
            }
        }

        if !parsed.needs.is_empty() {
            if file_rounds >= MAX_FILE_ROUNDS {
                return Err(
                    "The model kept asking for files without producing a patch. Nothing was applied."
                        .into(),
                );
            }
            file_rounds += 1;
            let root = patch::source_root()?;
            let extra = patch::read_named(&root, &parsed.needs)?;
            log(&format!(
                "Sending {} requested file(s) (round {} of {}).",
                extra.len(),
                file_rounds,
                MAX_FILE_ROUNDS
            ));
            messages.push(("assistant".to_string(), reply.clone()));
            messages.push(("user".to_string(), files_reply(&extra)));
            continue;
        }

        if parsed.plan.is_some() {
            if idle >= MAX_IDLE {
                return Err(
                    "The model kept planning without producing a patch. Nothing was applied."
                        .into(),
                );
            }
            idle += 1;
            messages.push(("assistant".to_string(), reply.clone()));
            messages.push((
                "user".to_string(),
                "Proceed: request the files you still need, or return the complete files."
                    .to_string(),
            ));
            continue;
        }

        if !parsed.notes.is_empty() {
            return Err(format!("The editor declined: {}", parsed.notes.join(" ")));
        }
    }

    Err("The model ran out of steps without producing a patch. Nothing was applied.".into())
}

/// The version history should show what the user asked for, not the headings
/// the orchestrator received. `run` prefixes the goal with the brief; undo that
/// for the label only.
fn stage_label(goal: &str) -> &str {
    let label = goal
        .strip_prefix("User request:\n")
        .and_then(|rest| rest.split("\n\nImplementation brief").next())
        .unwrap_or(goal)
        .trim();
    if label.is_empty() { goal.trim() } else { label }
}

fn opening(goal: &str, manifest: &str, grounding: &[(String, String)]) -> String {
    let mut out = String::new();
    out.push_str("Goal:\n");
    out.push_str(goal.trim());
    out.push_str("\n\nFiles you may edit:\n");
    out.push_str(manifest);
    out.push_str(
        "\nNew .rs files under those folders are allowed and are not in the list. Request read-only \
         context from crates/client/src/app/** or crates/client/src/cache.rs with NEED when you need \
         to know a field or action; never write those paths.\n",
    );
    out.push_str("\nGrounding files you already have:\n\n");
    append_files(&mut out, grounding);
    out.push_str(
        "Round 1. Reply with a PLAN and NEED lines for the files you must read before editing.\n\
         If the grounding files are already enough, return the complete files instead of asking.",
    );
    out
}

fn files_reply(files: &[(String, String)]) -> String {
    let mut out = String::from("Files you requested:\n\n");
    append_files(&mut out, files);
    out.push_str(
        "If you have everything, return the complete replacement files.\n\
         Otherwise request the rest with <<<NEED path>>>.",
    );
    out
}

fn repair_request(error: &str, changed: &[String], attempt: usize) -> String {
    format!(
        "The build failed (repair {attempt} of {MAX_REPAIRS}). Your patch was rolled back, so the \
         files on disk are the originals again.\n\n\
         Files in the failed patch: {}\n\n\
         Compiler output:\n{error}\n\n\
         Return the complete corrected files. Keep the goal and fix only the cause of the failure.",
        changed.join(", ")
    )
}

fn append_files(out: &mut String, files: &[(String, String)]) {
    for (name, body) in files {
        out.push_str("<<<FILE ");
        out.push_str(name);
        out.push('\n');
        out.push_str(body);
        if !body.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(">>>\n\n");
    }
}

/// Writes the marker the restart supervisor waits for. Called on the first poll.
pub fn mark_ready() {
    static DONE: AtomicBool = AtomicBool::new(false);
    if DONE.swap(true, Ordering::SeqCst) {
        return;
    }
    install::record_base();
    install::prune_now();
    if let Ok(path) = std::env::var("RUNECORD_READY_FILE") {
        let _ = std::fs::write(path, b"ready");
    }
}

/// Starts the supervisor for the next process. The caller then quits.
pub fn supervise_cli() -> bool {
    install::supervise_from_args()
}

/// True when the installed app dispatched to the current staged build and this
/// process should exit.
pub fn launch_current() -> bool {
    install::launch_current()
}

/// The note from a build that failed to start, read once per launch.
pub fn startup_note() -> Option<String> {
    static TAKEN: AtomicBool = AtomicBool::new(false);
    if TAKEN.swap(true, Ordering::SeqCst) {
        return None;
    }
    install::take_failure()
}

impl crate::app::App {
    pub fn ui_edit_save_key(&mut self) {
        if self.ui_edit.key.text.trim().is_empty() {
            secret::clear();
            self.ui_edit.key_saved = false;
            self.toast("API key removed");
            return;
        }
        match secret::save(&self.ui_edit.key.text) {
            Ok(()) => {
                self.ui_edit.key_saved = true;
                self.ui_edit.key.replace_text("");
                self.toast("API key saved to aiedit\\.env");
            }
            Err(error) => {
                self.ui_edit.key_saved = secret::exists();
                self.ui_edit.stage = Stage::Failed(error.clone());
                self.toast(error);
            }
        }
    }

    pub fn ui_edit_test_key(&mut self) {
        if !self.ui_edit.key_saved {
            self.ui_edit.stage = Stage::Failed("Save an API key first.".into());
            return;
        }
        let base_url = self.ui_edit.base_url.text.clone();
        let model = self.ui_edit.model.text.clone();
        self.ui_edit.detail.clear();
        set_stage(Stage::Running("Testing the key".into()));
        let _ = std::thread::Builder::new()
            .name("runecord-ui-key-test".into())
            .spawn(move || match test_key(&base_url, &model) {
                Ok(reply) => {
                    log(&format!("Provider answered: {reply}"));
                    set_stage(Stage::Checked(format!(
                        "Key works. {model} answered: {reply}"
                    )));
                }
                Err(error) => set_stage(Stage::Failed(error)),
            });
    }

    /// Adds a reference image picked from disk. The card owns the count limit so
    /// both the picker and a paste share one error path.
    pub fn ui_edit_attach(&mut self, path: std::path::PathBuf) {
        if self.ui_edit.attachments.len() >= attach::MAX_COUNT {
            self.toast(format!(
                "At most {} reference images can be attached.",
                attach::MAX_COUNT
            ));
            return;
        }
        match attach::from_path(&path) {
            Ok(attachment) => {
                self.toast(format!("Attached {}", attachment.name));
                self.ui_edit.attachments.push(attachment);
            }
            Err(error) => self.toast(error),
        }
    }

    /// Adds an image pasted or dropped on the prompt field.
    pub fn ui_edit_attach_bytes(&mut self, name: String, bytes: Vec<u8>) {
        if self.ui_edit.attachments.len() >= attach::MAX_COUNT {
            self.toast(format!(
                "At most {} reference images can be attached.",
                attach::MAX_COUNT
            ));
            return;
        }
        match attach::from_bytes(name, bytes) {
            Ok(attachment) => {
                self.toast(format!("Attached {}", attachment.name));
                self.ui_edit.attachments.push(attachment);
            }
            Err(error) => self.toast(error),
        }
    }

    /// The provider-facing copies of the attached references.
    fn ui_edit_images(&self) -> Vec<provider::Image> {
        self.ui_edit
            .attachments
            .iter()
            .map(attach::Attachment::image)
            .collect()
    }

    /// The brief is reused only when it was written for the current request;
    /// otherwise Generate asks the worker for a fresh one.
    fn ui_edit_brief_for_prompt(&self) -> Option<String> {
        (self.ui_edit.brief_for == self.ui_edit.prompt.text.trim())
            .then(|| self.ui_edit.brief.clone())
            .flatten()
    }

    pub fn ui_edit_begin(&mut self) {
        if !self.ui_edit.key_saved {
            self.ui_edit.stage = Stage::Failed("Save an API key first.".into());
            return;
        }
        let prompt = self.ui_edit.prompt.text.clone();
        let images = self.ui_edit_images();
        let brief = self.ui_edit_brief_for_prompt();
        match start(
            &prompt,
            &self.ui_edit.base_url.text,
            &self.ui_edit.model.text,
            images,
            brief,
        ) {
            Ok(()) => {
                self.ui_edit.detail.clear();
                self.ui_edit.stage = Stage::Running("Starting".into());
            }
            // Pressing the button again while a build runs must not replace the
            // progress with an error.
            Err(error) if busy() => self.toast(error),
            Err(error) => self.ui_edit.stage = Stage::Failed(error),
        }
    }

    /// Asks for the implementation brief without building; the Generate button
    /// reuses it while the request is unchanged.
    pub fn ui_edit_refine(&mut self) {
        if !self.ui_edit.key_saved {
            self.ui_edit.stage = Stage::Failed("Save an API key first.".into());
            return;
        }
        let prompt = self.ui_edit.prompt.text.clone();
        let images = self.ui_edit_images();
        match refine(
            &prompt,
            &self.ui_edit.base_url.text,
            &self.ui_edit.model.text,
            images,
        ) {
            Ok(()) => {
                self.ui_edit.detail.clear();
                self.ui_edit.stage = Stage::Running("Refining the idea".into());
            }
            Err(error) if busy() => self.toast(error),
            Err(error) => self.ui_edit.stage = Stage::Failed(error),
        }
    }

    /// Copies the generated brief into the prompt, so the user can edit the
    /// plan that the orchestrator will follow. Generate then reuses it instead
    /// of paying for a second brief.
    pub fn ui_edit_use_brief(&mut self) {
        let Some(brief) = self.ui_edit.brief.clone() else {
            self.toast("There is no brief to use yet.");
            return;
        };
        self.ui_edit.prompt.replace_text(brief.trim());
        self.ui_edit.brief_for = self.ui_edit.prompt.text.trim().to_string();
        self.toast("Brief copied into the prompt; edit it or press Generate.");
    }

    pub fn ui_edit_restart(&mut self) -> Result<(), String> {
        let result = install::restart_staged();
        if let Err(error) = &result {
            self.ui_edit.stage = Stage::Failed(error.clone());
        }
        result
    }

    /// Undo the last edit only, leaving earlier ones in place.
    pub fn ui_edit_undo(&mut self) -> Result<(), String> {
        let restored = install::restore_latest_backup()?;
        let result = install::restart_last_good();
        if let Err(error) = &result {
            self.ui_edit.stage = Stage::Failed(error.clone());
        }
        self.ui_edit.history = install::versions();
        self.toast(format!(
            "Undid the last edit ({restored} file(s)); restarting"
        ));
        result
    }

    /// Restart into one of the staged builds.
    pub fn ui_edit_use(&mut self, name: &str) -> Result<(), String> {
        let result = install::restart_into(name);
        if let Err(error) = &result {
            self.ui_edit.stage = Stage::Failed(error.clone());
        }
        result
    }

    /// Rebuild a source-only version, then leave it staged for Restart.
    pub fn ui_edit_rebuild(&mut self, name: &str) {
        match rebuild(name) {
            Ok(()) => {
                self.ui_edit.detail.clear();
                self.ui_edit.stage = Stage::Running(format!("Rebuilding {name}"));
            }
            Err(error) if busy() => self.toast(error),
            Err(error) => self.ui_edit.stage = Stage::Failed(error),
        }
    }

    pub fn ui_edit_reset(&mut self) -> Result<(), String> {
        let restored = install::restore_all_backups()?;
        let result = install::restart_base();
        if let Err(error) = &result {
            self.ui_edit.stage = Stage::Failed(error.clone());
        }
        self.ui_edit.stage = Stage::Idle;
        self.toast(format!("Restored {restored} file(s); restarting"));
        result
    }
}
