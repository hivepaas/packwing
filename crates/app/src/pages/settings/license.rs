//! License: which plan this computer is on, and where a key goes.

use std::time::SystemTime;

use gpui::{App, Context, Div, Entity, Render, Window, div, prelude::*, rgb};
use packwing_i18n::tr;
use packwing_license::{FREE_REMOTE_STORAGES, Plan, Standing};

use super::{rows, section, setting};

/// Where a license is bought and renewed.
pub const BUY: &str = "https://packwing.app/pricing";
use crate::colors::colors;
use crate::input::TextInput;
use crate::licensing::{self, Licensed};
use crate::store::Said;
use crate::text_size::ui;
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
            "Every feature but signing files, and {count} storage across a network. Folders on \
             this computer are not limited, and any signature can be checked.",
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

/// The Mac App Store's build: Pro bought there, or Buy and Restore Purchases, and what
/// the last of them came to.
fn app_store(cx: &App) -> Div {
    let store = crate::store::read(cx);
    let control = if store.holds {
        div()
            .debug_selector(|| "store-held".to_owned())
            .text_size(ui(15.))
            .child(tr!("Bought"))
    } else {
        let buy = match &store.price {
            Some(price) => tr!("Buy Pro · {price}", price = price.clone()),
            None => tr!("Buy Pro"),
        };
        div()
            .flex()
            .gap(ui(10.))
            .child(
                button("store-buy", buy, Fill::Accent, Size::Medium)
                    .when(store.busy, |d| d.opacity(0.5))
                    .on_click(|_, _, cx| crate::store::buy(cx)),
            )
            .child(
                button(
                    "store-restore",
                    tr!("Restore Purchases"),
                    Fill::Outline,
                    Size::Medium,
                )
                .when(store.busy, |d| d.opacity(0.5))
                .on_click(|_, _, cx| crate::store::restore(cx)),
            )
    };
    let said = store.said.map(|said| match said {
        Said::Pending => note(Tone::Good).child(tr!(
            "Waiting for approval: packwing turns Pro once it is given."
        )),
        Said::NothingToRestore => note(Tone::Warn).child(tr!(
            "The App Store found nothing bought with this Apple Account."
        )),
        Said::Failed(why) => {
            note(Tone::Warn).child(tr!("The App Store could not do it: {why}", why = why))
        }
    });
    div()
        .flex()
        .flex_col()
        .gap(ui(10.))
        .child(setting(
            tr!("App Store"),
            tr!("Pro, bought once in the App Store: yours on every Mac with your Apple Account."),
            control,
        ))
        .children(said.map(|n| n.debug_selector(|| "store-said".to_owned())))
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
            div().text_size(ui(15.)).child(plan.name()),
        )];
        if let Some(license) = &held.license {
            lines.push(setting(
                tr!("Issued to"),
                tr!("Who this license is for"),
                div().text_size(ui(15.)).child(license.holder.clone()),
            ));
            if let Some(seats) = license.seats {
                lines.push(setting(
                    tr!("Seats"),
                    tr!("What your organization’s agreement covers"),
                    div().text_size(ui(15.)).child(seats.to_string()),
                ));
            }
            let date = license.expires.date().to_string();
            if license.owned {
                let ended = license.expires < now;
                let (until, said) = if !license.covers(packwing_license::line_released()) {
                    (
                        tr!("Ended {date}", date = date),
                        tr!(
                            "This version came out after your updates ended, so it is Free. \
                             Renew, or use a version released by then: those stay Pro."
                        ),
                    )
                } else if ended {
                    (
                        tr!("Ended {date}", date = date),
                        tr!("This version stays Pro for good, with its fixes. Versions \
                             released after need a renewal."),
                    )
                } else {
                    (
                        tr!("Until {date}", date = date),
                        tr!(
                            "Every version released by then is yours for good, with its \
                             fixes. Renew for a year more of new versions."
                        ),
                    )
                };
                lines.push(setting(
                    tr!("Updates"),
                    said,
                    div()
                        .debug_selector(|| "license-updates".to_owned())
                        .text_size(ui(15.))
                        .child(until),
                ));
            } else {
                let until = match license.standing(now) {
                    Standing::Current if license.trial => {
                        tr!("Trial, until {date}", date = date)
                    }
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
                    div().text_size(ui(15.)).child(until),
                ));
            }
        }
        let problem = self.problem.clone().or(held.problem.clone());
        let store = crate::store::there();
        section()
            .child(rows(lines))
            .when(store && !held.by_policy, |d| d.child(app_store(cx)))
            .when(held.by_policy, |d| {
                d.child(note(Tone::Good).child(if held.refreshes {
                    tr!(
                        "Your organization manages this license. Once a day, packwing asks the \
                         portal for a newer key of it, sending only the key."
                    )
                } else {
                    tr!("Your organization manages this license.")
                }))
            })
            .when(!held.by_policy, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(ui(10.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .h(ui(40.))
                                .flex()
                                .items_center()
                                .px(ui(16.))
                                .rounded(ui(16.))
                                .bg(rgb(c.field))
                                .child(self.key.clone()),
                        )
                        .child(
                            button("license-enter", tr!("Activate"), Fill::Accent, Size::Medium)
                                .on_click(cx.listener(|this, _, _, cx| this.enter(cx))),
                        )
                        // The App Store's rules: what it sells is bought there, with no
                        // way out to another shop.
                        .when(!store, |d| {
                            d.child(
                                button(
                                    "license-buy",
                                    if held.license.as_ref().is_some_and(|l| l.owned) {
                                        tr!("Renew")
                                    } else {
                                        tr!("Buy")
                                    },
                                    Fill::Outline,
                                    Size::Medium,
                                )
                                .on_click(|_, _, cx| cx.open_url(BUY)),
                            )
                        })
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
            .when(!held.refreshes, |d| {
                d.child(div().text_size(ui(13.)).text_color(rgb(c.muted)).child(tr!(
                    "A key is checked on this computer. Nothing is sent anywhere, and no \
                             license changes what your organization’s policies do."
                )))
            })
    }
}
