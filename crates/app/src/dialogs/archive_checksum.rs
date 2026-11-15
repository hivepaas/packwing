//! What the Archive page's Checksums… reads, asked first: the files selected in
//! it, all of its files, or the archive's own file, which [`ChecksumDialog`] then
//! shows. Of its files, each one's, made in one read of the archive, are listed,
//! each shown with all of them, copied, and kept beside the archive as lists that
//! `sha256sum -c` checks what is extracted with.

use packwing_i18n::{tr, tr_n};
use std::ops::Range;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use gpui::{
    AnyElement, ClipboardItem, Context, Div, Entity, EventEmitter, FocusHandle, Focusable as _,
    FontWeight, KeyDownEvent, PathPromptOptions, Render, ScrollHandle, ScrollStrategy,
    SharedString, Stateful, Subscription, Task, UniformListScrollHandle, Window, div, prelude::*,
    relative, rgb, uniform_list,
};
use packwing_core::{
    Algorithm, Cancel, Control, EntryChecksums, Error, ListCompared, Listed, LocalFolder, Progress,
    Storage, Verdict, algorithms_of, checksum_in, checksum_list, checksum_list_in, compare_list,
    entry_checksums, save_checksum_list,
};

use super::{ChecksumDialog, DialogEvent, footer, header};
use crate::colors::colors;
use crate::input::{Changed, TextInput};
use crate::list_keys;
use crate::model::{Archive, count, size};
use crate::prefs;
use crate::scrollbar::{scrollbar, well_bar};
use crate::text_size::ui;
use crate::widgets::{
    Fill, Size, Tone, button, check_row, icon, icon_button, legend, mono_font, note, option_card,
    tip, titled, tonal_tint,
};

/// How often the bytes read are shown again.
const TICK: Duration = Duration::from_millis(200);

/// How long a checksum copied says so.
const COPIED_FOR: Duration = Duration::from_secs(2);

/// A file's row in the list, as tall as the button on it.
const ROW: f32 = 48.;

/// The most rows shown at once: more scroll.
const MOST_ROWS: usize = 6;

/// The fewest rows shown, where the window is short.
const FEWEST_ROWS: usize = 3;

/// Around the rows, inside their well.
const WELL_PAD: f32 = 6.;

/// What is read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// The files and folders selected, with what is in them.
    Picked,
    /// All of the archive's files.
    All,
    /// The archive's own file.
    File,
}

/// What is selected in the folder shown, and how much it holds.
#[derive(Clone, Debug, Default)]
pub struct Selected {
    /// By their paths in the archive.
    pub paths: Vec<Vec<String>>,
    /// The files in them: a folder's, all it holds.
    pub files: u64,
    /// Unpacked, in bytes.
    pub bytes: u64,
}

/// A read going on. Dropped, it stops.
struct Reading {
    read: Arc<AtomicU64>,
    /// What there is to read, once the archive says.
    total: Arc<AtomicU64>,
    cancel: Cancel,
    _task: Task<()>,
    _ticking: Task<()>,
}

impl Drop for Reading {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

/// Which Copy says it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Copied {
    /// One of the file shown.
    Shown(Algorithm),
    /// The list of all of them.
    All,
}

/// What a list pasted says.
enum Pasted {
    /// Nothing in it is a checksum.
    NotOne,
    Compared(ListCompared),
    /// One checksum alone, naming no file: `digits` long, and the files it is
    /// of, by where they are in what was made, with the algorithm. It may be
    /// one of `not_made`, which were not made.
    One {
        digits: usize,
        found: Vec<(usize, Algorithm)>,
        not_made: Vec<Algorithm>,
    },
}

/// What a file's row shows of what the list says of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mark {
    Matches,
    Differs,
    /// The list's is of a length none made has: it may be one not made yet.
    Maybe,
    NotListed,
}

impl Mark {
    fn of(verdict: &Verdict) -> Mark {
        match verdict {
            Verdict::Matches(_) => Mark::Matches,
            // Of none made of its length: it may be one not made yet.
            Verdict::Differs { made, .. } if made.is_empty() => Mark::Maybe,
            Verdict::Differs { .. } | Verdict::Unknown(_) => Mark::Differs,
            Verdict::NotListed => Mark::NotListed,
        }
    }

    fn key(self) -> &'static str {
        match self {
            Mark::Matches => "matches",
            Mark::Differs => "differs",
            Mark::Maybe => "maybe",
            Mark::NotListed => "not-listed",
        }
    }
}

/// Names, apart by slashes: they are not words to translate.
fn names(algorithms: &[Algorithm]) -> String {
    let named: Vec<_> = algorithms.iter().map(|a| a.name()).collect();
    named.join(" / ")
}

enum Stage {
    Choosing,
    /// The archive's own file, as the Storages page shows one.
    File {
        view: Entity<ChecksumDialog>,
        _told: Subscription,
    },
    /// Its files.
    Files,
}

pub struct ArchiveChecksumDialog {
    archive: Archive,
    selection: Selected,
    /// It was opened from another archive, from a copy of its own: what is
    /// kept goes where the user says, not beside that copy.
    inside: bool,
    scope: Scope,
    stage: Stage,
    /// Those made, in [`Algorithm::ALL`]'s order: the first is shown on each row.
    algorithms: Vec<Algorithm>,
    reading: Option<Reading>,
    made: Option<EntryChecksums>,
    problem: Option<String>,
    /// The file whose checksums are all shown.
    shown: Option<usize>,
    scroll: UniformListScrollHandle,
    /// What is under the header, when it is taller than the window.
    body: ScrollHandle,
    /// Up, Down and the others move the file shown.
    focus: FocusHandle,
    copied: Option<Copied>,
    saving: bool,
    /// The files they were kept in, the last time.
    saved: Vec<String>,
    /// Made too, as a list pasted asked: not shown in Settings › Security.
    more: Vec<Algorithm>,
    /// The field for a list to compare is shown.
    comparing: bool,
    pasted: Entity<TextInput>,
    /// What the list pasted says, once they are made.
    compared: Option<Pasted>,
    /// Only the files that are not as the list says are listed.
    only_differing: bool,
    mono: Option<SharedString>,
    _typing: Subscription,
}

impl EventEmitter<DialogEvent> for ArchiveChecksumDialog {}

impl ArchiveChecksumDialog {
    /// For `archive`, read through, with `selection` selected in it.
    pub fn new(
        archive: Archive,
        selection: Selected,
        inside: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        let scope = first_scope(&selection);
        let pasted = cx.new(|cx| {
            TextInput::said(
                || tr!("Paste the checksums given with the files, or one of them"),
                cx,
            )
            .multiline(2, 6)
        });
        let typing = cx.subscribe(&pasted, |this, _, _: &Changed, cx| this.compare(cx));
        ArchiveChecksumDialog {
            archive,
            selection,
            inside,
            scope,
            stage: Stage::Choosing,
            algorithms: Vec::new(),
            reading: None,
            made: None,
            problem: None,
            shown: None,
            scroll: UniformListScrollHandle::new(),
            body: ScrollHandle::new(),
            focus: cx.focus_handle(),
            copied: None,
            saving: false,
            saved: Vec::new(),
            more: Vec::new(),
            comparing: false,
            pasted,
            compared: None,
            only_differing: false,
            mono: None,
            _typing: typing,
        }
    }

    /// Reads `scope` now, as the screenshots draw it read.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn read_for_test(&mut self, scope: Scope, cx: &mut Context<Self>) {
        self.scope = scope;
        if scope == Scope::File {
            self.start(cx);
            if let Stage::File { view, .. } = &self.stage {
                view.update(cx, |d, cx| d.read_for_test(cx));
            }
            return;
        }
        self.algorithms = self.wanted(cx);
        self.stage = Stage::Files;
        let source = self.archive.source.clone().expect("a file of its own");
        let made = entry_checksums(
            source,
            &self.picked(),
            &self.archive.keys,
            &self.algorithms,
            &Control::default(),
        )
        .expect("the archive reads");
        self.read(Ok(made), cx);
    }

    /// Shows all of the checksums of the file at `i`, as a click on it does.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn show_for_test(&mut self, i: usize, cx: &mut Context<Self>) {
        self.shown = Some(i);
        cx.notify();
    }

    /// What scrolls under the header, for tests.
    #[cfg(all(test, feature = "shots"))]
    pub fn body_for_test(&self) -> ScrollHandle {
        self.body.clone()
    }

    /// Keeps the lists beside the archive now, as the screenshots draw them kept.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn save_for_test(&mut self, cx: &mut Context<Self>) {
        let (storage, path) = self.beside().expect("a file of its own");
        let files = &self.made.as_ref().expect("read").files;
        self.saved = self
            .algorithms
            .iter()
            .map(|a| save_checksum_list(storage.as_ref(), &path, *a, files))
            .collect::<Result<_, _>>()
            .expect("they are kept");
        cx.notify();
    }

    /// The paths read: those picked, or none for all of them.
    fn picked(&self) -> Vec<Vec<String>> {
        match self.scope {
            Scope::Picked => self.selection.paths.clone(),
            _ => Vec::new(),
        }
    }

    /// Whether the archive is across a network, where reading it may cost.
    fn far(&self) -> bool {
        self.archive
            .source
            .as_ref()
            .is_some_and(|s| s.path().is_none())
    }

    /// The archive's file, where the lists are kept beside it: none for one
    /// opened from another, whose file is a copy.
    fn beside(&self) -> Option<(Arc<dyn Storage>, Vec<String>)> {
        if self.inside {
            return None;
        }
        self.archive.source.as_ref().and_then(|s| s.place())
    }

    /// Reads what is chosen.
    fn start(&mut self, cx: &mut Context<Self>) {
        if self.scope == Scope::File {
            let Some((storage, path)) = self.archive.source.as_ref().and_then(|s| s.place()) else {
                return;
            };
            let view = cx.new(|cx| ChecksumDialog::reading(storage, path, cx));
            // What it asks is this dialog's to ask.
            let told = cx.subscribe(&view, |_, _, e: &DialogEvent, cx| match e {
                DialogEvent::Close => cx.emit(DialogEvent::Close),
                DialogEvent::Saved => cx.emit(DialogEvent::Saved),
                DialogEvent::ShowChecksumSettings => cx.emit(DialogEvent::ShowChecksumSettings),
                _ => {}
            });
            self.stage = Stage::File { view, _told: told };
            cx.notify();
            return;
        }
        let Some(source) = self.archive.source.clone() else {
            return;
        };
        self.stage = Stage::Files;
        self.algorithms = self.wanted(cx);
        self.problem = None;
        let (read, total) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
        let cancel = Cancel::new();
        let (told, all) = (read.clone(), total.clone());
        let control = Control {
            progress: Some(Arc::new(move |p: &Progress| {
                told.store(p.bytes, Ordering::Relaxed);
                all.store(p.total_bytes, Ordering::Relaxed);
            })),
            cancel: Some(cancel.clone()),
            ..Control::default()
        };
        let (picked, keys, algorithms) = (
            self.picked(),
            self.archive.keys.clone(),
            self.algorithms.clone(),
        );
        let work = cx.background_spawn(async move {
            entry_checksums(source, &picked, &keys, &algorithms, &control)
        });
        let task = cx.spawn(async move |this, cx| {
            let result = work.await;
            let _ = this.update(cx, |this, cx| this.read(result, cx));
        });
        let ticking = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        });
        self.reading = Some(Reading {
            read,
            total,
            cancel,
            _task: task,
            _ticking: ticking,
        });
        cx.notify();
    }

    fn read(&mut self, result: Result<EntryChecksums, Error>, cx: &mut Context<Self>) {
        self.reading = None;
        match result {
            Ok(made) => {
                // One file alone: all of its checksums are shown.
                if self.made.is_none() {
                    self.shown = (made.files.len() == 1).then_some(0);
                }
                self.made = Some(made);
                self.compare(cx);
            }
            Err(Error::Cancelled) => {}
            Err(e) => self.problem = Some(tr!("The file could not be read: {why}", why = e)),
        }
        cx.notify();
    }

    /// Up and Down, Page Up and Page Down, Home and End move the file shown,
    /// as they move the chosen row of a list (see [`list_keys`]).
    fn on_key(&mut self, e: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        // In the field, they move in what was pasted.
        if self.pasted.focus_handle(cx).is_focused(window) {
            return;
        }
        let listed = self.listed();
        let from = self.shown.and_then(|s| listed.iter().position(|i| *i == s));
        let page = list_keys::rows_per_page(&self.scroll, listed.len());
        let key = e.keystroke.key.as_str();
        let Some(to) = list_keys::moved(key, from, listed.len(), page) else {
            return;
        };
        self.shown = Some(listed[to]);
        self.scroll.scroll_to_item(to, ScrollStrategy::Nearest);
        cx.stop_propagation();
        cx.notify();
    }

    /// Those Settings › Security shows, and those a list pasted asked for.
    fn wanted(&self, cx: &gpui::App) -> Vec<Algorithm> {
        let mut wanted = shown_algorithms(cx);
        wanted.extend(self.more.iter().copied());
        Algorithm::ALL
            .into_iter()
            .filter(|a| wanted.contains(a))
            .collect()
    }

    /// The files listed, by where they are in what was made: all of them, or
    /// those the list pasted does not say are as it says.
    fn listed(&self) -> Vec<usize> {
        let n = self.made.as_ref().map_or(0, |m| m.files.len());
        match (&self.compared, self.only_differing) {
            (Some(Pasted::Compared(c)), true) => (0..n)
                .filter(|i| !matches!(c.verdicts.get(*i), Some(Verdict::Matches(_))))
                .collect(),
            _ => (0..n).collect(),
        }
    }

    /// What the list pasted says of the file at `i`.
    fn verdict(&self, i: usize) -> Option<Verdict> {
        match &self.compared {
            Some(Pasted::Compared(c)) => c.verdicts.get(i).cloned(),
            // One alone: those it is of match; it says nothing of the others.
            Some(Pasted::One { found, .. }) => found
                .iter()
                .find(|(f, _)| *f == i)
                .map(|(_, a)| Verdict::Matches(*a)),
            _ => None,
        }
    }

    /// The list pasted, compared with them again.
    fn compare(&mut self, cx: &mut Context<Self>) {
        let text = self.pasted.read(cx).text().to_owned();
        self.compared = match &self.made {
            Some(made) if !text.trim().is_empty() => {
                let mut listed = checksum_list_in(&text);
                // A checksum alone, of the one file there is.
                if listed.is_empty()
                    && let [one] = made.files.as_slice()
                    && let Some(sum) = checksum_in(&text, &one.name)
                {
                    listed.push(Listed {
                        sum,
                        name: one.name.clone(),
                    });
                }
                Some(if !listed.is_empty() {
                    Pasted::Compared(compare_list(&made.files, &listed))
                } else if let Some(sum) = checksum_in(&text, "") {
                    // One checksum alone, of whichever file has it.
                    let may_be = algorithms_of(sum.len());
                    let found: Vec<(usize, Algorithm)> = made
                        .files
                        .iter()
                        .enumerate()
                        .filter_map(|(i, f)| {
                            may_be
                                .iter()
                                .find(|a| f.sum(**a) == Some(sum.as_str()))
                                .map(|a| (i, *a))
                        })
                        .collect();
                    let not_made = may_be
                        .into_iter()
                        .filter(|a| !self.algorithms.contains(a))
                        .collect();
                    Pasted::One {
                        digits: sum.len(),
                        found,
                        not_made,
                    }
                } else {
                    Pasted::NotOne
                })
            }
            _ => None,
        };
        // The file it is of, shown.
        if let Some(Pasted::One { found, .. }) = &self.compared
            && let Some((i, _)) = found.first()
        {
            let i = *i;
            self.shown = Some(i);
            if let Some(at) = self.listed().iter().position(|f| *f == i) {
                self.scroll.scroll_to_item(at, ScrollStrategy::Nearest);
            }
        }
        cx.notify();
    }

    /// Reads them all again, with `more` made too, as the list pasted may
    /// hold checksums of those.
    fn make_more(&mut self, more: Vec<Algorithm>, cx: &mut Context<Self>) {
        if self.reading.is_some() {
            return;
        }
        self.more.extend(more);
        self.start(cx);
    }

    /// Shows the field for a list, with `pasted` in it, as the screenshots draw it.
    #[cfg(all(test, feature = "shots"))]
    pub fn compare_for_test(&mut self, pasted: &str, only_differing: bool, cx: &mut Context<Self>) {
        self.comparing = true;
        self.only_differing = only_differing;
        self.pasted.update(cx, |f, cx| {
            f.set_text(pasted, pasted.len()..pasted.len(), cx)
        });
        self.compare(cx);
    }

    /// The list of their first checksums, as Copy all copies one.
    #[cfg(all(test, feature = "shots"))]
    pub fn list_for_test(&self) -> String {
        checksum_list(&self.made.as_ref().expect("read").files, self.algorithms[0])
    }

    fn copy(&mut self, which: Copied, text: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.copied = Some(which);
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(COPIED_FOR).await;
            let _ = this.update(cx, |this, cx| {
                if this.copied == Some(which) {
                    this.copied = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// The list of them all: as `sha256sum` writes it, of one algorithm; with
    /// several, a line for each, tagged as BSD's tools and `pkw` write them.
    fn copy_all(&mut self, cx: &mut Context<Self>) {
        let Some(made) = &self.made else {
            return;
        };
        let list = match self.algorithms.as_slice() {
            [one] => checksum_list(&made.files, *one),
            several => made
                .files
                .iter()
                .flat_map(|f| {
                    several.iter().filter_map(move |a| {
                        f.sum(*a)
                            .map(|sum| format!("{} ({}) = {sum}\n", tag(*a), f.name))
                    })
                })
                .collect(),
        };
        self.copy(Copied::All, list, cx);
    }

    /// Keeps a list of each checksum beside the archive; or, for one opened from
    /// another, in a folder the user picks.
    fn save(&mut self, cx: &mut Context<Self>) {
        if self.saving || self.reading.is_some() || self.made.is_none() {
            return;
        }
        if let Some((storage, path)) = self.beside() {
            self.save_into(storage, path, cx);
            return;
        }
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(tr!("Choose").into()),
        });
        let name = self.archive.name.clone();
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(mut paths))) = picked.await
                && let Some(folder) = paths.pop()
            {
                let storage: Arc<dyn Storage> = Arc::new(LocalFolder::new(&folder));
                let _ = this.update(cx, |this, cx| this.save_into(storage, vec![name], cx));
            }
        })
        .detach();
    }

    fn save_into(&mut self, storage: Arc<dyn Storage>, path: Vec<String>, cx: &mut Context<Self>) {
        let Some(made) = &self.made else {
            return;
        };
        self.saving = true;
        self.problem = None;
        let (files, algorithms) = (made.files.clone(), self.algorithms.clone());
        let saving = cx.background_spawn(async move {
            algorithms
                .iter()
                .map(|a| save_checksum_list(storage.as_ref(), &path, *a, &files))
                .collect::<Result<Vec<_>, _>>()
        });
        cx.spawn(async move |this, cx| {
            let result = saving.await;
            let _ = this.update(cx, |this, cx| {
                this.saving = false;
                match result {
                    Ok(saved) => {
                        this.saved = saved;
                        cx.emit(DialogEvent::Saved);
                    }
                    Err(e) => {
                        this.saved.clear();
                        this.problem = Some(tr!("They could not be saved: {why}", why = e));
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    /// One of the three to choose from.
    fn option(
        &self,
        id: &'static str,
        scope: Scope,
        title: String,
        detail: String,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let on = self.scope == scope;
        let can = scope != Scope::Picked || !self.selection.paths.is_empty();
        option_card(id, on)
            .child(titled(title, detail, on).flex_1().min_w_0())
            .when(can, |d| {
                d.on_click(cx.listener(move |this, _, _, cx| {
                    this.scope = scope;
                    cx.notify();
                }))
            })
            .when(!can, |d| d.opacity(0.5).cursor_default())
    }

    fn choosing(&self, cx: &mut Context<Self>) -> Div {
        let c = colors();
        let files = |n: u64| tr_n!(n, "{count} file", "{count} files", count = count(n));
        let s = &self.selection;
        let picked = match s.paths.as_slice() {
            [one] => one.last().cloned().unwrap_or_default(),
            all => tr!("{picked} selected", picked = all.len()),
        };
        let picked_detail = if s.paths.is_empty() {
            tr!("Select files or folders in it first.")
        } else {
            format!("{} · {}", files(s.files), size(s.bytes))
        };
        let a = &self.archive;
        let options = div()
            .flex()
            .flex_col()
            .gap(ui(8.))
            .child(legend(tr!("What should it read?")))
            .child(self.option(
                "checksum-scope-picked",
                Scope::Picked,
                picked,
                picked_detail,
                cx,
            ))
            .child(self.option(
                "checksum-scope-all",
                Scope::All,
                tr!("All {counted}", counted = files(a.files)),
                size(a.unpacked),
                cx,
            ))
            .child(self.option(
                "checksum-scope-file",
                Scope::File,
                tr!("The archive itself"),
                format!("{} · {}", a.name, size(a.packed)),
                cx,
            ));
        let far = self.far().then(|| {
            let said = if self.scope == Scope::File {
                tr!(
                    "All of the file passes through this computer, and your provider may \
                     charge for what is read from it."
                )
            } else {
                tr!(
                    "What is read of it passes through this computer, and your provider \
                     may charge for it."
                )
            };
            div()
                .text_size(ui(13.))
                .text_color(rgb(c.muted))
                .child(said)
        });
        let close = icon_button("close", "x", 40.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        super::card()
            .child(header(
                "check",
                tonal_tint(),
                tr!("Checksums"),
                self.archive.name.clone(),
                Some(close),
            ))
            .child(options)
            .children(far)
            .child(
                footer()
                    .child(settings_button(cx))
                    .child(div().flex_1())
                    .child(
                        button("checksum-start", tr!("Read it"), Fill::Accent, Size::Medium)
                            .px(ui(28.))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.start(cx);
                                window.focus(&this.focus, cx);
                            })),
                    ),
            )
    }

    /// A file's row: its name and its size. A click shows its checksums.
    fn file_row(&self, i: usize, cx: &mut Context<Self>) -> AnyElement {
        let c = colors();
        let Some(file) = self.made.as_ref().and_then(|m| m.files.get(i)) else {
            return div().into_any_element();
        };
        let on = self.shown == Some(i);
        div()
            .id(("checksum-file", i))
            .debug_selector(move || format!("checksum-file-{i}"))
            .w_full()
            .h(ui(ROW))
            .flex()
            .items_center()
            .gap(ui(12.))
            .px(ui(14.))
            .rounded(ui(14.))
            .cursor_pointer()
            .when(on, |d| d.bg(rgb(c.tonal)).text_color(rgb(c.on_tonal)))
            .when(!on, |d| d.hover(|s| s.bg(rgb(c.well_hover))))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.shown = if this.shown == Some(i) { None } else { Some(i) };
                window.focus(&this.focus, cx);
                cx.notify();
            }))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(ui(14.))
                    .when(on, |d| d.font_weight(FontWeight::SEMIBOLD))
                    .child(file.name.clone()),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .text_size(ui(13.))
                    .text_color(rgb(if on { c.on_tonal } else { c.muted }))
                    .child(size(file.size)),
            )
            .children(self.verdict(i).map(|v| mark_view(Mark::of(&v), i)))
            .into_any_element()
    }

    /// The file shown: each of its checksums, with Copy.
    fn shown_file(&self, mono: SharedString, cx: &mut Context<Self>) -> Option<Div> {
        let c = colors();
        let at = self.shown?;
        let file = self.made.as_ref()?.files.get(at)?;
        let verdict = self.verdict(at);
        let matched = match &verdict {
            Some(Verdict::Matches(a)) => Some(*a),
            _ => None,
        };
        let rows = file.sums.iter().map(|(algorithm, sum)| {
            let (algorithm, sum) = (*algorithm, sum.clone());
            let on = matched == Some(algorithm);
            let copy = if self.copied == Some(Copied::Shown(algorithm)) {
                tr!("Copied")
            } else {
                tr!("Copy")
            };
            div()
                .flex()
                .items_center()
                .gap(ui(12.))
                .min_h(ui(48.))
                .px(ui(16.))
                .py(ui(6.))
                .rounded(ui(16.))
                .bg(rgb(if on { c.tonal } else { c.well }))
                .child(
                    div()
                        .w(ui(84.))
                        .flex_shrink_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(ui(4.))
                                .text_size(ui(14.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .when(on, |d| d.child(icon("check", 16., c.accent)))
                                .child(algorithm.name()),
                        )
                        .when(algorithm.is_weak(), |d| {
                            d.child(
                                div()
                                    .text_size(ui(12.))
                                    .text_color(rgb(c.muted))
                                    .child(tr!("weak")),
                            )
                        }),
                )
                .child(
                    div()
                        .debug_selector(move || format!("checksum-shown: {}", algorithm.key()))
                        .flex_1()
                        .min_w_0()
                        .font_family(mono.clone())
                        .text_size(ui(13.))
                        .line_height(relative(1.45))
                        .child(sum.clone()),
                )
                .child(
                    button(
                        ("checksum-shown-copy", algorithm as usize),
                        copy,
                        Fill::Text,
                        Size::Medium,
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.copy(Copied::Shown(algorithm), sum.clone(), cx)
                    })),
                )
        });
        // What the list pasted says of it, but where it matches, as is shown.
        let said = match verdict {
            Some(Verdict::Differs { made, .. }) if !made.is_empty() => Some(
                note(Tone::Bad)
                    .child(tr!(
                        "It does not match the file’s {name} checksum: this is not the \
                         file it was made of, or the file changed on the way.",
                        name = names(&made)
                    ))
                    .into_any_element(),
            ),
            Some(Verdict::Unknown(digits)) => Some(
                note(Tone::Warn)
                    .child(tr!(
                        "This is not a checksum packwing makes: it has {digits} digits.",
                        digits = digits
                    ))
                    .into_any_element(),
            ),
            Some(Verdict::NotListed) => Some(
                div()
                    .text_size(ui(13.))
                    .text_color(rgb(c.muted))
                    .child(tr!("The list does not name it."))
                    .into_any_element(),
            ),
            _ => None,
        };
        Some(
            div()
                .flex()
                .flex_col()
                .gap(ui(6.))
                .child(legend(file.name.clone()))
                .children(rows)
                .children(said),
        )
    }

    /// The field for a list of checksums to compare, and what it says.
    fn compare_section(&self, mono: SharedString, cx: &mut Context<Self>) -> Option<Div> {
        if !self.comparing {
            return None;
        }
        let c = colors();
        let mut said: Vec<AnyElement> = Vec::new();
        match &self.compared {
            None => {}
            Some(Pasted::NotOne) => said.push(
                note(Tone::Warn)
                    .child(tr!("There is no checksum in this."))
                    .into_any_element(),
            ),
            Some(Pasted::One {
                digits,
                found,
                not_made,
            }) => {
                let files = self.made.as_ref().map_or(&[][..], |m| &m.files[..]);
                let made_any = algorithms_of(*digits)
                    .iter()
                    .any(|a| self.algorithms.contains(a));
                if algorithms_of(*digits).is_empty() {
                    said.push(
                        note(Tone::Warn)
                            .child(tr!(
                                "This is not a checksum packwing makes: it has {digits} digits.",
                                digits = digits
                            ))
                            .into_any_element(),
                    );
                } else if let [(i, algorithm)] = found.as_slice() {
                    let name = files.get(*i).map(|f| f.name.clone()).unwrap_or_default();
                    said.push(
                        note(Tone::Good)
                            .debug_selector(|| "checksum-one-matches".to_owned())
                            .flex()
                            .items_center()
                            .gap(ui(8.))
                            .child(icon("check", 16., c.on_tonal))
                            .child(tr!(
                                "It matches the {name} checksum of {file}.",
                                name = algorithm.name(),
                                file = name
                            ))
                            .into_any_element(),
                    );
                } else if !found.is_empty() {
                    let n = found.len() as u64;
                    let first: Vec<&str> = found
                        .iter()
                        .take(3)
                        .filter_map(|(i, _)| files.get(*i).map(|f| f.name.as_str()))
                        .collect();
                    said.push(
                        note(Tone::Good)
                            .debug_selector(|| "checksum-one-matches".to_owned())
                            .child(tr_n!(
                                n,
                                "It matches {count} file: {names}",
                                "It matches {count} files: {names}",
                                count = count(n),
                                names = first.join(", ")
                            ))
                            .into_any_element(),
                    );
                } else if made_any {
                    said.push(
                        note(Tone::Bad)
                            .debug_selector(|| "checksum-none-match".to_owned())
                            .child(tr!("No file in it has this checksum."))
                            .into_any_element(),
                    );
                }
                if found.is_empty() && !not_made.is_empty() && self.reading.is_none() {
                    let asked = not_made.clone();
                    said.push(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap(ui(8.))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(ui(200.))
                                    .text_size(ui(13.))
                                    .text_color(rgb(c.muted))
                                    .child(tr!(
                                        "It may be a {name} checksum, which is not made yet.",
                                        name = names(not_made)
                                    )),
                            )
                            .child(
                                button("checksum-make", tr!("Make it"), Fill::Tonal, Size::Medium)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.make_more(asked.clone(), cx)
                                    })),
                            )
                            .into_any_element(),
                    );
                }
            }
            Some(Pasted::Compared(compared)) => {
                let tally = |mark: Mark| {
                    compared
                        .verdicts
                        .iter()
                        .filter(|v| Mark::of(v) == mark)
                        .count() as u64
                };
                let (matching, differing, unlisted) = (
                    tally(Mark::Matches),
                    tally(Mark::Differs),
                    tally(Mark::NotListed),
                );
                if compared.all_match() {
                    said.push(
                        note(Tone::Good)
                            .debug_selector(|| "checksum-all-match".to_owned())
                            .flex()
                            .items_center()
                            .gap(ui(8.))
                            .child(icon("check", 16., c.on_tonal))
                            .child(tr!("Every file matches the list."))
                            .into_any_element(),
                    );
                } else {
                    let mut parts = vec![tr_n!(
                        matching,
                        "{count} file matches",
                        "{count} files match",
                        count = count(matching)
                    )];
                    if differing > 0 {
                        parts.push(tr_n!(
                            differing,
                            "{count} file differs",
                            "{count} files differ",
                            count = count(differing)
                        ));
                    }
                    if unlisted > 0 {
                        parts.push(tr_n!(
                            unlisted,
                            "{count} file is not in the list",
                            "{count} files are not in the list",
                            count = count(unlisted)
                        ));
                    }
                    let line = parts.join(" · ");
                    said.push(if differing > 0 {
                        note(Tone::Bad)
                            .debug_selector(|| "checksum-differs".to_owned())
                            .child(line)
                            .into_any_element()
                    } else {
                        div()
                            .debug_selector(|| "checksum-summary".to_owned())
                            .text_size(ui(13.))
                            .text_color(rgb(c.muted))
                            .child(line)
                            .into_any_element()
                    });
                }
                if !compared.not_found.is_empty() {
                    let n = compared.not_found.len() as u64;
                    let first: Vec<&str> = compared
                        .not_found
                        .iter()
                        .take(3)
                        .map(String::as_str)
                        .collect();
                    said.push(
                        note(Tone::Warn)
                            .debug_selector(|| "checksum-not-found".to_owned())
                            .child(tr_n!(
                                n,
                                "{count} line names a file it does not hold: {names}",
                                "{count} lines name files it does not hold: {names}",
                                count = count(n),
                                names = first.join(", ")
                            ))
                            .into_any_element(),
                    );
                }
                // Checksums of a length only those not made have, which it may be.
                let not_made: Vec<Algorithm> = Algorithm::ALL
                    .into_iter()
                    .filter(|a| {
                        compared.verdicts.iter().any(|v| {
                            matches!(v, Verdict::Differs { made, not_made }
                                if made.is_empty() && not_made.contains(a))
                        })
                    })
                    .collect();
                if !not_made.is_empty() && self.reading.is_none() {
                    let asked = not_made.clone();
                    said.push(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap(ui(8.))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(ui(200.))
                                    .text_size(ui(13.))
                                    .text_color(rgb(c.muted))
                                    .child(tr!(
                                        "It may be a {name} checksum, which is not made yet.",
                                        name = names(&not_made)
                                    )),
                            )
                            .child(
                                button("checksum-make", tr!("Make it"), Fill::Tonal, Size::Medium)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.make_more(asked.clone(), cx)
                                    })),
                            )
                            .into_any_element(),
                    );
                }
                if let Some(digits) = compared.verdicts.iter().find_map(|v| match v {
                    Verdict::Unknown(d) => Some(*d),
                    _ => None,
                }) {
                    said.push(
                        note(Tone::Warn)
                            .child(tr!(
                                "This is not a checksum packwing makes: it has {digits} digits.",
                                digits = digits
                            ))
                            .into_any_element(),
                    );
                }
                if !compared.all_match() && matching > 0 {
                    said.push(
                        check_row(
                            "checksum-only-differing",
                            self.only_differing,
                            tr!("Show only those that do not match"),
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.only_differing = !this.only_differing;
                            cx.notify();
                        }))
                        .into_any_element(),
                    );
                }
            }
        }
        Some(
            div()
                .flex()
                .flex_col()
                .gap(ui(8.))
                .child(legend(tr!("Compare with")))
                .child(
                    div()
                        .px(ui(16.))
                        .py(ui(10.))
                        .rounded(ui(14.))
                        .bg(rgb(c.field))
                        .font_family(mono)
                        .text_size(ui(13.))
                        .child(self.pasted.clone()),
                )
                .children(said),
        )
    }

    fn files(&self, mono: SharedString, cx: &mut Context<Self>) -> Div {
        let c = colors();
        let a = &self.archive;
        let close = icon_button("close", "x", 40.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        let detail = match (&self.made, self.scope) {
            (Some(made), _) => {
                let bytes: u64 = made.files.iter().map(|f| f.size).sum();
                let n = made.files.len() as u64;
                format!(
                    "{} · {}",
                    tr_n!(n, "{count} file", "{count} files", count = count(n)),
                    size(bytes)
                )
            }
            (None, _) => a.name.clone(),
        };
        let progress = self.reading.as_ref().map(|r| {
            let read = r.read.load(Ordering::Relaxed);
            let total = r.total.load(Ordering::Relaxed);
            let (said, done) = if total > 0 {
                (
                    tr!(
                        "Reading… {size} of {size2}",
                        size = size(read),
                        size2 = size(total)
                    ),
                    (read as f32 / total as f32).min(1.),
                )
            } else {
                (tr!("Reading… {size}", size = size(read)), 0.)
            };
            div()
                .flex()
                .flex_col()
                .gap(ui(6.))
                .child(
                    div().h(ui(8.)).rounded_full().bg(rgb(c.field)).child(
                        div()
                            .h_full()
                            .w(relative(done))
                            .rounded_full()
                            .bg(rgb(c.accent)),
                    ),
                )
                .child(
                    div()
                        .text_size(ui(13.))
                        .text_color(rgb(c.muted))
                        .child(said),
                )
        });
        let n = self.made.as_ref().map_or(0, |m| m.files.len());
        let in_list = self.listed().len();
        let list = (in_list > 0).then(|| {
            let rows = |most: usize| in_list.min(most) as f32 * ROW + 2. * WELL_PAD;
            // Shorter, where the window is, but never less than a few rows:
            // then what is under the header scrolls.
            div()
                .relative()
                .h(ui(rows(MOST_ROWS)))
                .min_h(ui(rows(FEWEST_ROWS)))
                .p(ui(WELL_PAD))
                .rounded(ui(20.))
                .bg(rgb(c.well))
                .flex()
                .flex_col()
                .child(
                    uniform_list(
                        "checksum-files",
                        in_list,
                        cx.processor(|this, range: Range<usize>, _, cx| {
                            let listed = this.listed();
                            range
                                .filter_map(|r| listed.get(r).map(|i| this.file_row(*i, cx)))
                                .collect::<Vec<_>>()
                        }),
                    )
                    .track_scroll(&self.scroll)
                    .flex_1()
                    .min_h_0(),
                )
                .child(well_bar("checksum-files-bar", &self.scroll, WELL_PAD, 20.))
        });
        let comparing = self.compare_section(mono.clone(), cx);
        let shown = self.shown_file(mono, cx);
        // Until one is clicked, what a click does.
        let hint = (n > 1 && self.shown.is_none()).then(|| {
            div()
                .text_size(ui(13.))
                .text_color(rgb(c.muted))
                .child(tr!("Click a file to see its checksums."))
        });
        let empty = self
            .made
            .as_ref()
            .filter(|m| m.files.is_empty() && m.damaged.is_empty())
            .map(|_| {
                div()
                    .text_size(ui(13.))
                    .text_color(rgb(c.muted))
                    .child(tr!("There are no files in it to read."))
            });
        let damaged = self
            .made
            .as_ref()
            .filter(|m| !m.damaged.is_empty())
            .map(|m| {
                let n = m.damaged.len() as u64;
                let names: Vec<&str> = m.damaged.iter().take(3).map(|d| d.path.as_str()).collect();
                note(Tone::Warn)
                    .debug_selector(|| "checksum-damaged".to_owned())
                    .child(tr_n!(
                        n,
                        "{count} file is damaged, and has none: {names}",
                        "{count} files are damaged, and have none: {names}",
                        count = count(n),
                        names = names.join(", ")
                    ))
            });
        let links = self.made.as_ref().filter(|m| m.links > 0).map(|_| {
            div().text_size(ui(13.)).text_color(rgb(c.muted)).child(tr!(
                "Links have none of their own: a checking tool reads what they lead to."
            ))
        });
        let weak: Vec<_> = self
            .algorithms
            .iter()
            .copied()
            .filter(|a| a.is_weak())
            .collect();
        let weak = (self.made.is_some() && !weak.is_empty()).then(|| {
            let named: Vec<_> = weak.iter().map(|a| a.name()).collect();
            div().text_size(ui(13.)).text_color(rgb(c.muted)).child(tr!(
                "A weak checksum ({names}) still tells a damaged file, but not one \
                 changed on purpose, which can be made to match.",
                names = named.join(" / ")
            ))
        });
        let saved = (!self.saved.is_empty()).then(|| {
            note(Tone::Good)
                .debug_selector(|| "checksum-saved".to_owned())
                .child(tr!(
                    "Saved as {files}. Where they are extracted, sha256sum -c and the like \
                     check them with it, without packwing.",
                    files = self.saved.join(", ")
                ))
        });
        let read = self.made.as_ref().is_some_and(|m| !m.files.is_empty());
        let copy_all = if self.copied == Some(Copied::All) {
            tr!("Copied")
        } else {
            tr!("Copy all")
        };
        let save = if self.saving {
            tr!("Saving…")
        } else if self.beside().is_some() {
            tr!("Save beside it")
        } else {
            tr!("Save in…")
        };
        // The header and the buttons stay; what is between scrolls, when the
        // window is too short for it.
        let body = div()
            .id("checksum-body")
            .track_scroll(&self.body)
            .flex_shrink(1.)
            .min_h_0()
            .overflow_y_scroll()
            .px(ui(20.))
            .py(ui(10.))
            .flex()
            .flex_col()
            .gap(ui(20.))
            .children(progress)
            .children(comparing)
            .children(list)
            .children(hint)
            .children(shown)
            .children(empty)
            .children(self.problem.clone().map(|why| note(Tone::Warn).child(why)))
            .children(damaged)
            .children(links)
            .children(weak)
            .children(saved);
        super::card()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::on_key))
            .p(ui(0.))
            .gap(ui(0.))
            .overflow_hidden()
            .child(div().px(ui(20.)).pt(ui(22.)).pb(ui(10.)).child(header(
                "check",
                tonal_tint(),
                tr!("Checksums in {name}", name = a.name),
                detail,
                Some(close),
            )))
            .child(
                // As tall as what it holds, and shorter, scrolling, where the
                // card meets the window's height.
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .flex_shrink(1.)
                    .min_h_0()
                    .child(body)
                    .child(
                        scrollbar("checksum-body-bar", &self.body)
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .right_0(),
                    ),
            )
            .child(
                footer()
                    .px(ui(20.))
                    .pt(ui(10.))
                    .pb(ui(22.))
                    .when(read, |d| {
                        d.child(
                            button("checksum-save", save, Fill::Tonal, Size::Medium)
                                .when(self.saving, |d| d.opacity(0.5))
                                .on_click(cx.listener(|this, _, _, cx| this.save(cx))),
                        )
                        .child(
                            button("checksum-copy-all", copy_all, Fill::Tonal, Size::Medium)
                                .on_click(cx.listener(|this, _, _, cx| this.copy_all(cx))),
                        )
                    })
                    .when(read && !self.comparing, |d| {
                        d.child(
                            button(
                                "checksum-compare",
                                tr!("Compare…"),
                                Fill::Tonal,
                                Size::Medium,
                            )
                            .on_click(cx.listener(
                                |this, _, window, cx| {
                                    this.comparing = true;
                                    window.focus(&this.pasted.focus_handle(cx), cx);
                                    cx.notify();
                                },
                            )),
                        )
                    })
                    .child(settings_button(cx))
                    .child(div().flex_1())
                    .child(
                        button("checksum-done", tr!("Done"), Fill::Accent, Size::Medium)
                            .px(ui(28.))
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    ),
            )
    }
}

/// What a row shows of what the list pasted says of its file.
fn mark_view(mark: Mark, i: usize) -> Div {
    let c = colors();
    let at = div()
        .debug_selector(move || format!("checksum-mark-{i}-{}", mark.key()))
        .w(ui(20.))
        .flex_shrink_0()
        .flex()
        .justify_center()
        .text_size(ui(13.))
        .text_color(rgb(c.muted));
    match mark {
        Mark::Matches => at.child(icon("check", 16., c.accent)),
        Mark::Differs => at.child(icon("warning", 16., c.bad_text)),
        Mark::Maybe => at.child("?"),
        Mark::NotListed => at.child("—"),
    }
}

/// What is chosen to read first: what is selected, if anything is, or else the
/// archive's own file, as Checksums… read before it asked.
fn first_scope(selection: &Selected) -> Scope {
    if selection.paths.is_empty() {
        Scope::File
    } else {
        Scope::Picked
    }
}

/// An algorithm's tag on BSD's lines: `SHA256`, `BLAKE3`.
fn tag(algorithm: Algorithm) -> String {
    match algorithm {
        Algorithm::Blake3 => "BLAKE3".into(),
        a => a.key().to_ascii_uppercase(),
    }
}

/// Those Settings › Security shows, in [`Algorithm::ALL`]'s order; SHA-256 if
/// none is.
fn shown_algorithms(cx: &gpui::App) -> Vec<Algorithm> {
    let shown = prefs::prefs(cx).checksums;
    let algorithms: Vec<_> = Algorithm::ALL
        .into_iter()
        .filter(|a| shown.contains(a))
        .collect();
    if algorithms.is_empty() {
        vec![Algorithm::Sha256]
    } else {
        algorithms
    }
}

/// The button to Settings › Security, where the checksums shown are chosen.
fn settings_button(cx: &mut Context<ArchiveChecksumDialog>) -> Stateful<Div> {
    icon_button("checksum-settings", "sliders", 40.)
        .tooltip(tip(tr!("Choose the checksums shown")))
        .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::ShowChecksumSettings)))
}

impl Render for ArchiveChecksumDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mono = self.mono.get_or_insert_with(|| mono_font(window)).clone();
        match &self.stage {
            Stage::Choosing => self.choosing(cx).into_any_element(),
            Stage::File { view, .. } => view.clone().into_any_element(),
            Stage::Files => self.files(mono, cx).into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_checksum_that_differs_from_one_made_differs() {
        let differs = |made: Vec<Algorithm>, not_made: Vec<Algorithm>| {
            Mark::of(&Verdict::Differs { made, not_made })
        };
        // SHA-256 made and not it: it differs, though BLAKE3 has its length too.
        assert_eq!(
            differs(vec![Algorithm::Sha256], vec![Algorithm::Blake3]),
            Mark::Differs
        );
        assert_eq!(differs(vec![], vec![Algorithm::Sha1]), Mark::Maybe);
        assert_eq!(Mark::of(&Verdict::Unknown(3)), Mark::Differs);
        assert_eq!(Mark::of(&Verdict::NotListed), Mark::NotListed);
    }

    #[test]
    fn a_tag_is_as_bsd_s_tools_write_it() {
        assert_eq!(tag(Algorithm::Sha256), "SHA256");
        assert_eq!(tag(Algorithm::Blake3), "BLAKE3");
        let line = format!("{} (a.txt) = {}", tag(Algorithm::Sha1), "a".repeat(40));
        assert_eq!(packwing_core::checksum_list_in(&line).len(), 1);
    }

    #[test]
    fn what_is_selected_is_read_first_and_else_the_file() {
        assert_eq!(first_scope(&Selected::default()), Scope::File);
        let some = Selected {
            paths: vec![vec!["docs".into()]],
            files: 2,
            bytes: 10,
        };
        assert_eq!(first_scope(&some), Scope::Picked);
    }
}
