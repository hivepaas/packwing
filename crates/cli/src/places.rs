//! Where a file is, as typed: a path here, or `NAME:PATH` on a storage added
//! to the app (an S3 bucket, an SFTP server, a WebDAV folder, a folder), its
//! secrets in the system's keychain as the app keeps them. `NAME` is the
//! storage's own, so `C:\…` is a path on Windows unless a storage is called
//! `C`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use packwing_core::{Item, LocalFolder, Source, Storage, Stored};

use crate::Exit;
use crate::names::entry_path;
use crate::settings::{Kept, storages, text};

/// A file or folder, on this computer or on a storage.
pub struct Place {
    pub storage: Arc<dyn Storage>,
    /// Its path there: empty for the storage's top.
    pub path: Vec<String>,
    /// As typed, to say it back.
    pub shown: String,
    /// It is on this computer: its path here.
    pub here: Option<PathBuf>,
}

impl Place {
    /// What `typed` names.
    pub fn of(typed: &str) -> Result<Place, Exit> {
        if let Some((name, rest)) = typed.split_once(':')
            && let Some(kept) = kept_named(name)?
        {
            let storage = kept.storage.ok_or_else(|| {
                Exit::Failed(format!(
                    "{name} is a {} storage, which pkw cannot open",
                    kept.kind
                ))
            })?;
            if kept.asks_first {
                return Err(Exit::Failed(format!(
                    "{name} asks for this computer's password before it opens: open it in packwing"
                )));
            }
            let here = storage.local_path(&[]);
            let path = entry_path(rest);
            return Ok(Place {
                here: here.map(|top| path.iter().fold(top, |p, n| p.join(n))),
                storage,
                path,
                shown: typed.to_owned(),
            });
        }
        Ok(Place::local(Path::new(typed)))
    }

    /// The file or folder at `path` on this computer.
    pub fn local(path: &Path) -> Place {
        let folder = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
        // `.` or `/`: the folder itself, as its storage's top.
        let (storage, entry): (Arc<dyn Storage>, Vec<String>) = match name {
            Some(name) => (Arc::new(LocalFolder::new(folder)), vec![name]),
            None => (Arc::new(LocalFolder::new(path)), Vec::new()),
        };
        Place {
            storage,
            path: entry,
            shown: path.display().to_string(),
            here: Some(path.to_path_buf()),
        }
    }

    /// Its folder there, and its name: none for a storage's top.
    pub fn split(&self) -> Option<(&[String], &str)> {
        self.path
            .split_last()
            .map(|(name, folder)| (folder, name.as_str()))
    }

    /// What is there: none when nothing is.
    pub fn stat(&self) -> Result<Option<Item>, Exit> {
        let Some((folder, name)) = self.split() else {
            // A storage's top is a folder.
            return Ok(Some(Item {
                name: String::new(),
                dir: true,
                link: false,
                hidden: false,
                size: 0,
                modified: None,
                mode: None,
            }));
        };
        self.storage
            .stat(folder, name)
            .map_err(|e| Exit::Failed(format!("{}: {e}", self.shown)))
    }

    /// Whether it is a folder.
    pub fn is_folder(&self) -> Result<bool, Exit> {
        Ok(self.stat()?.is_some_and(|i| i.dir))
    }

    /// The archive it is, to read: a file here as it is, one on a storage
    /// read where it is, a part at a time.
    pub fn source(&self) -> Result<Source, Exit> {
        if let Some(here) = &self.here {
            return Ok(Source::from(here.as_path()));
        }
        let item = self
            .stat()?
            .ok_or_else(|| Exit::Failed(format!("{} is not there", self.shown)))?;
        if item.dir {
            return Err(Exit::Failed(format!("{} is a folder", self.shown)));
        }
        Ok(Source::Stored(Stored::new(
            self.storage.clone(),
            self.path.clone(),
            item.size,
        )))
    }

    /// Its name, as a person knows it.
    pub fn name(&self) -> String {
        self.path
            .last()
            .cloned()
            .unwrap_or_else(|| self.shown.clone())
    }
}

/// The storage added to the app as `name`, if one is.
fn kept_named(name: &str) -> Result<Option<Kept>, Exit> {
    if name.is_empty() {
        return Ok(None);
    }
    Ok(storages(&text(), &packwing_remote::kinds())
        .into_iter()
        .find(|k| k.name == name))
}
