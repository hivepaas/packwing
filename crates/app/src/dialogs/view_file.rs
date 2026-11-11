//! A file of a storage, or of the archive open, shown: a picture or the start of
//! a text, read in the background. What cannot be shown is offered to the app
//! the system has for it (a file of a storage unless that could run it), and to
//! packwing as an archive.

use std::sync::Arc;

use gpui::{
    Context, Entity, EventEmitter, Image, ImageFormat, ObjectFit, Render, StyledImage, Task,
    Window, div, img, prelude::*, rgb,
};
use packwing_core::{Control, Error, Item, Keys, Source, image_fits, preview_entry};
use packwing_i18n::tr;

use super::{DialogEvent, EntryOpen, footer, header};
use crate::colors::colors;
use crate::explorer::{Explorer, runs_when_opened};
use crate::input::TextInput;
use crate::model::{Kind, size, text_preview};
use crate::pages::archive::{MOST_PREVIEWED, MOST_TEXT, picture_format};
use crate::text_size::ui;
use crate::widgets::{Fill, Size, button, icon_button, tonal_tint};

enum Shown {
    Reading,
    Picture(Arc<Image>),
    Text(String),
    /// Why it cannot be shown.
    Not(String),
}

/// The file shown.
enum Of {
    /// An item of the storage an explorer shows.
    Stored {
        item: Item,
        explorer: Entity<Explorer>,
    },
    /// A file of the archive open, by its path in it.
    Entry { path: Vec<String>, kind: Kind },
}

pub struct ViewDialog {
    of: Of,
    name: String,
    size: u64,
    shown: Shown,
    /// The text, when it is one: it can be chosen and copied.
    text: Entity<TextInput>,
    _reading: Option<Task<()>>,
}

impl EventEmitter<DialogEvent> for ViewDialog {}

/// What `bytes` of a file of this picture `format`, or none, show.
fn shown_of(bytes: Vec<u8>, format: Option<ImageFormat>) -> Shown {
    match format {
        Some(format) if image_fits(&bytes) => {
            Shown::Picture(Arc::new(Image::from_bytes(format, bytes)))
        }
        Some(_) => Shown::Not(tr!("Too big to show here")),
        None => text_preview(&bytes, MOST_TEXT).map_or(
            Shown::Not(tr!("packwing can’t show this kind of file")),
            Shown::Text,
        ),
    }
}

impl ViewDialog {
    fn new_of(of: Of, name: String, size: u64, cx: &mut Context<Self>) -> ViewDialog {
        ViewDialog {
            of,
            name,
            size,
            shown: Shown::Reading,
            text: cx.new(|cx| TextInput::new("", cx).read_only()),
            _reading: None,
        }
    }

    pub fn new(item: Item, explorer: Entity<Explorer>, cx: &mut Context<Self>) -> ViewDialog {
        let format = picture_format(&item.name);
        let (name, size) = (item.name.clone(), item.size);
        let mut dialog = ViewDialog::new_of(
            Of::Stored {
                item: item.clone(),
                explorer: explorer.clone(),
            },
            name,
            size,
            cx,
        );
        if item.size > MOST_PREVIEWED {
            dialog.shown = Shown::Not(tr!("Too big to show here"));
            return dialog;
        }
        if Kind::of(&item.name, false) == Kind::Image && format.is_none() {
            dialog.shown = Shown::Not(tr!("packwing can’t show this kind of file"));
            return dialog;
        }
        let explorer = explorer.read(cx);
        let Some(place) = explorer.place().cloned() else {
            dialog.shown = Shown::Not(tr!("It could not be read"));
            return dialog;
        };
        let path = explorer.path_of_name(&item.name);
        let reading = cx.background_spawn(async move {
            match place.storage.read(&path, MOST_PREVIEWED) {
                Ok(bytes) => shown_of(bytes, format),
                Err(e) => Shown::Not(tr!("It could not be read: {why}", why = e)),
            }
        });
        dialog.read(reading, cx);
        dialog
    }

    /// The file at `path` of the archive at `source`, opened with `keys`: a
    /// picture, or a text, as the archive's preview shows them; any other
    /// file is offered to the system's app, or to packwing as an archive.
    pub fn of_entry(
        source: Source,
        keys: Keys,
        path: Vec<String>,
        size: u64,
        cx: &mut Context<Self>,
    ) -> ViewDialog {
        let name = path.last().cloned().unwrap_or_default();
        let kind = Kind::of(&name, false);
        let format = picture_format(&name);
        let mut dialog = ViewDialog::new_of(
            Of::Entry {
                path: path.clone(),
                kind,
            },
            name,
            size,
            cx,
        );
        // A script is a text; a program that is not says it is not.
        if format.is_none() && !matches!(kind, Kind::Text | Kind::Other | Kind::Program) {
            dialog.shown = Shown::Not(tr!("packwing can’t show this kind of file"));
            return dialog;
        }
        if size > MOST_PREVIEWED {
            dialog.shown = Shown::Not(tr!("Too big to show here"));
            return dialog;
        }
        let reading = cx.background_spawn(async move {
            let control = Control {
                progress: None,
                cancel: None,
                on_damaged: None,
                on_unnamable: None,
                on_links_outside: None,
                on_gone: None,
            };
            match preview_entry(&source, &path, &keys, MOST_PREVIEWED, &control) {
                Ok(bytes) => shown_of(bytes, format),
                Err(Error::Locked { .. } | Error::WrongPassword) => {
                    Shown::Not(tr!("It needs the password to show it"))
                }
                Err(e) => Shown::Not(tr!("It could not be read: {why}", why = e)),
            }
        });
        dialog.read(reading, cx);
        dialog
    }

    /// Shows what `reading` gives, once it does.
    fn read(&mut self, reading: Task<Shown>, cx: &mut Context<Self>) {
        self._reading = Some(cx.spawn(async move |this, cx| {
            let shown = reading.await;
            let _ = this.update(cx, |this, cx| {
                if let Shown::Text(text) = &shown {
                    this.text.update(cx, |t, cx| t.set_text(text, 0..0, cx));
                }
                this.shown = shown;
                cx.notify();
            });
        }));
    }
}

impl Render for ViewDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let close = icon_button("close", "x", 40.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        let content = div()
            .id("view-content")
            .h(ui(420.))
            .flex_shrink_0()
            .rounded(ui(20.))
            .bg(rgb(c.field))
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .text_size(ui(14.))
            .text_color(rgb(c.muted))
            .map(|d| match &self.shown {
                Shown::Reading => d.child(tr!("Reading…")),
                Shown::Picture(picture) => d.child(
                    img(picture.clone())
                        .size_full()
                        .object_fit(ObjectFit::Contain)
                        .with_fallback(move || {
                            div()
                                .text_size(ui(13.))
                                .text_color(rgb(c.muted))
                                .child(tr!("It could not be shown"))
                                .into_any_element()
                        }),
                ),
                Shown::Text(_) => d.items_start().justify_start().child(
                    div()
                        .size_full()
                        .flex()
                        .p(ui(16.))
                        .text_size(ui(13.))
                        .text_color(rgb(c.text))
                        .child(self.text.clone()),
                ),
                Shown::Not(why) => d.child(why.clone()),
            });
        // What could not be shown may be an archive by another name: a
        // package, a document, a book.
        let not_shown = matches!(self.shown, Shown::Not(_));
        let buttons = match &self.of {
            Of::Stored { item, .. } => {
                let offer = !runs_when_opened(item);
                let (edit, app, inside) = (item.clone(), item.clone(), item.clone());
                footer()
                    .justify_end()
                    .when(not_shown && !item.dir, |d| {
                        d.child(
                            button(
                                "view-as-archive",
                                tr!("Open as archive"),
                                Fill::Text,
                                Size::Medium,
                            )
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    if let Of::Stored { explorer, .. } = &this.of {
                                        explorer.update(cx, |e, cx| e.open_as_archive(&inside, cx));
                                    }
                                    cx.emit(DialogEvent::Close);
                                },
                            )),
                        )
                    })
                    .child(
                        button(
                            "view-edit",
                            Explorer::edit_label(cx),
                            Fill::Tonal,
                            Size::Medium,
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Of::Stored { explorer, .. } = &this.of {
                                explorer.update(cx, |e, cx| e.edit(&edit, cx));
                            }
                            cx.emit(DialogEvent::Close);
                        })),
                    )
                    .when(offer, |d| {
                        d.child(
                            button(
                                "view-app",
                                tr!("Open with default app"),
                                Fill::Outline,
                                Size::Medium,
                            )
                            .on_click(cx.listener(
                                move |this, _, _, cx| {
                                    if let Of::Stored { explorer, .. } = &this.of {
                                        explorer.update(cx, |e, cx| e.open_with_system(&app, cx));
                                    }
                                    cx.emit(DialogEvent::Close);
                                },
                            )),
                        )
                    })
            }
            Of::Entry { path, kind } => {
                let (app, inside) = (path.clone(), path.clone());
                footer()
                    .justify_end()
                    .when(not_shown && *kind != Kind::Archive, |d| {
                        d.child(
                            button(
                                "view-as-archive",
                                tr!("Open as archive"),
                                Fill::Text,
                                Size::Medium,
                            )
                            .on_click(cx.listener(
                                move |_, _, _, cx| {
                                    cx.emit(DialogEvent::Entry(
                                        inside.clone(),
                                        EntryOpen::AsArchive,
                                    ))
                                },
                            )),
                        )
                    })
                    // A program is offered too, from an archive: what opens it
                    // says first that it runs.
                    .child(
                        button(
                            "view-app",
                            tr!("Open with default app"),
                            if not_shown {
                                Fill::Accent
                            } else {
                                Fill::Outline
                            },
                            Size::Medium,
                        )
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.emit(DialogEvent::Entry(app.clone(), EntryOpen::WithApp))
                        })),
                    )
            }
        };
        super::card()
            .w(ui(820.))
            .child(header(
                "image",
                tonal_tint(),
                self.name.clone(),
                size(self.size),
                Some(close),
            ))
            .child(content)
            .child(buttons)
    }
}
