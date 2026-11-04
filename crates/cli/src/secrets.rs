//! What opens an encrypted archive, from where a script can give it, never
//! from the command line, where others see it: a file, standard input, or
//! `PACKWING_PASSWORD`; an age key file. In a terminal, what an archive asks
//! for and was not given is asked, as the window asks it.

use std::io::{BufRead as _, IsTerminal as _};
use std::path::{Path, PathBuf};

use packwing_core::{Asked, Error, Keys};

/// Where the secrets come from.
pub struct Secrets {
    keys: Keys,
    /// A person is at the terminal, and standard input is not the secret's.
    can_ask: bool,
}

impl Secrets {
    /// Reads what is given: the password (an archive's, and an age file's
    /// passphrase alike) from `password_file`, standard input or
    /// `PACKWING_PASSWORD`; the age key file `key`, and its passphrase from
    /// `PACKWING_KEY_PASSPHRASE`.
    pub fn read(
        password_file: Option<&Path>,
        password_stdin: bool,
        key: Option<&Path>,
    ) -> Result<Secrets, String> {
        let password = if let Some(file) = password_file {
            Some(first_line_of(file)?)
        } else if password_stdin {
            let mut line = String::new();
            std::io::stdin()
                .lock()
                .read_line(&mut line)
                .map_err(|e| format!("cannot read the password from standard input: {e}"))?;
            Some(trim_line(line.into_bytes()))
        } else {
            std::env::var_os("PACKWING_PASSWORD").map(|p| p.into_encoded_bytes())
        };
        let key_file = key
            .map(|k| std::fs::read(k).map_err(|e| format!("cannot read {}: {e}", k.display())))
            .transpose()?;
        let keys = Keys {
            passphrase: password.clone(),
            password,
            key_file,
            key_passphrase: std::env::var_os("PACKWING_KEY_PASSPHRASE")
                .map(|p| p.into_encoded_bytes()),
        };
        Ok(Secrets {
            keys,
            can_ask: !password_stdin && std::io::stdin().is_terminal(),
        })
    }

    /// The password given, for an archive to make.
    pub fn password(&self) -> Option<Vec<u8>> {
        self.keys.password.clone()
    }

    /// A key file's passphrase, as given: `PACKWING_KEY_PASSPHRASE`, else the
    /// password given, which is the one secret a script gives for one job.
    pub fn key_passphrase(&self) -> Option<Vec<u8>> {
        self.keys
            .key_passphrase
            .clone()
            .or_else(|| self.keys.password.clone())
    }

    /// Whether a person can be asked.
    pub fn can_ask(&self) -> bool {
        self.can_ask
    }

    /// Does `job` with the keys there are, and while it says it needs another,
    /// asks for it, a few times, then gives up with what it said.
    pub fn unlocking<T>(
        &mut self,
        name: &str,
        mut job: impl FnMut(&Keys) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let mut tries = 0;
        loop {
            let error = match job(&self.keys) {
                Ok(done) => return Ok(done),
                Err(e) => e,
            };
            let asked = match &error {
                Error::Locked { needs } => Keys::asks_for(needs),
                Error::WrongPassword => Some(Asked::Password),
                Error::WrongPassphrase => Some(Asked::Passphrase),
                Error::WrongKeyPassphrase => Some(Asked::KeyPassphrase),
                _ => None,
            };
            let Some(asked) = asked.filter(|_| self.can_ask && tries < 3) else {
                return Err(error);
            };
            if tries > 0 || !matches!(error, Error::Locked { .. }) {
                eprintln!("pkw: {error}");
            }
            tries += 1;
            let Some(secret) = ask(asked, name) else {
                return Err(error);
            };
            self.keys = self.keys.with(asked, secret);
        }
    }
}

/// Asks for what `asked` is, for the archive `name`: none when the person
/// gives nothing.
fn ask(asked: Asked, name: &str) -> Option<Vec<u8>> {
    let typed = |what: &str| {
        rpassword::prompt_password(format!("{what} for {name}: "))
            .ok()
            .filter(|s| !s.is_empty())
            .map(String::into_bytes)
    };
    match asked {
        Asked::Password => typed("Password"),
        Asked::Passphrase => typed("Passphrase"),
        Asked::KeyPassphrase => typed("The key file's passphrase"),
        Asked::KeyFile => {
            eprint!("Key file for {name} (age or SSH): ");
            let mut line = String::new();
            std::io::stdin().lock().read_line(&mut line).ok()?;
            let path = PathBuf::from(line.trim());
            std::fs::read(&path)
                .map_err(|e| eprintln!("pkw: cannot read {}: {e}", path.display()))
                .ok()
        }
    }
}

fn first_line_of(file: &Path) -> Result<Vec<u8>, String> {
    let text = std::fs::read(file).map_err(|e| format!("cannot read {}: {e}", file.display()))?;
    let line = text.split(|b| *b == b'\n').next().unwrap_or_default();
    Ok(trim_line(line.to_vec()))
}

/// A line without its end, `\n` or `\r\n`.
fn trim_line(mut line: Vec<u8>) -> Vec<u8> {
    while line.last().is_some_and(|b| *b == b'\n' || *b == b'\r') {
        line.pop();
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_secret_is_its_first_line_without_its_end() {
        assert_eq!(trim_line(b"secret\r\n".to_vec()), b"secret");
        assert_eq!(trim_line(b"two words\n".to_vec()), b"two words");
        let dir = std::env::temp_dir().join(format!("pkw-secret-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a folder");
        let file = dir.join("pw");
        std::fs::write(&file, "khoá bí mật\nthe rest\n").expect("the file");
        assert_eq!(
            first_line_of(&file).expect("read"),
            "khoá bí mật".as_bytes()
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
