//! The viewer: a file of a folder, or of an archive, over the whole window, the
//! rail and the page hidden. A picture fits the window, or shows at its own size;
//! a text shows with its lines' numbers, to choose and copy from; anything else says why it cannot be shown
//! and offers the system's app, and packwing as an archive. Previous and Next step
//! through the folder's other pictures and texts, the next read ahead.
//!
//! It is a view of its own, told what to show and saying what was asked of it,
//! so that what holds it (the workspace) may put it anywhere.

use std::cmp::Ordering;
use std::ops::Range;
use std::sync::Arc;

use gpui::{
    Context, Entity, EventEmitter, FocusHandle, Focusable, FontWeight, Image, KeyDownEvent,
    ObjectFit, Render, StyledImage, Subscription, Task, Window, div, img, prelude::*, px, rgb,
};
use packwing_core::{Control, Error, Keys, Source, Storage, image_fits, image_size, preview_entry};
use packwing_i18n::{tr, tr_n};

use crate::colors::colors;
use crate::input::{Changed, Paint, TextInput};
use crate::model::{Kind, count, reads_as_text, size, tag_of, text_preview};
use crate::pages::archive::{MOST_PREVIEWED, picture_format};
use crate::text_size::ui;
use crate::widgets::{Fill, Size, button, icon, icon_button, kind_tile, mono_font};

/// A text's paint: none, all in the text's color.
fn unpainted(_: &str) -> Vec<(std::ops::Range<usize>, Paint)> {
    Vec::new()
}

/// How much of a text the viewer shows: 1 MB draws in about 8 ms a frame,
/// once shaped (a third of a second, when it is opened).
const MOST_TEXT: usize = 1024 * 1024;

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
pub(crate) enum Shown {
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
    /// A picture at its own size, where it fits the window otherwise: kept
    /// from one picture to the next, as the user chose it.
    actual: bool,
    /// The file after the one shown, read ahead, and what it showed.
    ahead: Option<(usize, Shown)>,
    _read: Option<Task<()>>,
    _read_ahead: Option<Task<()>>,
    /// A search in the text shown, while its bar is open.
    finding: Option<Finding>,
    /// How the search reads what is typed: kept from one file to the next.
    how: HowFound,
    focus: FocusHandle,
}

/// A search in the text shown: what is typed, what it found, and the one
/// chosen of it.
struct Finding {
    field: Entity<TextInput>,
    found: Vec<Range<usize>>,
    at: usize,
    /// What is typed is no pattern.
    invalid: bool,
    _typing: Subscription,
}

/// The most a search finds: more is not counted, nor marked.
const MOST_FOUND: usize = 10_000;

/// How a search reads what is typed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HowFound {
    /// The letters' case as typed, where any case is found otherwise.
    pub case: bool,
    /// A pattern (Rust's regex: no look-around, no back-references), `^`
    /// and `$` at each line's start and end.
    pub regex: bool,
}

/// Where `query` is in `text`, as `how` says, by byte ranges, apart,
/// [`MOST_FOUND`] at most; none empty. `None` when it is no pattern.
pub fn find_all(text: &str, query: &str, how: HowFound) -> Option<Vec<Range<usize>>> {
    if query.is_empty() {
        return Some(Vec::new());
    }
    if how.regex {
        let pattern = regex::RegexBuilder::new(query)
            .case_insensitive(!how.case)
            .multi_line(true)
            // A pattern typed is small; one that compiles to more is refused.
            .size_limit(1 << 20)
            .build()
            .ok()?;
        return Some(
            pattern
                .find_iter(text)
                .filter(|m| !m.is_empty())
                .take(MOST_FOUND)
                .map(|m| m.range())
                .collect(),
        );
    }
    if how.case {
        return Some(
            text.match_indices(query)
                .take(MOST_FOUND)
                .map(|(at, m)| at..at + m.len())
                .collect(),
        );
    }
    let query: Vec<char> = query.chars().collect();
    let same = |a: char, b: char| a == b || a.to_lowercase().eq(b.to_lowercase());
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut found = Vec::new();
    let mut i = 0;
    while i + query.len() <= chars.len() && found.len() < MOST_FOUND {
        if query
            .iter()
            .enumerate()
            .all(|(k, q)| same(chars[i + k].1, *q))
        {
            let start = chars[i].0;
            let end = chars.get(i + query.len()).map_or(text.len(), |c| c.0);
            found.push(start..end);
            i += query.len();
        } else {
            i += 1;
        }
    }
    Some(found)
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
    // What its name says nothing of is shown when chosen, not stepped onto.
    picture_format(name).is_some() || (reads_as_text(name) && Kind::of(name, false) != Kind::Other)
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

/// What shows of the file at `path` named `name`, of `size` bytes, read from
/// `reading`: the one answer the archive's preview and the viewer both give, a
/// text cut to `most_text` bytes. It blocks: it is run in the background.
pub(crate) fn look(
    reading: &Reading,
    path: &[String],
    name: &str,
    size: u64,
    most_text: usize,
    control: &Control,
) -> Shown {
    let format = picture_format(name);
    if format.is_none() && !reads_as_text(name) {
        return Shown::Not(tr!("packwing can’t show this kind of file"));
    }
    // An archive gives a file whole, and refuses one over what is asked:
    // what is read of it at most is the same for a picture and a text.
    let whole = matches!(reading, Reading::Archive { .. }) || format.is_some();
    if whole && size > MOST_PREVIEWED {
        return Shown::Not(tr!("Too big to show here"));
    }
    let most = if whole {
        MOST_PREVIEWED
    } else {
        most_text as u64 + 4
    };
    let bytes = match reading {
        Reading::Stored(storage) => storage.read(path, most).map_err(|e| e.to_string()),
        Reading::Archive { source, keys } => {
            match preview_entry(source, path, keys, most, control) {
                Ok(bytes) => Ok(bytes),
                Err(Error::Cancelled) => return Shown::Reading,
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
            let cut = size.max(bytes.len() as u64) > most_text as u64;
            match text_preview(&bytes, most_text) {
                Some(text) => {
                    // As it is, its last line's end too: Copy all gives the file.
                    let text = text.replace('\t', "    ");
                    let lines = text.lines().count();
                    Shown::Text(Arc::new(text), lines, cut)
                }
                None => Shown::Not(tr!("packwing can’t show this kind of file")),
            }
        }
    }
}

/// [`look`] for the viewer.
fn read(reading: &Reading, file: &ViewFile) -> Shown {
    look(
        reading,
        &file.path,
        &file.name,
        file.size,
        MOST_TEXT,
        &Control::default(),
    )
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
            finding: None,
            how: HowFound::default(),
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

    /// What the search found, and the one chosen of it, while its bar is open.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn finding(&self) -> Option<(usize, usize, bool)> {
        self.finding
            .as_ref()
            .map(|f| (f.found.len(), f.at, f.invalid))
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
        // The search goes on in the next text.
        self.find(cx);
    }

    /// Opens the search's bar, for the text shown, its field taking the keys.
    fn open_find(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !matches!(self.shown, Shown::Text(..)) {
            return;
        }
        self.open_find_bar(window, cx);
    }

    /// Searches for `query`, as typed, once the text is read: as the
    /// screenshots draw it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn find_for_shot(&mut self, query: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.open_find_bar(window, cx);
        if let Some(f) = &self.finding {
            f.field
                .update(cx, |t, cx| t.set_text(query, query.len()..query.len(), cx));
        }
    }

    fn open_find_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.finding.is_none() {
            let field = cx.new(|cx| TextInput::new(tr!("Find in text"), cx));
            let typing = cx.subscribe(&field, |this: &mut Self, _, _: &Changed, cx| this.find(cx));
            self.finding = Some(Finding {
                field,
                found: Vec::new(),
                at: 0,
                invalid: false,
                _typing: typing,
            });
        }
        if let Some(f) = &self.finding {
            window.focus(&f.field.focus_handle(cx), cx);
        }
        cx.notify();
    }

    /// Finds what is typed in the text shown, marks all of it, and chooses
    /// the first.
    fn find(&mut self, cx: &mut Context<Self>) {
        let Some(finding) = &mut self.finding else {
            return;
        };
        let query = finding.field.read(cx).text().to_owned();
        let found = match &self.shown {
            Shown::Text(text, ..) => find_all(text, &query, self.how),
            _ => Some(Vec::new()),
        };
        finding.invalid = found.is_none();
        finding.found = found.unwrap_or_default();
        finding.at = 0;
        let found = finding.found.clone();
        self.text.update(cx, |t, cx| {
            t.mark(found.clone(), cx);
            if let Some(first) = found.first() {
                t.select(first.clone(), cx);
            }
        });
        cx.notify();
    }

    /// Chooses the next of what was found, or with `back` the one before,
    /// from the last to the first and round.
    fn find_next(&mut self, back: bool, cx: &mut Context<Self>) {
        let Some(finding) = &mut self.finding else {
            return;
        };
        let n = finding.found.len();
        if n == 0 {
            return;
        }
        finding.at = if back {
            (finding.at + n - 1) % n
        } else {
            (finding.at + 1) % n
        };
        let chosen = finding.found[finding.at].clone();
        self.text.update(cx, |t, cx| t.select(chosen, cx));
        cx.notify();
    }

    /// Closes the search's bar: what it found is no longer marked.
    fn close_find(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.finding = None;
        self.text.update(cx, |t, cx| t.mark(Vec::new(), cx));
        window.focus(&self.focus, cx);
        cx.notify();
    }

    /// A switch of how the search reads what is typed, `on` or not, which
    /// `flip` turns.
    fn find_toggle(
        &self,
        id: &'static str,
        label: &'static str,
        on: bool,
        flip: fn(&mut HowFound),
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        button(
            id,
            label,
            if on { Fill::Tonal } else { Fill::Text },
            Size::Medium,
        )
        .px(ui(12.))
        .font_family(crate::FONT)
        .on_click(cx.listener(move |this, _, window, cx| {
            flip(&mut this.how);
            this.find(cx);
            // Back to the field, to type on.
            if let Some(f) = &this.finding {
                window.focus(&f.field.focus_handle(cx), cx);
            }
        }))
    }

    fn find_bar(&self, cx: &mut Context<Self>) -> Option<impl IntoElement + use<>> {
        let c = colors();
        let finding = self.finding.as_ref()?;
        let typed = !finding.field.read(cx).text().is_empty();
        let said = match finding.found.len() {
            _ if finding.invalid => tr!("Invalid pattern"),
            0 if typed => tr!("No matches"),
            0 => String::new(),
            n if n >= MOST_FOUND => format!("{} / {}+", finding.at + 1, count(n as u64)),
            n => format!("{} / {}", finding.at + 1, count(n as u64)),
        };
        Some(
            div()
                .flex()
                .items_center()
                .gap(ui(8.))
                .child(
                    div()
                        .w(ui(340.))
                        .h(ui(40.))
                        .px(ui(14.))
                        .flex()
                        .items_center()
                        .rounded_full()
                        .bg(rgb(c.field))
                        .text_size(ui(14.))
                        .child(finding.field.clone()),
                )
                .child(self.find_toggle("find-case", "Aa", self.how.case, |h| h.case = !h.case, cx))
                .child(self.find_toggle(
                    "find-regex",
                    ".*",
                    self.how.regex,
                    |h| h.regex = !h.regex,
                    cx,
                ))
                .child(
                    div()
                        .debug_selector(|| "find-count".to_owned())
                        .min_w(ui(90.))
                        .text_size(ui(13.))
                        .text_color(rgb(c.muted))
                        .child(said),
                )
                .child(
                    icon_button("find-previous", "chevron-up", 36.)
                        .on_click(cx.listener(|this, _, _, cx| this.find_next(true, cx))),
                )
                .child(
                    icon_button("find-next", "chevron-down", 36.)
                        .on_click(cx.listener(|this, _, _, cx| this.find_next(false, cx))),
                )
                .child(
                    icon_button("find-close", "x", 36.)
                        .on_click(cx.listener(|this, _, window, cx| this.close_find(window, cx))),
                ),
        )
    }

    /// Shows the file at `at`, read now, or ahead already.
    fn show(&mut self, at: usize, cx: &mut Context<Self>) {
        let Some(file) = self.files.get(at).cloned() else {
            return;
        };
        self.at = at;
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
        let (key, m) = (e.keystroke.key.as_str(), &e.keystroke.modifiers);
        let finding = self.finding.is_some();
        // Typing in the search's field: its keys are the field's, but these.
        let typing = self
            .finding
            .as_ref()
            .is_some_and(|f| f.field.focus_handle(cx).is_focused(window));
        match key {
            "f" if m.secondary() => self.open_find(window, cx),
            "f3" => self.find_next(m.shift, cx),
            "enter" if finding => self.find_next(m.shift, cx),
            "escape" if finding => self.close_find(window, cx),
            _ if typing => return,
            _ => return self.on_view_key(e, window, cx),
        }
        cx.stop_propagation();
    }

    /// The keys that step through the files and change how they show.
    fn on_view_key(&mut self, e: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
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
            // Its bar open, the bar is where it is.
            .when(
                matches!(self.shown, Shown::Text(..)) && self.finding.is_none(),
                |d| {
                    d.child(
                        button("view-find", tr!("Find"), Fill::Outline, Size::Medium)
                            .flex_row_reverse()
                            .child(icon("search", 16., c.text))
                            .on_click(
                                cx.listener(|this, _, window, cx| this.open_find(window, cx)),
                            ),
                    )
                },
            )
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
                                // Each picture's own scroll, from its top left.
                                div()
                                    .id(("view-actual-scroll", self.at))
                                    .debug_selector(|| "view-at-size".to_owned())
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
                    said.push(tr!("Its first 1 MB is shown"));
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
            .children(self.find_bar(cx))
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
    fn a_search_finds_whatever_the_case() {
        let any = HowFound::default();
        let text = "Xin chào\nxin CHÀO\nxinh";
        let found = find_all(text, "xin chào", any).expect("found");
        assert_eq!(found.len(), 2);
        assert_eq!(&text[found[1].clone()], "xin CHÀO");
        // Apart: one match does not start inside another.
        assert_eq!(find_all("aaaa", "aa", any), Some(vec![0..2, 2..4]));
        assert_eq!(find_all(text, "", any), Some(Vec::new()));
        assert_eq!(find_all(text, "missing", any), Some(Vec::new()));
        // Its case as typed.
        let case = HowFound {
            case: true,
            regex: false,
        };
        assert_eq!(find_all(text, "xin", case).map(|f| f.len()), Some(2));
        assert_eq!(find_all(text, "CHÀO", case).map(|f| f.len()), Some(1));
    }

    #[test]
    fn a_search_by_pattern() {
        let regex = HowFound {
            case: false,
            regex: true,
        };
        let log = "ERROR disk full\nINFO ok\nerror: retry 3\nWARN 42 errors";
        // ^ at each line's start, any case.
        let found = find_all(log, "^error", regex).expect("a pattern");
        assert_eq!(found.len(), 2);
        assert_eq!(&log[found[1].clone()], "error");
        let digits = find_all(log, r"\d+", regex).expect("a pattern");
        let digits: Vec<&str> = digits.into_iter().map(|r| &log[r]).collect();
        assert_eq!(digits, ["3", "42"]);
        // Its case as typed.
        let case = HowFound {
            case: true,
            regex: true,
        };
        assert_eq!(find_all(log, "^error", case).map(|f| f.len()), Some(1));
        // What finds nothing but empty matches finds nothing.
        assert_eq!(find_all(log, "^", regex), Some(Vec::new()));
        assert_eq!(find_all(log, "x*", regex), Some(Vec::new()));
        // No pattern.
        assert_eq!(find_all(log, "[abc", regex), None);
        assert_eq!(find_all(log, "(?<=a)b", regex), None);
        // Not a pattern without the switch: the brackets are letters.
        let letters = find_all("a[abc", "[abc", HowFound::default()).expect("found");
        assert_eq!(letters.len(), 1);
        assert_eq!(letters[0], 1..5);
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
        let files = ["a.jpg", "b.pdf", "c.txt", "d.zip", "e.png", "f.sh", "g.dat"]
            .map(file)
            .to_vec();
        let (kept, at) = stepping(files.clone(), "c.txt");
        let names: Vec<_> = kept.iter().map(|f| f.name.as_str()).collect();
        // A script is a text; what its name says nothing of is not stepped onto.
        assert_eq!(names, ["a.jpg", "c.txt", "e.png", "f.sh"]);
        assert_eq!(at, 1);
        // What it does not show is still shown, when it was chosen.
        let (kept, at) = stepping(files.clone(), "b.pdf");
        assert_eq!(kept.len(), 5);
        assert_eq!(kept[at].name, "b.pdf");
        let (kept, at) = stepping(files, "g.dat");
        assert_eq!(kept[at].name, "g.dat");
    }
}
