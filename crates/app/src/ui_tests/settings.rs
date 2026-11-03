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
