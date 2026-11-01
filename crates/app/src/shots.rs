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

use crate::model::Archive;
use crate::pages::archive::Shown;
use crate::pages::settings::{Appearance, Section};
use crate::right_click::{Choice, RightClickMenu, Status};
use crate::workspace::{Page, Workspace};

type SetUp = fn(&mut Workspace, &mut Window, &mut Context<Workspace>);

/// Each screen's name, how to get to it, and whether packwing runs managed.
const SCREENS: [(&str, SetUp, bool); 31] = [
    ("archive", |_, _, _| {}, false),
    (
        "archive-opened",
        |ws, window, cx| ws.open_archive(sample_tar(), false, window, cx),
        false,
    ),
    (
        "archive-none",
        |ws, _, cx| ws.show_archive(Shown::Nothing, cx),
        false,
    ),
    (
        "archive-read",
        |ws, _, cx| ws.show_archive(read(), cx),
        false,
    ),
    (
        "archive-failed",
        |ws, _, cx| {
            ws.show_archive(
                Shown::Failed {
                    name: "photo.jpg".into(),
                    why: packwing_core::Error::NotAnArchive.to_string(),
                },
                cx,
            )
        },
        false,
    ),
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
            ws.open_create_sample(window, cx);
        },
        false,
    ),
    (
        "add",
        |ws, window, cx| ws.open_add_sample(window, cx),
        false,
    ),
    (
        "create-empty",
        |ws, window, cx| ws.open_create(Vec::new(), window, cx),
        false,
    ),
    (
        "create-advanced",
        |ws, window, cx| {
            ws.open_create_sample(window, cx)
                .update(cx, |d, cx| d.show_advanced(cx));
        },
        false,
    ),
    (
        "create-7z",
        |ws, window, cx| {
            ws.open_create_sample(window, cx).update(cx, |d, cx| {
                d.fill(crate::dialogs::Format::SevenZip, "100", cx)
            });
        },
        false,
    ),
    (
        "create-missing",
        |ws, window, cx| {
            ws.open_create_sample(window, cx).update(cx, |d, cx| {
                d.fill(crate::dialogs::Format::SevenZip, "", cx);
                d.press_create(cx);
            });
        },
        false,
    ),
    (
        "progress",
        |ws, window, cx| {
            let a = crate::demo::project();
            let job = crate::dialogs::Job::shown(
                crate::dialogs::Verb::Extract,
                a.name.clone().into(),
                Some(format!("{} › {}", a.location, a.stem()).into()),
                a.files,
                a.unpacked,
            );
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
    (
        "key-file",
        |ws, window, cx| ws.show_ask(packwing_core::Asked::KeyFile, None, window, cx),
        false,
    ),
    (
        "password",
        |ws, window, cx| ws.show_ask(packwing_core::Asked::Password, None, window, cx),
        false,
    ),
    (
        // An age file opened for real: its reading stops for the
        // passphrase, which is asked for.
        "passphrase-asked",
        |ws, window, cx| ws.open_archive(age_file(), false, window, cx),
        false,
    ),
    (
        "passphrase-wrong",
        |ws, window, cx| {
            let again = packwing_core::Error::WrongPassphrase.to_string();
            ws.show_ask(packwing_core::Asked::Passphrase, Some(again), window, cx)
        },
        false,
    ),
    (
        "progress-failed",
        |ws, window, cx| {
            let job = crate::dialogs::Job::shown(
                crate::dialogs::Verb::Extract,
                "site-backup-2026-10.tar.zst".into(),
                Some("Downloads › site-backup-2026-10".into()),
                6,
                131_000_000,
            );
            ws.open_progress(job, window, cx).update(cx, |p, cx| {
                p.set_failed(
                    "entry 3 (site/../evil.sh): the name climbs out of the folder".into(),
                    cx,
                )
            });
        },
        false,
    ),
];

/// A tar archive written for the screenshot to open through hivepack, as
/// `--open` does.
fn sample_tar() -> PathBuf {
    let mut tar = Vec::new();
    for (name, data) in [
        ("photos/", &b""[..]),
        ("photos/beach.jpg", &[7u8; 3000][..]),
        ("notes.txt", b"Read through hivepack.\n"),
    ] {
        let mut b = vec![0u8; 512];
        b[..name.len()].copy_from_slice(name.as_bytes());
        let dir = name.ends_with('/');
        b[100..108].copy_from_slice(if dir { b"0000755\0" } else { b"0000644\0" });
        b[124..136].copy_from_slice(format!("{:011o}\0", data.len()).as_bytes());
        b[136..148].copy_from_slice(b"15000000000\0");
        b[156] = if dir { b'5' } else { b'0' };
        b[257..265].copy_from_slice(b"ustar\x0000");
        b[148..156].fill(b' ');
        let sum: u32 = b.iter().map(|&c| u32::from(c)).sum();
        b[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
        tar.extend(b);
        tar.extend_from_slice(data);
        tar.resize(tar.len().div_ceil(512) * 512, 0);
    }
    tar.extend([0u8; 1024]);
    let path = out_dir().join("holiday-2026.tar");
    std::fs::write(&path, tar).expect("cannot write the sample tar");
    path
}

/// An age file encrypted to a passphrase, as packwing-core's tests keep it.
fn age_file() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../core/tests/data/notes.tar.zst.age")
}

/// An archive as packwing-core lists it, shown two folders down.
fn read() -> Shown {
    let entry = |path: &str, dir, size| packwing_core::Entry {
        path: path.split('/').map(str::to_owned).collect(),
        dir,
        size,
        modified: Some(1_760_000_000),
    };
    let listing = packwing_core::Listing {
        name: "site-backup-2026-10.tar.zst".into(),
        folder: PathBuf::from("/home/user/Downloads"),
        packed: 48_200_000,
        unpacked: 131_000_000,
        files: 6,
        layers: vec![packwing_core::Layer::Zstd],
        container: packwing_core::Container::Tar,
        encryption: None,
        entries: vec![
            entry("site", true, 0),
            entry("site/public", true, 0),
            entry("site/public/index.html", false, 18_000),
            entry("site/public/hero.jpg", false, 2_400_000),
            entry("site/public/fonts", true, 0),
            entry("site/public/fonts/Nunito.ttf", false, 280_000),
            entry("site/db.sql", false, 128_000_000),
            entry("site/README.md", false, 4_000),
            entry("site/deploy.sh", false, 2_000),
        ],
    };
    let mut a = Archive::read(
        PathBuf::from("/home/user/Downloads/site-backup-2026-10.tar.zst"),
        listing,
        packwing_core::Keys::default(),
        false,
    );
    a.show(vec!["site".into(), "public".into()]);
    Shown::Open(Box::new(a))
}

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
                    // The sample archive is open behind every screen.
                    ws.show_sample(cx);
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
