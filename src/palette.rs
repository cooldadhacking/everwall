//! The Everforest palettes.
//!
//! Hex values are the official ones from sainnhe/everforest (`palette.md`).
//!
//! Each variant is split into three groups:
//!  - the **neutral** background ramp (bg_dim … bg5) — the theme's actual page
//!    tones, a low-chroma green-grey (dark) / warm cream (light) ladder;
//!  - the shared **foreground** — fg, the seven accents, and three greys;
//!  - the muted **accent-tinted backgrounds** (bg_visual/red/green/blue/yellow)
//!    — these are UI chrome (selection, diff highlights), not page tones.
//!
//! The default recolor palette is neutral + foreground, which keeps a warm or
//! near-neutral image reading as green-grey Everforest instead of getting
//! dragged toward those muted pink/purple chrome tints. The tints are opt-in
//! via `rich` when you want extra colored anchors in the shadows.

use crate::color::{Oklab, parse_hex, srgb_to_oklab};

/// Foreground colors shared by all three dark variants.
const DARK_FG: &[&str] = &[
    "#D3C6AA", // fg
    "#E67E80", // red
    "#E69875", // orange
    "#DBBC7F", // yellow
    "#A7C080", // green
    "#83C092", // aqua
    "#7FBBB3", // blue
    "#D699B6", // purple
    "#7A8478", // grey0
    "#859289", // grey1
    "#9DA9A0", // grey2
];

// Neutral background ramp: bg_dim, bg0, bg1, bg2, bg3, bg4, bg5.
const DARK_HARD_BG: &[&str] =
    &["#1E2326", "#272E33", "#2E383C", "#374145", "#414B50", "#495156", "#4F5B58"];
const DARK_MEDIUM_BG: &[&str] =
    &["#232A2E", "#2D353B", "#343F44", "#3D484D", "#475258", "#4F585E", "#56635F"];
const DARK_SOFT_BG: &[&str] =
    &["#293136", "#333C43", "#3A464C", "#434F55", "#4D5960", "#555F66", "#5D6B66"];

// Muted accent-tinted backgrounds: bg_visual, bg_red, bg_yellow, bg_green,
// bg_blue, bg_purple.
const DARK_HARD_TINT: &[&str] =
    &["#4C3743", "#493B40", "#45443C", "#3C4841", "#384B55", "#463F48"];
const DARK_MEDIUM_TINT: &[&str] =
    &["#543A48", "#514045", "#4D4C43", "#425047", "#3A515D", "#4A444E"];
const DARK_SOFT_TINT: &[&str] =
    &["#5C3F4F", "#59464C", "#55544A", "#48584E", "#3F5865", "#4E4953"];

/// Foreground colors shared by all three light variants.
const LIGHT_FG: &[&str] = &[
    "#5C6A72", // fg
    "#F85552", // red
    "#F57D26", // orange
    "#DFA000", // yellow
    "#8DA101", // green
    "#35A77C", // aqua
    "#3A94C5", // blue
    "#DF69BA", // purple
    "#A6B0A0", // grey0
    "#939F91", // grey1
    "#829181", // grey2
];

// Neutral background ramp: bg_dim, bg0, bg1, bg2, bg3, bg4/bg5.
const LIGHT_HARD_BG: &[&str] = &["#F2EFDF", "#FFFBEF", "#F8F5E4", "#EDEADA", "#E8E5D5", "#BEC5B2"];
const LIGHT_MEDIUM_BG: &[&str] =
    &["#EFEBD4", "#FDF6E3", "#F4F0D9", "#E6E2CC", "#E0DCC7", "#BDC3AF"];
const LIGHT_SOFT_BG: &[&str] = &["#E5DFC5", "#F3EAD3", "#EAE4CA", "#DDD8BE", "#D8D3BA", "#B9C0AB"];

// Muted accent-tinted backgrounds.
const LIGHT_HARD_TINT: &[&str] =
    &["#F0F2D4", "#FFE7DE", "#FEF2D5", "#F3F5D9", "#ECF5ED", "#FCECED"];
const LIGHT_MEDIUM_TINT: &[&str] =
    &["#EAEDC8", "#FDE3DA", "#FAEDCD", "#F0F1D2", "#E9F0E9", "#FAE8E2"];
const LIGHT_SOFT_TINT: &[&str] =
    &["#E1E4BD", "#FADBD0", "#F1E4C5", "#E5E6C5", "#E1E7DD", "#F1DDD4"];

/// All selectable palette variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Variant {
    #[value(name = "dark-hard")]
    DarkHard,
    #[value(name = "dark-medium")]
    DarkMedium,
    #[value(name = "dark-soft")]
    DarkSoft,
    #[value(name = "light-hard")]
    LightHard,
    #[value(name = "light-medium")]
    LightMedium,
    #[value(name = "light-soft")]
    LightSoft,
}

impl Variant {
    pub const ALL: [Variant; 6] = [
        Variant::DarkHard,
        Variant::DarkMedium,
        Variant::DarkSoft,
        Variant::LightHard,
        Variant::LightMedium,
        Variant::LightSoft,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Variant::DarkHard => "dark-hard",
            Variant::DarkMedium => "dark-medium",
            Variant::DarkSoft => "dark-soft",
            Variant::LightHard => "light-hard",
            Variant::LightMedium => "light-medium",
            Variant::LightSoft => "light-soft",
        }
    }

    /// (neutral background ramp, shared foreground, muted accent tints).
    fn hex_groups(self) -> [&'static [&'static str]; 3] {
        match self {
            Variant::DarkHard => [DARK_HARD_BG, DARK_FG, DARK_HARD_TINT],
            Variant::DarkMedium => [DARK_MEDIUM_BG, DARK_FG, DARK_MEDIUM_TINT],
            Variant::DarkSoft => [DARK_SOFT_BG, DARK_FG, DARK_SOFT_TINT],
            Variant::LightHard => [LIGHT_HARD_BG, LIGHT_FG, LIGHT_HARD_TINT],
            Variant::LightMedium => [LIGHT_MEDIUM_BG, LIGHT_FG, LIGHT_MEDIUM_TINT],
            Variant::LightSoft => [LIGHT_SOFT_BG, LIGHT_FG, LIGHT_SOFT_TINT],
        }
    }

    /// Build the resolved palette (precomputed Oklab) for this variant.
    ///
    /// `rich` adds the muted accent-tinted backgrounds for extra colored
    /// anchors in the shadows; off by default for a cleaner, greener result.
    pub fn palette(self, rich: bool) -> Palette {
        let [bg, fg, tint] = self.hex_groups();
        let mut colors = Vec::new();
        let groups: &[&[&str]] = if rich { &[bg, fg, tint] } else { &[bg, fg] };
        for set in groups {
            for hex in *set {
                let rgb = parse_hex(hex).expect("palette hex constants are valid");
                colors.push(PaletteColor {
                    oklab: srgb_to_oklab(rgb),
                });
            }
        }
        Palette {
            variant: self,
            colors,
        }
    }
}

/// One palette entry, stored as its precomputed Oklab coordinates.
#[derive(Clone, Copy, Debug)]
pub struct PaletteColor {
    pub oklab: Oklab,
}

/// A fully resolved palette ready for recoloring.
#[derive(Clone, Debug)]
pub struct Palette {
    pub variant: Variant,
    pub colors: Vec<PaletteColor>,
}
