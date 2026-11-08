//! `pkw audit`: the audit log's records, whether its chain holds, and how it is
//! kept (docs/audit.md).

use std::path::PathBuf;

use packwing_core::audit;
use serde_json::{Map, Value, json};

use crate::commands::print_json;
use crate::{AuditCommand, Context, Exit};

pub fn run(what: AuditCommand, cx: &mut Context) -> Result<(), Exit> {
    match what {
        AuditCommand::List { since, csv } => list(since.as_deref(), csv, cx),
        AuditCommand::Verify => verify(cx),
        AuditCommand::Status => status(cx),
    }
}

/// Where the log is: where it is kept now, or would be.
fn folder() -> Result<PathBuf, Exit> {
    audit::config()
        .map(|c| c.folder)
        .or_else(|| audit::folder(packwing_policy::current()))
        .ok_or_else(|| Exit::Failed("this system has no folder for the audit log".into()))
}

/// The records there, or none where there is no log yet.
fn records(folder: &std::path::Path, since: Option<&str>) -> Result<Vec<Map<String, Value>>, Exit> {
    if !folder.is_dir() {
        return Ok(Vec::new());
    }
    audit::read(folder, since).map_err(|e| {
        Exit::Failed(format!(
            "cannot read the audit log in {}: {e}",
            folder.display()
        ))
    })
}

/// What a record is of: the archive or file it worked on, or where to.
fn subject(r: &Map<String, Value>) -> String {
    [
        "source",
        "archive",
        "file",
        "from",
        "destination",
        "folder",
        "part",
    ]
    .iter()
    .find_map(|k| r.get(*k).and_then(Value::as_str))
    .unwrap_or_default()
    .to_owned()
}

fn text(r: &Map<String, Value>, key: &str) -> String {
    match r.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

fn list(since: Option<&str>, csv: bool, cx: &Context) -> Result<(), Exit> {
    if csv && cx.json {
        return Err(Exit::Usage("--csv or --json, not both".into()));
    }
    if let Some(day) = since
        && !(day.len() == 10 && day.as_bytes()[4] == b'-' && day.as_bytes()[7] == b'-')
    {
        return Err(Exit::Usage(format!("not a day: {day} (as 2026-11-08)")));
    }
    let records = records(&folder()?, since)?;
    if cx.json {
        print_json(&Value::Array(
            records.into_iter().map(Value::Object).collect(),
        ));
        return Ok(());
    }
    const COLUMNS: [&str; 9] = [
        "time", "action", "phase", "result", "user", "host", "app", "job", "error",
    ];
    if csv {
        let cell = |s: String| {
            if s.contains([',', '"', '\n']) {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s
            }
        };
        println!("{},subject", COLUMNS.join(","));
        for r in &records {
            let mut row: Vec<String> = COLUMNS.iter().map(|k| cell(text(r, k))).collect();
            row.push(cell(subject(r)));
            println!("{}", row.join(","));
        }
        return Ok(());
    }
    if records.is_empty() {
        if !cx.quiet {
            eprintln!("The audit log has no records.");
        }
        return Ok(());
    }
    // An end says what its start was of.
    let mut of = std::collections::HashMap::new();
    for r in &records {
        let what = subject(r);
        let what = if what.is_empty() {
            of.get(&text(r, "job")).cloned().unwrap_or_default()
        } else {
            of.insert(text(r, "job"), what.clone());
            what
        };
        let result = match text(r, "phase").as_str() {
            "start" => "started".to_owned(),
            _ => text(r, "result"),
        };
        let error = text(r, "error");
        println!(
            "{}  {:<11} {:<9} {}{}",
            text(r, "time"),
            text(r, "action"),
            result,
            what,
            if error.is_empty() {
                String::new()
            } else {
                format!(": {error}")
            }
        );
    }
    Ok(())
}

fn verify(cx: &Context) -> Result<(), Exit> {
    let folder = folder()?;
    if !folder.is_dir() {
        if !cx.quiet {
            eprintln!("The audit log has no records.");
        }
        return Ok(());
    }
    let report = audit::verify(&folder).map_err(|e| {
        Exit::Failed(format!(
            "cannot read the audit log in {}: {e}",
            folder.display()
        ))
    })?;
    if cx.json {
        print_json(&json!({
            "folder": folder.display().to_string(),
            "files": report.files,
            "records": report.records,
            "anchor": report.anchor,
            "anchor_explained": report.anchor_explained,
            "breaks": report.breaks.iter().map(|b| json!({
                "file": b.file, "line": b.line, "why": b.why,
            })).collect::<Vec<_>>(),
        }));
    } else if !cx.quiet {
        for b in &report.breaks {
            println!("{} line {}: {}", b.file, b.line, b.why);
        }
        if !report.anchor_explained {
            println!(
                "The oldest record follows one that is gone, and no record says it was \
                 taken away as past its time."
            );
        }
    }
    if report.breaks.is_empty() && report.anchor_explained {
        if !cx.json && !cx.quiet {
            eprintln!(
                "{} records in {} files: the chain holds.",
                report.records, report.files
            );
        }
        Ok(())
    } else {
        Err(Exit::Failed("the audit log's chain does not hold".into()))
    }
}

fn status(cx: &Context) -> Result<(), Exit> {
    let policy = packwing_policy::current();
    let config = audit::config();
    let by = if policy.locks_audit_log() {
        "the organization's policy"
    } else {
        "the app's settings (Settings › Security)"
    };
    let folder = folder()?;
    if cx.json {
        print_json(&json!({
            "kept": config.is_some(),
            "decided_by": if policy.locks_audit_log() { "policy" } else { "user" },
            "folder": folder.display().to_string(),
            "retention_days": config.as_ref().map(|c| c.retention_days),
            "max_bytes": config.as_ref().map(|c| c.max_bytes),
            "names_entries": config.as_ref().map(|c| c.entries),
            "last_failure": audit::last_failure(),
        }));
        return Ok(());
    }
    match &config {
        None => println!("The audit log is not kept, as {by} says."),
        Some(c) => {
            println!(
                "The audit log is kept, as {by} says, in {}.",
                folder.display()
            );
            match c.retention_days {
                0 => println!("Records are kept for good."),
                days => println!("Records are kept for {days} days."),
            }
            if c.max_bytes > 0 {
                println!("It takes at most {} MiB.", c.max_bytes >> 20);
            }
        }
    }
    if let Some(why) = audit::last_failure() {
        println!("The last record could not be written: {why}");
    }
    Ok(())
}
