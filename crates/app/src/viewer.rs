//! The viewer: a file of a folder, or of an archive, over the whole window, the
//! rail and the page hidden. A picture fits the window, or shows at its own size;
//! a text shows with its lines' numbers, to choose and copy from; anything else says why it cannot be shown
//! and offers the system's app, and packwing as an archive. Previous and Next step
//! through the folder's other pictures and texts, the next read ahead.
//!
//! It is a view of its own, told what to show and saying what was asked of it,
//! so that what holds it (the workspace) may put it anywhere.

use std::cmp::Ordering;
use std::sync::Arc;

use gpui::{
    Context, Entity, EventEmitter, FocusHandle, Focusable, FontWeight, Image, KeyDownEvent,
    ObjectFit, Render, StyledImage, Task, Window, div, img, prelude::*, px, rgb,
};
use packwing_core::{Control, Error, Keys, Source, Storage, image_fits, image_size, preview_entry};
use packwing_i18n::{tr, tr_n};

use crate::colors::colors;
use crate::input::{Paint, TextInput};
use crate::model::{Kind, size, tag_of, text_preview};
use crate::pages::archive::{MOST_PREVIEWED, picture_format};
use crate::text_size::ui;
use crate::widgets::{Fill, Size, button, icon, icon_button, kind_tile, mono_font};

/// A text's paint: none, all in the text's color.
fn unpainted(_: &str) -> Vec<(std::ops::Range<usize>, Paint)> {
    Vec::new()
}

/// How much of a text the viewer shows.
const MOST_TEXT: usize = 256 * 1024;

/// Where the files shown are read from.
#[derive(Clone)]
pub enum Reading {
    /// A storage, by the files' paths in it.
    Stored(Arc<dyn Storage>),
    /// The archive open, by the files' paths in it, with its keys.
    Archive { source: Source, keys: Keys },
}

/// A file the viewer may show.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewFile {
    /// Its path in the storage or the archive.
    pub path: Vec<String>,
    pub name: String,
    pub size: u64,
    /// Whether the system's app may be offered for it: not for what would
    /// run, from a storage.
    pub app: bool,
    /// Whether it may be edited in the text editor: a file on this computer.
    pub edit: bool,
}

/// What the viewer was asked.
pub enum ViewerEvent {
    /// Go back to where it was opened, the file shown last chosen there.
    Close(ViewFile),
    OpenWithApp(ViewFile),
    OpenAsArchive(ViewFile),
    Edit(ViewFile),
}

/// What is shown of a file.
#[derive(Clone)]
enum Shown {
    Reading,
    /// A picture, and its size in pixels.
    Picture(Arc<Image>, Option<(u32, u32)>),
    /// A text, how many lines it has, and whether it was cut.
    Text(Arc<String>, usize, bool),
    /// Why it cannot be shown.
    Not(String),
}

pub struct Viewer {
    reading: Reading,
    /// What Back goes back to, as the user knows it.
    from: String,
    files: Vec<ViewFile>,
    at: usize,
    shown: Shown,
    /// The text shown, when it is one: it can be chosen and copied, with a
    /// right-click menu, its lines numbered.
    text: Entity<TextInput>,
    /// A picture at its own size, where it fits the window otherwise.
    actual: bool,
    /// The file after the one shown, read ahead, and what it showed.
    ahead: Option<(usize, Shown)>,
    _read: Option<Task<()>>,
    _read_ahead: Option<Task<()>>,
    focus: FocusHandle,
}

impl EventEmitter<ViewerEvent> for Viewer {}

impl Focusable for Viewer {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus.clone()
    }
}

/// Whether the viewer shows what a file of this name holds: a picture GPUI
/// draws, or a text.
pub fn viewable(name: &str) -> bool {
    picture_format(name).is_some() || Kind::of(name, false) == Kind::Text
}

/// The files of a folder the viewer steps through, from `chosen`: those it
/// shows, and `chosen` whatever it is, in the folder's order.
pub fn stepping(files: Vec<ViewFile>, chosen: &str) -> (Vec<ViewFile>, usize) {
    let kept: Vec<ViewFile> = files
        .into_iter()
        .filter(|f| f.name == chosen || viewable(&f.name))
        .collect();
    let at = kept.iter().position(|f| f.name == chosen).unwrap_or(0);
    (kept, at)
}

/// A comic book's pages: by their names as people number them, `page2`
/// before `page10`.
pub fn natural_order(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let take = |it: &mut std::iter::Peekable<std::str::Chars<'_>>| {
                    let mut n = String::new();
                    while let Some(c) = it.peek().copied().filter(char::is_ascii_digit) {
                        n.push(c);
                        it.next();
                    }
                    n
                };
                let (m, n) = (take(&mut a), take(&mut b));
                let (mt, nt) = (m.trim_start_matches('0'), n.trim_start_matches('0'));
                let by = mt.len().cmp(&nt.len()).then_with(|| mt.cmp(nt));
                if by != Ordering::Equal {
                    return by;
                }
            }
            (Some(x), Some(y)) => {
                let by = x.to_lowercase().cmp(y.to_lowercase());
                if by != Ordering::Equal {
                    return by;
                }
                a.next();
                b.next();
            }
        }
    }
}

/// Reads `file` from `reading` and makes what shows of it. It blocks: it is
/// run in the background.
fn read(reading: &Reading, file: &ViewFile) -> Shown {
    let format = picture_format(&file.name);
    let kind = Kind::of(&file.name, false);
    // A script is a text; a program that is not says so once read.
    if format.is_none() && !matches!(kind, Kind::Text | Kind::Other | Kind::Program) {
        return Shown::Not(tr!("packwing can’t show this kind of file"));
    }
    if format.is_some() && file.size > MOST_PREVIEWED {
        return Shown::Not(tr!("Too big to show here"));
    }
    let most = if format.is_some() {
        MOST_PREVIEWED
    } else {
        MOST_TEXT as u64
    };
    let bytes = match reading {
        Reading::Stored(storage) => storage.read(&file.path, most).map_err(|e| e.to_string()),
        Reading::Archive { source, keys } => {
            match preview_entry(source, &file.path, keys, most, &Control::default()) {
                Ok(bytes) => Ok(bytes),
                Err(Error::Locked { .. } | Error::WrongPassword) => {
                    return Shown::Not(tr!("It needs the password to show it"));
                }
                Err(e) => Err(e.to_string()),
            }
        }
    };
    let bytes = match bytes {
        Ok(bytes) => bytes,
        Err(why) => return Shown::Not(tr!("It could not be read: {why}", why = why)),
    };
    match format {
        Some(format) if image_fits(&bytes) => {
            let pixels = image_size(&bytes);
            Shown::Picture(Arc::new(Image::from_bytes(format, bytes)), pixels)
        }
        Some(_) => Shown::Not(tr!("Too big to show here")),
        None => {
            let cut = file.size > MOST_TEXT as u64;
            match text_preview(&bytes, MOST_TEXT) {
                Some(text) => {
                    // The last line's end starts no line of its own.
                    let text = text
                        .strip_suffix('\n')
                        .unwrap_or(&text)
                        .replace('\t', "    ");
                    let lines = text.lines().count();
                    Shown::Text(Arc::new(text), lines, cut)
                }
                None => Shown::Not(tr!("packwing can’t show this kind of file")),
            }
        }
    }
}

impl Viewer {
    /// Shows `files[at]`, read from `reading`; Back goes back to `from`.
    pub fn new(
        reading: Reading,
        from: String,
        files: Vec<ViewFile>,
        at: usize,
        cx: &mut Context<Self>,
    ) -> Viewer {
        let mut viewer = Viewer {
            reading,
            from,
            files,
            at,
            shown: Shown::Reading,
            text: cx.new(|cx| TextInput::new("", cx).read_only().code(unpainted)),
            actual: false,
            ahead: None,
            _read: None,
            _read_ahead: None,
            focus: cx.focus_handle(),
        };
        viewer.show(at, cx);
        viewer
    }

    /// The file shown.
    pub fn file(&self) -> Option<&ViewFile> {
        self.files.get(self.at)
    }

    /// The names of the files it steps through, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn names(&self) -> Vec<String> {
        self.files.iter().map(|f| f.name.clone()).collect()
    }

    /// Whether what is shown is read, and what kind it is, for tests.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn showing(&self) -> &'static str {
        match self.shown {
            Shown::Reading => "reading",
            Shown::Picture(..) => "picture",
            Shown::Text(..) => "text",
            Shown::Not(_) => "not",
        }
    }

    /// Shows `shown`, a text in its field.
    fn set_shown(&mut self, shown: Shown, cx: &mut Context<Self>) {
        if let Shown::Text(text, ..) = &shown {
            self.text.update(cx, |t, cx| t.set_text(text, 0..0, cx));
        }
        self.shown = shown;
    }

    /// Shows the file at `at`, read now, or ahead already.
    fn show(&mut self, at: usize, cx: &mut Context<Self>) {
        let Some(file) = self.files.get(at).cloned() else {
            return;
        };
        self.at = at;
        self.actual = false;
        if let Some((ahead, shown)) = self.ahead.take()
            && ahead == at
        {
            self.set_shown(shown, cx);
            self.read_ahead(cx);
            cx.notify();
            return;
        }
        self.shown = Shown::Reading;
        let reading = self.reading.clone();
        let task = cx.background_spawn(async move { read(&reading, &file) });
        self._read = Some(cx.spawn(async move |this, cx| {
            let shown = task.await;
            let _ = this.update(cx, |this, cx| {
                if this.at == at {
                    this.set_shown(shown, cx);
                    this.read_ahead(cx);
                    cx.notify();
                }
            });
        }));
        cx.notify();
    }

    /// Reads the file after the one shown, so that Next shows it at once.
    fn read_ahead(&mut self, cx: &mut Context<Self>) {
        let next = self.at + 1;
        let Some(file) = self.files.get(next).cloned() else {
            return;
        };
        let reading = self.reading.clone();
        let task = cx.background_spawn(async move { read(&reading, &file) });
        self._read_ahead = Some(cx.spawn(async move |this, cx| {
            let shown = task.await;
            let _ = this.update(cx, |this, _| {
                if this.at + 1 == next {
                    this.ahead = Some((next, shown));
                }
            });
        }));
    }

    fn step(&mut self, by: isize, cx: &mut Context<Self>) {
        let last = self.files.len().saturating_sub(1);
        let to = self.at.saturating_add_signed(by).min(last);
        if to != self.at {
            self.show(to, cx);
        }
    }

    fn ask(&self, what: fn(ViewFile) -> ViewerEvent, cx: &mut Context<Self>) {
        if let Some(file) = self.file().cloned() {
            cx.emit(what(file));
        }
    }

    fn on_key(&mut self, e: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match e.keystroke.key.as_str() {
            "left" | "pageup" => self.step(-1, cx),
            "right" | "pagedown" | "space" => self.step(1, cx),
            "home" => self.show(0, cx),
            "end" => self.show(self.files.len().saturating_sub(1), cx),
            "f11" => window.toggle_fullscreen(),
            // Out of the full screen first, then out of the viewer.
            "escape" if window.is_fullscreen() => window.toggle_fullscreen(),
            "escape" | "backspace" => self.ask(ViewerEvent::Close, cx),
            "0" if matches!(self.shown, Shown::Picture(..)) => {
                self.actual = false;
                cx.notify();
            }
            "1" if matches!(self.shown, Shown::Picture(..)) => {
                self.actual = true;
                cx.notify();
            }
            _ => return,
        }
        cx.stop_propagation();
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let file = self.file().cloned();
        let name = file.as_ref().map(|f| f.name.clone()).unwrap_or_default();
        let kind = Kind::of(&name, false);
        let picture = matches!(self.shown, Shown::Picture(..));
        let not = matches!(self.shown, Shown::Not(_));
        let (first, last) = (self.at == 0, self.at + 1 >= self.files.len());
        let step = |id: &'static str, name: &str, by: isize, off: bool| {
            icon_button(id, name, 40.)
                .when(off, |d| d.opacity(0.4).cursor_default())
                .when(!off, |d| {
                    d.on_click(cx.listener(move |this, _, _, cx| this.step(by, cx)))
                })
        };
        let toggle = |id: &'static str, label: String, on: bool, actual: bool| {
            button(
                id,
                label,
                if on { Fill::Tonal } else { Fill::Outline },
                Size::Medium,
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.actual = actual;
                cx.notify();
            }))
        };
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(ui(10.))
            .child(
                button(
                    "view-back",
                    tr!("Back to {name}", name = self.from.clone()),
                    Fill::Tonal,
                    Size::Medium,
                )
                // The arrow before the words, which are the button's first child.
                .flex_row_reverse()
                .child(icon("arrow-left", 16., c.on_tonal))
                .on_click(cx.listener(|this, _, _, cx| this.ask(ViewerEvent::Close, cx))),
            )
            .child(kind_tile(kind, tag_of(&name, kind), 36., 32., 11., 10.))
            .child(
                div()
                    .flex_1()
                    .min_w(ui(120.))
                    .truncate()
                    .text_size(ui(17.))
                    .font_weight(FontWeight::BOLD)
                    .child(name),
            )
            .child(step("view-previous", "chevron-left", -1, first))
            .child(
                div()
                    .debug_selector(|| "view-count".to_owned())
                    .min_w(ui(64.))
                    .text_center()
                    .text_size(ui(14.))
                    .font_weight(FontWeight::BOLD)
                    .child(format!("{} / {}", self.at + 1, self.files.len())),
            )
            .child(step("view-next", "chevron-right", 1, last))
            .when(picture, |d| {
                d.child(toggle("view-fit", tr!("Fit"), !self.actual, false))
                    .child(toggle("view-actual", "1:1".into(), self.actual, true))
            })
            .when(file.as_ref().is_some_and(|f| f.edit) && !picture, |d| {
                d.child(
                    button(
                        "view-edit",
                        crate::explorer::Explorer::edit_label(cx),
                        Fill::Outline,
                        Size::Medium,
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.ask(ViewerEvent::Edit, cx))),
                )
            })
            // What could not be shown may be an archive by another name.
            .when(not && kind != Kind::Archive, |d| {
                d.child(
                    button(
                        "view-as-archive",
                        tr!("Open as archive"),
                        Fill::Text,
                        Size::Medium,
                    )
                    .on_click(
                        cx.listener(|this, _, _, cx| this.ask(ViewerEvent::OpenAsArchive, cx)),
                    ),
                )
            })
            .when(file.as_ref().is_some_and(|f| f.app), |d| {
                d.child(
                    button(
                        "view-app",
                        tr!("Open with default app"),
                        if not { Fill::Accent } else { Fill::Outline },
                        Size::Medium,
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.ask(ViewerEvent::OpenWithApp, cx))),
                )
            })
    }

    fn stage(&self, window: &mut Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let c = colors();
        let stage = div()
            .id("view-stage")
            .flex_1()
            .min_h_0()
            .rounded(ui(20.))
            .bg(rgb(c.field))
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center()
            .text_size(ui(14.))
            .text_color(rgb(c.muted));
        match &self.shown {
            Shown::Reading => stage.child(tr!("Reading…")).into_any_element(),
            Shown::Not(why) => stage.child(why.clone()).into_any_element(),
            Shown::Picture(picture, pixels) => {
                let picture = picture.clone();
                let fallback = move || {
                    div()
                        .text_size(ui(13.))
                        .text_color(rgb(c.muted))
                        .child(tr!("It could not be shown"))
                        .into_any_element()
                };
                let toggle = cx.listener(|this, _, _, cx| {
                    this.actual = !this.actual;
                    cx.notify();
                });
                match (self.actual, pixels) {
                    // At its own size, in the screen's pixels, to scroll over.
                    (true, Some((w, h))) => {
                        let scale = window.scale_factor().max(1.);
                        stage
                            .items_start()
                            .justify_start()
                            .child(
                                div()
                                    .id("view-actual-scroll")
                                    .size_full()
                                    .overflow_scroll()
                                    .cursor_pointer()
                                    .on_click(toggle)
                                    .child(
                                        div()
                                            .min_w_full()
                                            .min_h_full()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                img(picture)
                                                    .w(px(*w as f32 / scale))
                                                    .h(px(*h as f32 / scale))
                                                    .flex_shrink_0()
                                                    .with_fallback(fallback),
                                            ),
                                    ),
                            )
                            .into_any_element()
                    }
                    _ => stage
                        .cursor_pointer()
                        .on_click(toggle)
                        .child(
                            img(picture)
                                .size_full()
                                .object_fit(ObjectFit::Contain)
                                .with_fallback(fallback),
                        )
                        .into_any_element(),
                }
            }
            Shown::Text(..) => stage
                .items_start()
                .justify_start()
                .bg(rgb(c.surface))
                .border_1()
                .border_color(rgb(c.field))
                .p(ui(12.))
                .font_family(mono_font(window))
                .text_size(ui(13.))
                .text_color(rgb(c.text))
                // The field is as high as what holds it, and scrolls past that.
                .child(div().size_full().flex().child(self.text.clone()))
                .into_any_element(),
        }
    }

    /// What is said under the file: its size, and the picture's or the text's.
    fn status(&self) -> String {
        let mut said = vec![self.file().map(|f| size(f.size)).unwrap_or_default()];
        match &self.shown {
            Shown::Picture(_, Some((w, h))) => said.push(format!("{w} × {h}")),
            Shown::Text(_, lines, cut) => {
                said.push(tr_n!(
                    *lines,
                    "{count} line",
                    "{count} lines",
                    count = crate::model::count(*lines as u64)
                ));
                if *cut {
                    said.push(tr!("Its first 256 KB are shown"));
                }
            }
            _ => {}
        }
        said.join(" · ")
    }
}

impl Render for Viewer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        div()
            .id("viewer")
            .debug_selector(|| "viewer".to_owned())
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::on_key))
            .flex_1()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .gap(ui(12.))
            .p(ui(16.))
            .rounded(ui(28.))
            .bg(rgb(c.surface))
            .child(self.toolbar(cx))
            .child(self.stage(window, cx))
            .child(
                div()
                    .px(ui(6.))
                    .text_size(ui(13.))
                    .text_color(rgb(c.muted))
                    .child(self.status()),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_go_as_people_number_them() {
        let mut pages = vec![
            "page10.jpg",
            "page2.jpg",
            "Page1.jpg",
            "page02b.jpg",
            "cover.jpg",
        ];
        pages.sort_by(|a, b| natural_order(a, b));
        assert_eq!(
            pages,
            [
                "cover.jpg",
                "Page1.jpg",
                "page2.jpg",
                "page02b.jpg",
                "page10.jpg"
            ]
        );
        assert_eq!(natural_order("a", "a"), Ordering::Equal);
        assert_eq!(natural_order("a", "ab"), Ordering::Less);
    }

    #[test]
    fn it_steps_through_what_it_shows() {
        let file = |name: &str| ViewFile {
            path: vec![name.to_owned()],
            name: name.to_owned(),
            size: 1,
            app: true,
            edit: false,
        };
        let files = ["a.jpg", "b.pdf", "c.txt", "d.zip", "e.png"]
            .map(file)
            .to_vec();
        let (kept, at) = stepping(files.clone(), "c.txt");
        let names: Vec<_> = kept.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["a.jpg", "c.txt", "e.png"]);
        assert_eq!(at, 1);
        // What it does not show is still shown, when it was chosen.
        let (kept, at) = stepping(files, "b.pdf");
        assert_eq!(kept.len(), 4);
        assert_eq!(kept[at].name, "b.pdf");
    }
}
