//! Add files: what goes in the archive open, and the folder of it they go
//! in, the one it shows. Files and folders are asked for apart, as a system's
//! picker on Linux and Windows gives one or the other.

use packwing_i18n::{tr, tr_n};
use std::path::{Path, PathBuf};

use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, ExternalPaths, PathPromptOptions, Render,
    SharedString, Task, Window, div, prelude::*, px, rgb,
};
use packwing_core::{Keys, OnExisting};

use super::chosen::{self, Choosing};
use super::{Adding, DialogEvent, Job, Verb, footer, header};
use crate::colors::colors;
use crate::model::{Archive, Selection, count, size};
use crate::widgets::{Fill, Select, SelectLook, Size, button, icon_button, legend, tonal_tint};

/// What a file of the same name in the archive can come to.
fn on_existing() -> [(String, OnExisting); 3] {
    [
        (tr!("Stop and say so"), OnExisting::Stop),
        (tr!("Replace it"), OnExisting::Replace),
        (tr!("Skip it"), OnExisting::Skip),
    ]
}

pub struct AddDialog {
    /// The archive's file.
    archive: PathBuf,
    name: SharedString,
    /// The folder of it they go in.
    folder: Vec<String>,
    keys: Keys,
    encrypted: bool,
    selection: Selection,
    /// The list of what goes in is open.
    list_shown: bool,
    _counting: Option<Task<()>>,
    on_existing: Entity<Select>,
    /// Add was pressed with nothing chosen: a note says so.
    checked: bool,
}

impl EventEmitter<DialogEvent> for AddDialog {}

impl Choosing for AddDialog {
    fn selection(&self) -> &Selection {
        &self.selection
    }

    fn list_shown(&self) -> bool {
        self.list_shown
    }

    fn show_list(&mut self, shown: bool) {
        self.list_shown = shown;
    }

    fn pick(&mut self, folders: bool, cx: &mut Context<Self>) {
        self.pick_paths(folders, cx);
    }

    fn remove(&mut self, path: &Path, cx: &mut Context<Self>) {
        self.selection.remove(path);
        self.list_shown &= !self.selection.paths.is_empty();
        self.count_what_goes_in(cx);
    }
}

impl AddDialog {
    /// For `archive`, or `None` if it has no file: the samples.
    pub fn new(archive: &Archive, cx: &mut App) -> Option<AddDialog> {
        let on_existing =
            cx.new(|_| Select::new(&on_existing().map(|(name, _)| name), 0, SelectLook::Pill));
        Some(AddDialog {
            archive: archive.path.clone()?,
            name: archive.name.clone().into(),
            folder: archive.folder.clone(),
            keys: archive.keys.clone(),
            encrypted: archive.encryption.is_some(),
            selection: Selection::default(),
            list_shown: false,
            _counting: None,
            on_existing,
            checked: false,
        })
    }

    /// For the screenshots: the sample's name, with what they choose.
    #[cfg(all(test, feature = "shots"))]
    pub fn sample(archive: &Archive, cx: &mut App) -> AddDialog {
        let on_existing =
            cx.new(|_| Select::new(&on_existing().map(|(name, _)| name), 0, SelectLook::Pill));
        AddDialog {
            archive: PathBuf::from("/home/you/Documents/sample.zip"),
            name: archive.name.clone().into(),
            folder: vec!["Thiết kế".into(), "Hồ sơ thi công".into()],
            keys: Keys::default(),
            encrypted: true,
            selection: crate::demo::selection(),
            list_shown: false,
            _counting: None,
            on_existing,
            checked: false,
        }
    }

    /// Adds `paths` to what goes in, and counts what is in them.
    pub fn add(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        self.selection.add(paths);
        self.count_what_goes_in(cx);
    }

    fn count_what_goes_in(&mut self, cx: &mut Context<Self>) {
        let paths = self.selection.paths.clone();
        let counting = cx
            .background_executor()
            .spawn(async move { packwing_core::measure(&paths) });
        self._counting = Some(cx.spawn(async move |this, cx| {
            // One that cannot be read says so when it is added.
            let measured = counting.await.unwrap_or_default();
            let _ = this.update(cx, |this, cx| {
                this.selection.files = measured.files;
                this.selection.bytes = measured.bytes;
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn pick_paths(&mut self, folders: bool, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: !folders,
            directories: folders,
            multiple: true,
            prompt: Some(tr!("Add").into()),
        });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(paths))) = picked.await {
                let _ = this.update(cx, |this, cx| this.add(paths, cx));
            }
        })
        .detach();
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        if self.selection.paths.is_empty() {
            self.checked = true;
            cx.notify();
            return;
        }
        let mut job = Job::shown(
            Verb::Add,
            self.name.clone(),
            Some(self.goes_in().into()),
            self.selection.files,
            self.selection.bytes,
        );
        job.keys = self.keys.clone();
        job.add = Some(Adding {
            paths: self.selection.paths.clone(),
            archive: self.archive.clone(),
            folder: self.folder.clone(),
            on_existing: on_existing()[self.on_existing.read(cx).chosen().min(2)].1,
        });
        cx.emit(DialogEvent::Start(job));
    }

    /// The folder they go in, as the page shows it.
    fn goes_in(&self) -> String {
        if self.folder.is_empty() {
            tr!("The top of {name}", name = self.name)
        } else {
            self.folder.join(" › ")
        }
    }
}

impl Render for AddDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let s = &self.selection;
        let detail = if s.paths.is_empty() {
            tr!("Choose what to add").to_owned()
        } else if s.files == 0 {
            tr_n!(
                s.paths.len() as u64,
                "{count} item",
                "{count} items",
                count = count(s.paths.len() as u64)
            )
        } else {
            format!(
                "{} · {}",
                tr_n!(
                    s.files,
                    "{count} file",
                    "{count} files",
                    count = count(s.files)
                ),
                size(s.bytes)
            )
        };
        let row = |label: String, text: String| {
            div().flex_1().min_w_0().child(legend(label)).child(
                div()
                    .truncate()
                    .text_size(px(13.))
                    .text_color(rgb(c.muted))
                    .child(text),
            )
        };
        let close = icon_button("close", "x", 44.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        super::card()
            .id("add-dialog")
            .overflow_y_scroll()
            .drag_over::<ExternalPaths>(|style, _, _, _| style.bg(rgb(colors().tonal)))
            .on_drop(cx.listener(|this, dropped: &ExternalPaths, _, cx| {
                this.add(dropped.paths().to_vec(), cx)
            }))
            .child(header(
                "logo",
                tonal_tint(),
                tr!("Add to {name}", name = self.name),
                detail,
                Some(close),
            ))
            .child(chosen::section(self, self.checked, cx))
            .child(div().flex().child(row(tr!("Goes in"), self.goes_in())))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .flex_1()
                            .child(legend(tr!("If a file of that name is there"))),
                    )
                    .child(self.on_existing.clone()),
            )
            .when(self.encrypted, |d| {
                d.child(div().text_size(px(13.)).text_color(rgb(c.muted)).child(tr!(
                    "The new files are encrypted with the archive’s password."
                )))
            })
            .child(
                footer()
                    .justify_end()
                    .child(
                        button("cancel", tr!("Cancel"), Fill::Text, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button("add", tr!("Add"), Fill::Accent, Size::Medium)
                            .px(px(28.))
                            .on_click(cx.listener(|this, _, _, cx| this.start(cx))),
                    ),
            )
    }
}
