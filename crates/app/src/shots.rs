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

use crate::pages::settings::{Appearance, Section};
use crate::right_click::{Choice, RightClickMenu, Status};
use crate::workspace::{Page, Workspace};

type SetUp = fn(&mut Workspace, &mut Window, &mut Context<Workspace>);

/// Each screen's name, how to get to it, and whether packwing runs managed.
const SCREENS: [(&str, SetUp, bool); 18] = [
    ("archive", |_, _, _| {}, false),
    ("recent", |ws, _, cx| ws.show_page(Page::Recent, cx), false),
    (
        "settings",
        |ws, _, cx| ws.show_page(Page::Settings, cx),
        false,
    ),
    (
        "settings-extracting",
        |ws, _, cx| ws.show_settings(Section::Extracting, cx),
        false,
    ),
    (
        "settings-compressing",
        |ws, _, cx| ws.show_settings(Section::Compressing, cx),
        false,
    ),
    (
        "settings-security",
        |ws, _, cx| ws.show_settings(Section::Security, cx),
        false,
    ),
    (
        "settings-managed",
        |ws, _, cx| ws.show_settings(Section::Security, cx),
        true,
    ),
    (
        "settings-right-click",
        |ws, _, cx| ws.show_settings(Section::RightClick, cx),
        false,
    ),
    (
        "settings-right-click-notes",
        |ws, _, cx| {
            ws.show_settings(Section::RightClick, cx);
            let notes = vec![
                "Nautilus: the items are in its Scripts submenu, for any file chosen. With \
                 nautilus-python installed (python3-nautilus, nautilus-python), or the app's \
                 package putting in the native extension, they show in the menu itself."
                    .into(),
                "Nemo: the items are not grouped in a submenu; Nemo's Actions layout can \
                 group them."
                    .into(),
            ];
            ws.right_click()
                .update(cx, |m, cx| m.set_status(Status::Done(notes), cx));
        },
        false,
    ),
    (
        "settings-right-click-off",
        |ws, _, cx| {
            ws.show_settings(Section::RightClick, cx);
            ws.right_click().update(cx, |m, cx| {
                m.show(false, cx);
                m.set_status(
                    Status::Failed("right-click menus are not supported here yet".into()),
                    cx,
                );
            });
        },
        false,
    ),
    (
        "settings-updates",
        |ws, _, cx| ws.show_settings(Section::Updates, cx),
        false,
    ),
    (
        "settings-about",
        |ws, _, cx| ws.show_settings(Section::About, cx),
        false,
    ),
    (
        "first-run",
        |ws, window, cx| ws.show_welcome(window, cx),
        false,
    ),
    (
        "extract",
        |ws, window, cx| ws.open_extract(window, cx),
        false,
    ),
    (
        "create",
        |ws, window, cx| {
            ws.open_create(window, cx);
        },
        false,
    ),
    (
        "create-advanced",
        |ws, window, cx| {
            ws.open_create(window, cx)
                .update(cx, |d, cx| d.show_advanced(cx));
        },
        false,
    ),
    (
        "progress",
        |ws, window, cx| {
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
        },
        false,
    ),
    (
        "damaged",
        |ws, window, cx| ws.open_damaged(window, cx),
        false,
    ),
];

fn out_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/shots");
    std::fs::create_dir_all(&dir).expect("cannot make target/shots");
    dir
}

#[test]
fn shots() {
    let dir = out_dir();
    for (name, set_up, managed) in SCREENS {
        for dark in [false, true] {
            let text = Arc::new(gpui_wgpu::CosmicTextSystem::new(crate::FONT));
            let mut cx = HeadlessAppContext::with_platform(text, Arc::new(crate::Assets), || {
                gpui_platform::current_headless_renderer()
            });
            cx.update(crate::load_fonts);
            let window = cx
                .open_window(size(px(1280.), px(800.)), |window, cx| {
                    // Nothing is put in the system's menu.
                    let menu =
                        cx.new(|_| RightClickMenu::offline(true, Choice::default(), Status::Idle));
                    cx.new(|cx| Workspace::new(false, managed, menu, window, cx))
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
