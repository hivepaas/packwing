//! Signing a file with a key: one chosen, or made here. The signature goes beside
//! the file. What only a license allows says so, rather than hiding the way there.

use packwing_i18n::tr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{
    App, Context, Entity, EventEmitter, Focusable as _, PathPromptOptions, Render, Subscription,
    Window, div, prelude::*, px, rgb,
};
use packwing_core::{
    KeyRefused, SigningKey, Storage, is_part_name, make_signing_key, read_signing_key,
    signature_name,
};

use super::password::read_key_file;
use super::{DialogEvent, Job, Signing, Verb, footer, header};
use crate::colors::colors;
use crate::input::{Changed, TextInput};
use crate::prefs;
use crate::widgets::{Fill, Size, Tone, button, icon_button, legend, note, tonal_tint};

/// What is under way that the buttons wait for.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Waiting {
    Nothing,
    OpeningKey,
    MakingKey,
}

pub struct SignDialog {
    storage: Arc<dyn Storage>,
    path: Vec<String>,
    name: String,
    /// The key file chosen, and the key once it is open.
    key_file: Option<PathBuf>,
    key: Option<SigningKey>,
    /// The key file chosen needs its passphrase.
    locked: bool,
    passphrase: Entity<TextInput>,
    /// A new key is being asked for, and its passphrase, typed twice.
    making: bool,
    new_passphrase: Entity<TextInput>,
    repeated: Entity<TextInput>,
    waiting: Waiting,
    problem: Option<String>,
    _typing: Vec<Subscription>,
}

impl EventEmitter<DialogEvent> for SignDialog {}

/// A field for a secret, as the password dialog draws one.
fn secret_field(label: String, field: Entity<TextInput>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(legend(label))
        .child(
            div()
                .flex()
                .items_center()
                .h(px(52.))
                .px(px(18.))
                .rounded(px(16.))
                .bg(rgb(colors().field))
                .text_size(px(16.))
                .child(field),
        )
}

/// What a key file is called in the list of keys trusted.
fn key_name(file: &Path) -> String {
    file.file_stem()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

impl SignDialog {
    /// For the file at `path`, starting with the key used last if it opens as it is.
    pub fn new(
        storage: Arc<dyn Storage>,
        path: Vec<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> SignDialog {
        let name = path.last().cloned().unwrap_or_default();
        let field = |placeholder: String, cx: &mut Context<Self>| {
            cx.new(|cx| TextInput::new(placeholder, cx).masked())
        };
        let passphrase = field(tr!("The key file’s passphrase"), cx);
        let new_passphrase = field(tr!("A passphrase for the new key"), cx);
        let repeated = field(tr!("The same passphrase again"), cx);
        let typing = [&passphrase, &new_passphrase, &repeated]
            .map(|f| cx.subscribe(f, |_, _, _: &Changed, cx| cx.notify()))
            .into();
        window.focus(&passphrase.focus_handle(cx), cx);
        let mut dialog = SignDialog {
            storage,
            path,
            name,
            key_file: None,
            key: None,
            locked: false,
            passphrase,
            making: false,
            new_passphrase,
            repeated,
            waiting: Waiting::Nothing,
            problem: None,
            _typing: typing,
        };
        if dialog.refused(cx).is_none()
            && let Some(file) = prefs::prefs(cx).signing_key
        {
            dialog.open_key(file, None, false, cx);
        }
        dialog
    }

    /// Why this file cannot be signed here, if it cannot.
    fn refused(&self, cx: &App) -> Option<String> {
        if !crate::licensing::plan(cx).may_sign() {
            return Some(tr!(
                "Signing takes a Pro or Enterprise license: Settings › License. Checking \
                 signatures is free, on every plan."
            ));
        }
        is_part_name(&self.name).then(|| {
            tr!(
                "This is one part of a file cut in several. A signature covers one file, and \
                 would seem to cover them all, so packwing does not sign parts."
            )
        })
    }

    /// Shows the new key's fields, as the screenshots draw them.
    #[cfg(all(test, feature = "test-hooks"))]
    pub fn ask_new_key_for_test(&mut self, cx: &mut Context<Self>) {
        self.making = true;
        cx.notify();
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn use_key_for_test(&mut self, key: SigningKey, cx: &mut Context<Self>) {
        self.key = Some(key);
        cx.notify();
    }

    #[cfg(all(test, feature = "test-hooks"))]
    pub fn start_for_test(&mut self, cx: &mut Context<Self>) {
        self.start(cx);
    }

    /// Asks for a key file.
    fn pick_key(&mut self, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(tr!("Choose").into()),
        });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(mut paths))) = picked.await
                && let Some(file) = paths.pop()
            {
                let _ = this.update(cx, |this, cx| {
                    this.passphrase.update(cx, |f, cx| f.set_text("", 0..0, cx));
                    this.open_key(file, None, false, cx);
                });
            }
        })
        .detach();
    }

    /// Opens the key at `file` in the background: with `then_sign`, signing follows.
    /// One that needs a passphrase asks for it; the one used last, if it has gone,
    /// is simply not offered.
    fn open_key(
        &mut self,
        file: PathBuf,
        passphrase: Option<Vec<u8>>,
        then_sign: bool,
        cx: &mut Context<Self>,
    ) {
        self.key = None;
        self.key_file = Some(file.clone());
        self.locked = false;
        self.problem = None;
        self.waiting = Waiting::OpeningKey;
        let reading = file.clone();
        let opening = cx.background_spawn(async move {
            let mut contents = read_key_file(&reading).map_err(KeyRefused::Not)?;
            let key = read_signing_key(&contents, passphrase.as_deref());
            contents.fill(0);
            key
        });
        cx.spawn(async move |this, cx| {
            let opened = opening.await;
            let _ = this.update(cx, |this, cx| {
                this.waiting = Waiting::Nothing;
                match opened {
                    Ok(key) => {
                        this.key = Some(key);
                        if then_sign {
                            this.start(cx);
                        }
                    }
                    Err(KeyRefused::NeedsPassphrase) => this.locked = true,
                    Err(KeyRefused::WrongPassphrase) => {
                        this.locked = true;
                        this.problem = Some(tr!("That passphrase does not open this key."));
                    }
                    Err(KeyRefused::Not(why)) => {
                        this.key_file = None;
                        this.problem = Some(tr!(
                            "This is not a key packwing can sign with: {why}.",
                            why = why
                        ));
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    /// Why the new key's passphrase will not do, if it will not.
    fn new_passphrase_problem(&self, cx: &App) -> Option<String> {
        let typed = self.new_passphrase.read(cx).text();
        if typed.is_empty() {
            return None;
        }
        if let Err(refused) = crate::managed::policy(cx).check_password(typed.as_bytes()) {
            return Some(refused.to_string());
        }
        let again = self.repeated.read(cx).text();
        (!again.is_empty() && again != typed).then(|| tr!("The two passphrases are not the same."))
    }

    fn can_make(&self, cx: &App) -> bool {
        let typed = self.new_passphrase.read(cx).text();
        !typed.is_empty()
            && typed == self.repeated.read(cx).text()
            && self.new_passphrase_problem(cx).is_none()
            && self.waiting == Waiting::Nothing
    }

    /// Asks for a folder, and makes the new key there.
    fn make_key(&mut self, cx: &mut Context<Self>) {
        if !self.can_make(cx) {
            return;
        }
        let passphrase = self.new_passphrase.read(cx).text().as_bytes().to_vec();
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(tr!("Keep the key here").into()),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(mut paths))) = picked.await else {
                return;
            };
            let Some(folder) = paths.pop() else { return };
            let Ok(making) = this.update(cx, |this, cx| {
                this.waiting = Waiting::MakingKey;
                this.problem = None;
                cx.notify();
                cx.background_spawn(async move { make_signing_key(&folder, &passphrase) })
            }) else {
                return;
            };
            let made = making.await;
            let _ = this.update(cx, |this, cx| {
                this.waiting = Waiting::Nothing;
                match made {
                    Ok((key, file)) => {
                        this.key = Some(key);
                        this.key_file = Some(file);
                        this.locked = false;
                        this.making = false;
                    }
                    Err(e) => this.problem = Some(e.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn can_start(&self, cx: &App) -> bool {
        self.refused(cx).is_none()
            && !self.making
            && self.waiting == Waiting::Nothing
            && (self.key.is_some() || (self.locked && !self.passphrase.read(cx).text().is_empty()))
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        if !self.can_start(cx) {
            return;
        }
        let Some(key) = self.key.clone() else {
            if let Some(file) = self.key_file.clone() {
                let typed = self.passphrase.read(cx).text().as_bytes().to_vec();
                self.open_key(file, Some(typed), true, cx);
            }
            return;
        };
        let file = self.key_file.clone();
        // Whoever holds the key trusts it: what they sign shows as signed by them.
        prefs::change(cx, |p| {
            let name = file.as_deref().map_or_else(String::new, key_name);
            p.trust(name, key.public());
            p.signing_key = file;
        });
        let mut job = Job::shown(
            Verb::Sign,
            self.name.clone().into(),
            Some(signature_name(&self.name).into()),
            1,
            0,
        );
        job.sign = Some(Signing(self.storage.clone(), self.path.clone(), key));
        cx.emit(DialogEvent::Start(job));
    }

    fn choosing(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let chosen = match (&self.key, &self.key_file) {
            (Some(key), file) => format!(
                "{} · {}",
                file.as_deref().map_or_else(String::new, key_name),
                key.public().fingerprint()
            ),
            (None, Some(file)) => key_name(file),
            (None, None) => tr!("No key chosen"),
        };
        div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(legend(tr!("Key")))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(160.))
                                    .text_size(px(14.))
                                    .text_color(rgb(if self.key.is_some() {
                                        c.text
                                    } else {
                                        c.muted
                                    }))
                                    .child(chosen),
                            )
                            .child(
                                button("sign-choose", tr!("Choose…"), Fill::Outline, Size::Small)
                                    .on_click(cx.listener(|this, _, _, cx| this.pick_key(cx))),
                            )
                            .child(
                                button("sign-new-key", tr!("New key…"), Fill::Outline, Size::Small)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.making = true;
                                        this.problem = None;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .when(self.locked, |d| {
                d.child(secret_field(tr!("Passphrase"), self.passphrase.clone()))
            })
    }

    fn asking_new_key(&self, cx: &App) -> impl IntoElement {
        let c = colors();
        div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .child(secret_field(tr!("Passphrase"), self.new_passphrase.clone()))
            .child(secret_field(tr!("Again"), self.repeated.clone()))
            .children(
                self.new_passphrase_problem(cx)
                    .map(|why| note(Tone::Warn).child(why)),
            )
            .child(div().text_size(px(13.)).text_color(rgb(c.muted)).child(tr!(
                "A new key holds Ed25519 and ML-DSA-65 together, so that a signature \
                         stands even if one of them falls. Only packwing and hivepack check \
                         these: for ssh-keygen, sign with an SSH Ed25519 key. There is no way \
                         back into a key whose passphrase is lost."
            )))
    }
}

impl Render for SignDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = colors();
        let close = icon_button("close", "x", 44.)
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DialogEvent::Close)));
        let refused = self.refused(cx);
        let far = self.storage.local_path(&[]).is_none();
        let can_start = self.can_start(cx);
        let can_make = self.can_make(cx);
        let making = self.making;
        let body = match &refused {
            Some(why) => note(Tone::Warn).child(why.clone()).into_any_element(),
            None if making => self.asking_new_key(cx).into_any_element(),
            None => self.choosing(cx).into_any_element(),
        };
        let told = if far {
            tr!(
                "The signature goes beside it as {signature}, in place of one that is there. \
                 All of the file passes through this computer, and your provider may charge for \
                 what is read from it.",
                signature = signature_name(&self.name)
            )
        } else {
            tr!(
                "The signature goes beside it as {signature}, in place of one that is there.",
                signature = signature_name(&self.name)
            )
        };
        super::card()
            .w(px(560.))
            .child(header(
                "shield",
                tonal_tint(),
                tr!("Sign {name}", name = self.name),
                tr!("Anyone can check it, with the key’s public half"),
                Some(close),
            ))
            .child(body)
            .children(self.problem.clone().map(|why| note(Tone::Warn).child(why)))
            .when(refused.is_none() && !making, |d| {
                d.child(
                    div()
                        .text_size(px(13.))
                        .text_color(rgb(c.muted))
                        .child(told),
                )
            })
            .child(
                footer()
                    .justify_end()
                    .child(
                        button("cancel", tr!("Cancel"), Fill::Text, Size::Medium).on_click(
                            cx.listener(|this, _, _, cx| {
                                if this.making {
                                    this.making = false;
                                    this.problem = None;
                                    cx.notify();
                                } else {
                                    cx.emit(DialogEvent::Close);
                                }
                            }),
                        ),
                    )
                    .when(making && refused.is_none(), |d| {
                        d.child(
                            button("make-key", tr!("Make the key…"), Fill::Accent, Size::Medium)
                                .px(px(28.))
                                .when(!can_make, |d| d.opacity(0.5))
                                .when(can_make, |d| {
                                    d.on_click(cx.listener(|this, _, _, cx| this.make_key(cx)))
                                }),
                        )
                    })
                    .when(!making || refused.is_some(), |d| {
                        d.child(
                            button("sign-start", tr!("Sign"), Fill::Accent, Size::Medium)
                                .px(px(28.))
                                .when(!can_start, |d| d.opacity(0.5))
                                .when(can_start, |d| {
                                    d.on_click(cx.listener(|this, _, _, cx| this.start(cx)))
                                }),
                        )
                    }),
            )
    }
}
