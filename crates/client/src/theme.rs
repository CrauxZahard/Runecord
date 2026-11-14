//! Discord's palette and type scale, as tokens.

use rune::{Color, TextStyle};

pub fn color(hex: &str) -> Color {
    rune::color::hex(hex).unwrap_or(Color::BLACK)
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)] // full token set, even where a surface has not used a token yet
pub struct Palette {
    pub dark: bool,
    pub rail: Color,
    pub rail_hover: Color,
    pub sidebar: Color,
    pub sidebar_hover: Color,
    pub sidebar_active: Color,
    pub user_panel: Color,
    pub chat: Color,
    pub chat_hover: Color,
    pub header: Color,
    pub input: Color,
    pub input_hover: Color,
    pub modal: Color,
    pub modal_sidebar: Color,
    pub modal_footer: Color,
    pub card: Color,
    pub card_hover: Color,
    pub text: Color,
    pub text_bright: Color,
    pub text_muted: Color,
    pub text_faint: Color,
    pub text_link: Color,
    pub interactive: Color,
    pub divider: Color,
    pub divider_soft: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_text: Color,
    pub danger: Color,
    pub danger_hover: Color,
    pub green: Color,
    pub yellow: Color,
    pub scrim: Color,
    pub mention: Color,
    pub mention_hover: Color,
    pub selected: Color,
    pub shadow: Color,
    pub code_bg: Color,
}

impl Palette {
    pub fn dark() -> Self {
        Palette {
            dark: true,
            rail: color("#1e1f22"),
            rail_hover: color("#35373c"),
            sidebar: color("#2b2d31"),
            sidebar_hover: color("#35373c"),
            sidebar_active: color("#404249"),
            user_panel: color("#232428"),
            chat: color("#313338"),
            chat_hover: color("#2e3035"),
            header: color("#313338"),
            input: color("#383a40"),
            input_hover: color("#3f4147"),
            modal: color("#313338"),
            modal_sidebar: color("#2b2d31"),
            modal_footer: color("#2b2d31"),
            card: color("#2b2d31"),
            card_hover: color("#35373c"),
            text: color("#dbdee1"),
            text_bright: color("#f2f3f5"),
            text_muted: color("#949ba4"),
            text_faint: color("#80848e"),
            text_link: color("#00a8fc"),
            interactive: color("#b5bac1"),
            divider: color("#3f4147"),
            divider_soft: color("#26272b"),
            accent: color("#5865f2"),
            accent_hover: color("#4752c4"),
            accent_text: color("#ffffff"),
            danger: color("#f23f43"),
            danger_hover: color("#da373c"),
            green: color("#23a55a"),
            yellow: color("#f0b232"),
            scrim: color("#000000").alpha(0.5),
            mention: color("#414675"),
            mention_hover: color("#5865f2"),
            selected: color("#404249"),
            shadow: color("#000000").alpha(0.3),
            code_bg: color("#2b2d31"),
        }
    }

    pub fn light() -> Self {
        Palette {
            dark: false,
            rail: color("#e3e5e8"),
            rail_hover: color("#d7d9dc"),
            sidebar: color("#f2f3f5"),
            sidebar_hover: color("#ebedef"),
            sidebar_active: color("#e3e5e8"),
            user_panel: color("#ebedef"),
            chat: color("#ffffff"),
            chat_hover: color("#f2f3f5"),
            header: color("#ffffff"),
            input: color("#ebedef"),
            input_hover: color("#e3e5e8"),
            modal: color("#ffffff"),
            modal_sidebar: color("#f2f3f5"),
            modal_footer: color("#f2f3f5"),
            card: color("#f2f3f5"),
            card_hover: color("#ebedef"),
            text: color("#2e3338"),
            text_bright: color("#060607"),
            text_muted: color("#5c5e66"),
            text_faint: color("#80848e"),
            text_link: color("#006ce7"),
            interactive: color("#4e5058"),
            divider: color("#d4d7dc"),
            divider_soft: color("#e3e5e8"),
            accent: color("#5865f2"),
            accent_hover: color("#4752c4"),
            accent_text: color("#ffffff"),
            danger: color("#d83c3e"),
            danger_hover: color("#a12d2f"),
            green: color("#248046"),
            yellow: color("#b8891a"),
            scrim: color("#000000").alpha(0.35),
            mention: color("#e8e9fd"),
            mention_hover: color("#dfe1fd"),
            selected: color("#e3e5e8"),
            shadow: color("#000000").alpha(0.15),
            code_bg: color("#f2f3f5"),
        }
    }
}

pub const BODY: TextStyle = TextStyle::new(16.0, 400.0).lh(21.0);
pub const BODY_MED: TextStyle = TextStyle::new(16.0, 500.0).lh(21.0);
pub const BODY_BOLD: TextStyle = TextStyle::new(16.0, 700.0).lh(21.0);
pub const SMALL: TextStyle = TextStyle::new(14.0, 400.0).lh(18.0);
pub const SMALL_MED: TextStyle = TextStyle::new(14.0, 500.0).lh(18.0);
pub const LABEL: TextStyle = TextStyle::new(12.0, 600.0).lh(16.0);
pub const SECTION: TextStyle = TextStyle::new(12.0, 700.0).lh(16.0);
pub const TINY: TextStyle = TextStyle::new(11.0, 400.0).lh(14.0);
pub const NAME: TextStyle = TextStyle::new(16.0, 600.0).lh(20.0);
pub const TITLE: TextStyle = TextStyle::new(20.0, 700.0).lh(24.0);
pub const HERO: TextStyle = TextStyle::new(24.0, 800.0).lh(30.0);
pub const INTRO: TextStyle = TextStyle::new(32.0, 700.0).lh(38.0);
pub const MONO: TextStyle = TextStyle::new(13.0, 400.0).lh(18.0).mono();

pub fn status_color(p: Palette, status: runecord_protocol::Presence) -> Color {
    use runecord_protocol::Presence;
    match status {
        Presence::Online => p.green,
        Presence::Idle => p.yellow,
        Presence::Dnd => p.danger,
        Presence::Offline => p.text_faint,
    }
}

pub fn avatar_color(seed: u32) -> Color {
    color(&format!("#{:06x}", avatar_color_value(seed)))
}

/// Profile banner colors may be stored as a palette seed (legacy rows) or a
/// literal sRGB value.
pub fn profile_banner_color(value: u32) -> Color {
    if value < 0x100 {
        avatar_color(value)
    } else {
        color(&format!("#{:06x}", value))
    }
}

pub fn avatar_color_value(seed: u32) -> u32 {
    const COLORS: [u32; 10] = [
        0x5865f2, 0x3ba55d, 0xfaa61a, 0xed4245, 0xeb459e, 0x9b59b6, 0x1abc9c, 0xe67e22, 0x2ecc71,
        0x7289da,
    ];
    COLORS[(seed as usize) % COLORS.len()]
}
