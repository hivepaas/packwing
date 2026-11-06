//! Files whose data is damaged, while extracting: asked about in a terminal, or
//! done with as `--on-damaged` says. Files whose names this system cannot take:
//! asked about once, all together, in a terminal, or done with as
//! `--on-bad-name` says. Symbolic links that lead outside the folder: asked
//! about once, all together, in a terminal, or done with as `--links-outside`
//! says. Then each one said, and written to `--report`. A name that leads
//! outside, or a limit, always stops.

use std::io::{BufRead as _, Write as _};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use clap::{Args, ValueEnum};
use packwing_core::{
    BadName, Control, Damage, OnDamaged, OnLinksOutside, OnUnnamable, OutsideLink, Renaming, size,
};
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
    /// Write each file damaged, left out or renamed, and why, to this file: a line
    /// each, tab-separated.
    #[arg(long, value_name = "FILE")]
    pub report: Option<PathBuf>,
}

/// What `extract` does with files whose names this system cannot take. Not
/// `convert`'s: an archive holds any name.
#[derive(Args)]
pub struct Naming {
    /// When files' names cannot be used on this system (as a:b.txt on Windows,
    /// or README beside readme on macOS): ask, stop, skip them, or rename them.
    /// By default asked in a terminal, and skipped otherwise.
    #[arg(long, value_name = "WHAT", value_enum)]
    pub on_bad_name: Option<NameChoice>,
    /// Take only names every system can use: refuse Windows' characters and
    /// device names, and names differing only in case, on Linux too.
    #[arg(long)]
    pub portable_names: bool,
    /// When symbolic links lead outside the folder extracted to (to an absolute
    /// path, as ~/snap's to ~/.config, or climbing out): ask, stop, skip them,
    /// or create them as they are, for an archive you trust. By default asked
    /// in a terminal, and skipped otherwise. On Windows links are written as
    /// files holding their targets, whatever this says.
    #[arg(long, value_name = "WHAT", value_enum)]
    pub links_outside: Option<LinkChoice>,
}

impl Naming {
    /// What `--links-outside` says, or its default here.
    fn link_choice(&self, cx: &Context) -> Result<LinkChoice, Exit> {
        Ok(match self.links_outside {
            Some(LinkChoice::Ask) if !cx.can_ask => {
                return Err(Exit::Usage(
                    "--links-outside ask needs a terminal: give stop, skip or create".into(),
                ));
            }
            Some(choice) => choice,
            None if cx.can_ask => LinkChoice::Ask,
            None => LinkChoice::Skip,
        })
    }

    /// What becomes of the links that lead outside when nothing asks.
    pub fn links_outside(&self, cx: &Context) -> Result<OnLinksOutside, Exit> {
        Ok(match self.link_choice(cx)? {
            LinkChoice::Stop => OnLinksOutside::Stop,
            LinkChoice::Create => OnLinksOutside::Create,
            LinkChoice::Skip | LinkChoice::Ask => OnLinksOutside::Skip,
        })
    }

    /// `control`, which asks or does as was said.
    pub fn control(&self, mut control: Control, cx: &Context) -> Result<Control, Exit> {
        if self.link_choice(cx)? == LinkChoice::Ask {
            control.on_links_outside = Some(Arc::new(ask_links));
        }
        let names = match self.on_bad_name {
            Some(NameChoice::Ask) if !cx.can_ask => {
                return Err(Exit::Usage(
                    "--on-bad-name ask needs a terminal: give stop, skip or rename".into(),
                ));
            }
            Some(choice) => choice,
            None if cx.can_ask => NameChoice::Ask,
            None => NameChoice::Skip,
        };
        control.on_unnamable = match names {
            NameChoice::Stop => None,
            NameChoice::Skip => Some(Arc::new(|_: &[BadName]| OnUnnamable::Skip)),
            NameChoice::Rename => Some(Arc::new(|_: &[BadName]| OnUnnamable::Rename)),
            NameChoice::Ask => Some(Arc::new(ask_names)),
        };
        Ok(control)
    }
}

/// What to do with files whose names this system cannot take.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum NameChoice {
    /// Ask, once for all of them: in a terminal only.
    Ask,
    /// Stop, and write nothing.
    Stop,
    /// Leave them out, and extract the rest.
    Skip,
    /// Give them names this system takes, as a_b.txt or readme (2).
    Rename,
}

/// What to do with symbolic links that lead outside the folder extracted to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum LinkChoice {
    /// Ask, once for all of them: in a terminal only.
    Ask,
    /// Stop, and write nothing.
    Stop,
    /// Leave them out, and extract the rest.
    Skip,
    /// Create them as the archive has them.
    Create,
}

/// What came of an extraction, to be said.
#[derive(Default)]
pub struct Outcome {
    pub damaged: Vec<Damage>,
    pub left_out: Vec<BadName>,
    pub renamed: Vec<Renaming>,
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

    /// Says each file damaged, left out or renamed, and writes them to the
    /// report: an error, as [`Exit::Damaged`], if any was damaged or left out.
    pub fn finish(&self, outcome: &Outcome) -> Result<(), Exit> {
        let Outcome {
            damaged,
            left_out,
            renamed,
        } = outcome;
        if let Some(report) = &self.report {
            let lines: String = damaged
                .iter()
                .map(|d| format!("{}\t{}\t{}\n", d.path, kept(d), d.why))
                .chain(
                    left_out
                        .iter()
                        .map(|b| format!("{}\tleft out\t{}\n", b.path, b.why)),
                )
                .chain(
                    renamed
                        .iter()
                        .map(|r| format!("{}\trenamed {}\t{}\n", r.path, r.to, r.why)),
                )
                .collect();
            std::fs::write(report, lines)
                .map_err(|e| Exit::Failed(format!("cannot write {}: {e}", report.display())))?;
        }
        for r in renamed {
            eprintln!("pkw: renamed: {} to {} ({})", r.path, r.to, r.why);
        }
        for b in left_out {
            eprintln!("pkw: left out: {} ({})", b.path, b.why);
        }
        for d in damaged {
            eprintln!("pkw: damaged: {}: {} ({})", d.path, d.why, kept(d));
        }
        let mut said = Vec::new();
        match damaged.len() {
            0 => {}
            1 => said.push("1 file was damaged".to_owned()),
            n => said.push(format!("{n} files were damaged")),
        }
        match left_out.len() {
            0 => {}
            1 => said.push("1 file was left out".to_owned()),
            n => said.push(format!("{n} files were left out")),
        }
        if said.is_empty() {
            return Ok(());
        }
        Err(Exit::Damaged(format!(
            "{}; the rest is done",
            said.join(", ")
        )))
    }
}

/// What was done with `damage`, in a few words.
fn kept(damage: &Damage) -> String {
    match damage.kept {
        Some(bytes) => format!("kept {}", size(bytes)),
        None => "left out".into(),
    }
}

/// `left_out`, as JSON says them.
pub fn left_out_json(left_out: &[BadName]) -> Value {
    left_out
        .iter()
        .map(|b| json!({ "path": b.path, "why": b.why }))
        .collect()
}

/// `renamed`, as JSON says them.
pub fn renamed_json(renamed: &[Renaming]) -> Value {
    renamed
        .iter()
        .map(|r| json!({ "path": r.path, "to": r.to, "why": r.why }))
        .collect()
}

/// Asks once what to do with the files of `bad`, whose names this system cannot
/// take.
fn ask_names(bad: &[BadName]) -> OnUnnamable {
    let mut err = std::io::stderr().lock();
    // Over the progress line, if one is shown.
    let _ = writeln!(
        err,
        "\r\x1b[Kpkw: {} can't be extracted under {} names here:",
        if bad.len() == 1 {
            "1 file".to_owned()
        } else {
            format!("{} files", bad.len())
        },
        if bad.len() == 1 { "its" } else { "their" }
    );
    for b in bad {
        let _ = writeln!(err, "  {} ({})", b.path, b.why);
    }
    loop {
        let _ = write!(
            err,
            "  Stop (q), skip them (s), or rename them (r)? Nothing is written yet. [q] "
        );
        let _ = err.flush();
        let mut line = String::new();
        if !matches!(std::io::stdin().lock().read_line(&mut line), Ok(n) if n > 0) {
            let _ = writeln!(err);
            return OnUnnamable::Stop;
        }
        if let Some(chosen) = name_answer(line.trim()) {
            return chosen;
        }
    }
}

/// Asks once what to do with `links`, which lead outside the folder.
fn ask_links(links: &[OutsideLink]) -> OnLinksOutside {
    let mut err = std::io::stderr().lock();
    // Over the progress line, if one is shown.
    let _ = writeln!(
        err,
        "\r\x1b[Kpkw: {} outside the folder:",
        if links.len() == 1 {
            "1 link leads".to_owned()
        } else {
            format!("{} links lead", links.len())
        }
    );
    for link in links {
        let _ = writeln!(err, "  {} -> {}", link.path, link.target);
    }
    loop {
        let _ = write!(
            err,
            "  Skip them (s), create them as they are (c), or stop (q)? Nothing is written yet. [s] "
        );
        let _ = err.flush();
        let mut line = String::new();
        if !matches!(std::io::stdin().lock().read_line(&mut line), Ok(n) if n > 0) {
            let _ = writeln!(err);
            return OnLinksOutside::Stop;
        }
        if let Some(chosen) = link_answer(line.trim()) {
            return chosen;
        }
    }
}

/// What `typed` answers about links that lead outside, if it does.
fn link_answer(typed: &str) -> Option<OnLinksOutside> {
    Some(match typed {
        "" | "s" | "S" => OnLinksOutside::Skip,
        "c" | "C" => OnLinksOutside::Create,
        "q" | "Q" => OnLinksOutside::Stop,
        _ => return None,
    })
}

/// What `typed` answers about bad names, if it does.
fn name_answer(typed: &str) -> Option<OnUnnamable> {
    Some(match typed {
        "" | "q" | "Q" => OnUnnamable::Stop,
        "s" | "S" => OnUnnamable::Skip,
        "r" | "R" => OnUnnamable::Rename,
        _ => return None,
    })
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
        assert_eq!(name_answer(""), Some(OnUnnamable::Stop));
        assert_eq!(name_answer("r"), Some(OnUnnamable::Rename));
        assert_eq!(name_answer("S"), Some(OnUnnamable::Skip));
        assert_eq!(name_answer("k"), None);
        assert_eq!(link_answer(""), Some(OnLinksOutside::Skip));
        assert_eq!(link_answer("C"), Some(OnLinksOutside::Create));
        assert_eq!(link_answer("q"), Some(OnLinksOutside::Stop));
        assert_eq!(link_answer("r"), None);
    }
}
