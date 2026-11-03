//! Jobs in the background: they go on without their dialog, and the Jobs page says
//! how they ended.

use super::{Ui, listed, scenario};

/// Opens `archive` and extracts it to a new folder beside it, as Extract does.
fn extract(ui: &mut Ui, archive: &std::path::Path) {
    ui.open(archive);
    ui.click("extract");
    ui.wait_for("dest-new");
    ui.click("dest-new");
    ui.click("extract");
}

scenario!(
    /// An extraction sent to the background goes on without its dialog; the menu
    /// has Jobs while it is there, and the page shows where it went, until cleared.
    an_extraction_goes_on_in_the_background,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        let held = listed(&archive, None);
        assert!(!ui.shows("nav-jobs"));
        ui.ws(|ws, _, _| ws.send_jobs_behind_for_test());
        extract(ui, &archive);
        assert!(!ui.dialog_open(), "its dialog went with it");
        assert!(ui.shows("nav-jobs"));
        let out = ui.work.join("sample");
        ui.wait_until("the files out", |ui| {
            ui.ws(|ws, _, cx| ws.jobs_summary(cx)).running == 0
        });
        for name in held.iter().filter(|n| !n.ends_with('/')) {
            assert!(out.join(name).exists(), "{name} did not come out in {out:?}");
        }
        assert!(ui.ws(|ws, _, cx| ws.jobs_summary(cx)).unseen);
        ui.click("nav-jobs");
        assert!(!ui.ws(|ws, _, cx| ws.jobs_summary(cx)).unseen);
        ui.click("job-file-manager-0");
        assert_eq!(crate::file_manager::asked_for_test(), [("show", out)]);
        ui.click("job-clear-0");
        assert!(!ui.shows("job-0"));
        ui.click("nav-archive");
        assert!(!ui.shows("nav-jobs"), "no jobs, no Jobs");
    }
);

scenario!(
    /// A job in the background that needs a password waits for the user, and runs
    /// with it once it is given.
    a_job_in_the_background_waits_for_its_password,
    |ui| {
        let archive = ui.copy_of("secret.7z");
        let held = listed(&archive, Some("secret"));
        ui.ws(|ws, _, _| ws.send_jobs_behind_for_test());
        extract(ui, &archive);
        ui.wait_until("it waits", |ui| {
            ui.ws(|ws, _, cx| ws.jobs_summary(cx)).waiting
        });
        assert!(!ui.dialog_open(), "nothing is asked over what the user does");
        ui.click("nav-jobs");
        ui.click("job-ask-0");
        assert!(!ui.shows("job-0"));
        ui.type_in("The archive’s password", "secret");
        ui.click("open");
        let out = ui.work.join("secret");
        ui.wait_until("the files out", |_| {
            held.iter()
                .filter(|n| !n.ends_with('/'))
                .all(|n| out.join(n).exists())
        });
    }
);

scenario!(
    /// A new archive of the name one in the background is making there does not start:
    /// New archive says why, and stays for another name.
    two_archives_of_one_name_do_not_start,
    |ui| {
        let file = ui.file("a.txt", "held");
        let wanted = ui.work.join("a.zip");
        let mut job = crate::dialogs::Job::shown(
            crate::dialogs::Verb::Create,
            "a.zip".into(),
            Some(ui.work.display().to_string().into()),
            1,
            4,
        );
        job.create = Some(crate::dialogs::Creating {
            paths: vec![file.clone()],
            to: crate::dialogs::Saving::Here(wanted.clone()),
            how: packwing_core::NewArchive::default(),
        });
        ui.ws(|ws, window, cx| ws.hold_job_for_test(job, window, cx));
        ui.ws(|ws, window, cx| ws.open_create(vec![file], window, cx));
        ui.wait_for("create");
        ui.click("create");
        assert!(ui.shows("busy"), "it says why");
        assert!(ui.dialog_open(), "New archive stays");
        assert!(!wanted.exists());
        assert!(!ui.work.join("a (2).zip").exists());
    }
);

scenario!(
    /// An archive is not extracted into a folder another job extracts into now.
    two_extractions_into_one_folder_do_not_start,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        let out = ui.work.join("sample");
        let mut job = crate::dialogs::Job::shown(
            crate::dialogs::Verb::Extract,
            "sample.7z".into(),
            Some(out.display().to_string().into()),
            1,
            4,
        );
        job.work = vec![(packwing_core::Source::from(archive.as_path()), Some(out.clone()))];
        ui.ws(|ws, window, cx| ws.hold_job_for_test(job, window, cx));
        extract(ui, &archive);
        assert!(ui.shows("busy"), "it says why");
        assert!(ui.dialog_open(), "Extract stays");
        assert!(!out.exists());
    }
);
