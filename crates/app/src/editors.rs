//! Text editors: the ones this system has, the system's own for text files,
//! and starting one on a file. Taken from shellwing (`crates/app/src/editors.rs`
//! and `crates/core/src/command.rs`).
//!
//! An editor is started from the window's thread, whose signals are not
//! blocked, so that on macOS it can be stopped as any other program.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use packwing_i18n::tr;

/// Editors worth offering, by the name shown and the command that starts one.
#[cfg(target_os = "macos")]
const KNOWN: &[(&str, &str)] = &[
    ("VS Code", "code"),
    ("VSCodium", "codium"),
    ("Zed", "zed"),
    ("Sublime Text", "subl"),
    ("BBEdit", "bbedit"),
    ("TextMate", "mate"),
];
#[cfg(target_os = "windows")]
const KNOWN: &[(&str, &str)] = &[
    ("VS Code", "code"),
    ("VSCodium", "codium"),
    ("Zed", "zed"),
    ("Sublime Text", "subl"),
    ("Notepad++", "notepad++"),
    ("Notepad", "notepad"),
];
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const KNOWN: &[(&str, &str)] = &[
    ("VS Code", "code"),
    ("VSCodium", "codium"),
    ("Zed", "zed"),
    ("Sublime Text", "subl"),
    ("GNOME Text Editor", "gnome-text-editor"),
    ("gedit", "gedit"),
    ("Kate", "kate"),
    ("KWrite", "kwrite"),
    ("Mousepad", "mousepad"),
    ("Xed", "xed"),
    ("Pluma", "pluma"),
    ("Geany", "geany"),
];

/// Where `cmd` is on `PATH`, if it is.
fn which(cmd: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let names: Vec<String> = if cfg!(windows) && Path::new(cmd).extension().is_none() {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT".into())
            .split(';')
            .filter(|e| !e.is_empty())
            .map(|e| format!("{cmd}{}", e.to_lowercase()))
            .collect()
    } else {
        vec![cmd.to_owned()]
    };
    std::env::split_paths(&path)
        .find_map(|dir| names.iter().map(|n| dir.join(n)).find(|p| p.is_file()))
}

/// Splits a command as a shell would its words: spaces part them, quotes keep
/// them together.
fn words(cmd: &str) -> Vec<String> {
    let (mut out, mut cur, mut quote, mut any) = (vec![], String::new(), None, false);
    for c in cmd.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => cur.push(c),
            (None, '"' | '\'') => {
                quote = Some(c);
                any = true;
            }
            (None, c) if c.is_whitespace() => {
                if any || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    any = false;
                }
            }
            (None, c) => cur.push(c),
        }
    }
    if any || !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// The known editors this system has: (name, command).
pub fn installed() -> Vec<(String, String)> {
    KNOWN
        .iter()
        .filter(|(_, cmd)| which(cmd).is_some())
        .map(|(name, cmd)| ((*name).to_owned(), (*cmd).to_owned()))
        .collect()
}

/// The name of a known editor's command, else its program's file name.
pub fn name_of(cmd: &str) -> String {
    let first = words(cmd).into_iter().next().unwrap_or_default();
    KNOWN
        .iter()
        .find(|(_, c)| *c == first)
        .map(|(n, _)| (*n).to_owned())
        .unwrap_or_else(|| {
            Path::new(&first)
                .file_name()
                .map_or(first.clone(), |f| f.to_string_lossy().into_owned())
        })
}

fn spawn(program: &str, args: &[&std::ffi::OsStr]) -> std::io::Result<()> {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

/// Starts `cmd`, the editor chosen in Settings, on `path`, added at the end.
pub fn open_with(cmd: &str, path: &Path) -> std::io::Result<()> {
    let w = words(cmd);
    let Some((program, rest)) = w.split_first() else {
        return Err(std::io::Error::other(tr!("no editor command")));
    };
    let mut args: Vec<&std::ffi::OsStr> = rest.iter().map(|a| a.as_ref()).collect();
    args.push(path.as_os_str());
    spawn(program, &args)
}

/// The desktop's app for plain text, as its desktop file id (Linux).
fn text_plain_app() -> Option<String> {
    Command::new("xdg-mime")
        .args(["query", "default", "text/plain"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .filter(|id| !id.is_empty())
}

/// Opens `path` in the system's text editor, whatever kind of file it is.
pub fn open_text(path: &Path) -> std::io::Result<()> {
    if cfg!(target_os = "macos") {
        return spawn("open", &["-t".as_ref(), path.as_os_str()]);
    }
    if cfg!(target_os = "windows") {
        return spawn("notepad", &[path.as_os_str()]);
    }
    // The desktop's app for plain text, then any known editor there is.
    if let Some(id) = text_plain_app()
        && which("gtk-launch").is_some()
        && spawn("gtk-launch", &[id.as_ref(), path.as_os_str()]).is_ok()
    {
        return Ok(());
    }
    match installed().first() {
        Some((_, cmd)) => open_with(cmd, path),
        None => Err(std::io::Error::other(tr!("no text editor found"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_split_like_a_shell() {
        assert_eq!(words("code -n"), ["code", "-n"]);
        assert_eq!(
            words(r#""/opt/My Editor/bin/edit" --wait"#),
            ["/opt/My Editor/bin/edit", "--wait"]
        );
        assert_eq!(words("  subl  "), ["subl"]);
        assert_eq!(words("vim -c ''"), ["vim", "-c", ""]);
    }

    #[test]
    fn known_commands_have_names() {
        assert_eq!(name_of("code -n"), "VS Code");
        assert_eq!(name_of("/opt/x/edit --wait"), "edit");
    }
}
