//! A folder on a server over SFTP, through shellwing's SSH: `ssh_config`, the agent
//! and key files, jump hosts. It is checked strictly against known_hosts, as there
//! is nowhere to ask yet: a host not in it is refused.
//!
//! Everything blocks, as `Storage` asks. A connection is made when first needed,
//! and again after one is lost.

use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use packwing_core::{
    Field, Item, Lister, Storage, StorageKind, StoredFile, Values, bad_name, by_name,
};
use packwing_i18n::tr;
use shellwing_ssh::sftp::{Entry, Kind, join};
use shellwing_ssh::{
    Auth, FilesConnection, HostKeyPolicy, Learned, Options, RemoteFile, Secrets, Sftp, config,
    connect_files,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::dal::runtime;
use crate::secrets;

/// What a server is reached with.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SftpSettings {
    /// `[user@]host[:port]`, or a name in `ssh_config`.
    pub host: String,
    /// Where it starts; empty for the home folder.
    pub start: String,
    /// Empty to go by the agent and key files alone.
    pub password: String,
    /// Key files to offer, in place of `ssh_config`'s (tests).
    pub key_file: Option<PathBuf>,
    /// In place of `~/.ssh/known_hosts` (tests).
    pub known_hosts: Option<PathBuf>,
}

/// The password, for any target: a storage has one server.
struct Password(String);

impl Secrets for Password {
    fn password(&self, _: &str) -> Option<String> {
        Some(self.0.clone()).filter(|p| !p.is_empty())
    }

    fn passphrase(&self, _: &str) -> Option<String> {
        None
    }

    fn learned(&self, _: Learned) {}
}

/// A connection, and the folder things are under.
struct Open {
    connection: FilesConnection,
    top: String,
}

/// A folder on a server as a storage.
pub struct SftpStorage {
    settings: SftpSettings,
    /// Where the password is kept, unless it was typed.
    id: Option<String>,
    open: Mutex<Option<Arc<Open>>>,
}

fn said(e: shellwing_ssh::Error) -> io::Error {
    io::Error::other(e.to_string())
}

fn taken() -> io::Error {
    io::Error::new(
        io::ErrorKind::AlreadyExists,
        tr!("That name is taken in this folder."),
    )
}

/// A storage over SFTP. With `id`, its password is the one kept under it.
pub fn sftp(settings: SftpSettings, id: Option<String>) -> SftpStorage {
    SftpStorage {
        settings,
        id,
        open: Mutex::new(None),
    }
}

impl SftpStorage {
    /// The connection, made if there is none.
    fn connected(&self) -> io::Result<Arc<Open>> {
        let mut open = self.open.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(open) = open.as_ref() {
            return Ok(open.clone());
        }
        let s = &self.settings;
        let password = match &self.id {
            Some(id) => secrets::get(id, "password")?,
            None => s.password.clone(),
        };
        let mut target =
            config::resolve_without_commands(&s.host, config::default_config_path().as_deref())
                .map_err(said)?;
        if let Some(key) = &s.key_file {
            target.identity_files = vec![key.clone()];
            target.identities_only = true;
        }
        let options = Options {
            target,
            auth: Auth {
                agent: true,
                password: Some(password.clone()).filter(|p| !p.is_empty()),
                agent_socket: None,
            },
            policy: HostKeyPolicy::Strict,
            known_hosts: s.known_hosts.clone(),
            prompter: None,
            secrets: Some(Arc::new(Password(password))),
            forwarded: None,
            keepalive: None,
            size: None,
            cols: 80,
            rows: 24,
            term: String::new(),
        };
        let start = s.start.clone();
        let made = runtime().block_on(async move {
            let connection = connect_files(options).await?;
            let sftp = connection.sftp();
            let top = if start.is_empty() {
                sftp.home().await?
            } else {
                sftp.expand(&start).await?
            };
            Ok::<_, shellwing_ssh::Error>(Open { connection, top })
        });
        let made = Arc::new(made.map_err(said)?);
        *open = Some(made.clone());
        Ok(made)
    }

    /// Does `work` on the connection. If it fails and the connection is gone, the
    /// next use makes another.
    fn with<T>(
        &self,
        work: impl AsyncFnOnce(&Sftp, &str) -> Result<T, shellwing_ssh::Error>,
    ) -> io::Result<T> {
        let open = self.connected()?;
        let done = runtime().block_on(work(open.connection.sftp(), &open.top));
        if done.is_err() {
            let alive = runtime().block_on(open.connection.sftp().home()).is_ok();
            if !alive {
                *self.open.lock().unwrap_or_else(|e| e.into_inner()) = None;
            }
        }
        done.map_err(said)
    }

    /// The path of `path` under `top`.
    fn at(top: &str, path: &[String]) -> io::Result<String> {
        let mut at = top.to_owned();
        for step in path {
            if let Some(why) = bad_name(step) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, why));
            }
            at = join(&at, step);
        }
        Ok(at)
    }

    fn checked(path: &[String]) -> io::Result<()> {
        Self::at("", path).map(|_| ())
    }
}

fn seconds(when: Option<std::time::SystemTime>) -> Option<i64> {
    when?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64)
}

fn item_of(e: Entry) -> Item {
    Item {
        hidden: e.name.starts_with('.'),
        dir: e.is_dir(),
        link: e.kind == Kind::Link,
        size: if e.is_dir() { 0 } else { e.size },
        modified: seconds(e.modified),
        mode: None,
        name: e.name,
    }
}

/// SFTP gives a folder whole: what is left of it, a page at a time.
struct SftpLister(std::vec::IntoIter<Item>);

impl Lister for SftpLister {
    fn next(&mut self, most: usize) -> io::Result<Vec<Item>> {
        Ok(self.0.by_ref().take(most).collect())
    }

    fn done(&self) -> bool {
        self.0.len() == 0
    }
}

/// A file read through. It keeps the connection it is read over.
struct SftpReader {
    file: RemoteFile,
    _open: Arc<Open>,
}

impl Read for SftpReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        runtime().block_on(self.file.read(out))
    }
}

/// A name for a file on its way, in the folder it goes to.
fn part_name(to: &str) -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let dir = shellwing_ssh::sftp::parent(to);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    join(&dir, &format!(".packwing-part-{}-{n}", std::process::id()))
}

/// The part file written gets `like`'s details, and goes to its name.
fn settle(sftp: &Sftp, part: &str, to: &str, like: &Item, there: bool) -> io::Result<()> {
    let modified = like.modified.and_then(|s| u32::try_from(s).ok());
    let mode = like.mode.map(|m| m & 0o777);
    if modified.is_some() || mode.is_some() {
        runtime()
            .block_on(sftp.set_details(part, mode, modified))
            .map_err(said)?;
    }
    // SFTP's rename does not write over a file: what is there moves aside first,
    // and comes back if the rename fails.
    let aside = format!("{part}.old");
    if there {
        runtime().block_on(sftp.rename(to, &aside)).map_err(said)?;
    }
    if let Err(e) = runtime().block_on(sftp.rename(part, to)) {
        if there {
            let _ = runtime().block_on(sftp.rename(&aside, to));
        }
        return Err(said(e));
    }
    if there {
        let _ = runtime().block_on(sftp.remove_file(&aside));
    }
    Ok(())
}

/// How much is held before it is written: a write waits for the server's answer.
const FILE_BUFFER: usize = 256 << 10;

/// A new file on the server, under a part file's name till it is finished.
struct SftpFile {
    file: RemoteFile,
    buffer: Vec<u8>,
    part: String,
    to: String,
    like: Item,
    length: u64,
    position: u64,
    settled: bool,
    open: Arc<Open>,
}

impl SftpFile {
    fn write_out(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            runtime().block_on(self.file.write_all(&self.buffer))?;
            self.buffer.clear();
        }
        Ok(())
    }
}

impl Write for SftpFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.buffer.len() + buf.len() > FILE_BUFFER {
            self.write_out()?;
        }
        if buf.len() >= FILE_BUFFER {
            runtime().block_on(self.file.write_all(buf))?;
        } else {
            self.buffer.extend_from_slice(buf);
        }
        self.position += buf.len() as u64;
        self.length = self.length.max(self.position);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.write_out()
    }
}

impl Seek for SftpFile {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let position = match to {
            SeekFrom::Start(p) => Some(p),
            SeekFrom::Current(d) => self.position.checked_add_signed(d),
            SeekFrom::End(d) => self.length.checked_add_signed(d),
        }
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek before 0"))?;
        // Telling where it is, as a writer often asks, costs nothing.
        if position != self.position {
            self.write_out()?;
            runtime().block_on(self.file.seek(SeekFrom::Start(position)))?;
            self.position = position;
        }
        Ok(position)
    }
}

impl StoredFile for SftpFile {
    fn finish(mut self: Box<Self>) -> io::Result<u64> {
        self.write_out()?;
        runtime().block_on(self.file.shutdown())?;
        settle(
            self.open.connection.sftp(),
            &self.part,
            &self.to,
            &self.like,
            false,
        )?;
        self.settled = true;
        Ok(self.length)
    }
}

impl Drop for SftpFile {
    fn drop(&mut self) {
        if !self.settled {
            let _ = runtime().block_on(self.open.connection.sftp().remove_file(&self.part));
        }
    }
}

impl Storage for SftpStorage {
    fn list(&self, folder: &[String]) -> io::Result<Box<dyn Lister>> {
        Self::checked(folder)?;
        let mut items: Vec<Item> = self.with(async |sftp, top| {
            let at = Self::at(top, folder).expect("checked");
            Ok(sftp.list(&at).await?.into_iter().map(item_of).collect())
        })?;
        items.sort_by(|a, b| b.dir.cmp(&a.dir).then_with(|| by_name(&a.name, &b.name)));
        Ok(Box::new(SftpLister(items.into_iter())))
    }

    fn read(&self, path: &[String], most: u64) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        Storage::open(self, path)?
            .take(most)
            .read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    fn read_at(&self, path: &[String], offset: u64, len: usize) -> io::Result<Vec<u8>> {
        Self::checked(path)?;
        let open = self.connected()?;
        let at = Self::at(&open.top, path)?;
        let sftp = open.connection.sftp();
        runtime().block_on(async {
            let mut file = sftp.reader(&at).await.map_err(said)?;
            file.seek(std::io::SeekFrom::Start(offset)).await?;
            let mut bytes = Vec::with_capacity(len);
            (&mut file).take(len as u64).read_to_end(&mut bytes).await?;
            Ok(bytes)
        })
    }

    fn stat(&self, folder: &[String], name: &str) -> io::Result<Option<Item>> {
        let mut path = folder.to_vec();
        path.push(name.to_owned());
        Self::checked(&path)?;
        self.with(async |sftp, top| {
            let at = Self::at(top, &path).expect("checked");
            let Some(kind) = sftp.lstat(&at).await? else {
                return Ok(None);
            };
            // One whose link leads nowhere is the link alone.
            let found = sftp.stat(&at).await?;
            let dir = found.as_ref().is_some_and(Entry::is_dir);
            Ok(Some(Item {
                name: name.to_owned(),
                dir,
                link: kind == Kind::Link,
                hidden: name.starts_with('.'),
                size: found.as_ref().filter(|_| !dir).map_or(0, |e| e.size),
                modified: seconds(found.and_then(|e| e.modified)),
                mode: None,
            }))
        })
    }

    fn open(&self, path: &[String]) -> io::Result<Box<dyn Read + Send>> {
        Self::checked(path)?;
        let open = self.connected()?;
        let at = Self::at(&open.top, path)?;
        let file = runtime()
            .block_on(open.connection.sftp().reader(&at))
            .map_err(said)?;
        Ok(Box::new(SftpReader { file, _open: open }))
    }

    fn put(
        &self,
        path: &[String],
        from: &mut dyn Read,
        like: &Item,
        replace: bool,
    ) -> io::Result<u64> {
        Self::checked(path)?;
        let open = self.connected()?;
        let sftp = open.connection.sftp();
        let to = Self::at(&open.top, path)?;
        let there = runtime().block_on(sftp.lstat(&to)).map_err(said)?.is_some();
        if there && !replace {
            return Err(taken());
        }
        let part = part_name(&to);
        let sent = (|| {
            let mut file = runtime().block_on(sftp.writer(&part)).map_err(said)?;
            let mut buffer = vec![0; 256 << 10];
            let mut written = 0;
            loop {
                let n = match from.read(&mut buffer) {
                    Ok(n) => n,
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                };
                if n == 0 {
                    break;
                }
                runtime().block_on(file.write_all(&buffer[..n]))?;
                written += n as u64;
            }
            runtime().block_on(file.shutdown())?;
            settle(sftp, &part, &to, like, there)?;
            Ok(written)
        })();
        if sent.is_err() {
            let _ = runtime().block_on(sftp.remove_file(&part));
        }
        sent
    }

    fn sends_as_read(&self) -> bool {
        true
    }

    fn writes_anywhere(&self) -> bool {
        true
    }

    fn new_file(&self, path: &[String], like: &Item) -> io::Result<Box<dyn StoredFile>> {
        Self::checked(path)?;
        let open = self.connected()?;
        let sftp = open.connection.sftp();
        let to = Self::at(&open.top, path)?;
        if runtime().block_on(sftp.lstat(&to)).map_err(said)?.is_some() {
            return Err(taken());
        }
        let part = part_name(&to);
        let file = runtime().block_on(sftp.writer(&part)).map_err(said)?;
        Ok(Box::new(SftpFile {
            file,
            buffer: Vec::with_capacity(FILE_BUFFER),
            part,
            to,
            like: like.clone(),
            length: 0,
            position: 0,
            settled: false,
            open,
        }))
    }

    fn remove(&self, path: &[String]) -> io::Result<()> {
        Self::checked(path)?;
        self.with(async |sftp, top| {
            let at = Self::at(top, path).expect("checked");
            if sftp.lstat(&at).await? == Some(Kind::Dir) {
                sftp.remove_empty_dir(&at).await
            } else {
                sftp.remove_file(&at).await
            }
        })
    }

    fn make_folder(&self, folder: &[String], name: &str) -> io::Result<()> {
        let mut path = folder.to_vec();
        path.push(name.to_owned());
        Self::checked(&path)?;
        self.with(async |sftp, top| sftp.mkdir(&Self::at(top, &path).expect("checked")).await)
    }

    fn rename(&self, folder: &[String], from: &str, to: &str) -> io::Result<()> {
        let (mut a, mut b) = (folder.to_vec(), folder.to_vec());
        a.push(from.to_owned());
        b.push(to.to_owned());
        Self::checked(&a)?;
        Self::checked(&b)?;
        self.with(async |sftp, top| {
            sftp.rename(
                &Self::at(top, &a).expect("checked"),
                &Self::at(top, &b).expect("checked"),
            )
            .await
        })
    }

    fn delete(&self, folder: &[String], names: &[String]) -> io::Result<()> {
        for name in names {
            let mut path = folder.to_vec();
            path.push(name.clone());
            Self::checked(&path)?;
            self.with(async |sftp, top| {
                sftp.remove(&Self::at(top, &path).expect("checked")).await
            })?;
        }
        Ok(())
    }

    fn has_trash(&self) -> bool {
        false
    }

    fn local_path(&self, _: &[String]) -> Option<PathBuf> {
        None
    }

    fn show_path(&self, path: &[String]) -> String {
        let top = self
            .open
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .map_or_else(|| self.settings.start.clone(), |o| o.top.clone());
        let at = Self::at(&top, path).unwrap_or(top);
        format!("sftp://{}{at}", self.settings.host)
    }
}

/// Servers over SFTP as a kind of storage.
pub struct SftpKind;

impl StorageKind for SftpKind {
    fn id(&self) -> &'static str {
        "sftp"
    }

    fn tag(&self) -> &'static str {
        "SSH"
    }

    fn name(&self) -> String {
        tr!("SFTP")
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field {
                key: "host",
                label: tr!("Host or ssh_config name"),
                hint: "deploy@build.example.com".to_owned(),
                secret: false,
            },
            Field {
                key: "start",
                label: tr!("Start in"),
                hint: tr!("Empty for the home folder"),
                secret: false,
            },
            Field {
                key: "password",
                label: tr!("Password"),
                hint: tr!("Empty to use the SSH agent and keys"),
                secret: true,
            },
        ]
    }

    fn describe(&self, values: &Values) -> String {
        format!("SFTP · {}", values.get("host").map_or("", String::as_str))
    }

    fn open(&self, id: &str, values: &Values, typed: Option<&Values>) -> Arc<dyn Storage> {
        let value = |key: &str| values.get(key).cloned().unwrap_or_default();
        let settings = SftpSettings {
            host: value("host"),
            start: value("start"),
            password: typed
                .and_then(|t| t.get("password"))
                .cloned()
                .unwrap_or_default(),
            key_file: None,
            known_hosts: None,
        };
        Arc::new(sftp(settings, typed.is_none().then(|| id.to_owned())))
    }

    fn keep_secrets(&self, id: &str, secrets: &Values) -> io::Result<()> {
        secrets::keep(id, secrets)
    }

    fn kept_secrets(&self, id: &str) -> io::Result<Values> {
        secrets::kept(id, &["password"])
    }

    fn forget_secrets(&self, id: &str) {
        secrets::forget(id, &["password"]);
    }
}
