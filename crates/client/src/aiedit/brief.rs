//! Turns a one-line idea into an implementation brief before the orchestrator
//! starts editing.
//!
//! A short request ("add a mascot in the corner") leaves too many decisions to
//! the patching round: where it goes, which module owns it, whether the icon is
//! new, how it behaves in both themes. The brief pass answers those questions
//! first, with the reference images in front of the model, so the orchestrator
//! spends its rounds on code instead of rediscovering the intent. A failed
//! brief is not fatal: the original request is still a valid goal.

use super::provider;

const BRIEF_PROMPT: &str = r#"You are the idea architect for Runecord's built-in UI editor. You turn a short
request, plus any reference images, into an implementation brief for a second model that will edit
Rust UI source and rebuild the app.

You do not write code in this reply. Return a plain-text brief of at most 700 words with exactly
these headings:

INTENT — the user's request in one sentence, including what success looks like.
REFERENCE — if an image is attached: the palette, shapes, proportions, typography and any visible
text the UI should echo. If there is no image, write "none".
WHERE — the surfaces and files that change, with concrete placements (rail, sidebar, chat header,
message row, composer, member list, settings, modal, title bar).
NEW — new files, modules, icons or widgets to create, with exact names, and how each one is
registered or imported.
STEPS — ordered, concrete edits an implementer can follow without guessing.
LOOK — layout, spacing, type scale and palette tokens, in both light and dark themes.
LIMITS — anything that needs server state, networking, storage, credentials or a new crate;
propose the closest UI-only version instead.
ACCEPTANCE — three checks a reviewer can see on screen.

Rules:
- Expanding the app is allowed. If the user asks for a character, mascot, sticker, badge, decorative
  panel or other element that does not exist yet, plan it: a new `stroke_icon!` or `fill_icon!` glyph
  in icons.rs, a new widget module, and a mount point. Do not answer "not implemented".
- Do not reproduce a trademarked character exactly. Describe an original glyph inspired by the
  reference: same spirit, different silhouette.
- The executor may create new .rs files under crates/client/src/ui/ or crates/ui/src/, and may read
  app/ files for state context. It cannot add crates, networking, persistence or credentials.
- Prefer additive changes. Never plan to remove a working feature unless the request says so.
- Name real files when you can. Say "unknown" rather than inventing a path."#;

/// A model call that may carry reference images.
pub type Ask =
    dyn Fn(&provider::Config, &[(String, String)], &[provider::Image]) -> Result<String, String>;

/// The conversation sent for the brief. Kept separate so the payload can be
/// inspected without a network.
pub fn messages(goal: &str, image_count: usize) -> Vec<(String, String)> {
    let mut request = String::from("Request:\n");
    request.push_str(goal.trim());
    if image_count == 0 {
        request.push_str("\n\nNo reference images are attached.");
    } else if image_count == 1 {
        request.push_str("\n\nOne reference image is attached.");
    } else {
        request.push_str(&format!("\n\n{image_count} reference images are attached."));
    }
    vec![
        ("system".to_string(), BRIEF_PROMPT.to_string()),
        ("user".to_string(), request),
    ]
}

/// Asks for a brief. Never fails the edit: on a provider error the request is
/// returned unchanged so the orchestrator still has something to work with.
pub fn expand(
    goal: &str,
    config: &provider::Config,
    images: &[provider::Image],
    ask: &Ask,
) -> String {
    let request = messages(goal, images.len());
    match ask(config, &request, images) {
        Ok(text) => {
            let brief = clean(&text);
            if brief.is_empty() {
                super::log("The brief came back empty; using the request as written.");
                goal.trim().to_string()
            } else {
                brief
            }
        }
        Err(error) => {
            if images.is_empty() {
                super::log(&format!(
                    "No separate brief was produced ({error}); using the request as written."
                ));
            } else {
                super::log(&format!(
                    "No separate brief was produced ({error}); using the request as written and \
                     passing the reference image to the first planning turn."
                ));
            }
            goal.trim().to_string()
        }
    }
}

/// Caps a runaway brief and drops a Markdown fence around it.
fn clean(text: &str) -> String {
    let mut lines: Vec<&str> = text.trim().lines().collect();
    if lines
        .first()
        .is_some_and(|line| line.trim_start().starts_with("```"))
    {
        lines.remove(0);
    }
    if lines.last().is_some_and(|line| line.trim() == "```") {
        lines.pop();
    }
    let flattened = lines.join("\n");
    let trimmed = flattened.trim();
    let limited: String = trimmed.chars().take(8_000).collect();
    limited.trim().to_string()
}

/// The goal the orchestrator receives: the user's words, then the brief.
pub fn orchestration_goal(goal: &str, brief: &str) -> String {
    let goal = goal.trim();
    let brief = brief.trim();
    if brief.is_empty() || brief == goal {
        return goal.to_string();
    }
    format!(
        "User request:\n{goal}\n\n\
         Implementation brief (generated from the request and its reference images):\n{brief}\n\n\
         The user's request is authoritative. Treat the brief as guidance: if it asks for something \
         the UI layer cannot do, build the closest UI-only version and say what was left out in PLAN."
    )
}

#[cfg(test)]
mod tests {
    use super::{expand, messages, orchestration_goal};
    use crate::aiedit::provider;

    #[test]
    fn the_brief_prompt_carries_the_idea_and_the_image_count() {
        let request = messages("add a mascot", 0);
        assert_eq!(request[0].0, "system");
        assert!(request[0].1.contains("INTENT"));
        assert!(request[1].1.contains("add a mascot"));
        assert!(request[1].1.contains("No reference images"));

        let with_images = messages("match this", 2);
        assert!(with_images[1].1.contains("2 reference images"));
    }

    #[test]
    fn a_failed_brief_falls_back_to_the_request() {
        let config = provider::Config {
            base_url: "http://stub.invalid/v1".into(),
            model: "stub".into(),
            key: "stub".into(),
        };
        let fallback = |_: &provider::Config,
                        _: &[(String, String)],
                        _: &[provider::Image]|
         -> Result<String, String> { Err("no provider".into()) };
        assert_eq!(
            expand("add a pink mascot", &config, &[], &fallback),
            "add a pink mascot"
        );
    }

    #[test]
    fn a_real_brief_is_prepended_to_the_request() {
        let goal = orchestration_goal("add a pink mascot", "INTENT — a mascot.\nWHERE — sidebar.");
        assert!(goal.starts_with("User request:\nadd a pink mascot"));
        assert!(goal.contains("Implementation brief"));
        assert!(goal.contains("sidebar"));
    }

    #[test]
    fn a_missing_brief_leaves_the_request_alone() {
        assert_eq!(
            orchestration_goal("add a mascot", "  "),
            "add a mascot",
            "an unavailable brief must not add empty headings"
        );
        assert_eq!(
            orchestration_goal("add a mascot", "add a mascot"),
            "add a mascot"
        );
    }
}
