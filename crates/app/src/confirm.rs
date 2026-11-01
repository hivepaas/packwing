//! Asking that it is the user at this computer, as the system does, before a
//! storage that asks for it opens: a light guard for a screen left unlocked.
//! Nothing is kept or opened by it: the answer is a yes or a no. From
//! shellwing's `presence` crate.
//!
//! - **macOS**: LocalAuthentication, Touch ID or the login password. Unlike
//!   the vault's Touch ID item, it needs no signed build.
//! - **Windows**: Windows Hello (the face, a fingerprint or the PIN), else
//!   the system's own password prompt.
//! - **Linux**: the login password, typed in packwing and checked by PAM's
//!   `login` service, as screen lockers check it: for the user running
//!   packwing only, with no rights of its own.
//!
//! Calls block until the user answers: never on the UI thread.

/// How the user is asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum How {
    /// The system asks, in its own dialog: [`ask`].
    System,
    /// packwing asks for the password, and the system checks it:
    /// [`check_password`].
    Password,
}

/// How this system asks; None where it cannot.
pub fn how() -> Option<How> {
    if cfg!(any(target_os = "macos", windows)) {
        Some(How::System)
    } else if cfg!(target_os = "linux") {
        Some(How::Password)
    } else {
        None
    }
}

/// The system asks the user: true when it is them, false when they
/// cancelled or could not prove it. macOS says `reason` after "packwing is
/// trying to"; Windows shows `description`.
pub fn ask(reason: &str, description: &str) -> Result<bool, String> {
    #[cfg(any(target_os = "macos", windows))]
    return system::ask(reason, description);
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        let _ = (reason, description);
        Err("this system has no prompt of its own for it".into())
    }
}

/// Whether `password` is the user's own login password. `said` gets what
/// the system says on the way, such as "Place your finger on the reader"
/// where a fingerprint is asked for first.
pub fn check_password(password: &str, said: &dyn Fn(String)) -> Result<bool, String> {
    #[cfg(target_os = "linux")]
    return pam::check(password, said);
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (password, said);
        Err("this system checks it in its own prompt".into())
    }
}

#[cfg(any(target_os = "macos", windows))]
mod system {
    use robius_authentication::{
        AndroidText, BiometricStrength, Context, Error, PolicyBuilder, Text, WindowsText,
    };

    pub fn ask(reason: &str, description: &str) -> Result<bool, String> {
        // Touch ID or Windows Hello, else the account's password.
        let policy = PolicyBuilder::new()
            .biometrics(Some(BiometricStrength::Strong))
            .password(true)
            .build()
            .ok_or("this system cannot ask that it is you")?;
        let text = Text {
            android: AndroidText {
                title: "packwing",
                subtitle: None,
                description: None,
            },
            apple: reason,
            windows: WindowsText::new_truncated("packwing", description),
        };
        let (tx, rx) = std::sync::mpsc::channel();
        Context::new(())
            .authenticate(text, &policy, move |answer| {
                let _ = tx.send(answer);
            })
            .map_err(|e| e.to_string())?;
        match rx.recv() {
            Ok(Ok(())) => Ok(true),
            Ok(Err(
                Error::UserCanceled
                | Error::AppCanceled
                | Error::SystemCanceled
                | Error::UserFallback
                | Error::Authentication
                | Error::Exhausted
                | Error::Timeout,
            )) => Ok(false),
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => Err("the system gave no answer".into()),
        }
    }
}

#[cfg(target_os = "linux")]
mod pam {
    use std::cell::Cell;
    use std::ffi::{OsStr, OsString};

    use nonstick::{
        AuthnFlags, ConversationAdapter, ErrorCode, Result as PamResult, Transaction,
        TransactionBuilder,
    };

    /// The service whose rules check it: there on every distribution, as the
    /// login's own.
    const SERVICE: &str = "login";

    /// Answers PAM with the password typed, once, and passes on what it says.
    struct Typed<'a> {
        user: &'a str,
        password: &'a str,
        said: &'a dyn Fn(String),
        answered: Cell<bool>,
    }

    impl ConversationAdapter for Typed<'_> {
        fn prompt(&self, _request: impl AsRef<OsStr>) -> PamResult<OsString> {
            Ok(OsString::from(self.user))
        }

        fn masked_prompt(&self, _request: impl AsRef<OsStr>) -> PamResult<OsString> {
            // A second secret (a one-time code) is not what was typed.
            if self.answered.replace(true) {
                return Err(ErrorCode::ConversationError);
            }
            Ok(OsString::from(self.password))
        }

        fn error_msg(&self, message: impl AsRef<OsStr>) {
            (self.said)(message.as_ref().to_string_lossy().into_owned());
        }

        fn info_msg(&self, message: impl AsRef<OsStr>) {
            (self.said)(message.as_ref().to_string_lossy().into_owned());
        }
    }

    pub fn check(password: &str, said: &dyn Fn(String)) -> Result<bool, String> {
        let user = nix::unistd::User::from_uid(nix::unistd::getuid())
            .ok()
            .flatten()
            .map(|u| u.name)
            .ok_or("the user running packwing is not known")?;
        let typed = Typed {
            user: &user,
            password,
            said,
            answered: Cell::new(false),
        };
        let mut txn = TransactionBuilder::new_with_service(SERVICE)
            .username(&user)
            .build(typed.into_conversation())
            .map_err(|e| e.to_string())?;
        match txn.authenticate(AuthnFlags::empty()) {
            Ok(()) => Ok(true),
            Err(
                ErrorCode::AuthenticationError
                | ErrorCode::MaxTries
                | ErrorCode::CredentialsInsufficient
                | ErrorCode::ConversationError,
            ) => Ok(false),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// What says it is the user: the system ([`System`]), or the tests'.
pub trait Confirmer: Send + Sync {
    fn how(&self) -> Option<How>;
    fn ask(&self, reason: &str, description: &str) -> Result<bool, String>;
    fn check_password(&self, password: &str) -> Result<bool, String>;
}

/// The system's own.
pub struct System;

impl Confirmer for System {
    fn how(&self) -> Option<How> {
        how()
    }

    fn ask(&self, reason: &str, description: &str) -> Result<bool, String> {
        ask(reason, description)
    }

    fn check_password(&self, password: &str) -> Result<bool, String> {
        check_password(password, &|_| {})
    }
}

/// The confirmer the tests put in, in place of the system's.
#[derive(Clone)]
pub struct ConfirmWith(pub std::sync::Arc<dyn Confirmer>);

impl gpui::Global for ConfirmWith {}

pub fn confirmer(cx: &gpui::App) -> std::sync::Arc<dyn Confirmer> {
    cx.try_global::<ConfirmWith>()
        .map_or_else(|| std::sync::Arc::new(System) as _, |c| c.0.clone())
}
