//! A file's signature, looked at in the background, and what it says to the user.
//! Checking takes no license: only signing does.

use std::sync::Arc;

use gpui::{App, Context, Task, prelude::*};
use packwing_core::{Cancel, Control, Error, Signed, Storage, check_signature, look_at_signature};
use packwing_i18n::tr;

use crate::colors::colors;
use crate::prefs;

/// A look going on. Dropped, it stops.
pub struct Looking {
    cancel: Cancel,
    _task: Task<()>,
}

impl Drop for Looking {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

/// Looks at the signature of the file at `path`. With `read_all`, one by a key
/// trusted is checked against the whole file; without, it says it was not.
pub fn start<T: 'static>(
    storage: Arc<dyn Storage>,
    path: Vec<String>,
    read_all: bool,
    cx: &mut Context<T>,
    done: impl FnOnce(&mut T, Signed, &mut Context<T>) + 'static,
) -> Looking {
    let trusted = prefs::prefs(cx).trusted_public_keys();
    let cancel = Cancel::new();
    let control = Control {
        progress: None,
        cancel: Some(cancel.clone()),
    };
    let work = cx.background_spawn(async move {
        if !read_all {
            return Ok(look_at_signature(storage.as_ref(), &path, &trusted));
        }
        check_signature(storage.as_ref(), &path, &trusted, &control)
    });
    let task = cx.spawn(async move |this, cx| {
        let result: Result<Signed, Error> = work.await;
        if let Ok(signed) = result {
            let _ = this.update(cx, |this, cx| done(this, signed, cx));
        }
    });
    Looking {
        cancel,
        _task: task,
    }
}

/// How a signature's state looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    /// It proves what it should.
    Good,
    /// Nothing is wrong, and nothing is proved yet.
    Plain,
    /// It proves nothing.
    Warn,
    /// The file is not what was signed.
    Bad,
}

impl Standing {
    /// The ground and the ink of a chip saying it.
    pub fn colors(self) -> (u32, u32) {
        let c = colors();
        match self {
            Standing::Good => (c.accent, c.on_accent),
            Standing::Plain => (c.surface, c.on_tonal),
            Standing::Warn => (c.warn_bg, c.warn_text),
            Standing::Bad => (c.bad_bg, c.bad_text),
        }
    }
}

/// What `signed` says, in a line, and how it looks; none if there is no signature.
pub fn said(signed: &Signed, cx: &App) -> Option<(String, Standing)> {
    let kept = prefs::prefs(cx);
    let signer = |fingerprint: &str| {
        kept.trusted_key_name(fingerprint)
            .unwrap_or_else(|| fingerprint.to_owned())
    };
    Some(match signed {
        Signed::No => return None,
        Signed::Good { fingerprint } => (
            tr!("Signed by {signer}", signer = signer(fingerprint)),
            Standing::Good,
        ),
        Signed::NotChecked { fingerprint } => (
            tr!(
                "Signed by {signer}, not checked yet",
                signer = signer(fingerprint)
            ),
            Standing::Plain,
        ),
        Signed::Stranger { fingerprint } => (
            tr!(
                "Signed by a key you do not trust: {fingerprint}",
                fingerprint = fingerprint
            ),
            Standing::Warn,
        ),
        Signed::Mismatch { .. } => (tr!("It does not match its signature"), Standing::Bad),
        Signed::Unreadable(_) => (tr!("Its signature cannot be read"), Standing::Warn),
        Signed::Failed(_) => (tr!("Its signature could not be checked"), Standing::Warn),
    })
}

/// What a look going on says.
pub fn looking_words() -> String {
    tr!("Checking its signature…")
}
