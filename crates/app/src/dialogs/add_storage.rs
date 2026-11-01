//! Adding a storage, where this build has more kinds than a folder on this
//! computer: which kind, what it needs, and a test of whether it answers.

use std::sync::Arc;

use gpui::{
    Context, Entity, EventEmitter, Focusable as _, FontWeight, Render, Task, Window, div,
    prelude::*, px, rgb,
};
use packwing_core::{Field, StorageKind, Values};
use packwing_i18n::tr;

use super::{DialogEvent, footer, header};
use crate::Extensions;
use crate::colors::colors;
use crate::explorer::LOCAL_TAG;
use crate::input::TextInput;
use crate::pages::storages::StoragesPage;
use crate::widgets::{Fill, Size, Tone, button, icon_button, note, tile, tonal_tint};

/// How a test of the settings went.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Tested {
    Not,
    Running,
    Worked,
    Failed(String),
}

pub struct AddStorageDialog {
    page: Entity<StoragesPage>,
    kinds: Vec<Arc<dyn StorageKind>>,
    /// The kind chosen, by its index; none for a folder on this computer.
    chosen: Option<usize>,
    name: Entity<TextInput>,
    fields: Vec<(Field, Entity<TextInput>)>,
    tested: Tested,
    /// Why it could not be added.
    problem: Option<String>,
    _testing: Option<Task<()>>,
}

impl EventEmitter<DialogEvent> for AddStorageDialog {}

impl AddStorageDialog {
    pub fn new(page: Entity<StoragesPage>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let kinds = cx
            .try_global::<Extensions>()
            .map(|e| e.storage_kinds.clone())
            .unwrap_or_default();
        let name = cx.new(|cx| TextInput::said(|| tr!("Name"), cx));
        window.focus(&name.focus_handle(cx), cx);
        let mut dialog = AddStorageDialog {
            page,
            kinds,
            chosen: None,
            name,
            fields: Vec::new(),
            tested: Tested::Not,
            problem: None,
            _testing: None,
        };
        dialog.choose(Some(0), cx);
        dialog
    }

    fn choose(&mut self, kind: Option<usize>, cx: &mut Context<Self>) {
        self.chosen = kind.filter(|&k| k < self.kinds.len());
        self.tested = Tested::Not;
        self.problem = None;
        self.fields = self
            .chosen
            .and_then(|k| self.kinds.get(k))
            .map(|k| k.fields())
            .unwrap_or_default()
            .into_iter()
            .map(|field| {
                let (hint, secret) = (field.hint.clone(), field.secret);
                let input = cx.new(|cx| {
                    let input = TextInput::new(hint, cx);
                    if secret { input.masked() } else { input }
                });
                (field, input)
            })
            .collect();
        cx.notify();
    }

    /// What is typed: the settings, and the secrets.
    fn typed(&self, cx: &Context<Self>) -> (Values, Values) {
        let (mut values, mut secrets) = (Values::new(), Values::new());
        for (field, input) in &self.fields {
            let text = input.read(cx).text().trim().to_owned();
            if field.secret {
                secrets.insert(field.key.to_owned(), text);
            } else {
                values.insert(field.key.to_owned(), text);
            }
        }
        (values, secrets)
    }

    /// Lists its top: whether it answers, with these settings and secrets.
    fn test(&mut self, cx: &mut Context<Self>) {
        let Some(kind) = self.chosen.and_then(|k| self.kinds.get(k)).cloned() else {
            return;
        };
        let (values, secrets) = self.typed(cx);
        self.tested = Tested::Running;
        let work = cx.background_spawn(async move {
            let storage = kind.open("", &values, Some(&secrets));
            storage.list(&[])?.next(1).map(|_| ())
        });
        self._testing = Some(cx.spawn(async move |this, cx| {
            let result: std::io::Result<()> = work.await;
            let _ = this.update(cx, |this, cx| {
                this.tested = match result {
                    Ok(()) => Tested::Worked,
                    Err(e) => Tested::Failed(e.to_string()),
                };
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn add(&mut self, cx: &mut Context<Self>) {
        let Some(kind) = self.chosen.and_then(|k| self.kinds.get(k)).cloned() else {
            return;
        };
        let (values, secrets) = self.typed(cx);
        let typed = self.name.read(cx).text().trim().to_owned();
        let name = if typed.is_empty() {
            kind.describe(&values)
        } else {
            typed
        };
        let added = self.page.update(cx, |p, cx| {
            p.add_other(kind.id(), name, values, &secrets, cx)
        });
        match added {
            Ok(()) => cx.emit(DialogEvent::Close),
            Err(e) => {
                self.problem = Some(tr!("Its secrets could not be kept: {why}", why = e));
                cx.notify();
            }
        }
    }
}

/// A field with its label over it.
fn labelled(label: String, input: Entity<TextInput>) -> gpui::Div {
    let c = colors();
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .text_size(px(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(c.muted))
        .child(label)
        .child(
            div()
                .h(px(44.))
                .flex()
                .items_center()
                .px(px(16.))
                .rounded(px(16.))
                .bg(rgb(c.field))
                .text_size(px(15.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(c.text))
                .child(input),
        )
}

impl Render for AddStorageDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let close = icon_button("close", "x", 44.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        let card = |id: (&'static str, usize), tag: &'static str, name: String, on: bool| {
            div()
                .id(id)
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(8.))
                .p(px(14.))
                .rounded(px(20.))
                .cursor_pointer()
                .bg(rgb(if on { c.tonal } else { c.well }))
                .text_color(rgb(if on { c.on_tonal } else { c.text }))
                .when(!on, |d| d.hover(|s| s.bg(rgb(c.well_hover))))
                .child(
                    tile(40., 14., tonal_tint())
                        .bg(rgb(c.surface))
                        .text_size(px(11.))
                        .font_weight(FontWeight::BOLD)
                        .child(tag),
                )
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(name),
                )
        };
        let kinds: Vec<_> = self
            .kinds
            .iter()
            .enumerate()
            .map(|(i, k)| {
                card(("kind", i), k.tag(), k.name(), self.chosen == Some(i))
                    .on_click(cx.listener(move |this, _, _, cx| this.choose(Some(i), cx)))
            })
            .collect();
        let page = self.page.clone();
        let local = card(("kind", usize::MAX), LOCAL_TAG, tr!("Local folder"), false).on_click(
            cx.listener(move |_, _, _, cx| {
                page.update(cx, |p, cx| p.pick_folder(cx));
                cx.emit(DialogEvent::Close);
            }),
        );
        let fields = self.fields.iter().map(|(field, input)| {
            div()
                .w(px(300.))
                .flex_grow(1.)
                .child(labelled(field.label.clone(), input.clone()))
        });
        let tested = match &self.tested {
            Tested::Not => None,
            Tested::Running => Some(tr!("Trying it…")),
            Tested::Worked => Some(tr!("It answers.")),
            Tested::Failed(why) => Some(tr!("It did not answer: {why}", why = why)),
        };
        super::card()
            .w(px(720.))
            .child(header(
                "storages",
                tonal_tint(),
                tr!("Add a storage"),
                tr!("Secrets go in the system’s keychain, not with the settings."),
                Some(close),
            ))
            .child(div().flex().gap(px(10.)).children(kinds).child(local))
            .child(
                div()
                    .id("storage-fields")
                    .min_h_0()
                    .overflow_y_scroll()
                    .flex()
                    .flex_wrap()
                    .gap(px(14.))
                    .child(
                        div()
                            .w(px(300.))
                            .flex_grow(1.)
                            .child(labelled(tr!("Name"), self.name.clone())),
                    )
                    .children(fields),
            )
            .children(tested.map(|said| {
                div()
                    .text_size(px(13.))
                    .text_color(rgb(if matches!(self.tested, Tested::Failed(_)) {
                        c.bad_text
                    } else {
                        c.muted
                    }))
                    .child(said)
            }))
            .children(self.problem.clone().map(|why| note(Tone::Warn).child(why)))
            .child(
                footer()
                    .child(
                        button(
                            "storage-test",
                            tr!("Test connection"),
                            Fill::Tonal,
                            Size::Medium,
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.test(cx))),
                    )
                    .child(div().flex_1())
                    .child(
                        button("cancel", tr!("Cancel"), Fill::Outline, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button("storage-add", tr!("Add"), Fill::Accent, Size::Medium)
                            .on_click(cx.listener(|this, _, _, cx| this.add(cx))),
                    ),
            )
    }
}
