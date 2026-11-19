//! Kiln's palette and type scale, as tokens.
//!
//! The look is "Dusk Vice": deep navy and aubergine surfaces, hot magenta and
//! electric cyan accents, and a sunset ramp of amber into magenta into violet
//! laid over the title bar, selected rows, primary buttons and the auth card.
//! The daylight counterpart is the same city at noon: sun-bleached lavender
//! surfaces carrying the very same accents.

use rune::{Color, TextStyle};

pub fn color(hex: &str) -> Color {
    rune::color::hex(hex).unwrap_or(Color::BLACK)
}

/// Amber head of the sunset ramp.
pub fn sunset_amber(dark: bool) -> Color {
    if dark {
        color("#ffb648")
    } else {
        color("#ffcf87")
    }
}

/// Hot magenta middle of the sunset ramp.
pub fn sunset_magenta(dark: bool) -> Color {
    if dark {
        color("#ff2d95")
    } else {
        color("#e0268a")
    }
}

/// Violet foot of the sunset ramp.
pub fn sunset_violet(dark: bool) -> Color {
    if dark {
        color("#7b2ff7")
    } else {
        color("#8b5fd6")
    }
}

/// Electric cyan, the cool counter-accent that keeps the magenta honest.
pub fn electric(dark: bool) -> Color {
    if dark {
        color("#31e6ff")
    } else {
        color("#0089a7")
    }
}

/// The thin bright edge every glossy card carries.
pub fn gloss_edge(p: Palette) -> Color {
    if p.dark {
        p.accent.alpha(0.6)
    } else {
        p.accent.alpha(0.42)
    }
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
            // Night neon: navy and aubergine, almost black in the gutters.
            rail: color("#0a0716"),
            rail_hover: color("#1a1030"),
            sidebar: color("#110b22"),
            sidebar_hover: color("#1e1438"),
            sidebar_active: color("#2a1a4c"),
            user_panel: color("#0a0716"),
            chat: color("#0d0920"),
            chat_hover: color("#1a1033"),
            header: color("#140d28"),
            input: color("#1c1338"),
            input_hover: color("#271a4a"),
            modal: color("#150e2a"),
            modal_sidebar: color("#100a1e"),
            modal_footer: color("#100a1e"),
            card: color("#1b1236"),
            card_hover: color("#261848"),
            text: color("#ece6ff"),
            text_bright: color("#fdfbff"),
            text_muted: color("#b4a7d6"),
            text_faint: color("#7f74a6"),
            text_link: color("#ff6ad5"),
            interactive: color("#d7cdf5"),
            divider: color("#2e1f57"),
            divider_soft: color("#221741"),
            accent: color("#ff2d95"),
            accent_hover: color("#ff62b4"),
            accent_text: color("#150a20"),
            danger: color("#ff4d6d"),
            danger_hover: color("#ff7a92"),
            green: color("#35e0b0"),
            yellow: color("#ffc857"),
            scrim: color("#05020c").alpha(0.72),
            mention: color("#451a4e"),
            mention_hover: color("#ff2d95"),
            selected: color("#3a1050"),
            shadow: color("#1a0018").alpha(0.72),
            code_bg: color("#120b24"),
        }
    }

    pub fn light() -> Self {
        Palette {
            dark: false,
            // Sun-bleached day: the same city at noon, lavender paper.
            rail: color("#f3e6f2"),
            rail_hover: color("#e8d6ea"),
            sidebar: color("#faf0f7"),
            sidebar_hover: color("#f0dff0"),
            sidebar_active: color("#e6cfec"),
            user_panel: color("#f3e6f2"),
            chat: color("#fffafd"),
            chat_hover: color("#fbeef7"),
            header: color("#fdf3fa"),
            input: color("#f6e9f4"),
            input_hover: color("#ecdaee"),
            modal: color("#fffafd"),
            modal_sidebar: color("#f9eef6"),
            modal_footer: color("#f9eef6"),
            card: color("#f8ecf6"),
            card_hover: color("#eedcef"),
            text: color("#241a2e"),
            text_bright: color("#150c1e"),
            text_muted: color("#5d4a6e"),
            text_faint: color("#8a779c"),
            text_link: color("#b81e7a"),
            interactive: color("#3a2c4a"),
            divider: color("#e3cfe2"),
            divider_soft: color("#f0e0ee"),
            accent: color("#d01a84"),
            accent_hover: color("#ad1268"),
            accent_text: color("#fff7fc"),
            danger: color("#b02243"),
            danger_hover: color("#8d1735"),
            green: color("#0a6f5c"),
            yellow: color("#8a5a00"),
            scrim: color("#1a0a18").alpha(0.42),
            mention: color("#fbd6e6"),
            mention_hover: color("#f7bcd8"),
            selected: color("#f7d3e6"),
            shadow: color("#4a1030").alpha(0.18),
            code_bg: color("#f6ebf4"),
        }
    }
}

// ---------------------------------------------------------------------------
// Type scale: chunky display weights, generous leading, roomier sections.
// ---------------------------------------------------------------------------

pub const BODY: TextStyle = TextStyle::new(14.0, 400.0).lh(22.0);
pub const BODY_MED: TextStyle = TextStyle::new(14.0, 500.0).lh(22.0);
pub const BODY_BOLD: TextStyle = TextStyle::new(14.0, 700.0).lh(22.0);
pub const SMALL: TextStyle = TextStyle::new(12.0, 400.0).lh(19.0);
pub const SMALL_MED: TextStyle = TextStyle::new(12.0, 500.0).lh(19.0);
pub const LABEL: TextStyle = TextStyle::new(11.0, 700.0).lh(15.0);
// Uppercase group labels ride the mono face, which spaces the letters wide.
pub const SECTION: TextStyle = TextStyle::new(10.0, 800.0).lh(14.0).mono();
pub const TINY: TextStyle = TextStyle::new(10.0, 400.0).lh(15.0);
pub const NAME: TextStyle = TextStyle::new(14.5, 700.0).lh(21.0);
pub const TITLE: TextStyle = TextStyle::new(20.0, 800.0).lh(27.0);
pub const HERO: TextStyle = TextStyle::new(26.0, 800.0).lh(33.0);
pub const INTRO: TextStyle = TextStyle::new(32.0, 800.0).lh(40.0);
pub const MONO: TextStyle = TextStyle::new(11.5, 400.0).lh(18.0).mono();

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
    // Mid-dark dusk hues: white initials stay legible on every one.
    const COLORS: [u32; 10] = [
        0xb01a6e, 0x2f5f8c, 0x6b3fa0, 0x2f7a8c, 0x8c3f6a, 0x3f8073, 0x7a4a8c, 0x55637f, 0x9c3f5a,
        0x3f6f8c,
    ];
    COLORS[(seed as usize) % COLORS.len()]
}

fn parse_hex(value: &str) -> Option<u32> {
    u32::from_str_radix(value.trim_start_matches('#'), 16).ok()
}

/// Encode a custom nameplate. Kept inside the existing `nameplate: String`
/// field so no schema change is needed: `p:rrggbb` is a solid primary colour,
/// `g:rrggbb:rrggbb` is a primary -> secondary gradient.
pub fn encode_nameplate(primary: u32, secondary: u32, gradient: bool) -> String {
    if gradient {
        format!("g:{primary:06x}:{secondary:06x}")
    } else {
        format!("p:{primary:06x}")
    }
}

/// (primary, secondary, is_gradient)
pub fn nameplate_values(kind: &str) -> Option<(u32, u32, bool)> {
    match kind {
        "midnight" => Some((0x8fa6c8, 0x33456b, true)),
        "aurora" => Some((0xffb648, 0xff2d95, true)),
        "sunset" => Some((0xffb648, 0x7b2ff7, true)),
        "candy" => Some((0xff6ad5, 0x7b2ff7, true)),
        _ => {
            let mut parts = kind.split(':');
            match parts.next()? {
                "p" => {
                    let primary = parse_hex(parts.next()?)?;
                    Some((primary, primary, false))
                }
                "g" => {
                    let primary = parse_hex(parts.next()?)?;
                    let secondary = parse_hex(parts.next()?)?;
                    Some((primary, secondary, true))
                }
                _ => None,
            }
        }
    }
}

/// Nameplate colours for the profile. The name is drawn in the primary colour,
/// with the secondary colour faded in toward the bottom for gradients.
pub fn nameplate_colors(kind: &str) -> Option<(Color, Color, bool)> {
    nameplate_values(kind).map(|(primary, secondary, gradient)| {
        (
            color(&format!("#{primary:06x}")),
            color(&format!("#{secondary:06x}")),
            gradient,
        )
    })
}
