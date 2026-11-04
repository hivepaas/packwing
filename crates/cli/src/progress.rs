//! How far a job is, on standard error, when a person watches it: one line,
//! written over, a few times a second, and gone when it ends.

use std::io::Write as _;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use packwing_core::{Control, Progress, size};

/// The line, while it is shown.
pub struct Bar {
    shown: bool,
}

/// A job's control, saying how far it is as `what` when `show`.
pub fn control(what: &str, show: bool) -> (Control, Bar) {
    if !show {
        return (Control::default(), Bar { shown: false });
    }
    let what = what.to_owned();
    let last = Mutex::new(Instant::now() - Duration::from_secs(1));
    let progress = move |p: &Progress| {
        let Ok(mut last) = last.lock() else {
            return;
        };
        if last.elapsed() < Duration::from_millis(120) {
            return;
        }
        *last = Instant::now();
        let line = if let Some(percent) = p.bytes.saturating_mul(100).checked_div(p.total_bytes) {
            format!(
                "{what} {percent:>3}%  {} of {}",
                size(p.bytes),
                size(p.total_bytes)
            )
        } else {
            format!("{what} {}", size(p.bytes))
        };
        let mut err = std::io::stderr().lock();
        let _ = write!(err, "\r\x1b[K{line}");
        let _ = err.flush();
    };
    let control = Control {
        progress: Some(Arc::new(progress)),
        ..Control::default()
    };
    (control, Bar { shown: true })
}

impl Drop for Bar {
    fn drop(&mut self) {
        if self.shown {
            let mut err = std::io::stderr().lock();
            let _ = write!(err, "\r\x1b[K");
            let _ = err.flush();
        }
    }
}
