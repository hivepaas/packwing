//! What a license key says. A key is what the portal issues for every product that takes
//! one (its `pkg/licensesig`): a payload and two signatures, with packwing's own signing
//! context, so that a key of another product can never pass as packwing's. It is checked
//! here with no server, and nothing is ever sent anywhere.
//!
//! packwing does everything without a key but keep more than [`FREE_REMOTE_STORAGES`]
//! storages across a network. No key ever turns a policy or a security feature on or off:
//! a bill unpaid must not be a security incident. The code is free software, so this
//! asks people to pay rather than stopping them: anyone can build it without the check.

pub mod verify;

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD};
use packwing_i18n::tr;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// The signing context of packwing's keys.
const CONTEXT: &[u8] = b"packwing-license-v1";

/// The most text taken as a key, as the portal and HivePaaS bound it.
const MOST_KEY_TEXT: usize = 32 << 10;

/// The most signatures looked at in a key, as the portal bounds it.
const MOST_SIGNATURES: usize = 16;

/// How many storages across a network a computer can add with no paid license.
pub const FREE_REMOTE_STORAGES: usize = 1;

/// Longest a key's grace may be.
const MOST_GRACE_DAYS: i64 = 365;

/// The public keys that a key's signatures must come from, by id: one of each algorithm.
struct Trusted {
    ed25519: (&'static str, &'static str),
    mldsa65: (&'static str, &'static str),
}

/// The keys a release trusts: those every product of the portal signs with, each under a
/// context of its own. Copies of its `pkg/licensesig/prodkeys/shared`, byte for byte.
const RELEASE: Trusted = Trusted {
    ed25519: (
        "lic-2026-ed",
        include_str!("../keys/prod/lic-2026-ed.pub.pem"),
    ),
    mldsa65: (
        "lic-2026-ml",
        include_str!("../keys/prod/lic-2026-ml.pub.pem"),
    ),
};

/// The development keys, whose private halves are public, so a release never trusts them.
const DEVELOPMENT: Trusted = Trusted {
    ed25519: ("dev-ed", include_str!("../keys/dev/dev-ed.pem")),
    mldsa65: ("dev-ml", include_str!("../keys/dev/dev-ml.pem")),
};

fn trusted() -> Trusted {
    if cfg!(debug_assertions) {
        DEVELOPMENT
    } else {
        RELEASE
    }
}

/// What a license is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plan {
    Free,
    Pro,
    Enterprise,
}

impl Plan {
    pub fn name(self) -> &'static str {
        match self {
            Plan::Free => "Free",
            Plan::Pro => "Pro",
            Plan::Enterprise => "Enterprise",
        }
    }

    /// Whether another storage across a network can be added to `kept` of them.
    pub fn may_add_remote_storage(self, kept: usize) -> bool {
        self != Plan::Free || kept < FREE_REMOTE_STORAGES
    }

    /// Whether files can be signed. Checking a signature takes no plan.
    pub fn may_sign(self) -> bool {
        self != Plan::Free
    }
}

/// A key that verified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct License {
    /// What it is for, past its grace or not.
    pub bought: Plan,
    /// Who it was issued to.
    pub holder: String,
    /// How many computers or people it is for, if it says.
    pub seats: Option<u64>,
    pub trial: bool,
    pub expires: OffsetDateTime,
    grace_days: i64,
}

/// Where a license stands on a day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    Current,
    /// Past its end, working for this many days more.
    InGrace(i64),
    Over,
}

impl License {
    pub fn standing(&self, now: OffsetDateTime) -> Standing {
        if now <= self.expires {
            return Standing::Current;
        }
        let left = (self.expires + time::Duration::days(self.grace_days) - now).whole_days();
        if self.expires + time::Duration::days(self.grace_days) >= now {
            Standing::InGrace(left)
        } else {
            Standing::Over
        }
    }

    /// What it gives on a day: nothing past its grace.
    pub fn plan(&self, now: OffsetDateTime) -> Plan {
        if self.standing(now) == Standing::Over {
            Plan::Free
        } else {
            self.bought
        }
    }
}

#[derive(Deserialize)]
struct Envelope {
    payload: String,
    sha256: String,
    signatures: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "keyId")]
    key_id: String,
    alg: String,
    sig: String,
}

#[derive(Deserialize)]
struct Payload {
    version: i64,
    #[serde(rename = "licenseId")]
    license_id: String,
    customer: Customer,
    plan: String,
    #[serde(default)]
    limits: std::collections::BTreeMap<String, u64>,
    #[serde(rename = "issuedAt")]
    issued_at: String,
    #[serde(rename = "expiresAt")]
    expires_at: String,
    #[serde(rename = "graceDays")]
    grace_days: i64,
    #[serde(default)]
    trial: bool,
}

#[derive(Default, Deserialize)]
struct Customer {
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String,
}

/// The bytes of a PEM public key's last `length` bytes: what comes after the header of
/// `header` bytes in its DER.
fn raw_key<const N: usize>(pem: &str, header: usize) -> Option<[u8; N]> {
    let text: String = pem.lines().filter(|l| !l.starts_with("-----")).collect();
    let der = STANDARD.decode(text).ok()?;
    if der.len() != header + N {
        return None;
    }
    der[header..].try_into().ok()
}

/// A key's text as it is issued, or as an email wrapped it, without its whitespace.
fn envelope_of(text: &str) -> Option<Envelope> {
    let text: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    if text.is_empty() || text.len() > MOST_KEY_TEXT {
        return None;
    }
    let raw = if text.starts_with('{') {
        text.into_bytes()
    } else {
        STANDARD_NO_PAD.decode(text.trim_end_matches('=')).ok()?
    };
    serde_json::from_slice(&raw).ok()
}

/// The bytes signed, if `trusted`'s two keys both signed them, and neither says otherwise.
fn opened(envelope: &Envelope, trusted: &Trusted) -> Option<Vec<u8>> {
    if envelope.signatures.len() > MOST_SIGNATURES {
        return None;
    }
    let data = STANDARD
        .decode(&envelope.payload)
        .ok()
        .filter(|d| !d.is_empty())?;
    let sum = Sha256::digest(&data);
    let hex: String = sum.iter().map(|b| format!("{b:02x}")).collect();
    if hex != envelope.sha256 {
        return None;
    }
    let ed: [u8; verify::ED25519_PUBLIC] = raw_key(trusted.ed25519.1, 12)?;
    let ml: [u8; verify::MLDSA65_PUBLIC] = raw_key(trusted.mldsa65.1, 22)?;
    let (mut by_ed, mut by_ml) = (false, false);
    for entry in &envelope.signatures {
        let signature = STANDARD.decode(&entry.sig).ok();
        if entry.key_id == trusted.ed25519.0 {
            if entry.alg != "ed25519" || !verify::ed25519ctx(&ed, &data, CONTEXT, &signature?) {
                return None;
            }
            by_ed = true;
        } else if entry.key_id == trusted.mldsa65.0 {
            if entry.alg != "ml-dsa-65" || !verify::mldsa65(&ml, &data, CONTEXT, &signature?) {
                return None;
            }
            by_ml = true;
        }
    }
    (by_ed && by_ml).then_some(data)
}

fn license_of(data: &[u8]) -> Option<License> {
    let payload: Payload = serde_json::from_slice(data).ok()?;
    let bought = match payload.plan.as_str() {
        "pro" => Plan::Pro,
        "enterprise" => Plan::Enterprise,
        _ => return None,
    };
    let issued = OffsetDateTime::parse(&payload.issued_at, &Rfc3339).ok()?;
    let expires = OffsetDateTime::parse(&payload.expires_at, &Rfc3339).ok()?;
    if payload.version != 1
        || payload.license_id.is_empty()
        || expires <= issued
        || !(0..=MOST_GRACE_DAYS).contains(&payload.grace_days)
    {
        return None;
    }
    let holder = if payload.customer.name.is_empty() {
        payload.customer.email
    } else {
        payload.customer.name
    };
    Some(License {
        bought,
        holder,
        seats: payload.limits.get("seats").copied(),
        trial: payload.trial,
        expires,
        grace_days: payload.grace_days,
    })
}

/// What `text` says, if it is a license key for packwing that this build trusts. It says
/// nothing of why not, as the text may hold a customer's email.
pub fn read(text: &str) -> Result<License, String> {
    let wrong = || tr!("This is not a valid packwing license key.");
    let trusted = trusted();
    let envelope = envelope_of(text).ok_or_else(wrong)?;
    let data = opened(&envelope, &trusted).ok_or_else(wrong)?;
    license_of(&data).ok_or_else(wrong)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRO: &str = include_str!("../testdata/pro.key");
    #[cfg(debug_assertions)]
    const ENTERPRISE: &str = include_str!("../testdata/enterprise.key");
    #[cfg(debug_assertions)]
    const EXPIRED: &str = include_str!("../testdata/expired.key");
    const OTHER_PRODUCT: &str = include_str!("../testdata/other-product.key");
    const ONE_SIGNATURE: &str = include_str!("../testdata/one-signature.key");
    const UNKNOWN_PLAN: &str = include_str!("../testdata/unknown-plan.key");

    #[cfg(debug_assertions)]
    fn at(text: &str) -> OffsetDateTime {
        OffsetDateTime::parse(text, &Rfc3339).expect("a time")
    }

    #[cfg(debug_assertions)]
    #[test]
    fn a_key_from_the_portals_code_is_read() {
        let license = read(PRO).expect("it verifies");
        assert_eq!(license.bought, Plan::Pro);
        assert_eq!(license.holder, "Example Co.");
        assert_eq!(license.seats, None);
        assert!(!license.trial);
        let license = read(ENTERPRISE).expect("it verifies");
        assert_eq!(license.bought, Plan::Enterprise);
        assert_eq!(license.seats, Some(250));
    }

    #[cfg(debug_assertions)]
    #[test]
    fn a_key_wrapped_by_an_email_is_the_same_key() {
        let wrapped: String = PRO
            .trim()
            .as_bytes()
            .chunks(76)
            .map(|line| format!("  {}\n", String::from_utf8_lossy(line)))
            .collect();
        assert_eq!(read(&wrapped), read(PRO));
    }

    #[test]
    fn what_is_not_a_key_for_packwing_is_refused() {
        for bad in [
            "",
            "not a key",
            "{}",
            OTHER_PRODUCT,
            ONE_SIGNATURE,
            UNKNOWN_PLAN,
        ] {
            assert!(read(bad).is_err(), "{bad:.30}");
        }
        assert!(read(&"A".repeat(MOST_KEY_TEXT + 1)).is_err());
    }

    /// An envelope of `PRO`'s with `change` done to it.
    #[cfg(debug_assertions)]
    fn changed(change: impl Fn(&mut serde_json::Value)) -> String {
        let raw = STANDARD_NO_PAD
            .decode(PRO.trim().trim_end_matches('='))
            .expect("base64");
        let mut envelope: serde_json::Value = serde_json::from_slice(&raw).expect("json");
        change(&mut envelope);
        envelope.to_string()
    }

    #[cfg(debug_assertions)]
    #[test]
    fn a_key_changed_is_refused() {
        // Its payload, with its hash made to match, so that only the signatures can tell.
        let forged = changed(|envelope| {
            let data = STANDARD
                .decode(envelope["payload"].as_str().expect("a payload"))
                .expect("base64");
            let text = String::from_utf8(data)
                .expect("text")
                .replace("\"pro\"", "\"enterprise\"");
            let hex: String = Sha256::digest(text.as_bytes())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            envelope["payload"] = STANDARD.encode(&text).into();
            envelope["sha256"] = hex.into();
        });
        assert!(read(&forged).is_err());
        // A signature claiming another algorithm than its key has.
        let claiming = changed(|envelope| envelope["signatures"][0]["alg"] = "rsa".into());
        assert!(read(&claiming).is_err());
        // Unchanged, the same test passes: it is the changes that are refused.
        assert!(read(&changed(|_| {})).is_ok());
    }

    #[cfg(debug_assertions)]
    #[test]
    fn a_license_past_its_end_works_through_its_grace_then_is_free() {
        let license = read(EXPIRED).expect("it verifies");
        assert_eq!(
            license.standing(at("2020-05-01T00:00:00Z")),
            Standing::Current
        );
        assert_eq!(license.plan(at("2020-05-01T00:00:00Z")), Plan::Pro);
        assert_eq!(
            license.standing(at("2020-06-11T00:00:00Z")),
            Standing::InGrace(20)
        );
        assert_eq!(license.plan(at("2020-06-11T00:00:00Z")), Plan::Pro);
        assert_eq!(license.standing(at("2020-07-02T00:00:00Z")), Standing::Over);
        assert_eq!(license.plan(at("2020-07-02T00:00:00Z")), Plan::Free);
    }

    #[test]
    fn a_release_trusts_its_own_keys_and_no_development_key() {
        assert!(raw_key::<{ verify::ED25519_PUBLIC }>(RELEASE.ed25519.1, 12).is_some());
        assert!(raw_key::<{ verify::MLDSA65_PUBLIC }>(RELEASE.mldsa65.1, 22).is_some());
        assert!(!RELEASE.ed25519.0.starts_with("dev-") && !RELEASE.mldsa65.0.starts_with("dev-"));
        let envelope = envelope_of(PRO).expect("an envelope");
        assert!(opened(&envelope, &DEVELOPMENT).is_some());
        assert!(opened(&envelope, &RELEASE).is_none());
    }

    /// `cargo test --release`: what a release build does with a development key.
    #[cfg(not(debug_assertions))]
    #[test]
    fn a_release_build_refuses_a_development_key() {
        assert!(read(PRO).is_err());
    }

    #[test]
    fn free_keeps_one_storage_across_a_network_and_paid_plans_any() {
        assert!(Plan::Free.may_add_remote_storage(0));
        assert!(!Plan::Free.may_add_remote_storage(1));
        assert!(!Plan::Free.may_add_remote_storage(5));
        assert!(Plan::Pro.may_add_remote_storage(100));
        assert!(Plan::Enterprise.may_add_remote_storage(100));
    }

    #[test]
    fn signing_takes_a_paid_plan() {
        assert!(!Plan::Free.may_sign());
        assert!(Plan::Pro.may_sign());
        assert!(Plan::Enterprise.may_sign());
    }
}
