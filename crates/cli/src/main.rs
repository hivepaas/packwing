//! `pkw`: packwing on the command line. It opens what the app opens, as
//! safely, and keeps to the organization's policies as the app does: the
//! work is packwing-core's. What it finds goes to standard output, as text or
//! as JSON (`--json`); what goes wrong, and how far a job is, to standard
//! error. Its exit code says how it went ([`Exit`]).

mod commands;
mod names;
mod progress;
mod secrets;

use std::io::IsTerminal as _;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use crate::secrets::Secrets;

#[derive(Parser)]
#[command(
    name = "pkw",
    version,
    about = "packwing on the command line: open, extract, check and make archives",
    long_about = "packwing on the command line: open, extract, check and make archives.\n\n\
        It reads ZIP, 7z, RAR, tar and its compressed kinds, ISO and UDF images, cpio, \
        RPM, XAR and age, telling each by its bytes, and makes ZIP, 7z and tar ones. \
        Extraction never writes outside where it is told, and keeps the mark of \
        files from the Internet.\n\n\
        A password is never given on the command line: pkw asks for it in a \
        terminal, or reads it from --password-file, --password-stdin or \
        PACKWING_PASSWORD.",
    after_help = "Exit codes: 0 done; 1 failed; 2 not understood; 3 a password or key is \
        needed, or wrong; 4 not an archive pkw can open."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[command(flatten)]
    output: Output,
    #[command(flatten)]
    secrets: SecretArgs,
}

/// How what is found is written.
#[derive(Args)]
struct Output {
    /// Write what is found as JSON, for scripts.
    #[arg(long, global = true)]
    json: bool,
    /// Say nothing but errors, and show no progress.
    #[arg(short, long, global = true)]
    quiet: bool,
}

/// Where secrets come from.
#[derive(Args)]
struct SecretArgs {
    /// Read the password or passphrase from this file's first line.
    #[arg(long, global = true, value_name = "FILE")]
    password_file: Option<PathBuf>,
    /// Read the password or passphrase from standard input's first line.
    #[arg(long, global = true, conflicts_with = "password_file")]
    password_stdin: bool,
    /// The age or SSH key file an archive was encrypted to.
    #[arg(long, global = true, value_name = "FILE")]
    key: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// List what an archive holds.
    #[command(visible_alias = "ls")]
    List {
        archive: PathBuf,
        /// Only what is under these paths in it.
        paths: Vec<String>,
        /// With each one's size and date.
        #[arg(short, long)]
        long: bool,
    },
    /// Extract an archive, or files from it.
    #[command(visible_alias = "x")]
    Extract {
        archive: PathBuf,
        /// Only these files or folders in it, each straight into where it goes, as the
        /// app extracts them: `docs/a/b.txt` comes out as `b.txt`, `docs/a` as `a/`.
        files: Vec<String>,
        /// Where to: by default a new folder named after the archive, here.
        #[arg(short, long, value_name = "DIR", conflicts_with = "here")]
        output: Option<PathBuf>,
        /// Into this folder itself, with no folder of its own.
        #[arg(long)]
        here: bool,
        /// Replace files that are there; otherwise one stops it.
        #[arg(long)]
        overwrite: bool,
        /// Write at most this much, as 50GB: what stops an archive bomb.
        #[arg(long, value_name = "SIZE", default_value = "100GB")]
        limit: String,
    },
    /// Check that an archive reads whole, as it was made.
    #[command(visible_alias = "t")]
    Test { archive: PathBuf },
    /// Write a file in an archive to standard output.
    Cat {
        archive: PathBuf,
        file: String,
        /// Read at most this much.
        #[arg(long, value_name = "SIZE", default_value = "10GB")]
        limit: String,
    },
    /// Say what an archive is: its format, layers, size and encryption.
    Info { archive: PathBuf },
    /// Make an archive of files and folders.
    #[command(visible_alias = "c")]
    Create {
        /// The archive to make: its extension says its format, unless --format does.
        archive: PathBuf,
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// zip, 7z, tar, tar.zst, tar.gz, tar.xz, tar.bz2, tar.lz4, or one file alone:
        /// zst, gz, xz, bz2, lz4.
        #[arg(short, long, value_name = "FORMAT")]
        format: Option<String>,
        /// store, lowest, low, medium, high or highest (0 to 5).
        #[arg(short, long, value_name = "LEVEL", default_value = "medium")]
        level: String,
        /// Protect it with a password: asked, or read as --password-file says.
        #[arg(short, long)]
        password: bool,
        /// Encrypt it whole with age to this public key (age1…, age1pq1…, ssh-ed25519 …);
        /// again for more.
        #[arg(long = "encrypt-to", value_name = "KEY")]
        encrypt_to: Vec<String>,
        /// A ZIP or 7z archive encrypted whole with age, not with its own AES-256.
        #[arg(long)]
        age: bool,
        /// A 7z archive's names hidden behind its password too.
        #[arg(long)]
        hide_names: bool,
        /// Cut it into parts of this size, as 25MB.
        #[arg(long, value_name = "SIZE")]
        split: Option<String>,
        /// Compress pictures, video, sound and archives too, not only store them.
        #[arg(long)]
        compress_all: bool,
    },
    /// Add files and folders to a ZIP archive.
    Add {
        archive: PathBuf,
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// Into this folder in it.
        #[arg(long, value_name = "PATH", default_value = "")]
        into: String,
        /// When a name is taken: stop, replace or skip.
        #[arg(long, value_name = "WHAT", default_value = "stop")]
        on_existing: String,
    },
    /// Delete files or folders from a ZIP archive.
    #[command(visible_alias = "rm")]
    Delete {
        archive: PathBuf,
        #[arg(required = true)]
        files: Vec<String>,
    },
    /// Rename a file or folder in a ZIP archive.
    #[command(visible_alias = "mv")]
    Rename {
        archive: PathBuf,
        file: String,
        /// Its new name, in the same folder.
        name: String,
    },
    /// Make or check the checksums of files: sha256, sha512, sha1, md5, blake3.
    #[command(visible_alias = "sum")]
    Checksum {
        files: Vec<PathBuf>,
        /// Which: sha256 by default; again, or `all`, for more.
        #[arg(short, long, value_name = "ALGORITHM")]
        algorithm: Vec<String>,
        /// Check the files a sums file names (sha256sum's lines, or BSD's).
        #[arg(short, long, value_name = "SUMS", conflicts_with = "files")]
        check: Option<PathBuf>,
        /// Save each file's in a file beside it, as `name.sha256`.
        #[arg(long)]
        save: bool,
    },
}

/// How it went, as the exit code says.
#[derive(Debug)]
pub enum Exit {
    /// It could not be done: 1.
    Failed(String),
    /// What was typed is not understood: 2.
    Usage(String),
    /// A password or key is needed, or wrong: 3.
    Locked(String),
    /// Not an archive pkw opens: 4.
    NotAnArchive(String),
}

impl Exit {
    fn code(&self) -> u8 {
        match self {
            Exit::Failed(_) => 1,
            Exit::Usage(_) => 2,
            Exit::Locked(_) => 3,
            Exit::NotAnArchive(_) => 4,
        }
    }

    fn message(&self) -> &str {
        match self {
            Exit::Failed(m) | Exit::Usage(m) | Exit::Locked(m) | Exit::NotAnArchive(m) => m,
        }
    }
}

impl From<packwing_core::Error> for Exit {
    fn from(e: packwing_core::Error) -> Exit {
        use packwing_core::Error;
        let said = e.to_string();
        match e {
            Error::NotAnArchive => Exit::NotAnArchive(said),
            Error::Locked { .. }
            | Error::WrongPassword
            | Error::WrongPassphrase
            | Error::WrongKeyPassphrase => Exit::Locked(said),
            _ => Exit::Failed(said),
        }
    }
}

/// What every command is given.
pub struct Context {
    pub json: bool,
    pub quiet: bool,
    /// How far a job is, shown: a person watches, and nothing is asked.
    pub progress: bool,
    pub secrets: Secrets,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    // The core's messages, in the system's language, as the app's are.
    packwing_i18n::set_language(None);
    let secrets = match Secrets::read(
        cli.secrets.password_file.as_deref(),
        cli.secrets.password_stdin,
        cli.secrets.key.as_deref(),
    ) {
        Ok(s) => s,
        Err(why) => return fail(&Exit::Failed(why)),
    };
    let mut cx = Context {
        json: cli.output.json,
        quiet: cli.output.quiet,
        progress: !cli.output.quiet && std::io::stderr().is_terminal(),
        secrets,
    };
    match commands::run(cli.command, &mut cx) {
        Ok(()) => ExitCode::SUCCESS,
        Err(exit) => fail(&exit),
    }
}

fn fail(exit: &Exit) -> ExitCode {
    eprintln!("pkw: {}", exit.message());
    ExitCode::from(exit.code())
}
