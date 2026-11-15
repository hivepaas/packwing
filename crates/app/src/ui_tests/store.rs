//! The Mac App Store's build (`--features ui-tests,app-store`): what it leaves out.
//! The other scenarios are the build for everywhere else's, and are not run with it.

use super::scenario;

scenario!(
    /// Settings' right-click menu says packwing's items are macOS services and how to
    /// turn them on and off, with no switch of its own; and About says the App Store
    /// updates packwing, with nothing to look for updates with.
    settings_leave_out_what_the_app_store_does,
    |ui| {
        ui.click("nav-settings");
        ui.click("section-4");
        assert!(ui.shows("services-note"));
        for i in 0..5 {
            assert!(ui.shows(&format!("service-{i}")), "service {i}");
        }
        assert!(ui.shows("services-settings"));
        assert!(!ui.shows("shown"), "the menu's switch");
        assert!(!ui.shows("submenu"), "the submenu's switch");
        ui.click("section-5");
        ui.click("section-6");
        assert!(!ui.shows("update-check"));
        assert!(!ui.shows("update-auto"));
    }
);
