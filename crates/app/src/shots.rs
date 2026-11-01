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
const SCREENS: [(&str, SetUp, bool); 51] = [
    (
        "storages-copy",
        |ws, _, cx| {
            storage_with(ws, "readme.txt", cx);
            ws.explorer(cx)
                .update(cx, |e, cx| e.ask_transfer(false, cx));
        },
        false,
    ),
    (
        "storages",
        |ws, _, cx| {
            ws.show_storage(sample_storage(), cx);
        },
        false,
    ),
    (
        "storages-chosen",
        |ws, _, cx| storage_with(ws, "site-2026-10.zip", cx),
        false,
    ),
    (
        "storages-folder-info",
        |ws, _, cx| storage_with(ws, "assets", cx),
        false,
    ),
    (
        "storages-menu",
        |ws, _, cx| {
            storage_with(ws, "readme.txt", cx);
            ws.explorer(cx)
                .update(cx, |e, cx| e.show_menu(640., 330., false, cx));
        },
        false,
    ),
    (
        "storages-rename",
        |ws, window, cx| {
            storage_with(ws, "readme.txt", cx);
            ws.explorer(cx)
                .update(cx, |e, cx| e.type_rename(window, cx));
        },
        false,
    ),
    (
        "storages-delete",
        |ws, _, cx| {
            storage_with(ws, "readme.txt", cx);
            let explorer = ws.explorer(cx);
            explorer.update(cx, |e, cx| e.ask_delete(cx));
        },
        false,
    ),
    (
        "storages-view",
        |ws, _, cx| {
            storage_with(ws, "readme.txt", cx);
            let explorer = ws.explorer(cx);
            let item = explorer.read(cx).selection().remove(0);
            explorer.update(cx, |e, cx| e.open(&item, cx));
        },
        false,
    ),
    (
        "settings-editor",
        |ws, _, cx| ws.show_settings(Section::General, cx),
        false,
    ),
    ("archive", |_, _, _| {}, false),
    (
        "archive-opened",
        |ws, window, cx| ws.open_archive(sample_tar(), false, window, cx),
        false,
    ),
    (
        "archive-search",
        |ws, _, cx| {
            ws.show_archive(read(), cx);
            ws.search_archive("readme", cx);
        },
        false,
    ),
    (
        "archive-selected",
        |ws, _, cx| {
            ws.show_archive(read(), cx);
            ws.select_in_archive(&[1, 2], cx);
        },
        false,
    ),
    (
        "extract-selected",
        |ws, window, cx| {
            ws.show_archive(read(), cx);
            ws.select_in_archive(&[1, 2], cx);
            ws.open_extract(window, cx);
        },
        false,
    ),
    (
        "open-file",
        |ws, window, cx| {
            ws.show_archive(read(), cx);
            let entry = ["site", "public", "index.html"].map(String::from).to_vec();
            ws.open_entry(entry, window, cx);
        },
        false,
    ),
    (
        "open-program",
        |ws, window, cx| {
            ws.show_archive(read(), cx);
            let entry = ["site", "deploy.exe"].map(String::from).to_vec();
            ws.open_entry(entry, window, cx);
        },
        false,
    ),
    (
        "preview-picture",
        |ws, _, cx| {
            ws.show_archive(read(), cx);
            ws.select_in_archive(&[1], cx);
            ws.preview_in_archive(Some(sample_png()), "", cx);
        },
        false,
    ),
    (
        "preview-text",
        |ws, _, cx| {
            ws.show_archive(read(), cx);
            ws.select_in_archive(&[1], cx);
            let text = "# Trang của công ty\n\nCách chạy:\n\n    ./deploy.sh --production\n\nHỏi ai nếu có lỗi? Xem docs/.\n";
            ws.preview_in_archive(None, text, cx);
        },
        false,
    ),
    (
        "rename",
        |ws, window, cx| {
            ws.show_archive(read(), cx);
            ws.select_in_archive(&[1], cx);
            ws.open_edit(true, window, cx);
        },
        false,
    ),
    (
        "delete",
        |ws, window, cx| {
            ws.show_archive(read(), cx);
            ws.select_in_archive(&[1, 2], cx);
            ws.open_edit(false, window, cx);
        },
        false,
    ),
    (
        "archive-long-name",
        |ws, _, cx| ws.show_archive(read_named(LONG_NAME), cx),
        false,
    ),
    (
        "extract-long-name",
        |ws, window, cx| {
            ws.show_archive(read_named(LONG_NAME), cx);
            ws.open_extract(window, cx);
        },
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
    ("recent", |ws, _, cx| ws.show_recent_samples(cx), false),
    (
        "recent-empty",
        |ws, _, cx| ws.show_page(Page::Recent, cx),
        false,
    ),
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
        "create-chosen",
        |ws, window, cx| {
            ws.open_create_many(window, cx);
        },
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

/// A folder for the Storages screens.
fn sample_storage() -> PathBuf {
    let dir = out_dir().join("sample-storage");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("assets")).expect("cannot make a folder");
    std::fs::create_dir_all(dir.join("Báo cáo")).expect("cannot make a folder");
    std::fs::write(dir.join("assets/logo.svg"), "<svg/>").expect("cannot write");
    for (name, bytes) in [
        ("site-2026-10.zip", 42_000_000usize),
        ("photos.7z", 18_600_000),
        ("logs-2026-q3.tar.zst", 9_400_000),
        ("readme.txt", 2_100),
        ("notes.pdf", 1_400_000),
        ("deploy.sh", 840),
        ("hero.png", 480_000),
    ] {
        let f = std::fs::File::create(dir.join(name)).expect("cannot write");
        f.set_len(bytes as u64).expect("cannot size");
    }
    std::fs::write(
        dir.join("readme.txt"),
        "acme-backups\n\nSite archives are written on the last day of each month.\n",
    )
    .expect("cannot write");
    dir
}

/// The sample folder browsed, once listed, with `name` chosen.
fn storage_with(ws: &mut Workspace, name: &str, cx: &mut Context<Workspace>) {
    ws.show_storage(sample_storage(), cx);
    ws.explorer(cx).update(cx, |e, cx| {
        e.list_now(cx);
        e.choose(name, cx);
    });
}

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
    read_named("site-backup-2026-10.tar.zst")
}

/// A name too long for anything it is shown in.
const LONG_NAME: &str = "Báo-cáo-tài-chính-hợp-nhất-toàn-bộ-công-ty-và-các-đơn-vị-thành-viên-năm-2026-bản-cuối-cùng-đã-được-ban-giám-đốc-phê-duyệt-và-ký-tên-đóng-dấu.tar.zst";

fn read_named(name: &str) -> Shown {
    let entry = |path: &str, dir, size| packwing_core::Entry {
        path: path.split('/').map(str::to_owned).collect(),
        dir,
        size,
        modified: Some(1_760_000_000),
    };
    let listing = packwing_core::Listing {
        name: name.into(),
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

/// A small picture for the preview: a green and orange gradient.
fn sample_png() -> Vec<u8> {
    let picture = image::RgbImage::from_fn(320, 200, |x, y| {
        image::Rgb([(x * 255 / 320) as u8, 120, (y * 255 / 200) as u8])
    });
    let mut bytes = std::io::Cursor::new(Vec::new());
    picture
        .write_to(&mut bytes, image::ImageFormat::Png)
        .expect("cannot make a picture");
    bytes.into_inner()
}

fn out_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/shots");
    std::fs::create_dir_all(&dir).expect("cannot make target/shots");
    dir.canonicalize().expect("cannot find target/shots")
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

/// A press and a release at `at`, the pointer having gone there first, as a
/// person's click comes to the window.
fn click(
    cx: &mut HeadlessAppContext,
    window: gpui::WindowHandle<Workspace>,
    at: gpui::Point<gpui::Pixels>,
) {
    use gpui::{
        Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PlatformInput,
    };
    let events = [
        PlatformInput::MouseMove(MouseMoveEvent {
            position: at,
            pressed_button: None,
            modifiers: Modifiers::default(),
        }),
        PlatformInput::MouseDown(MouseDownEvent {
            button: MouseButton::Left,
            position: at,
            modifiers: Modifiers::default(),
            click_count: 1,
            first_mouse: false,
        }),
        PlatformInput::MouseUp(MouseUpEvent {
            button: MouseButton::Left,
            position: at,
            modifiers: Modifiers::default(),
            click_count: 1,
        }),
    ];
    for event in events {
        window
            .update(cx, |_, window, cx| {
                window.dispatch_event(event, cx);
            })
            .expect("cannot send the event");
        cx.run_until_parked();
        cx.capture_screenshot(window.into())
            .expect("cannot draw the window");
    }
}

/// An option of a menu takes a click, which a screenshot does not show.
#[test]
fn a_menu_option_takes_a_click() {
    let text = Arc::new(gpui_wgpu::CosmicTextSystem::new(crate::FONT));
    let mut cx = HeadlessAppContext::with_platform(text, Arc::new(crate::Assets), || {
        gpui_platform::current_headless_renderer()
    });
    cx.update(crate::load_fonts);
    let window = cx
        .open_window(size(px(1280.), px(800.)), |window, cx| {
            let menu = cx.new(|_| RightClickMenu::offline(true, Choice::default(), Status::Idle));
            cx.new(|cx| Workspace::new(false, false, menu, window, cx))
        })
        .expect("cannot open a window");
    let dialog = window
        .update(&mut cx, |ws, window, cx| {
            ws.show_sample(cx);
            ws.open_create_sample(window, cx)
        })
        .expect("cannot set the screen up");
    cx.run_until_parked();
    cx.capture_screenshot(window.into())
        .expect("cannot draw the window");
    // The format's pill, then the second option under it (.7z), as
    // `create-light.png` draws them.
    let (pill, option) = (
        gpui::point(px(890.), px(449.)),
        gpui::point(px(890.), px(540.)),
    );
    click(&mut cx, window, pill);
    click(&mut cx, window, option);
    let chosen = dialog.read_with(&cx, |d, cx| d.format_chosen(cx));
    assert_eq!(chosen, crate::dialogs::Format::SevenZip);
    // A press elsewhere closes it, and what was under the option then stays.
    click(&mut cx, window, pill);
    click(&mut cx, window, gpui::point(px(40.), px(700.)));
    click(&mut cx, window, gpui::point(px(890.), px(590.)));
    let chosen = dialog.read_with(&cx, |d, cx| d.format_chosen(cx));
    assert_eq!(chosen, crate::dialogs::Format::SevenZip);
}

/// A click on a real archive's file selects it, and reads it for the preview.
#[test]
fn a_click_on_a_file_shows_it_in_the_preview() {
    let folder = std::env::temp_dir().join(format!("packwing-preview-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).expect("cannot make a folder");
    let note = folder.join("ghi chú.txt");
    std::fs::write(&note, "xin chào\n").expect("cannot write a file");
    let archive = packwing_core::create(
        &[note],
        &folder.join("a.zip"),
        &packwing_core::NewArchive::default(),
        &packwing_core::Control::default(),
    )
    .expect("cannot make an archive");

    let text = Arc::new(gpui_wgpu::CosmicTextSystem::new(crate::FONT));
    let mut cx = HeadlessAppContext::with_platform(text, Arc::new(crate::Assets), || {
        gpui_platform::current_headless_renderer()
    });
    cx.update(crate::load_fonts);
    let window = cx
        .open_window(size(px(1280.), px(800.)), |window, cx| {
            let menu = cx.new(|_| RightClickMenu::offline(true, Choice::default(), Status::Idle));
            cx.new(|cx| Workspace::new(false, false, menu, window, cx))
        })
        .expect("cannot open a window");
    window
        .update(&mut cx, |ws, window, cx| {
            ws.open_archive(archive, false, window, cx)
        })
        .expect("cannot open the archive");
    let page = window
        .update(&mut cx, |ws, _, _| ws.archive_page())
        .expect("no page");
    // Reading happens on other threads.
    let wait_for = |cx: &mut HeadlessAppContext, done: &dyn Fn(&mut HeadlessAppContext) -> bool| {
        for _ in 0..200 {
            cx.run_until_parked();
            if done(cx) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    };
    let listed = page.clone();
    wait_for(&mut cx, &|cx| {
        listed.read_with(cx, |p, _| p.archive().is_some())
    });
    cx.capture_screenshot(window.into())
        .expect("cannot draw the window");
    // The first row of the list, as `archive-read-light.png` draws it.
    click(&mut cx, window, gpui::point(px(300.), px(307.)));
    let reading = page.clone();
    wait_for(&mut cx, &|cx| {
        !reading.read_with(cx, |p, _| p.previewed().1)
    });
    let (shown, picked) = page.read_with(&cx, |p, _| (p.previewed().0, p.picked()));
    assert_eq!(shown.as_deref(), Some("xin chào\n"));
    assert_eq!(picked, [vec!["ghi chú.txt".to_owned()]]);
    let _ = std::fs::remove_dir_all(&folder);
}

/// Files dragged over the window and let go at `at`.
fn drop_files(
    cx: &mut HeadlessAppContext,
    window: gpui::WindowHandle<Workspace>,
    at: gpui::Point<gpui::Pixels>,
    paths: Vec<PathBuf>,
) {
    use gpui::{ExternalPaths, FileDropEvent, PlatformInput};
    let events = [
        FileDropEvent::Entered {
            position: at,
            paths: ExternalPaths(paths.into_iter().collect()),
        },
        FileDropEvent::Pending { position: at },
        FileDropEvent::Submit { position: at },
    ];
    for event in events {
        // Not through the workspace: it takes the drop, and is not to be held.
        cx.update_window(window.into(), |_, window, cx| {
            window.dispatch_event(PlatformInput::FileDrop(event), cx);
        })
        .expect("cannot send the event");
        cx.run_until_parked();
        cx.capture_screenshot(window.into())
            .expect("cannot draw the window");
    }
}

/// A file dropped on the window makes a new archive of it, and more dropped on
/// that dialog go in it.
#[test]
fn a_drop_on_the_window_starts_a_new_archive_and_the_dialog_takes_more() {
    let text = Arc::new(gpui_wgpu::CosmicTextSystem::new(crate::FONT));
    let mut cx = HeadlessAppContext::with_platform(text, Arc::new(crate::Assets), || {
        gpui_platform::current_headless_renderer()
    });
    cx.update(crate::load_fonts);
    let window = cx
        .open_window(size(px(1280.), px(800.)), |window, cx| {
            let menu = cx.new(|_| RightClickMenu::offline(true, Choice::default(), Status::Idle));
            cx.new(|cx| Workspace::new(false, false, menu, window, cx))
        })
        .expect("cannot open a window");
    cx.run_until_parked();
    cx.capture_screenshot(window.into())
        .expect("cannot draw the window");
    let centre = gpui::point(px(640.), px(400.));
    drop_files(&mut cx, window, centre, vec![PathBuf::from("/tmp/a.txt")]);
    let opened = window
        .update(&mut cx, |ws, _, _| ws.dialog_open())
        .expect("no window");
    assert!(opened, "the drop opened no dialog");
    drop_files(&mut cx, window, centre, vec![PathBuf::from("/tmp/b.txt")]);
    let held = window
        .update(&mut cx, |ws, _, cx| ws.create_dialog_paths(cx))
        .expect("no window");
    assert_eq!(
        held,
        [PathBuf::from("/tmp/a.txt"), PathBuf::from("/tmp/b.txt")]
    );
}

/// In a large window the preview takes a quarter of the row.
#[test]
fn a_large_window_gives_the_preview_more() {
    let text = Arc::new(gpui_wgpu::CosmicTextSystem::new(crate::FONT));
    let mut cx = HeadlessAppContext::with_platform(text, Arc::new(crate::Assets), || {
        gpui_platform::current_headless_renderer()
    });
    cx.update(crate::load_fonts);
    let window = cx
        .open_window(size(px(1920.), px(1080.)), |window, cx| {
            let menu = cx.new(|_| RightClickMenu::offline(true, Choice::default(), Status::Idle));
            cx.new(|cx| Workspace::new(false, false, menu, window, cx))
        })
        .expect("cannot open a window");
    window
        .update(&mut cx, |ws, _, cx| {
            ws.show_archive(read(), cx);
            ws.select_in_archive(&[1], cx);
            ws.preview_in_archive(Some(sample_png()), "", cx);
        })
        .expect("cannot set the screen up");
    cx.run_until_parked();
    let image = cx
        .capture_screenshot(window.into())
        .expect("cannot draw the window");
    image
        .save(out_dir().join("preview-wide-light.png"))
        .expect("cannot save the picture");
}

/// A folder of `n` files, for one test.
fn folder_of(test: &str, n: usize) -> PathBuf {
    let dir = out_dir().join(format!("storage-{test}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("cannot make a folder");
    for i in 0..n {
        std::fs::write(dir.join(format!("file{i}.txt")), "x").expect("cannot write");
    }
    dir
}

/// A window with the Storages page browsing `dir`, listed.
fn explorer_on(
    dir: PathBuf,
) -> (
    HeadlessAppContext,
    gpui::WindowHandle<Workspace>,
    gpui::Entity<crate::explorer::Explorer>,
) {
    let text = Arc::new(gpui_wgpu::CosmicTextSystem::new(crate::FONT));
    let mut cx = HeadlessAppContext::with_platform(text, Arc::new(crate::Assets), || {
        gpui_platform::current_headless_renderer()
    });
    let window = cx
        .open_window(size(px(1280.), px(800.)), |window, cx| {
            let menu = cx.new(|_| RightClickMenu::offline(true, Choice::default(), Status::Idle));
            cx.new(|cx| Workspace::new(false, false, menu, window, cx))
        })
        .expect("cannot open a window");
    let explorer = window
        .update(&mut cx, |ws, _, cx| {
            ws.show_storage(dir, cx);
            ws.explorer(cx)
        })
        .expect("cannot show the page");
    cx.run_until_parked();
    (cx, window, explorer)
}

#[test]
fn a_folder_comes_a_page_at_a_time() {
    let dir = folder_of("pages", 2_300);
    let (mut cx, window, explorer) = explorer_on(dir);
    let names = |cx: &mut HeadlessAppContext| {
        window
            .update(cx, |ws, _, cx| ws.explorer(cx).read(cx).shown_names())
            .expect("the window")
    };
    let more = |cx: &mut HeadlessAppContext| {
        window
            .update(cx, |ws, _, cx| ws.explorer(cx).read(cx).has_more())
            .expect("the window")
    };
    assert_eq!(names(&mut cx).len(), crate::explorer::PAGE);
    assert!(more(&mut cx));
    // In the order people read: 2 before 10.
    assert_eq!(names(&mut cx)[..3], ["file0.txt", "file1.txt", "file2.txt"]);
    window
        .update(&mut cx, |_, _, cx| {
            explorer.update(cx, |e, cx| e.load_more(crate::explorer::PAGE, cx))
        })
        .expect("the window");
    cx.run_until_parked();
    assert_eq!(names(&mut cx).len(), 2 * crate::explorer::PAGE);
    assert!(more(&mut cx));
    // A search of what is loaded, then of all of it.
    window
        .update(&mut cx, |_, _, cx| {
            explorer.update(cx, |e, cx| e.search_for("file2299", cx))
        })
        .expect("the window");
    cx.run_until_parked();
    assert!(names(&mut cx).is_empty());
    window
        .update(&mut cx, |_, _, cx| {
            explorer.update(cx, |e, cx| e.load_more(usize::MAX, cx))
        })
        .expect("the window");
    cx.run_until_parked();
    assert_eq!(names(&mut cx), ["file2299.txt"]);
    assert!(!more(&mut cx));
}

#[test]
fn a_name_typed_makes_a_folder_and_renames_a_file() {
    let dir = folder_of("names", 3);
    let (mut cx, window, explorer) = explorer_on(dir.clone());
    window
        .update(&mut cx, |_, window, cx| {
            explorer.update(cx, |e, cx| {
                e.type_new_folder(window, cx);
                e.type_name("Thư mục", cx);
                e.commit_name(window, cx);
            })
        })
        .expect("the window");
    cx.run_until_parked();
    assert!(dir.join("Thư mục").is_dir());
    window
        .update(&mut cx, |_, window, cx| {
            explorer.update(cx, |e, cx| {
                assert_eq!(e.selection().remove(0).name, "Thư mục");
                e.choose("file1.txt", cx);
                e.type_rename(window, cx);
                // Taken: it says so, and the name can be typed again.
                e.type_name("file2.txt", cx);
                e.commit_name(window, cx);
            })
        })
        .expect("the window");
    cx.run_until_parked();
    window
        .update(&mut cx, |_, window, cx| {
            explorer.update(cx, |e, cx| {
                assert!(e.problem().is_some());
                e.type_name("../out.txt", cx);
                e.commit_name(window, cx);
            })
        })
        .expect("the window");
    cx.run_until_parked();
    assert!(dir.join("file1.txt").exists());
    assert!(!dir.parent().expect("a parent").join("out.txt").exists());
    window
        .update(&mut cx, |_, window, cx| {
            explorer.update(cx, |e, cx| {
                e.type_name("đổi tên.txt", cx);
                e.commit_name(window, cx);
            })
        })
        .expect("the window");
    cx.run_until_parked();
    assert!(dir.join("đổi tên.txt").exists());
    assert!(!dir.join("file1.txt").exists());
    let shown = explorer.read_with(&cx, |e, _| e.shown_names());
    assert!(shown.contains(&"đổi tên.txt".to_owned()), "{shown:?}");
}

#[test]
fn a_folder_that_cannot_be_read_says_so() {
    let dir = out_dir().join("storage-not-there");
    let _ = std::fs::remove_dir_all(&dir);
    let (mut cx, window, _) = explorer_on(dir);
    cx.run_until_parked();
    let image = cx
        .capture_screenshot(window.into())
        .expect("cannot draw the window");
    image
        .save(out_dir().join("storages-not-there-light.png"))
        .expect("cannot save the picture");
}

/// A press and a release of `button`, the pointer having gone there first.
fn press(
    cx: &mut HeadlessAppContext,
    window: gpui::WindowHandle<Workspace>,
    at: gpui::Point<gpui::Pixels>,
    button: gpui::MouseButton,
    clicks: usize,
) {
    use gpui::{Modifiers, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PlatformInput};
    let mut events = vec![PlatformInput::MouseMove(MouseMoveEvent {
        position: at,
        pressed_button: None,
        modifiers: Modifiers::default(),
    })];
    for click_count in 1..=clicks {
        events.push(PlatformInput::MouseDown(MouseDownEvent {
            button,
            position: at,
            modifiers: Modifiers::default(),
            click_count,
            first_mouse: false,
        }));
        events.push(PlatformInput::MouseUp(MouseUpEvent {
            button,
            position: at,
            modifiers: Modifiers::default(),
            click_count,
        }));
    }
    for event in events {
        window
            .update(cx, |_, window, cx| {
                window.dispatch_event(event, cx);
            })
            .expect("cannot send the event");
        cx.run_until_parked();
        cx.capture_screenshot(window.into())
            .expect("cannot draw the window");
    }
}

#[test]
fn the_mouse_chooses_opens_and_goes_up() {
    use gpui::{MouseButton, point};
    let dir = out_dir().join("storage-mouse");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("inner")).expect("cannot make a folder");
    std::fs::write(dir.join("inner/deep.txt"), "x").expect("cannot write");
    std::fs::write(dir.join("one.txt"), "x").expect("cannot write");
    std::fs::write(dir.join("two.txt"), "x").expect("cannot write");
    let (mut cx, window, explorer) = explorer_on(dir);
    cx.capture_screenshot(window.into())
        .expect("cannot draw the window");
    // The rows, as `storages-light.png` draws them.
    let row = |n: f32| point(px(620.), px(141. + 48. * n));
    press(&mut cx, window, row(1.), MouseButton::Left, 1);
    let chosen = |cx: &HeadlessAppContext| {
        explorer.read_with(cx, |e, _| {
            e.selection()
                .into_iter()
                .map(|i| i.name)
                .collect::<Vec<_>>()
        })
    };
    assert_eq!(chosen(&cx), ["one.txt"]);
    // A right click on another chooses it alone.
    press(&mut cx, window, row(2.), MouseButton::Right, 1);
    assert_eq!(chosen(&cx), ["two.txt"]);
    // A click beside the menu closes it; another, beside the rows, chooses nothing.
    press(
        &mut cx,
        window,
        point(px(430.), px(650.)),
        MouseButton::Left,
        1,
    );
    press(
        &mut cx,
        window,
        point(px(430.), px(650.)),
        MouseButton::Left,
        1,
    );
    assert!(chosen(&cx).is_empty());
    // A double click goes into a folder.
    press(&mut cx, window, row(0.), MouseButton::Left, 2);
    assert_eq!(
        explorer.read_with(&cx, |e, _| e.shown_names()),
        ["deep.txt"]
    );
}

#[test]
fn moving_runs_as_a_job_and_lists_again() {
    let top = out_dir().join("storage-transfer");
    let _ = std::fs::remove_dir_all(&top);
    let (a, b) = (top.join("a"), top.join("b"));
    std::fs::create_dir_all(&a).expect("cannot make a folder");
    std::fs::create_dir_all(&b).expect("cannot make a folder");
    std::fs::write(a.join("đi.txt"), "x").expect("cannot write");
    let (mut cx, window, explorer) = explorer_on(a.clone());
    window
        .update(&mut cx, |ws, window, cx| {
            explorer.update(cx, |e, cx| e.choose("đi.txt", cx));
            ws.start_transfer_to(true, &b, window, cx);
        })
        .expect("the window");
    cx.run_until_parked();
    assert!(b.join("đi.txt").exists());
    assert!(!a.join("đi.txt").exists());
    window
        .update(&mut cx, |ws, _, _| assert!(!ws.dialog_open()))
        .expect("the window");
    assert!(explorer.read_with(&cx, |e, _| e.shown_names()).is_empty());
}

#[test]
fn a_large_window_gives_the_info_a_quarter() {
    let text = Arc::new(gpui_wgpu::CosmicTextSystem::new(crate::FONT));
    let mut cx = HeadlessAppContext::with_platform(text, Arc::new(crate::Assets), || {
        gpui_platform::current_headless_renderer()
    });
    cx.update(crate::load_fonts);
    let window = cx
        .open_window(size(px(1920.), px(1080.)), |window, cx| {
            let menu = cx.new(|_| RightClickMenu::offline(true, Choice::default(), Status::Idle));
            cx.new(|cx| Workspace::new(false, false, menu, window, cx))
        })
        .expect("cannot open a window");
    window
        .update(&mut cx, |ws, _, cx| {
            storage_with(ws, "site-2026-10.zip", cx)
        })
        .expect("cannot set the screen up");
    cx.run_until_parked();
    let image = cx
        .capture_screenshot(window.into())
        .expect("cannot draw the window");
    image
        .save(out_dir().join("storages-wide-light.png"))
        .expect("cannot save the picture");
}

/// The list of what goes in takes real clicks: `✕` takes a row out, and the eye
/// closes it.
#[test]
fn a_click_takes_a_row_out_and_the_eye_closes_the_list() {
    let text = Arc::new(gpui_wgpu::CosmicTextSystem::new(crate::FONT));
    let mut cx = HeadlessAppContext::with_platform(text, Arc::new(crate::Assets), || {
        gpui_platform::current_headless_renderer()
    });
    cx.update(crate::load_fonts);
    let window = cx
        .open_window(size(px(1280.), px(800.)), |window, cx| {
            let menu = cx.new(|_| RightClickMenu::offline(true, Choice::default(), Status::Idle));
            cx.new(|cx| Workspace::new(false, false, menu, window, cx))
        })
        .expect("cannot open a window");
    let dialog = window
        .update(&mut cx, |ws, window, cx| {
            ws.show_sample(cx);
            ws.open_create_many(window, cx)
        })
        .expect("cannot set the screen up");
    cx.run_until_parked();
    cx.capture_screenshot(window.into())
        .expect("cannot draw the window");
    let paths = dialog.read_with(&cx, |d, _| d.selection_paths());
    assert_eq!(paths.len(), 9);
    // As `create-chosen-light.png` draws them: the first row's `✕`, and the eye.
    click(&mut cx, window, gpui::point(px(915.), px(207.)));
    let left = dialog.read_with(&cx, |d, _| d.selection_paths());
    assert_eq!(left, paths[1..]);
    assert!(dialog.read_with(&cx, |d, _| d.list_is_open()));
    click(&mut cx, window, gpui::point(px(722.), px(148.)));
    assert!(!dialog.read_with(&cx, |d, _| d.list_is_open()));
    assert_eq!(
        dialog.read_with(&cx, |d, _| d.selection_paths()),
        paths[1..]
    );
}
