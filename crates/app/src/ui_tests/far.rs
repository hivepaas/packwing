//! A kind of storage that stands for those across a network: its `folder` is a folder
//! here that it does not say the path of, and it keeps its secrets in memory, as a test
//! must not touch the user's keychain.

use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::Arc;

use packwing_core::{Field, Item, Lister, LocalFolder, Storage, StorageKind, Values};

struct Far(LocalFolder);

impl Storage for Far {
    fn list(&self, f: &[String]) -> io::Result<Box<dyn Lister>> {
        self.0.list(f)
    }
    fn read(&self, p: &[String], most: u64) -> io::Result<Vec<u8>> {
        self.0.read(p, most)
    }
    fn read_at(&self, p: &[String], offset: u64, len: usize) -> io::Result<Vec<u8>> {
        self.0.read_at(p, offset, len)
    }
    fn stat(&self, f: &[String], n: &str) -> io::Result<Option<Item>> {
        self.0.stat(f, n)
    }
    fn open(&self, p: &[String]) -> io::Result<Box<dyn Read + Send>> {
        self.0.open(p)
    }
    fn put(
        &self,
        p: &[String],
        from: &mut dyn Read,
        like: &Item,
        replace: bool,
    ) -> io::Result<u64> {
        self.0.put(p, from, like, replace)
    }
    fn remove(&self, p: &[String]) -> io::Result<()> {
        self.0.remove(p)
    }
    fn make_folder(&self, f: &[String], n: &str) -> io::Result<()> {
        self.0.make_folder(f, n)
    }
    fn rename(&self, f: &[String], a: &str, b: &str) -> io::Result<()> {
        self.0.rename(f, a, b)
    }
    fn delete(&self, f: &[String], n: &[String]) -> io::Result<()> {
        self.0.delete(f, n)
    }
    fn has_trash(&self) -> bool {
        false
    }
    fn local_path(&self, _: &[String]) -> Option<PathBuf> {
        None
    }
    fn show_path(&self, p: &[String]) -> String {
        format!("far://{}", p.join("/"))
    }
}

struct FarKind;

impl StorageKind for FarKind {
    fn id(&self) -> &'static str {
        "far"
    }
    fn tag(&self) -> &'static str {
        "FAR"
    }
    fn name(&self) -> String {
        "Far bucket".into()
    }
    fn fields(&self) -> Vec<Field> {
        vec![Field {
            key: "folder",
            label: "Folder".into(),
            hint: "Folder".into(),
            secret: false,
        }]
    }
    fn describe(&self, values: &Values) -> String {
        format!("Far · {}", values.get("folder").map_or("", String::as_str))
    }
    fn open(&self, _: &str, values: &Values, _: Option<&Values>) -> Arc<dyn Storage> {
        let folder = values.get("folder").cloned().unwrap_or_default();
        Arc::new(Far(LocalFolder::new(folder)))
    }
    fn kept_secrets(&self, _: &str) -> io::Result<Values> {
        Ok(Values::new())
    }
    fn keep_secrets(&self, _: &str, _: &Values) -> io::Result<()> {
        Ok(())
    }
    fn forget_secrets(&self, _: &str) {}
}

pub fn kinds() -> Vec<Arc<dyn StorageKind>> {
    vec![Arc::new(FarKind)]
}
