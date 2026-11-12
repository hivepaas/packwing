//! Security: what packwing checks and keeps when it extracts, its audit log, the keys
//! whose signatures it trusts and the checksums it shows. What an organization fixed is shown locked,
//! with its name in the policy, and only the rest can be changed.

use gpui::{
    AnyElement, Context, Entity, EventEmitter, FontWeight, PathPromptOptions, Render, SharedString,
    Subscription, Window, div, prelude::*, relative, rgb,
};
use packwing_core::Algorithm;
use packwing_core::audit::RETENTION_CHOICES;
use packwing_i18n::tr;
use packwing_policy::{
    ALLOWED_ENCRYPTION_METHODS, AUDIT_LOG, AUDIT_LOG_DETAIL, AUDIT_LOG_MAX_SIZE_MB, AUDIT_LOG_PATH,
    AUDIT_LOG_REQUIRED, AUDIT_LOG_RETENTION_DAYS, AUDIT_LOG_SYSTEM, LICENSE_KEY, LICENSE_REFRESH,
    MARK_STORAGE_FILES, MIN_PASSWORD_LENGTH, ORGANIZATION_NAME, ORGANIZATION_URL, PROPAGATE_MOTW,
    Policy, REQUIRE_ENCRYPTION, Row, Setting,
};

use super::{rows, section, setting};
use crate::audit_log::By;
use crate::colors::{Tint, colors};
use crate::input::TextInput;
use crate::prefs;
use crate::text_size::ui;
use crate::widgets::{
    Chosen, Fill, Select, SelectLook, Size, Tone, button, check_row, icon, legend, mono_font, note,
    switch, tile,
};

/// How long the audit log's records are kept, as [`RETENTION_CHOICES`] holds them.
fn retention_names() -> [String; 4] {
    [
        tr!("30 days"),
        tr!("90 days"),
        tr!("1 year"),
        tr!("Keep all"),
    ]
}

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

/// The audit log's records were asked for: Browse shows them.
pub struct ShowAuditLog;

impl EventEmitter<ShowAuditLog> for Security {}

pub struct Security {
    /// The monospace font found, once looked for.
    mono: Option<SharedString>,
    /// The settings the user may change, unless a policy fixed them.
    toggles: Vec<Toggle>,
    /// Where a public key is pasted, and why the last one was not taken.
    new_key: Entity<TextInput>,
    key_problem: Option<String>,
    /// How long the audit log's records are kept, where the user says.
    retention: Entity<Select>,
    _retention_chosen: Subscription,
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
        LICENSE_REFRESH => (
            tr!("Newer license keys from the portal"),
            tr!(
                "Once a day, packwing asks for a newer key of your organization’s, sending only the key"
            ),
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
        AUDIT_LOG => (
            tr!("Keep an audit log"),
            tr!("A record of what packwing does, whatever the license"),
        ),
        AUDIT_LOG_PATH => (
            tr!("Where the audit log is kept"),
            tr!("A folder your organization chose"),
        ),
        AUDIT_LOG_RETENTION_DAYS => (
            tr!("How long the audit log is kept"),
            tr!("In days; 0 keeps every record"),
        ),
        AUDIT_LOG_MAX_SIZE_MB => (
            tr!("Most the audit log may take"),
            tr!("In MB; the oldest records go first"),
        ),
        AUDIT_LOG_DETAIL => (
            tr!("What the audit log says of archives"),
            tr!("“entries” names each file extracted, and those found damaged or left out"),
        ),
        AUDIT_LOG_REQUIRED => (
            tr!("Every job recorded"),
            tr!("A job whose start the audit log cannot write does not start"),
        ),
        AUDIT_LOG_SYSTEM => (
            tr!("The audit log in the system’s log"),
            tr!(
                "Each record goes to {log} as well, where your organization’s tools read it",
                log = packwing_core::audit::system_log_name()
            ),
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
        let days = prefs::prefs(cx).audit_retention_days;
        let retention = cx.new(|_| {
            Select::new(
                &retention_names(),
                RETENTION_CHOICES
                    .iter()
                    .position(|d| *d == days)
                    .unwrap_or(1),
                SelectLook::Pill,
            )
            .marked("audit-retention")
        });
        let retention_chosen = cx.subscribe(&retention, |_, select, _: &Chosen, cx| {
            let days = RETENTION_CHOICES[select.read(cx).chosen().min(RETENTION_CHOICES.len() - 1)];
            prefs::change(cx, |p| p.audit_retention_days = days);
            crate::audit_log::apply(cx, By::User);
        });
        Security {
            mono: None,
            toggles,
            new_key: cx.new(|cx| TextInput::new(tr!("Paste a public key"), cx)),
            key_problem: None,
            retention,
            _retention_chosen: retention_chosen,
        }
    }

    /// The audit log: whether it is kept, for how long, and where, as the
    /// organization fixed it, the plan has it, or the user chose.
    fn audit_log(&self, policy: &Policy, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let c = colors();
        let kept = prefs::prefs(cx);
        let mut items: Vec<AnyElement> = Vec::new();
        if !policy.locks_audit_log() {
            let label = tr!("Keep an audit log");
            if crate::licensing::plan(cx).has_audit_log() {
                items.push(
                    setting(
                        label,
                        tr!(
                            "What packwing does, kept on this computer: archives extracted, \
                             made, converted and copied, when and by whom. Never a password \
                             or what is in a file."
                        ),
                        switch("audit-log", kept.audit_log).on_click(cx.listener(|_, _, _, cx| {
                            prefs::change(cx, |p| p.audit_log = !p.audit_log);
                            crate::audit_log::apply(cx, By::User);
                            cx.notify();
                        })),
                    )
                    .into_any_element(),
                );
                if kept.audit_log && !policy.locks_audit_log_retention() {
                    items.push(
                        setting(
                            tr!("Keep its records for"),
                            tr!("Older ones are taken away, and that is recorded too"),
                            self.retention.clone(),
                        )
                        .into_any_element(),
                    );
                }
            } else {
                items.push(
                    setting(
                        label,
                        tr!("Takes a Pro or Enterprise license: Settings › License."),
                        div()
                            .flex()
                            .items_center()
                            .gap(ui(6.))
                            .px(ui(12.))
                            .py(ui(4.))
                            .rounded_full()
                            .bg(rgb(c.surface))
                            .text_size(ui(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(icon("lock", 13., c.text))
                            .child(packwing_license::Plan::Pro.name()),
                    )
                    .into_any_element(),
                );
            }
        }
        if let Some(config) = packwing_core::audit::config() {
            let folder = config.folder.clone();
            items.push(
                setting(
                    tr!("Where the audit log is"),
                    config.folder.to_string_lossy().into_owned(),
                    div()
                        .flex()
                        .flex_wrap()
                        .justify_end()
                        .gap(ui(8.))
                        .child(
                            button("audit-view", tr!("View records"), Fill::Tonal, Size::Medium)
                                .on_click(cx.listener(|_, _, _, cx| cx.emit(ShowAuditLog))),
                        )
                        .child(
                            button(
                                "audit-open",
                                tr!("Open folder"),
                                Fill::Outline,
                                Size::Medium,
                            )
                            .on_click(move |_, _, cx| {
                                let _ = std::fs::create_dir_all(&folder);
                                crate::file_manager::open(&folder, cx);
                            }),
                        ),
                )
                .into_any_element(),
            );
        }
        let failure = packwing_core::audit::last_failure().map(|why| {
            note(Tone::Warn).child(tr!(
                "The last record could not be written: {why}",
                why = why
            ))
        });
        div()
            .flex()
            .flex_col()
            .gap(ui(8.))
            .child(legend(tr!("Audit log")))
            .child(div().text_size(ui(13.)).text_color(rgb(c.muted)).child(tr!(
                "pkw audit verify checks that no record was changed or taken out. Whoever \
                     runs this computer can still remove the log: it tells what packwing \
                     did, it does not stop anyone."
            )))
            .when(!items.is_empty(), |d| d.child(rows(items)))
            .children(failure)
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
                    Size::Medium,
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
            .gap(ui(8.))
            .child(legend(tr!("Keys you trust")))
            .child(div().text_size(ui(13.)).text_color(rgb(c.muted)).child(tr!(
                "A signature by one of these says who signed a file. One by any other key \
                     proves nothing, whatever it matches."
            )))
            .child(rows(listed))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(ui(8.))
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .min_w(ui(240.))
                            .items_center()
                            .h(ui(40.))
                            .px(ui(14.))
                            .rounded(ui(14.))
                            .bg(rgb(c.field))
                            .text_size(ui(14.))
                            .child(self.new_key.clone()),
                    )
                    .child(
                        button("trusted-add", tr!("Add"), Fill::Tonal, Size::Medium).on_click(
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
                            Size::Medium,
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

    /// Which checksums a file's Checksums… shows: one at least.
    fn checksums(cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let c = colors();
        let shown = prefs::prefs(cx).checksums;
        let boxes = Algorithm::ALL.into_iter().map(|a| {
            let on = shown.contains(&a);
            let label = if a.is_weak() {
                tr!("{name} (weak)", name = a.name())
            } else {
                a.name().to_owned()
            };
            check_row(("checksum", a as usize), on, label).on_click(cx.listener(
                move |_, _, _, cx| {
                    prefs::change(cx, |p| p.show_checksum(a, !on));
                    cx.notify();
                },
            ))
        });
        div()
            .flex()
            .flex_col()
            .gap(ui(8.))
            .child(legend(tr!("Checksums")))
            .child(div().text_size(ui(13.)).text_color(rgb(c.muted)).child(tr!(
                "What Checksums… in a file’s menu shows, to compare with the one given \
                     where the file came from. A weak one still tells a damaged file, but \
                     not one changed on purpose."
            )))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_x(ui(24.))
                    .gap_y(ui(12.))
                    .px(ui(18.))
                    .py(ui(16.))
                    .rounded(ui(22.))
                    .bg(rgb(c.well))
                    .children(boxes),
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
            .gap(ui(14.))
            .px(ui(18.))
            .py(ui(16.))
            .rounded(ui(22.))
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
                    .text_size(ui(14.))
                    .line_height(relative(1.5))
                    .child(
                        div()
                            .text_size(ui(15.))
                            .font_weight(FontWeight::BOLD)
                            .child(tr!("Some settings are managed by your organization")),
                    )
                    .when_some(by, |d, by| d.child(by))
                    .when_some(wrong, |d, wrong| d.child(wrong)),
            )
            .when_some(url, |d, url| {
                d.child(
                    button("policy-url", tr!("Get help"), Fill::Raised, Size::Medium)
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
            .gap(ui(3.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(ui(6.))
                    .px(ui(12.))
                    .py(ui(4.))
                    .rounded_full()
                    .bg(rgb(c.surface))
                    .text_size(ui(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .whitespace_nowrap()
                    .child(icon("lock", 13., c.text))
                    .child(value),
            )
            .child(
                div()
                    .font_family(mono)
                    .text_size(ui(11.))
                    .text_color(rgb(c.muted))
                    .child(row.key.clone()),
            )
    }
}

impl Render for Security {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mono = self.mono.get_or_insert_with(|| mono_font(window)).clone();
        let policy = crate::managed::policy(cx).clone();
        let locks_mark = policy.locks_mark();
        let trusted = self.trusted_keys(cx);
        let checksums = Self::checksums(cx);
        let audit_log = self.audit_log(&policy, cx);
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
            return section()
                .gap(ui(20.))
                .child(rows(open))
                .child(audit_log)
                .child(trusted)
                .child(checksums);
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
            .gap(ui(16.))
            .child(Self::banner(&policy))
            .child(rows(fixed.chain(open)))
            .child(audit_log)
            .child(trusted)
            .child(checksums)
    }
}
