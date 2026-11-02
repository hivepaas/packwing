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

/// Each screen's name, and how to get to it.
const SCREENS: [(&str, SetUp); 58] = [
    ("storages-edit", |ws, window, cx| {
        cx.set_global(fake_extensions());
        add_far(ws, "Far one", std::path::Path::new("/far"), true, cx);
        let page = ws.storages_page();
        let last = page.read(cx).count() - 1;
        ws.edit_storage(last, window, cx);
    }),
    ("storages-edit-local", |ws, window, cx| {
        ws.show_storage(sample_storage(), cx);
        let page = ws.storages_page();
        let last = page.read(cx).count() - 1;
        ws.edit_storage(last, window, cx);
    }),
    ("storages-locked", |ws, _, cx| {
        cx.set_global(fake_extensions());
        ws.show_storage(sample_storage(), cx);
        add_far(ws, "Far one", std::path::Path::new("/far"), true, cx);
    }),
    ("storages-add", |ws, window, cx| {
        cx.set_global(fake_extensions());
        ws.open_add_storage(window, cx);
    }),
    ("storages-copy", |ws, _, cx| {
        storage_with(ws, "readme.txt", cx);
        ws.explorer(cx)
            .update(cx, |e, cx| e.ask_transfer(false, cx));
    }),
    ("storages", |ws, _, cx| {
        ws.show_storage(sample_storage(), cx);
    }),
    ("storages-chosen", |ws, _, cx| {
        storage_with(ws, "site-2026-10.zip", cx)
    }),
    ("storages-folder-info", |ws, _, cx| {
        storage_with(ws, "assets", cx)
    }),
    ("storages-menu", |ws, _, cx| {
        storage_with(ws, "readme.txt", cx);
        ws.explorer(cx)
            .update(cx, |e, cx| e.show_menu(640., 330., false, cx));
    }),
    ("storages-rename", |ws, window, cx| {
        storage_with(ws, "readme.txt", cx);
        ws.explorer(cx)
            .update(cx, |e, cx| e.type_rename(window, cx));
    }),
    ("storages-delete", |ws, _, cx| {
        storage_with(ws, "readme.txt", cx);
        let explorer = ws.explorer(cx);
        explorer.update(cx, |e, cx| e.ask_delete(cx));
    }),
    ("storages-view", |ws, _, cx| {
        storage_with(ws, "readme.txt", cx);
        let explorer = ws.explorer(cx);
        let item = explorer.read(cx).selection().remove(0);
        explorer.update(cx, |e, cx| e.open(&item, cx));
    }),
    ("settings-editor", |ws, _, cx| {
        ws.show_settings(Section::General, cx)
    }),
    ("archive", |_, _, _| {}),
    ("archive-opened", |ws, window, cx| {
        ws.open_archive(sample_tar(), false, window, cx)
    }),
    ("archive-search", |ws, _, cx| {
        ws.show_archive(read(), cx);
        ws.search_archive("readme", cx);
    }),
    ("archive-selected", |ws, _, cx| {
        ws.show_archive(read(), cx);
        ws.select_in_archive(&[1, 2], cx);
    }),
    ("extract-selected", |ws, window, cx| {
        ws.show_archive(read(), cx);
        ws.select_in_archive(&[1, 2], cx);
        ws.open_extract(window, cx);
    }),
    ("open-file", |ws, window, cx| {
        ws.show_archive(read(), cx);
        let entry = ["site", "public", "index.html"].map(String::from).to_vec();
        ws.open_entry(entry, window, cx);
    }),
    ("open-program", |ws, window, cx| {
        ws.show_archive(read(), cx);
        let entry = ["site", "deploy.exe"].map(String::from).to_vec();
        ws.open_entry(entry, window, cx);
    }),
    ("preview-picture", |ws, _, cx| {
        ws.show_archive(read(), cx);
        ws.select_in_archive(&[1], cx);
        ws.preview_in_archive(Some(sample_png()), "", cx);
    }),
    ("preview-text", |ws, _, cx| {
        ws.show_archive(read(), cx);
        ws.select_in_archive(&[1], cx);
        let text = "# Trang của công ty\n\nCách chạy:\n\n    ./deploy.sh --production\n\nHỏi ai nếu có lỗi? Xem docs/.\n";
        ws.preview_in_archive(None, text, cx);
    }),
    ("rename", |ws, window, cx| {
        ws.show_archive(read(), cx);
        ws.select_in_archive(&[1], cx);
        ws.open_edit(true, window, cx);
    }),
    ("delete", |ws, window, cx| {
        ws.show_archive(read(), cx);
        ws.select_in_archive(&[1, 2], cx);
        ws.open_edit(false, window, cx);
    }),
    ("archive-long-name", |ws, _, cx| {
        ws.show_archive(read_named(LONG_NAME), cx)
    }),
    ("extract-long-name", |ws, window, cx| {
        ws.show_archive(read_named(LONG_NAME), cx);
        ws.open_extract(window, cx);
    }),
    ("archive-none", |ws, _, cx| {
        ws.show_archive(Shown::Nothing, cx)
    }),
    ("archive-read", |ws, _, cx| ws.show_archive(read(), cx)),
    ("archive-failed", |ws, _, cx| {
        ws.show_archive(
            Shown::Failed {
                name: "photo.jpg".into(),
                why: packwing_core::Error::NotAnArchive.to_string(),
            },
            cx,
        )
    }),
    ("browse", |ws, _, cx| {
        let sample = sample_storage();
        ws.browse_page().update(cx, |b, cx| b.go_to(&sample, cx));
        ws.show_page(Page::Browse, cx);
    }),
    ("recent", |ws, _, cx| ws.show_recent_samples(cx)),
    ("recent-empty", |ws, _, cx| ws.show_recent_tab(cx)),
    ("settings", |ws, _, cx| ws.show_page(Page::Settings, cx)),
    ("settings-extracting", |ws, _, cx| {
        ws.show_settings(Section::Extracting, cx)
    }),
    ("settings-compressing", |ws, _, cx| {
        ws.show_settings(Section::Compressing, cx)
    }),
    ("settings-managed", |ws, _, cx| {
        cx.set_global(sample_policy());
        ws.show_settings(Section::Security, cx);
    }),
    ("create-required", |ws, window, cx| {
        cx.set_global(sample_policy());
        ws.open_create(vec!["/home/you/Documents".into()], window, cx);
    }),
    ("settings-security", |ws, _, cx| {
        ws.show_settings(Section::Security, cx)
    }),
    ("settings-right-click", |ws, _, cx| {
        ws.show_settings(Section::RightClick, cx)
    }),
    ("settings-right-click-notes", |ws, _, cx| {
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
    }),
    ("settings-right-click-off", |ws, _, cx| {
        ws.show_settings(Section::RightClick, cx);
        ws.right_click().update(cx, |m, cx| {
            m.show(false, cx);
            m.set_status(
                Status::Failed("right-click menus are not supported here yet".into()),
                cx,
            );
        });
    }),
    ("settings-about", |ws, _, cx| {
        ws.show_settings(Section::About, cx)
    }),
    ("first-run", |ws, window, cx| ws.show_welcome(window, cx)),
    ("extract", |ws, window, cx| ws.open_extract(window, cx)),
    ("create", |ws, window, cx| {
        ws.open_create_sample(window, cx);
    }),
    ("add", |ws, window, cx| ws.open_add_sample(window, cx)),
    ("create-chosen", |ws, window, cx| {
        ws.open_create_many(window, cx);
    }),
    ("create-empty", |ws, window, cx| {
        ws.open_create(Vec::new(), window, cx)
    }),
    ("create-advanced", |ws, window, cx| {
        ws.open_create_sample(window, cx)
            .update(cx, |d, cx| d.show_advanced(cx));
    }),
    ("create-7z", |ws, window, cx| {
        ws.open_create_sample(window, cx).update(cx, |d, cx| {
            d.fill(crate::dialogs::Format::SevenZip, "100", cx)
        });
    }),
    ("create-missing", |ws, window, cx| {
        ws.open_create_sample(window, cx).update(cx, |d, cx| {
            d.fill(crate::dialogs::Format::SevenZip, "", cx);
            d.press_create(cx);
        });
    }),
    ("progress", |ws, window, cx| {
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
    }),
    ("key-file", |ws, window, cx| {
        ws.show_ask(packwing_core::Asked::KeyFile, None, window, cx)
    }),
    ("password", |ws, window, cx| {
        ws.show_ask(packwing_core::Asked::Password, None, window, cx)
    }),
    (
        // An age file opened for real: its reading stops for the
        // passphrase, which is asked for.
        "passphrase-asked",
        |ws, window, cx| ws.open_archive(age_file(), false, window, cx),
    ),
    ("passphrase-wrong", |ws, window, cx| {
        let again = packwing_core::Error::WrongPassphrase.to_string();
        ws.show_ask(packwing_core::Asked::Passphrase, Some(again), window, cx)
    }),
    ("test-passed", |ws, window, cx| {
        let job = crate::dialogs::Job::shown(
            crate::dialogs::Verb::Test,
            "site-backup-2026-10.tar.zst".into(),
            None,
            1284,
            131_000_000,
        );
        ws.open_progress(job, window, cx)
            .update(cx, |p, cx| p.set_passed(1284, 131_000_000, cx));
    }),
    ("progress-failed", |ws, window, cx| {
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
    }),
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
        PathBuf::from("/home/user/Downloads/site-backup-2026-10.tar.zst").into(),
        listing,
        packwing_core::Keys::default(),
        false,
    );
    a.show(vec!["site".into(), "public".into()]);
    Shown::Open(Box::new(a))
}

/// What an organization might fix, with one mistake in it.
fn sample_policy() -> crate::managed::Shown {
    use packwing_policy::{
        ALLOWED_ENCRYPTION_METHODS, Found, MIN_PASSWORD_LENGTH, ORGANIZATION_NAME,
        ORGANIZATION_URL, PROPAGATE_MOTW, Policy, REQUIRE_ENCRYPTION, Value,
    };
    let text = |t: &str| Value::Text(t.into());
    crate::managed::Shown(Policy::of(Found {
        origin: "the sample".into(),
        values: vec![
            (ORGANIZATION_NAME.into(), text("IT department, Example Co.")),
            (ORGANIZATION_URL.into(), text("https://it.example.com")),
            (PROPAGATE_MOTW.into(), Value::Bool(true)),
            (REQUIRE_ENCRYPTION.into(), Value::Bool(true)),
            (
                ALLOWED_ENCRYPTION_METHODS.into(),
                text("7z-aes256, age-key"),
            ),
            (MIN_PASSWORD_LENGTH.into(), Value::Int(15)),
            ("BlockEverything".into(), Value::Bool(true)),
        ],
        failure: None,
    }))
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
    for (name, set_up) in SCREENS {
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
                    cx.new(|cx| Workspace::new(false, menu, window, cx))
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
            cx.new(|cx| Workspace::new(false, menu, window, cx))
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
            cx.new(|cx| Workspace::new(false, menu, window, cx))
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
            cx.new(|cx| Workspace::new(false, menu, window, cx))
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
            cx.new(|cx| Workspace::new(false, menu, window, cx))
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
            cx.new(|cx| Workspace::new(false, menu, window, cx))
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
            cx.new(|cx| Workspace::new(false, menu, window, cx))
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

/// A folder that says it is not on this computer, as a storage across a network.
struct Far(packwing_core::LocalFolder);

impl packwing_core::Storage for Far {
    fn list(&self, f: &[String]) -> std::io::Result<Box<dyn packwing_core::Lister>> {
        self.0.list(f)
    }
    fn read(&self, p: &[String], most: u64) -> std::io::Result<Vec<u8>> {
        self.0.read(p, most)
    }
    fn read_at(&self, p: &[String], offset: u64, len: usize) -> std::io::Result<Vec<u8>> {
        self.0.read_at(p, offset, len)
    }
    fn stat(&self, f: &[String], n: &str) -> std::io::Result<Option<packwing_core::Item>> {
        self.0.stat(f, n)
    }
    fn open(&self, p: &[String]) -> std::io::Result<Box<dyn std::io::Read + Send>> {
        self.0.open(p)
    }
    fn put(
        &self,
        p: &[String],
        from: &mut dyn std::io::Read,
        like: &packwing_core::Item,
        replace: bool,
    ) -> std::io::Result<u64> {
        self.0.put(p, from, like, replace)
    }
    fn remove(&self, p: &[String]) -> std::io::Result<()> {
        self.0.remove(p)
    }
    fn make_folder(&self, f: &[String], n: &str) -> std::io::Result<()> {
        self.0.make_folder(f, n)
    }
    fn rename(&self, f: &[String], a: &str, b: &str) -> std::io::Result<()> {
        self.0.rename(f, a, b)
    }
    fn delete(&self, f: &[String], n: &[String]) -> std::io::Result<()> {
        self.0.delete(f, n)
    }
    fn has_trash(&self) -> bool {
        false
    }
    fn local_path(&self, _: &[String]) -> Option<PathBuf> {
        None
    }
    fn show_path(&self, p: &[String]) -> String {
        format!("far://{}", p.join("/"))
    }
}

/// A kind of storage, as a program adds them: its setting `folder` is the
/// folder it is, and its secret goes where the test can see it.
struct FarKind(Arc<std::sync::Mutex<Vec<(String, packwing_core::Values)>>>);

impl packwing_core::StorageKind for FarKind {
    fn id(&self) -> &'static str {
        "far"
    }
    fn tag(&self) -> &'static str {
        "FAR"
    }
    fn name(&self) -> String {
        "Far bucket".into()
    }
    fn fields(&self) -> Vec<packwing_core::Field> {
        let field = |key, label: &str, secret| packwing_core::Field {
            key,
            label: label.into(),
            hint: String::new(),
            secret,
        };
        vec![
            field("folder", "Folder", false),
            field("token", "Token", true),
        ]
    }
    fn describe(&self, values: &packwing_core::Values) -> String {
        format!("Far · {}", values.get("folder").map_or("", String::as_str))
    }
    fn open(
        &self,
        _: &str,
        values: &packwing_core::Values,
        _: Option<&packwing_core::Values>,
    ) -> Arc<dyn packwing_core::Storage> {
        let folder = values.get("folder").cloned().unwrap_or_default();
        Arc::new(Far(packwing_core::LocalFolder::new(folder)))
    }
    fn kept_secrets(&self, id: &str) -> std::io::Result<packwing_core::Values> {
        Ok(self
            .0
            .lock()
            .expect("the list")
            .iter()
            .rev()
            .find(|(i, _)| i == id)
            .map(|(_, secrets)| secrets.clone())
            .unwrap_or_default())
    }
    fn keep_secrets(&self, id: &str, secrets: &packwing_core::Values) -> std::io::Result<()> {
        self.0
            .lock()
            .expect("the list")
            .push((id.to_owned(), secrets.clone()));
        Ok(())
    }
    fn forget_secrets(&self, id: &str) {
        self.0.lock().expect("the list").retain(|(i, _)| i != id);
    }
}

/// Adds a far storage of `folder`, chosen, that asks first if told to.
fn add_far(
    ws: &mut Workspace,
    name: &str,
    folder: &std::path::Path,
    asks: bool,
    cx: &mut Context<Workspace>,
) {
    let mut values: packwing_core::Values =
        [("folder".to_owned(), folder.display().to_string())].into();
    if asks {
        values.insert(crate::prefs::ASKS_FIRST.to_owned(), "yes".to_owned());
    }
    let secrets = [("token".to_owned(), "kept".to_owned())].into();
    ws.storages_page().update(cx, |p, cx| {
        p.add_other("far", name.into(), values, &secrets, cx)
            .expect("added");
    });
}

/// Says yes to one password, and counts how it was asked.
struct Confirms(&'static str);

impl crate::confirm::Confirmer for Confirms {
    fn how(&self) -> Option<crate::confirm::How> {
        Some(crate::confirm::How::Password)
    }
    fn ask(&self, _: &str, _: &str) -> Result<bool, String> {
        Err("it is typed here".into())
    }
    fn check_password(&self, password: &str) -> Result<bool, String> {
        Ok(password == self.0)
    }
}

#[test]
fn a_storage_that_asks_first_shows_nothing_until_it_has() {
    let far = folder_of("far-asks", 2);
    let (mut cx, window, explorer) = explorer_on(out_dir());
    let page = window
        .update(&mut cx, |ws, _, cx| {
            cx.set_global(fake_extensions());
            cx.set_global(crate::confirm::ConfirmWith(Arc::new(Confirms("right"))));
            add_far(ws, "Far one", &far, true, cx);
            ws.storages_page()
        })
        .expect("the window");
    cx.run_until_parked();
    let shut = |cx: &mut HeadlessAppContext| {
        window
            .update(cx, |_, _, cx| {
                let p = page.read(cx);
                let offered = p.places(cx).0.iter().any(|s| s.name == "Far one");
                (
                    p.is_chosen_locked(),
                    offered,
                    explorer.read(cx).place().is_some(),
                )
            })
            .expect("the window")
    };
    assert_eq!(shut(&mut cx), (true, false, false));

    let say = |cx: &mut HeadlessAppContext, typed: &'static str| {
        window
            .update(cx, |_, _, cx| {
                page.update(cx, |p, cx| p.type_password(typed, cx))
            })
            .expect("the window");
        cx.run_until_parked();
    };
    say(&mut cx, "wrong");
    assert_eq!(shut(&mut cx), (true, false, false));
    say(&mut cx, "");
    assert_eq!(shut(&mut cx), (true, false, false));
    say(&mut cx, "right");
    assert_eq!(shut(&mut cx), (false, true, true));

    // Used, it stays open; left, it shuts, and shows nothing again.
    let left = window
        .update(&mut cx, |_, _, cx| {
            page.update(cx, |p, cx| p.leave_for_long(cx))
        })
        .expect("the window");
    assert!(left);
    cx.run_until_parked();
    assert_eq!(shut(&mut cx), (true, false, false));
}

#[test]
fn a_storage_that_asks_first_asks_before_it_is_removed_or_edited() {
    let far = folder_of("far-asks-remove", 0);
    let (mut cx, window, _explorer) = explorer_on(out_dir());
    let page = window
        .update(&mut cx, |ws, _, cx| {
            cx.set_global(fake_extensions());
            cx.set_global(crate::confirm::ConfirmWith(Arc::new(Confirms("right"))));
            add_far(ws, "Far one", &far, true, cx);
            ws.storages_page()
        })
        .expect("the window");
    let count = |cx: &mut HeadlessAppContext| {
        window
            .update(cx, |_, _, cx| page.read(cx).count())
            .expect("the window")
    };
    let before = count(&mut cx);
    window
        .update(&mut cx, |_, window, cx| {
            page.update(cx, |p, cx| p.press_remove(before - 1, window, cx))
        })
        .expect("the window");
    cx.run_until_parked();
    assert_eq!(count(&mut cx), before, "still there, until it has asked");
    window
        .update(&mut cx, |_, _, cx| {
            page.update(cx, |p, cx| p.type_password("right", cx))
        })
        .expect("the window");
    cx.run_until_parked();
    assert_eq!(count(&mut cx), before - 1, "gone once it has");
}

#[test]
fn editing_a_storage_keeps_it_and_changes_what_was_changed() {
    let far = folder_of("far-edit", 0);
    let other = folder_of("far-edit-other", 1);
    let (mut cx, window, explorer) = explorer_on(out_dir());
    let (page, dialog) = window
        .update(&mut cx, |ws, window, cx| {
            cx.set_global(fake_extensions());
            add_far(ws, "Far one", &far, false, cx);
            let page = ws.storages_page();
            let last = page.read(cx).count() - 1;
            let dialog = cx.new(|cx| {
                crate::dialogs::AddStorageDialog::editing(page.clone(), last, window, cx)
            });
            (page, dialog)
        })
        .expect("the window");
    let other_text = other.display().to_string();
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| {
                // It starts with what it has, its secret too.
                assert_eq!(d.field_text("folder", cx), far.display().to_string());
                assert_eq!(d.field_text("token", cx), "kept");
                d.set_field("folder", &other_text, cx);
                d.set_name("Far renamed", cx);
                d.press_asks_first(cx);
                d.press_save(cx);
            })
        })
        .expect("the window");
    cx.run_until_parked();
    window
        .update(&mut cx, |_, _, cx| {
            let p = page.read(cx);
            let s = p.storage(p.count() - 1).expect("still there");
            assert_eq!(s.name, "Far renamed");
            assert!(s.asks_first());
            let crate::prefs::KeptPlace::Other { id, values, .. } = &s.place else {
                panic!("still of its kind");
            };
            assert_eq!(values.get("folder"), Some(&other_text));
            let kinds = fake_kind_secrets(cx, id);
            assert_eq!(kinds.get("token").map(String::as_str), Some("kept"));
            assert!(p.is_chosen_locked());
            assert!(explorer.read(cx).place().is_none());
        })
        .expect("the window");
}

fn fake_kind_secrets(cx: &gpui::App, id: &str) -> packwing_core::Values {
    cx.global::<crate::Extensions>().storage_kinds[0]
        .kept_secrets(id)
        .expect("kept")
}

#[test]
fn a_local_storage_is_not_moved_to_a_folder_that_is_not_there() {
    let here = folder_of("edit-local", 0);
    let moved = folder_of("edit-local-moved", 0);
    let (mut cx, window, _explorer) = explorer_on(here.clone());
    let (page, dialog) = window
        .update(&mut cx, |ws, window, cx| {
            let page = ws.storages_page();
            let last = page.read(cx).count() - 1;
            let dialog = cx.new(|cx| {
                crate::dialogs::AddStorageDialog::editing(page.clone(), last, window, cx)
            });
            (page, dialog)
        })
        .expect("the window");
    let place = |cx: &mut HeadlessAppContext| {
        window
            .update(cx, |_, _, cx| {
                let p = page.read(cx);
                p.storage(p.count() - 1).expect("there").place.clone()
            })
            .expect("the window")
    };
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| {
                d.set_folder("/not/here/at/all", cx);
                d.press_save(cx);
            })
        })
        .expect("the window");
    assert_eq!(place(&mut cx), crate::prefs::KeptPlace::Local(here));
    let moved_text = moved.display().to_string();
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| {
                d.set_folder(&moved_text, cx);
                d.press_save(cx);
            })
        })
        .expect("the window");
    assert_eq!(place(&mut cx), crate::prefs::KeptPlace::Local(moved));
}

fn fake_extensions() -> crate::Extensions {
    crate::Extensions {
        storage_kinds: vec![Arc::new(FarKind(Default::default()))],
    }
}

#[test]
fn a_new_archive_goes_to_a_storage_chosen_and_is_shown_there() {
    let what = folder_of("far-what", 2);
    let far = folder_of("far-to", 0);
    let (mut cx, window, explorer) = explorer_on(out_dir());
    let values: packwing_core::Values = [("folder".to_owned(), far.display().to_string())].into();
    window
        .update(&mut cx, |ws, _, cx| {
            cx.set_global(fake_extensions());
            ws.storages_page().update(cx, |p, cx| {
                p.add_other("far", "Far one".into(), values, &Default::default(), cx)
                    .expect("added");
            });
        })
        .expect("the window");
    cx.run_until_parked();
    let dialog = window
        .update(&mut cx, |ws, window, cx| {
            let (places, _) = ws.storages_page().read(cx).places(cx);
            let far_one = places
                .iter()
                .position(|p| p.name == "Far one")
                .expect("on the list");
            let dialog = ws.open_create_of(vec![what.clone()], window, cx);
            dialog.update(cx, |d, cx| d.press_storage(far_one, cx));
            dialog
        })
        .expect("the window");
    cx.run_until_parked();
    cx.capture_screenshot(window.into())
        .expect("cannot draw the window")
        .save(out_dir().join("create-save-in.png"))
        .expect("cannot save the picture");
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| {
                assert!(d.is_picking());
                d.press_save_here(cx);
                assert!(!d.is_picking());
            })
        })
        .expect("the window");
    cx.run_until_parked();
    cx.capture_screenshot(window.into())
        .expect("cannot draw the window")
        .save(out_dir().join("create-on-storage.png"))
        .expect("cannot save the picture");
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| d.set_counted(2_000_000_000, cx))
        })
        .expect("the window");
    cx.run_until_parked();
    cx.capture_screenshot(window.into())
        .expect("cannot draw the window")
        .save(out_dir().join("create-on-storage-large.png"))
        .expect("cannot save the picture");
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| d.set_counted(2, cx))
        })
        .expect("the window");
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| d.press_create(cx))
        })
        .expect("the window");
    for _ in 0..200 {
        cx.run_until_parked();
        if far.read_dir().expect("the folder").count() > 0 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    cx.run_until_parked();
    let name = format!(
        "{}.zip",
        what.file_name().expect("a name").to_string_lossy()
    );
    assert!(far.join(&name).is_file(), "{name} is not there");
    let (names, page) = (
        explorer.read_with(&cx, |e, _| e.shown_names()),
        window
            .update(&mut cx, |ws, _, _| ws.page_shown())
            .expect("the window"),
    );
    assert_eq!(page, Page::Storages);
    assert!(names.contains(&name), "{names:?}");
}

/// Runs a test of `archive` through to its end, in a window.
fn tested(archive: PathBuf) -> (Option<(u64, u64)>, Option<String>) {
    let (mut cx, window, _) = explorer_on(out_dir());
    let dialog = window
        .update(&mut cx, |ws, window, cx| {
            let mut job = crate::dialogs::Job::shown(
                crate::dialogs::Verb::Test,
                "the archive".into(),
                None,
                0,
                0,
            );
            job.work = vec![(archive.clone().into(), None)];
            ws.open_progress(job, window, cx)
        })
        .expect("the window");
    for _ in 0..200 {
        cx.run_until_parked();
        let ended = dialog.read_with(&cx, |d, _| d.passed().is_some() || d.failed().is_some());
        if ended {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    dialog.read_with(&cx, |d, _| (d.passed(), d.failed().map(str::to_owned)))
}

#[test]
fn a_test_says_how_it_went_whichever_way() {
    let dir = folder_of("tested", 0);
    let sound = dir.join("sound.zip");
    let bytes = zip_of(&[("a.txt", b"hello"), ("b.txt", b"world!")]);
    std::fs::write(&sound, &bytes).expect("cannot write");
    let (passed, failed) = tested(sound);
    assert_eq!(passed, Some((2, 11)), "{failed:?}");
    // What the first file holds, changed: its checksum no longer agrees.
    let mut broken_bytes = bytes;
    let at = broken_bytes
        .windows(5)
        .position(|w| w == b"hello")
        .expect("stored as it is");
    broken_bytes[at] = b'j';
    let broken = dir.join("broken.zip");
    std::fs::write(&broken, broken_bytes).expect("cannot write");
    let (passed, failed) = tested(broken);
    assert_eq!(passed, None);
    assert!(failed.is_some());
}

#[test]
fn browse_goes_where_it_is_told_and_remembers_it() {
    let (mut cx, window, _) = explorer_on(out_dir());
    let dir = folder_of("browse", 3);
    window
        .update(&mut cx, |ws, _, cx| {
            ws.show_page(Page::Browse, cx);
            ws.browse_page().update(cx, |b, cx| b.go_to(&dir, cx));
        })
        .expect("the window");
    for _ in 0..40 {
        cx.run_until_parked();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let (names, kept) = window
        .update(&mut cx, |ws, _, cx| {
            let explorer = ws.browse_page().read(cx).explorer().clone();
            (
                explorer.read(cx).shown_names(),
                crate::prefs::prefs(cx).browse_folder,
            )
        })
        .expect("the window");
    assert_eq!(names.len(), 3, "{names:?}");
    assert_eq!(kept, Some(dir.clone()));
    // It can go up from there, past where it started.
    window
        .update(&mut cx, |ws, _, cx| {
            let explorer = ws.browse_page().read(cx).explorer().clone();
            explorer.update(cx, |e, cx| e.go_up(cx));
        })
        .expect("the window");
    cx.run_until_parked();
    let kept = window
        .update(&mut cx, |_, _, cx| crate::prefs::prefs(cx).browse_folder)
        .expect("the window");
    assert_eq!(kept.as_deref(), dir.parent());
}

#[test]
fn one_file_offers_more_formats_and_keeps_all_its_name() {
    use crate::dialogs::Format;
    let dir = folder_of("one-format", 2);
    let file = dir.join("dump.sql");
    std::fs::write(&file, "insert").expect("cannot write");
    let (mut cx, window, _) = explorer_on(out_dir());
    let dialog = window
        .update(&mut cx, |ws, window, cx| {
            ws.open_create_of(vec![file.clone()], window, cx)
        })
        .expect("the window");
    cx.run_until_parked();
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| {
                assert_eq!(d.name_typed(cx), "dump");
                d.choose_format(Format::Zst, cx);
            })
        })
        .expect("the window");
    cx.run_until_parked();
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| {
                assert_eq!(d.format_chosen(cx), Format::Zst);
                assert_eq!(d.name_typed(cx), "dump.sql");
            })
        })
        .expect("the window");
    cx.capture_screenshot(window.into())
        .expect("cannot draw the window")
        .save(out_dir().join("create-one-file.png"))
        .expect("cannot save the picture");
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| d.open_formats(cx))
        })
        .expect("the window");
    cx.run_until_parked();
    cx.capture_screenshot(window.into())
        .expect("cannot draw the window")
        .save(out_dir().join("create-formats.png"))
        .expect("cannot save the picture");
    // Another file: it is no longer one, and the format gives way.
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| d.add(vec![dir.clone()], cx))
        })
        .expect("the window");
    cx.run_until_parked();
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| assert!(!d.format_chosen(cx).one_file()))
        })
        .expect("the window");
}

#[test]
fn a_file_encrypted_to_a_key_is_saved_with_age_at_the_end_of_its_name() {
    use crate::dialogs::Format;
    let dir = folder_of("age-key", 0);
    let file = dir.join("dump.sql");
    std::fs::write(&file, "insert").expect("cannot write");
    let far = folder_of("age-far", 0);
    let (mut cx, window, _) = explorer_on(out_dir());
    let values: packwing_core::Values = [("folder".to_owned(), far.display().to_string())].into();
    window
        .update(&mut cx, |ws, _, cx| {
            cx.set_global(fake_extensions());
            ws.storages_page().update(cx, |p, cx| {
                p.add_other("far", "Far one".into(), values, &Default::default(), cx)
                    .expect("added");
            });
        })
        .expect("the window");
    cx.run_until_parked();
    let dialog = window
        .update(&mut cx, |ws, window, cx| {
            let (places, _) = ws.storages_page().read(cx).places(cx);
            let far_one = places
                .iter()
                .position(|p| p.name == "Far one")
                .expect("on the list");
            let dialog = ws.open_create_of(vec![file.clone()], window, cx);
            dialog.update(cx, |d, cx| {
                d.press_storage(far_one, cx);
                d.press_save_here(cx);
            });
            dialog
        })
        .expect("the window");
    cx.run_until_parked();
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| {
                d.choose_format(Format::Zst, cx);
                d.encrypt_to_key(
                    "age1ml7zyg7w7uqg5qrlunupaygdcckn0vun4xwvlpx2cdmwn7vqdugqhzawyg",
                    cx,
                );
            })
        })
        .expect("the window");
    cx.run_until_parked();
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| d.scroll_to_end(cx))
        })
        .expect("the window");
    cx.run_until_parked();
    cx.run_until_parked();
    window
        .update(&mut cx, |_, _, cx| cx.refresh_windows())
        .expect("the window");
    cx.run_until_parked();
    cx.capture_screenshot(window.into())
        .expect("cannot draw the window")
        .save(out_dir().join("create-age-key.png"))
        .expect("cannot save the picture");
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| d.press_create(cx))
        })
        .expect("the window");
    let made = far.join("dump.sql.zst.age");
    for _ in 0..200 {
        cx.run_until_parked();
        if made.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    cx.run_until_parked();
    assert!(made.is_file(), "not there");
    assert!(
        std::fs::read(&made)
            .expect("read")
            .starts_with(b"age-encryption.org/v1")
    );
}

#[test]
fn a_post_quantum_key_of_2000_characters_says_what_opens_it() {
    use crate::dialogs::Format;
    let key = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../core/tests/data/test-age-pq.key"),
    )
    .expect("read");
    let key = key
        .lines()
        .find_map(|line| line.strip_prefix("# public key: "))
        .expect("a public key")
        .to_owned();
    let file = folder_of("age-pq", 0).join("dump.sql");
    std::fs::write(&file, "insert").expect("cannot write");
    let (mut cx, window, _) = explorer_on(out_dir());
    let dialog = window
        .update(&mut cx, |ws, window, cx| {
            ws.open_create_of(vec![file.clone()], window, cx)
        })
        .expect("the window");
    cx.run_until_parked();
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| {
                d.choose_format(Format::Zst, cx);
                d.encrypt_to_key(&key, cx);
            })
        })
        .expect("the window");
    cx.run_until_parked();
    window
        .update(&mut cx, |_, _, cx| {
            dialog.update(cx, |d, cx| d.scroll_to_end(cx))
        })
        .expect("the window");
    cx.run_until_parked();
    cx.run_until_parked();
    window
        .update(&mut cx, |_, _, cx| cx.refresh_windows())
        .expect("the window");
    cx.run_until_parked();
    cx.capture_screenshot(window.into())
        .expect("cannot draw the window")
        .save(out_dir().join("create-age-pq.png"))
        .expect("cannot save the picture");
}

#[test]
fn a_kind_a_build_adds_is_browsed_without_a_local_path() {
    let dir = folder_of("far", 2);
    std::fs::write(dir.join("backup.zip"), "not one").expect("cannot write");
    let (mut cx, window, explorer) = explorer_on(out_dir());
    let kept = Arc::new(std::sync::Mutex::new(Vec::new()));
    let kind = FarKind(kept.clone());
    let values: packwing_core::Values = [("folder".to_owned(), dir.display().to_string())].into();
    let secrets: packwing_core::Values = [("token".to_owned(), "hush".to_owned())].into();
    window
        .update(&mut cx, |ws, _, cx| {
            cx.set_global(crate::Extensions {
                storage_kinds: vec![Arc::new(kind)],
            });
            ws.storages_page().update(cx, |p, cx| {
                p.add_other("far", "Far one".into(), values, &secrets, cx)
                    .expect("added");
            });
        })
        .expect("the window");
    cx.run_until_parked();
    assert_eq!(kept.lock().expect("the list")[0].1["token"], "hush");
    // Listed like any other, and an archive there is read in place.
    let names = explorer.read_with(&cx, |e, _| e.shown_names());
    assert!(names.contains(&"backup.zip".to_owned()), "{names:?}");
    window
        .update(&mut cx, |_, _, cx| {
            explorer.update(cx, |e, cx| {
                assert!(!e.is_local());
                e.choose("backup.zip", cx);
                let item = e.selection().remove(0);
                e.open(&item, cx);
            })
        })
        .expect("the window");
    cx.run_until_parked();
    let page = window
        .update(&mut cx, |ws, _, cx| {
            ws.archive_page().read(cx).shown_failed()
        })
        .expect("the window");
    // It is not an archive, and says so as one on this computer would.
    assert!(page.is_some(), "{page:?}");
    let zip = dir.join("real.zip");
    std::fs::write(&zip, zip_of(&[("inner.txt", b"hello")])).expect("cannot write");
    window
        .update(&mut cx, |_, _, cx| {
            explorer.update(cx, |e, cx| e.refresh(cx));
        })
        .expect("the window");
    cx.run_until_parked();
    window
        .update(&mut cx, |_, _, cx| {
            explorer.update(cx, |e, cx| {
                e.choose("real.zip", cx);
                let item = e.selection().remove(0);
                e.open(&item, cx);
            })
        })
        .expect("the window");
    cx.run_until_parked();
    window
        .update(&mut cx, |ws, _, cx| {
            let page = ws.archive_page();
            let archive = page.read(cx).archive().expect("it opens");
            assert_eq!(archive.files, 1);
            // Nothing on it to add to or change: it is only read.
            assert!(archive.path.is_none());
            assert!(archive.source.as_ref().and_then(|s| s.fetched()).is_some());
        })
        .expect("the window");
    let image = cx
        .capture_screenshot(window.into())
        .expect("cannot draw the window");
    image
        .save(out_dir().join("archive-on-storage-light.png"))
        .expect("cannot save the picture");
    // Secrets do not go with the settings.
    let text = window
        .update(&mut cx, |_, _, cx| crate::prefs::prefs(cx).text_for_test())
        .expect("the window");
    assert!(text.contains("storage=other far\tFar one"), "{text}");
    assert!(!text.contains("hush"), "{text}");
    // Taken off the list, its secrets go.
    window
        .update(&mut cx, |ws, _, cx| {
            ws.storages_page().update(cx, |p, cx| p.remove_last(cx));
        })
        .expect("the window");
    assert!(kept.lock().expect("the list").is_empty());
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
            cx.new(|cx| Workspace::new(false, menu, window, cx))
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

/// The wheel over a list that has more to show moves that list alone; anywhere
/// else in the dialog, it moves the dialog.
#[test]
fn the_wheel_moves_only_what_is_under_the_pointer() {
    use gpui::{
        Modifiers, MouseMoveEvent, PlatformInput, ScrollDelta, ScrollWheelEvent, TouchPhase, point,
    };
    let text = Arc::new(gpui_wgpu::CosmicTextSystem::new(crate::FONT));
    let mut cx = HeadlessAppContext::with_platform(text, Arc::new(crate::Assets), || {
        gpui_platform::current_headless_renderer()
    });
    cx.update(crate::load_fonts);
    let window = cx
        .open_window(size(px(1280.), px(800.)), |window, cx| {
            let menu = cx.new(|_| RightClickMenu::offline(true, Choice::default(), Status::Idle));
            cx.new(|cx| Workspace::new(false, menu, window, cx))
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
    let wheel = |cx: &mut HeadlessAppContext, at: gpui::Point<gpui::Pixels>| {
        let move_there = PlatformInput::MouseMove(MouseMoveEvent {
            position: at,
            pressed_button: None,
            modifiers: Modifiers::default(),
        });
        let turn = PlatformInput::ScrollWheel(ScrollWheelEvent {
            position: at,
            delta: ScrollDelta::Pixels(point(px(0.), px(-60.))),
            modifiers: Modifiers::default(),
            touch_phase: TouchPhase::Moved,
        });
        for event in [move_there, turn] {
            cx.update_window(window.into(), |_, window, cx| {
                window.dispatch_event(event, cx);
            })
            .expect("cannot send the event");
            cx.run_until_parked();
            cx.capture_screenshot(window.into())
                .expect("cannot draw the window");
        }
    };
    assert_eq!(
        dialog.read_with(&cx, |d, _| d.scrolled()),
        (px(0.), px(0.)),
        "where it starts"
    );
    // Over the list, as `create-chosen-light.png` draws it.
    wheel(&mut cx, point(px(640.), px(300.)));
    let (list, body) = dialog.read_with(&cx, |d, _| d.scrolled());
    assert!(list < px(0.), "the list moved");
    assert_eq!(body, px(0.), "the dialog held still");
    // Over "Save in", below it.
    wheel(&mut cx, point(px(640.), px(540.)));
    let (after, body) = dialog.read_with(&cx, |d, _| d.scrolled());
    assert_eq!(after, list, "the list held still");
    assert!(body < px(0.), "the dialog moved");
}

/// A ZIP archive of `files`, stored, written as the format has it.
fn zip_of(files: &[(&str, &[u8])]) -> Vec<u8> {
    let dir = out_dir().join(format!("zip-of-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("cannot make a folder");
    let mut paths = Vec::new();
    for (name, body) in files {
        std::fs::write(dir.join(name), body).expect("cannot write");
        paths.push(dir.join(name));
    }
    let made = packwing_core::create(
        &paths,
        &dir.join("made.zip"),
        &packwing_core::NewArchive::default(),
        &packwing_core::Control {
            progress: None,
            cancel: None,
        },
    )
    .expect("cannot make an archive");
    let bytes = std::fs::read(made).expect("cannot read");
    let _ = std::fs::remove_dir_all(&dir);
    bytes
}
