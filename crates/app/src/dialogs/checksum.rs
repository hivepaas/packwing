//! A file's checksums, those Settings › Security shows, kept beside it if asked, and
//! one pasted compared with them. All of the file is read once for them; one across a
//! network only when asked.

use packwing_i18n::tr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use gpui::{
    App, ClipboardItem, Context, Entity, EventEmitter, Focusable as _, FontWeight, Render,
    SharedString, Subscription, Task, Window, div, prelude::*, relative, rgb,
};
use packwing_core::{
    Algorithm, Cancel, Control, Error, Progress, Storage, algorithms_of, checksum_in, checksums,
    save_checksum,
};

use super::{DialogEvent, footer, header};
use crate::colors::colors;
use crate::input::{Changed, TextInput};
use crate::model::size;
use crate::prefs;
use crate::text_size::ui;
use crate::widgets::{
    Fill, Size, Tone, button, icon, icon_button, legend, mono_font, note, tip, tonal_tint,
};

/// How often the bytes read are shown again.
const TICK: Duration = Duration::from_millis(200);

/// How long a checksum copied says so.
const COPIED_FOR: Duration = Duration::from_secs(2);

/// A read going on. Dropped, it stops.
struct Reading {
    algorithms: Vec<Algorithm>,
    read: Arc<AtomicU64>,
    cancel: Cancel,
    _task: Task<()>,
    _ticking: Task<()>,
}

impl Drop for Reading {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

/// What the checksum pasted says of the file.
#[derive(Debug, PartialEq, Eq)]
enum Compared {
    /// Nothing in it is a checksum.
    NotOne,
    /// No algorithm makes one this many digits long.
    Unknown(usize),
    Matches(Algorithm),
    /// It is none of these made, and may be one of these not made yet.
    Differs {
        made: Vec<Algorithm>,
        not_made: Vec<Algorithm>,
    },
    /// It may be one of these, being made.
    Waiting,
}

pub struct ChecksumDialog {
    storage: Arc<dyn Storage>,
    path: Vec<String>,
    name: String,
    /// In bytes, once the storage has said.
    size: Option<u64>,
    /// Made so far, in [`Algorithm::ALL`]'s order.
    made: Vec<(Algorithm, String)>,
    reading: Option<Reading>,
    problem: Option<String>,
    /// The field for a checksum to compare is shown.
    comparing: bool,
    pasted: Entity<TextInput>,
    copied: Option<Algorithm>,
    /// They are being kept beside the file.
    saving: bool,
    /// The files they were kept in, the last time.
    saved: Vec<String>,
    mono: Option<SharedString>,
    _typing: Subscription,
}

impl EventEmitter<DialogEvent> for ChecksumDialog {}

/// Names, apart by slashes: they are not words to translate.
fn names(algorithms: &[Algorithm]) -> String {
    let named: Vec<_> = algorithms.iter().map(|a| a.name()).collect();
    named.join(" / ")
}

impl ChecksumDialog {
    /// For the file at `path`: one on this computer is read at once.
    pub fn new(storage: Arc<dyn Storage>, path: Vec<String>, cx: &mut Context<Self>) -> Self {
        let name = path.last().cloned().unwrap_or_default();
        let pasted = cx.new(|cx| {
            TextInput::said(|| tr!("Paste the checksum given with the file"), cx).multiline(2, 5)
        });
        let typing = cx.subscribe(&pasted, |_, _, _: &Changed, cx| cx.notify());
        let mut dialog = ChecksumDialog {
            storage,
            path,
            name,
            size: None,
            made: Vec::new(),
            reading: None,
            problem: None,
            comparing: false,
            pasted,
            copied: None,
            saving: false,
            saved: Vec::new(),
            mono: None,
            _typing: typing,
        };
        dialog.ask_size(cx);
        if !dialog.far() {
            dialog.start(prefs::prefs(cx).checksums, cx);
        }
        dialog
    }

    /// Reads the file now, as the screenshots draw it read.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn read_for_test(&mut self, cx: &mut Context<Self>) {
        self.reading = None;
        let shown = prefs::prefs(cx).checksums;
        self.made = checksums(
            self.storage.as_ref(),
            &self.path,
            &shown,
            &Control::default(),
        )
        .expect("the file reads");
        cx.notify();
    }

    /// Saves them beside the file now, as the screenshots draw them saved.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn save_for_test(&mut self, cx: &mut Context<Self>) {
        self.saved = self
            .made
            .iter()
            .map(|(a, sum)| save_checksum(self.storage.as_ref(), &self.path, *a, sum))
            .collect::<Result<_, _>>()
            .expect("they are saved");
        cx.notify();
    }

    /// Shows `pasted` compared, as the screenshots draw it.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn compare_for_test(&mut self, pasted: &str, cx: &mut Context<Self>) {
        self.comparing = true;
        self.pasted.update(cx, |f, cx| {
            f.set_text(pasted, pasted.len()..pasted.len(), cx)
        });
        cx.notify();
    }

    /// Asks the storage how long the file is, in the background.
    fn ask_size(&mut self, cx: &mut Context<Self>) {
        let Some((name, folder)) = self.path.split_last() else {
            return;
        };
        let (storage, folder, name) = (self.storage.clone(), folder.to_vec(), name.clone());
        let asking = cx.background_spawn(async move { storage.stat(&folder, &name) });
        cx.spawn(async move |this, cx| {
            if let Ok(Some(item)) = asking.await {
                let _ = this.update(cx, |this, cx| {
                    this.size = Some(item.size);
                    cx.notify();
                });
            }
        })
        .detach();
    }

    /// Keeps each checksum made beside the file, in a file of its own.
    fn save(&mut self, cx: &mut Context<Self>) {
        if self.saving || self.made.is_empty() {
            return;
        }
        self.saving = true;
        self.problem = None;
        let (storage, path, made) = (self.storage.clone(), self.path.clone(), self.made.clone());
        let saving = cx.background_spawn(async move {
            made.iter()
                .map(|(a, sum)| save_checksum(storage.as_ref(), &path, *a, sum))
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

    /// Whether the file is across a network, where reading it may cost.
    fn far(&self) -> bool {
        self.storage.local_path(&[]).is_none()
    }

    /// Whether it has been read, or is being read.
    fn started(&self) -> bool {
        self.reading.is_some() || !self.made.is_empty()
    }

    /// Reads all of the file for these checksums, in the background.
    fn start(&mut self, algorithms: Vec<Algorithm>, cx: &mut Context<Self>) {
        let wanted: Vec<_> = algorithms
            .into_iter()
            .filter(|a| !self.made.iter().any(|(m, _)| m == a))
            .collect();
        if wanted.is_empty() || self.reading.is_some() {
            return;
        }
        self.problem = None;
        let read = Arc::new(AtomicU64::new(0));
        let cancel = Cancel::new();
        let told = read.clone();
        let control = Control {
            progress: Some(Arc::new(move |p: &Progress| {
                told.store(p.bytes, Ordering::Relaxed);
            })),
            cancel: Some(cancel.clone()),
            on_damaged: None,
            on_unnamable: None,
            on_links_outside: None,
        };
        let (storage, path, asked) = (self.storage.clone(), self.path.clone(), wanted.clone());
        let work = cx
            .background_spawn(async move { checksums(storage.as_ref(), &path, &asked, &control) });
        let task = cx.spawn(async move |this, cx| {
            let result = work.await;
            let _ = this.update(cx, |this, cx| {
                this.reading = None;
                match result {
                    Ok(made) => {
                        this.made.extend(made);
                        this.made.sort_by_key(|(a, _)| {
                            Algorithm::ALL.iter().position(|b| b == a).unwrap_or(0)
                        });
                    }
                    Err(Error::Cancelled) => {}
                    Err(e) => {
                        this.problem = Some(tr!("The file could not be read: {why}", why = e));
                    }
                }
                cx.notify();
            });
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
            algorithms: wanted,
            read,
            cancel,
            _task: task,
            _ticking: ticking,
        });
        cx.notify();
    }

    fn copy(&mut self, algorithm: Algorithm, sum: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(sum));
        self.copied = Some(algorithm);
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(COPIED_FOR).await;
            let _ = this.update(cx, |this, cx| {
                if this.copied == Some(algorithm) {
                    this.copied = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// What the checksum pasted says, if one was.
    fn compared(&self, cx: &App) -> Option<Compared> {
        let pasted = self.pasted.read(cx).text();
        if pasted.trim().is_empty() {
            return None;
        }
        let Some(given) = checksum_in(pasted, &self.name) else {
            return Some(Compared::NotOne);
        };
        let may_be = algorithms_of(given.len());
        if may_be.is_empty() {
            return Some(Compared::Unknown(given.len()));
        }
        if let Some((a, _)) = self
            .made
            .iter()
            .find(|(a, sum)| may_be.contains(a) && *sum == given)
        {
            return Some(Compared::Matches(*a));
        }
        let reading = self.reading.as_ref().map(|r| r.algorithms.as_slice());
        if may_be
            .iter()
            .any(|a| reading.unwrap_or_default().contains(a))
        {
            return Some(Compared::Waiting);
        }
        let (made, not_made) = may_be
            .into_iter()
            .partition(|a| self.made.iter().any(|(m, _)| m == a));
        Some(Compared::Differs { made, not_made })
    }

    /// A checksum's row: its name, its digits and Copy.
    fn row(
        &self,
        algorithm: Algorithm,
        matched: bool,
        mono: SharedString,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let c = colors();
        let sum = self
            .made
            .iter()
            .find(|(a, _)| *a == algorithm)
            .map(|(_, s)| s.clone());
        let shown = match &sum {
            Some(sum) => sum.clone(),
            None if self.reading.is_some() => tr!("Reading…"),
            None => "—".to_owned(),
        };
        let copy = if self.copied == Some(algorithm) {
            tr!("Copied")
        } else {
            tr!("Copy")
        };
        div()
            .flex()
            .items_center()
            .gap(ui(12.))
            .min_h(ui(56.))
            .px(ui(16.))
            .py(ui(8.))
            .rounded(ui(16.))
            .bg(rgb(if matched { c.tonal } else { c.well }))
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
                            .when(matched, |d| d.child(icon("check", 16., c.accent)))
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
                    .debug_selector(move || format!("checksum: {}", algorithm.key()))
                    .flex_1()
                    .min_w_0()
                    .font_family(mono)
                    .text_size(ui(13.))
                    .line_height(relative(1.45))
                    .text_color(rgb(if sum.is_some() { c.text } else { c.muted }))
                    .child(shown),
            )
            .when_some(sum, |d, sum| {
                d.child(
                    button(
                        ("checksum-copy", algorithm as usize),
                        copy,
                        Fill::Text,
                        Size::Medium,
                    )
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.copy(algorithm, sum.clone(), cx)),
                    ),
                )
            })
    }

    /// The field for a checksum to compare, and what it says.
    fn compare_field(
        &self,
        mono: SharedString,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let c = colors();
        let said = self.compared(cx).map(|compared| match compared {
            Compared::NotOne => note(Tone::Warn)
                .child(tr!("There is no checksum in this."))
                .into_any_element(),
            Compared::Unknown(digits) => note(Tone::Warn)
                .child(tr!(
                    "This is not a checksum packwing makes: it has {digits} digits.",
                    digits = digits
                ))
                .into_any_element(),
            Compared::Matches(a) => note(Tone::Good)
                .debug_selector(|| "checksum-matches".to_owned())
                .flex()
                .items_center()
                .gap(ui(8.))
                .child(icon("check", 16., c.on_tonal))
                .child(tr!(
                    "It matches the file’s {name} checksum.",
                    name = a.name()
                ))
                .into_any_element(),
            Compared::Waiting => div()
                .text_size(ui(13.))
                .text_color(rgb(c.muted))
                .child(tr!("It is compared once the file is read."))
                .into_any_element(),
            Compared::Differs { made, not_made } => div()
                .flex()
                .flex_col()
                .gap(ui(8.))
                .when(!made.is_empty(), |d| {
                    d.child(
                        note(Tone::Bad)
                            .debug_selector(|| "checksum-differs".to_owned())
                            .child(tr!(
                                "It does not match the file’s {name} checksum: this is not the \
                             file it was made of, or the file changed on the way.",
                                name = names(&made)
                            )),
                    )
                })
                .when(!not_made.is_empty(), |d| {
                    let asked = not_made.clone();
                    d.child(
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
                                        this.start(asked.clone(), cx)
                                    })),
                            ),
                    )
                })
                .into_any_element(),
        });
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
            .children(said)
    }
}

impl Render for ChecksumDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let mono = self.mono.get_or_insert_with(|| mono_font(window)).clone();
        let close = icon_button("close", "x", 40.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        let shown = {
            let mut shown = prefs::prefs(cx).checksums;
            // One made for a checksum pasted shows too.
            shown.extend(self.made.iter().map(|(a, _)| *a));
            if let Some(r) = &self.reading {
                shown.extend(r.algorithms.iter().copied());
            }
            Algorithm::ALL
                .into_iter()
                .filter(|a| shown.contains(a))
                .collect::<Vec<_>>()
        };
        let matched = match self.compared(cx) {
            Some(Compared::Matches(a)) => Some(a),
            _ => None,
        };
        let rows: Vec<_> = shown
            .iter()
            .map(|a| self.row(*a, matched == Some(*a), mono.clone(), cx))
            .collect();
        let weak: Vec<_> = shown.iter().copied().filter(|a| a.is_weak()).collect();
        let far = self.far();
        let progress = self.reading.as_ref().map(|r| {
            let read = r.read.load(Ordering::Relaxed);
            let (said, done) = match self.size.filter(|s| *s > 0) {
                Some(all) => (
                    tr!(
                        "Reading… {size} of {size2}",
                        size = size(read),
                        size2 = size(all)
                    ),
                    (read as f32 / all as f32).min(1.),
                ),
                None => (tr!("Reading… {size}", size = size(read)), 0.),
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
        let comparing = self.comparing.then(|| self.compare_field(mono.clone(), cx));
        super::card()
            .child(header(
                "check",
                tonal_tint(),
                tr!("Checksums of {name}", name = self.name),
                self.size.map(size).unwrap_or_default(),
                Some(close),
            ))
            .child(div().flex().flex_col().gap(ui(6.)).children(rows))
            .children(progress)
            .children(self.problem.clone().map(|why| note(Tone::Warn).child(why)))
            .when(far && !self.started(), |d| {
                d.child(div().text_size(ui(13.)).text_color(rgb(c.muted)).child(tr!(
                    "All of the file passes through this computer, and your provider may \
                     charge for what is read from it."
                )))
            })
            .when(!weak.is_empty(), |d| {
                d.child(div().text_size(ui(13.)).text_color(rgb(c.muted)).child(tr!(
                    "A weak checksum ({names}) still tells a damaged file, but not one \
                     changed on purpose, which can be made to match.",
                    names = names(&weak)
                )))
            })
            .when(!self.saved.is_empty(), |d| {
                d.child(
                    note(Tone::Good)
                        .debug_selector(|| "checksum-saved".to_owned())
                        .child(tr!(
                            "Saved beside it as {files}, where sha256sum -c and the like check \
                             it without packwing.",
                            files = self.saved.join(", ")
                        )),
                )
            })
            .children(comparing)
            .child(
                footer()
                    .when(!self.comparing, |d| {
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
                    .when(!self.made.is_empty(), |d| {
                        let label = if self.saving {
                            tr!("Saving…")
                        } else {
                            tr!("Save beside it")
                        };
                        d.child(
                            button("checksum-save", label, Fill::Tonal, Size::Medium)
                                .when(self.saving || self.reading.is_some(), |d| d.opacity(0.5))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if this.reading.is_none() {
                                        this.save(cx);
                                    }
                                })),
                        )
                    })
                    .child(
                        icon_button("checksum-settings", "sliders", 40.)
                            .tooltip(tip(tr!("Choose the checksums shown")))
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(DialogEvent::ShowChecksumSettings)
                            })),
                    )
                    .child(div().flex_1())
                    .when(far && !self.started(), |d| {
                        d.child(
                            button("checksum-read", tr!("Read it"), Fill::Accent, Size::Medium)
                                .px(ui(28.))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.start(prefs::prefs(cx).checksums, cx)
                                })),
                        )
                    })
                    .when(!far || self.started(), |d| {
                        d.child(
                            button("checksum-done", tr!("Done"), Fill::Accent, Size::Medium)
                                .px(ui(28.))
                                .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                        )
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_apart_by_slashes() {
        assert_eq!(names(&[Algorithm::Sha256]), "SHA-256");
        assert_eq!(names(&algorithms_of(64)), "SHA-256 / BLAKE3");
    }
}
