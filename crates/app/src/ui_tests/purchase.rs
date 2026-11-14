//! Pro bought in the Mac App Store, played by `store::fake`: Buy, Ask to Buy's
//! approval, Restore Purchases, and a refund.

use packwing_license::Plan;
use packwing_store::Bought;

use super::{Ui, scenario};
use crate::store::fake;

/// Settings › License, with the App Store selling Pro: `holds` if it is the user's.
fn license_page(ui: &mut Ui, holds: bool) {
    fake::start("$19.99", holds);
    ui.start();
    ui.settings(5);
}

scenario!(
    /// Buy Pro makes packwing Pro, for good: it is held after a restart too, and the
    /// License page sends no one to another shop.
    buying_pro_in_the_app_store_makes_it_pro,
    |ui| {
        license_page(ui, false);
        assert_eq!(ui.plan(), Plan::Free);
        assert!(ui.shows("store-restore"));
        assert!(!ui.shows("license-buy"), "a way out to the website");
        fake::answer_buy(Ok(Bought::Yes));
        ui.click("store-buy");
        ui.wait_until("Pro", |ui| ui.plan() == Plan::Pro);
        ui.wait_for("store-held");
        assert!(!ui.shows("store-buy"));
        ui.start();
        ui.wait_until("Pro again", |ui| ui.plan() == Plan::Pro);
    }
);

scenario!(
    /// A purchase someone must approve says so, and packwing turns Pro when the
    /// approval comes as a transaction.
    a_purchase_waiting_for_approval_turns_pro_once_approved,
    |ui| {
        license_page(ui, false);
        fake::answer_buy(Ok(Bought::Pending));
        ui.click("store-buy");
        ui.wait_for("store-said");
        assert_eq!(ui.plan(), Plan::Free);
        fake::transaction(true);
        ui.wait_until("Pro", |ui| ui.plan() == Plan::Pro);
        assert!(!ui.shows("store-said"));
    }
);

scenario!(
    /// Restore Purchases says when it finds nothing, and makes packwing Pro when it
    /// finds Pro; a refund later makes it Free again.
    restore_finds_pro_and_a_refund_takes_it_back,
    |ui| {
        license_page(ui, false);
        fake::answer_restore(Ok(false));
        ui.click("store-restore");
        ui.wait_for("store-said");
        assert_eq!(ui.plan(), Plan::Free);
        fake::answer_restore(Ok(true));
        ui.click("store-restore");
        ui.wait_until("Pro", |ui| ui.plan() == Plan::Pro);
        fake::transaction(false);
        ui.wait_until("Free", |ui| ui.plan() == Plan::Free);
        ui.wait_for("store-buy");
    }
);

scenario!(
    /// Outside the App Store's build, the License page has its Buy, and no App Store.
    elsewhere_the_license_page_has_no_app_store,
    |ui| {
        ui.settings(5);
        assert!(ui.shows("license-buy"));
        assert!(!ui.shows("store-buy"));
    }
);
