//! A WebDAV folder, such as Nextcloud's. A PUT has to send a file whole, and is
//! held in memory while it goes, so an upload is limited to what that can bear.

use std::io;
use std::sync::Arc;

use opendal::{Operator, services};
use packwing_core::{Field, Storage, StorageKind, Values};
use packwing_i18n::tr;

use crate::dal::Dal;
use crate::secrets;

/// The most sent at once.
const MOST_HELD: u64 = 1_000_000_000;

/// What a WebDAV folder is reached with.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WebDavSettings {
    pub address: String,
    pub user: String,
    pub password: String,
}

/// A WebDAV folder as a storage. With `id`, its password is the one kept under it.
pub fn webdav(settings: WebDavSettings, id: Option<String>) -> Dal {
    let shown = format!("{}/", settings.address.trim_end_matches('/'));
    let make = move || {
        let s = &settings;
        let password = match &id {
            Some(id) => secrets::get(id, "password")?,
            None => s.password.clone(),
        };
        let mut b = services::Webdav::default().endpoint(&s.address);
        if !s.user.is_empty() {
            b = b.username(&s.user).password(&password);
        }
        Operator::new(b).map_err(io::Error::other)
    };
    Dal::new(Box::new(make), "", shown, Some(MOST_HELD))
}

/// WebDAV folders as a kind of storage.
pub struct WebDavKind;

impl StorageKind for WebDavKind {
    fn id(&self) -> &'static str {
        "webdav"
    }

    fn tag(&self) -> &'static str {
        "DAV"
    }

    fn name(&self) -> String {
        tr!("WebDAV folder")
    }

    fn fields(&self) -> Vec<Field> {
        vec![
            Field {
                key: "address",
                label: tr!("Address"),
                hint: "https://cloud.example.com/remote.php/dav/files/me".to_owned(),
                secret: false,
            },
            Field {
                key: "user",
                label: tr!("User"),
                hint: String::new(),
                secret: false,
            },
            Field {
                key: "password",
                label: tr!("Password or app token"),
                hint: String::new(),
                secret: true,
            },
        ]
    }

    fn describe(&self, values: &Values) -> String {
        format!(
            "WebDAV · {}",
            values.get("address").map_or("", String::as_str)
        )
    }

    fn open(&self, id: &str, values: &Values, typed: Option<&Values>) -> Arc<dyn Storage> {
        let value = |key: &str| values.get(key).cloned().unwrap_or_default();
        let settings = WebDavSettings {
            address: value("address"),
            user: value("user"),
            password: typed
                .and_then(|t| t.get("password"))
                .cloned()
                .unwrap_or_default(),
        };
        Arc::new(webdav(settings, typed.is_none().then(|| id.to_owned())))
    }

    fn keep_secrets(&self, id: &str, secrets: &Values) -> io::Result<()> {
        secrets::keep(id, secrets)
    }

    fn forget_secrets(&self, id: &str) {
        secrets::forget(id, &["password"]);
    }
}
