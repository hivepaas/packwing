//! Add files: what goes in the archive open, and the folder of it they go
//! in, the one it shows. Files and folders are asked for apart, as a system's
//! picker on Linux and Windows gives one or the other.

use std::path::PathBuf;

use gpui::{
    Context, EventEmitter, PathPromptOptions, Render, SharedString, Task, Window, div, prelude::*,
    px, rgb,
};
use packwing_core::Keys;

use super::{Adding, DialogEvent, Job, Verb, footer, header};
use crate::colors::colors;
use crate::model::{Archive, Selection, counted, size};
use crate::widgets::{Fill, Size, Tone, button, icon_button, legend, note, tonal_tint};

pub struct AddDialog {
    /// The archive's file.
    archive: PathBuf,
    name: SharedString,
    /// The folder of it they go in.
    folder: Vec<String>,
    keys: Keys,
    encrypted: bool,
    selection: Selection,
    _counting: Option<Task<()>>,
    /// Add was pressed with nothing chosen: a note says so.
    checked: bool,
}

impl EventEmitter<DialogEvent> for AddDialog {}

impl AddDialog {
    /// For `archive`, or `None` if it has no file: the samples.
    pub fn new(archive: &Archive) -> Option<AddDialog> {
        Some(AddDialog {
            archive: archive.path.clone()?,
            name: archive.name.clone().into(),
            folder: archive.folder.clone(),
            keys: archive.keys.clone(),
            encrypted: archive.encryption.is_some(),
            selection: Selection::default(),
            _counting: None,
            checked: false,
        })
    }

    /// For the screenshots: the sample's name, with what they choose.
    #[cfg(all(test, feature = "shots"))]
    pub fn sample(archive: &Archive) -> AddDialog {
        AddDialog {
            archive: PathBuf::from("/home/you/Documents/sample.zip"),
            name: archive.name.clone().into(),
            folder: vec!["Thiết kế".into(), "Hồ sơ thi công".into()],
            keys: Keys::default(),
            encrypted: true,
            selection: crate::demo::selection(),
            _counting: None,
            checked: false,
        }
    }

    /// Adds `paths` to what goes in, and counts what is in them.
    pub fn add(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        for path in paths {
            if !self.selection.paths.contains(&path) {
                self.selection.paths.push(path);
            }
        }
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

    fn pick(&mut self, folders: bool, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: !folders,
            directories: folders,
            multiple: true,
            prompt: Some("Add".into()),
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
        });
        cx.emit(DialogEvent::Start(job));
    }

    /// The folder they go in, as the page shows it.
    fn goes_in(&self) -> String {
        if self.folder.is_empty() {
            format!("The top of {}", self.name)
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
            "Choose what to add".to_owned()
        } else if s.files == 0 {
            counted(s.paths.len() as u64, "item", "items")
        } else {
            format!("{} · {}", counted(s.files, "file", "files"), size(s.bytes))
        };
        let names: Vec<String> = s
            .paths
            .iter()
            .take(3)
            .filter_map(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .collect();
        let more = s.paths.len().saturating_sub(names.len());
        let line = match (names.is_empty(), more) {
            (true, _) => "Nothing chosen yet".to_owned(),
            (false, 0) => names.join(", "),
            (false, more) => format!("{} and {more} more", names.join(", ")),
        };
        let row = |label: &'static str, text: String| {
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
            .child(header(
                "logo",
                tonal_tint(),
                format!("Add to {}", self.name),
                detail,
                Some(close),
            ))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(row("What goes in", line))
                    .child(
                        button("add-files", "Add files…", Fill::Outline, Size::Small)
                            .on_click(cx.listener(|this, _, _, cx| this.pick(false, cx))),
                    )
                    .child(
                        button("add-folder", "Add a folder…", Fill::Outline, Size::Small)
                            .on_click(cx.listener(|this, _, _, cx| this.pick(true, cx))),
                    ),
            )
            .when(s.paths.is_empty() && self.checked, |d| {
                d.child(note(Tone::Warn).child("Add the files or a folder to put in the archive."))
            })
            .child(div().flex().child(row("Goes in", self.goes_in())))
            .when(self.encrypted, |d| {
                d.child(
                    div()
                        .text_size(px(13.))
                        .text_color(rgb(c.muted))
                        .child("The new files are encrypted with the archive’s password."),
                )
            })
            .child(
                footer()
                    .justify_end()
                    .child(
                        button("cancel", "Cancel", Fill::Text, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button("add", "Add", Fill::Accent, Size::Medium)
                            .px(px(28.))
                            .on_click(cx.listener(|this, _, _, cx| this.start(cx))),
                    ),
            )
    }
}
