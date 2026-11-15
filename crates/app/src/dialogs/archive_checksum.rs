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
    AnyElement, ClipboardItem, Context, Div, Entity, EventEmitter, FontWeight, PathPromptOptions,
    Render, SharedString, Stateful, Subscription, Task, UniformListScrollHandle, Window, div,
    prelude::*, relative, rgb, uniform_list,
};
use packwing_core::{
    Algorithm, Cancel, Control, EntryChecksums, Error, LocalFolder, Progress, Storage,
    checksum_list, entry_checksums, save_checksum_list,
};

use super::{ChecksumDialog, DialogEvent, footer, header};
use crate::colors::colors;
use crate::model::{Archive, count, size};
use crate::prefs;
use crate::scrollbar::well_bar;
use crate::text_size::ui;
use crate::widgets::{
    Fill, Size, Tone, button, icon_button, legend, mono_font, note, option_card, tip, titled,
    tonal_tint,
};

/// How often the bytes read are shown again.
const TICK: Duration = Duration::from_millis(200);

/// How long a checksum copied says so.
const COPIED_FOR: Duration = Duration::from_secs(2);

/// A file's row in the list, as tall as the button on it.
const ROW: f32 = 48.;

/// The most rows shown at once: more scroll.
const MOST_ROWS: usize = 6;

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
    /// A file's first checksum, on its row.
    Row(usize),
    /// One of the file shown.
    Shown(Algorithm),
    /// The list of all of them.
    All,
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
    copied: Option<Copied>,
    saving: bool,
    /// The files they were kept in, the last time.
    saved: Vec<String>,
    mono: Option<SharedString>,
}

impl EventEmitter<DialogEvent> for ArchiveChecksumDialog {}

impl ArchiveChecksumDialog {
    /// For `archive`, read through, with `selection` selected in it.
    pub fn new(archive: Archive, selection: Selected, inside: bool) -> Self {
        let scope = if selection.paths.is_empty() {
            Scope::File
        } else {
            Scope::Picked
        };
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
            copied: None,
            saving: false,
            saved: Vec::new(),
            mono: None,
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
        self.algorithms = shown_algorithms(cx);
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
        self.algorithms = shown_algorithms(cx);
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
                self.shown = (made.files.len() == 1).then_some(0);
                self.made = Some(made);
            }
            Err(Error::Cancelled) => {}
            Err(e) => self.problem = Some(tr!("The file could not be read: {why}", why = e)),
        }
        cx.notify();
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

    /// The list of the first checksum of each, as `sha256sum` writes it.
    fn copy_all(&mut self, cx: &mut Context<Self>) {
        let (Some(made), Some(first)) = (&self.made, self.algorithms.first()) else {
            return;
        };
        let list = checksum_list(&made.files, *first);
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
                            .on_click(cx.listener(|this, _, _, cx| this.start(cx))),
                    ),
            )
    }

    /// A file's row: its name, its size, its first checksum and Copy.
    fn file_row(&self, i: usize, mono: SharedString, cx: &mut Context<Self>) -> AnyElement {
        let c = colors();
        let (Some(made), Some(first)) = (&self.made, self.algorithms.first().copied()) else {
            return div().into_any_element();
        };
        let Some(file) = made.files.get(i) else {
            return div().into_any_element();
        };
        let sum = file.sum(first).unwrap_or_default().to_owned();
        let on = self.shown == Some(i);
        let copy = if self.copied == Some(Copied::Row(i)) {
            tr!("Copied")
        } else {
            tr!("Copy")
        };
        div()
            .id(("checksum-file", i))
            .debug_selector(move || format!("checksum-file-{i}"))
            .w_full()
            .h(ui(ROW))
            .flex()
            .items_center()
            .gap(ui(12.))
            .pl(ui(14.))
            .rounded(ui(14.))
            .cursor_pointer()
            .when(on, |d| d.bg(rgb(c.tonal)).text_color(rgb(c.on_tonal)))
            .when(!on, |d| d.hover(|s| s.bg(rgb(c.well_hover))))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.shown = if this.shown == Some(i) { None } else { Some(i) };
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
            .child(
                div()
                    .w(ui(150.))
                    .flex_shrink_0()
                    .truncate()
                    .font_family(mono)
                    .text_size(ui(13.))
                    .child(sum.clone()),
            )
            .child(
                button(("checksum-row-copy", i), copy, Fill::Text, Size::Medium).on_click(
                    cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.copy(Copied::Row(i), sum.clone(), cx);
                    }),
                ),
            )
            .into_any_element()
    }

    /// The file shown: each of its checksums, with Copy.
    fn shown_file(&self, mono: SharedString, cx: &mut Context<Self>) -> Option<Div> {
        let c = colors();
        let file = self.made.as_ref()?.files.get(self.shown?)?;
        let rows = file.sums.iter().map(|(algorithm, sum)| {
            let (algorithm, sum) = (*algorithm, sum.clone());
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
                .bg(rgb(c.well))
                .child(
                    div()
                        .w(ui(84.))
                        .flex_shrink_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(ui(14.))
                                .font_weight(FontWeight::SEMIBOLD)
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
        Some(
            div()
                .flex()
                .flex_col()
                .gap(ui(6.))
                .child(legend(file.name.clone()))
                .children(rows),
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
        let list = (n > 0).then(|| {
            let tall = n.min(MOST_ROWS) as f32 * ROW + 2. * WELL_PAD;
            div()
                .relative()
                .h(ui(tall))
                .p(ui(WELL_PAD))
                .rounded(ui(20.))
                .bg(rgb(c.well))
                .flex()
                .flex_col()
                .child(
                    uniform_list(
                        "checksum-files",
                        n,
                        cx.processor(move |this, range: Range<usize>, _, cx| {
                            range
                                .map(|i| this.file_row(i, mono.clone(), cx))
                                .collect::<Vec<_>>()
                        }),
                    )
                    .track_scroll(&self.scroll)
                    .flex_1()
                    .min_h_0(),
                )
                .child(well_bar("checksum-files-bar", &self.scroll, WELL_PAD, 20.))
        });
        let mono = self.mono.clone().unwrap_or_default();
        let shown = self.shown_file(mono, cx);
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
        super::card()
            .child(header(
                "check",
                tonal_tint(),
                tr!("Checksums in {name}", name = a.name),
                detail,
                Some(close),
            ))
            .children(progress)
            .children(list)
            .children(shown)
            .children(empty)
            .children(self.problem.clone().map(|why| note(Tone::Warn).child(why)))
            .children(damaged)
            .children(links)
            .children(weak)
            .children(saved)
            .child(
                footer()
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
    fn what_is_selected_is_read_first_and_else_the_file() {
        let archive = crate::demo::project();
        let none = ArchiveChecksumDialog::new(archive.clone(), Selected::default(), false);
        assert_eq!(none.scope, Scope::File);
        let some = Selected {
            paths: vec![vec!["docs".into()]],
            files: 2,
            bytes: 10,
        };
        let picked = ArchiveChecksumDialog::new(archive, some, false);
        assert_eq!(picked.scope, Scope::Picked);
        assert_eq!(picked.picked(), [vec!["docs".to_owned()]]);
    }
}
