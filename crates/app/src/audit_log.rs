//! The audit log, kept as the organization says, else as the user chose where
//! their plan has it (docs/audit.md). Each change of whether, or how long, is a
//! record of its own, written while the log is still kept: started, stopped or
//! changed, and by what.

use gpui::App;
use packwing_core::audit::{self, Chosen};
use serde_json::json;

/// What decided a change: the user in Settings, the license, or packwing
/// starting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum By {
    Startup,
    User,
    License,
}

impl By {
    fn name(self) -> &'static str {
        match self {
            By::Startup => "startup",
            By::User => "user",
            By::License => "license",
        }
    }
}

/// How the log is to be kept now, if it is.
pub fn settled(cx: &App) -> Option<audit::Config> {
    let p = crate::prefs::prefs(cx);
    audit::settled(
        crate::managed::policy(cx),
        crate::licensing::plan(cx).has_audit_log(),
        Chosen {
            on: p.audit_log,
            retention_days: p.audit_retention_days,
        },
        "packwing",
    )
}

/// Keeps the log as it is to be kept now, saying what changed in it.
pub fn apply(cx: &App, by: By) {
    let before = audit::config();
    let after = settled(cx);
    let managed = crate::managed::policy(cx).locks_audit_log();
    let said = |action: &str, config: &audit::Config| {
        audit::record(
            action,
            serde_json::Map::from_iter([
                (
                    "by".to_owned(),
                    json!(if managed { "policy" } else { by.name() }),
                ),
                ("retention_days".to_owned(), json!(config.retention_days)),
                ("max_bytes".to_owned(), json!(config.max_bytes)),
                ("folder".to_owned(), json!(config.folder.to_string_lossy())),
            ]),
        );
    };
    match (&before, &after) {
        (Some(was), None) => {
            said("audit.stopped", was);
            audit::set(None);
        }
        (None, Some(now)) => {
            audit::set(Some(now.clone()));
            said("audit.started", now);
        }
        (Some(was), Some(now)) if was != now => {
            audit::set(Some(now.clone()));
            said("audit.changed", now);
        }
        (Some(now), Some(_)) if by == By::Startup => said("audit.started", now),
        _ => {}
    }
}
