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
    /// cannot show; Escape goes back to the archive, the last shown chosen.
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
