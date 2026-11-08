//! Ctrl-C, and the system asking pkw to end (SIGTERM): a job running is
//! stopped as the app's Cancel stops it, and what it was making is removed;
//! another Ctrl-C, or one while no job runs, ends pkw at once, as before.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use packwing_core::Cancel;

/// What stops the job running, if one is.
static JOB: Mutex<Option<Cancel>> = Mutex::new(None);

/// A job was asked to stop already.
static ASKED: AtomicBool = AtomicBool::new(false);

/// The exit status of pkw ended by a signal: 128 and SIGINT's 2, as shells
/// say it.
pub const STATUS: i32 = 130;

/// Listens for Ctrl-C and SIGTERM. If that cannot be, they end pkw at once,
/// as they did.
pub fn listen() {
    let _ = ctrlc::set_handler(heard);
}

fn heard() {
    let job = JOB.lock().ok().and_then(|job| job.clone());
    match job {
        Some(cancel) if !ASKED.swap(true, Ordering::Relaxed) => {
            cancel.cancel();
            eprint!("\r\x1b[Kpkw: stopping; Ctrl-C again ends it at once\n");
        }
        _ => std::process::exit(STATUS),
    }
}

/// What stops the job that starts, which Ctrl-C sets until `Job` is dropped.
pub fn job() -> (Cancel, Job) {
    let cancel = Cancel::new();
    if let Ok(mut job) = JOB.lock() {
        *job = Some(cancel.clone());
    }
    (cancel, Job)
}

/// The job running: Ctrl-C ends pkw again once it is dropped.
pub struct Job;

impl Drop for Job {
    fn drop(&mut self) {
        if let Ok(mut job) = JOB.lock() {
            *job = None;
        }
    }
}
