//! Security: what packwing checks and keeps when it extracts. What an organization
//! fixed is shown locked, with its name in the policy, and only the rest can be changed.

use gpui::{Context, FontWeight, Render, SharedString, Window, div, prelude::*, px, relative, rgb};
use packwing_i18n::tr;
use packwing_policy::{
    ALLOWED_ENCRYPTION_METHODS, MARK_STORAGE_FILES, MIN_PASSWORD_LENGTH, ORGANIZATION_NAME,
    ORGANIZATION_URL, PROPAGATE_MOTW, Policy, REQUIRE_ENCRYPTION, Row, Setting,
};

use super::{rows, section, setting};
use crate::colors::{Tint, colors};
use crate::widgets::{Fill, Size, button, icon, switch, tile};

/// Monospace fonts for the policies' names, the first one installed.
const MONO_FONTS: [&str; 8] = [
    "SF Mono",
    "Menlo",
    "Cascadia Mono",
    "Consolas",
    "JetBrains Mono",
    "DejaVu Sans Mono",
    "Liberation Mono",
    "Ubuntu Mono",
];

/// What a switch sets.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sets {
    KeepMark,
    AskOpen,
}

struct Toggle {
    sets: Sets,
    label: String,
    detail: String,
    on: bool,
}

pub struct Security {
    /// The monospace font found, once looked for.
    mono: Option<SharedString>,
    /// The settings the user may change, unless a policy fixed them.
    toggles: Vec<Toggle>,
}

/// A policy as the user reads it: what it is about, and what it does.
fn told(key: &str) -> (String, String) {
    match key {
        PROPAGATE_MOTW => (
            tr!("Keep the “downloaded from the Internet” mark"),
            if cfg!(any(windows, target_os = "macos")) {
                tr!("The system warns before running files that came from a downloaded archive")
            } else {
                tr!("This system has no such mark, so this changes nothing here")
            },
        ),
        MARK_STORAGE_FILES => (
            tr!("Treat files from storages as from the Internet"),
            tr!("What comes from S3, WebDAV and SFTP gets the mark too"),
        ),
        REQUIRE_ENCRYPTION => (
            tr!("Require encryption for new archives"),
            tr!("Applies to archives made and files added here, not to other apps"),
        ),
        ALLOWED_ENCRYPTION_METHODS => (
            tr!("Kinds of encryption allowed"),
            tr!("New archives can be encrypted only these ways"),
        ),
        MIN_PASSWORD_LENGTH => (
            tr!("Minimum password length"),
            tr!("In characters, for every new archive with a password"),
        ),
        _ => (key.to_owned(), tr!("packwing has no policy of this name.")),
    }
}

impl Security {
    pub fn new(keep_mark: bool, ask_open: bool) -> Self {
        let toggles = vec![
            Toggle {
                sets: Sets::KeepMark,
                label: tr!("Keep the “downloaded from the Internet” mark"),
                detail: tr!(
                    "The system warns before running files that came from a downloaded \
                         archive"
                ),
                on: keep_mark,
            },
            Toggle {
                sets: Sets::AskOpen,
                label: tr!("Ask before opening files without extracting"),
                detail: tr!(
                    "Opening a file inside an archive puts a copy in a temporary folder. \
                         A program always asks."
                ),
                on: ask_open,
            },
        ];
        Security {
            mono: None,
            toggles,
        }
    }

    fn banner(policy: &Policy) -> impl IntoElement {
        let c = colors();
        let by = match &policy.organization {
            Setting::Set(name) => Some(name.clone()),
            _ => None,
        };
        let url = match &policy.url {
            Setting::Set(url) => Some(url.clone()),
            _ => None,
        };
        let wrong = policy.failure().map(|why| {
            tr!(
                "Its policies cannot be read, so what they guard refuses: {why}",
                why = why
            )
        });
        div()
            .flex()
            .items_center()
            .gap(px(14.))
            .px(px(18.))
            .py(px(16.))
            .rounded(px(22.))
            .bg(rgb(c.warn_bg))
            .text_color(rgb(c.warn_text))
            .child(
                tile(
                    44.,
                    15.,
                    Tint {
                        ground: c.surface,
                        ink: c.warn_text,
                    },
                )
                .child(icon("building", 22., c.warn_text)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(px(14.))
                    .line_height(relative(1.5))
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_weight(FontWeight::BOLD)
                            .child(tr!("Some settings are managed by your organization")),
                    )
                    .when_some(by, |d, by| d.child(by))
                    .when_some(wrong, |d, wrong| d.child(wrong)),
            )
            .when_some(url, |d, url| {
                d.child(
                    button("policy-url", tr!("Get help"), Fill::Raised, Size::Small)
                        .text_color(rgb(c.warn_text))
                        .on_click(move |_, _, cx| cx.open_url(&url)),
                )
            })
    }

    /// What a policy fixed the setting at, with a lock, and its name.
    fn locked(row: &Row, mono: SharedString) -> impl IntoElement {
        let c = colors();
        let value = match (&row.problem, row.value.as_str()) {
            (Some(_), _) => tr!("Not understood"),
            (None, "true") => tr!("On"),
            (None, "false") => tr!("Off"),
            (None, value) => value.to_owned(),
        };
        div()
            .flex()
            .flex_col()
            .items_end()
            .gap(px(3.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .px(px(12.))
                    .py(px(4.))
                    .rounded_full()
                    .bg(rgb(c.surface))
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .whitespace_nowrap()
                    .child(icon("lock", 13., c.text))
                    .child(value),
            )
            .child(
                div()
                    .font_family(mono)
                    .text_size(px(11.))
                    .text_color(rgb(c.muted))
                    .child(row.key.clone()),
            )
    }
}

impl Render for Security {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mono = self
            .mono
            .get_or_insert_with(|| {
                let installed = window.text_system().all_font_names();
                MONO_FONTS
                    .iter()
                    .find(|f| installed.iter().any(|i| i == *f))
                    .copied()
                    .unwrap_or(crate::FONT)
                    .into()
            })
            .clone();
        let policy = crate::managed::policy(cx).clone();
        let locks_mark = policy.locks_mark();
        let open = self
            .toggles
            .iter()
            .enumerate()
            .filter(|(_, t)| !(locks_mark && t.sets == Sets::KeepMark))
            .map(|(i, t)| {
                setting(
                    t.label.clone(),
                    t.detail.clone(),
                    switch(("security", i), t.on).on_click(cx.listener(move |this, _, _, cx| {
                        this.toggles[i].on = !this.toggles[i].on;
                        if this.toggles[i].sets == Sets::KeepMark {
                            let on = this.toggles[i].on;
                            crate::prefs::change(cx, |p| p.keep_mark = on);
                        }
                        if this.toggles[i].sets == Sets::AskOpen {
                            let on = this.toggles[i].on;
                            crate::prefs::change(cx, |p| p.ask_before_open = on);
                        }
                        cx.notify();
                    })),
                )
            });
        if !policy.is_managed() {
            return section().child(rows(open));
        }
        let shown = policy.rows();
        let fixed = shown
            .iter()
            // These two are in the banner, unless they are wrong.
            .filter(|r| {
                r.problem.is_some() || (r.key != ORGANIZATION_NAME && r.key != ORGANIZATION_URL)
            })
            .map(|r| {
                let (label, detail) = told(&r.key);
                let detail = r.problem.clone().unwrap_or(detail);
                setting(label, detail, Self::locked(r, mono.clone()))
            });
        section()
            .gap(px(16.))
            .child(Self::banner(&policy))
            .child(rows(fixed.chain(open)))
    }
}
