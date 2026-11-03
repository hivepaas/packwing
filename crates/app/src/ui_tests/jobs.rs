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

/// A job that never ends, in the background: one more that runs.
fn hold(ui: &mut Ui, name: &str) {
    let job = crate::dialogs::Job::shown(crate::dialogs::Verb::Test, name.into(), None, 1, 4);
    ui.ws(|ws, window, cx| ws.hold_job_for_test(job, window, cx));
}

scenario!(
    /// Two jobs run at once: a third waits for its turn, and starts when one ends.
    a_third_job_waits_for_its_turn,
    |ui| {
        use crate::dialogs::State;
        let archive = ui.copy_of("sample.7z");
        let held = listed(&archive, None);
        hold(ui, "one.zip");
        hold(ui, "two.zip");
        ui.ws(|ws, _, _| ws.send_jobs_behind_for_test());
        extract(ui, &archive);
        let out = ui.work.join("sample");
        assert_eq!(
            ui.ws(|ws, _, cx| ws.job_states(cx)),
            [State::Running, State::Running, State::Queued]
        );
        assert!(!out.exists(), "it waits");
        ui.click("nav-jobs");
        ui.click("job-stop-0");
        ui.wait_until("the files out", |_| {
            held.iter()
                .filter(|n| !n.ends_with('/'))
                .all(|n| out.join(n).exists())
        });
        assert!(
            ui.cx.shown_system_notifications().is_empty(),
            "the window is the user's: the page says it"
        );
    }
);

scenario!(
    /// A job that ends in the background while the user is in another window is told
    /// through the system, and its notification brings the Jobs page.
    the_system_tells_of_a_job_that_ended,
    |ui| {
        let archive = ui.copy_of("sample.7z");
        ui.ws(|ws, _, _| ws.send_jobs_behind_for_test());
        ui.open(&archive);
        ui.click("extract");
        ui.wait_for("dest-new");
        ui.click("dest-new");
        let mut vcx = gpui::VisualTestContext::from_window(ui.window().into(), &ui.cx);
        vcx.deactivate_window();
        ui.click("extract");
        ui.wait_until("it is told", |ui| {
            !ui.cx.shown_system_notifications().is_empty()
        });
        let told = ui.cx.shown_system_notifications().remove(0);
        assert_eq!(told.title.as_ref(), "Extracted");
        assert!(told.body.contains("sample.7z"), "{}", told.body);
        ui.cx
            .simulate_system_notification_response(gpui::SystemNotificationResponse {
                tag: told.tag,
                action_id: None,
            });
        ui.cx.run_until_parked();
        assert!(ui.shows("job-0"), "the Jobs page is shown");
    }
);

scenario!(
    /// Closing the window while a job runs asks first: the job goes on, or it stops and
    /// the window closes.
    closing_while_a_job_runs_asks,
    |ui| {
        hold(ui, "one.zip");
        let mut vcx = gpui::VisualTestContext::from_window(ui.window().into(), &ui.cx);
        assert!(!vcx.simulate_close(), "it stays open, to ask");
        ui.cx.run_until_parked();
        assert!(ui.shows("quit-stop"));
        ui.click("quit-keep");
        assert!(!ui.dialog_open());
        assert_eq!(ui.ws(|ws, _, cx| ws.jobs_summary(cx)).running, 1);
        let mut vcx = gpui::VisualTestContext::from_window(ui.window().into(), &ui.cx);
        assert!(!vcx.simulate_close());
        ui.cx.run_until_parked();
        ui.click("quit-stop");
        assert!(
            ui.cx.update(|cx| cx.windows().is_empty()),
            "the window closed once the job stopped"
        );
    }
);
