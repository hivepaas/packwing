//! A storage through OpenDAL, whose operators speak S3, WebDAV and others in one
//! way. A kind says how to make its operator, and whether an upload can go in parts.
//! A folder is not renamed at once: OpenDAL does not offer it, and on S3 it would be
//! every object under it. Move to… goes a file at a time.
//!
//! Everything blocks, as `Storage` asks: on a runtime of its own. A folder is a
//! prefix that something is under, or an empty one whose key ends in `/`.

use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::OnceLock;

use futures::StreamExt;
use opendal::{EntryMode, ErrorKind, Lister as OdLister, Operator};
use packwing_core::{Item, Lister, Storage, bad_name, by_name};
use packwing_i18n::tr;
use tokio::runtime::Runtime;

/// How much is asked for at once when reading a file through.
const READ_RANGE: u64 = 8 << 20;
/// How much is sent at a time.
const PART: usize = 16 << 20;
/// How many items a listing asks for at a time.
const LIST_PAGE: usize = 1000;

pub(crate) fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("a runtime for storages")
    })
}

fn said(e: opendal::Error) -> io::Error {
    let kind = match e.kind() {
        ErrorKind::NotFound => io::ErrorKind::NotFound,
        ErrorKind::PermissionDenied => io::ErrorKind::PermissionDenied,
        ErrorKind::AlreadyExists => io::ErrorKind::AlreadyExists,
        _ => io::ErrorKind::Other,
    };
    io::Error::new(kind, e.to_string())
}

fn taken() -> io::Error {
    io::Error::new(
        io::ErrorKind::AlreadyExists,
        tr!("That name is taken in this folder."),
    )
}

fn too_big(most: u64) -> io::Error {
    io::Error::other(tr!(
        "This storage takes files of at most {size} for now, as each is held in memory while it is sent.",
        size = format!("{} MB", most / 1_000_000)
    ))
}

/// How an operator is made: when first needed, as that reads the keychain.
pub type MakeOperator = Box<dyn Fn() -> io::Result<Operator> + Send + Sync>;

/// A storage through an OpenDAL operator.
pub struct Dal {
    make: MakeOperator,
    op: OnceLock<io::Result<Operator>>,
    /// Under this prefix only; empty for all there is.
    prefix: String,
    /// What a path shown starts with.
    shown: String,
    /// The most a file may be when an upload cannot go in parts and has to be
    /// held in memory; none when it can.
    most_held: Option<u64>,
}

impl Dal {
    pub fn new(make: MakeOperator, prefix: &str, shown: String, most_held: Option<u64>) -> Dal {
        Dal {
            make,
            op: OnceLock::new(),
            prefix: prefix.trim_matches('/').to_owned(),
            shown,
            most_held,
        }
    }

    fn op(&self) -> io::Result<&Operator> {
        self.op
            .get_or_init(|| (self.make)())
            .as_ref()
            .map_err(|e| io::Error::new(e.kind(), e.to_string()))
    }

    /// The key of `path`, with no `/` at the end.
    fn key(&self, path: &[String]) -> io::Result<String> {
        let mut key = self.prefix.clone();
        for step in path {
            if let Some(why) = bad_name(step) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, why));
            }
            if !key.is_empty() {
                key.push('/');
            }
            key.push_str(step);
        }
        Ok(key)
    }

    /// The prefix of what is in the folder `path`.
    fn folder_key(&self, path: &[String]) -> io::Result<String> {
        let key = self.key(path)?;
        Ok(if key.is_empty() { key } else { key + "/" })
    }

    fn joined(folder: &[String], name: &str) -> Vec<String> {
        let mut path = folder.to_vec();
        path.push(name.to_owned());
        path
    }

    /// Whether anything is under the prefix: if so, it is a folder.
    fn is_folder(&self, path: &[String]) -> io::Result<bool> {
        let op = self.op()?.clone();
        let key = self.folder_key(path)?;
        runtime().block_on(async move {
            let mut lister = match op.lister_with(&key).limit(1).await {
                Ok(lister) => lister,
                Err(e) if e.kind() == ErrorKind::NotFound => return Ok(false),
                Err(e) => return Err(said(e)),
            };
            while let Some(entry) = lister.next().await {
                let entry = entry.map_err(said)?;
                if entry.path().trim_start_matches('/') != key.trim_start_matches('/') {
                    return Ok(true);
                }
            }
            // A marker alone: a folder with nothing in it.
            op.exists(&key).await.map_err(said)
        })
    }
}

fn item_of(name: String, dir: bool, size: u64, modified: Option<i64>) -> Item {
    Item {
        name,
        dir,
        link: false,
        hidden: false,
        size,
        modified,
        mode: None,
    }
}

struct DalLister {
    lister: OdLister,
    folder_key: String,
    left: Option<Item>,
    /// The stream has ended.
    ended: bool,
}

impl DalLister {
    /// The next item of the stream, leaving out the folder's own marker.
    fn pull(&mut self) -> io::Result<Option<Item>> {
        if self.ended {
            return Ok(None);
        }
        let (lister, folder_key) = (&mut self.lister, &self.folder_key);
        let next = runtime().block_on(async {
            while let Some(entry) = lister.next().await {
                let entry = entry.map_err(said)?;
                let path = entry.path().trim_start_matches('/');
                if path.is_empty() || path == folder_key.trim_start_matches('/') {
                    continue;
                }
                let meta = entry.metadata();
                let dir = meta.mode() == EntryMode::DIR;
                let name = entry.name().trim_end_matches('/').to_owned();
                let modified = meta.last_modified().map(|t| t.into_inner().as_second());
                return Ok::<_, io::Error>(Some(item_of(
                    name,
                    dir,
                    if dir { 0 } else { meta.content_length() },
                    modified,
                )));
            }
            Ok(None)
        })?;
        if next.is_none() {
            self.ended = true;
        }
        Ok(next)
    }
}

impl Lister for DalLister {
    fn next(&mut self, most: usize) -> io::Result<Vec<Item>> {
        let mut items = Vec::new();
        while items.len() < most {
            let item = match self.left.take() {
                Some(item) => Some(item),
                None => self.pull()?,
            };
            match item {
                Some(item) => items.push(item),
                None => break,
            }
        }
        // Keys come in byte order, folders among them: within a page, in the order
        // files are read, folders first.
        items.sort_by(|a, b| b.dir.cmp(&a.dir).then_with(|| by_name(&a.name, &b.name)));
        if self.left.is_none() {
            self.left = self.pull()?;
        }
        Ok(items)
    }

    fn done(&self) -> bool {
        self.ended && self.left.is_none()
    }
}

/// A file read through, a range at a time.
struct RangeReader {
    op: Operator,
    key: String,
    at: u64,
    size: u64,
    buffer: Vec<u8>,
    taken: usize,
}

impl Read for RangeReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.taken == self.buffer.len() {
            if self.at >= self.size {
                return Ok(0);
            }
            let end = (self.at + READ_RANGE).min(self.size);
            let (op, key, at) = (self.op.clone(), self.key.clone(), self.at);
            self.buffer = runtime()
                .block_on(async move { op.read_with(&key).range(at..end).await })
                .map_err(said)?
                .to_vec();
            self.taken = 0;
            self.at = end;
        }
        let n = out.len().min(self.buffer.len() - self.taken);
        out[..n].copy_from_slice(&self.buffer[self.taken..self.taken + n]);
        self.taken += n;
        Ok(n)
    }
}

impl Dal {
    fn size_of(&self, op: &Operator, key: &str) -> io::Result<u64> {
        let (op, key) = (op.clone(), key.to_owned());
        Ok(runtime()
            .block_on(async move { op.stat(&key).await })
            .map_err(said)?
            .content_length())
    }

    /// Sends `from` to `key`, in parts where that is possible. Stopped or failed,
    /// what was sent goes, and nothing appears.
    fn send(&self, op: &Operator, key: &str, from: &mut dyn Read) -> io::Result<u64> {
        // Sent whole: held in memory first, so that a failed read sends nothing.
        if let Some(most) = self.most_held {
            let mut whole = Vec::new();
            from.take(most + 1).read_to_end(&mut whole)?;
            if whole.len() as u64 > most {
                return Err(too_big(most));
            }
            let written = whole.len() as u64;
            return runtime()
                .block_on(async { op.write(key, whole).await })
                .map_err(said)
                .map(|_| written);
        }
        let mut writer = runtime()
            .block_on(async { op.writer_with(key).chunk(PART).await })
            .map_err(said)?;
        let mut written = 0;
        let mut buffer = vec![0; PART];
        let copied = loop {
            let mut filled = 0;
            let mut failed = None;
            while filled < PART {
                match from.read(&mut buffer[filled..]) {
                    Ok(0) => break,
                    Ok(n) => filled += n,
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                    Err(e) => {
                        failed = Some(e);
                        break;
                    }
                }
            }
            if let Some(e) = failed {
                break Err(e);
            }
            if filled == 0 {
                break Ok(());
            }
            written += filled as u64;
            let part = buffer[..filled].to_vec();
            if let Err(e) = runtime().block_on(async { writer.write(part).await }) {
                break Err(said(e));
            }
        };
        match copied {
            Ok(()) => runtime()
                .block_on(async { writer.close().await })
                .map_err(said)
                .map(|_| written),
            Err(e) => {
                let _ = runtime().block_on(async { writer.abort().await });
                Err(e)
            }
        }
    }
}

impl Storage for Dal {
    fn list(&self, folder: &[String]) -> io::Result<Box<dyn Lister>> {
        let op = self.op()?.clone();
        let key = self.folder_key(folder)?;
        let at = key.clone();
        let lister = runtime()
            .block_on(async move { op.lister_with(&at).limit(LIST_PAGE).await })
            .map_err(said)?;
        Ok(Box::new(DalLister {
            lister,
            folder_key: key,
            left: None,
            ended: false,
        }))
    }

    fn read(&self, path: &[String], most: u64) -> io::Result<Vec<u8>> {
        let op = self.op()?.clone();
        let key = self.key(path)?;
        let size = self.size_of(&op, &key)?;
        if size == 0 {
            return Ok(Vec::new());
        }
        let end = most.min(size);
        let got = runtime()
            .block_on(async move { op.read_with(&key).range(0..end).await })
            .map_err(said)?;
        Ok(got.to_vec())
    }

    fn stat(&self, folder: &[String], name: &str) -> io::Result<Option<Item>> {
        let op = self.op()?.clone();
        let path = Self::joined(folder, name);
        let key = self.key(&path)?;
        let found = runtime().block_on(async move { op.stat(&key).await });
        match found {
            Ok(meta) if meta.mode() == EntryMode::DIR => {
                Ok(Some(item_of(name.to_owned(), true, 0, None)))
            }
            Ok(meta) => Ok(Some(item_of(
                name.to_owned(),
                false,
                meta.content_length(),
                meta.last_modified().map(|t| t.into_inner().as_second()),
            ))),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(self
                .is_folder(&path)?
                .then(|| item_of(name.to_owned(), true, 0, None))),
            Err(e) => Err(said(e)),
        }
    }

    fn open(&self, path: &[String]) -> io::Result<Box<dyn Read + Send>> {
        let op = self.op()?.clone();
        let key = self.key(path)?;
        let size = self.size_of(&op, &key)?;
        Ok(Box::new(RangeReader {
            op,
            key,
            at: 0,
            size,
            buffer: Vec::new(),
            taken: 0,
        }))
    }

    fn put(
        &self,
        path: &[String],
        from: &mut dyn Read,
        like: &Item,
        replace: bool,
    ) -> io::Result<u64> {
        if let Some(most) = self.most_held
            && like.size > most
        {
            return Err(too_big(most));
        }
        let op = self.op()?.clone();
        let key = self.key(path)?;
        if !replace {
            let (o, k) = (op.clone(), key.clone());
            if runtime()
                .block_on(async move { o.exists(&k).await })
                .map_err(said)?
            {
                return Err(taken());
            }
        }
        self.send(&op, &key, from)
    }

    fn remove(&self, path: &[String]) -> io::Result<()> {
        let op = self.op()?.clone();
        let (file, folder) = (self.key(path)?, self.folder_key(path)?);
        let is_dir = self.is_folder(path)?;
        runtime().block_on(async move {
            if is_dir {
                op.delete(&folder).await.map_err(said)
            } else {
                op.delete(&file).await.map_err(said)
            }
        })
    }

    fn make_folder(&self, folder: &[String], name: &str) -> io::Result<()> {
        if self.stat(folder, name)?.is_some() {
            return Err(taken());
        }
        let op = self.op()?.clone();
        let key = self.folder_key(&Self::joined(folder, name))?;
        runtime()
            .block_on(async move { op.create_dir(&key).await })
            .map_err(said)
    }

    fn rename(&self, folder: &[String], from: &str, to: &str) -> io::Result<()> {
        let Some(item) = self.stat(folder, from)? else {
            return Err(io::ErrorKind::NotFound.into());
        };
        if item.dir {
            return Err(io::Error::other(tr!(
                "This storage cannot rename a folder at once, as everything under it would be copied. Use Move to…."
            )));
        }
        if self.stat(folder, to)?.is_some() {
            return Err(taken());
        }
        let op = self.op()?.clone();
        let (a, b) = (
            self.key(&Self::joined(folder, from))?,
            self.key(&Self::joined(folder, to))?,
        );
        let native = op.info().capability().rename;
        runtime().block_on(async move {
            if native {
                return op.rename(&a, &b).await.map_err(said);
            }
            op.copy(&a, &b).await.map_err(said)?;
            op.delete(&a).await.map_err(said)
        })
    }

    fn delete(&self, folder: &[String], names: &[String]) -> io::Result<()> {
        for name in names {
            let path = Self::joined(folder, name);
            let Some(item) = self.stat(folder, name)? else {
                continue;
            };
            let op = self.op()?.clone();
            let (key, dir) = if item.dir {
                (self.folder_key(&path)?, true)
            } else {
                (self.key(&path)?, false)
            };
            runtime()
                .block_on(async move {
                    if dir {
                        op.delete_with(&key).recursive(true).await
                    } else {
                        op.delete(&key).await
                    }
                })
                .map_err(said)?;
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
        let key = self.key(path).unwrap_or_default();
        format!("{}{key}", self.shown)
    }
}
