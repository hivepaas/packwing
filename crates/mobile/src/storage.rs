//! Storages across the network, as the desktop has them (packwing-remote): their
//! kinds and the settings each asks for, their secrets in the system's keychain,
//! their folders listed, and an archive on one opened where it is, reading only
//! what it needs of it.
//!
//! The app keeps the storages' settings (their ids, kinds and values); the
//! secrets go to the keychain under each storage's id. SFTP asks what it needs
//! as it connects through the app ([`crate::ask_ssh_with`]).

use std::collections::{BTreeMap, HashMap};
use std::io;
use std::sync::Arc;

use std::path::PathBuf;

use packwing_core::{LocalFolder, Source, Storage, StorageKind, Stored, Transfer, Values};

use crate::{Archive, Job, MobileError, NewArchiveHow, OnExisting, Secrets};

/// Something a storage of a kind asks for when it is added.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct StorageField {
    pub key: String,
    pub label: String,
    /// What the empty field says.
    pub hint: String,
    /// Kept in the system's keychain, not with the settings, and not shown.
    pub secret: bool,
}

/// A kind of storage the phone can add.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct StorageKindRow {
    /// What the app keeps it as.
    pub id: String,
    /// The letters on its tile.
    pub tag: String,
    pub name: String,
    pub fields: Vec<StorageField>,
    /// The setting naming a key kept to sign in with (SFTP's), chosen from
    /// those kept; empty or missing for any of them.
    pub key_value: Option<String>,
}

/// A file or folder in a folder of a storage.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct StoredItem {
    pub name: String,
    pub folder: bool,
    pub link: bool,
    pub hidden: bool,
    /// In bytes; 0 for a folder.
    pub size: u64,
    /// In seconds since 1970, if the storage knows.
    pub modified: Option<i64>,
}

fn kinds() -> Vec<Arc<dyn StorageKind>> {
    packwing_remote::kinds()
}

fn kind(id: &str) -> Result<Arc<dyn StorageKind>, MobileError> {
    kinds()
        .into_iter()
        .find(|k| k.id() == id)
        .ok_or_else(|| MobileError::Failed {
            reason: format!("no storage of kind {id}"),
        })
}

fn values(map: HashMap<String, String>) -> Values {
    map.into_iter().collect::<BTreeMap<_, _>>()
}

fn failed(e: io::Error) -> MobileError {
    MobileError::Failed {
        reason: e.to_string(),
    }
}

/// The kinds of storage a phone can add, in the order they are shown.
#[uniffi::export]
pub fn storage_kinds() -> Vec<StorageKindRow> {
    kinds()
        .iter()
        .map(|k| StorageKindRow {
            id: k.id().to_owned(),
            tag: k.tag().to_owned(),
            name: k.name(),
            fields: k
                .fields()
                .into_iter()
                .map(|f| StorageField {
                    key: f.key.to_owned(),
                    label: f.label,
                    hint: f.hint,
                    secret: f.secret,
                })
                .collect(),
            key_value: k.key_value().map(str::to_owned),
        })
        .collect()
}

/// Where a storage of `kind` with these settings is, as its row says it.
#[uniffi::export]
pub fn describe_storage(kind: String, values: HashMap<String, String>) -> String {
    self::kind(&kind)
        .map(|k| k.describe(&self::values(values)))
        .unwrap_or_default()
}

/// Keeps the secrets of the storage `id` in the system's keychain.
#[uniffi::export]
pub fn keep_storage_secrets(
    kind: String,
    id: String,
    secrets: HashMap<String, String>,
) -> Result<(), MobileError> {
    self::kind(&kind)?
        .keep_secrets(&id, &values(secrets))
        .map_err(failed)
}

/// The secrets kept for the storage `id`, to change them.
#[uniffi::export]
pub fn kept_storage_secrets(
    kind: String,
    id: String,
) -> Result<HashMap<String, String>, MobileError> {
    Ok(self::kind(&kind)?
        .kept_secrets(&id)
        .map_err(failed)?
        .into_iter()
        .collect())
}

/// Forgets the secrets of the storage `id`, once it is removed.
#[uniffi::export]
pub fn forget_storage_secrets(kind: String, id: String) {
    if let Ok(kind) = self::kind(&kind) {
        kind.forget_secrets(&id);
    }
}

/// How a copy or a move went.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, uniffi::Record)]
pub struct Transferred {
    pub files: u64,
    pub bytes: u64,
    /// Left where they were, as their names were taken.
    pub skipped: u64,
    /// Links left where they were: one may lead anywhere.
    pub links_left: u64,
}

/// A storage, to browse. It touches neither the network nor the keychain until
/// first used.
#[derive(uniffi::Object)]
pub struct RemoteStorage {
    storage: Arc<dyn Storage>,
}

#[uniffi::export]
impl RemoteStorage {
    /// The storage `id` of `kind`, with its settings `values`, and its secrets
    /// those kept; or, with `typed`, those (to try it before it is kept).
    #[uniffi::constructor]
    pub fn open(
        kind: String,
        id: String,
        values: HashMap<String, String>,
        typed: Option<HashMap<String, String>>,
    ) -> Result<Arc<RemoteStorage>, MobileError> {
        let typed = typed.map(self::values);
        let storage = self::kind(&kind)?.open(&id, &self::values(values), typed.as_ref());
        Ok(Arc::new(RemoteStorage { storage }))
    }

    /// A folder of this phone, as a storage: to copy to and from one across
    /// the network (a folder of Files, packwing's own).
    #[uniffi::constructor]
    pub fn local(path: String) -> Arc<RemoteStorage> {
        Arc::new(RemoteStorage {
            storage: Arc::new(LocalFolder::new(PathBuf::from(path))),
        })
    }

    /// Makes the folder `name` in `folder`; a name taken is an error.
    pub fn make_folder(&self, folder: Vec<String>, name: String) -> Result<(), MobileError> {
        self.storage.make_folder(&folder, &name).map_err(failed)
    }

    /// Renames `from` in `folder` to `to`; a name taken is an error.
    pub fn rename(&self, folder: Vec<String>, from: String, to: String) -> Result<(), MobileError> {
        self.storage.rename(&folder, &from, &to).map_err(failed)
    }

    /// Deletes `names` of `folder`, for good: a storage across the network has
    /// no Trash.
    pub fn delete(&self, folder: Vec<String>, names: Vec<String>) -> Result<(), MobileError> {
        self.storage.delete(&folder, &names).map_err(failed)
    }

    /// Makes an archive of `paths` (files and folders of this phone) in
    /// `folder`, named `name` or the first name after it that is free: written
    /// there, or made here first and sent. Says the name it has.
    pub fn create_archive(
        &self,
        paths: Vec<String>,
        folder: Vec<String>,
        name: String,
        how: NewArchiveHow,
        job: Arc<Job>,
    ) -> Result<String, MobileError> {
        let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
        let made = packwing_core::create_on(
            &paths,
            self.storage.as_ref(),
            &folder,
            &name,
            &how.new_archive(),
            &job.control(),
            &|| {},
        )?;
        Ok(made
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or(name))
    }

    /// What is in `folder` (its path; empty for the top), folders first, by
    /// name, as the storage reads them.
    pub fn list(&self, folder: Vec<String>, job: Arc<Job>) -> Result<Vec<StoredItem>, MobileError> {
        let mut lister = self.storage.list(&folder).map_err(failed)?;
        let mut items = Vec::new();
        while !lister.done() {
            if job.is_cancelled() {
                return Err(packwing_core::Error::Cancelled.into());
            }
            let page = lister.next(500).map_err(failed)?;
            if page.is_empty() {
                break;
            }
            items.extend(page.into_iter().map(|i| StoredItem {
                name: i.name,
                folder: i.dir,
                link: i.link,
                hidden: i.hidden,
                size: i.size,
                modified: i.modified,
            }));
        }
        Ok(items)
    }

    /// `path` as the user would write it.
    pub fn show_path(&self, path: Vec<String>) -> String {
        self.storage.show_path(&path)
    }

    /// Up to `most` bytes of the file at `path`: to preview it.
    pub fn read(&self, path: Vec<String>, most: u64) -> Result<Vec<u8>, MobileError> {
        self.storage.read(&path, most).map_err(failed)
    }

    /// Opens the archive at `path`, `size` bytes long, where it is: only what
    /// listing it needs is read, and what is read is kept for what follows.
    pub fn open_archive(
        &self,
        path: Vec<String>,
        size: u64,
        secrets: Secrets,
        job: Arc<Job>,
    ) -> Result<Arc<Archive>, MobileError> {
        let source = Source::Stored(Stored::new(self.storage.clone(), path, size));
        Archive::of(source, secrets, job)
    }
}

/// Copies, or moves, `names` of `from_folder` of `from` into `to_folder` of
/// `to`, each with what is under it: between storages, or with a folder of
/// this phone ([`RemoteStorage::local`]). A name taken goes as `on_existing`
/// says.
#[uniffi::export]
#[allow(clippy::too_many_arguments)]
pub fn transfer_items(
    from: Arc<RemoteStorage>,
    from_folder: Vec<String>,
    names: Vec<String>,
    to: Arc<RemoteStorage>,
    to_folder: Vec<String>,
    moving: bool,
    on_existing: OnExisting,
    job: Arc<Job>,
) -> Result<Transferred, MobileError> {
    let done = packwing_core::transfer(
        &Transfer {
            from: from.storage.clone(),
            from_folder,
            names,
            to: to.storage.clone(),
            to_folder,
            moving,
            on_existing: on_existing.into(),
            keep_mark: false,
        },
        &job.control(),
    )?;
    Ok(Transferred {
        files: done.files,
        bytes: done.bytes,
        skipped: done.skipped,
        links_left: done.links_left,
    })
}
