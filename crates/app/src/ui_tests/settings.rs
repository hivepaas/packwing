//! Settings: what is chosen stays, and the license is taken or refused.

use super::{Ui, scenario};
use packwing_license::Plan;

scenario!(
    /// A switch turned off stays off after a restart.
    a_setting_survives_a_restart,
    |ui| {
        ui.settings(3);
        assert!(ui.ws(|_, _, cx| crate::prefs::prefs(cx).keep_mark));
        ui.click("security-0");
        assert!(!ui.ws(|_, _, cx| crate::prefs::prefs(cx).keep_mark));
        ui.start();
        assert!(!ui.ws(|_, _, cx| crate::prefs::prefs(cx).keep_mark));
    }
);

scenario!(
    /// The checksums shown can be chosen, and the last one cannot be left out.
    checksums_shown_are_chosen,
    |ui| {
        use packwing_core::Algorithm;
        let kept = |ui: &mut Ui| ui.ws(|_, _, cx| crate::prefs::prefs(cx).checksums);
        ui.settings(3);
        assert_eq!(kept(ui), [Algorithm::Sha256, Algorithm::Sha1]);
        ui.click("checksum-3");
        ui.click("checksum-0");
        ui.click("checksum-2");
        assert_eq!(kept(ui), [Algorithm::Md5]);
        ui.click("checksum-3");
        assert_eq!(kept(ui), [Algorithm::Md5]);
        ui.start();
        assert_eq!(kept(ui), [Algorithm::Md5]);
    }
);

scenario!(
    /// The extensions stored as they are can be typed, stay through a restart, and Reset
    /// brings back the known list.
    stored_extensions_are_typed_and_reset,
    |ui| {
        let kept = |ui: &mut Ui| ui.ws(|_, _, cx| crate::prefs::prefs(cx).stored_extensions);
        let mine = Some(vec!["raw".to_owned(), "cr2".to_owned()]);
        ui.settings(2);
        assert_eq!(kept(ui), None);
        assert!(!ui.shows("extensions-reset"));
        ui.replace_in("Extensions, apart by spaces", ".RAW, cr2");
        assert_eq!(kept(ui), mine);
        assert_eq!(
            ui.ws(|_, _, cx| crate::prefs::stored_extensions(cx)),
            ["raw", "cr2"]
        );
        ui.start();
        assert_eq!(kept(ui), mine);
        ui.settings(2);
        ui.click("extensions-reset");
        assert_eq!(kept(ui), None);
        assert!(!ui.shows("extensions-reset"));
    }
);

scenario!(
    /// A language chosen is spoken after a restart, and every section of Settings draws.
    a_language_is_kept_and_spoken,
    |ui| {
        for i in 0..7 {
            ui.settings(i);
        }
        ui.ws(|_, _, cx| crate::prefs::change(cx, |p| p.language = Some("de".to_owned())));
        ui.start();
        assert_eq!(packwing_i18n::language(), "de");
        assert_eq!(packwing_i18n::tr!("Settings"), "Einstellungen");
        ui.settings(0);
    }
);

scenario!(
    /// A key that is not one is refused, and a good one stays through a restart and can be
    /// removed.
    a_license_is_entered_and_removed,
    |ui| {
        ui.settings(5);
        ui.type_in("Paste your license key", "not a key");
        ui.click("license-enter");
        assert_eq!(ui.plan(), Plan::Free);
        let key = include_str!("../../../license/testdata/enterprise.key").trim();
        ui.replace_in("Paste your license key", key);
        ui.click("license-enter");
        assert_eq!(ui.plan(), Plan::Enterprise);
        ui.start();
        assert_eq!(ui.plan(), Plan::Enterprise);
        ui.settings(5);
        ui.click("license-remove");
        assert_eq!(ui.plan(), Plan::Free);
        ui.start();
        assert_eq!(ui.plan(), Plan::Free);
    }
);

scenario!(
    /// A text size chosen in Settings draws every size at it at once, stays through
    /// a restart, and the system's comes back when chosen.
    a_text_size_is_drawn_and_kept,
    |ui| {
        let rem = |ui: &mut Ui| ui.ws(|_, window, _| window.rem_size());
        ui.settings(0);
        assert_eq!(rem(ui), gpui::px(16.), "the design's, as the tests' system is");
        ui.choose("text-size", "12 px");
        assert_eq!(ui.ws(|_, _, cx| crate::prefs::prefs(cx).text_size), Some(12));
        ui.draw();
        assert_eq!(rem(ui), crate::text_size::rem_for(12.));
        ui.start();
        ui.draw();
        assert_eq!(rem(ui), crate::text_size::rem_for(12.));
        ui.settings(0);
        ui.choose("text-size", "The system’s (15 px)");
        assert_eq!(ui.ws(|_, _, cx| crate::prefs::prefs(cx).text_size), None);
        ui.draw();
        assert_eq!(rem(ui), gpui::px(16.));
    }
);

scenario!(
    /// Where Windows shows packwing's items in one entry, Put them in a submenu is
    /// held on, and stays so as the items change.
    the_submenu_is_held_where_windows_makes_one_entry,
    |ui| {
        ui.settings(4);
        assert!(ui.shows("submenu"));
        assert!(!ui.shows("submenu-held"));
        ui.ws(|ws, _, cx| ws.right_click().update(cx, |m, cx| m.set_one_entry(cx)));
        assert!(!ui.shows("submenu"), "a switch that does nothing");
        assert!(ui.shows("submenu-held"));
        let held = |ui: &mut Ui| ui.ws(|ws, _, cx| ws.right_click().read(cx).choice().submenu);
        ui.click("item-0");
        assert!(held(ui), "the items left their submenu");
        // The first run's choice, which has no submenu, as the welcome makes it.
        ui.ws(|ws, _, cx| {
            ws.right_click()
                .update(cx, |m, cx| m.choose(crate::right_click::Choice::default(), cx))
        });
        assert!(held(ui), "chosen without a submenu");
        // Out of the menu with no items, and in again with the first run's.
        ui.ws(|ws, _, cx| {
            let none = crate::right_click::Choice {
                items: Vec::new(),
                submenu: false,
            };
            ws.right_click().update(cx, |m, cx| m.choose(none, cx))
        });
        ui.click("shown");
        assert!(ui.ws(|ws, _, cx| ws.right_click().read(cx).shown()));
        assert!(held(ui), "in again without a submenu");
    }
);

scenario!(
    /// The logo atop the menu shows Settings › About.
    the_logo_shows_about,
    |ui| {
        ui.click("nav-about");
        assert_eq!(ui.ws(|ws, _, _| ws.page()), crate::workspace::Page::Settings);
        assert_eq!(
            ui.ws(|ws, _, cx| ws.settings_section(cx)),
            crate::pages::settings::Section::About
        );
    }
);

#[cfg(any(target_os = "linux", target_os = "macos"))]
scenario!(
    /// What a backup gives back, owners by number and extended attributes, is
    /// chosen in Extracting, stays through a restart, and goes to extracting.
    what_a_backup_gives_back_is_chosen,
    |ui| {
        let kept = |ui: &mut Ui| {
            ui.ws(|_, _, cx| {
                let how = crate::prefs::prefs(cx).extract_options();
                (how.owners, how.xattrs)
            })
        };
        ui.settings(1);
        assert_eq!(kept(ui), (packwing_core::Owners::Ignore, false));
        ui.choose("restore-owners", "By number");
        ui.click("restore-xattrs");
        assert_eq!(kept(ui), (packwing_core::Owners::ByNumber, true));
        ui.start();
        assert_eq!(kept(ui), (packwing_core::Owners::ByNumber, true));
    }
);

scenario!(
    /// The audit log takes a paid plan: on Free its switch is locked. On Pro it is
    /// turned on, records jobs and its own changes, stays on through a restart,
    /// and turned off records that last.
    the_audit_log_takes_a_paid_plan_and_records_jobs,
    |ui| {
        use packwing_core::audit;
        ui.settings(3);
        assert!(!ui.shows("audit-log"), "Free has no switch");
        assert!(audit::config().is_none());
        let key = include_str!("../../../license/testdata/pro.key").trim();
        ui.settings(5);
        ui.type_in("Paste your license key", key);
        ui.click("license-enter");
        ui.settings(3);
        ui.click("audit-log");
        let folder = audit::config().expect("kept").folder;
        let file = ui.file("in/a.txt", "a");
        packwing_core::create(
            &[file],
            &ui.work.join("a.zip"),
            &packwing_core::NewArchive::default(),
            &packwing_core::Control::default(),
        )
        .expect("the archive");
        ui.choose("audit-retention", "1 year");
        assert_eq!(audit::config().expect("kept").retention_days, 365);
        ui.start();
        assert!(audit::config().is_some(), "on after a restart");
        ui.settings(3);
        ui.click("audit-log");
        assert!(audit::config().is_none());
        let records = audit::read(&folder, None).expect("the records");
        let actions: Vec<&str> = records
            .iter()
            .filter_map(|r| r["action"].as_str())
            .collect();
        assert_eq!(actions.first(), Some(&"audit.started"));
        assert!(actions.contains(&"create"));
        assert!(actions.contains(&"audit.changed"));
        assert_eq!(actions.last(), Some(&"audit.stopped"));
        assert_eq!(records[0]["by"], "user");
        assert_eq!(records[0]["app"], "packwing");
        let report = audit::verify(&folder).expect("verified");
        assert!(report.breaks.is_empty(), "{report:?}");
    }
);
