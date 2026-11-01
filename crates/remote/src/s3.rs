//! An S3 bucket, or a prefix of one. S3 has no folders, and objects cannot be
//! renamed: they are copied, then deleted, and a folder would be all that is
//! under it, so it is not renamed at once.

use std::io;
use std::sync::Arc;

use opendal::{Operator, services};
use packwing_core::{Field, Storage, StorageKind, Values};
use packwing_i18n::tr;

use crate::dal::Dal;
use crate::secrets;

/// What a bucket is reached with.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct S3Settings {
    /// Empty for AWS.
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    /// Under this prefix only; empty for the whole bucket.
    pub prefix: String,
    /// Empty for a bucket open to anyone.
    pub access_key: String,
    pub secret_key: String,
}

/// A bucket as a storage. With `id`, its secret key is the one kept under it.
pub fn s3(settings: S3Settings, id: Option<String>) -> Dal {
    let (prefix, shown) = (
        settings.prefix.clone(),
        format!("s3://{}/", settings.bucket),
    );
    let make = move || {
        let s = &settings;
        let secret = match &id {
            Some(id) => secrets::get(id, "secret_key")?,
            None => s.secret_key.clone(),
        };
        let mut b = services::S3::default()
            .bucket(&s.bucket)
            .region(if s.region.is_empty() {
                "us-east-1"
            } else {
                &s.region
            })
            .disable_config_load()
            .disable_ec2_metadata();
        if !s.endpoint.is_empty() {
            b = b.endpoint(&s.endpoint);
        }
        b = if s.access_key.is_empty() {
            b.skip_signature()
        } else {
            b.access_key_id(&s.access_key).secret_access_key(&secret)
        };
        Operator::new(b).map_err(io::Error::other)
    };
    Dal::new(Box::new(make), &prefix, shown, None)
}

/// S3 buckets as a kind of storage.
pub struct S3Kind;

impl StorageKind for S3Kind {
    fn id(&self) -> &'static str {
        "s3"
    }

    fn tag(&self) -> &'static str {
        "S3"
    }

    fn name(&self) -> String {
        tr!("S3 bucket")
    }

    fn fields(&self) -> Vec<Field> {
        let field = |key, label: String, hint: String, secret| Field {
            key,
            label,
            hint,
            secret,
        };
        vec![
            field("bucket", tr!("Bucket"), "acme-backups".to_owned(), false),
            field("region", tr!("Region"), "eu-west-1".to_owned(), false),
            field(
                "endpoint",
                tr!("Endpoint"),
                tr!("Empty for AWS, or https://…"),
                false,
            ),
            field(
                "prefix",
                tr!("Start in"),
                tr!("A prefix, if not the whole bucket"),
                false,
            ),
            field("access_key", tr!("Access key ID"), String::new(), false),
            field("secret_key", tr!("Secret access key"), String::new(), true),
        ]
    }

    fn describe(&self, values: &Values) -> String {
        let value = |key: &str| values.get(key).map_or("", String::as_str);
        match value("region") {
            "" => format!("S3 · {}", value("bucket")),
            region => format!("S3 · {} · {region}", value("bucket")),
        }
    }

    fn open(&self, id: &str, values: &Values, typed: Option<&Values>) -> Arc<dyn Storage> {
        let value = |key: &str| values.get(key).cloned().unwrap_or_default();
        let settings = S3Settings {
            endpoint: value("endpoint"),
            region: value("region"),
            bucket: value("bucket"),
            prefix: value("prefix"),
            access_key: value("access_key"),
            secret_key: typed
                .and_then(|t| t.get("secret_key"))
                .cloned()
                .unwrap_or_default(),
        };
        Arc::new(s3(settings, typed.is_none().then(|| id.to_owned())))
    }

    fn keep_secrets(&self, id: &str, secrets: &Values) -> io::Result<()> {
        secrets::keep(id, secrets)
    }

    fn forget_secrets(&self, id: &str) {
        secrets::forget(id, &["secret_key"]);
    }
}
