//! Extract: where the files go, the password, and what is checked.

use packwing_i18n::{tr, tr_n};
use std::path::PathBuf;

use gpui::{
    Context, Entity, EventEmitter, FontWeight, PathPromptOptions, Render, SharedString, Window,
    div, prelude::*, rgb,
};

use super::{DialogEvent, Job, Verb, card, footer, header};
use crate::colors::colors;
use crate::input::TextInput;
use crate::model::{Archive, count, size};
use crate::prefs::ExtractTo;
use crate::text_size::ui;
use crate::widgets::{
    Fill, Size, Tone, button, chip, icon, icon_button, legend, note, option_card, segment,
    segments, titled, tonal_tint,
};

/// Where the files go.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Dest {
    /// Right here if one thing goes, else a new folder.
    Smart,
    /// A new folder beside the archive, named after it.
    NewFolder,
    /// The archive's own folder.
    Here,
    /// A folder the user picks: none picked yet.
    Folder(Option<PathBuf>),
}

/// The Downloads folder of this user, or their home folder if there is none.
fn downloads() -> Option<PathBuf> {
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)?;
    let downloads = home.join("Downloads");
    Some(if downloads.is_dir() { downloads } else { home })
}

pub struct ExtractDialog {
    archive: Archive,
    picked: Vec<Vec<String>>,
    /// Only what is picked goes, not all of it.
    only_picked: bool,
    dest: Dest,
    password: Entity<TextInput>,
    /// The password is shown as typed.
    shown: bool,
}

impl EventEmitter<DialogEvent> for ExtractDialog {}

impl ExtractDialog {
    /// `picked`: what is chosen in the archive, by path in it, which goes
    /// rather than all of it unless the user says otherwise.
    pub fn new(archive: Archive, picked: Vec<Vec<String>>, cx: &mut Context<Self>) -> Self {
        ExtractDialog {
            only_picked: !picked.is_empty(),
            picked,
            archive,
            dest: match crate::prefs::prefs(cx).extract_to {
                ExtractTo::Smart => Dest::Smart,
                ExtractTo::NewFolder => Dest::NewFolder,
                ExtractTo::Here => Dest::Here,
            },
            password: cx.new(|cx| TextInput::new(tr!("The archive’s password"), cx).masked()),
            shown: false,
        }
    }

    /// The new folder beside the archive, as the user knows the place.
    fn new_folder(&self) -> String {
        format!("{} › {}", self.archive.location, self.archive.stem())
    }

    /// What goes, by the names it will have at the top of where it goes.
    fn going(&self) -> Vec<String> {
        if self.only_picked {
            self.picked
                .iter()
                .filter_map(|path| path.last().cloned())
                .collect()
        } else {
            self.archive.top_names()
        }
    }

    /// What [`Dest::Smart`] comes to.
    fn smart(&self) -> Dest {
        if self.going().len() == 1 {
            Dest::Here
        } else {
            Dest::NewFolder
        }
    }

    /// The choice, with Smart's made.
    fn resolved(&self) -> Dest {
        match &self.dest {
            Dest::Smart => self.smart(),
            dest => dest.clone(),
        }
    }

    /// Smart's line: where the files go, and why.
    fn smart_detail(&self) -> String {
        let going = self.going();
        match going.as_slice() {
            [one] => {
                let kind = match self.archive.top_is_folder(one) {
                    true => tr!("one folder"),
                    false => tr!("one file"),
                };
                tr!(
                    "{location} › {one} · {kind}, so right here",
                    location = self.archive.location,
                    one = one,
                    kind = kind
                )
            }
            many => tr!(
                "{new_folder} · {counted}, so in a new folder",
                new_folder = self.new_folder(),
                counted = tr_n!(
                    many.len() as u64,
                    "{count} item",
                    "{count} items",
                    count = count(many.len() as u64)
                )
            ),
        }
    }

    fn destination(&self) -> SharedString {
        match &self.resolved() {
            // `resolved` leaves none.
            Dest::Smart => SharedString::default(),
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
        let a = &self.archive;
        // One on a storage has no folder here: it comes to Downloads.
        let folder = match &a.path {
            Some(path) => path.parent()?.to_path_buf(),
            None if a.source.is_some() => downloads()?,
            None => return None,
        };
        match &self.resolved() {
            Dest::Smart => None,
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
            prompt: Some(tr!("Extract here").into()),
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
        if let Some(source) = a.source.clone() {
            // A folder still to pick: nothing starts yet.
            let Some(destination) = self.destination_path() else {
                return;
            };
            job.work = vec![(source, Some(destination))];
            // What the archive was opened with; a password typed here
            // over it.
            job.keys = a.keys.clone();
            job.how = crate::prefs::prefs(cx).extract_options();
            if self.only_picked {
                job.picked = self.picked.clone();
            } else if self.dest == Dest::Smart
                && self.resolved() == Dest::Here
                && a.has_macos_files()
            {
                // What macOS left would land beside the one thing here.
                job.picked = self.going().into_iter().map(|name| vec![name]).collect();
            }
            let typed = self.password.read(cx).text();
            if !typed.is_empty() {
                job.keys.password = Some(typed.as_bytes().to_vec());
            }
        }
        cx.emit(DialogEvent::Start(job));
    }

    /// All of it, or what is chosen in it.
    fn what(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let picked = if self.picked.len() == 1 {
            self.picked[0].last().cloned().unwrap_or_default()
        } else {
            tr!("{picked} selected", picked = self.picked.len())
        };
        let all = tr!(
            "All {counted}",
            counted = tr_n!(
                self.archive.files,
                "{count} file",
                "{count} files",
                count = count(self.archive.files)
            )
        );
        div()
            .flex()
            .flex_col()
            .gap(ui(6.))
            .child(legend(tr!("What should it extract?")))
            .child(segments([
                segment("only-picked", picked, self.only_picked).on_click(cx.listener(
                    |this, _, _, cx| {
                        this.only_picked = true;
                        cx.notify();
                    },
                )),
                segment("all-of-it", all, !self.only_picked).on_click(cx.listener(
                    |this, _, _, cx| {
                        this.only_picked = false;
                        cx.notify();
                    },
                )),
            ]))
    }

    fn destinations(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let on = |d: &Dest| std::mem::discriminant(d) == std::mem::discriminant(&self.dest);
        let smart = on(&Dest::Smart);
        let recommended = option_card("dest-smart", smart)
            .on_click(cx.listener(|this, _, _, cx| {
                this.dest = Dest::Smart;
                cx.notify();
            }))
            .child(
                titled(tr!("Smart"), self.smart_detail(), smart)
                    .flex_1()
                    .min_w_0(),
            )
            .child(chip(
                tr!("Recommended"),
                c.surface,
                if smart { c.on_tonal } else { c.text },
            ));
        let title = |text: String, on: bool| {
            div()
                .text_size(ui(15.))
                .when(on, |d| d.font_weight(FontWeight::BOLD))
                .child(text)
        };
        let new_folder = on(&Dest::NewFolder);
        let here = on(&Dest::Here);
        let folder = on(&Dest::Folder(None));
        let chosen = matches!(&self.dest, Dest::Folder(Some(_)));
        div()
            .flex()
            .flex_col()
            .gap(ui(8.))
            .child(legend(tr!("Where should the files go?")))
            .child(recommended)
            .child(
                div()
                    .grid()
                    .grid_cols(3)
                    .min_w_0()
                    .gap(ui(8.))
                    .child(
                        option_card("dest-new", new_folder)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.dest = Dest::NewFolder;
                                cx.notify();
                            }))
                            .child(title(tr!("New folder"), new_folder)),
                    )
                    .child(
                        option_card("dest-here", here)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.dest = Dest::Here;
                                cx.notify();
                            }))
                            .child(title(tr!("Right here"), here)),
                    )
                    .child(
                        option_card("dest-folder", folder)
                            .on_click(cx.listener(|this, _, _, cx| this.choose_folder(cx)))
                            .child(title(
                                if chosen {
                                    tr!("Another folder")
                                } else {
                                    tr!("Choose…")
                                },
                                folder,
                            )),
                    ),
            )
            // Smart's card says where already.
            .when(!smart, |d| {
                d.child(
                    div()
                        .px(ui(4.))
                        .text_size(ui(13.))
                        .text_color(rgb(c.muted))
                        .truncate()
                        .child(tr!(
                            "Goes to {destination}",
                            destination = self.destination()
                        )),
                )
            })
    }

    fn password(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let shown = self.shown;
        div()
            .flex()
            .flex_col()
            .gap(ui(6.))
            .child(legend(tr!("Password")))
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(ui(52.))
                    .pl(ui(18.))
                    .rounded(ui(16.))
                    .bg(rgb(c.field))
                    .text_size(ui(16.))
                    .child(self.password.clone())
                    .child(
                        div()
                            .id("show-password")
                            .size(ui(52.))
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

    fn safety(&self) -> impl IntoElement {
        let c = colors();
        // What hivepack::extract does for every archive, said before it has run:
        // no path leaves the folder, no link points out of it, and what is
        // written stops at the limit. It is not a verdict on this archive.
        div()
            .flex()
            .flex_col()
            .gap(ui(8.))
            .px(ui(16.))
            .py(ui(14.))
            .rounded(ui(20.))
            .bg(rgb(c.tonal))
            .text_color(rgb(c.on_tonal))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(ui(10.))
                    .child(icon("shield", 20., c.on_tonal))
                    .child(
                        div()
                            .text_size(ui(15.))
                            .font_weight(FontWeight::BOLD)
                            .child(tr!("Checked as it goes")),
                    )
                    .child(
                        div()
                            .text_size(ui(13.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(tr!("· Nothing leaves the folder or passes the size limit")),
                    ),
            )
            .children(self.archive.marked.iter().map(|name| {
                note(Tone::Warn).child(tr!("{name} came from the Internet. It will stay marked, so you’ll be asked before it runs.", name = name))
            }))
    }
}

impl Render for ExtractDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let a = &self.archive;
        let detail = format!(
            "{} · {} · {}",
            a.name,
            tr_n!(
                a.files,
                "{count} file",
                "{count} files",
                count = count(a.files)
            ),
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
                tr!("Extract archive"),
                detail,
                Some(close),
            ))
            .when(!self.picked.is_empty(), |d| d.child(self.what(cx)))
            .child(self.destinations(cx))
            // An archive opened with its password needs no more; one whose
            // files only are encrypted is asked for it here.
            .when(a.encryption.is_some() && a.keys.password.is_none(), |d| {
                d.child(self.password(cx))
            })
            .child(self.safety())
            .child(
                footer()
                    .child(div().flex_1())
                    .child(
                        button("cancel", tr!("Cancel"), Fill::Text, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button("extract", tr!("Extract"), Fill::Accent, Size::Medium)
                            .px(ui(28.))
                            .on_click(cx.listener(|this, _, _, cx| this.start(cx))),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use crate::model::left_by_macos;

    #[test]
    fn what_macos_leaves() {
        assert!(left_by_macos("__MACOSX"));
        assert!(left_by_macos(".DS_Store"));
        assert!(left_by_macos("._report.pdf"));
        assert!(!left_by_macos("report.pdf"));
        assert!(!left_by_macos(".config"));
    }
}
