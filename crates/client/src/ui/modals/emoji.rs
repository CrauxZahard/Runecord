//! Emoji and reaction picker.

use super::super::*;

const CATEGORIES: &[&str] = &[
    "Frequently Used",
    "Smileys & People",
    "Nature",
    "Food & Drink",
    "Activity",
    "Travel & Places",
    "Objects",
    "Symbols",
];

const EMOJIS: &[(&str, &str, &str)] = &[
    ("😀", "grinning", "Smileys & People"),
    ("😂", "joy", "Smileys & People"),
    ("🥹", "pleading", "Smileys & People"),
    ("😍", "heart eyes", "Smileys & People"),
    ("😎", "sunglasses", "Smileys & People"),
    ("🤔", "thinking", "Smileys & People"),
    ("😴", "sleeping", "Smileys & People"),
    ("🥳", "partying", "Smileys & People"),
    ("😭", "sob", "Smileys & People"),
    ("😅", "sweat smile", "Smileys & People"),
    ("🙏", "pray", "Smileys & People"),
    ("👍", "thumbs up", "Smileys & People"),
    ("👎", "thumbs down", "Smileys & People"),
    ("💯", "hundred", "Smileys & People"),
    ("👀", "eyes", "Smileys & People"),
    ("🔥", "fire", "Nature"),
    ("⚡", "zap", "Nature"),
    ("🌊", "wave", "Nature"),
    ("🌙", "moon", "Nature"),
    ("🐛", "bug", "Nature"),
    ("🦀", "crab", "Nature"),
    ("🍕", "pizza", "Food & Drink"),
    ("🍔", "burger", "Food & Drink"),
    ("☕", "coffee", "Food & Drink"),
    ("🍺", "beer", "Food & Drink"),
    ("🎉", "tada", "Activity"),
    ("🎨", "art", "Activity"),
    ("🎧", "headphones", "Activity"),
    ("🎵", "music", "Activity"),
    ("🎮", "video game", "Activity"),
    ("🚀", "rocket", "Travel & Places"),
    ("🌍", "earth", "Travel & Places"),
    ("📌", "pushpin", "Objects"),
    ("📷", "camera", "Objects"),
    ("💻", "laptop", "Objects"),
    ("💡", "bulb", "Objects"),
    ("🧠", "brain", "Objects"),
    ("✨", "sparkles", "Symbols"),
    ("💜", "purple heart", "Symbols"),
    ("✅", "check", "Symbols"),
    ("❌", "x", "Symbols"),
];

pub(crate) fn emoji_sheet(s: &App, p: Palette, target: Option<Id>) -> El {
    let query = s.emoji_search.text.trim().to_lowercase();
    let matches = |name: &str| query.is_empty() || name.to_lowercase().contains(&query);
    let category = s.emoji_category.min(CATEGORIES.len() - 1);
    let category_name = CATEGORIES[category];

    let mut grid = row().w_full().gap(2.0).flex_wrap();
    let mut count = 0;
    for (emoji, name, _group) in EMOJIS.iter().filter(|(_, name, group)| {
        matches(name) && (!query.is_empty() || category == 0 || *group == category_name)
    }) {
        let label = emoji.to_string();
        let insert = label.clone();
        let description = *name;
        grid = grid.child(
            el().size(40.0, 40.0)
                .radius(6.0)
                .center()
                .key(format!("react-{label}"))
                .role("button")
                .label(if target.is_some() {
                    format!("React with {description}")
                } else {
                    format!("Insert {description}")
                })
                .help(*name)
                .hover(p.selected)
                .on_press(move |s: &mut App| {
                    if let Some(message) = target {
                        s.react(message, &insert);
                    } else {
                        s.composer.insert(&insert);
                    }
                    s.modal = Modal::None;
                })
                .child(text(label).font(TextStyle::new(26.0, 400.0)).color(p.text)),
        );
        count += 1;
    }

    let mut custom = row().w_full().gap(2.0).flex_wrap();
    let mut custom_count = 0;
    for emoji in s
        .emojis
        .iter()
        .filter(|emoji| !emoji.sticker && matches(&emoji.name))
    {
        let name = emoji.name.clone();
        let token = format!(":{name}:");
        let url = emoji.url.clone();
        let body = if let Some(pixels) = s.images.get(&url) {
            el().size(30.0, 30.0)
                .pixels_fit(pixels, rune::SurfaceFit::Fit)
        } else {
            text(name.clone()).font(theme::TINY).color(p.text_faint)
        };
        custom = custom.child(
            el().size(40.0, 40.0)
                .radius(6.0)
                .center()
                .key(format!("custom-emoji-{}", emoji.id))
                .role("button")
                .label(format!("Insert {token}"))
                .help(token.clone())
                .hover(p.selected)
                .on_press(move |s: &mut App| {
                    if let Some(message) = target {
                        s.react(message, &token);
                    } else {
                        s.composer.insert(&token);
                    }
                    s.modal = Modal::None;
                })
                .child(body),
        );
        custom_count += 1;
    }

    let mut body = col().w_full().gap(6.0);
    if count > 0 {
        body = body
            .child(
                text(if !query.is_empty() {
                    "SEARCH RESULTS".to_string()
                } else {
                    category_name.to_uppercase()
                })
                .font(theme::SECTION)
                .color(p.text_muted)
                .padding(4.0),
            )
            .child(grid);
    }
    if custom_count > 0 {
        body = body
            .child(
                text("THIS SERVER")
                    .font(theme::SECTION)
                    .color(p.text_muted)
                    .padding(4.0)
                    .pt(8.0),
            )
            .child(custom);
    }
    if count == 0 && custom_count == 0 {
        body = body.child(
            col()
                .w_full()
                .py(36.0)
                .items_center()
                .gap(6.0)
                .child(
                    text("NO EMOJI FOUND")
                        .font(theme::SMALL_MED)
                        .color(p.text_bright),
                )
                .child(
                    text("Try a different search.")
                        .font(theme::TINY)
                        .color(p.text_faint),
                ),
        );
    }

    let searching = !s.emoji_search.text.is_empty();
    let glyphs = ["🕘", "😀", "🌿", "🍔", "🎮", "🌍", "💡", "✅"];
    let mut categories = row().w_full().h(32.0).gap(2.0).center();
    for (index, glyph) in glyphs.iter().enumerate() {
        let selected = s.emoji_category == index;
        categories = categories.child(
            el().size(32.0, 32.0)
                .radius(6.0)
                .center()
                .key(format!("emoji-cat-{index}"))
                .role("tab")
                .label(CATEGORIES[index])
                .fill(if selected {
                    p.selected
                } else {
                    Color::TRANSPARENT
                })
                .hover(p.input_hover)
                .on_press(move |s: &mut App| {
                    s.emoji_search.clear();
                    s.emoji_category = index;
                })
                .child(
                    text(*glyph)
                        .font(TextStyle::new(18.0, 400.0))
                        .color(if selected {
                            p.text_bright
                        } else {
                            p.text_muted
                        }),
                ),
        );
    }
    col()
        .key("emoji-sheet")
        .w(408.0)
        .radius(10.0)
        .fill(theme::color("#111214"))
        .padding(12.0)
        .gap(10.0)
        .shadow(12.0, 32.0, p.shadow)
        .modal(true, |s: &mut App| s.modal = Modal::None)
        .child(
            row()
                .w_full()
                .items_center()
                .gap(8.0)
                .child(
                    text(if target.is_some() {
                        "Add Reaction"
                    } else {
                        "Emoji"
                    })
                    .font(theme::BODY_BOLD)
                    .color(p.text_bright)
                    .grow(1.0),
                )
                .children(searching.then(|| {
                    el().key("emoji-search-clear")
                        .h(24.0)
                        .px(8.0)
                        .radius(4.0)
                        .center()
                        .role("button")
                        .label("Clear emoji search")
                        .hover(p.selected)
                        .on_press(|s: &mut App| s.emoji_search.clear())
                        .child(text("Clear").font(theme::TINY).color(p.text_muted))
                })),
        )
        .child(
            row()
                .w_full()
                .h(36.0)
                .radius(4.0)
                .fill(p.input)
                .px(8.0)
                .gap(6.0)
                .center_y()
                .child(icon(&icons::SEARCH, 16.0).color(p.text_faint))
                .child(
                    text_input(&s.emoji_search, theme::SMALL)
                        .key("emoji-search")
                        .grow(1.0)
                        .min_w(0.0)
                        .h(28.0)
                        .color(p.text)
                        .role("textbox")
                        .label("Search emoji")
                        .placeholder("Search emoji", p.text_faint)
                        .padding(0.0)
                        .on_edit(|s: &mut App| &mut s.emoji_search),
                ),
        )
        .child(
            col()
                .key("emoji-scroll")
                .w_full()
                .max_h(360.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(body),
        )
        .child(categories)
}
