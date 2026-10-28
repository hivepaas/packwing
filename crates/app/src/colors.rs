//! The app's colors, in one place: what each one is for, and its value in the
//! light and the dark palette of the Soft tonal design.
//!
//! Views ask [`colors`] for a role (`colors().text`, `colors().tonal`), never
//! for a value, so a palette is changed here alone. The workspace picks one
//! with [`set_dark`] and redraws.
//!
//! Values are `0xRRGGBB` for `gpui::rgb`; those ending in an alpha byte
//! (`0xRRGGBBAA`, see [`Palette::scrim`]) are for `gpui::rgba`.

use std::sync::atomic::{AtomicBool, Ordering};

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
    plain_tag: Tint {
        ground: 0xe2ece6,
        ink: 0x2c3a33,
    },
    orange_tag: Tint {
        ground: 0xffe1cc,
        ink: 0x5a2e0c,
    },
};

pub const DARK: Palette = Palette {
    page: 0x0f1512,
    surface: 0x18201c,
    well: 0x212b26,
    field: 0x2a352f,
    chosen: 0x212b26,
    hover: 0x1f2924,
    well_hover: 0x29342e,
    outline: 0x3a4741,

    text: 0xe1e9e4,
    soft: 0xc9d5ce,
    muted: 0xa9b6af,
    faint: 0x93a098,

    accent: 0x7fd8b0,
    accent_hover: 0xa8ebcb,
    on_accent: 0x003824,
    tonal: 0x1c4f3b,
    tonal_hover: 0x235c46,
    on_tonal: 0xb5f0d2,
    knob_on: 0x003824,
    knob_off: 0xa9b6af,

    warn_bg: 0x5a3a20,
    warn_text: 0xffdcc2,
    bad_bg: 0x5c1f1a,
    bad_text: 0xffdad6,

    scrim: 0x00000080,
    shadow: 0x00000080,

    window_edge: 0x050806,
    button: 0x212b26,
    button_hover: 0x2f3b35,

    folder_tag: Tint {
        ground: 0x4a3a10,
        ink: 0xffe08a,
    },
    red_tag: Tint {
        ground: 0x5c1f1a,
        ink: 0xffdad6,
    },
    blue_tag: Tint {
        ground: 0x1e3352,
        ink: 0xcfe2ff,
    },
    green_tag: Tint {
        ground: 0x1c4f3b,
        ink: 0xb5f0d2,
    },
    plain_tag: Tint {
        ground: 0x212b26,
        ink: 0xc9d5ce,
    },
    orange_tag: Tint {
        ground: 0x5a3a20,
        ink: 0xffdcc2,
    },
};

/// The dark palette is in use.
static DARK_IN_USE: AtomicBool = AtomicBool::new(false);

/// The palette in use.
pub fn colors() -> Palette {
    if DARK_IN_USE.load(Ordering::Relaxed) {
        DARK
    } else {
        LIGHT
    }
}

/// Chooses the palette; the caller redraws.
pub fn set_dark(dark: bool) {
    DARK_IN_USE.store(dark, Ordering::Relaxed);
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

    #[test]
    fn text_is_readable_on_its_grounds() {
        for p in [LIGHT, DARK] {
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
            ] {
                assert!(contrast(t.ink, t.ground) >= 4.5, "{t:?}");
            }
        }
    }

    #[test]
    fn alpha_goes_last() {
        assert_eq!(with_alpha(0x17211c, 0x40), 0x17211c40);
    }
}
