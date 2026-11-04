//! What each command does, through packwing-core, and what it writes.

use std::io::Write as _;
use std::path::{Path, PathBuf};

use packwing_core::{
    Algorithm, COMPRESSED_EXTENSIONS, Entry, ExtractOptions, Kind, Listing, LocalFolder,
    NewArchive, OnExisting, Source, add_with, algorithms_of, came_from_internet, checksum_line,
    checksums, create, delete, extract_selected, extract_with, has_signature, list, read_entry,
    rename, save_checksum, size, stem, test,
};
use serde_json::{Value, json};

use crate::names::{
    date, entry_path, format_named, format_of, format_words, level_named, size_typed,
};
use crate::{Command, Context, Exit};

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
        } => extract_cmd(&archive, &files, output, here, overwrite, &limit, cx),
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
            format,
            level,
            password,
            encrypt_to,
            age,
            hide_names,
            split,
            compress_all,
        } => create_cmd(
            &archive,
            &paths,
            Making {
                format,
                level,
                password,
                encrypt_to,
                age,
                hide_names,
                split,
                compress_all,
            },
            cx,
        ),
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

fn listed(path: &Path, cx: &mut Context) -> Result<Listing, Exit> {
    let name = named(path);
    Ok(cx
        .secrets
        .unlocking(&name, |keys| list(Source::from(path), keys))?)
}

fn print_json(value: &Value) {
    println!("{value:#}");
}

fn list_cmd(archive: &Path, paths: &[String], long: bool, cx: &mut Context) -> Result<(), Exit> {
    let listing = listed(archive, cx)?;
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
    let mut out = std::io::stdout().lock();
    for e in shown {
        let path = e.path.join("/") + if e.dir { "/" } else { "" };
        let line = if long {
            let when = e.modified.map(date).unwrap_or_else(|| " ".repeat(16));
            let bytes = if e.dir {
                String::new()
            } else {
                e.size.to_string()
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
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn extract_cmd(
    archive: &Path,
    files: &[String],
    output: Option<PathBuf>,
    here: bool,
    overwrite: bool,
    limit: &str,
    cx: &mut Context,
) -> Result<(), Exit> {
    let name = named(archive);
    let destination = match (output, here) {
        (Some(dir), _) => dir,
        (None, true) => PathBuf::from("."),
        // As the menu's Extract to folder: a folder named after it, here.
        (None, false) => PathBuf::from(stem(&name)),
    };
    let how = ExtractOptions {
        overwrite,
        limit: Some(limit_of(limit)?),
        ..ExtractOptions::default()
    };
    let picked = picked(files);
    let (control, bar) = crate::progress::control("Extracting", cx.progress);
    let summary = cx.secrets.unlocking(&name, |keys| {
        if picked.is_empty() {
            extract_with(Source::from(archive), &destination, keys, &how, &control)
        } else {
            extract_selected(
                Source::from(archive),
                &destination,
                &picked,
                keys,
                &how,
                &control,
            )
        }
    })?;
    drop(bar);
    if cx.json {
        print_json(&json!({
            "to": destination.display().to_string(),
            "files": summary.files,
            "directories": summary.directories,
            "links": summary.links,
            "bytes": summary.bytes,
        }));
    } else if !cx.quiet {
        eprintln!(
            "{}, {}, into {}",
            file_count(summary.files),
            size(summary.bytes),
            destination.display()
        );
    }
    Ok(())
}

fn test_cmd(archive: &Path, cx: &mut Context) -> Result<(), Exit> {
    let name = named(archive);
    let (control, bar) = crate::progress::control("Testing", cx.progress);
    let summary = cx
        .secrets
        .unlocking(&name, |keys| test(Source::from(archive), keys, &control))?;
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

fn cat_cmd(archive: &Path, file: &str, limit: &str, cx: &mut Context) -> Result<(), Exit> {
    let name = named(archive);
    let entry = entry_path(file);
    let most = limit_of(limit)?;
    let control = packwing_core::Control::default();
    let bytes = cx.secrets.unlocking(&name, |keys| {
        read_entry(Source::from(archive), &entry, keys, most, &control)
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

fn info_cmd(archive: &Path, cx: &mut Context) -> Result<(), Exit> {
    let l = listed(archive, cx)?;
    let layers: Vec<String> = l.layers.iter().map(|x| format!("{x:?}")).collect();
    let format = std::iter::once(format!("{:?}", l.container))
        .chain(layers.iter().rev().cloned())
        .collect::<Vec<_>>()
        .join(" in ");
    let ratio = (l.unpacked > 0).then(|| l.packed as f64 / l.unpacked as f64);
    let signed = has_signature(archive);
    let from_internet = came_from_internet(&Source::from(archive));
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

/// How `pkw create` was asked to make it.
struct Making {
    format: Option<String>,
    level: String,
    password: bool,
    encrypt_to: Vec<String>,
    age: bool,
    hide_names: bool,
    split: Option<String>,
    compress_all: bool,
}

fn create_cmd(archive: &Path, paths: &[PathBuf], m: Making, cx: &mut Context) -> Result<(), Exit> {
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
            None if cx.secrets.can_ask() => ask_new_password()?,
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
    let new = NewArchive {
        kind,
        level,
        password,
        age_keys: m.encrypt_to,
        whole_with_age: m.age,
        hide_names: m.hide_names,
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
    let (control, bar) = crate::progress::control("Compressing", cx.progress);
    let made = create(paths, &archive, &new, &control)?;
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

/// A new password, typed twice, the same.
fn ask_new_password() -> Result<Vec<u8>, Exit> {
    let typed = |what: &str| {
        rpassword::prompt_password(what).map_err(|e| Exit::Failed(format!("cannot ask: {e}")))
    };
    let first = typed("New password: ")?;
    if first.is_empty() {
        return Err(Exit::Usage("no password typed".into()));
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
    let on_existing = match on_existing {
        "stop" => OnExisting::Stop,
        "replace" => OnExisting::Replace,
        "skip" => OnExisting::Skip,
        other => {
            return Err(Exit::Usage(format!(
                "--on-existing is stop, replace or skip, not {other}"
            )));
        }
    };
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
    cx.secrets
        .unlocking(&named(archive), |keys| delete(archive, &picked, keys))?;
    done(cx, format!("deleted from {}", archive.display()))
}

fn rename_cmd(archive: &Path, file: &str, name: &str, cx: &mut Context) -> Result<(), Exit> {
    let from = entry_path(file);
    cx.secrets
        .unlocking(&named(archive), |keys| rename(archive, &from, name, keys))?;
    done(cx, format!("renamed in {}", archive.display()))
}

/// Says a change was made: in JSON, that it was.
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
    files: &[PathBuf],
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
        let (storage, path) = local(file);
        let (control, bar) =
            crate::progress::control(&format!("Reading {}", named(file)), cx.progress);
        let sums = checksums(&storage, &path, &algorithms, &control)?;
        drop(bar);
        for (algorithm, sum) in sums {
            if save {
                save_checksum(&storage, &path, algorithm, &sum)?;
            }
            found.push((file.display().to_string(), algorithm, sum));
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
        Err(Exit::Failed(format!(
            "{failed} of {} files did not match",
            results.len()
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
