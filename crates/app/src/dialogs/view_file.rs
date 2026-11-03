//! A file of a storage shown: a picture or the start of a text, read in the
//! background. What cannot be shown is offered to the app the system has for it,
//! unless that could run it.

use std::sync::Arc;

use gpui::{
    Context, Entity, EventEmitter, Image, ObjectFit, Render, StyledImage, Task, Window, div, img,
    prelude::*, rgb,
};
use packwing_core::{Item, image_fits};
use packwing_i18n::tr;

use super::{DialogEvent, footer, header};
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

pub struct ViewDialog {
    item: Item,
    explorer: Entity<Explorer>,
    shown: Shown,
    /// The text, when it is one: it can be chosen and copied.
    text: Entity<TextInput>,
    _reading: Option<Task<()>>,
}

impl EventEmitter<DialogEvent> for ViewDialog {}

impl ViewDialog {
    pub fn new(item: Item, explorer: Entity<Explorer>, cx: &mut Context<Self>) -> ViewDialog {
        let format = picture_format(&item.name);
        let mut dialog = ViewDialog {
            item: item.clone(),
            explorer: explorer.clone(),
            shown: Shown::Reading,
            text: cx.new(|cx| TextInput::new("", cx).read_only()),
            _reading: None,
        };
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
                Ok(bytes) => match format {
                    Some(format) if image_fits(&bytes) => {
                        Shown::Picture(Arc::new(Image::from_bytes(format, bytes)))
                    }
                    Some(_) => Shown::Not(tr!("Too big to show here")),
                    None => text_preview(&bytes, MOST_TEXT).map_or(
                        Shown::Not(tr!("packwing can’t show this kind of file")),
                        Shown::Text,
                    ),
                },
                Err(e) => Shown::Not(tr!("It could not be read: {why}", why = e)),
            }
        });
        dialog._reading = Some(cx.spawn(async move |this, cx| {
            let shown = reading.await;
            let _ = this.update(cx, |this, cx| {
                if let Shown::Text(text) = &shown {
                    this.text.update(cx, |t, cx| t.set_text(text, 0..0, cx));
                }
                this.shown = shown;
                cx.notify();
            });
        }));
        dialog
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
        let offer = !runs_when_opened(&self.item);
        let (edit, app) = (self.item.clone(), self.item.clone());
        super::card()
            .w(ui(820.))
            .child(header(
                "image",
                tonal_tint(),
                self.item.name.clone(),
                size(self.item.size),
                Some(close),
            ))
            .child(content)
            .child(
                footer()
                    .justify_end()
                    .child(
                        button(
                            "view-edit",
                            Explorer::edit_label(cx),
                            Fill::Tonal,
                            Size::Medium,
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.explorer.update(cx, |e, cx| e.edit(&edit, cx));
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
                                    this.explorer
                                        .update(cx, |e, cx| e.open_with_system(&app, cx));
                                    cx.emit(DialogEvent::Close);
                                },
                            )),
                        )
                    }),
            )
    }
}
