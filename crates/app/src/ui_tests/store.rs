//! The Mac App Store's build (`--features ui-tests,app-store`): what it leaves out.
//! The other scenarios are the build for everywhere else's, and are not run with it.

use super::scenario;

scenario!(
    /// Settings has no section for the right-click menu, and About says the App Store
    /// updates packwing, with nothing to look for updates with.
    settings_leave_out_what_the_app_store_does,
    |ui| {
        ui.click("nav-settings");
        assert!(ui.shows("section-3"));
        assert!(!ui.shows("section-4"), "the right-click menu's section");
        ui.click("section-5");
        ui.click("section-6");
        assert!(!ui.shows("update-check"));
        assert!(!ui.shows("update-auto"));
    }
);
