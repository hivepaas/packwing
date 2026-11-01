//! Copying or moving what is chosen in a storage: where to, in the same explorer
//! as the page's, showing folders alone. The folder it shows is where they go.

use gpui::{Context, Entity, EventEmitter, Render, Subscription, Window, div, prelude::*, px, rgb};
use packwing_core::{OnExisting, Transfer};
use packwing_i18n::{tr, tr_n};
use std::sync::{Arc, Mutex};

use super::{DialogEvent, Job, Transferring, Verb, footer, header, places};
use crate::colors::colors;
use crate::explorer::{Explorer, Place};
use crate::widgets::{
    Fill, Select, SelectLook, Size, button, icon_button, legend, segment, segments, tonal_tint,
};

/// What a name taken can come to.
fn on_existing() -> [(String, OnExisting); 3] {
    [
        (tr!("Stop and say so"), OnExisting::Stop),
        (tr!("Skip it"), OnExisting::Skip),
        (tr!("Replace it"), OnExisting::Replace),
    ]
}

pub struct TransferDialog {
    from: Place,
    from_folder: Vec<String>,
    names: Vec<String>,
    moving: bool,
    places: Vec<Place>,
    chosen: usize,
    picker: Entity<Explorer>,
    on_existing: Entity<Select>,
    _picker_changed: Subscription,
}

impl EventEmitter<DialogEvent> for TransferDialog {}

impl TransferDialog {
    /// Starts at `chosen` of `places`, in the folder they are in if it is theirs.
    pub fn new(
        from: Place,
        from_folder: Vec<String>,
        names: Vec<String>,
        moving: bool,
        places: Vec<Place>,
        chosen: usize,
        cx: &mut Context<Self>,
    ) -> TransferDialog {
        let start = places.get(chosen).cloned();
        let folder = from_folder.clone();
        let picker = cx.new(|cx| {
            let mut picker = Explorer::picker(cx);
            picker.show_place(start, cx);
            picker.open_folder(folder, cx);
            picker
        });
        let changed = cx.observe(&picker, |_, _, cx| cx.notify());
        let on_existing =
            cx.new(|_| Select::new(&on_existing().map(|(name, _)| name), 0, SelectLook::Pill));
        TransferDialog {
            from,
            from_folder,
            names,
            moving,
            places,
            chosen,
            picker,
            on_existing,
            _picker_changed: changed,
        }
    }

    fn pick_place(&mut self, i: usize, cx: &mut Context<Self>) {
        self.chosen = i;
        let place = self.places.get(i).cloned();
        self.picker.update(cx, |p, cx| p.show_place(place, cx));
        cx.notify();
    }

    /// Whether the folder shown is the one they are in already.
    fn is_where_they_are(&self, cx: &Context<Self>) -> bool {
        let picker = self.picker.read(cx);
        let here = self.from.storage.local_path(&self.from_folder);
        let there = picker
            .place()
            .and_then(|p| p.storage.local_path(picker.folder()));
        here.is_some() && here == there
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        let picker = self.picker.read(cx);
        let Some(to) = picker.place().cloned() else {
            return;
        };
        let to_folder = picker.folder().to_vec();
        let on_existing = on_existing()[self.on_existing.read(cx).chosen().min(2)].1;
        let what = match self.names.as_slice() {
            [one] => one.clone(),
            many => tr_n!(many.len() as u64, "{n} item", "{n} items"),
        };
        let shown_to = places::shown(&to.name, &to_folder);
        let verb = if self.moving { Verb::Move } else { Verb::Copy };
        let mut job = Job::shown(verb, what.into(), Some(shown_to.into()), 0, 0);
        job.transfer = Some(Transferring(
            Transfer {
                from: self.from.storage.clone(),
                from_folder: self.from_folder.clone(),
                names: self.names.clone(),
                to: to.storage,
                to_folder,
                moving: self.moving,
                on_existing,
            },
            Arc::new(Mutex::default()),
        ));
        cx.emit(DialogEvent::Start(job));
    }
}

impl Render for TransferDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let close = icon_button("close", "x", 44.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        let n = self.names.len() as u64;
        let what = match self.names.as_slice() {
            [one] => one.clone(),
            _ => tr_n!(n, "{n} item", "{n} items"),
        };
        let title = if self.moving {
            tr!("Move {what}", what = what)
        } else {
            tr!("Copy {what}", what = what)
        };
        let from = places::shown(&self.from.name, &self.from_folder);
        let places = places::list(&self.places, self.chosen, Self::pick_place, cx);
        let same = self.is_where_they_are(cx);
        let there = self.picker.read(cx).is_local();
        let through = !there || self.from.storage.local_path(&[]).is_none();
        let action = if self.moving {
            tr!("Move here")
        } else {
            tr!("Copy here")
        };
        let picker = self.picker.clone();
        super::card()
            .w(px(900.))
            .child(header("storages", tonal_tint(), title, from, Some(close)))
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .h(px(340.))
                    .flex_shrink_0()
                    .p(px(10.))
                    .rounded(px(22.))
                    .bg(rgb(c.well))
                    .child(places)
                    .child(div().flex_1().min_w_0().child(self.picker.clone())),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(16.))
                    .child(div().w(px(220.)).child(segments([
                        segment("do-copy", tr!("Copy"), !self.moving).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.moving = false;
                                cx.notify();
                            },
                        )),
                        segment("do-move", tr!("Move"), self.moving).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.moving = true;
                                cx.notify();
                            },
                        )),
                    ])))
                    .child(legend(tr!("If a name is taken")))
                    .child(self.on_existing.clone()),
            )
            .child(
                div()
                    .text_size(px(13.))
                    .text_color(rgb(c.muted))
                    .child(if same {
                        tr!("They are in this folder already. Choose another.")
                    } else if through {
                        tr!("All of it passes through this computer, and your provider may charge for what is read from it. Each moved original goes once its copy has been read back and matches.")
                    } else if self.moving {
                        tr!("Each original goes once its copy has been read back and matches.")
                    } else {
                        tr!("The originals stay where they are.")
                    }),
            )
            .child(
                footer()
                    .child(
                        button("new-folder", tr!("New folder"), Fill::Text, Size::Medium).on_click(
                            move |_, window, cx| {
                                picker.update(cx, |p, cx| p.new_folder(window, cx));
                            },
                        ),
                    )
                    .child(div().flex_1())
                    .child(
                        button("cancel", tr!("Cancel"), Fill::Outline, Size::Medium)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close))),
                    )
                    .child(
                        button("transfer-start", action, Fill::Accent, Size::Medium)
                            .when(same, |d| d.opacity(0.5))
                            .when(!same, |d| {
                                d.on_click(cx.listener(|this, _, _, cx| this.start(cx)))
                            }),
                    ),
            )
    }
}
