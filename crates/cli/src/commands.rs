//! What each command does, through packwing-core, and what it writes.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use packwing_core::{
    Algorithm, Blocks, COMPRESSED_EXTENSIONS, Entry, ExtractOptions, Filter, Frames, Keep, Kind,
    Listing, LocalFolder, NewArchive, OnExisting, Owners, Source, Times, Transfer, add_with,
    algorithms_of, came_from_internet, checksum_line, checksums, convert, create, damages, delete,
    extract_selected, extract_with, has_signature, join_parts, left_out, list, read_entry, rename,
    renamings, save_checksum, signature_name, size, split_file, stem, test, transfer,
};
use serde_json::{Value, json};

use crate::damaged::{Damaging, Naming, Outcome};
use crate::names::{
    date, entry_path, format_named, format_of, format_words, level_named, size_typed,
};
use crate::places::Place;
use crate::signing::{keygen_cmd, sign_cmd, verify_cmd};
use crate::{Backup, Command, Context, Exit, Making, Restore};

pub fn run(command: Command, cx: &mut Context) -> Result<(), Exit> {
    match command {
        Command::List {
            archive,
            paths,
            long,
        } => list_cmd(&archive, &paths, long, cx),
        Command::Extract {
            archive,
            files,
            output,
            here,
            overwrite,
            limit,
            no_mark,
            damaging,
            naming,
            restore,
        } => {
            let (owners, xattrs) = restored(&restore)?;
            let how = ExtractOptions {
                overwrite,
                limit: Some(limit_of(&limit)?),
                keep_mark: keep_mark(no_mark),
                portable_names: naming.portable_names,
                links_outside: naming.links_outside(cx)?,
                owners,
                xattrs,
            };
            extract_cmd(&archive, &files, output, here, &how, &damaging, &naming, cx)
        }
        Command::Test { archive } => test_cmd(&archive, cx),
        Command::Cat {
            archive,
            file,
            limit,
        } => cat_cmd(&archive, &file, &limit, cx),
        Command::Info { archive } => info_cmd(&archive, cx),
        Command::Create {
            archive,
            paths,
            make,
            backup,
        } => {
            let (archive, mut new) = making(&archive, make, cx)?;
            new.keep = kept_for_backup(&backup, new.kind)?;
            new.times = times_of(
                backup.mtime.as_deref(),
                std::env::var("SOURCE_DATE_EPOCH").ok(),
            )?;
            write_archive(&archive, &paths, &new, cx)
        }
        Command::Convert {
            from,
            to,
            include,
            exclude,
            make,
            damaging,
        } => {
            let filter = Filter::new(&include, &exclude)?;
            convert_cmd(&from, &to, make, &filter, &damaging, cx)
        }
        Command::Split { file, size } => split_cmd(&file, &size, cx),
        Command::Join { part } => join_cmd(&part, cx),
        Command::Keygen { output } => keygen_cmd(&output, cx),
        Command::Sign { files, by } => sign_cmd(&files, &by, cx),
        Command::Verify { files, trust, only } => verify_cmd(&files, &trust, only, cx),
        Command::Passphrase { min } => passphrase_cmd(min, cx),
        Command::Storages => storages_cmd(cx),
        // Done by main, before anything is read.
        Command::Completions { .. } | Command::Man { .. } => Ok(()),
        Command::Copy {
            places,
            on_existing,
            no_mark,
        } => transfer_cmd(&places, &on_existing, false, keep_mark(no_mark), cx),
        Command::Move {
            places,
            on_existing,
            no_mark,
        } => transfer_cmd(&places, &on_existing, true, keep_mark(no_mark), cx),
        Command::Add {
            archive,
            paths,
            into,
            on_existing,
        } => add_cmd(&archive, &paths, &into, &on_existing, cx),
        Command::Delete { archive, files } => delete_cmd(&archive, &files, cx),
        Command::Rename {
            archive,
            file,
            name,
        } => rename_cmd(&archive, &file, &name, cx),
        Command::Audit { what } => crate::audit::run(what, cx),
        Command::Checksum {
            files,
            algorithm,
            check,
            save,
        } => match check {
            Some(sums) => check_sums(&sums, cx),
            None => checksum_cmd(&files, &algorithm, save, cx),
        },
    }
}

/// `n` files, said as a person says it.
fn file_count(n: u64) -> String {
    if n == 1 {
        "1 file".into()
    } else {
        format!("{n} files")
    }
}

/// The name a person knows an archive by.
fn named(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// What is typed, as an archive's files' paths.
fn picked(files: &[String]) -> Vec<Vec<String>> {
    files.iter().map(|f| entry_path(f)).collect()
}

fn limit_of(typed: &str) -> Result<u64, Exit> {
    size_typed(typed).ok_or_else(|| Exit::Usage(format!("not a size: {typed} (as 50GB)")))
}

fn listed(place: &Place, cx: &mut Context) -> Result<Listing, Exit> {
    let source = place.source()?;
    Ok(cx
        .secrets
        .unlocking(&place.name(), |keys| list(source.clone(), keys))?)
}

pub(crate) fn print_json(value: &Value) {
    println!("{value:#}");
}

fn list_cmd(typed: &str, paths: &[String], long: bool, cx: &mut Context) -> Result<(), Exit> {
    let place = Place::of(typed)?;
    if place.is_folder()? {
        return list_folder(&place, long, cx);
    }
    let listing = listed(&place, cx)?;
    let under: Vec<Vec<String>> = picked(paths);
    let shown: Vec<&Entry> = listing
        .entries
        .iter()
        .filter(|e| under.is_empty() || under.iter().any(|u| e.path.starts_with(u)))
        .collect();
    if cx.json {
        let entries: Vec<Value> = shown
            .iter()
            .map(|e| {
                json!({
                    "path": e.path.join("/"),
                    "dir": e.dir,
                    "size": e.size,
                    "modified": e.modified,
                })
            })
            .collect();
        print_json(&json!(entries));
        return Ok(());
    }
    let rows = shown
        .iter()
        .map(|e| (e.path.join("/"), e.dir, e.size, e.modified));
    print_rows(rows, long);
    Ok(())
}

/// A folder's files and folders, here or on a storage, folders first.
fn list_folder(place: &Place, long: bool, cx: &Context) -> Result<(), Exit> {
    let failed = |e: std::io::Error| Exit::Failed(format!("{}: {e}", place.shown));
    let mut lister = place.storage.list(&place.path).map_err(failed)?;
    let mut items = Vec::new();
    while !lister.done() {
        items.extend(lister.next(1000).map_err(failed)?);
    }
    items.sort_by(|a, b| b.dir.cmp(&a.dir).then_with(|| a.name.cmp(&b.name)));
    if cx.json {
        let rows: Vec<Value> = items
            .iter()
            .map(|i| {
                json!({
                    "name": i.name,
                    "dir": i.dir,
                    "link": i.link,
                    "size": i.size,
                    "modified": i.modified,
                })
            })
            .collect();
        print_json(&json!(rows));
        return Ok(());
    }
    print_rows(
        items
            .iter()
            .map(|i| (i.name.clone(), i.dir, i.size, i.modified)),
        long,
    );
    Ok(())
}

/// Lines of what is listed: a folder's with a `/`, and with `long` its size and date.
fn print_rows(rows: impl Iterator<Item = (String, bool, u64, Option<i64>)>, long: bool) {
    let mut out = std::io::stdout().lock();
    for (path, dir, bytes, modified) in rows {
        let path = path + if dir { "/" } else { "" };
        let line = if long {
            let when = modified.map(date).unwrap_or_else(|| " ".repeat(16));
            let bytes = if dir {
                String::new()
            } else {
                bytes.to_string()
            };
            format!("{bytes:>12}  {when}  {path}")
        } else {
            path
        };
        if writeln!(out, "{line}").is_err() {
            // A pipe closed early, as `| head` does.
            break;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn extract_cmd(
    archive: &str,
    files: &[String],
    output: Option<PathBuf>,
    here: bool,
    how: &ExtractOptions,
    damaging: &Damaging,
    naming: &Naming,
    cx: &mut Context,
) -> Result<(), Exit> {
    let place = Place::of(archive)?;
    let source = place.source()?;
    let name = place.name();
    let destination = match (output, here) {
        (Some(dir), _) => dir,
        (None, true) => PathBuf::from("."),
        // As the menu's Extract to folder: a folder named after it, here.
        (None, false) => PathBuf::from(stem(&name)),
    };
    let picked = picked(files);
    let (control, bar) = crate::progress::control("Extracting", cx.progress);
    let control = naming.control(damaging.control(control, cx)?, cx)?;
    let summary = cx.secrets.unlocking(&name, |keys| {
        if picked.is_empty() {
            extract_with(source.clone(), &destination, keys, how, &control)
        } else {
            extract_selected(source.clone(), &destination, &picked, keys, how, &control)
        }
    })?;
    drop(bar);
    let outcome = outcome_of(&summary);
    if cx.json {
        print_json(&json!({
            "to": destination.display().to_string(),
            "files": summary.files,
            "directories": summary.directories,
            "links": summary.links,
            "bytes": summary.bytes,
            "damaged": crate::damaged::as_json(&outcome.damaged),
            "skipped": crate::damaged::left_out_json(&outcome.left_out),
            "renamed": crate::damaged::renamed_json(&outcome.renamed),
            "owners_left": summary.owners_left,
        }));
    } else if !cx.quiet {
        eprintln!(
            "{}, {}, into {}",
            file_count(summary.files),
            size(summary.bytes),
            destination.display()
        );
        if summary.owners_left > 0 {
            eprintln!(
                "{} files and folders stay yours: the owners the archive gives them are not \
                 yours to give (only an administrator gives files to others)",
                summary.owners_left
            );
        }
    }
    damaging.finish(&outcome)
}

/// Whose files are, extracted, and whether their extended attributes come back,
/// as `restore` asks: on Linux and macOS, where hivepack gives them.
fn restored(restore: &Restore) -> Result<(Owners, bool), Exit> {
    let owners = if restore.numeric_owner {
        Owners::ByNumber
    } else if restore.same_owner {
        Owners::ByName
    } else {
        Owners::Ignore
    };
    let here = cfg!(any(target_os = "linux", target_os = "macos"));
    if !here && owners != Owners::Ignore {
        return Err(Exit::Usage(
            "--same-owner and --numeric-owner give Unix owners, on Linux and macOS only".into(),
        ));
    }
    if !here && restore.xattrs {
        return Err(Exit::Usage(
            "--xattrs gives extended attributes on Linux and macOS only".into(),
        ));
    }
    Ok((owners, restore.xattrs))
}

/// What came of extracting, as `summary` says.
fn outcome_of(summary: &packwing_core::Summary) -> Outcome {
    Outcome {
        damaged: damages(summary),
        left_out: left_out(summary),
        renamed: renamings(summary),
    }
}

fn test_cmd(archive: &str, cx: &mut Context) -> Result<(), Exit> {
    let place = Place::of(archive)?;
    let source = place.source()?;
    let name = place.name();
    let (control, bar) = crate::progress::control("Testing", cx.progress);
    let summary = cx
        .secrets
        .unlocking(&name, |keys| test(source.clone(), keys, &control))?;
    drop(bar);
    if cx.json {
        print_json(&json!({ "ok": true, "files": summary.files, "bytes": summary.bytes }));
    } else if !cx.quiet {
        println!(
            "{name}: OK, {}, {}",
            file_count(summary.files),
            size(summary.bytes)
        );
    }
    Ok(())
}

fn cat_cmd(archive: &str, file: &str, limit: &str, cx: &mut Context) -> Result<(), Exit> {
    let place = Place::of(archive)?;
    let source = place.source()?;
    let name = place.name();
    let entry = entry_path(file);
    let most = limit_of(limit)?;
    let control = packwing_core::Control::default();
    let bytes = cx.secrets.unlocking(&name, |keys| {
        read_entry(source.clone(), &entry, keys, most, &control)
    })?;
    let mut out = std::io::stdout().lock();
    out.write_all(&bytes)
        .and_then(|()| out.flush())
        .or_else(|e| {
            if e.kind() == std::io::ErrorKind::BrokenPipe {
                Ok(())
            } else {
                Err(e)
            }
        })
        .map_err(|e| Exit::Failed(e.to_string()))
}

fn info_cmd(archive: &str, cx: &mut Context) -> Result<(), Exit> {
    let place = Place::of(archive)?;
    let l = listed(&place, cx)?;
    let layers: Vec<String> = l.layers.iter().map(|x| format!("{x:?}")).collect();
    let format = std::iter::once(format!("{:?}", l.container))
        .chain(layers.iter().rev().cloned())
        .collect::<Vec<_>>()
        .join(" in ");
    let ratio = (l.unpacked > 0).then(|| l.packed as f64 / l.unpacked as f64);
    // Its signature beside it, wherever it is; the mark, a file here's alone.
    let signed = match (&place.here, place.split()) {
        (Some(here), _) => has_signature(here),
        (None, Some((folder, name))) => place
            .storage
            .stat(folder, &signature_name(name))
            .is_ok_and(|i| i.is_some()),
        (None, None) => false,
    };
    let from_internet = place
        .here
        .as_deref()
        .is_some_and(|h| came_from_internet(&Source::from(h)));
    if cx.json {
        print_json(&json!({
            "name": l.name,
            "container": format!("{:?}", l.container),
            "layers": layers,
            "encryption": l.encryption,
            "packed": l.packed,
            "unpacked": l.unpacked,
            "files": l.files,
            "entries": l.entries.len(),
            "signature": signed,
            "from_internet": from_internet,
        }));
        return Ok(());
    }
    let row = |k: &str, v: String| println!("{k:<14}{v}");
    row("Name", l.name.clone());
    row("Format", format);
    row(
        "Encrypted",
        l.encryption.clone().unwrap_or_else(|| "no".into()),
    );
    row("Files", l.files.to_string());
    row("Size", size(l.packed));
    row("Unpacked", size(l.unpacked));
    if let Some(r) = ratio {
        row("Ratio", format!("{:.0}%", r * 100.));
    }
    row(
        "Signature",
        if signed { "beside it" } else { "none" }.into(),
    );
    row(
        "From Internet",
        if from_internet { "yes" } else { "no" }.into(),
    );
    Ok(())
}

/// What `m` asks for, of an archive to make at `archive`: the path it is made
/// at, an age file's `.age` added as the app adds it, and how it is made.
fn making(archive: &Path, m: Making, cx: &Context) -> Result<(PathBuf, NewArchive), Exit> {
    let kind: Kind = match &m.format {
        Some(word) => format_named(word).ok_or_else(|| {
            Exit::Usage(format!("no format {word}: it is one of {}", format_words()))
        })?,
        None => format_of(&named(archive)).ok_or_else(|| {
            Exit::Usage(format!(
                "its extension says no format pkw makes: give one with --format ({})",
                format_words()
            ))
        })?,
    };
    let level = level_named(&m.level).ok_or_else(|| {
        Exit::Usage(format!(
            "no level {}: store, lowest, low, medium, high or highest",
            m.level
        ))
    })?;
    let password = if m.password {
        Some(match cx.secrets.password() {
            Some(p) => p,
            None if cx.secrets.can_ask() => ask_new_secret("New password")?,
            None => {
                return Err(Exit::Usage(
                    "--password needs one: from a terminal, --password-file, --password-stdin or PACKWING_PASSWORD".into(),
                ));
            }
        })
    } else {
        None
    };
    let part_size = m
        .split
        .as_deref()
        .map(|s| size_typed(s).ok_or_else(|| Exit::Usage(format!("not a size: {s} (as 25MB)"))))
        .transpose()?;
    let frames = frames_named(m.frames.as_deref(), kind)?;
    let blocks = blocks_named(m.blocks.as_deref(), kind)?;
    let new = NewArchive {
        kind,
        level,
        frames,
        blocks,
        password,
        age_keys: m.encrypt_to,
        whole_with_age: m.age,
        hide_names: m.hide_names,
        // As the app does by default: what only a Mac uses is left out.
        skip_mac_files: !m.keep_mac_files,
        stored_extensions: if m.compress_all {
            Vec::new()
        } else {
            COMPRESSED_EXTENSIONS
                .iter()
                .map(|e| (*e).to_owned())
                .collect()
        },
        part_size,
        ..NewArchive::default()
    };
    // Encrypted whole with age, it is an age file, and named so, as the app names it.
    let sealed = !new.age_keys.is_empty()
        || (new.password.is_some()
            && (new.whole_with_age || !matches!(kind, Kind::Zip | Kind::SevenZip)));
    let archive = if sealed && !named(archive).to_ascii_lowercase().ends_with(".age") {
        let mut with = archive.as_os_str().to_owned();
        with.push(".age");
        PathBuf::from(with)
    } else {
        archive.to_path_buf()
    };
    // Even one thread taking more memory than is free is said before it starts.
    if !cx.quiet
        && let Some(warning) = packwing_core::memory_warning(&new)
    {
        eprintln!("pkw: {warning}");
    }
    Ok((archive, new))
}

/// How `--frames` cuts a zstd stream: as the size says if it is not given, and
/// only for a format of zstd's.
fn frames_named(word: Option<&str>, kind: Kind) -> Result<Frames, Exit> {
    let Some(word) = word else {
        return Ok(Frames::Auto);
    };
    if !matches!(kind, Kind::TarZst | Kind::Zst) {
        return Err(Exit::Usage(
            "--frames is for zst and tar.zst, which are cut into frames".into(),
        ));
    }
    match word.to_ascii_lowercase().as_str() {
        "one" | "1" => Ok(Frames::One),
        "several" | "many" => Ok(Frames::Several),
        _ => Err(Exit::Usage(format!("no --frames {word}: one or several"))),
    }
}

/// How `--blocks` cuts LZMA2: large, the format's own, if it is not given,
/// and only for xz, tar.xz and 7z.
fn blocks_named(word: Option<&str>, kind: Kind) -> Result<Blocks, Exit> {
    let Some(word) = word else {
        return Ok(Blocks::Large);
    };
    if !matches!(kind, Kind::TarXz | Kind::Xz | Kind::SevenZip) {
        return Err(Exit::Usage(
            "--blocks is for xz, tar.xz and 7z, which are cut into blocks".into(),
        ));
    }
    match word.to_ascii_lowercase().as_str() {
        "large" => Ok(Blocks::Large),
        "small" => Ok(Blocks::Small),
        _ => Err(Exit::Usage(format!("no --blocks {word}: large or small"))),
    }
}

/// What `backup` asks a new archive of `kind` to keep: tar alone has a place for it.
fn kept_for_backup(backup: &Backup, kind: Kind) -> Result<Keep, Exit> {
    let keep = Keep {
        owners: backup.owners,
        xattrs: backup.xattrs,
    };
    if keep.any() && !kind.is_tar() {
        return Err(Exit::Usage(
            "--owners and --xattrs keep them in a tar archive: tar, tar.zst, tar.gz, tar.xz, tar.bz2 or tar.lz4".into(),
        ));
    }
    if keep.owners && !cfg!(unix) {
        return Err(Exit::Usage(
            "--owners keeps Unix owners, which this system has not".into(),
        ));
    }
    if keep.xattrs && !cfg!(any(target_os = "linux", target_os = "macos")) {
        return Err(Exit::Usage(
            "--xattrs keeps extended attributes on Linux and macOS only".into(),
        ));
    }
    Ok(keep)
}

/// The times a new archive gives its files: all `mtime` (`--mtime`), else none
/// later than `source_date_epoch`, the environment's `SOURCE_DATE_EPOCH`, as
/// reproducible builds ask, else as they are.
fn times_of(mtime: Option<&str>, source_date_epoch: Option<String>) -> Result<Times, Exit> {
    if let Some(when) = mtime {
        return time_typed(when).map(Times::All).ok_or_else(|| {
            Exit::Usage(format!(
                "not a time: {when} (as @1700000000, 2026-11-07 or 2026-11-07T10:30:00)"
            ))
        });
    }
    match source_date_epoch.as_deref().map(str::trim) {
        None | Some("") => Ok(Times::AsTheyAre),
        Some(secs) => secs.parse().map(Times::AtMost).map_err(|_| {
            Exit::Usage(format!(
                "SOURCE_DATE_EPOCH is not seconds since 1970: {secs}"
            ))
        }),
    }
}

/// `@SECONDS` since 1970, `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM:SS` (a space for the
/// `T`, and a `Z` after, taken too), in UTC: seconds since 1970.
fn time_typed(when: &str) -> Option<i64> {
    let when = when.trim();
    if let Some(secs) = when.strip_prefix('@') {
        return secs.parse().ok();
    }
    let when = when.strip_suffix('Z').unwrap_or(when);
    let (date, time) = match when.split_once(['T', ' ']) {
        Some((date, time)) => (date, Some(time)),
        None => (when, None),
    };
    let number = |s: &str, digits: usize| {
        (s.len() == digits && s.bytes().all(|b| b.is_ascii_digit()))
            .then(|| s.parse::<i64>().ok())
            .flatten()
    };
    let mut parts = date.split('-');
    let (year, month, day) = (
        number(parts.next()?, 4)?,
        number(parts.next()?, 2)?,
        number(parts.next()?, 2)?,
    );
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let (hour, minute, second) = match time {
        None => (0, 0, 0),
        Some(time) => {
            let mut parts = time.split(':');
            let hms = (
                number(parts.next()?, 2)?,
                number(parts.next()?, 2)?,
                number(parts.next()?, 2)?,
            );
            if parts.next().is_some() || hms.0 > 23 || hms.1 > 59 || hms.2 > 60 {
                return None;
            }
            hms
        }
    };
    // Howard Hinnant's days_from_civil.
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + hour * 3_600 + minute * 60 + second)
}

/// Makes the archive `new` at `archive`, of `paths`, and says what it made.
fn write_archive(
    archive: &Path,
    paths: &[PathBuf],
    new: &NewArchive,
    cx: &Context,
) -> Result<(), Exit> {
    let (mut control, bar) = crate::progress::control("Compressing", cx.progress);
    // What another program removed while it ran is left out, and said.
    let gone = Arc::new(Mutex::new(Vec::new()));
    let noted = gone.clone();
    control.on_gone = Some(Arc::new(move |path: &Path| {
        noted
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(path.display().to_string());
    }));
    let made = create(paths, archive, new, &control)?;
    drop(bar);
    let gone = std::mem::take(&mut *gone.lock().unwrap_or_else(PoisonError::into_inner));
    let bytes = std::fs::metadata(&made)
        .map(|m| m.len())
        .unwrap_or_default();
    if cx.json {
        print_json(&json!({ "made": made.display().to_string(), "size": bytes, "gone": gone }));
        return Ok(());
    }
    for path in &gone {
        eprintln!("pkw: left out, gone by the time it was read: {path}");
    }
    if !cx.quiet {
        println!("{} ({})", made.display(), size(bytes));
    }
    Ok(())
}

/// A new secret, `what`, typed twice, the same.
pub fn ask_new_secret(what: &str) -> Result<Vec<u8>, Exit> {
    let typed = |what: &str| {
        rpassword::prompt_password(what).map_err(|e| Exit::Failed(format!("cannot ask: {e}")))
    };
    let first = typed(&format!("{what}: "))?;
    if first.is_empty() {
        return Err(Exit::Usage(format!("no {} typed", what.to_lowercase())));
    }
    if typed("Again: ")? != first {
        return Err(Exit::Usage("the two passwords are not the same".into()));
    }
    Ok(first.into_bytes())
}

fn add_cmd(
    archive: &Path,
    paths: &[PathBuf],
    into: &str,
    on_existing: &str,
    cx: &mut Context,
) -> Result<(), Exit> {
    let on_existing = on_existing_named(on_existing)?;
    let folder = entry_path(into);
    let stored: Vec<String> = COMPRESSED_EXTENSIONS
        .iter()
        .map(|e| (*e).to_owned())
        .collect();
    let (control, bar) = crate::progress::control("Adding", cx.progress);
    cx.secrets.unlocking(&named(archive), |keys| {
        add_with(
            paths,
            archive,
            &folder,
            on_existing,
            &stored,
            keys,
            &control,
        )
    })?;
    drop(bar);
    done(cx, format!("added to {}", archive.display()))
}

fn delete_cmd(archive: &Path, files: &[String], cx: &mut Context) -> Result<(), Exit> {
    let picked = picked(files);
    let (control, bar) = crate::progress::control("Deleting", cx.progress);
    cx.secrets.unlocking(&named(archive), |keys| {
        delete(archive, &picked, keys, &control)
    })?;
    drop(bar);
    done(cx, format!("deleted from {}", archive.display()))
}

fn rename_cmd(archive: &Path, file: &str, name: &str, cx: &mut Context) -> Result<(), Exit> {
    let from = entry_path(file);
    cx.secrets
        .unlocking(&named(archive), |keys| rename(archive, &from, name, keys))?;
    done(cx, format!("renamed in {}", archive.display()))
}

/// Says a change was made: in JSON, that it was.
/// Whether what arrives gets the mark of where it came from, as `--no-mark`
/// asks: the core keeps to the organization's policy over it, and pkw says
/// so when the policy keeps the mark asked away.
fn keep_mark(no_mark: bool) -> bool {
    let policy = packwing_policy::current();
    if no_mark
        && cfg!(any(windows, target_os = "macos"))
        && policy.locks_mark()
        && policy.keeps_mark(false)
    {
        eprintln!(
            "pkw: the organization's policy keeps the mark of files from the Internet: --no-mark is not followed"
        );
    }
    !no_mark
}

/// What `--on-existing` says to do with a name taken.
fn on_existing_named(typed: &str) -> Result<OnExisting, Exit> {
    match typed {
        "stop" => Ok(OnExisting::Stop),
        "replace" => Ok(OnExisting::Replace),
        "skip" => Ok(OnExisting::Skip),
        other => Err(Exit::Usage(format!(
            "--on-existing is stop, replace or skip, not {other}"
        ))),
    }
}

fn storages_cmd(cx: &Context) -> Result<(), Exit> {
    let kept = crate::settings::storages(&crate::settings::text(), &crate::settings::kinds());
    if cx.json {
        let rows: Vec<Value> = kept
            .iter()
            .map(|k| {
                json!({
                    "name": k.name,
                    "kind": k.kind,
                    "where": k.describe,
                    "asks_first": k.asks_first,
                    "opens": k.storage.is_some() && !k.asks_first,
                })
            })
            .collect();
        print_json(&json!(rows));
        return Ok(());
    }
    if kept.is_empty() {
        if !cx.quiet {
            eprintln!("No storages are added: add them in packwing, in Storages.");
        }
        return Ok(());
    }
    let wide = kept
        .iter()
        .map(|k| k.name.chars().count())
        .max()
        .unwrap_or(0);
    let kind_wide = kept
        .iter()
        .map(|k| k.kind.chars().count())
        .max()
        .unwrap_or(0);
    for k in &kept {
        let note = if k.asks_first {
            "  (asks for this computer's password: open it in packwing)"
        } else if k.storage.is_none() && cfg!(feature = "remote") {
            "  (pkw cannot open this kind)"
        } else if k.storage.is_none() {
            "  (this pkw is built without S3, SFTP and WebDAV)"
        } else {
            ""
        };
        println!(
            "{:wide$}  {:kind_wide$}  {}{note}",
            k.name, k.kind, k.describe
        );
    }
    Ok(())
}

/// `copy` and `move`: files and folders, here or on storages, into a folder.
fn transfer_cmd(
    places: &[String],
    on_existing: &str,
    moving: bool,
    keep_mark: bool,
    cx: &Context,
) -> Result<(), Exit> {
    let on_existing = on_existing_named(on_existing)?;
    let (to, from) = places.split_last().expect("clap asks for two at least");
    let to = Place::of(to)?;
    if !to.is_folder()? {
        return Err(Exit::Usage(format!(
            "{} is not a folder: what is copied or moved goes into a folder",
            to.shown
        )));
    }
    let mut all = packwing_core::Transferred::default();
    for typed in from {
        let place = Place::of(typed)?;
        let Some((folder, name)) = place.split() else {
            return Err(Exit::Usage(format!(
                "{typed} is a storage's top: name what is in it, as {typed}NAME"
            )));
        };
        if place.stat()?.is_none() {
            return Err(Exit::Failed(format!("{typed} is not there")));
        }
        let verb = if moving { "Moving" } else { "Copying" };
        let (control, bar) = crate::progress::control(&format!("{verb} {name}"), cx.progress);
        let did = transfer(
            &Transfer {
                from: place.storage.clone(),
                from_folder: folder.to_vec(),
                names: vec![name.to_owned()],
                to: to.storage.clone(),
                to_folder: to.path.clone(),
                moving,
                on_existing,
                keep_mark,
            },
            &control,
        )?;
        drop(bar);
        all.files += did.files;
        all.bytes += did.bytes;
        all.skipped += did.skipped;
        all.links_left += did.links_left;
    }
    if cx.json {
        print_json(&json!({
            "files": all.files,
            "bytes": all.bytes,
            "skipped": all.skipped,
            "links_left": all.links_left,
        }));
        return Ok(());
    }
    if !cx.quiet {
        let verb = if moving { "Moved" } else { "Copied" };
        let mut said = format!(
            "{verb} {} ({}) into {}",
            file_count(all.files),
            size(all.bytes),
            to.shown
        );
        if all.skipped > 0 {
            said += &format!(", {} skipped as their names were taken", all.skipped);
        }
        if all.links_left > 0 {
            said += &format!(", {} links left where they were", all.links_left);
        }
        eprintln!("{said}");
    }
    Ok(())
}

fn done(cx: &Context, said: String) -> Result<(), Exit> {
    if cx.json {
        print_json(&json!({ "ok": true }));
    } else if !cx.quiet {
        eprintln!("{said}");
    }
    Ok(())
}

/// The storage a local file is in, and its path there.
fn local(file: &Path) -> (LocalFolder, Vec<String>) {
    let folder = file
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    (LocalFolder::new(folder), vec![named(file)])
}

fn algorithms(typed: &[String]) -> Result<Vec<Algorithm>, Exit> {
    if typed.is_empty() {
        return Ok(vec![Algorithm::Sha256]);
    }
    if typed.iter().any(|t| t == "all") {
        return Ok(Algorithm::ALL.to_vec());
    }
    typed
        .iter()
        .map(|t| {
            let t = t.to_ascii_lowercase();
            // BLAKE3's own name, beside the core's key for it.
            let key = if t == "blake3" { "b3" } else { t.as_str() };
            Algorithm::keyed(key).ok_or_else(|| {
                Exit::Usage(format!(
                    "no algorithm {t}: sha256, sha512, sha1, md5, blake3 or all"
                ))
            })
        })
        .collect()
}

fn checksum_cmd(
    files: &[String],
    typed: &[String],
    save: bool,
    cx: &mut Context,
) -> Result<(), Exit> {
    if files.is_empty() {
        return Err(Exit::Usage("which files? (or --check SUMS)".into()));
    }
    let algorithms = algorithms(typed)?;
    let mut found = Vec::new();
    for file in files {
        let place = Place::of(file)?;
        let (control, bar) =
            crate::progress::control(&format!("Reading {}", place.name()), cx.progress);
        let sums = checksums(&*place.storage, &place.path, &algorithms, &control)?;
        drop(bar);
        for (algorithm, sum) in sums {
            if save {
                save_checksum(&*place.storage, &place.path, algorithm, &sum)?;
            }
            found.push((place.shown.clone(), algorithm, sum));
        }
    }
    if cx.json {
        let rows: Vec<Value> = found
            .iter()
            .map(|(f, a, s)| json!({ "file": f, "algorithm": a.key(), "sum": s }))
            .collect();
        print_json(&json!(rows));
    } else if !cx.quiet {
        let mut out = std::io::stdout().lock();
        for (file, algorithm, sum) in &found {
            // sha256sum's line with one algorithm, BSD's tagged one with several.
            let line = if algorithms.len() == 1 {
                checksum_line(sum, file)
            } else {
                format!("{} ({file}) = {sum}\n", tag(*algorithm))
            };
            let _ = out.write_all(line.as_bytes());
        }
    }
    Ok(())
}

/// An algorithm's tag on BSD's lines: `SHA256`, `BLAKE3`.
fn tag(algorithm: Algorithm) -> String {
    match algorithm {
        Algorithm::Blake3 => "BLAKE3".into(),
        a => a.key().to_ascii_uppercase(),
    }
}

/// A line of a sums file: its checksum and the file's name, as `sha256sum`
/// writes it (`sum  name`, `sum *name`) or BSD's tools (`SHA256 (name) = sum`).
fn sum_line(line: &str) -> Option<(String, String)> {
    let line = line.trim_end();
    if let Some((tag, rest)) = line.split_once(" (")
        && !tag.contains(' ')
        && let Some((name, sum)) = rest.rsplit_once(") = ")
    {
        return Some((sum.trim().to_ascii_lowercase(), name.to_owned()));
    }
    let line = line.strip_prefix('\\').unwrap_or(line);
    let (sum, name) = line.split_once(' ')?;
    let name = name.strip_prefix(['*', ' ']).unwrap_or(name);
    (!sum.is_empty() && sum.bytes().all(|b| b.is_ascii_hexdigit()) && !name.is_empty())
        .then(|| (sum.to_ascii_lowercase(), name.to_owned()))
}

fn check_sums(sums: &Path, cx: &mut Context) -> Result<(), Exit> {
    let text = std::fs::read_to_string(sums)
        .map_err(|e| Exit::Failed(format!("cannot read {}: {e}", sums.display())))?;
    let folder = sums
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut results = Vec::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let Some((expected, name)) = sum_line(line) else {
            continue;
        };
        let candidates = algorithms_of(expected.len());
        let file = folder.join(&name);
        let (storage, path) = local(&file);
        let ok = !candidates.is_empty()
            && checksums(
                &storage,
                &path,
                &candidates,
                &packwing_core::Control::default(),
            )
            .map(|found| found.iter().any(|(_, s)| s.eq_ignore_ascii_case(&expected)))
            .unwrap_or(false);
        results.push((name, ok));
    }
    if results.is_empty() {
        return Err(Exit::Failed(format!(
            "{} has no checksum lines",
            sums.display()
        )));
    }
    let failed = results.iter().filter(|(_, ok)| !ok).count();
    if cx.json {
        let rows: Vec<Value> = results
            .iter()
            .map(|(n, ok)| json!({ "file": n, "ok": ok }))
            .collect();
        print_json(&json!(rows));
    } else {
        for (name, ok) in &results {
            if !cx.quiet || !ok {
                println!("{name}: {}", if *ok { "OK" } else { "FAILED" });
            }
        }
    }
    if failed > 0 {
        Err(Exit::Failed(match (failed, results.len()) {
            (1, 1) => "it did not match".into(),
            (failed, all) => format!("{failed} of {all} did not match"),
        }))
    } else {
        Ok(())
    }
}

/// Reads `from` into a folder of its own beside `to`, makes `to` of what is in
/// it, and takes the folder away, whatever came of it.
fn convert_cmd(
    from: &Path,
    to: &Path,
    make: Making,
    filter: &Filter,
    damaging: &Damaging,
    cx: &mut Context,
) -> Result<(), Exit> {
    let (to, new) = making(to, make, cx)?;
    let name = named(from);
    // An archive holds any name: none is asked about, but what leads outside.
    let how = ExtractOptions::default();
    let (control, bar) = crate::progress::control("Converting", cx.progress);
    let control = damaging.control(control, cx)?;
    let converted = cx.secrets.unlocking(&name, |keys| {
        convert(Source::from(from), keys, &to, &new, &how, filter, &control)
    })?;
    drop(bar);
    let outcome = outcome_of(&converted.summary);
    let made = &converted.made;
    let bytes = std::fs::metadata(made).map(|m| m.len()).unwrap_or_default();
    // What signed the one read signed its bytes: the new archive's are others.
    let unsigned = has_signature(from);
    if cx.json {
        print_json(&json!({
            "made": made.display().to_string(),
            "size": bytes,
            "files": converted.summary.files,
            "damaged": crate::damaged::as_json(&outcome.damaged),
            "skipped": crate::damaged::left_out_json(&outcome.left_out),
            "renamed": crate::damaged::renamed_json(&outcome.renamed),
            "signature_left_behind": unsigned,
        }));
    } else if !cx.quiet {
        println!("{} ({})", made.display(), size(bytes));
        if unsigned {
            eprintln!(
                "pkw: {} is signed; {} is not, as a signature is of the bytes it signs",
                named(from),
                named(made)
            );
        }
    }
    damaging.finish(&outcome)
}

fn split_cmd(file: &Path, typed: &str, cx: &Context) -> Result<(), Exit> {
    let part = size_typed(typed)
        .filter(|&n| n > 0)
        .ok_or_else(|| Exit::Usage(format!("not a size: {typed} (as 25MB)")))?;
    let (control, bar) = crate::progress::control("Splitting", cx.progress);
    let parts = split_file(file, part, &control)?;
    drop(bar);
    if cx.json {
        let names: Vec<String> = parts.iter().map(|p| p.display().to_string()).collect();
        print_json(&json!({ "parts": names }));
    } else if !cx.quiet
        && let (Some(first), Some(last)) = (parts.first(), parts.last())
    {
        println!(
            "{} parts of {}: {} to {}",
            parts.len(),
            size(part),
            first.display(),
            named(last)
        );
    }
    Ok(())
}

fn join_cmd(part: &Path, cx: &Context) -> Result<(), Exit> {
    let (control, bar) = crate::progress::control("Joining", cx.progress);
    let made = join_parts(part, &control)?;
    drop(bar);
    let bytes = std::fs::metadata(&made)
        .map(|m| m.len())
        .unwrap_or_default();
    if cx.json {
        print_json(&json!({ "made": made.display().to_string(), "size": bytes }));
    } else if !cx.quiet {
        println!("{} ({})", made.display(), size(bytes));
    }
    Ok(())
}

fn passphrase_cmd(min: usize, cx: &Context) -> Result<(), Exit> {
    // Never shorter than what the organization asks of a password.
    let least = packwing_policy::current()
        .least_password_length()
        .map_or(0, |n| n as usize);
    let made = packwing_core::suggest_passphrase(min.max(least))?;
    if cx.json {
        print_json(&json!({ "passphrase": made }));
    } else {
        println!("{made}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_are_typed_as_gnu_tar_takes_them() {
        assert_eq!(time_typed("@1700000000"), Some(1_700_000_000));
        assert_eq!(time_typed("@-1"), Some(-1));
        assert_eq!(time_typed("1970-01-01"), Some(0));
        assert_eq!(time_typed("2026-11-07"), Some(1_794_009_600));
        assert_eq!(time_typed("2026-11-07T10:30:00"), Some(1_794_047_400));
        assert_eq!(time_typed("2026-11-07 10:30:00Z"), Some(1_794_047_400));
        assert_eq!(time_typed("2000-02-29"), Some(951_782_400));
        assert_eq!(time_typed("1969-12-31T23:59:59"), Some(-1));
        for bad in [
            "",
            "@",
            "2026-13-01",
            "2026-1-07",
            "2026-11-07T25:00:00",
            "yesterday",
        ] {
            assert_eq!(time_typed(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn source_date_epoch_takes_later_times_back_unless_one_time_is_given() {
        let epoch = Some("1700000000".to_owned());
        assert!(matches!(times_of(None, None), Ok(Times::AsTheyAre)));
        assert!(matches!(
            times_of(None, epoch.clone()),
            Ok(Times::AtMost(1_700_000_000))
        ));
        assert!(matches!(times_of(Some("@5"), epoch), Ok(Times::All(5))));
        assert!(times_of(None, Some("soon".into())).is_err());
        assert!(times_of(Some("soon"), None).is_err());
    }

    #[test]
    fn owners_and_attributes_are_for_tar_alone() {
        let backup = Backup {
            owners: true,
            ..Backup::default()
        };
        assert!(kept_for_backup(&backup, Kind::Zip).is_err());
        assert!(kept_for_backup(&Backup::default(), Kind::Zip).is_ok());
        if cfg!(unix) {
            assert!(kept_for_backup(&backup, Kind::TarZst).is_ok_and(|keep| keep.owners));
        }
    }

    #[test]
    fn owners_come_back_by_name_or_by_number_as_asked() {
        let asked = |same_owner, numeric_owner, xattrs| {
            restored(&Restore {
                same_owner,
                numeric_owner,
                xattrs,
            })
        };
        assert!(matches!(
            asked(false, false, false),
            Ok((Owners::Ignore, false))
        ));
        if cfg!(any(target_os = "linux", target_os = "macos")) {
            assert!(matches!(
                asked(true, false, false),
                Ok((Owners::ByName, false))
            ));
            // As GNU tar takes the two together: by number.
            for same_owner in [false, true] {
                assert!(matches!(
                    asked(same_owner, true, false),
                    Ok((Owners::ByNumber, false))
                ));
            }
            assert!(matches!(
                asked(false, false, true),
                Ok((Owners::Ignore, true))
            ));
        } else {
            assert!(asked(true, false, false).is_err());
            assert!(asked(false, false, true).is_err());
        }
    }

    #[test]
    fn a_sums_file_s_lines_are_read_as_tools_write_them() {
        let sum = "ab".repeat(32);
        assert_eq!(
            sum_line(&format!("{sum}  notes.txt")),
            Some((sum.clone(), "notes.txt".into()))
        );
        assert_eq!(
            sum_line(&format!("{sum} *with space.bin")),
            Some((sum.clone(), "with space.bin".into()))
        );
        assert_eq!(
            sum_line(&format!("SHA256 (a (b).zip) = {}", sum.to_uppercase())),
            Some((sum.clone(), "a (b).zip".into()))
        );
        assert_eq!(sum_line("# a comment"), None);
        assert_eq!(sum_line("not a sum"), None);
    }
}
