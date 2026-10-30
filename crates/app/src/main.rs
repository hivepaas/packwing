//! packwing: a file archiver that opens archives safely.
//!
//! Usage: `packwing [--first-run] [--managed] [--<item> PATH…]`. Archives
//! are read, extracted and tested through packwing-core and hivepack
//! (`--open PATH` opens one); writing them is to come, and New archive,
//! Recent and the screenshots show made-up ones (`demo.rs`). `--first-run`
//! opens on the welcome; `--managed` runs as an organization would, with its
//! policies. An item of the right-click menu runs packwing with its option
//! and the paths chosen (`right_click.rs`).

// A GUI program on Windows: no console window opens beside it.
#![cfg_attr(windows, windows_subsystem = "windows")]

mod chrome;
mod colors;
mod demo;
mod dialogs;
mod input;
mod model;
mod open_with;
mod pages;
mod right_click;
mod widgets;
mod workspace;

#[cfg(all(test, feature = "shots", target_os = "linux"))]
mod shots;

use std::borrow::Cow;

use gpui::{
    App, AppContext as _, Bounds, Focusable as _, TitlebarOptions, WindowBounds, WindowDecorations,
    WindowOptions, px, size,
};

use right_click::{Action, Request, RightClickMenu};

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
    "appearance-dark",
    "appearance-light",
    "appearance-system",
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
    "logo-feather",
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

fn usage() -> String {
    let items: Vec<String> = Action::ALL.iter().map(|a| a.option()).collect();
    format!(
        "usage: packwing [--first-run] [--managed] [--<item> PATH…]
  --first-run  open on the welcome, as on the first start
  --managed    as an organization runs it: Settings › Security shows its policies
  --<item>     what an item of the right-click menu runs, with the paths chosen:
               {}",
        items.join(", ")
    )
}

fn main() {
    let (mut first_run, mut managed, mut request) = (false, false, None);
    let mut args = std::env::args_os().skip(1);
    while let Some(a) = args.next() {
        let text = a.to_string_lossy();
        match text.as_ref() {
            "-h" | "--help" => {
                println!("{}", usage());
                return;
            }
            "--first-run" => first_run = true,
            "--managed" => managed = true,
            option => match Request::parse(option, args.by_ref()) {
                Some(r) => request = Some(r),
                None => {
                    eprintln!("packwing: unknown option {text}\n{}", usage());
                    std::process::exit(2);
                }
            },
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
            let menu = cx.new(RightClickMenu::installed);
            open_with::keep_current(cx);
            let opened = cx.open_window(options, |window, cx| {
                let ws = cx.new(|cx| {
                    let mut ws = workspace::Workspace::new(first_run, managed, menu, window, cx);
                    if let Some(r) = request {
                        ws.start(r, window, cx);
                    }
                    ws
                });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_is_drawn_into_the_binary() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons");
        for entry in std::fs::read_dir(dir).expect("assets/icons") {
            let name = entry.expect("an icon").file_name();
            let path = format!("icons/{}", name.to_string_lossy());
            assert!(
                ICONS.iter().any(|(p, _)| *p == path),
                "{path} is not in ICONS"
            );
        }
    }
}
