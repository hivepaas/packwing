//! The app's colors, in one place: what each one is for, and its value in the
//! light and the dark palette of the Soft tonal design.
//!
//! Views ask [`colors`] for a role (`colors().text`, `colors().tonal`), never
//! for a value, so a palette is changed here alone. The workspace picks one
//! with [`set_dark`] and redraws.
//!
//! Light and dark each come in styles ([`LightStyle`], [`DarkStyle`]), chosen
//! in Settings › General: a style puts the mode's palette on other grounds
//! (the page, the cards, the wells, the fields and the lines). The text, the
//! accent, the notes and the files' tags stay the mode's, so a role added to
//! the palette is in every style at once.
//!
//! Values are `0xRRGGBB` for `gpui::rgb`; those ending in an alpha byte
//! (`0xRRGGBBAA`, see [`Palette::scrim`]) are for `gpui::rgba`.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use packwing_i18n::tr;

/// A file's tag: its ground and the letters on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tint {
    pub ground: u32,
    pub ink: u32,
}

/// Every color the app draws with, by what it is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    // Surfaces, from the window inwards.
    /// The window's ground.
    pub page: u32,
    /// Cards and dialogs on it.
    pub surface: u32,
    /// Quiet wells in a card or a dialog: an option not chosen, a note.
    pub well: u32,
    /// Text fields and the search bar.
    pub field: u32,
    /// The row that is chosen in a list.
    pub chosen: u32,
    /// A row or a quiet button under the pointer.
    pub hover: u32,
    /// A well under the pointer.
    pub well_hover: u32,
    /// Around small buttons and selects; a switch that is off.
    pub outline: u32,

    // Text and icons.
    pub text: u32,
    /// Second to the text: a scroll bar being dragged.
    pub soft: u32,
    /// Hints, sizes, dates.
    pub muted: u32,
    /// Placeholders.
    pub faint: u32,

    // The accent.
    pub accent: u32,
    pub accent_hover: u32,
    /// Text on the accent.
    pub on_accent: u32,
    /// The accent's soft fill: the archive's header, a chosen option, the
    /// current place in the menu.
    pub tonal: u32,
    pub tonal_hover: u32,
    /// Text on the soft fill.
    pub on_tonal: u32,
    /// A switch's knob, on and off.
    pub knob_on: u32,
    pub knob_off: u32,
    /// The logo's feather: the design's orange, light and dark (see
    /// [`Palette::feather`]).
    pub feather_light: u32,
    pub feather_dark: u32,

    // Notes, by tone: a ground and its text.
    /// Something to know before going on: a file keeps its Internet mark.
    pub warn_bg: u32,
    pub warn_text: u32,
    /// Something went wrong.
    pub bad_bg: u32,
    pub bad_text: u32,

    // Translucent, `0xRRGGBBAA`.
    /// Behind a dialog.
    pub scrim: u32,
    /// Under a dialog and a menu.
    pub shadow: u32,

    // The window's own frame, where the app draws it (Linux).
    /// The window's outline.
    pub window_edge: u32,
    /// The title bar's buttons.
    pub button: u32,
    pub button_hover: u32,

    // A file's tag, by kind.
    pub folder_tag: Tint,
    pub red_tag: Tint,
    pub blue_tag: Tint,
    pub green_tag: Tint,
    pub plain_tag: Tint,
    pub orange_tag: Tint,
    /// A video's: rose, a colour no other kind has.
    pub video_tag: Tint,
    /// A symbolic link's.
    pub link_tag: Tint,
}

pub const LIGHT: Palette = Palette {
    page: 0xf3f7f4,
    surface: 0xffffff,
    well: 0xf3f7f4,
    field: 0xe2ece6,
    chosen: 0xe2ece6,
    hover: 0xebf2ed,
    well_hover: 0xe6eee9,
    outline: 0xb9c7bf,

    text: 0x17211c,
    soft: 0x2c3a33,
    muted: 0x46524b,
    faint: 0x5c6861,

    accent: 0x1f6b4f,
    accent_hover: 0x18573f,
    on_accent: 0xffffff,
    tonal: 0xcdebdd,
    tonal_hover: 0xbfe3d2,
    on_tonal: 0x0b3b2a,
    knob_on: 0xffffff,
    knob_off: 0xffffff,
    feather_light: 0xffe1cc,
    feather_dark: 0x5a2e0c,

    warn_bg: 0xffe1cc,
    warn_text: 0x5a2e0c,
    bad_bg: 0xffdad6,
    bad_text: 0x7a1e16,

    scrim: 0x17211c40,
    shadow: 0x17211c24,

    window_edge: 0xb9c7bf,
    button: 0xe2ece6,
    button_hover: 0xcdd9d2,

    folder_tag: Tint {
        ground: 0xffe9b8,
        ink: 0x5c4200,
    },
    red_tag: Tint {
        ground: 0xffdad6,
        ink: 0x7a1e16,
    },
    blue_tag: Tint {
        ground: 0xd6e6ff,
        ink: 0x123c73,
    },
    green_tag: Tint {
        ground: 0xcdebdd,
        ink: 0x0b3b2a,
    },
    // A shade off `chosen`, so that a chosen row's tile still shows.
    plain_tag: Tint {
        ground: 0xd5e1da,
        ink: 0x2c3a33,
    },
    orange_tag: Tint {
        ground: 0xffe1cc,
        ink: 0x5a2e0c,
    },
    video_tag: Tint {
        ground: 0xfbdaea,
        ink: 0x6b1a48,
    },
    link_tag: Tint {
        ground: 0xe8defc,
        ink: 0x3d2470,
    },
};

/// The dark palette, "night blue": grounds that lean cool blue, the accent
/// leaning from mint toward teal, and the light palette's orange.
pub const DARK: Palette = Palette {
    page: 0x0e1318,
    surface: 0x161d24,
    well: 0x1e262f,
    field: 0x273038,
    chosen: 0x212a33,
    hover: 0x1c242c,
    well_hover: 0x26303a,
    outline: 0x36414c,

    text: 0xe2e8ee,
    soft: 0xc8d1da,
    muted: 0xa8b3be,
    faint: 0x919ca7,

    accent: 0x7ad6c0,
    accent_hover: 0xa5ead9,
    on_accent: 0x00352b,
    tonal: 0x1b4747,
    tonal_hover: 0x225656,
    on_tonal: 0xb3eee4,
    knob_on: 0x00352b,
    knob_off: 0xa8b3be,
    feather_light: 0xffdcc2,
    feather_dark: 0x55391f,

    warn_bg: 0x55391f,
    warn_text: 0xffdcc2,
    bad_bg: 0x55211d,
    bad_text: 0xffdad6,

    scrim: 0x00000080,
    shadow: 0x00000080,

    window_edge: 0x04070a,
    button: 0x1e262f,
    button_hover: 0x2b3540,

    folder_tag: Tint {
        ground: 0x433712,
        ink: 0xffe08a,
    },
    red_tag: Tint {
        ground: 0x55211d,
        ink: 0xffdad6,
    },
    blue_tag: Tint {
        ground: 0x1c3557,
        ink: 0xcfe2ff,
    },
    green_tag: Tint {
        ground: 0x1b4747,
        ink: 0xb3eee4,
    },
    // A shade off `chosen`, so that a chosen row's tile still shows.
    plain_tag: Tint {
        ground: 0x2b3540,
        ink: 0xcbd3db,
    },
    orange_tag: Tint {
        ground: 0x55391f,
        ink: 0xffdcc2,
    },
    video_tag: Tint {
        ground: 0x4f2340,
        ink: 0xffd8ee,
    },
    link_tag: Tint {
        ground: 0x352a57,
        ink: 0xe4dbff,
    },
};

/// The grounds a style puts under the mode's colors; the rest of them are
/// worked out from these as the design's own palettes have them.
struct Grounds {
    page: u32,
    surface: u32,
    field: u32,
    outline: u32,
}

/// `percent` of `a` over `b`, channel by channel.
const fn mix(a: u32, b: u32, percent: u32) -> u32 {
    let mut out = 0;
    let mut shift = 0;
    while shift < 24 {
        let x = (a >> shift) & 0xff;
        let y = (b >> shift) & 0xff;
        out |= ((x * percent + y * (100 - percent) + 50) / 100) << shift;
        shift += 8;
    }
    out
}

/// [`LIGHT`] on other grounds: a well is the page, the chosen row a field.
const fn light(g: Grounds) -> Palette {
    Palette {
        page: g.page,
        surface: g.surface,
        well: g.page,
        field: g.field,
        chosen: g.field,
        hover: mix(g.field, g.surface, 70),
        well_hover: mix(g.field, g.page, 75),
        outline: g.outline,
        window_edge: g.outline,
        button: g.field,
        button_hover: mix(g.outline, g.field, 50),
        plain_tag: Tint {
            ground: mix(g.outline, g.field, 30),
            ink: LIGHT.plain_tag.ink,
        },
        ..LIGHT
    }
}

/// [`DARK`] on other grounds: a well, the chosen row and a row under the
/// pointer each between the card and the field.
const fn dark(g: Grounds) -> Palette {
    let well = mix(g.field, g.surface, 45);
    Palette {
        page: g.page,
        surface: g.surface,
        well,
        field: g.field,
        chosen: mix(g.field, g.surface, 60),
        hover: mix(g.field, g.surface, 35),
        well_hover: mix(g.outline, g.field, 10),
        outline: g.outline,
        window_edge: mix(g.page, 0x000000, 30),
        button: well,
        button_hover: mix(g.outline, g.field, 40),
        plain_tag: Tint {
            ground: mix(g.outline, g.field, 30),
            ink: DARK.plain_tag.ink,
        },
        ..DARK
    }
}

const PAPER: Palette = light(Grounds {
    page: 0xf4f0e8,
    surface: 0xfffcf6,
    field: 0xeae4d8,
    outline: 0xc9bfac,
});
const SOFT_GRAY: Palette = Palette {
    // Darker than the mint's, as the grounds are: a placeholder as easy to read.
    faint: 0x59655e,
    ..light(Grounds {
        page: 0xe9ebee,
        surface: 0xf5f6f8,
        field: 0xdcdfe4,
        outline: 0xb4bac4,
    })
};
/// White cards, near-black text and firmer lines: for a pale screen or a
/// bright room.
const CONTRAST: Palette = Palette {
    text: 0x05070b,
    soft: 0x1c222d,
    muted: 0x2c3240,
    faint: 0x4a5262,
    ..light(Grounds {
        page: 0xeef1f5,
        surface: 0xffffff,
        field: 0xe3e7ee,
        outline: 0x8a94a5,
    })
};
const MIST: Palette = Palette {
    // Darker than the mint's, as the grounds are: a placeholder as easy to read.
    faint: 0x57635c,
    ..light(Grounds {
        page: 0xe3e9f2,
        surface: 0xf2f5fa,
        field: 0xd3dce9,
        outline: 0xaebbd0,
    })
};
const LAVENDER: Palette = Palette {
    // Darker than the mint's, as the grounds are: a placeholder as easy to read.
    faint: 0x5a665f,
    ..light(Grounds {
        page: 0xefedf7,
        surface: 0xfbfaff,
        field: 0xe2def1,
        outline: 0xc0b7da,
    })
};
const DUSK: Palette = Palette {
    // Darker than the mint's, as the grounds are: a placeholder as easy to read.
    faint: 0x525d57,
    ..light(Grounds {
        page: 0xdedbd5,
        surface: 0xebe9e4,
        field: 0xd6d2ca,
        outline: 0xb0a99c,
    })
};

const NEUTRAL: Palette = Palette {
    // Lighter than the night's, as the fields are: a placeholder as easy to read.
    faint: 0x98a3ad,
    ..dark(Grounds {
        page: 0x1e1f22,
        surface: 0x26272b,
        field: 0x37383e,
        outline: 0x4a4c54,
    })
};
const GRAPHITE: Palette = dark(Grounds {
    page: 0x141517,
    surface: 0x1c1d20,
    field: 0x2c2e33,
    outline: 0x3d4047,
});
/// Near black, for a screen whose black is off.
const MIDNIGHT: Palette = dark(Grounds {
    page: 0x08090a,
    surface: 0x121315,
    field: 0x26272c,
    outline: 0x37393f,
});
const DIM: Palette = Palette {
    // Lighter than the night's, as the fields are: as easy to read.
    muted: 0xb6c0ca,
    faint: 0xb1bac3,
    ..dark(Grounds {
        page: 0x2a2e35,
        surface: 0x32373f,
        field: 0x434a55,
        outline: 0x565e6b,
    })
};
const DEEP_SEA: Palette = dark(Grounds {
    page: 0x0e1626,
    surface: 0x152036,
    field: 0x22304d,
    outline: 0x30406a,
});
const ESPRESSO: Palette = dark(Grounds {
    page: 0x1b1815,
    surface: 0x24201c,
    field: 0x342e28,
    outline: 0x473f37,
});
const FOREST: Palette = dark(Grounds {
    page: 0x121b18,
    surface: 0x192420,
    field: 0x26342e,
    outline: 0x36463f,
});
const PLUM: Palette = dark(Grounds {
    page: 0x1a1521,
    surface: 0x221c2b,
    field: 0x342b45,
    outline: 0x473c5d,
});

/// Kept in the prefs by a name of its own: a name this build does not know,
/// from a later one, is the default rather than prefs that cannot be read.
macro_rules! style {
    ($(#[$doc:meta])* $style:ident { $($variant:ident = $key:literal => $palette:ident,)+ }) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum $style {
            #[default]
            $($variant,)+
        }

        impl $style {
            /// Every one, the default first, as Settings shows them.
            pub const ALL: [$style; [$($style::$variant),+].len()] = [$($style::$variant),+];

            /// Its name in the prefs.
            pub fn key(self) -> &'static str {
                match self {
                    $($style::$variant => $key,)+
                }
            }

            /// The one kept as `key`; the default for a name this build does not know.
            pub fn from_key(key: &str) -> Self {
                Self::ALL.into_iter().find(|s| s.key() == key).unwrap_or_default()
            }

            pub fn palette(self) -> &'static Palette {
                match self {
                    $($style::$variant => &$palette,)+
                }
            }

            fn from_index(index: u8) -> Self {
                Self::ALL.get(usize::from(index)).copied().unwrap_or_default()
            }
        }
    };
}

style! {
    /// How the light palette looks; Mint is the design's.
    LightStyle {
        Mint = "mint" => LIGHT,
        Paper = "paper" => PAPER,
        SoftGray = "soft-gray" => SOFT_GRAY,
        Contrast = "contrast" => CONTRAST,
        Mist = "mist" => MIST,
        Lavender = "lavender" => LAVENDER,
        Dusk = "dusk" => DUSK,
    }
}

style! {
    /// How the dark palette looks; Night, the design's night blue, unless
    /// another is chosen.
    DarkStyle {
        Night = "night" => DARK,
        Neutral = "neutral" => NEUTRAL,
        Graphite = "graphite" => GRAPHITE,
        Midnight = "midnight" => MIDNIGHT,
        Dim = "dim" => DIM,
        DeepSea = "deep-sea" => DEEP_SEA,
        Espresso = "espresso" => ESPRESSO,
        Forest = "forest" => FOREST,
        Plum = "plum" => PLUM,
    }
}

impl LightStyle {
    pub fn name(self) -> String {
        match self {
            Self::Mint => tr!("Mint"),
            Self::Paper => tr!("Paper"),
            Self::SoftGray => tr!("Soft gray"),
            Self::Contrast => tr!("High contrast"),
            Self::Mist => tr!("Mist"),
            Self::Lavender => tr!("Lavender"),
            Self::Dusk => tr!("Dusk"),
        }
    }
}

impl DarkStyle {
    pub fn name(self) -> String {
        match self {
            Self::Night => tr!("Night"),
            Self::Neutral => tr!("Neutral"),
            Self::Graphite => tr!("Graphite"),
            Self::Midnight => tr!("Midnight"),
            Self::Dim => tr!("Dim"),
            Self::DeepSea => tr!("Deep sea"),
            Self::Espresso => tr!("Espresso"),
            Self::Forest => tr!("Forest"),
            Self::Plum => tr!("Plum"),
        }
    }
}

/// The dark palette is in use.
static DARK_IN_USE: AtomicBool = AtomicBool::new(false);
/// Which style each is in, as its place in `ALL`.
static LIGHT_STYLE: AtomicU8 = AtomicU8::new(0);
static DARK_STYLE: AtomicU8 = AtomicU8::new(0);

/// The palette in use.
pub fn colors() -> Palette {
    *chosen(
        DARK_IN_USE.load(Ordering::Relaxed),
        LightStyle::from_index(LIGHT_STYLE.load(Ordering::Relaxed)),
        DarkStyle::from_index(DARK_STYLE.load(Ordering::Relaxed)),
    )
}

fn chosen(dark: bool, light_style: LightStyle, dark_style: DarkStyle) -> &'static Palette {
    if dark {
        dark_style.palette()
    } else {
        light_style.palette()
    }
}

/// Chooses the palette; the caller redraws.
pub fn set_dark(dark: bool) {
    DARK_IN_USE.store(dark, Ordering::Relaxed);
}

/// Chooses how light looks and how dark does; the caller redraws.
pub fn set_styles(light: LightStyle, dark: DarkStyle) {
    LIGHT_STYLE.store(light as u8, Ordering::Relaxed);
    DARK_STYLE.store(dark as u8, Ordering::Relaxed);
}

impl Palette {
    /// The logo's feather on a tile of `tint`: the design's orange, light
    /// where the tile's ink is light and dark where it is dark, so it reads
    /// on the tile as the ink does.
    pub fn feather(&self, tint: Tint) -> u32 {
        let (r, g, b) = (
            (tint.ink >> 16) & 0xff,
            (tint.ink >> 8) & 0xff,
            tint.ink & 0xff,
        );
        if 299 * r + 587 * g + 114 * b > 127_500 {
            self.feather_light
        } else {
            self.feather_dark
        }
    }
}

/// `color` (`0xRRGGBB`) with an alpha byte, for `gpui::rgba`.
pub fn with_alpha(color: u32, alpha: u8) -> u32 {
    (color << 8) | alpha as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// WCAG's contrast ratio of two `0xRRGGBB` colors.
    fn contrast(a: u32, b: u32) -> f64 {
        let lum = |c: u32| {
            let ch = |v: u32| {
                let s = f64::from(v & 0xff) / 255.;
                if s <= 0.03928 {
                    s / 12.92
                } else {
                    ((s + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * ch(c >> 16) + 0.7152 * ch(c >> 8) + 0.0722 * ch(c)
        };
        let (x, y) = (lum(a), lum(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    /// Every style of both, by name.
    fn every_palette() -> Vec<(&'static str, &'static Palette)> {
        LightStyle::ALL
            .into_iter()
            .map(|s| (s.key(), s.palette()))
            .chain(DarkStyle::ALL.into_iter().map(|s| (s.key(), s.palette())))
            .collect()
    }

    #[test]
    fn the_defaults_are_the_design_s_palettes() {
        assert_eq!(LightStyle::default().palette(), &LIGHT);
        assert_eq!(DarkStyle::default().palette(), &DARK);
        assert_eq!(LIGHT.page, 0xf3f7f4);
        assert_eq!(DARK.page, 0x0e1318);
    }

    #[test]
    fn text_is_readable_on_its_grounds() {
        for (_, &p) in every_palette() {
            for (fg, bg) in [
                (p.text, p.page),
                (p.text, p.surface),
                (p.text, p.well),
                (p.text, p.field),
                (p.muted, p.page),
                (p.muted, p.surface),
                (p.muted, p.well),
                (p.muted, p.field),
                (p.faint, p.field),
                (p.on_accent, p.accent),
                (p.on_tonal, p.tonal),
                (p.accent, p.surface),
                (p.warn_text, p.warn_bg),
                (p.bad_text, p.bad_bg),
            ] {
                let c = contrast(fg, bg);
                assert!(c >= 4.5, "{fg:06x} on {bg:06x}: {c:.2}");
            }
            for t in [
                p.folder_tag,
                p.red_tag,
                p.blue_tag,
                p.green_tag,
                p.plain_tag,
                p.orange_tag,
                p.video_tag,
                p.link_tag,
            ] {
                assert!(contrast(t.ink, t.ground) >= 4.5, "{t:?}");
            }
        }
    }

    /// Held to what the design's palettes give: text at 7:1 on every
    /// ground, the least text no fainter than a placeholder can be read.
    #[test]
    fn every_style_reads_as_the_design_s_do() {
        let mut short = Vec::new();
        for (name, p) in every_palette() {
            let mut need = |what: &str, fore: u32, back: u32, least: f64| {
                let got = contrast(fore, back);
                if got < least {
                    short.push(format!("{name}: {what} {got:.2} < {least}"));
                }
            };
            for (ground, back) in [
                ("page", p.page),
                ("surface", p.surface),
                ("well", p.well),
                ("field", p.field),
                ("chosen", p.chosen),
            ] {
                need(&format!("text on {ground}"), p.text, back, 7.);
            }
            for (ground, back) in [("page", p.page), ("surface", p.surface)] {
                need(&format!("faint on {ground}"), p.faint, back, 4.5);
            }
            need("accent on page", p.accent, p.page, 3.);
        }
        assert!(short.is_empty(), "{}", short.join("\n"));
    }

    /// The chosen row, a row under the pointer, a switch's outline and a
    /// plain file's tag on the chosen row each stand out from what they are on.
    #[test]
    fn what_is_chosen_can_be_seen() {
        let mut short = Vec::new();
        for (name, p) in every_palette() {
            for (what, a, b, least) in [
                ("chosen row on surface", p.chosen, p.surface, 1.08),
                ("hover on surface", p.hover, p.surface, 1.06),
                ("well hover on well", p.well_hover, p.well, 1.03),
                ("outline on surface", p.outline, p.surface, 1.5),
                ("plain tag on chosen", p.plain_tag.ground, p.chosen, 1.05),
            ] {
                let got = contrast(a, b);
                if got < least {
                    short.push(format!("{name}: {what} {got:.3}"));
                }
            }
        }
        assert!(short.is_empty(), "{}", short.join("\n"));
    }

    #[test]
    fn a_style_is_kept_by_name_and_an_unknown_one_is_the_default() {
        for style in LightStyle::ALL {
            assert_eq!(LightStyle::from_key(style.key()), style);
            assert_eq!(LightStyle::from_index(style as u8), style);
        }
        for style in DarkStyle::ALL {
            assert_eq!(DarkStyle::from_key(style.key()), style);
            assert_eq!(DarkStyle::from_index(style as u8), style);
        }
        assert_eq!(LightStyle::from_key("aurora"), LightStyle::Mint);
        assert_eq!(DarkStyle::from_key(""), DarkStyle::Night);
        assert_eq!(DarkStyle::from_index(200), DarkStyle::Night);
    }

    #[test]
    fn styles_are_chosen_apart_for_light_and_dark() {
        assert_eq!(chosen(false, LightStyle::Paper, DarkStyle::Forest), &PAPER);
        assert_eq!(chosen(true, LightStyle::Paper, DarkStyle::Forest), &FOREST);
    }

    #[test]
    fn the_feather_reads_on_the_logo_s_tiles() {
        for (_, &p) in every_palette() {
            for t in [
                Tint {
                    ground: p.accent,
                    ink: p.on_accent,
                },
                Tint {
                    ground: p.tonal,
                    ink: p.on_tonal,
                },
            ] {
                let c = contrast(p.feather(t), t.ground);
                assert!(c >= 3., "{t:?}: {c:.2}");
            }
        }
    }

    #[test]
    fn alpha_goes_last() {
        assert_eq!(with_alpha(0x17211c, 0x40), 0x17211c40);
    }
}
