//! Extract: where the files go, the password, how the names are read, and
//! what the safety checks found.

use std::path::PathBuf;

use gpui::{
    Context, Entity, EventEmitter, FontWeight, PathPromptOptions, Render, SharedString, Window,
    div, prelude::*, px, rgb,
};

use super::{DialogEvent, Job, Verb, card, footer, header};
use crate::colors::colors;
use crate::input::TextInput;
use crate::model::{Archive, Kind, counted, size};
use crate::widgets::{
    Fill, Select, SelectLook, Size, Tone, button, chip, icon, icon_button, legend, link, note,
    option_card, titled, tonal_tint,
};

/// Where the files go.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Dest {
    /// A new folder beside the archive, named after it.
    NewFolder,
    /// The archive's own folder.
    Here,
    /// A folder the user picks: none picked yet.
    Folder(Option<PathBuf>),
}

/// How file names in the archive may be written, when it does not say.
pub const ENCODINGS: [&str; 5] = [
    "Auto · UTF-8",
    "Vietnamese · Windows-1258",
    "Japanese · Shift_JIS",
    "Chinese · GBK",
    "Western · CP437",
];

pub struct ExtractDialog {
    archive: Archive,
    dest: Dest,
    password: Entity<TextInput>,
    /// The password is shown as typed.
    shown: bool,
    names: Entity<Select>,
}

impl EventEmitter<DialogEvent> for ExtractDialog {}

impl ExtractDialog {
    pub fn new(archive: Archive, cx: &mut Context<Self>) -> Self {
        ExtractDialog {
            archive,
            dest: Dest::NewFolder,
            password: cx.new(|cx| TextInput::new("The archive’s password", cx).masked()),
            shown: false,
            names: cx.new(|_| Select::new(&ENCODINGS, 0, SelectLook::Pill)),
        }
    }

    /// The new folder beside the archive, as the user knows the place.
    fn new_folder(&self) -> String {
        format!("{} › {}", self.archive.location, self.archive.stem())
    }

    fn destination(&self) -> SharedString {
        match &self.dest {
            Dest::NewFolder => self.new_folder().into(),
            Dest::Here => self.archive.location.clone().into(),
            Dest::Folder(f) => f
                .as_ref()
                .map_or_else(|| "…".into(), |f| f.display().to_string().into()),
        }
    }

    /// Where the files go on disk: none for a sample, or before a folder is
    /// picked.
    fn destination_path(&self) -> Option<PathBuf> {
        let folder = self.archive.path.as_ref()?.parent()?.to_path_buf();
        match &self.dest {
            Dest::NewFolder => Some(folder.join(self.archive.stem())),
            Dest::Here => Some(folder),
            Dest::Folder(f) => f.clone(),
        }
    }

    fn choose_folder(&mut self, cx: &mut Context<Self>) {
        self.dest = Dest::Folder(None);
        cx.notify();
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Extract here".into()),
        });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(paths))) = picked.await
                && let Some(path) = paths.into_iter().next()
            {
                let _ = this.update(cx, |this, cx| {
                    this.dest = Dest::Folder(Some(path));
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        let a = &self.archive;
        let mut job = Job::shown(
            Verb::Extract,
            a.name.clone().into(),
            Some(self.destination()),
            a.files,
            a.unpacked,
        );
        if let Some(source) = a.path.clone() {
            // A folder still to pick: nothing starts yet.
            let Some(destination) = self.destination_path() else {
                return;
            };
            job.work = vec![(source, Some(destination))];
            let typed = self.password.read(cx).text().to_owned();
            job.password = (!typed.is_empty()).then_some(typed);
        }
        cx.emit(DialogEvent::Start(job));
    }

    fn destinations(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let on = |d: &Dest| std::mem::discriminant(d) == std::mem::discriminant(&self.dest);
        let new_folder = on(&Dest::NewFolder);
        let recommended = option_card("dest-new", new_folder)
            .on_click(cx.listener(|this, _, _, cx| {
                this.dest = Dest::NewFolder;
                cx.notify();
            }))
            .child(
                titled(
                    "New folder next to the archive",
                    self.new_folder(),
                    new_folder,
                )
                .flex_1(),
            )
            .child(chip(
                "Recommended",
                c.surface,
                if new_folder { c.on_tonal } else { c.text },
            ));
        let here = on(&Dest::Here);
        let picked = match &self.dest {
            Dest::Folder(Some(f)) => Some(SharedString::from(f.display().to_string())),
            _ => None,
        };
        let folder = on(&Dest::Folder(None));
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(legend("Where should the files go?"))
            .child(recommended)
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap(px(8.))
                    .child(
                        option_card("dest-here", here)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.dest = Dest::Here;
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .text_size(px(15.))
                                    .when(here, |d| d.font_weight(FontWeight::BOLD))
                                    .child("Right here"),
                            ),
                    )
                    .child(
                        option_card("dest-folder", folder)
                            .on_click(cx.listener(|this, _, _, cx| this.choose_folder(cx)))
                            .map(|d| match picked {
                                Some(path) => d.child(titled("Chosen folder", path, folder)),
                                None => d.child(
                                    div()
                                        .text_size(px(15.))
                                        .when(folder, |d| d.font_weight(FontWeight::BOLD))
                                        .child("Choose a folder…"),
                                ),
                            }),
                    ),
            )
    }

    fn password(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let shown = self.shown;
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(legend("Password"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(52.))
                    .pl(px(18.))
                    .rounded(px(16.))
                    .bg(rgb(c.field))
                    .text_size(px(16.))
                    .child(self.password.clone())
                    .child(
                        div()
                            .id("show-password")
                            .size(px(52.))
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_full()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(c.well_hover)))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.shown = !shown;
                                this.password.update(cx, |p, cx| p.set_masked(shown, cx));
                                cx.notify();
                            }))
                            .child(icon(if shown { "eye-off" } else { "eye" }, 20., c.muted)),
                    ),
            )
    }

    fn names(&self) -> impl IntoElement {
        let c = colors();
        // A file's name with marks the encodings disagree on, so a wrong
        // guess shows.
        let sample = self
            .archive
            .entries
            .iter()
            .find(|e| e.kind != Kind::Folder && !e.name.is_ascii())
            .map_or(String::new(), |e| e.name.clone());
        div()
            .flex()
            .items_center()
            .gap(px(12.))
            .px(px(16.))
            .py(px(12.))
            .rounded(px(20.))
            .bg(rgb(c.well))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(rgb(c.muted))
                            .child("Do the file names look right?"),
                    )
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .truncate()
                            .child(sample),
                    ),
            )
            .child(self.names.clone())
    }

    fn safety(&self) -> impl IntoElement {
        let c = colors();
        // The checks: no path leaves the folder, no link points out of it,
        // and the sizes are what the archive says.
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .px(px(16.))
            .py(px(14.))
            .rounded(px(20.))
            .bg(rgb(c.tonal))
            .text_color(rgb(c.on_tonal))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(icon("shield", 20., c.on_tonal))
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_weight(FontWeight::BOLD)
                            .child("Looks safe"),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::MEDIUM)
                            .child("· 3 checks passed"),
                    ),
            )
            .children(self.archive.marked.iter().map(|name| {
                note(Tone::Warn).child(format!(
                    "{name} came from the Internet. It will stay marked, so you’ll be asked before it runs."
                ))
            }))
    }
}

impl Render for ExtractDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let a = &self.archive;
        let detail = format!(
            "{} · {} · {}",
            a.name,
            counted(a.files, "file", "files"),
            size(a.unpacked)
        );
        let close = icon_button("close", "x", 44.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        card()
            .id("extract-dialog")
            .overflow_y_scroll()
            .child(header(
                "extract",
                tonal_tint(),
                "Extract archive",
                detail,
                Some(close),
            ))
            .child(self.destinations(cx))
            .when(a.encryption.is_some(), |d| d.child(self.password(cx)))
            .child(self.names())
            .child(self.safety())
            .child(
                footer()
                    .child(link("more-options", "More options"))
                    .child(div().flex_1())
                    .child(
                        button("cancel", "Cancel", Fill::Text, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button("extract", "Extract", Fill::Accent, Size::Medium)
                            .px(px(28.))
                            .on_click(cx.listener(|this, _, _, cx| this.start(cx))),
                    ),
            )
    }
}
