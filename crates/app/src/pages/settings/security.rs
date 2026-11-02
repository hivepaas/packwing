//! Security: what packwing checks and keeps when it extracts. What an organization
//! fixed is shown locked, with its name in the policy, and only the rest can be changed.

use gpui::{
    Context, Entity, FontWeight, PathPromptOptions, Render, SharedString, Window, div, prelude::*,
    px, relative, rgb,
};
use packwing_i18n::tr;
use packwing_policy::{
    ALLOWED_ENCRYPTION_METHODS, LICENSE_KEY, MARK_STORAGE_FILES, MIN_PASSWORD_LENGTH,
    ORGANIZATION_NAME, ORGANIZATION_URL, PROPAGATE_MOTW, Policy, REQUIRE_ENCRYPTION, Row, Setting,
};

use super::{rows, section, setting};
use crate::colors::{Tint, colors};
use crate::input::TextInput;
use crate::prefs;
use crate::widgets::{Fill, Size, Tone, button, icon, legend, note, switch, tile};

/// Monospace fonts for the policies' names, the first one installed.
pub(super) const MONO_FONTS: [&str; 8] = [
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
    /// Where a public key is pasted, and why the last one was not taken.
    new_key: Entity<TextInput>,
    key_problem: Option<String>,
}

/// What a public key's line calls it: what follows the key, if anything does.
fn comment_of(line: &str) -> String {
    let mut words = line.trim().splitn(3, char::is_whitespace);
    words.nth(2).unwrap_or_default().trim().to_owned()
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
        LICENSE_KEY => (
            tr!("License"),
            tr!("Your organization’s, in Settings › License"),
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
    pub fn new(keep_mark: bool, ask_open: bool, cx: &mut Context<Self>) -> Self {
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
            new_key: cx.new(|cx| TextInput::new(tr!("Paste a public key"), cx)),
            key_problem: None,
        }
    }

    /// Trusts the key in `text`, a line of a `.pub` file.
    fn trust(&mut self, text: &str, cx: &mut Context<Self>) {
        match packwing_core::read_public_key(text) {
            Ok(key) => {
                let name = comment_of(text.lines().next().unwrap_or_default());
                prefs::change(cx, |p| p.trust(name, &key));
                self.key_problem = None;
                self.new_key.update(cx, |k, cx| k.set_text("", 0..0, cx));
            }
            Err(why) => {
                self.key_problem = Some(tr!(
                    "This is not a public key packwing can check with: {why}.",
                    why = why
                ));
            }
        }
        cx.notify();
    }

    /// Asks for a `.pub` file, and trusts its key.
    fn pick_key(&mut self, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(tr!("Choose").into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(mut paths))) = picked.await else {
                return;
            };
            let Some(path) = paths.pop() else { return };
            let read = cx
                .background_executor()
                .spawn(async move {
                    std::fs::File::open(&path).and_then(|f| {
                        let mut text = String::new();
                        std::io::Read::read_to_string(
                            &mut std::io::Read::take(f, 64 << 10),
                            &mut text,
                        )?;
                        Ok(text)
                    })
                })
                .await;
            let _ = this.update(cx, |this, cx| match read {
                Ok(text) => this.trust(&text, cx),
                Err(e) => {
                    this.key_problem = Some(e.to_string());
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// The keys whose signatures are taken as proof, on every plan.
    fn trusted_keys(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let c = colors();
        let kept = prefs::prefs(cx).trusted_keys;
        let listed = kept.into_iter().enumerate().map(|(i, key)| {
            let told = packwing_core::read_public_key(&key.line).map_or_else(
                |_| tr!("This line cannot be read, so the key is not used"),
                |public| format!("{} · {}", public.key_type(), public.fingerprint()),
            );
            setting(
                key.name,
                told,
                button(
                    ("trusted-remove", i),
                    tr!("Remove"),
                    Fill::Outline,
                    Size::Small,
                )
                .on_click(cx.listener(move |_, _, _, cx| {
                    prefs::change(cx, |p| {
                        if i < p.trusted_keys.len() {
                            p.trusted_keys.remove(i);
                        }
                    });
                    cx.notify();
                })),
            )
        });
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(legend(tr!("Keys you trust")))
            .child(div().text_size(px(13.)).text_color(rgb(c.muted)).child(tr!(
                "A signature by one of these says who signed a file. One by any other key \
                     proves nothing, whatever it matches."
            )))
            .child(rows(listed))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .min_w(px(240.))
                            .items_center()
                            .h(px(40.))
                            .px(px(14.))
                            .rounded(px(14.))
                            .bg(rgb(c.field))
                            .text_size(px(14.))
                            .child(self.new_key.clone()),
                    )
                    .child(
                        button("trusted-add", tr!("Add"), Fill::Tonal, Size::Small).on_click(
                            cx.listener(|this, _, _, cx| {
                                let text = this.new_key.read(cx).text().to_owned();
                                this.trust(&text, cx);
                            }),
                        ),
                    )
                    .child(
                        button(
                            "trusted-pick",
                            tr!("Choose a file…"),
                            Fill::Outline,
                            Size::Small,
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.pick_key(cx))),
                    ),
            )
            .children(
                self.key_problem
                    .clone()
                    .map(|why| note(Tone::Warn).child(why)),
            )
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
        let trusted = self.trusted_keys(cx);
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
            return section().gap(px(20.)).child(rows(open)).child(trusted);
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
            .child(trusted)
    }
}
