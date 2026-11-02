//! Settings: what is chosen stays, and the license is taken or refused.

use super::scenario;
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
