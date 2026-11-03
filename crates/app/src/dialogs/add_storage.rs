//! Adding a storage, where this build has more kinds than a folder on this
//! computer: which kind, what it needs, and a test of whether it answers; and
//! editing one, of any kind.

use std::path::PathBuf;
use std::sync::Arc;

use gpui::{
    App, Context, Entity, EventEmitter, Focusable as _, FontWeight, Render, Task, Window, div,
    prelude::*, rgb,
};
use packwing_core::{Field, StorageKind, Values};
use packwing_i18n::tr;

use super::{DialogEvent, footer, header};
use crate::Extensions;
use crate::colors::colors;
use crate::confirm::confirmer;
use crate::explorer::LOCAL_TAG;
use crate::input::TextInput;
use crate::pages::storages::StoragesPage;
use crate::prefs::{ASKS_FIRST, KeptPlace};
use crate::text_size::ui;
use crate::widgets::{Fill, Size, Tone, button, check_row, icon_button, note, tile, tonal_tint};

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
    /// It asks that it is the user before it opens.
    asks_first: bool,
    /// The storage being edited, by its index in the list.
    editing: Option<usize>,
    /// The folder, when a local folder is being edited.
    folder: Option<Entity<TextInput>>,
    /// Its kept secrets could not be read: saving would lose them.
    secrets_unread: bool,
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
            asks_first: false,
            editing: None,
            folder: None,
            secrets_unread: false,
            tested: Tested::Not,
            problem: None,
            _testing: None,
        };
        dialog.choose(Some(0), cx);
        dialog
    }

    /// Edits the storage at `at`, with what it has now in the fields.
    pub fn editing(
        page: Entity<StoragesPage>,
        at: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut dialog = Self::new(page.clone(), window, cx);
        dialog.editing = Some(at);
        dialog.fields.clear();
        dialog.chosen = None;
        let Some(storage) = page.read(cx).storage(at).cloned() else {
            return dialog;
        };
        let name = storage.name.clone();
        let chars = name.len();
        dialog
            .name
            .update(cx, |n, cx| n.set_text(&name, chars..chars, cx));
        dialog.asks_first = storage.asks_first();
        match storage.place {
            KeptPlace::Local(path) => {
                let path = path.display().to_string();
                dialog.folder = Some(cx.new(|cx| {
                    let mut input = TextInput::said(|| tr!("Folder"), cx);
                    input.set_text(&path, 0..0, cx);
                    input
                }));
            }
            KeptPlace::Other { kind, id, values } => {
                let at = dialog.kinds.iter().position(|k| k.id() == kind);
                dialog.choose(at, cx);
                let kept = dialog
                    .chosen
                    .and_then(|k| dialog.kinds.get(k))
                    .map(|k| k.kept_secrets(&id));
                let secrets = match kept {
                    Some(Ok(secrets)) => secrets,
                    Some(Err(e)) => {
                        dialog.secrets_unread = true;
                        dialog.problem = Some(tr!("Its secrets could not be read: {why}", why = e));
                        Values::new()
                    }
                    None => Values::new(),
                };
                for (field, input) in &dialog.fields {
                    let from = if field.secret { &secrets } else { &values };
                    if let Some(text) = from.get(field.key) {
                        input.update(cx, |i, cx| i.set_text(text, 0..0, cx));
                    }
                }
            }
        }
        dialog
    }

    /// Whether the plan has no room for what is being added. One already there can be
    /// changed whatever the plan, and is never taken away.
    fn over_the_limit(&self, cx: &App) -> bool {
        self.editing.is_none()
            && self.chosen.is_some()
            && !crate::licensing::plan(cx).may_add_remote_storage(self.page.read(cx).remote_count())
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
        if self.asks_first {
            values.insert(ASKS_FIRST.to_owned(), "yes".to_owned());
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

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn field_text(&self, key: &str, cx: &gpui::App) -> String {
        self.fields
            .iter()
            .find(|(f, _)| f.key == key)
            .map(|(_, i)| i.read(cx).text().to_owned())
            .unwrap_or_default()
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn set_field(&mut self, key: &str, text: &str, cx: &mut Context<Self>) {
        if let Some((_, input)) = self.fields.iter().find(|(f, _)| f.key == key) {
            input.update(cx, |i, cx| i.set_text(text, 0..0, cx));
        }
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn set_name(&mut self, text: &str, cx: &mut Context<Self>) {
        self.name.update(cx, |i, cx| i.set_text(text, 0..0, cx));
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn set_folder(&mut self, text: &str, cx: &mut Context<Self>) {
        if let Some(folder) = &self.folder {
            folder.update(cx, |i, cx| i.set_text(text, 0..0, cx));
        }
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn press_asks_first(&mut self, cx: &mut Context<Self>) {
        self.asks_first = !self.asks_first;
        cx.notify();
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn press_save(&mut self, cx: &mut Context<Self>) {
        self.add(cx);
    }

    /// Saves the local folder being edited, if it is there.
    fn save_local(&mut self, at: usize, folder: Entity<TextInput>, cx: &mut Context<Self>) {
        let path = PathBuf::from(folder.read(cx).text().trim());
        if !path.is_dir() {
            self.problem = Some(tr!("That folder is not on this computer."));
            cx.notify();
            return;
        }
        let typed = self.name.read(cx).text().trim().to_owned();
        let name = if typed.is_empty() {
            path.display().to_string()
        } else {
            typed
        };
        self.page
            .update(cx, |p, cx| p.change_local(at, name, path, cx));
        cx.emit(DialogEvent::Close);
    }

    fn add(&mut self, cx: &mut Context<Self>) {
        if let (Some(at), Some(folder)) = (self.editing, self.folder.clone()) {
            self.save_local(at, folder, cx);
            return;
        }
        let Some(kind) = self.chosen.and_then(|k| self.kinds.get(k)).cloned() else {
            return;
        };
        if self.secrets_unread || self.over_the_limit(cx) {
            return;
        }
        let (values, secrets) = self.typed(cx);
        let typed = self.name.read(cx).text().trim().to_owned();
        let name = if typed.is_empty() {
            kind.describe(&values)
        } else {
            typed
        };
        let editing = self.editing;
        let added = self.page.update(cx, |p, cx| match editing {
            Some(at) => p.change_other(at, name, values, &secrets, cx),
            None => p.add_other(kind.id(), name, values, &secrets, cx),
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
        .gap(ui(6.))
        .text_size(ui(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(c.muted))
        .child(label)
        .child(
            div()
                .h(ui(40.))
                .flex()
                .items_center()
                .px(ui(16.))
                .rounded(ui(16.))
                .bg(rgb(c.field))
                .text_size(ui(15.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(c.text))
                .child(input),
        )
}

impl Render for AddStorageDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let close = icon_button("close", "x", 40.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        let card = |id: (&'static str, usize), tag: &'static str, name: String, on: bool| {
            div()
                .debug_selector(|| format!("{}-{}", id.0, id.1))
                .id(id)
                .flex_1()
                .flex()
                .flex_col()
                .gap(ui(8.))
                .p(ui(14.))
                .rounded(ui(20.))
                .cursor_pointer()
                .bg(rgb(if on { c.tonal } else { c.well }))
                .text_color(rgb(if on { c.on_tonal } else { c.text }))
                .when(!on, |d| d.hover(|s| s.bg(rgb(c.well_hover))))
                .child(
                    tile(40., 14., tonal_tint())
                        .bg(rgb(c.surface))
                        .text_size(ui(11.))
                        .font_weight(FontWeight::BOLD)
                        .child(tag),
                )
                .child(
                    div()
                        .text_size(ui(15.))
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
        let editing = self.editing.is_some();
        let remote = self.chosen.is_some();
        // With no room for it on this plan, nothing is asked: what lifts the limit is said.
        let limited = self.over_the_limit(cx);
        let can_ask = confirmer(cx).how().is_some();
        let folder = self.folder.clone().map(|input| {
            div()
                .w(ui(300.))
                .flex_grow(1.)
                .child(labelled(tr!("Folder"), input))
        });
        let fields = self.fields.iter().map(|(field, input)| {
            div()
                .w(ui(300.))
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
            .w(ui(720.))
            .child(header(
                "storages",
                tonal_tint(),
                if editing {
                    tr!("Edit storage")
                } else {
                    tr!("Add a storage")
                },
                if remote {
                    tr!("Secrets go in the system’s keychain, not with the settings.")
                } else {
                    tr!("Its files stay where they are.")
                },
                Some(close),
            ))
            .when(!editing, |d| {
                d.child(div().flex().gap(ui(10.)).children(kinds).child(local))
            })
            .when(!limited, |d| {
                d.child(
                div()
                    .id("storage-fields")
                    .min_h_0()
                    .overflow_y_scroll()
                    .flex()
                    .flex_wrap()
                    .gap(ui(14.))
                    .child(
                        div()
                            .w(ui(300.))
                            .flex_grow(1.)
                            .child(labelled(tr!("Name"), self.name.clone())),
                    )
                    .children(folder)
                    .children(fields),
            )
            })
            .when(remote && can_ask && !limited, |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(ui(4.))
                        .child(
                            check_row(
                                "storage-asks-first",
                                self.asks_first,
                                tr!("Ask for this computer’s password to open it"),
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.asks_first = !this.asks_first;
                                cx.notify();
                            })),
                        )
                        .child(
                            div()
                                .pl(ui(30.))
                                .text_size(ui(12.))
                                .text_color(rgb(c.muted))
                                .child(tr!(
                                    "For a computer left unlocked: what unlocks this computer opens it, and it asks again after 5 minutes left unused."
                                )),
                        ),
                )
            })
            .children(tested.map(|said| {
                div()
                    .text_size(ui(13.))
                    .text_color(rgb(if matches!(self.tested, Tested::Failed(_)) {
                        c.bad_text
                    } else {
                        c.muted
                    }))
                    .child(said)
            }))
            .children(self.problem.clone().map(|why| note(Tone::Warn).child(why)))
            .when(limited, |d| {
                d.child(note(Tone::Warn).child(tr!(
                    "packwing Free keeps one storage across a network. A Pro or Enterprise \
                     license lifts this: Settings › License. Folders on this computer are not \
                     limited."
                )))
            })
            .child(
                footer()
                    .when(remote && !limited, |d| {
                        d.child(
                            button(
                                "storage-test",
                                tr!("Test connection"),
                                Fill::Tonal,
                                Size::Medium,
                            )
                            .on_click(cx.listener(|this, _, _, cx| this.test(cx))),
                        )
                    })
                    .child(div().flex_1())
                    .child(
                        button("cancel", tr!("Cancel"), Fill::Outline, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(if limited {
                        button(
                            "storage-license",
                            tr!("Enter a license key"),
                            Fill::Accent,
                            Size::Medium,
                        )
                        .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::ShowLicense)))
                    } else {
                        button(
                            "storage-add",
                            if editing { tr!("Save") } else { tr!("Add") },
                            Fill::Accent,
                            Size::Medium,
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.add(cx)))
                    }),
            )
    }
}
