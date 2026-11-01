//! Changing what a ZIP archive holds: deleting what is selected, asked first as
//! it cannot be undone, and renaming the one selected.

use std::path::PathBuf;

use gpui::{
    Context, Entity, EventEmitter, Focusable as _, Render, Window, div, prelude::*, px, rgb,
};
use packwing_core::Keys;

use super::{DialogEvent, Editing, Job, Verb, footer, header};
use crate::colors::colors;
use crate::input::{Changed, TextInput};
use crate::widgets::{Fill, Size, Tone, button, icon_button, legend, note, tonal_tint};

pub struct EditDialog {
    archive: PathBuf,
    name: String,
    folder: Vec<String>,
    keys: Keys,
    picked: Vec<Vec<String>>,
    /// What a rename calls it: none when deleting.
    new_name: Option<Entity<TextInput>>,
    _typing: Option<gpui::Subscription>,
}

impl EventEmitter<DialogEvent> for EditDialog {}

/// Whether `name` can name an entry in its folder.
fn name_fits(name: &str) -> bool {
    !name.is_empty() && !name.contains(['/', '\\']) && name != "." && name != ".."
}

impl EditDialog {
    pub fn new(
        (archive, name, folder, keys): (PathBuf, String, Vec<String>, Keys),
        picked: Vec<Vec<String>>,
        rename: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> EditDialog {
        let old = picked
            .first()
            .and_then(|p| p.last())
            .cloned()
            .unwrap_or_default();
        let new_name = rename.then(|| {
            let field = cx.new(|cx| {
                let mut field = TextInput::new("Name", cx);
                field.set_text(&old, old.len()..old.len(), cx);
                field
            });
            window.focus(&field.focus_handle(cx), cx);
            field
        });
        let typing = new_name
            .as_ref()
            .map(|f| cx.subscribe(f, |_, _, _: &Changed, cx| cx.notify()));
        EditDialog {
            archive,
            name,
            folder,
            keys,
            picked,
            new_name,
            _typing: typing,
        }
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        let edit = match &self.new_name {
            Some(field) => {
                let typed = field.read(cx).text().trim().to_owned();
                let Some(from) = self.picked.first().cloned().filter(|_| name_fits(&typed)) else {
                    return;
                };
                Editing::Rename { from, to: typed }
            }
            None => Editing::Delete(self.picked.clone()),
        };
        let mut job = Job::shown(Verb::Edit, self.name.clone().into(), None, 0, 0);
        job.keys = self.keys.clone();
        job.edit = Some((self.archive.clone(), self.folder.clone(), edit));
        cx.emit(DialogEvent::Start(job));
    }
}

impl Render for EditDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let close = icon_button("close", "x", 44.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        let what = match self.picked.as_slice() {
            [one] => one.last().cloned().unwrap_or_default(),
            many => format!("{} items", many.len()),
        };
        let renaming = self.new_name.is_some();
        let title = if renaming {
            format!("Rename {what}")
        } else {
            format!("Delete {what}?")
        };
        let bad = self
            .new_name
            .as_ref()
            .is_some_and(|f| !name_fits(f.read(cx).text().trim()));
        super::card()
            .w(px(520.))
            .child(header(
                "logo",
                tonal_tint(),
                title,
                self.name.clone(),
                Some(close),
            ))
            .children(self.new_name.clone().map(|field| {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(legend("New name"))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .h(px(52.))
                            .px(px(18.))
                            .rounded(px(16.))
                            .bg(rgb(c.field))
                            .text_size(px(16.))
                            .child(field),
                    )
            }))
            .when(bad, |d| {
                d.child(note(Tone::Warn).child("Give it a name, without / or \\ in it."))
            })
            .when(!renaming, |d| {
                d.child(note(Tone::Warn).child(
                    "It goes from the archive for good, folders with all they hold. This can’t be undone.",
                ))
            })
            .child(
                footer()
                    .justify_end()
                    .child(
                        button("cancel", "Cancel", Fill::Text, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button(
                            "go",
                            if renaming { "Rename" } else { "Delete" },
                            Fill::Accent,
                            Size::Medium,
                        )
                        .px(px(28.))
                        .on_click(cx.listener(|this, _, _, cx| this.start(cx))),
                    ),
            )
    }
}
