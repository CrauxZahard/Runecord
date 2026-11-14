//! Shared view primitives surface.

use super::*;

pub(crate) fn rail_icon(
    key: &'static str,
    ic: &'static Icon,
    size: f32,
    selected: bool,
    selected_fill: Color,
    hover_fill: Color,
    help: &'static str,
) -> El {
    el().key(key)
        .size(48.0, 48.0)
        .radius(if selected { 16.0 } else { 24.0 })
        .fill(if selected { selected_fill } else { hover_fill })
        .center()
        .role("button")
        .label(help)
        .motion(motion::SNAPPY)
        .hover(Color::WHITE.alpha(0.06))
        .help(help)
        .child(icon(ic, size).color(Color::WHITE))
}

pub(crate) fn with_indicator(button: El, active: bool, unread: bool, p: Palette) -> El {
    let h = if active {
        40.0
    } else if unread {
        8.0
    } else {
        0.0
    };
    row().w_full().h(48.0).center().child(button).child(
        el().key("indicator")
            .absolute(0.0, if active { 4.0 } else { 20.0 })
            .w(4.0)
            .h(h)
            .radius(2.0)
            .fill(p.text_bright),
    )
}

// ---------------------------------------------------------------------------
// Channel sidebar
// ---------------------------------------------------------------------------

pub(crate) fn panel_button(key: &'static str, ic: &'static Icon, p: Palette, active: bool) -> El {
    let label = match (key, active) {
        ("mic", true) => "Unmute",
        ("mic", false) => "Mute",
        ("deafen", true) => "Undeafen",
        ("deafen", false) => "Deafen",
        ("settings", _) => "User settings",
        _ => "Control",
    };
    el().key(key)
        .size(32.0, 32.0)
        .radius(4.0)
        .center()
        .role("button")
        .label(label)
        .help(label)
        .hover(p.sidebar_hover)
        .child(icon(ic, 20.0).color(if active { p.danger } else { p.interactive }))
}

// ---------------------------------------------------------------------------
// Chat
// ---------------------------------------------------------------------------

pub(crate) fn tool_button(key: &'static str, ic: &'static Icon, p: Palette) -> El {
    let label = match key {
        "threads" => "Threads",
        "notifications" => "Notification settings",
        "pins" => "Pinned messages",
        "members" => "Toggle member list",
        "permissions" => "Channel permissions",
        "inbox" => "Inbox",
        "help" => "About Runecord",
        _ => "Tool",
    };
    el().key(key)
        .size(32.0, 32.0)
        .radius(4.0)
        .center()
        .role("button")
        .label(label)
        .help(label)
        .hover(p.rail)
        .child(icon(ic, 20.0).color(p.interactive))
}

pub(crate) fn action_button(ic: &'static Icon, p: Palette, run: impl Fn(&mut App) + 'static) -> El {
    el().size(32.0, 30.0)
        .center()
        .role("button")
        .hover(p.rail)
        .on_press(run)
        .child(icon(ic, 18.0).color(p.interactive))
}

pub(crate) fn markdown(s: &App, content: &str, p: Palette) -> El {
    if let Some(el) = custom_emoji_text(s, content, p) {
        return el;
    }
    let style = rune_kit::MarkdownStyle::neutral(p.dark)
        .text(p.text)
        .muted(p.text_muted)
        .link(p.text_link)
        .inline_code_bg(p.code_bg)
        .code_block(p.code_bg, p.divider_soft)
        .body_font(theme::BODY)
        .code_block_font(theme::MONO)
        .strong_weight(700.0);
    rune_kit::Markdown::new(content.to_string())
        .render(&style)
        .selectable(true)
        .key("markdown")
        .min_w(0.0)
}

/// Messages containing a server emoji render as text with inline image icons.
/// Rune's Markdown lifts images into their own block, so `:name:` cannot stay in
/// a sentence through Markdown; inline icons keep the sentence and wrapping.
fn custom_emoji_text(s: &App, content: &str, p: Palette) -> Option<El> {
    if s.emojis.is_empty() || !content.contains(':') {
        return None;
    }
    let mut display = String::new();
    let mut icons: Vec<(usize, rune::Pixels)> = Vec::new();
    let mut found = false;
    for (index, segment) in content.split('`').enumerate() {
        if index > 0 {
            display.push('`');
        }
        if index % 2 == 1 {
            display.push_str(segment);
            continue;
        }
        let mut rest = segment;
        while let Some(start) = rest.find(':') {
            display.push_str(&rest[..start]);
            let after = &rest[start + 1..];
            let Some(end) = after.find(':') else {
                display.push(':');
                rest = after;
                continue;
            };
            let name = &after[..end];
            if let Some(emoji) = s
                .emojis
                .iter()
                .find(|emoji| !emoji.sticker && emoji.name.eq_ignore_ascii_case(name))
            {
                found = true;
                if let Some(pixels) = s.images.get(&emoji.url) {
                    let at = display.len();
                    display.push('\u{2060}');
                    icons.push((at, pixels.clone()));
                } else {
                    display.push(':');
                    display.push_str(&emoji.name);
                    display.push(':');
                }
            } else {
                display.push(':');
                display.push_str(name);
                display.push(':');
            }
            rest = &after[end + 1..];
        }
        display.push_str(rest);
    }
    if !found {
        return None;
    }
    let mut el = text(display)
        .font(theme::BODY)
        .color(p.text)
        .wrap()
        .selectable(true);
    for (at, pixels) in icons {
        el = el.inline_icon(at, rune::InlineIcon::image(pixels).size(22.0).gap(2.0));
    }
    Some(el)
}

pub(crate) fn plain_thread_name(content: &str) -> String {
    let line = content.lines().next().unwrap_or("New thread");
    let cleaned: String = line
        .chars()
        .filter(|c| !matches!(c, '*' | '_' | '`' | '~' | '|' | '>' | '#'))
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let name = collapsed.chars().take(60).collect::<String>();
    if name.trim().is_empty() {
        "New thread".to_string()
    } else {
        name
    }
}

pub(crate) fn custom_emoji_pixels<'a>(s: &'a App, token: &str) -> Option<&'a rune::Pixels> {
    let name = token.strip_prefix(':')?.strip_suffix(':')?;
    let emoji = s
        .emojis
        .iter()
        .find(|emoji| emoji.name.eq_ignore_ascii_case(name))?;
    s.images.get(&emoji.url)
}

pub(crate) fn attachment_card(s: &App, a: &runecord_protocol::Attachment, p: Palette) -> El {
    let max_w = 400.0;
    let max_h = 300.0;
    let ratio = if a.width > 0 && a.height > 0 {
        a.width as f32 / a.height as f32
    } else {
        16.0 / 9.0
    };
    let scale = (max_w / a.width.max(1) as f32)
        .min(max_h / a.height.max(1) as f32)
        .min(1.0);
    let w = (a.width.max(1) as f32 * scale).max(48.0);
    let h = (a.height.max(1) as f32 * scale).max(48.0);
    let _ = ratio;
    let image_url = crate::app::attachment_image_url(a);
    let body = if let Some(pixels) = image_url.as_ref().and_then(|url| s.images.get(url)) {
        el().w_full()
            .h_full()
            .pixels_fit(pixels, rune::SurfaceFit::Fit)
    } else {
        col()
            .w_full()
            .h_full()
            .center()
            .gap(8.0)
            .child(icon(&icons::IMAGE, 40.0).color(p.text_faint))
            .child(
                text(a.filename.clone())
                    .font(theme::SMALL)
                    .color(p.text_muted),
            )
    };
    let mut card = el()
        .key(format!("attachment-{}", a.id))
        .w(w)
        .h(h)
        .radius(8.0)
        .clip_rounded(8.0)
        .fill(p.input)
        .border(1.0, p.divider)
        .role("button")
        .child(body);
    if let Some(url) = image_url {
        let full = if a.url.is_empty() {
            url.clone()
        } else {
            a.url.clone()
        };
        let natural = (a.width.max(1) as f32, a.height.max(1) as f32);
        card = card
            .help("Open image")
            .label(format!("Image attachment {}", a.filename))
            .hover(p.input_hover)
            .on_press(move |s: &mut App| {
                s.send(crate::net::Command::FetchImage { url: full.clone() });
                s.zoom.reset();
                s.viewing_image = Some((full.clone(), natural.0, natural.1));
            });
    } else {
        card = card.label(format!("Attachment {}", a.filename));
    }
    card
}

pub(crate) fn group_label(label: &str, p: Palette) -> El {
    el().key(format!("group-{label}"))
        .w_full()
        .h(32.0)
        .px(8.0)
        .center_y()
        .child(
            text(label.to_string())
                .font(theme::SECTION)
                .color(p.text_muted),
        )
}

pub(crate) fn guild_icon(s: &App, guild: &GuildSummary, size: f32, radius: f32, p: Palette) -> El {
    let color = theme::color(&format!("#{:06x}", guild.icon_color));
    let base = el()
        .size(size, size)
        .radius(radius)
        .clip_rounded(radius)
        .fill(color);
    if let Some(pixels) = s
        .images
        .get(&guild.icon_url)
        .filter(|_| !guild.icon_url.is_empty())
    {
        return base.child(
            el().size(size, size)
                .pixels_fit(pixels, rune::SurfaceFit::Fill),
        );
    }
    let initials: String = guild
        .name
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    base.center().child(
        text(initials)
            .font(TextStyle::new((size * 0.4).clamp(10.0, 20.0), 700.0))
            .color(Color::WHITE),
    )
}

pub(crate) fn guild_banner_el(s: &App, guild: &GuildSummary, height: f32, p: Palette) -> El {
    if let Some(pixels) = s
        .images
        .get(&guild.banner_url)
        .filter(|_| !guild.banner_url.is_empty())
    {
        return el().w_full().h(height).clip_rounded(0.0).child(
            el().w_full()
                .h(height)
                .pixels_fit(pixels, rune::SurfaceFit::Fill),
        );
    }
    el().w_full()
        .h(height)
        .fill(theme::color(&format!("#{:06x}", guild.icon_color)).alpha(0.35))
}

static AVATAR_PIXELS: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<Id, rune::Pixels>>,
> = std::sync::OnceLock::new();

pub(crate) fn set_avatar_pixels(user: Id, pixels: rune::Pixels) {
    let map = AVATAR_PIXELS.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    if let Ok(mut map) = map.lock() {
        map.insert(user, pixels);
    }
}

pub(crate) fn clear_avatar_pixels(user: Id) {
    let map = AVATAR_PIXELS.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    if let Ok(mut map) = map.lock() {
        map.remove(&user);
    }
}

fn avatar_pixels(user: Id) -> Option<rune::Pixels> {
    AVATAR_PIXELS
        .get()
        .and_then(|map| map.lock().ok().and_then(|map| map.get(&user).cloned()))
}

pub(crate) const BADGE_KEYS: [(&str, &str); 7] = [
    ("staff", "Discord Staff"),
    ("partner", "Partnered Server Owner"),
    ("early_supporter", "Early Supporter"),
    ("bug_hunter", "Bug Hunter"),
    ("nitro", "Nitro"),
    ("booster", "Server Booster"),
    ("developer", "Active Developer"),
];

pub(crate) const DECORATIONS: [(&str, &str); 5] = [
    ("", "None"),
    ("aurora", "Aurora"),
    ("sakura", "Sakura"),
    ("flame", "Flame"),
    ("smoke", "Smoke"),
];

fn badge_color(key: &str) -> Color {
    theme::color(match key {
        "staff" => "#5865f2",
        "partner" => "#3ba55d",
        "early_supporter" => "#faa61a",
        "bug_hunter" => "#ed4245",
        "nitro" => "#eb459e",
        "booster" => "#ff73fa",
        "developer" => "#1abc9c",
        _ => "#747f8d",
    })
}

fn badge_icon(key: &str) -> &'static Icon {
    match key {
        "staff" => &icons::SHIELD,
        "partner" => &icons::USERS,
        "early_supporter" => &icons::SPARKLES,
        "bug_hunter" => &icons::SEARCH,
        "nitro" => &icons::SPARKLES,
        "booster" => &icons::CROWN,
        "developer" => &icons::LINK,
        _ => &icons::CHECK,
    }
}

pub(crate) fn badge_chip(key: &str, size: f32, p: Palette) -> El {
    let label = BADGE_KEYS
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, label)| *label)
        .unwrap_or("Profile badge");
    el().size(size, size)
        .radius(size * 0.22)
        .fill(badge_color(key).alpha(0.22))
        .center()
        .help(label)
        .child(icon(badge_icon(key), size * 0.66).color(badge_color(key)))
}

pub(crate) fn badges_row(badges: &[String], size: f32, p: Palette, limit: usize) -> El {
    let mut row_el = row().gap(3.0).center_y();
    for badge in badges.iter().filter(|badge| !badge.is_empty()).take(limit) {
        row_el = row_el.child(badge_chip(badge, size, p));
    }
    row_el
}

pub(crate) fn decoration_colors(key: &str) -> (Color, Color) {
    match key {
        "aurora" => (theme::color("#5865f2"), theme::color("#eb459e")),
        "sakura" => (theme::color("#ffb6c1"), theme::color("#eb459e")),
        "flame" => (theme::color("#faa61a"), theme::color("#ed4245")),
        "smoke" => (theme::color("#4e5058"), theme::color("#1e1f22")),
        _ => (Color::TRANSPARENT, Color::TRANSPARENT),
    }
}

fn avatar_visual(user: &User, size: f32, _p: Palette) -> El {
    let mut e = el().size(size, size).radius(size / 2.0);
    if let Some(pixels) = avatar_pixels(user.id) {
        e = e
            .clip_rounded(size / 2.0)
            .pixels_fit(&pixels, rune::SurfaceFit::Fill);
    } else {
        e = e
            .fill(theme::avatar_color(user.avatar_color))
            .center()
            .child(
                text(user.initials())
                    .font(TextStyle::new((size * 0.4).max(11.0), 700.0))
                    .color(Color::WHITE),
            );
    }
    if user.avatar_border != 0 {
        e = e.border(2.0, theme::color(&format!("#{:06x}", user.avatar_border)));
    }
    e
}

pub fn avatar(user: &User, size: f32, p: Palette) -> El {
    let decoration = user.avatar_decoration.as_str();
    if decoration.is_empty() || decoration == "none" {
        return avatar_visual(user, size, p)
            .key("avatar")
            .role("img")
            .label(format!("{} avatar", user.display_name));
    }
    let (top, bottom) = decoration_colors(decoration);
    el().key("avatar")
        .size(size, size)
        .radius(size / 2.0)
        .fill(top)
        .gradient(top, bottom)
        .center()
        .role("img")
        .label(format!("{} decorated avatar", user.display_name))
        .child(avatar_visual(user, (size - 6.0).max(8.0), p))
}

pub fn avatar_status(user: &User, size: f32, p: Palette, ring: Color, show_status: bool) -> El {
    let dot = (size * 0.32).max(10.0);
    el().size(size, size)
        .child(avatar(user, size, p))
        .children(show_status.then(|| {
            el().key("status-dot")
                .absolute(size - dot, size - dot)
                .size(dot, dot)
                .radius(dot / 2.0)
                .fill(theme::status_color(p, user.status))
                .border((dot * 0.22).max(2.0), ring)
        }))
}

pub(crate) fn time_label(ts: i64) -> String {
    use chrono::{Local, TimeZone};
    let Some(dt) = Local.timestamp_millis_opt(ts).single() else {
        return String::new();
    };
    if dt.date_naive() == Local::now().date_naive() {
        let t = dt.format("%I:%M %p").to_string();
        format!("Today at {}", t.trim_start_matches('0'))
    } else {
        dt.format("%m/%d/%Y %-I:%M %p").to_string()
    }
}

pub(crate) fn same_day(a: i64, b: i64) -> bool {
    use chrono::{Local, TimeZone};
    let Some(a) = Local.timestamp_millis_opt(a).single() else {
        return false;
    };
    let Some(b) = Local.timestamp_millis_opt(b).single() else {
        return false;
    };
    a.date_naive() == b.date_naive()
}

pub(crate) fn day_divider_label(ts: i64) -> String {
    use chrono::{Local, TimeZone};
    let Some(dt) = Local.timestamp_millis_opt(ts).single() else {
        return String::new();
    };
    dt.format("%B %-d, %Y").to_string()
}

pub(crate) fn expiry_label(ts: i64) -> String {
    use chrono::{Datelike, Local, TimeZone};
    let Some(dt) = Local.timestamp_millis_opt(ts).single() else {
        return String::new();
    };
    let today = Local::now().date_naive();
    let time = dt.format("%I:%M %p").to_string();
    let time = time.trim_start_matches('0');
    if dt.date_naive() == today {
        format!("Today at {time}")
    } else if dt.date_naive().num_days_from_ce() == today.num_days_from_ce() + 1 {
        format!("Tomorrow at {time}")
    } else {
        dt.format("%b %-d at %I:%M %p")
            .to_string()
            .replace(" 0", " ")
    }
}

pub(crate) fn date_label(ts: i64) -> String {
    use chrono::{Local, TimeZone};
    let Some(dt) = Local.timestamp_millis_opt(ts).single() else {
        return String::new();
    };
    dt.format("%b %-d, %Y").to_string()
}

pub(crate) fn toast(t: &Toast, p: Palette) -> El {
    col()
        .key("toast-layer")
        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
        .items_center()
        .justify_end()
        .pb(80.0)
        .inert()
        .child(
            row()
                .key("toast")
                .radius(8.0)
                .fill(theme::color("#111214"))
                .px(16.0)
                .py(12.0)
                .role("status")
                .live_region(rune::LiveRegion::Polite)
                .shadow(4.0, 16.0, p.shadow)
                .child(text(t.text.clone()).font(theme::SMALL).color(Color::WHITE)),
        )
}

// ---------------------------------------------------------------------------
// Modals
// ---------------------------------------------------------------------------

pub(crate) fn close_button(p: Palette) -> El {
    el().key("modal-close")
        .size(32.0, 32.0)
        .radius(4.0)
        .center()
        .role("button")
        .label("Close")
        .help("Close")
        .hover(p.rail)
        .on_press(|s: &mut App| s.modal = Modal::None)
        .child(icon(&icons::CLOSE, 20.0).color(p.text_muted))
}

pub(crate) fn developer_bar(s: &App, p: Palette) -> El {
    let messages: usize = s.messages.values().map(Vec::len).sum();
    let channels = s.channels.values().map(Vec::len).sum::<usize>();
    let members: usize = s.members.values().map(Vec::len).sum();
    let images_mb = s.image_bytes as f64 / (1024.0 * 1024.0);
    let cache_age = if s.last_cache_save < 0.0 {
        "never".to_string()
    } else {
        format!("{:.0}s", (s.clock - s.last_cache_save).max(0.0))
    };
    let gateway = if s.connected {
        "online"
    } else if s.connecting {
        "connecting"
    } else {
        "offline"
    };
    row()
        .key("developer-bar")
        .w_full()
        .h(24.0)
        .px(12.0)
        .gap(16.0)
        .center_y()
        .fill(p.rail)
        .border_sides([1.0, 0.0, 0.0, 0.0], p.divider)
        .role("status")
        .live_region(rune::LiveRegion::Polite)
        .child(text("DEV").font(theme::LABEL).color(p.yellow))
        .child(
            text(format!(
                "ws {gateway} · guilds {} · channels {channels} · msgs {messages}/5000 · members {members} · details {}/6 · images {} ({images_mb:.1} MB) · emojis {} · cache {cache_age}",
                s.guilds.len(),
                s.guild_details.len(),
                s.images.len(),
                s.emojis.len()
            ))
            .font(TextStyle::new(11.0, 400.0).mono())            .color(p.text_muted)
            .grow(1.0)
            .min_w(0.0)
            .truncate(),
        )
        .child(
            text(format!("profile {}", s.cache_profile))
                .font(TextStyle::new(11.0, 400.0).mono())
                .color(p.text_faint),
        )
}

fn skeleton_alpha(clock: f64) -> f32 {
    0.45 + 0.3 * ((clock * 2.2).sin() * 0.5 + 0.5) as f32
}

pub(crate) fn skeleton_bar(width: f32, height: f32, p: Palette, clock: f64) -> El {
    el().w(width)
        .h(height)
        .radius(height / 2.0)
        .fill(p.rail_hover.alpha(skeleton_alpha(clock)))
}

pub(crate) fn skeleton_channel_rows(count: usize, p: Palette, clock: f64) -> El {
    let mut rows = col().w_full().gap(2.0);
    for index in 0..count {
        let label = 70.0 + ((index * 29) % 70) as f32;
        rows = rows.child(
            row()
                .w_full()
                .h(34.0)
                .px(10.0)
                .gap(8.0)
                .center_y()
                .child(
                    el().size(18.0, 18.0)
                        .radius(9.0)
                        .fill(p.rail_hover.alpha(skeleton_alpha(clock))),
                )
                .child(skeleton_bar(label, 9.0, p, clock)),
        );
    }
    rows
}

pub(crate) fn skeleton_member_rows(count: usize, p: Palette, clock: f64) -> El {
    let mut rows = col().w_full().gap(2.0);
    for index in 0..count {
        let label = 80.0 + ((index * 23) % 60) as f32;
        rows = rows.child(
            row()
                .w_full()
                .h(46.0)
                .px(10.0)
                .gap(12.0)
                .center_y()
                .child(
                    el().size(32.0, 32.0)
                        .radius(16.0)
                        .fill(p.rail_hover.alpha(skeleton_alpha(clock))),
                )
                .child(
                    col()
                        .grow(1.0)
                        .gap(6.0)
                        .child(skeleton_bar(label, 9.0, p, clock))
                        .child(skeleton_bar(label * 0.7, 7.0, p, clock)),
                ),
        );
    }
    rows
}

pub(crate) fn skeleton_message_rows(count: usize, p: Palette, clock: f64) -> El {
    let mut rows = col().w_full().gap(18.0).padding(16.0);
    for index in 0..count {
        let name = 90.0 + ((index * 31) % 70) as f32;
        let body = 320.0 + ((index * 61) % 190) as f32;
        rows = rows.child(
            row()
                .w_full()
                .gap(12.0)
                .child(
                    el().size(40.0, 40.0)
                        .radius(20.0)
                        .fill(p.rail_hover.alpha(skeleton_alpha(clock))),
                )
                .child(
                    col()
                        .grow(1.0)
                        .gap(7.0)
                        .child(
                            row()
                                .gap(8.0)
                                .center_y()
                                .child(skeleton_bar(name, 10.0, p, clock))
                                .child(skeleton_bar(54.0, 8.0, p, clock)),
                        )
                        .child(skeleton_bar(body, 9.0, p, clock))
                        .child(skeleton_bar(body * 0.55, 9.0, p, clock)),
                ),
        );
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::{day_divider_label, same_day};

    #[test]
    fn day_dividers_group_messages_by_local_day() {
        use chrono::Local;
        let now = Local::now().timestamp_millis();
        assert!(same_day(now, now + 1_000));
        assert!(!same_day(now, now + 2 * 86_400_000));
        assert!(!day_divider_label(now).is_empty());
    }
}
