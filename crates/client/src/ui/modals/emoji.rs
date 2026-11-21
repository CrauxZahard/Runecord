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
    // Smileys & People
    ("😀", "grinning", "Smileys & People"),
    ("😃", "smile", "Smileys & People"),
    ("😄", "happy", "Smileys & People"),
    ("😁", "beaming", "Smileys & People"),
    ("😆", "laughing", "Smileys & People"),
    ("😅", "sweat smile", "Smileys & People"),
    ("😂", "joy", "Smileys & People"),
    ("🤣", "rolling on the floor", "Smileys & People"),
    ("🙂", "slight smile", "Smileys & People"),
    ("🙃", "upside down", "Smileys & People"),
    ("😉", "wink", "Smileys & People"),
    ("😊", "blush", "Smileys & People"),
    ("😍", "heart eyes", "Smileys & People"),
    ("🥰", "smiling with hearts", "Smileys & People"),
    ("😘", "kiss", "Smileys & People"),
    ("😋", "yum", "Smileys & People"),
    ("😜", "tongue wink", "Smileys & People"),
    ("🤪", "zany", "Smileys & People"),
    ("😎", "sunglasses", "Smileys & People"),
    ("🥳", "partying", "Smileys & People"),
    ("🤔", "thinking", "Smileys & People"),
    ("🤨", "raised eyebrow", "Smileys & People"),
    ("😐", "neutral", "Smileys & People"),
    ("😑", "expressionless", "Smileys & People"),
    ("😶", "no mouth", "Smileys & People"),
    ("🙄", "eye roll", "Smileys & People"),
    ("😏", "smirk", "Smileys & People"),
    ("😴", "sleeping", "Smileys & People"),
    ("🥺", "pleading", "Smileys & People"),
    ("🥹", "teary smile", "Smileys & People"),
    ("😭", "sob", "Smileys & People"),
    ("😢", "sad", "Smileys & People"),
    ("😤", "triumph", "Smileys & People"),
    ("😡", "angry", "Smileys & People"),
    ("🤯", "mind blown", "Smileys & People"),
    ("😱", "scream", "Smileys & People"),
    ("🥵", "hot", "Smileys & People"),
    ("😇", "angel", "Smileys & People"),
    ("🤓", "nerd", "Smileys & People"),
    ("🧐", "monocle", "Smileys & People"),
    ("🤠", "cowboy", "Smileys & People"),
    ("🥸", "disguised", "Smileys & People"),
    ("😈", "devil", "Smileys & People"),
    ("💀", "skull", "Smileys & People"),
    ("👻", "ghost", "Smileys & People"),
    ("👽", "alien", "Smileys & People"),
    ("🤖", "robot", "Smileys & People"),
    ("🙌", "raised hands", "Smileys & People"),
    ("👏", "clap", "Smileys & People"),
    ("👋", "wave hand", "Smileys & People"),
    ("🤝", "handshake", "Smileys & People"),
    ("🙏", "pray", "Smileys & People"),
    ("👍", "thumbs up", "Smileys & People"),
    ("👎", "thumbs down", "Smileys & People"),
    ("✌️", "victory", "Smileys & People"),
    ("🤞", "fingers crossed", "Smileys & People"),
    ("🤙", "call me", "Smileys & People"),
    ("👊", "fist bump", "Smileys & People"),
    ("💪", "muscle", "Smileys & People"),
    ("🫡", "salute", "Smileys & People"),
    ("🫶", "heart hands", "Smileys & People"),
    ("👀", "eyes", "Smileys & People"),
    ("💯", "hundred", "Smileys & People"),
    // Nature
    ("🔥", "fire", "Nature"),
    ("⚡", "zap", "Nature"),
    ("🌊", "wave", "Nature"),
    ("🌙", "moon", "Nature"),
    ("☀️", "sun", "Nature"),
    ("🌈", "rainbow", "Nature"),
    ("⭐", "star", "Nature"),
    ("🌟", "glowing star", "Nature"),
    ("❄️", "snowflake", "Nature"),
    ("🌸", "blossom", "Nature"),
    ("🌻", "sunflower", "Nature"),
    ("🌲", "tree", "Nature"),
    ("🐛", "bug", "Nature"),
    ("🦀", "crab", "Nature"),
    ("🐶", "dog", "Nature"),
    ("🐱", "cat", "Nature"),
    ("🦊", "fox", "Nature"),
    ("🐼", "panda", "Nature"),
    ("🐧", "penguin", "Nature"),
    ("🦄", "unicorn", "Nature"),
    ("🐝", "bee", "Nature"),
    ("🦋", "butterfly", "Nature"),
    // Food & Drink
    ("🍕", "pizza", "Food & Drink"),
    ("🍔", "burger", "Food & Drink"),
    ("🌮", "taco", "Food & Drink"),
    ("🍣", "sushi", "Food & Drink"),
    ("🍜", "ramen", "Food & Drink"),
    ("🍦", "ice cream", "Food & Drink"),
    ("🍩", "donut", "Food & Drink"),
    ("🍪", "cookie", "Food & Drink"),
    ("☕", "coffee", "Food & Drink"),
    ("🍵", "tea", "Food & Drink"),
    ("🍺", "beer", "Food & Drink"),
    ("🍻", "cheers", "Food & Drink"),
    ("🥂", "champagne", "Food & Drink"),
    // Activity
    ("🎉", "tada", "Activity"),
    ("🎊", "confetti", "Activity"),
    ("🎨", "art", "Activity"),
    ("🎧", "headphones", "Activity"),
    ("🎵", "music", "Activity"),
    ("🎶", "notes", "Activity"),
    ("🎮", "video game", "Activity"),
    ("🎲", "dice", "Activity"),
    ("🏆", "trophy", "Activity"),
    ("⚽", "football", "Activity"),
    ("🏀", "basketball", "Activity"),
    // Travel & Places
    ("🚀", "rocket", "Travel & Places"),
    ("✈️", "airplane", "Travel & Places"),
    ("🚗", "car", "Travel & Places"),
    ("🚲", "bicycle", "Travel & Places"),
    ("🌍", "earth", "Travel & Places"),
    ("🏠", "house", "Travel & Places"),
    ("🏔️", "mountain", "Travel & Places"),
    // Objects
    ("📌", "pushpin", "Objects"),
    ("📷", "camera", "Objects"),
    ("💻", "laptop", "Objects"),
    ("📱", "phone", "Objects"),
    ("💡", "bulb", "Objects"),
    ("🧠", "brain", "Objects"),
    ("📚", "books", "Objects"),
    ("✏️", "pencil", "Objects"),
    ("🔒", "lock", "Objects"),
    ("🔑", "key", "Objects"),
    ("⏰", "alarm", "Objects"),
    ("💰", "money", "Objects"),
    ("🎁", "gift", "Objects"),
    ("☕", "coffee", "Objects"),
    // Symbols
    ("✨", "sparkles", "Symbols"),
    ("💜", "purple heart", "Symbols"),
    ("❤️", "red heart", "Symbols"),
    ("🧡", "orange heart", "Symbols"),
    ("💛", "yellow heart", "Symbols"),
    ("💚", "green heart", "Symbols"),
    ("💙", "blue heart", "Symbols"),
    ("💖", "sparkling heart", "Symbols"),
    ("✅", "check", "Symbols"),
    ("❌", "x", "Symbols"),
    ("⚠️", "warning", "Symbols"),
    ("❓", "question", "Symbols"),
    ("❗", "exclamation", "Symbols"),
    ("💬", "speech bubble", "Symbols"),
    ("🔔", "bell", "Symbols"),
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
