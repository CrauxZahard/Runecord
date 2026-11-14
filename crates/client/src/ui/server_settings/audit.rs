use super::super::prelude::*;
use super::super::*;
use super::*;

pub(crate) fn audit_view(s: &App, p: Palette) -> El {
    let mut list = col().key("audit-list").w_full().gap(2.0);
    if s.audit.is_empty() {
        list = list.child(
            text(if s.audit_loading {
                "Loading audit log…"
            } else {
                "No audit entries yet."
            })
            .font(theme::SMALL)
            .color(p.text_faint)
            .padding(8.0),
        );
    }
    for entry in &s.audit {
        let (label, color) = audit_action(entry.action_type, p);
        let actor = s.user_name(entry.actor_id);
        let target = audit_target(s, entry);
        let changes = audit_changes(&entry.changes);
        list = list.child(
            row()
                .key(format!("audit-{}", entry.id))
                .w_full()
                .min_h(48.0)
                .px(10.0)
                .py(6.0)
                .radius(5.0)
                .gap(10.0)
                .center_y()
                .hover(p.card_hover)
                .child(el().w(4.0).h(30.0).radius(2.0).fill(color))
                .child(
                    col()
                        .grow(1.0)
                        .min_w(0.0)
                        .gap(2.0)
                        .child(
                            row()
                                .gap(6.0)
                                .center_y()
                                .child(text(label).font(theme::SMALL_MED).color(color))
                                .child(
                                    text(target)
                                        .font(theme::SMALL)
                                        .color(p.text_bright)
                                        .truncate(),
                                ),
                        )
                        .child(
                            text(format!("by {actor}"))
                                .font(theme::TINY)
                                .color(p.text_muted),
                        )
                        .children((!changes.is_empty()).then(|| {
                            text(changes)
                                .font(theme::TINY)
                                .color(p.text_faint)
                                .truncate()
                        })),
                )
                .child(
                    text(time_label(entry.created_at))
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        );
    }

    let mut footer = col().w_full().gap(8.0);
    if s.audit_loading {
        footer = footer.child(text("Loading…").font(theme::SMALL).color(p.text_faint));
    } else if !s.audit_complete && !s.audit.is_empty() {
        footer = footer.child(
            el().h(32.0)
                .px(16.0)
                .radius(4.0)
                .key("audit-more")
                .role("button")
                .label("Load older audit entries")
                .fill(p.rail)
                .hover(p.rail_hover)
                .center()
                .on_press(|s: &mut App| s.load_audit(true))
                .child(
                    text("Load Older")
                        .font(theme::SMALL_MED)
                        .color(p.text_bright),
                ),
        );
    }

    col()
        .key("audit")
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .gap(10.0)
        .child(
            text(format!("{} entries loaded", s.audit.len()))
                .font(theme::TINY)
                .color(p.text_faint),
        )
        .child(
            col()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(list),
        )
        .child(footer)
}

pub(crate) fn audit_action(action: i32, p: Palette) -> (String, Color) {
    let (label, color) = match action {
        10 => ("Created role", p.green),
        11 => ("Updated role", p.yellow),
        12 => ("Deleted role", p.danger),
        20 => ("Updated member roles", p.yellow),
        21 => ("Kicked member", p.danger),
        22 => ("Banned member", p.danger),
        23 => ("Timed out member", p.yellow),
        24 => ("Unbanned member", p.green),
        25 => ("Pruned members", p.yellow),
        26 => ("Accepted ban appeal", p.green),
        27 => ("Denied ban appeal", p.danger),
        28 => ("Submitted ban appeal", p.yellow),
        50 => ("Created webhook", p.green),
        51 => ("Deleted webhook", p.danger),
        60 => ("Created emoji", p.green),
        61 => ("Deleted emoji", p.danger),
        other => (return (format!("Action {other}"), p.text_muted)),
    };
    (label.to_string(), color)
}

pub(crate) fn audit_target(s: &App, entry: &AuditEntry) -> String {
    let Some(target) = entry.target_id else {
        return "server".to_string();
    };
    let name = audit_change_name(entry);
    match entry.action_type {
        10 | 11 | 12 => s
            .guild_roles(entry.guild_id)
            .iter()
            .find(|role| role.id == target)
            .map(|role| role.name.clone())
            .or(name)
            .unwrap_or_else(|| format!("Role #{target}")),
        20 | 21 | 22 | 23 | 24 | 26 | 27 => s
            .user(target)
            .map(|user| user.display_name.clone())
            .or(name)
            .unwrap_or_else(|| format!("#{target}")),
        60 | 61 => s
            .emojis
            .iter()
            .find(|emoji| emoji.id == target)
            .map(|emoji| format!(":{}:", emoji.name))
            .or(name.map(|name| format!(":{name}:")))
            .unwrap_or_else(|| format!("Emoji #{target}")),
        50 | 51 => name.unwrap_or_else(|| format!("Webhook #{target}")),
        _ => format!("#{target}"),
    }
}

pub(crate) fn audit_change_name(entry: &AuditEntry) -> Option<String> {
    match &entry.changes {
        serde_json::Value::Array(items) => items.iter().find_map(|item| {
            let object = item.as_object()?;
            if object.get("key")?.as_str()? != "name" {
                return None;
            }
            object
                .get("new_value")
                .or_else(|| object.get("old_value"))
                .and_then(|value| value.as_str())
                .map(str::to_string)
        }),
        serde_json::Value::Object(map) => map
            .get("name")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        _ => None,
    }
}

pub(crate) fn audit_changes(changes: &serde_json::Value) -> String {
    let pairs: Vec<String> = match changes {
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                let object = item.as_object()?;
                let key = object.get("key")?.as_str()?;
                let value = object
                    .get("new_value")
                    .or_else(|| object.get("old_value"))?;
                Some(format!("{key}: {}", audit_value(value)))
            })
            .collect(),
        serde_json::Value::Object(map) => map
            .iter()
            .filter(|(_, value)| !value.is_null())
            .map(|(key, value)| format!("{key}: {}", audit_value(value)))
            .collect(),
        _ => Vec::new(),
    };
    pairs.join(", ")
}

pub(crate) fn audit_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}
