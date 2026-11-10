//! About: what packwing is and which build this is, updates, where the code is, and
//! the other apps from HivePaaS.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui::{
    ClipboardItem, Context, Div, Entity, FontWeight, Render, Subscription, Window, div, prelude::*,
    relative, rgb, svg,
};
use packwing_i18n::tr;
use packwing_update::Install;

use super::{explain, group, heading, label, rows, section, setting};
use crate::colors::colors;
use crate::text_size::ui;
use crate::updates::{self, Stage, Updates};
use crate::widgets::{
    Fill, Size, Tone, accent_tint, button, chip, logo, mono_font, note, switch, tile,
};

const SOURCE: &str = "https://github.com/hivepaas/packwing";
const LICENSE: &str = "https://github.com/hivepaas/packwing/blob/main/LICENSE";
const ISSUES: &str = "https://github.com/hivepaas/packwing/issues/new";
const RELEASES: &str = "https://github.com/hivepaas/packwing/releases";
const SHELLWING: &str = "https://shellwing.app";
const SHELLWING_SOURCE: &str = "https://github.com/hivepaas/shellwing";
const TRACEWING: &str = "https://tracewing.app";
const TRACEWING_SOURCE: &str = "https://github.com/hivepaas/tracewing";
const HIVEPAAS: &str = "https://hivepaas.com";
const HIVEPAAS_SOURCE: &str = "https://github.com/hivepaas/hivepaas";
/// The year in the copyright line.
const YEAR: u32 = 2026;
/// How long Copy version info says it did.
const COPIED_FOR: Duration = Duration::from_secs(2);

/// What a problem report needs to say about this build.
fn version_info() -> String {
    format!(
        "packwing {} ({} {})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

pub struct About {
    updates: Entity<Updates>,
    /// Which Copy says it did.
    copied: Option<&'static str>,
    _updates: Subscription,
}

impl About {
    pub fn new(updates: Entity<Updates>, cx: &mut Context<Self>) -> About {
        let observed = cx.observe(&updates, |_, _, cx| cx.notify());
        About {
            updates,
            copied: None,
            _updates: observed,
        }
    }

    fn copy(&mut self, which: &'static str, text: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.copied = Some(which);
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(COPIED_FOR).await;
            let _ = this.update(cx, |this, cx| {
                if this.copied == Some(which) {
                    this.copied = None;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// The logo, the name, what it is and which build this is.
    fn identity(&self, cx: &mut Context<Self>) -> Div {
        let c = colors();
        let copy = if self.copied == Some("version") {
            tr!("Copied")
        } else {
            tr!("Copy version info")
        };
        group()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(ui(18.))
            .py(ui(20.))
            .child(tile(72., 24., accent_tint()).child(logo(36., accent_tint())))
            .child(
                div()
                    .flex_1()
                    .min_w(ui(220.))
                    .flex()
                    .flex_col()
                    .gap(ui(2.))
                    .child(
                        div()
                            .text_size(ui(28.))
                            .font_weight(FontWeight::BOLD)
                            .line_height(relative(1.2))
                            .child("packwing"),
                    )
                    .child(div().text_size(ui(14.)).text_color(rgb(c.muted)).child(tr!(
                        "Version {version} · {os} {arch}",
                        version = env!("CARGO_PKG_VERSION"),
                        os = std::env::consts::OS,
                        arch = std::env::consts::ARCH
                    )))
                    .child(div().pt(ui(6.)).text_size(ui(15.)).child(tr!(
                        "A file archiver that puts every core to work and opens archives safely."
                    ))),
            )
            .child(
                button("about-copy-version", copy, Fill::Outline, Size::Medium).on_click(
                    cx.listener(|this, _, _, cx| this.copy("version", version_info(), cx)),
                ),
            )
    }

    /// Which channel this build is on, what a look found, and getting it.
    fn updates(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        let u = self.updates.read(cx);
        let keys = u.has_keys();
        let (checking, on) = (u.checking(), u.checks());
        let beta = packwing_update::current().channel() == packwing_update::Channel::Beta;
        let status = self.status(window, cx);
        let check = self.updates.clone();
        let toggle = self.updates.clone();
        group()
            .gap(ui(14.))
            .child(heading(
                tr!("Updates"),
                if beta {
                    tr!(
                        "This is a beta: you are told of newer betas, and of the release that follows them."
                    )
                } else {
                    tr!("This is a release: you are told of newer releases, not of betas.")
                },
            ))
            .child(status)
            .when(keys, |d| {
                d.child(
                    div().flex().child(
                        button("update-check", tr!("Check now"), Fill::Outline, Size::Medium)
                            .when(checking, |d| d.opacity(0.5))
                            .on_click(move |_, _, cx| check.update(cx, |u, cx| u.check(cx))),
                    ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(ui(16.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(label(tr!("Look for updates once a day")))
                                .child(explain(tr!(
                                    "It reads the list of releases from GitHub, and believes it only when its signatures verify. Nothing is downloaded or installed without you."
                                ))),
                        )
                        .child(switch("update-auto", on).on_click(move |_, _, cx| {
                            toggle.update(cx, |u, cx| u.set_checks(!on, cx));
                        })),
                )
            })
    }

    /// What a look found, and what can be done with it.
    fn status(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        let c = colors();
        let u = self.updates.read(cx);
        let handle = self.updates.clone();
        let skip = |handle: Entity<Updates>| {
            button(
                "update-skip",
                tr!("Skip this version"),
                Fill::Text,
                Size::Medium,
            )
            .on_click(move |_, _, cx| handle.update(cx, |u, cx| u.skip(cx)))
        };
        if !u.has_keys() {
            return explain(tr!(
                "This build has no release keys, so it cannot tell of updates."
            ));
        }
        if let Stage::Ready(v) = u.stage() {
            return card(
                tr!("packwing {version} is ready", version = v.to_string()),
                explain(tr!("It is installed when you quit, or now with Restart.")),
            )
            .child(
                actions().child(
                    button(
                        "update-restart",
                        tr!("Restart now"),
                        Fill::Accent,
                        Size::Medium,
                    )
                    .on_click(move |_, _, cx| handle.update(cx, |u, cx| u.restart(cx))),
                ),
            );
        }
        let offered = u.offered().map(|v| v.to_string());
        if let Stage::Downloading(got, of) = *u.stage() {
            let done = of
                .filter(|n| *n > 0)
                .map(|of| (got as f32 / of as f32).clamp(0., 1.));
            return card(
                tr!("packwing {version}", version = offered.unwrap_or_default()),
                explain(updates::download_said(got, of)),
            )
            .when_some(done, |d, done| {
                d.child(
                    div()
                        .h(ui(4.))
                        .w_full()
                        .rounded_full()
                        .bg(rgb(c.outline))
                        .child(
                            div()
                                .h_full()
                                .w(relative(done))
                                .rounded_full()
                                .bg(rgb(c.accent)),
                        ),
                )
            });
        }
        if u.checking() {
            return explain(tr!("Looking for a newer version…"));
        }
        if let Some(version) = offered {
            let notes = {
                let page = u.offered().map(|v| packwing_update::release_page(&v));
                button(
                    "update-notes",
                    tr!("Release notes"),
                    Fill::Outline,
                    Size::Medium,
                )
                .on_click(move |_, _, cx| {
                    if let Some(page) = &page {
                        cx.open_url(page);
                    }
                })
            };
            let title = tr!("packwing {version} is out", version = version);
            if let Some(steps) = u.steps() {
                let line = steps.join(" && ");
                let mono = mono_font(window);
                let copy = if self.copied == Some("steps") {
                    tr!("Copied")
                } else {
                    tr!("Copy")
                };
                return card(
                    title,
                    explain(tr!(
                        "Run these in a terminal. They check the download against the signed list of releases, and sudo asks for your password."
                    )),
                )
                .child(
                    div()
                        .id("update-steps")
                        .px(ui(12.))
                        .py(ui(10.))
                        .rounded(ui(12.))
                        .bg(rgb(c.field))
                        .font_family(mono)
                        .text_size(ui(12.))
                        .children(steps.iter().enumerate().map(|(i, step)| {
                            let more = if i + 1 < steps.len() { " &&" } else { "" };
                            div().child(format!("{step}{more}"))
                        })),
                )
                .child(
                    actions()
                        .child(
                            button("update-copy", copy, Fill::Accent, Size::Medium).on_click(
                                cx.listener(move |this, _, _, cx| {
                                    this.copy("steps", line.clone(), cx)
                                }),
                            ),
                        )
                        .child(notes)
                        .child(skip(handle)),
                );
            }
            let how = u.install();
            let said = match &how {
                Install::Managed {
                    by,
                    command: Some(command),
                } => tr!(
                    "{by} updates it: {command}",
                    by = updates::by_said(by),
                    command = *command
                ),
                Install::Managed { by, command: None } => {
                    tr!("{by} updates it.", by = updates::by_said(by))
                }
                i if i.updates_itself() => {
                    tr!("It is downloaded and checked, then installed when you quit.")
                }
                _ => tr!("Download it, and install it in place of this one."),
            };
            let failed = match u.stage() {
                Stage::Failed(why) => Some(tr!("The download failed: {why}", why = why.clone())),
                _ => None,
            };
            let get = match (&how, &failed) {
                (Install::Managed { .. }, _) => None,
                (i, Some(_)) if i.updates_itself() => Some(tr!("Try again")),
                (i, None) if i.updates_itself() => Some(tr!("Download and install")),
                _ => Some(tr!("Download")),
            };
            let getter = handle.clone();
            return card(title, explain(said))
                .when_some(failed, |d, why| d.child(note(Tone::Warn).child(why)))
                .child(
                    actions()
                        .when_some(get, |d, get| {
                            d.child(
                                button("update-get", get, Fill::Accent, Size::Medium)
                                    .on_click(move |_, _, cx| getter.update(cx, |u, cx| u.get(cx))),
                            )
                        })
                        .child(notes)
                        .child(skip(handle)),
                );
        }
        if let Some(problem) = u.problem() {
            return note(Tone::Warn)
                .child(tr!("The last look failed: {why}", why = problem.to_owned()));
        }
        match u.checked_at() {
            Some(then) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                explain(tr!(
                    "Up to date (looked {when}).",
                    when = updates::ago(then, now)
                ))
            }
            None => explain(tr!("Not looked yet.")),
        }
    }
}

impl Render for About {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        section()
            .child(self.identity(cx))
            .child(self.updates(window, cx))
            .child(rows([
                setting(
                    tr!("Free software"),
                    tr!("Under the GNU General Public License, version 3 or later"),
                    link("license", tr!("View license"), LICENSE, Fill::Outline),
                ),
                setting(
                    tr!("Made by HivePaaS"),
                    tr!("On hivepack, our library for reading and writing archives"),
                    link("source", tr!("Source code"), SOURCE, Fill::Outline),
                ),
                setting(
                    tr!("Something wrong?"),
                    tr!("Tell us, with the version above"),
                    link("issue", tr!("Report a problem"), ISSUES, Fill::Outline),
                ),
                setting(
                    tr!("What changed"),
                    tr!("Every release, and what is new in it"),
                    link("releases", tr!("Release notes"), RELEASES, Fill::Outline),
                ),
            ]))
            .child(other_apps())
            .child(explain(tr!(
                "© {year} HivePaaS. The packwing name and logo are trademarks of HivePaaS.",
                year = YEAR.to_string()
            )))
    }
}

/// A button that opens a web page.
fn link(id: &'static str, name: String, url: &'static str, fill: Fill) -> gpui::Stateful<Div> {
    button(id, name, fill, Size::Medium).on_click(move |_, _, cx| cx.open_url(url))
}

/// Buttons side by side.
fn actions() -> Div {
    div().flex().flex_wrap().items_center().gap(ui(8.))
}

/// A release, as Updates shows it: what it is and what is said of it; what can be done
/// follows.
fn card(title: String, body: Div) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(ui(10.))
        .p(ui(14.))
        .rounded(ui(16.))
        .bg(rgb(colors().surface))
        .child(label(title))
        .child(body)
}

/// shellwing's icon: the wing on a pale tile, as it draws it, in either look.
fn shellwing_logo() -> Div {
    logo_tile(0xeef1f5).child(
        svg()
            .path("icons/shellwing.svg")
            .size(ui(36.))
            .text_color(rgb(0x0284c7)),
    )
}

/// tracewing's icon: its trace in white on its blue tile, as it draws it, in either
/// look.
fn tracewing_logo() -> Div {
    logo_tile(0x2f55b4).child(
        svg()
            .path("icons/tracewing.svg")
            .size(ui(30.))
            .text_color(rgb(0xffffff)),
    )
}

/// HivePaaS's mark, seven cells of a hive in its brown and amber, on a pale tile (the
/// brown does not show on a dark one).
fn hivepaas_logo() -> Div {
    const BROWN: u32 = 0x543d2b;
    const AMBER: u32 = 0xf59e0b;
    // Centers in the mark's 36-unit box.
    const CELLS: [(f32, f32, u32); 7] = [
        (12.98, 9.30, BROWN),
        (23.02, 9.30, AMBER),
        (7.95, 18.0, BROWN),
        (18.0, 18.0, AMBER),
        (28.05, 18.0, AMBER),
        (12.98, 26.70, BROWN),
        (23.02, 26.70, AMBER),
    ];
    logo_tile(0xfbf7f0).child(
        div()
            .relative()
            .size(ui(36.))
            .children(CELLS.map(|(x, y, c)| {
                div()
                    .absolute()
                    .left(ui(x - 4.3))
                    .top(ui(y - 4.3))
                    .size(ui(8.6))
                    .rounded_full()
                    .bg(rgb(c))
            })),
    )
}

/// A logo's tile, of its own ground.
fn logo_tile(ground: u32) -> Div {
    div()
        .size(ui(56.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(ui(16.))
        .bg(rgb(ground))
}

/// An app of the same family: its logo, name and what it is, its strong points, and the
/// way to it.
fn app(mark: Div, name: &'static str, what: String, points: Vec<String>, ways: gpui::Div) -> Div {
    let c = colors();
    div()
        .flex()
        .flex_col()
        .gap(ui(12.))
        .p(ui(14.))
        .rounded(ui(16.))
        .bg(rgb(c.surface))
        .child(
            div().flex().items_center().gap(ui(14.)).child(mark).child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(ui(2.))
                    .child(
                        div()
                            .text_size(ui(18.))
                            .font_weight(FontWeight::BOLD)
                            .child(name),
                    )
                    .child(explain(what)),
            ),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(ui(6.))
                .children(points.into_iter().map(|p| chip(p, c.field, c.text))),
        )
        .child(ways)
}

/// The other apps from HivePaaS: what each is, and where to see more.
fn other_apps() -> Div {
    group()
        .gap(ui(14.))
        .child(heading(
            tr!("More from HivePaaS"),
            tr!("Free and open source, like packwing."),
        ))
        .child(app(
            shellwing_logo(),
            "shellwing",
            tr!("A fast, native SSH client for Linux, macOS and Windows."),
            vec![
                tr!("SSH and SFTP"),
                tr!("Follows your ssh_config"),
                tr!("A real terminal"),
                tr!("GPL-3.0-or-later"),
            ],
            actions()
                .child(link(
                    "about-shellwing",
                    tr!("Visit shellwing.app"),
                    SHELLWING,
                    Fill::Tonal,
                ))
                .child(link(
                    "about-shellwing-source",
                    "GitHub".to_owned(),
                    SHELLWING_SOURCE,
                    Fill::Outline,
                )),
        ))
        .child(app(
            tracewing_logo(),
            "tracewing",
            tr!(
                "A native HTTP(S) debugging proxy that keeps up with big sessions. In early development: nothing to download yet."
            ),
            vec![
                tr!("HTTP/2, WebSocket and SSE"),
                tr!("A million requests, smooth"),
                tr!("Breakpoints and rewrites"),
                tr!("Phones and emulators"),
                tr!("GPL-3.0-or-later"),
            ],
            actions()
                .child(link(
                    "about-tracewing",
                    tr!("Visit tracewing.app"),
                    TRACEWING,
                    Fill::Tonal,
                ))
                .child(link(
                    "about-tracewing-source",
                    "GitHub".to_owned(),
                    TRACEWING_SOURCE,
                    Fill::Outline,
                )),
        ))
        .child(app(
            hivepaas_logo(),
            "HivePaaS",
            tr!(
                "A self-hosted platform for your apps, built on Docker Swarm: an open-source alternative to Heroku, Render and Coolify."
            ),
            vec![
                tr!("300+ one-click apps"),
                tr!("Deploy on push"),
                tr!("Pull request previews"),
                tr!("Automatic TLS"),
                tr!("Live logs and metrics"),
                tr!("Apache-2.0"),
            ],
            actions()
                .child(link(
                    "about-hivepaas",
                    tr!("Visit hivepaas.com"),
                    HIVEPAAS,
                    Fill::Tonal,
                ))
                .child(link(
                    "about-hivepaas-source",
                    "GitHub".to_owned(),
                    HIVEPAAS_SOURCE,
                    Fill::Outline,
                )),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_info_names_the_build() {
        let info = version_info();
        assert!(info.starts_with(&format!("packwing {} (", env!("CARGO_PKG_VERSION"))));
        assert!(info.ends_with(&format!("{})", std::env::consts::ARCH)));
    }
}
