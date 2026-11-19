//! "Edit your own UI": an AI-authored change to the view layer, built into a
//! separate executable and staged for a restart.
//!
//! The model may only write files under [`patch::WRITABLE`]. The writer rejects
//! any other path, so "UI only, never logic" is enforced here rather than by the
//! system prompt. The build is a child process, the result is a new binary under
//! `versions/`, and the previous one stays runnable.

mod build;
pub mod install;
mod patch;
mod provider;
mod secret;
#[cfg(test)]
mod tests;

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use rune::TextEdit;

/// The orchestrator contract. Sent once, ahead of every session.
const SYSTEM_PROMPT: &str = r#"You are the UI orchestrator for Runecord, a desktop chat client written in Rust
with the Rune UI toolkit. A session has one goal, written by the user in plain language.
You work in rounds: read, plan, patch, repair. You cannot ask the user questions; when a
goal cannot be met inside the view layer, you say so with a NOTE.

## Scope

You may read and write only:
  crates/client/src/ui/**   screens, shell, settings, modals
  crates/ui/src/**          palette, type scale, icon catalogue

The host rejects the whole patch if it names any other path. Networking (net/), application
state (app/), configuration (config.rs, paths.rs), startup (bootstrap.rs), the binary
(main.rs), manifests and dependencies are permanently out of reach; do not request them.

A change that needs new state, a new server call, a new dependency, or a file outside those
two folders cannot be made. Return a NOTE explaining that instead of a patch that cannot
compile.

## Rounds

1. You receive the goal, the file list, and three grounding files. Reply with a PLAN and NEED
   lines for the files you must read. Do not write code for a file you have not seen.
2. You receive the files you requested. When you have everything, return the complete
   replacement files.
3. The host builds. If it fails you are shown the compiler errors and your patch has been
   rolled back, so the files on disk are the originals again; return the complete corrected
   files. At most three file requests and two repairs. Do not loop.

## Reply format

Exactly one action per message, with these markers:

<<<PLAN
Which elements move, resize, restyle, appear or disappear; which files; what stays untouched.
Three to six lines.
>>>
<<<NEED crates/client/src/ui/sidebar.rs>>>
<<<NOTE This needs a stored preference, which the view layer cannot add.>>>
<<<FILE crates/client/src/ui/sidebar.rs
<the complete file>
>>>

## Patch rules

- Whole files, not diffs, and only files you actually change.
- It must compile on the first try. Use only `s.<field>` reads and `s.<method>()` calls that
  appear in the files you were given; inventing one fails the build. Import only what the file
  already imports or what `rune`/`rune_kit` offers there. No new crates.
- Keep every existing `key`, `role`, `label`, `help` and handler unless the goal is about them.
- Do not reformat code you are not changing. A reviewable diff is part of the deliverable.
- Both themes: take colours from `Palette` (`p.*`) or `theme::*`; never hardcode a light-only
  value.
- Layout must not overflow: `.wrap()` or `.truncate()` text, `.min_w(0.0)` on flexible columns,
  and leave room for the member list, resize handles and the title bar.
- "Whole new UI", "redesign", "different app", "reorganize": these are structural briefs. Change
  where things are — column order, navigation model, header and composer placement, density, and
  the product name on screen — not only colours. Repainting the tokens while keeping the same
  columns does not satisfy that goal, and the user will reject it.
- Position lives in the shell: `crates/client/src/ui/mod.rs` holds `guild_shell` and `dm_shell`
  (column order is the order of their `.child(...)` calls) and the title bar; `rail.rs` is the
  server column, `sidebar.rs` the channel list, `members.rs` the member column, `dms.rs` the DM
  list and DM view, `composer.rs` the message box. Request these with NEED before a structural
  edit instead of patching only the tokens.
- For a structural goal, the smallest change that satisfies it is a layout change; do not fall
  back to a recolour. For a narrow goal, still change as little as possible and say so in PLAN.
- If the goal is already satisfied by the code, return a NOTE saying so; do not rewrite files
  for no reason."#;

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
    Failed(String),
}

/// Progress messages the poll loop drains.
pub enum Event {
    Stage(Stage),
    Line(String),
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
        }
    }
}

static BUSY: AtomicBool = AtomicBool::new(false);
static EVENTS: Mutex<Vec<Event>> = Mutex::new(Vec::new());

/// Model calls per session, across planning, file requests and repairs.
const MAX_STEPS: usize = 10;
/// Rounds of "send me these files" before the session is abandoned. A
/// structural redesign needs the shell modules too, not just the tokens.
const MAX_FILE_ROUNDS: usize = 4;
/// Compiler-error repairs after the first failed build.
const MAX_REPAIRS: usize = 2;
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
pub fn start(prompt: &str, base_url: &str, model: &str) -> Result<(), String> {
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
            let result = run(&goal, &config);
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

/// A model call. The orchestrator gets replies through this so a test can drive
/// the whole loop without a network.
type Ask = dyn Fn(&provider::Config, &[(String, String)]) -> Result<String, String>;

fn run(goal: &str, config: &provider::Config) -> Result<String, String> {
    run_with(goal, config, &|cfg, messages| provider::ask(cfg, messages))
}

/// The orchestrator, with the model call passed in so a test can drive the whole
/// loop — planning, file requests, patch, build, staging — without a network.
fn run_with(goal: &str, config: &provider::Config, ask: &Ask) -> Result<String, String> {
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
                    let version = install::stage(&exe, goal, &changed, &backup)?;
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

fn opening(goal: &str, manifest: &str, grounding: &[(String, String)]) -> String {
    let mut out = String::new();
    out.push_str("Goal:\n");
    out.push_str(goal.trim());
    out.push_str("\n\nFiles you may edit:\n");
    out.push_str(manifest);
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
    if let Ok(path) = std::env::var("RUNECORD_READY_FILE") {
        let _ = std::fs::write(path, b"ready");
    }
}

/// Starts the supervisor for the next process. The caller then quits.
pub fn supervise_cli() -> bool {
    install::supervise_from_args()
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

    pub fn ui_edit_begin(&mut self) {
        if !self.ui_edit.key_saved {
            self.ui_edit.stage = Stage::Failed("Save an API key first.".into());
            return;
        }
        match start(
            &self.ui_edit.prompt.text,
            &self.ui_edit.base_url.text,
            &self.ui_edit.model.text,
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

    pub fn ui_edit_reset(&mut self) -> Result<(), String> {
        let restored = install::restore_all_backups()?;
        let result = install::restart_last_good();
        if let Err(error) = &result {
            self.ui_edit.stage = Stage::Failed(error.clone());
        }
        self.ui_edit.stage = Stage::Idle;
        self.toast(format!("Restored {restored} file(s); restarting"));
        result
    }
}
