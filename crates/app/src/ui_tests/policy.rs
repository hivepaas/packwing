//! What an organization fixes, as the system would say it.

use packwing_license::Plan;
use packwing_policy::{
    AUDIT_LOG, AUDIT_LOG_PATH, AUDIT_LOG_REQUIRED, Found, LICENSE_KEY, MIN_PASSWORD_LENGTH,
    PROPAGATE_MOTW, REQUIRE_ENCRYPTION, Value,
};

use super::{Ui, listed, scenario};

fn found(values: Vec<(&str, Value)>) -> Found {
    Found {
        origin: "the test".into(),
        values: values.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
        failure: None,
    }
}

const LONG: &str = "fifteen or more characters";

/// New archive open on the one file `a.txt`.
fn new_archive(ui: &mut Ui) -> std::path::PathBuf {
    let file = ui.file("a.txt", "held");
    ui.ws(|ws, window, cx| ws.open_create(vec![file], window, cx));
    ui.wait_for("create");
    ui.work.join("a.zip")
}

scenario!(
    /// Required, encryption cannot be turned off, a short password is refused, and a long
    /// one makes the archive.
    encryption_required,
    policy: found(vec![
        (REQUIRE_ENCRYPTION, Value::Bool(true)),
        (MIN_PASSWORD_LENGTH, Value::Int(15)),
    ]),
    |ui| {
        let zip = new_archive(ui);
        assert!(!ui.shows("protect"), "the switch can still be turned off");
        ui.click("create");
        assert!(ui.dialog_open() && !zip.exists(), "made with no password");
        ui.type_in("Password", "short");
        ui.click("create");
        ui.cx.run_until_parked();
        assert!(ui.dialog_open() && !zip.exists(), "made with a short password");
        ui.replace_in("Password", LONG);
        ui.click("create");
        ui.wait_for_no_dialog();
        assert_eq!(listed(&zip, Some(LONG)), ["a.txt"]);
    }
);

scenario!(
    /// Required, Compress to ZIP asks for a password instead of making one in the clear.
    the_right_click_menu_obeys,
    policy: found(vec![(REQUIRE_ENCRYPTION, Value::Bool(true))]),
    |ui| {
        let file = ui.file("a.txt", "held");
        let request = crate::right_click::Request {
            action: crate::right_click::Action::CompressZip,
            paths: vec![file],
        };
        ui.ws(|ws, window, cx| ws.start(request, window, cx));
        ui.wait_for("create");
        assert!(!ui.work.join("a.zip").exists());
    }
);

scenario!(
    /// Fixed by policy, the user has no switch for the mark, and Settings says so.
    the_mark_is_locked,
    policy: found(vec![(PROPAGATE_MOTW, Value::Bool(true))]),
    |ui| {
        ui.settings(3);
        assert!(!ui.shows("security-0"), "the mark's switch is still there");
        assert!(ui.shows("security-1"), "the other switch is gone");
    }
);

scenario!(
    /// The organization's key is the plan, and the user is not asked for one.
    a_license_by_policy,
    policy: found(vec![(
        LICENSE_KEY,
        Value::Text(include_str!("../../../license/testdata/enterprise.key").into()),
    )]),
    |ui| {
        assert_eq!(ui.plan(), Plan::Enterprise);
        ui.settings(5);
        assert!(!ui.shows("license-enter"));
    }
);

scenario!(
    /// A policy that cannot be read refuses, rather than letting everything through.
    what_cannot_be_read_refuses,
    policy: Found {
        origin: "the test".into(),
        values: Vec::new(),
        failure: Some("broken".into()),
    },
    |ui| {
        let zip = new_archive(ui);
        ui.type_in("Password", LONG);
        ui.click("create");
        ui.cx.run_until_parked();
        assert!(ui.dialog_open() && !zip.exists());
    }
);

scenario!(
    /// Turned on by the organization, the audit log is kept on Free too, its switch
    /// locked, its records in the folder the policy names.
    the_audit_log_is_kept_as_the_organization_says,
    policy: found(vec![
        (packwing_policy::AUDIT_LOG, Value::Bool(true)),
        (packwing_policy::AUDIT_LOG_RETENTION_DAYS, Value::Int(30)),
    ]),
    |ui| {
        use packwing_core::audit;
        assert_eq!(ui.plan(), Plan::Free);
        ui.settings(3);
        assert!(!ui.shows("audit-log"), "the user's switch");
        assert!(!ui.shows("audit-retention"), "the user's choice of how long");
        let config = audit::config().expect("kept on Free");
        assert_eq!(config.retention_days, 30);
        let records = audit::read(&config.folder, None).expect("the records");
        assert_eq!(records[0]["action"], "audit.started");
        assert_eq!(records[0]["by"], "policy");
    }
);

/// A file where the audit log's folder would be, so that no record can be
/// written: one for this process, made before the policy names it.
fn no_audit_folder() -> std::path::PathBuf {
    let path = no_audit_path();
    std::fs::write(&path, "a file where the audit log's folder would be").expect("the file");
    path
}

scenario!(
    /// Required by the organization, a job whose start the audit log cannot
    /// write does not start: New archive says why, and makes nothing.
    a_job_the_audit_log_cannot_record_does_not_start,
    policy: found(vec![
        (AUDIT_LOG, Value::Bool(true)),
        (AUDIT_LOG_REQUIRED, Value::Bool(true)),
        (AUDIT_LOG_PATH, Value::Text(no_audit_folder().display().to_string())),
    ]),
    |ui| {
        let zip = new_archive(ui);
        ui.click("create");
        ui.wait_until("the job to be refused", |ui| {
            ui.ws(|ws, _, cx| ws.job_failed(cx)).is_some()
        });
        let why = ui.ws(|ws, _, cx| ws.job_failed(cx)).expect("why");
        assert!(why.contains("requires each job to be recorded"), "{why}");
        assert!(!zip.exists(), "a job refused makes nothing");
        let _ = std::fs::remove_file(no_audit_path());
    }
);

/// Where [`no_audit_folder`] put its file.
fn no_audit_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("packwing-no-audit-{}", std::process::id()))
}
