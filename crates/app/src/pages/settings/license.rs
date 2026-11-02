//! License: which plan this computer is on, and where a key goes.

use std::time::SystemTime;

use gpui::{App, Context, Entity, Render, Window, div, prelude::*, px, rgb};
use packwing_i18n::tr;
use packwing_license::{FREE_REMOTE_STORAGES, Plan, Standing};

use super::{rows, section, setting};
use crate::colors::colors;
use crate::input::TextInput;
use crate::licensing::{self, Licensed};
use crate::widgets::{Fill, Size, Tone, button, note};

pub struct LicenseSection {
    key: Entity<TextInput>,
    problem: Option<String>,
}

impl LicenseSection {
    pub fn new(cx: &mut Context<Self>) -> Self {
        LicenseSection {
            key: cx.new(|cx| TextInput::new(tr!("Paste your license key"), cx)),
            problem: None,
        }
    }

    fn enter(&mut self, cx: &mut Context<Self>) {
        let text = self.key.read(cx).text().to_owned();
        match licensing::enter(&text, cx) {
            Ok(()) => {
                self.problem = None;
                self.key.update(cx, |k, cx| k.set_text("", 0..0, cx));
            }
            Err(why) => self.problem = Some(why),
        }
        cx.notify();
    }
}

/// What a plan gives, in a line.
fn gives(plan: Plan) -> String {
    match plan {
        Plan::Free => tr!(
            "Every feature, and {count} storage across a network. Folders on this computer are \
             not limited.",
            count = FREE_REMOTE_STORAGES
        ),
        Plan::Pro => tr!("Every feature, and any number of storages."),
        Plan::Enterprise => {
            tr!("Every feature, and any number of storages, for your organization.")
        }
    }
}

fn read(cx: &App) -> Licensed {
    cx.try_global::<Licensed>().cloned().unwrap_or_default()
}

impl Render for LicenseSection {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let held = read(cx);
        let plan = licensing::plan(cx);
        let now = SystemTime::now().into();
        let mut lines = vec![setting(
            tr!("Plan"),
            gives(plan),
            div().text_size(px(15.)).child(plan.name()),
        )];
        if let Some(license) = &held.license {
            lines.push(setting(
                tr!("Issued to"),
                tr!("Who this license is for"),
                div().text_size(px(15.)).child(license.holder.clone()),
            ));
            if let Some(seats) = license.seats {
                lines.push(setting(
                    tr!("Seats"),
                    tr!("What your organization’s agreement covers"),
                    div().text_size(px(15.)).child(seats.to_string()),
                ));
            }
            let date = license.expires.date().to_string();
            let until = match license.standing(now) {
                Standing::Current if license.trial => tr!("Trial, until {date}", date = date),
                Standing::Current => tr!("Until {date}", date = date),
                Standing::InGrace(days) => tr!(
                    "Ended {date}. It keeps working for {days} days more.",
                    date = date,
                    days = days
                ),
                Standing::Over => tr!("Ended {date}", date = date),
            };
            lines.push(setting(
                tr!("Valid"),
                tr!("Past this, packwing is Free. Nothing is taken away but adding storages."),
                div().text_size(px(15.)).child(until),
            ));
        }
        let problem = self.problem.clone().or(held.problem.clone());
        section()
            .child(rows(lines))
            .when(held.by_policy, |d| {
                d.child(note(Tone::Good).child(tr!("Your organization manages this license.")))
            })
            .when(!held.by_policy, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .h(px(44.))
                                .flex()
                                .items_center()
                                .px(px(16.))
                                .rounded(px(16.))
                                .bg(rgb(c.field))
                                .child(self.key.clone()),
                        )
                        .child(
                            button("license-enter", tr!("Activate"), Fill::Accent, Size::Medium)
                                .on_click(cx.listener(|this, _, _, cx| this.enter(cx))),
                        )
                        .when(held.license.is_some(), |d| {
                            d.child(
                                button(
                                    "license-remove",
                                    tr!("Remove"),
                                    Fill::Outline,
                                    Size::Medium,
                                )
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        licensing::remove(cx);
                                        this.problem = None;
                                        cx.notify();
                                    },
                                )),
                            )
                        }),
                )
            })
            .when_some(problem, |d, why| d.child(note(Tone::Warn).child(why)))
            .child(div().text_size(px(13.)).text_color(rgb(c.muted)).child(tr!(
                "A key is checked on this computer. Nothing is sent anywhere, and no \
                         license changes what your organization’s policies do."
            )))
    }
}
