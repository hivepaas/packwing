//! What the window takes from the organization's policy.

use gpui::App;
#[cfg(all(test, feature = "test-hooks"))]
use gpui::Global;
use packwing_policy::Policy;

/// A policy in place of the system's, for the screenshots.
#[cfg(all(test, feature = "test-hooks"))]
pub struct Shown(pub Policy);

#[cfg(all(test, feature = "test-hooks"))]
impl Global for Shown {}

/// What is fixed on this computer.
#[cfg(not(all(test, feature = "test-hooks")))]
pub fn policy(_: &App) -> &'static Policy {
    packwing_policy::current()
}

/// What is fixed on this computer, or for this screenshot.
#[cfg(all(test, feature = "test-hooks"))]
pub fn policy(cx: &App) -> &Policy {
    match cx.try_global::<Shown>() {
        Some(shown) => &shown.0,
        None => packwing_policy::current(),
    }
}
