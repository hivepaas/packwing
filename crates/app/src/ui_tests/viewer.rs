//! The viewer: a file of an archive or of Browse over the whole window,
//! stepping through the folder's pictures and texts, and back to where it was
//! opened, the file shown last chosen; a comic book read as a book.

use std::path::PathBuf;

use packwing_core::{Control, NewArchive};

use super::{Ui, scenario};

/// A picture GPUI draws: packwing's own icon.
const PICTURE: &[u8] = include_bytes!("../../../../packaging/icons/png/packwing-64.png");

/// `name` in the work folder, holding `bytes`.
fn file(ui: &Ui, name: &str, bytes: &[u8]) -> PathBuf {
    let path = ui.work.join(name);
    std::fs::create_dir_all(path.parent().expect("a folder")).expect("its folder");
    std::fs::write(&path, bytes).expect("the file");
    path
}

/// A ZIP archive of `paths` at `name` in the work folder.
fn zip(ui: &Ui, paths: &[PathBuf], name: &str) -> PathBuf {
    let path = ui.work.join(name);
    packwing_core::create(paths, &path, &NewArchive::default(), &Control::default())
        .expect("the archive");
    path
}

/// What the viewer steps through, the file it shows, and what it shows of it.
fn viewing(ui: &mut Ui) -> Option<(Vec<String>, String, &'static str)> {
    ui.ws(|ws, _, cx| ws.viewing(cx))
}

/// Waits until the viewer shows `name`, read, as `what`.
fn wait_for_shown(ui: &mut Ui, name: &str, what: &'static str) {
    ui.wait_until(&format!("{name} shown"), |ui| {
        viewing(ui).is_some_and(|(_, shown, as_what)| shown == name && as_what == what)
    });
}

scenario!(
    /// Open on a picture of an archive shows the viewer over the window; Next
    /// and Previous step through the folder's pictures and texts, not what it
    /// cannot show; a text is copied from its right-click menu; Escape goes
    /// back to the archive, the last shown chosen.
    the_viewer_steps_through_an_archive_and_goes_back,
    |ui| {
        let paths = vec![
            file(ui, "src/a.png", PICTURE),
            file(ui, "src/b.txt", b"a note"),
            file(ui, "src/c.pdf", b"%PDF-1.7"),
            file(ui, "src/d.png", PICTURE),
        ];
        let made = zip(ui, &paths, "pictures.zip");
        ui.open(&made);
        assert_eq!(ui.archive_failed(), None);
        let at = ui
            .shown_in_archive()
            .iter()
            .position(|n| n == "a.png")
            .expect("a.png listed");
        ui.right_click(&format!("entry-{at}"));
        ui.wait_for("entry-menu");
        ui.click("menu-open");
        wait_for_shown(ui, "a.png", "picture");
        let (names, _, _) = viewing(ui).expect("the viewer");
        assert_eq!(names, ["a.png", "b.txt", "d.png"]);
        assert!(!ui.shows("archive-more"), "the page is hidden");
        ui.click("view-next");
        wait_for_shown(ui, "b.txt", "text");
        ui.keys("right");
        wait_for_shown(ui, "d.png", "picture");
        ui.keys("left");
        wait_for_shown(ui, "b.txt", "text");
        // A text has its right-click menu, as the preview's had.
        ui.right_click("field: ");
        ui.wait_for("text-copy-all");
        ui.click("text-copy-all");
        let copied = ui.ws(|_, _, cx| cx.read_from_clipboard().and_then(|c| c.text()));
        assert_eq!(copied.as_deref(), Some("a note"));
        ui.keys("escape");
        ui.wait_until("the viewer closed", |ui| viewing(ui).is_none());
        let chosen = ui.ws(|ws, _, cx| ws.archive_page().read(cx).selected_names());
        assert_eq!(chosen, ["b.txt"]);
    }
);

scenario!(
    /// A comic book opens as a book: its one folder gone into, its pages in
    /// the order people number them, from the first.
    a_comic_book_opens_at_its_first_page,
    |ui| {
        for page in ["page10.png", "page2.png", "page1.png"] {
            file(ui, &format!("book/story/{page}"), PICTURE);
        }
        let book = zip(ui, &[ui.work.join("book/story")], "story.cbz");
        ui.open(&book);
        ui.wait_until("the book open", |ui| viewing(ui).is_some());
        let (names, shown, _) = viewing(ui).expect("the viewer");
        assert_eq!(names, ["page1.png", "page2.png", "page10.png"]);
        assert_eq!(shown, "page1.png");
        assert_eq!(ui.archive_folder(), ["story"]);
    }
);

scenario!(
    /// Browse opens its files in the viewer too, and Back goes back to the
    /// folder, the last shown chosen.
    browse_opens_the_viewer,
    |ui| {
        file(ui, "shown/x.png", PICTURE);
        file(ui, "shown/y.txt", b"why");
        let at = ui.work.join("shown");
        ui.click("nav-browse");
        ui.ws(|ws, _, cx| ws.browse_page().update(cx, |b, cx| b.go_to(&at, cx)));
        ui.ws(|ws, _, cx| {
            let explorer = ws.browse_page().read(cx).explorer().clone();
            explorer.update(cx, |e, cx| {
                e.list_now(cx);
                if let Some(item) = e.item_named("x.png") {
                    e.open(&item, cx);
                }
            });
        });
        wait_for_shown(ui, "x.png", "picture");
        ui.keys("right");
        wait_for_shown(ui, "y.txt", "text");
        ui.click("view-back");
        ui.wait_until("the viewer closed", |ui| viewing(ui).is_none());
        let chosen = ui.ws(|ws, _, cx| {
            let explorer = ws.browse_page().read(cx).explorer().clone();
            explorer
                .read(cx)
                .selection()
                .into_iter()
                .map(|i| i.name)
                .collect::<Vec<_>>()
        });
        assert_eq!(chosen, ["y.txt"]);
    }
);

scenario!(
    /// The viewer shows what the archive's preview shows: a text longer than
    /// it showed before (its first MB), and a script, as texts.
    the_viewer_shows_long_texts_and_scripts,
    |ui| {
        let mut long = String::new();
        let mut n = 0;
        while long.len() < 1_200_000 {
            long.push_str(&format!("{n:07} a line of a long log\n"));
            n += 1;
        }
        let paths = vec![
            file(ui, "logs/long.log", long.as_bytes()),
            file(ui, "logs/run.sh", b"#!/bin/sh\necho hello\n"),
        ];
        let made = zip(ui, &paths, "logs.zip");
        ui.open(&made);
        assert_eq!(ui.archive_failed(), None);
        let at = ui
            .shown_in_archive()
            .iter()
            .position(|n| n == "long.log")
            .expect("long.log listed");
        ui.click(&format!("entry-{at}"));
        ui.wait_for("preview-text");
        ui.keys("enter");
        wait_for_shown(ui, "long.log", "text");
        ui.keys("right");
        wait_for_shown(ui, "run.sh", "text");
    }
);

scenario!(
    /// A text is searched: Ctrl+F (Cmd+F) opens the search's bar, what is
    /// typed is found whatever its case, Enter goes to the next and round,
    /// Shift+Enter back; by a pattern, and with its case as typed; Escape
    /// closes the bar, then the viewer.
    a_text_is_searched,
    |ui| {
        let paths = vec![file(
            ui,
            "notes/todo.txt",
            "Back up the site\nback up the logs\nCheck the BACKUP\n".as_bytes(),
        )];
        let made = zip(ui, &paths, "notes.zip");
        ui.open(&made);
        assert_eq!(ui.archive_failed(), None);
        ui.click("entry-0");
        ui.keys("enter");
        wait_for_shown(ui, "todo.txt", "text");
        let finding = |ui: &mut Ui| ui.ws(|ws, _, cx| ws.viewer_finding(cx));
        assert_eq!(finding(ui), None);
        ui.keys("secondary-f");
        ui.wait_for("find-count");
        ui.type_in("Find in text", "back");
        assert_eq!(finding(ui), Some((3, 0, false)));
        ui.keys("enter");
        assert_eq!(finding(ui), Some((3, 1, false)));
        ui.keys("enter enter");
        assert_eq!(finding(ui), Some((3, 0, false)), "round to the first");
        ui.keys("shift-enter");
        assert_eq!(finding(ui), Some((3, 2, false)));
        ui.replace_in("Find in text", "nowhere");
        assert_eq!(finding(ui), Some((0, 0, false)));
        // By pattern: ^ at each line's start, any case, then the case as typed.
        ui.click("find-regex");
        ui.replace_in("Find in text", "^back");
        assert_eq!(finding(ui), Some((2, 0, false)));
        ui.click("find-case");
        assert_eq!(finding(ui), Some((1, 0, false)));
        ui.replace_in("Find in text", "back(up");
        assert_eq!(finding(ui), Some((0, 0, true)), "no pattern");
        ui.click("find-regex");
        assert_eq!(finding(ui), Some((0, 0, false)), "the brackets are letters");
        ui.keys("escape");
        assert_eq!(finding(ui), None);
        assert!(ui.shows("viewer"), "the viewer stays");
        ui.keys("escape");
        ui.wait_until("the viewer closed", |ui| !ui.shows("viewer"));
    }
);

scenario!(
    /// A picture fits the window when the viewer opens; once the user shows
    /// it at its own size (1), Next and Previous keep that, past a text, until
    /// Fit is chosen again; the size's one button goes round the two.
    the_size_chosen_stays_from_picture_to_picture,
    |ui| {
        let paths = vec![
            file(ui, "sizes/a.png", PICTURE),
            file(ui, "sizes/b.txt", b"between"),
            file(ui, "sizes/c.png", PICTURE),
        ];
        let made = zip(ui, &paths, "sizes.zip");
        ui.open(&made);
        assert_eq!(ui.archive_failed(), None);
        let at = ui
            .shown_in_archive()
            .iter()
            .position(|n| n == "a.png")
            .expect("a.png listed");
        ui.click(&format!("entry-{at}"));
        ui.keys("enter");
        wait_for_shown(ui, "a.png", "picture");
        assert!(!ui.shows("view-at-size"), "it fits at first");
        ui.keys("1");
        ui.wait_for("view-at-size");
        ui.keys("right");
        wait_for_shown(ui, "b.txt", "text");
        ui.keys("right");
        wait_for_shown(ui, "c.png", "picture");
        assert!(ui.shows("view-at-size"), "still at its own size");
        ui.keys("left left");
        wait_for_shown(ui, "a.png", "picture");
        assert!(ui.shows("view-at-size"), "back, at its own size");
        // The one button goes round: back to Fit, then to 1:1 again.
        ui.click("view-size");
        assert!(!ui.shows("view-at-size"));
        ui.keys("right right");
        wait_for_shown(ui, "c.png", "picture");
        assert!(!ui.shows("view-at-size"), "it fits, as chosen");
        ui.click("view-size");
        assert!(ui.shows("view-at-size"), "round to its own size");
    }
);

scenario!(
    /// Up and Down step through the files as Left and Right do, but for what
    /// does not fit: a long text, a picture at its own size larger than the
    /// window. Those they scroll, and step no further at its top or bottom;
    /// each picture shows from its top.
    up_and_down_scroll_what_does_not_fit_and_step_otherwise,
    |ui| {
        const LARGE: &[u8] = include_bytes!("../../../../packaging/icons/png/packwing-1024.png");
        let long: String = (0..400).map(|n| format!("line {n}\n")).collect();
        let paths = vec![
            file(ui, "updown/a.png", PICTURE),
            file(ui, "updown/b.txt", b"short"),
            file(ui, "updown/c.log", long.as_bytes()),
            file(ui, "updown/cc.log", long.as_bytes()),
            file(ui, "updown/d.png", LARGE),
        ];
        let made = zip(ui, &paths, "updown.zip");
        ui.open(&made);
        assert_eq!(ui.archive_failed(), None);
        let at = ui
            .shown_in_archive()
            .iter()
            .position(|n| n == "a.png")
            .expect("a.png listed");
        ui.click(&format!("entry-{at}"));
        ui.keys("enter");
        wait_for_shown(ui, "a.png", "picture");
        let scrolled = |ui: &mut Ui| ui.ws(|ws, _, cx| ws.viewer_scrolled(cx)).expect("viewing");
        // A picture that fits, a short text: Down steps.
        ui.keys("down");
        wait_for_shown(ui, "b.txt", "text");
        ui.keys("down");
        wait_for_shown(ui, "c.log", "text");
        // A long text: Down and Up scroll it, and at its top Up stays.
        assert_eq!(scrolled(ui), 0.);
        ui.keys("down down");
        let down = scrolled(ui);
        assert!(down > 0., "scrolled down");
        assert_eq!(viewing(ui).expect("the viewer").1, "c.log");
        ui.keys("up up up");
        assert_eq!(scrolled(ui), 0.);
        assert_eq!(viewing(ui).expect("the viewer").1, "c.log", "Up stays at its top");
        // The next text shows from its top, the one before scrolled or not.
        ui.keys("down down");
        ui.keys("right");
        wait_for_shown(ui, "cc.log", "text");
        assert_eq!(scrolled(ui), 0., "the next text from its top");
        // A large picture, higher than the window (its 1024 pixels are 512
        // on this screen of 2): at its own size, Down scrolls it.
        ui.resize(900., 400.);
        ui.keys("right");
        wait_for_shown(ui, "d.png", "picture");
        ui.keys("1");
        ui.wait_for("view-at-size");
        ui.keys("down down");
        assert!(scrolled(ui) > 0., "the picture scrolled");
        assert_eq!(viewing(ui).expect("the viewer").1, "d.png");
        ui.keys("left");
        wait_for_shown(ui, "cc.log", "text");
        ui.keys("right");
        wait_for_shown(ui, "d.png", "picture");
        assert_eq!(scrolled(ui), 0., "from its top again");
        // A small picture at its own size has nothing to scroll: Down steps.
        ui.keys("home");
        wait_for_shown(ui, "a.png", "picture");
        assert!(ui.shows("view-at-size"));
        ui.keys("down");
        wait_for_shown(ui, "b.txt", "text");
    }
);

scenario!(
    /// The list of files beside the viewer: hidden at first, shown by its
    /// button left of Back, the files Previous and Next step through in their
    /// order, the one shown chosen; a click shows another, and the keys stay
    /// the viewer's. Kept as left when the viewer opens again; Ctrl+B (Cmd+B)
    /// hides it; over the file's left in a narrow window.
    the_list_of_files_beside_the_viewer,
    |ui| {
        let paths = vec![
            file(ui, "list/p10.png", PICTURE),
            file(ui, "list/p2.png", PICTURE),
            file(ui, "list/p1.png", PICTURE),
            file(ui, "list/notes.pdf", b"%PDF-1.7"),
            file(ui, "list/p3.png", PICTURE),
        ];
        let made = zip(ui, &paths, "list.zip");
        ui.open(&made);
        assert_eq!(ui.archive_failed(), None);
        let at = ui
            .shown_in_archive()
            .iter()
            .position(|n| n == "p1.png")
            .expect("p1.png listed");
        ui.click(&format!("entry-{at}"));
        ui.keys("enter");
        wait_for_shown(ui, "p1.png", "picture");
        assert!(!ui.shows("view-files"), "hidden at first");
        ui.click("view-sidebar");
        ui.wait_for("view-files");
        // As Previous and Next go, the PDF they pass over left out.
        let (names, _, _) = viewing(ui).expect("the viewer");
        assert_eq!(names.len(), 4);
        assert!(!names.iter().any(|n| n == "notes.pdf"));
        for i in 0..names.len() {
            assert!(ui.shows(&format!("view-file-{i}")), "row {i}");
        }
        assert!(!ui.shows(&format!("view-file-{}", names.len())));
        // Beside the file, in a wide window.
        let list = ui.find("view-files").expect("the list");
        let stage = ui.find("view-stage").expect("the stage");
        assert!(stage.left() >= list.right(), "beside");
        // A row shows its file; Left goes on from it, as Previous does.
        ui.click("view-file-2");
        wait_for_shown(ui, &names[2], "picture");
        ui.keys("left");
        wait_for_shown(ui, &names[1], "picture");
        // Opened again, it is as it was left.
        ui.keys("escape");
        ui.wait_until("the viewer closed", |ui| viewing(ui).is_none());
        ui.keys("enter");
        wait_for_shown(ui, &names[1], "picture");
        assert!(ui.shows("view-files"), "kept shown");
        // Over the file in a narrow window.
        ui.resize(600., 500.);
        let list = ui.find("view-files").expect("the list");
        let stage = ui.find("view-stage").expect("the stage");
        assert!(stage.left() < list.right(), "over the file");
        ui.keys("secondary-b");
        assert!(!ui.shows("view-files"), "hidden by its keys");
    }
);
