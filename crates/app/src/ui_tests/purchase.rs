//! Pro by subscription in the Mac App Store, played by `store::fake`: a month or a year,
//! Ask to Buy's approval, Restore Purchases, and a subscription that ends.

use packwing_license::Plan;
use packwing_store::{Bought, Plan as StorePlan};

use super::{Ui, scenario};
use crate::store::fake;

/// Settings › License, with the App Store selling Pro by the month and by the year:
/// `holds` if the user is subscribed.
fn license_page(ui: &mut Ui, holds: bool) {
    fake::start(Some(("$0.99", "$8.99")), holds);
    ui.start();
    ui.settings(5);
}

scenario!(
    /// Subscribing for a year makes packwing Pro: it is held after a restart too, the
    /// page then offers Manage Subscription, and it sends no one to another shop. Where
    /// it is sold, it says how the subscription renews, with the terms and the privacy
    /// policy.
    subscribing_in_the_app_store_makes_it_pro,
    |ui| {
        license_page(ui, false);
        assert_eq!(ui.plan(), Plan::Free);
        ui.wait_for("store-buy-monthly");
        for shown in ["store-buy-yearly", "store-restore", "store-terms", "store-privacy"] {
            assert!(ui.shows(shown), "{shown}");
        }
        assert!(!ui.shows("license-buy"), "a way out to the website");
        fake::answer_buy(Ok(Bought::Yes));
        ui.click("store-buy-yearly");
        ui.wait_until("Pro", |ui| ui.plan() == Plan::Pro);
        assert_eq!(fake::bought(), Some(StorePlan::Yearly));
        ui.wait_for("store-held");
        assert!(ui.shows("store-manage"));
        assert!(!ui.shows("store-buy-monthly"));
        ui.start();
        ui.wait_until("Pro again", |ui| ui.plan() == Plan::Pro);
    }
);

scenario!(
    /// A price the App Store leaves out at first is asked for again, so both plans
    /// show theirs.
    a_price_left_out_at_first_is_asked_again,
    |ui| {
        fake::start(Some(("$0.99", "$8.99")), false);
        fake::monthly_late(1);
        ui.start();
        ui.settings(5);
        ui.wait_until("the yearly price", |ui| {
            ui.ws(|_, _, cx| crate::store::read(cx)).yearly.is_some()
        });
        assert!(ui.ws(|_, _, cx| crate::store::read(cx)).monthly.is_none());
        ui.cx
            .executor()
            .advance_clock(std::time::Duration::from_secs(4));
        ui.wait_until("both prices", |ui| {
            let s = ui.ws(|_, _, cx| crate::store::read(cx));
            s.monthly.is_some() && s.yearly.is_some()
        });
    }
);

scenario!(
    /// A month's subscription is the monthly plan.
    subscribing_for_a_month_buys_the_monthly_plan,
    |ui| {
        license_page(ui, false);
        fake::answer_buy(Ok(Bought::Yes));
        ui.click("store-buy-monthly");
        ui.wait_until("Pro", |ui| ui.plan() == Plan::Pro);
        assert_eq!(fake::bought(), Some(StorePlan::Monthly));
    }
);

scenario!(
    /// A purchase someone must approve says so, and packwing turns Pro when the
    /// approval comes as a transaction.
    a_purchase_waiting_for_approval_turns_pro_once_approved,
    |ui| {
        license_page(ui, false);
        fake::answer_buy(Ok(Bought::Pending));
        ui.click("store-buy-yearly");
        ui.wait_for("store-said");
        assert_eq!(ui.plan(), Plan::Free);
        fake::transaction(true);
        ui.wait_until("Pro", |ui| ui.plan() == Plan::Pro);
        assert!(!ui.shows("store-said"));
    }
);

scenario!(
    /// Restore Purchases says when it finds nothing, and makes packwing Pro when it
    /// finds a subscription; once it ends, packwing is Free again.
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
        ui.wait_for("store-buy-yearly");
    }
);

scenario!(
    /// Where the App Store does not sell Pro, the License page offers no Buy, says why,
    /// sends no one to another shop, and still restores what was bought elsewhere.
    where_pro_is_not_sold_there_is_no_buy_but_restore,
    |ui| {
        fake::start(None, false);
        ui.start();
        ui.settings(5);
        ui.wait_for("store-not-sold");
        assert!(!ui.shows("store-buy-yearly") && !ui.shows("store-buy-monthly"));
        assert!(!ui.shows("license-buy"), "a way out to the website");
        fake::answer_restore(Ok(true));
        ui.click("store-restore");
        ui.wait_until("Pro", |ui| ui.plan() == Plan::Pro);
        ui.wait_for("store-held");
        assert!(!ui.shows("store-not-sold"));
    }
);

scenario!(
    /// Outside the App Store's build, the License page has its Buy, and no App Store.
    elsewhere_the_license_page_has_no_app_store,
    |ui| {
        ui.settings(5);
        assert!(ui.shows("license-buy"));
        assert!(!ui.shows("store-buy-yearly"));
    }
);
