//! The size text is drawn at, as shellwing does it: every size the views name is
//! written for body text at [`DRAWN_AT`] px and given in rems ([`ui`]), and the
//! window's rem follows the body size chosen ([`body`]): the system's UI font's,
//! unless Settings name another. So on Windows packwing's text is as large as
//! Explorer's, and on GNOME as the desktop's, scaled as its accessibility settings
//! scale text.

use gpui::{App, Pixels, Rems, px, rems};

/// The body size the views' sizes are written for: the design's.
pub const DRAWN_AT: f32 = 15.;

/// The body sizes Settings offer, in px, besides the system's.
pub const CHOICES: [u8; 9] = [11, 12, 13, 14, 15, 16, 17, 18, 20];

/// A text size as drawn at [`DRAWN_AT`]: it grows and shrinks with the body size.
pub fn ui(px: f32) -> Rems {
    rems(px / 16.)
}

/// The window's rem for body text of `body` px.
pub fn rem_for(body: f32) -> Pixels {
    px(16. * body / DRAWN_AT)
}

/// The body size of the system's UI font, read when packwing starts.
pub struct SystemSize(pub f32);

impl gpui::Global for SystemSize {}

/// The body size: the one Settings name, else the system's.
pub fn body(cx: &App) -> f32 {
    crate::prefs::prefs(cx)
        .text_size
        .map_or_else(|| system(cx), f32::from)
}

/// The system's body size, as read when packwing started.
pub fn system(cx: &App) -> f32 {
    cx.try_global::<SystemSize>().map_or(DRAWN_AT, |s| s.0)
}

/// The system's UI font size, in px at 96 dpi: what its own apps draw body text
/// at. The design's where it cannot be told, and in tests, whose pictures stay.
pub fn read_system() -> f32 {
    let found = if cfg!(test) { None } else { platform() };
    found
        .filter(|s| (8. ..=32.).contains(s))
        .unwrap_or(DRAWN_AT)
}

/// macOS draws body text at 13 px.
#[cfg(target_os = "macos")]
fn platform() -> Option<f32> {
    Some(13.)
}

/// Windows's message font, which Explorer's lists and dialogs use (9 pt Segoe
/// UI, 12 px), times Accessibility's text size: as the registry keeps them, at
/// the DPI it notes.
#[cfg(windows)]
fn platform() -> Option<f32> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;
    let user = RegKey::predef(HKEY_CURRENT_USER);
    let metrics = user
        .open_subkey(r"Control Panel\Desktop\WindowMetrics")
        .ok()?;
    let font = metrics.get_raw_value("MessageFont").ok()?.bytes;
    let dpi = metrics
        .get_value::<u32, _>("AppliedDPI")
        .ok()
        .filter(|d| *d > 0)
        .unwrap_or(96);
    let scale = user
        .open_subkey(r"Software\Microsoft\Accessibility")
        .and_then(|k| k.get_value::<u32, _>("TextScaleFactor"))
        .ok()
        .filter(|s| (100..=225).contains(s))
        .unwrap_or(100);
    message_font_px(&font, dpi, scale)
}

/// A LOGFONT's height in px at 96 dpi, as Windows keeps it at `dpi`, scaled by
/// `scale` percent.
#[cfg(any(test, windows))]
fn message_font_px(logfont: &[u8], dpi: u32, scale: u32) -> Option<f32> {
    let height = i32::from_le_bytes(logfont.get(..4)?.try_into().ok()?);
    (height != 0 && dpi > 0)
        .then(|| height.unsigned_abs() as f32 * 96. / dpi as f32 * scale as f32 / 100.)
}

/// The desktop's font, in points, as GNOME and those built on it keep it, times
/// the text scaling of its accessibility settings; KDE's otherwise.
#[cfg(all(unix, not(target_os = "macos")))]
fn platform() -> Option<f32> {
    gnome().or_else(kde)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn gnome() -> Option<f32> {
    let get = |key: &str| {
        let out = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", key])
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
    };
    let points = points_of(&get("font-name")?)?;
    let scale = get("text-scaling-factor")
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|s| *s > 0.)
        .unwrap_or(1.);
    Some(points * 96. / 72. * scale)
}

#[cfg(all(unix, not(target_os = "macos")))]
fn kde() -> Option<f32> {
    let home = std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))?;
    let text = std::fs::read_to_string(home.join("kdeglobals")).ok()?;
    kde_points(&text).map(|p| p * 96. / 72.)
}

/// The size, in points, at the end of a GNOME font name: `'Ubuntu Sans 11'`.
#[cfg(any(test, all(unix, not(target_os = "macos"))))]
fn points_of(name: &str) -> Option<f32> {
    name.trim_matches(['\'', '"'])
        .rsplit_once(' ')
        .and_then(|(_, size)| size.parse::<f32>().ok())
        .filter(|p| *p > 0.)
}

/// The size, in points, of KDE's general font: `font=Noto Sans,10,-1,5,…` in
/// `[General]`.
#[cfg(any(test, all(unix, not(target_os = "macos"))))]
fn kde_points(kdeglobals: &str) -> Option<f32> {
    let mut general = false;
    for line in kdeglobals.lines().map(str::trim) {
        if line.starts_with('[') {
            general = line == "[General]";
        } else if general && let Some(value) = line.strip_prefix("font=") {
            return value
                .split(',')
                .nth(1)?
                .trim()
                .parse()
                .ok()
                .filter(|p: &f32| *p > 0.);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_desktop_s_font_says_its_size() {
        assert_eq!(points_of("'Ubuntu Sans 11'"), Some(11.));
        assert_eq!(points_of("'Cantarell 10.5'"), Some(10.5));
        assert_eq!(points_of("'Noto Sans'"), None);
        let kde = "[Colors:View]\nfont=x\n[General]\nColorScheme=Breeze\nfont=Noto Sans,10,-1,5,50,0,0,0,0,0\n";
        assert_eq!(kde_points(kde), Some(10.));
        assert_eq!(kde_points("[General]\nfixed=Hack,10\n"), None);
    }

    #[test]
    fn windows_s_message_font_says_its_size() {
        // -15 at 120 dpi: 9 pt at 125 %, 12 px as GPUI counts them.
        let font = (-15i32).to_le_bytes();
        assert_eq!(message_font_px(&font, 120, 100), Some(12.));
        assert_eq!(message_font_px(&(-12i32).to_le_bytes(), 96, 150), Some(18.));
        assert_eq!(message_font_px(&[0, 0], 96, 100), None);
        assert_eq!(message_font_px(&0i32.to_le_bytes(), 96, 100), None);
    }

    #[test]
    fn sizes_follow_the_body_size() {
        // At the design's body size, a rem is GPUI's own, and sizes are as written.
        assert_eq!(rem_for(DRAWN_AT), px(16.));
        assert_eq!(ui(15.).to_pixels(rem_for(DRAWN_AT)), px(15.));
        // At Windows's 12 px, all of it is four fifths as large.
        assert_eq!(ui(15.).to_pixels(rem_for(12.)), px(12.));
        assert_eq!(ui(30.).to_pixels(rem_for(12.)), px(24.));
        // In tests, the design's: the pictures stay as they are.
        assert_eq!(read_system(), DRAWN_AT);
    }
}
