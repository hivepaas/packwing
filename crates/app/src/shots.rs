//! Every screen drawn to a PNG, light and dark, with no display: GPUI's
//! headless app on its wgpu renderer (Mesa's lavapipe will do). To look at the
//! screens beside the design, or after a change:
//!
//! `cargo test -p packwing --features shots shots`
//!
//! writes them to `target/shots/`. One test draws them all in turn: the
//! palette in use is the app's one global, which tests at once would share.

use std::path::PathBuf;
use std::sync::Arc;

use gpui::{AppContext as _, Context, HeadlessAppContext, Window, px, size};

use crate::pages::settings::Appearance;
use crate::workspace::{Page, Workspace};

type SetUp = fn(&mut Workspace, &mut Window, &mut Context<Workspace>);

const SCREENS: [(&str, SetUp); 9] = [
    ("archive", |_, _, _| {}),
    ("recent", |ws, _, cx| ws.show_page(Page::Recent, cx)),
    ("settings", |ws, _, cx| ws.show_page(Page::Settings, cx)),
    ("first-run", |ws, window, cx| ws.show_welcome(window, cx)),
    ("extract", |ws, window, cx| ws.open_extract(window, cx)),
    ("create", |ws, window, cx| {
        ws.open_create(window, cx);
    }),
    ("create-advanced", |ws, window, cx| {
        ws.open_create(window, cx)
            .update(cx, |d, cx| d.show_advanced(cx));
    }),
    ("progress", |ws, window, cx| {
        let a = crate::demo::PROJECT;
        let job = crate::dialogs::Job {
            verb: crate::dialogs::Verb::Extract,
            name: a.name.into(),
            to: Some(format!("{} › {}", a.location, a.stem()).into()),
            files: a.files,
            bytes: a.unpacked,
        };
        ws.open_progress(job, window, cx)
            .update(cx, |p, cx| p.set_done(0.64, cx));
    }),
    ("damaged", |ws, window, cx| ws.open_damaged(window, cx)),
];

fn out_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/shots");
    std::fs::create_dir_all(&dir).expect("cannot make target/shots");
    dir
}

#[test]
fn shots() {
    let dir = out_dir();
    for (name, set_up) in SCREENS {
        for dark in [false, true] {
            let text = Arc::new(gpui_wgpu::CosmicTextSystem::new(crate::FONT));
            let mut cx = HeadlessAppContext::with_platform(text, Arc::new(crate::Assets), || {
                gpui_platform::current_headless_renderer()
            });
            cx.update(crate::load_fonts);
            let window = cx
                .open_window(size(px(1280.), px(800.)), |window, cx| {
                    cx.new(|cx| Workspace::new(false, window, cx))
                })
                .expect("cannot open a window");
            window
                .update(&mut cx, |ws, window, cx| {
                    let look = if dark {
                        Appearance::Dark
                    } else {
                        Appearance::Light
                    };
                    ws.set_appearance(look, window, cx);
                    set_up(ws, window, cx);
                })
                .expect("cannot set the screen up");
            cx.run_until_parked();
            let image = cx
                .capture_screenshot(window.into())
                .expect("cannot draw the window");
            let file = dir.join(format!(
                "{name}-{}.png",
                if dark { "dark" } else { "light" }
            ));
            image.save(&file).expect("cannot save the picture");
        }
    }
}
