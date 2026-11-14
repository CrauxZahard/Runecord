//! A small icon catalog in Discord's visual language. Every icon is a 24x24 body drawn in the
//! element's colour, so `.color()` tints it.

use rune::Icon;

macro_rules! stroke_icon {
    ($name:ident, $key:literal, $body:literal) => {
        pub static $name: Icon = Icon {
            name: $key,
            view: 24.0,
            body: concat!(
                r#"<g fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">"#,
                $body,
                "</g>"
            ),
        };
    };
}

stroke_icon!(
    HASH,
    "hash",
    r#"<path d="M6 3 4.5 21M13.5 3 12 21M3 8.5h18M2 15.5h18"/>"#
);
stroke_icon!(CHEVRON_DOWN, "chevron-down", r#"<path d="m6 9 6 6 6-6"/>"#);
stroke_icon!(
    CHEVRON_RIGHT,
    "chevron-right",
    r#"<path d="m9 6 6 6-6 6"/>"#
);
stroke_icon!(CHEVRON_LEFT, "chevron-left", r#"<path d="m15 6-6 6 6 6"/>"#);
stroke_icon!(PLUS, "plus", r#"<path d="M12 5v14M5 12h14"/>"#);
stroke_icon!(CLOSE, "close", r#"<path d="M6 6l12 12M18 6 6 18"/>"#);
stroke_icon!(CHECK, "check", r#"<path d="m5 12 5 5L20 7"/>"#);
stroke_icon!(
    SEARCH,
    "search",
    r#"<circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/>"#
);
stroke_icon!(
    COMPASS,
    "compass",
    r#"<circle cx="12" cy="12" r="9"/><path d="m16 8-2.5 5.5L8 16l2.5-5.5Z"/>"#
);
stroke_icon!(
    SETTINGS,
    "settings",
    r#"<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .34 1.87l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.7 1.7 0 0 0-1.87-.34 1.7 1.7 0 0 0-1 1.55V21a2 2 0 1 1-4 0v-.09a1.7 1.7 0 0 0-1.11-1.55 1.7 1.7 0 0 0-1.87.34l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.7 1.7 0 0 0 4.6 15a1.7 1.7 0 0 0-1.55-1H3a2 2 0 1 1 0-4h.09A1.7 1.7 0 0 0 4.64 8.9a1.7 1.7 0 0 0-.34-1.87l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.7 1.7 0 0 0 9 4.6a1.7 1.7 0 0 0 1-1.55V3a2 2 0 1 1 4 0v.09a1.7 1.7 0 0 0 1 1.55 1.7 1.7 0 0 0 1.87-.34l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.7 1.7 0 0 0 19.4 9v.09a1.7 1.7 0 0 0 1.55 1H21a2 2 0 1 1 0 4h-.09a1.7 1.7 0 0 0-1.51 1Z"/>"#
);
stroke_icon!(
    HOME,
    "home",
    r#"<path d="m3 11 9-8 9 8"/><path d="M5 10v10h14V10"/>"#
);
stroke_icon!(
    MIC,
    "mic",
    r#"<rect x="9" y="2.5" width="6" height="12" rx="3"/><path d="M5 11a7 7 0 0 0 14 0M12 18v3"/>"#
);
stroke_icon!(
    HEADPHONES,
    "headphones",
    r#"<path d="M4 15v-3a8 8 0 0 1 16 0v3"/><rect x="3" y="14" width="4" height="7" rx="2"/><rect x="17" y="14" width="4" height="7" rx="2"/>"#
);
stroke_icon!(
    BELL,
    "bell",
    r#"<path d="M6 16v-5a6 6 0 1 1 12 0v5l2 2H4Z"/><path d="M10 21a2 2 0 0 0 4 0"/>"#
);
stroke_icon!(
    PIN,
    "pin",
    r#"<path d="M9 3h6l-1 6 3 3H7l3-3Z"/><path d="M12 12v9"/>"#
);
stroke_icon!(
    PHONE,
    "phone",
    r#"<path d="M5 3h4l2 5-3 2a13 13 0 0 0 6 6l2-3 5 2v4a2 2 0 0 1-2 2A17 17 0 0 1 3 5a2 2 0 0 1 2-2Z"/>"#
);
stroke_icon!(
    CHAT,
    "chat",
    r#"<path d="M21 11.5a8.5 8.5 0 0 1-8.5 8.5H4l2.5-3A8.5 8.5 0 1 1 21 11.5Z"/>"#
);
stroke_icon!(
    VIDEO,
    "video",
    r#"<rect x="3" y="6" width="12" height="12" rx="2"/><path d="m15 10 6-3v10l-6-3Z"/>"#
);
stroke_icon!(
    BELL_OFF,
    "bell-off",
    r#"<path d="M3 3l18 18"/><path d="M18 8a6 6 0 0 0-9.3-5"/><path d="M6 9v3l-2 4h12"/><path d="M10 21h4"/>"#
);
stroke_icon!(
    CLOCK,
    "clock",
    r#"<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>"#
);
stroke_icon!(
    USERS,
    "users",
    r#"<circle cx="9" cy="8" r="3.2"/><path d="M2.5 20a6.5 6.5 0 0 1 13 0"/><path d="M16 5.5a3.2 3.2 0 0 1 0 5.9M18 14.5A5.5 5.5 0 0 1 22 20"/>"#
);
stroke_icon!(
    INBOX,
    "inbox",
    r#"<path d="M3 4.5h18v15H3Z"/><path d="M3 12h5l2 3h4l2-3h5"/>"#
);
stroke_icon!(
    HELP,
    "help",
    r#"<circle cx="12" cy="12" r="9"/><path d="M9.5 9a2.6 2.6 0 1 1 3.4 2.5c-.8.3-.9 1-.9 1.7"/><circle cx="12" cy="17" r=".6" fill="currentColor"/>"#
);
stroke_icon!(
    GIFT,
    "gift",
    r#"<rect x="3" y="8" width="18" height="13" rx="1.5"/><path d="M3 12h18M12 8v13"/><path d="M8 8a2.5 2.5 0 1 1 0-5c2 0 4 5 4 5s2-5 4-5a2.5 2.5 0 1 1 0 5"/>"#
);
stroke_icon!(
    SMILE,
    "smile",
    r#"<circle cx="12" cy="12" r="9"/><path d="M8.5 14.5a4.5 4.5 0 0 0 7 0"/><circle cx="9" cy="10" r=".8" fill="currentColor"/><circle cx="15" cy="10" r=".8" fill="currentColor"/>"#
);
stroke_icon!(
    SEND,
    "send",
    r#"<path d="M21 3 3 10.5l7 2.5 2.5 7Z"/><path d="M21 3 10 13"/>"#
);
stroke_icon!(
    REPLY,
    "reply",
    r#"<path d="M9 7 3 13l6 6"/><path d="M3 13h10a7 7 0 0 1 7 7v1"/>"#
);
stroke_icon!(
    EDIT,
    "edit",
    r#"<path d="M4 20h4L20 8l-4-4L4 16Z"/><path d="M14 6l4 4"/>"#
);
stroke_icon!(
    TRASH,
    "trash",
    r#"<path d="M4 7h16M9 7V4h6v3M6 7l1 14h10l1-14"/><path d="M10 11v6M14 11v6"/>"#
);
stroke_icon!(
    THREAD,
    "thread",
    r#"<path d="M4 4h16v11H9l-5 4Z"/><path d="M8 9h8M8 12h5"/>"#
);
stroke_icon!(
    IMAGE,
    "image",
    r#"<rect x="3" y="4" width="18" height="16" rx="2"/><circle cx="8.5" cy="9" r="1.5"/><path d="m4 18 5-5 4 4 3-3 4 4"/>"#
);
stroke_icon!(
    DOWNLOAD,
    "download",
    r#"<path d="M12 3v12M7 10l5 5 5-5"/><path d="M3 20h18"/>"#
);
stroke_icon!(
    LOCK,
    "lock",
    r#"<rect x="4" y="10" width="16" height="11" rx="2"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/>"#
);
stroke_icon!(
    SHIELD,
    "shield",
    r#"<path d="M12 2 4 5v7c0 5 8 10 8 10s8-5 8-10V5Z"/><path d="m8.5 12 2.5 2.5 4.5-5"/>"#
);
stroke_icon!(
    CROWN,
    "crown",
    r#"<path d="M3 7l4 4 5-7 5 7 4-4-2 12H5Z"/>"#
);
stroke_icon!(
    LINK,
    "link",
    r#"<path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1.2 1.2"/><path d="M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1.2-1.2"/>"#
);
stroke_icon!(
    MONITOR,
    "monitor",
    r#"<rect x="2" y="4" width="20" height="13" rx="2"/><path d="M8 21h8M12 17v4"/>"#
);
stroke_icon!(
    SPARKLES,
    "sparkles",
    r#"<path d="M12 3l1.6 4.4L18 9l-4.4 1.6L12 15l-1.6-4.4L6 9l4.4-1.6Z"/><path d="M19 15l.8 2.2L22 18l-2.2.8L19 21l-.8-2.2L16 18l2.2-.8Z"/>"#
);
stroke_icon!(
    VOICE,
    "voice",
    r#"<path d="M11 5 6 9H3v6h3l5 4Z" fill="currentColor"/><path d="M15 8.5a5 5 0 0 1 0 7M18 5a9 9 0 0 1 0 14"/>"#
);
stroke_icon!(
    MEGAPHONE,
    "megaphone",
    r#"<path d="M4 10v4h3l5 4V6L7 10Z" fill="currentColor"/><path d="M15 8a5 5 0 0 1 0 8M18 5a9 9 0 0 1 0 14"/>"#
);
stroke_icon!(
    FORUM,
    "forum",
    r#"<path d="M3 5h13v8H8l-5 4Z"/><path d="M17 8h4v9h-4"/>"#
);
stroke_icon!(
    ADD_FRIEND,
    "add-friend",
    r#"<circle cx="9" cy="8" r="3.5"/><path d="M2.5 20a6.5 6.5 0 0 1 13 0"/><path d="M19 8v6M16 11h6"/>"#
);
stroke_icon!(
    SLASH,
    "slash",
    r#"<circle cx="12" cy="12" r="9"/><path d="M8 17 16 7"/>"#
);
stroke_icon!(FOLDER, "folder", r#"<path d="M3 6h6l2 2h10v11H3Z"/>"#);

// The Discord mark, approximated as a round-faced mark with two eyes cut out.
pub static DISCORD: Icon = Icon {
    name: "discord",
    view: 24.0,
    body: r#"<path fill="currentColor" fill-rule="evenodd" d="M6.2 5.4c3.9-1.6 7.7-1.6 11.6 0 1.7 3.1 2.6 6.6 2.4 10.4-2 1.9-4 2.3-5.6 1.4l-.8-1.5c-1.5.5-3.1.5-4.6 0l-.8 1.5c-1.6.9-3.6.5-5.6-1.4-.2-3.8.7-7.3 2.4-10.4Zm2.9 3.7a1.75 1.75 0 1 0 0 3.5 1.75 1.75 0 0 0 0-3.5Zm5.8 0a1.75 1.75 0 1 0 0 3.5 1.75 1.75 0 0 0 0-3.5Z"/>"#,
};

pub static MORE: Icon = Icon {
    name: "more",
    view: 24.0,
    body: r#"<g fill="currentColor"><circle cx="5" cy="12" r="1.8"/><circle cx="12" cy="12" r="1.8"/><circle cx="19" cy="12" r="1.8"/></g>"#,
};

pub static DOT: Icon = Icon {
    name: "dot",
    view: 24.0,
    body: r#"<g fill="currentColor"><circle cx="12" cy="12" r="5"/></g>"#,
};

pub static STICKER: Icon = Icon {
    name: "sticker",
    view: 24.0,
    body: r#"<path fill="currentColor" d="M4 4h11l5 5v11H4Zm11 0v5h5M8 13h3M8 16h6" fill-rule="evenodd"/>"#,
};
