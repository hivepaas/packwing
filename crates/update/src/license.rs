//! Asking the portal for a newer key of the organization's license, where its
//! `LicenseRefresh` policy says to: the only thing packwing ever sends about a
//! license, as HivePaaS installations ask (the portal's `docs/licensing.md`, The
//! refresh contract). The key held is the request's only content; what comes
//! back is only taken once it verifies as a newer key of the same license.

use std::time::Duration;

use serde::Deserialize;

/// Where the portal is.
pub const PORTAL: &str = "https://portal.hivepaas.com";

/// The most of an answer read: a key is at most 32 KiB.
const MOST_ANSWER: usize = 64 << 10;
const TIMEOUT: Duration = Duration::from_secs(15);

/// What the portal answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// A newer key, as the portal sent it: the caller checks it.
    Newer(String),
    /// The key held is the one to have.
    Current,
    /// The portal knows no such key, or no longer serves it: the key held stays.
    Unknown,
}

#[derive(Deserialize)]
struct Envelope {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    result: String,
    #[serde(default)]
    key: Option<String>,
}

/// What `body`, the portal's answer with `status`, says.
fn answer_of(status: u16, body: &[u8]) -> Result<Answer, String> {
    match status {
        200 => {}
        404 | 410 => return Ok(Answer::Unknown),
        429 => return Err("the portal asks to wait".into()),
        other => return Err(format!("the portal answered {other}")),
    }
    let envelope: Envelope = serde_json::from_slice(body)
        .map_err(|_| "the portal's answer cannot be read".to_owned())?;
    match (envelope.data.result.as_str(), envelope.data.key) {
        ("newer", Some(key)) if !key.trim().is_empty() => Ok(Answer::Newer(key)),
        ("current", _) => Ok(Answer::Current),
        _ => Ok(Answer::Unknown),
    }
}

/// Asks `portal` for a newer key than `key`, and waits for the answer.
pub fn refresh_blocking(portal: &str, key: &str) -> Result<Answer, String> {
    let url = format!("{}/api/licenses/refresh", portal.trim_end_matches('/'));
    let body = serde_json::json!({ "key": key.trim() }).to_string();
    let work = async move {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .user_agent(concat!("packwing/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| e.to_string())?;
        let res = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = res.status().as_u16();
        if res
            .content_length()
            .is_some_and(|n| n as usize > MOST_ANSWER)
        {
            return Err("the portal's answer is too large".to_owned());
        }
        let bytes = res.bytes().await.map_err(|e| e.to_string())?;
        if bytes.len() > MOST_ANSWER {
            return Err("the portal's answer is too large".to_owned());
        }
        answer_of(status, &bytes)
    };
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?
        .block_on(work)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_portals_answers_are_read() {
        let newer = br#"{"data": {"result": "newer", "key": "abc"}}"#;
        assert_eq!(answer_of(200, newer), Ok(Answer::Newer("abc".into())));
        let current = br#"{"data": {"result": "current"}}"#;
        assert_eq!(answer_of(200, current), Ok(Answer::Current));
        let moved = br#"{"data": {"result": "moved"}}"#;
        assert_eq!(answer_of(200, moved), Ok(Answer::Unknown));
        assert_eq!(answer_of(404, b""), Ok(Answer::Unknown));
        assert!(answer_of(429, b"").is_err() && answer_of(500, b"").is_err());
        assert!(answer_of(200, b"not json").is_err());
        let empty = br#"{"data": {"result": "newer", "key": " "}}"#;
        assert_eq!(answer_of(200, empty), Ok(Answer::Unknown));
    }
}
