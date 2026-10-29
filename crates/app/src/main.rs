//! packwing: a file archiver that opens archives safely.
//!
//! Usage: `packwing [--first-run] [--managed]`. For now the window shows
//! made-up archives (`demo.rs`) and its buttons start nothing real: reading
//! and writing archives comes with hivepack. `--first-run` opens on the
//! welcome; `--managed` runs as an organization would, with its policies.

// A GUI program on Windows: no console window opens beside it.
#![cfg_attr(windows, windows_subsystem = "windows")]

mod chrome;
mod colors;
mod demo;
mod dialogs;
mod input;
mod pages;
mod widgets;
mod workspace;

#[cfg(all(test, feature = "shots", target_os = "linux"))]
mod shots;

use std::borrow::Cow;

use gpui::{
    App, AppContext as _, Bounds, Focusable as _, TitlebarOptions, WindowBounds, WindowDecorations,
    WindowOptions, px, size,
};

/// The UI's typeface, drawn into the binary. The design asks for Figtree,
/// which has no Vietnamese letters; Nunito has every one of them, and its
/// rounded ends suit the Soft tonal design. Its four weights are static
/// instances of google/fonts' variable `ofl/nunito/Nunito[wght].ttf`, made
/// with fontTools' `varLib.instancer --static --update-name-table`.
pub const FONT: &str = "Nunito";

const FONTS: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/Nunito-Regular.ttf"),
    include_bytes!("../assets/fonts/Nunito-Medium.ttf"),
    include_bytes!("../assets/fonts/Nunito-SemiBold.ttf"),
    include_bytes!("../assets/fonts/Nunito-Bold.ttf"),
];

/// Hands [`FONTS`] to GPUI; without them the system's font draws the UI.
pub fn load_fonts(cx: &mut App) {
    let fonts = FONTS.iter().map(|f| Cow::Borrowed(*f)).collect();
    if let Err(e) = cx.text_system().add_fonts(fonts) {
        eprintln!("packwing: cannot load the UI font: {e}");
    }
}

/// Icons drawn into the binary: the app has no other files.
pub struct Assets;

macro_rules! icons {
    ($($name:literal),* $(,)?) => {
        &[$((
            concat!("icons/", $name, ".svg"),
            include_bytes!(concat!("../assets/icons/", $name, ".svg")),
        )),*]
    };
}

const ICONS: &[(&str, &[u8])] = icons![
    "archive",
    "building",
    "check",
    "chevron-down",
    "extract",
    "eye",
    "eye-off",
    "image",
    "lock",
    "logo",
    "plus",
    "recent",
    "search",
    "shield",
    "sliders",
    "warning",
    "x",
    // The window's buttons, as GNOME and Windows draw them.
    "minimize",
    "maximize",
    "close",
    "win-minimize",
    "win-maximize",
    "win-restore",
    "win-close",
];

impl gpui::AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok(ICONS
            .iter()
            .find(|(p, _)| *p == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        Ok(ICONS
            .iter()
            .filter(|(p, _)| p.starts_with(path))
            .map(|(p, _)| (*p).into())
            .collect())
    }
}

const USAGE: &str = "usage: packwing [--first-run] [--managed]
  --first-run  open on the welcome, as on the first start
  --managed    as an organization runs it: Settings › Security shows its policies";

fn main() {
    let (mut first_run, mut managed) = (false, false);
    for a in std::env::args().skip(1) {
        match a.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return;
            }
            "--first-run" => first_run = true,
            "--managed" => managed = true,
            _ => {
                eprintln!("packwing: unknown option {a}\n{USAGE}");
                std::process::exit(2);
            }
        }
    }
    gpui_platform::application()
        .with_assets(Assets)
        .run(move |cx: &mut App| {
            load_fonts(cx);
            // GNOME's Wayland compositor draws no window frame, so Linux gets
            // client-side decorations. macOS and Windows keep the system's
            // frame but hide its title bar, and the app draws its own.
            let client_side = cfg!(any(target_os = "linux", target_os = "freebsd"));
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1280.), px(800.)),
                    cx,
                ))),
                window_min_size: Some(size(px(900.), px(600.))),
                window_decorations: client_side.then_some(WindowDecorations::Client),
                titlebar: (!client_side).then(|| TitlebarOptions {
                    title: Some("packwing".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(chrome::TRAFFIC_LIGHTS),
                }),
                app_id: Some("packwing".into()),
                ..Default::default()
            };
            let opened = cx.open_window(options, |window, cx| {
                let ws = cx.new(|cx| workspace::Workspace::new(first_run, managed, window, cx));
                window.focus(&ws.read(cx).focus_handle(cx), cx);
                ws
            });
            if opened.is_err() {
                eprintln!("packwing: cannot open a window");
                cx.quit();
                return;
            }
            cx.activate(true);
            // One window: closing it quits, on macOS too.
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
        });
}
