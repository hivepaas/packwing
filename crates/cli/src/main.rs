//! `pkw`: packwing on the command line. It opens what the app opens, as
//! safely, and keeps to the organization's policies as the app does: the
//! work is packwing-core's. What it finds goes to standard output, as text or
//! as JSON (`--json`); what goes wrong, and how far a job is, to standard
//! error. Its exit code says how it went ([`Exit`]).

mod commands;
mod damaged;
mod names;
mod places;
mod progress;
mod secrets;
mod settings;
mod signing;

use std::io::IsTerminal as _;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, CommandFactory as _, Parser, Subcommand};

use crate::damaged::Damaging;
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
        needed, or wrong; 4 not an archive pkw can open; 5 done, but some files' data was \
        damaged (--on-damaged)."
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
    /// List what an archive holds, or a folder, here or on a storage.
    #[command(visible_alias = "ls")]
    List {
        /// An archive or a folder: a path, or NAME:PATH on a storage added to the app.
        archive: String,
        /// Only what is under these paths in it.
        paths: Vec<String>,
        /// With each one's size and date.
        #[arg(short, long)]
        long: bool,
    },
    /// Extract an archive, or files from it.
    #[command(visible_alias = "x")]
    Extract {
        /// A path, or NAME:PATH on a storage added to the app.
        archive: String,
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
        /// On Windows and macOS: no mark that the files came from the Internet, where
        /// the archive has one, as far as the organization's policy allows.
        #[arg(long)]
        no_mark: bool,
        #[command(flatten)]
        damaging: Damaging,
    },
    /// Check that an archive reads whole, as it was made.
    #[command(visible_alias = "t")]
    Test {
        /// A path, or NAME:PATH on a storage added to the app.
        archive: String,
    },
    /// Write a file in an archive to standard output.
    Cat {
        /// A path, or NAME:PATH on a storage added to the app.
        archive: String,
        file: String,
        /// Read at most this much.
        #[arg(long, value_name = "SIZE", default_value = "10GB")]
        limit: String,
    },
    /// Say what an archive is: its format, layers, size and encryption.
    Info {
        /// A path, or NAME:PATH on a storage added to the app.
        archive: String,
    },
    /// Make an archive of files and folders.
    #[command(visible_alias = "c")]
    Create {
        /// The archive to make: its extension says its format, unless --format does.
        archive: PathBuf,
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[command(flatten)]
        make: Making,
    },
    /// Make an archive of what another holds, in another format: `pkw convert a.rar a.tar.zst`.
    ///
    /// Each entry goes from one into the other as it is read, with no folder in between,
    /// through the checks extracting makes. A hard link in a ZIP or 7z archive is a
    /// symbolic link to its file. The new archive keeps the mark of one from the Internet.
    Convert {
        /// The archive to read.
        from: PathBuf,
        /// The archive to make: its extension says its format, unless --format does.
        to: PathBuf,
        /// Only what is under these paths, or has these names: `docs/**`, `*.pdf`; again
        /// for more. `*` and `?` stay within a name, `**` is any folders.
        #[arg(long, value_name = "PATTERN")]
        include: Vec<String>,
        /// Not what is under these paths, or has these names: `**/node_modules`, `*.log`;
        /// again for more. It wins over --include.
        #[arg(long, value_name = "PATTERN")]
        exclude: Vec<String>,
        #[command(flatten)]
        make: Making,
        #[command(flatten)]
        damaging: Damaging,
    },
    /// Make a key to sign files with: Ed25519 and ML-DSA-65 together, its file
    /// encrypted with a passphrase, and its public key beside it, as .pub.
    Keygen {
        /// The folder it goes in, under the first name free.
        #[arg(short, long, value_name = "DIR", default_value = ".")]
        output: PathBuf,
    },
    /// Sign files: each one's signature goes beside it, as NAME.sig.
    Sign {
        #[arg(required = true)]
        files: Vec<String>,
        /// The key to sign with: an SSH Ed25519 key, or one `pkw keygen` made.
        #[arg(long, value_name = "KEY", required = true)]
        by: PathBuf,
    },
    /// Check files' signatures against the keys trusted: the app's, and those given.
    Verify {
        #[arg(required = true)]
        files: Vec<String>,
        /// Trust this public key too, from a .pub file; again for more.
        #[arg(long, value_name = "KEY.pub")]
        trust: Vec<PathBuf>,
        /// Trust only those given, not the app's.
        #[arg(long)]
        only: bool,
    },
    /// The storages added to the app, which NAME:PATH names.
    Storages,
    /// Copy files and folders, between this computer and storages, or storages.
    #[command(visible_alias = "cp")]
    Copy {
        /// What to copy, then the folder it goes in: paths, or NAME:PATH.
        #[arg(required = true, num_args = 2..)]
        places: Vec<String>,
        /// When a name is taken: stop, replace or skip.
        #[arg(long, value_name = "WHAT", default_value = "stop")]
        on_existing: String,
        /// On Windows and macOS: no mark that the files came from the Internet, for what
        /// comes from a storage or has one, as far as the organization's policy allows.
        #[arg(long)]
        no_mark: bool,
    },
    /// Move files and folders, as copy does, taking them away once they are there.
    #[command(visible_alias = "mv")]
    Move {
        /// What to move, then the folder it goes in: paths, or NAME:PATH.
        #[arg(required = true, num_args = 2..)]
        places: Vec<String>,
        /// When a name is taken: stop, replace or skip.
        #[arg(long, value_name = "WHAT", default_value = "stop")]
        on_existing: String,
        /// On Windows and macOS: no mark that the files came from the Internet, for what
        /// comes from a storage or has one, as far as the organization's policy allows.
        #[arg(long)]
        no_mark: bool,
    },
    /// Write the shell's completions for pkw, to load or keep where it looks.
    #[command(
        after_help = "bash: pkw completions bash > ~/.local/share/bash-completion/completions/pkw\n\
        zsh: pkw completions zsh > \"${fpath[1]}/_pkw\"\n\
        fish: pkw completions fish > ~/.config/fish/completions/pkw.fish\n\
        PowerShell: pkw completions powershell | Out-String | Invoke-Expression, in $PROFILE"
    )]
    Completions {
        #[arg(value_name = "SHELL")]
        shell: clap_complete::Shell,
    },
    /// Write pkw's manual pages, pkw.1 and one for each command, into a folder.
    #[command(hide = true)]
    Man {
        #[arg(value_name = "DIR")]
        folder: PathBuf,
    },
    /// Suggest a passphrase of random words, stronger than most typed ones.
    Passphrase {
        /// At least this many characters; the organization may ask for more.
        #[arg(long, value_name = "N", default_value_t = 0)]
        min: usize,
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
    Rename {
        archive: PathBuf,
        file: String,
        /// Its new name, in the same folder.
        name: String,
    },
    /// Make or check the checksums of files: sha256, sha512, sha1, md5, blake3.
    #[command(visible_alias = "sum")]
    Checksum {
        /// Paths, or NAME:PATH on a storage added to the app.
        files: Vec<String>,
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

/// How an archive is made, by `create` and `convert`.
#[derive(Args)]
pub struct Making {
    /// zip, 7z, tar, tar.zst, tar.gz, tar.xz, tar.bz2, tar.lz4, or one file alone:
    /// zst, gz, xz, bz2, lz4.
    #[arg(short, long, value_name = "FORMAT")]
    pub format: Option<String>,
    /// store, lowest, low, medium, high or highest (0 to 5).
    #[arg(short, long, value_name = "LEVEL", default_value = "medium")]
    pub level: String,
    /// Protect it with a password: asked, or read as --password-file says.
    #[arg(short, long)]
    pub password: bool,
    /// Encrypt it whole with age to this public key (age1…, age1pq1…, ssh-ed25519 …);
    /// again for more.
    #[arg(long = "encrypt-to", value_name = "KEY")]
    pub encrypt_to: Vec<String>,
    /// A ZIP or 7z archive encrypted whole with age, not with its own AES-256.
    #[arg(long)]
    pub age: bool,
    /// A 7z archive's names hidden behind its password too.
    #[arg(long)]
    pub hide_names: bool,
    /// Cut it into parts of this size, as 25MB.
    #[arg(long, value_name = "SIZE")]
    pub split: Option<String>,
    /// Compress pictures, video, sound and archives too, not only store them.
    #[arg(long)]
    pub compress_all: bool,
    /// Keep what only a Mac uses, which is left out otherwise: .DS_Store,
    /// __MACOSX and ._ files.
    #[arg(long)]
    pub keep_mac_files: bool,
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
    /// Done, but some files' data was damaged, and skipped or kept: 5.
    Damaged(String),
}

impl Exit {
    fn code(&self) -> u8 {
        match self {
            Exit::Failed(_) => 1,
            Exit::Usage(_) => 2,
            Exit::Locked(_) => 3,
            Exit::NotAnArchive(_) => 4,
            Exit::Damaged(_) => 5,
        }
    }

    fn message(&self) -> &str {
        match self {
            Exit::Failed(m)
            | Exit::Usage(m)
            | Exit::Locked(m)
            | Exit::NotAnArchive(m)
            | Exit::Damaged(m) => m,
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
    /// A person is at the terminal, to be asked what to do, and standard input
    /// is theirs.
    pub can_ask: bool,
    pub secrets: Secrets,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    // What is written of pkw itself, with nothing to read or ask.
    match &cli.command {
        Command::Completions { shell } => {
            clap_complete::generate(*shell, &mut Cli::command(), "pkw", &mut std::io::stdout());
            return ExitCode::SUCCESS;
        }
        Command::Man { folder } => {
            return match man_pages(folder) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => fail(&Exit::Failed(format!("cannot write the pages: {e}"))),
            };
        }
        _ => {}
    }
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
        can_ask: !cli.output.quiet
            && !cli.secrets.password_stdin
            && std::io::stdin().is_terminal()
            && std::io::stderr().is_terminal(),
        secrets,
    };
    match commands::run(cli.command, &mut cx) {
        Ok(()) => ExitCode::SUCCESS,
        Err(exit) => fail(&exit),
    }
}

/// `pkw.1`, and `pkw-COMMAND.1` for each command shown, into `folder`.
fn man_pages(folder: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(folder)?;
    let mut pkw = Cli::command().version(env!("CARGO_PKG_VERSION"));
    pkw.build();
    let write = |name: &str, command: clap::Command| {
        let mut page = Vec::new();
        clap_mangen::Man::new(command.name(name.to_owned()))
            .render(&mut page)
            .and_then(|()| std::fs::write(folder.join(format!("{name}.1")), page))
    };
    for command in pkw
        .get_subcommands()
        .filter(|c| !c.is_hide_set() && c.get_name() != "help")
    {
        write(&format!("pkw-{}", command.get_name()), command.clone())?;
    }
    write("pkw", pkw)
}

fn fail(exit: &Exit) -> ExitCode {
    eprintln!("pkw: {}", exit.message());
    ExitCode::from(exit.code())
}
