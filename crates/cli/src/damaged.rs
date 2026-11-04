//! Files whose data is damaged, while extracting: asked about in a terminal, or
//! done with as `--on-damaged` says; then each one said, and written to
//! `--report`. A name, a link or a limit refused is no damage: it always stops.

use std::io::{BufRead as _, Write as _};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use clap::{Args, ValueEnum};
use packwing_core::{Control, Damage, OnDamaged, size};
use serde_json::{Value, json};

use crate::{Context, Exit};

/// What to do with a file whose data is damaged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Choice {
    /// Ask, for each one: in a terminal only.
    Ask,
    /// Stop, as with any error.
    Stop,
    /// Leave it out, and go on.
    Skip,
    /// Keep what was read of it, and go on.
    Keep,
}

/// What `extract` and `convert` do with files whose data is damaged.
#[derive(Args)]
pub struct Damaging {
    /// When a file's data is damaged (a check that fails, data that does not
    /// decompress or is cut short): ask, stop, skip it, or keep what was read of
    /// it. By default asked in a terminal, and stop otherwise.
    #[arg(long, value_name = "WHAT", value_enum)]
    pub on_damaged: Option<Choice>,
    /// Write each file whose data was damaged, and what was wrong with it, to this
    /// file: a line each, tab-separated.
    #[arg(long, value_name = "FILE")]
    pub report: Option<PathBuf>,
}

impl Damaging {
    /// `control`, which asks or does as was said.
    pub fn control(&self, mut control: Control, cx: &Context) -> Result<Control, Exit> {
        let choice = match self.on_damaged {
            Some(Choice::Ask) if !cx.can_ask => {
                return Err(Exit::Usage(
                    "--on-damaged ask needs a terminal: give stop, skip or keep".into(),
                ));
            }
            Some(choice) => choice,
            None if cx.can_ask => Choice::Ask,
            None => Choice::Stop,
        };
        control.on_damaged = match choice {
            Choice::Stop => None,
            Choice::Skip => Some(Arc::new(|_: &Damage| OnDamaged::Skip)),
            Choice::Keep => Some(Arc::new(|_: &Damage| OnDamaged::Keep)),
            Choice::Ask => {
                // What was answered for all the rest, once it is.
                let for_all = Mutex::new(None);
                Some(Arc::new(move |damage: &Damage| {
                    let Ok(mut for_all) = for_all.lock() else {
                        return OnDamaged::Stop;
                    };
                    if let Some(chosen) = *for_all {
                        return chosen;
                    }
                    let (chosen, all) = ask(damage);
                    if all {
                        *for_all = Some(chosen);
                    }
                    chosen
                }))
            }
        };
        Ok(control)
    }

    /// Says each of `damaged`, and writes them to the report: an error, as
    /// [`Exit::Damaged`], if there are any.
    pub fn finish(&self, damaged: &[Damage]) -> Result<(), Exit> {
        if let Some(report) = &self.report {
            let lines: String = damaged
                .iter()
                .map(|d| format!("{}\t{}\t{}\n", d.path, kept(d), d.why))
                .collect();
            std::fs::write(report, lines)
                .map_err(|e| Exit::Failed(format!("cannot write {}: {e}", report.display())))?;
        }
        if damaged.is_empty() {
            return Ok(());
        }
        for d in damaged {
            eprintln!("pkw: damaged: {}: {} ({})", d.path, d.why, kept(d));
        }
        let n = damaged.len();
        Err(Exit::Damaged(if n == 1 {
            "1 file was damaged; the rest is done".into()
        } else {
            format!("{n} files were damaged; the rest is done")
        }))
    }
}

/// What was done with `damage`, in a few words.
fn kept(damage: &Damage) -> String {
    match damage.kept {
        Some(bytes) => format!("kept {}", size(bytes)),
        None => "left out".into(),
    }
}

/// `damaged`, as JSON says them.
pub fn as_json(damaged: &[Damage]) -> Value {
    damaged
        .iter()
        .map(|d| json!({ "path": d.path, "error": d.why, "kept": d.kept }))
        .collect()
}

/// Asks what to do with `damage`, and whether for all the rest too.
fn ask(damage: &Damage) -> (OnDamaged, bool) {
    let mut err = std::io::stderr().lock();
    // Over the progress line, if one is shown.
    let _ = writeln!(
        err,
        "\r\x1b[Kpkw: {} is damaged: {}",
        damage.path, damage.why
    );
    loop {
        let _ = write!(
            err,
            "  Stop (q), skip it (s), or keep what was read of it (k)? S or K for all the rest. [q] "
        );
        let _ = err.flush();
        let mut line = String::new();
        // Nothing more to read is no answer: stop.
        if !matches!(std::io::stdin().lock().read_line(&mut line), Ok(n) if n > 0) {
            let _ = writeln!(err);
            return (OnDamaged::Stop, false);
        }
        match answer(line.trim()) {
            Some(chosen) => return chosen,
            None => continue,
        }
    }
}

/// What `typed` answers, if it does.
fn answer(typed: &str) -> Option<(OnDamaged, bool)> {
    Some(match typed {
        "" | "q" | "Q" => (OnDamaged::Stop, false),
        "s" => (OnDamaged::Skip, false),
        "k" => (OnDamaged::Keep, false),
        "S" => (OnDamaged::Skip, true),
        "K" => (OnDamaged::Keep, true),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_typed_is_an_answer_or_asked_again() {
        assert_eq!(answer(""), Some((OnDamaged::Stop, false)));
        assert_eq!(answer("s"), Some((OnDamaged::Skip, false)));
        assert_eq!(answer("K"), Some((OnDamaged::Keep, true)));
        assert_eq!(answer("x"), None);
    }
}
