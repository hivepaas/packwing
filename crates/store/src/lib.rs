//! In-app purchases for the Mac App Store's packwing: Pro, as a subscription of the App
//! Store, a month or a year ([`Plan`]), through StoreKit 2 (swift/, which only Swift can
//! call). Only what the App Store signed counts; StoreKit checks it on this Mac. Pro is
//! held while either is: one bought, renewed, in its billing grace period, and not
//! refunded.
//!
//! Elsewhere (another system, or a build without the `storekit` feature) every call
//! says there is no App Store.

#[cfg(storekit)]
mod macos;

/// The subscriptions to packwing Pro, of one subscription group in App Store Connect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Plan {
    Monthly,
    Yearly,
}

impl Plan {
    /// As the License page offers them.
    pub const ALL: [Plan; 2] = [Plan::Monthly, Plan::Yearly];

    /// Its product id in App Store Connect.
    pub fn id(self) -> &'static str {
        match self {
            Plan::Monthly => "com.hivepaas.packwing.pro.monthly",
            Plan::Yearly => "com.hivepaas.packwing.pro.yearly",
        }
    }
}

/// Every plan's id, comma-separated, as the Swift code takes a set of them.
#[cfg_attr(not(storekit), allow(dead_code))]
fn every_id() -> String {
    Plan::ALL.map(Plan::id).join(",")
}

/// How a purchase went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bought {
    /// It is the user's.
    Yes,
    /// Someone must approve it first (Ask to Buy): [`watch`] tells when they do.
    Pending,
    /// The user said no.
    Cancelled,
}

#[cfg(not(storekit))]
fn none<T>() -> Result<T, String> {
    Err("no App Store here".to_owned())
}

/// Whether the user holds Pro: a plan of theirs is current, and not refunded.
pub async fn holds() -> Result<bool, String> {
    #[cfg(storekit)]
    {
        macos::holds(&every_id()).await
    }
    #[cfg(not(storekit))]
    {
        none()
    }
}

/// `plan`'s price for its period, as the App Store shows it in the user's country; none
/// when it has no such product there.
pub async fn price(plan: Plan) -> Result<Option<String>, String> {
    #[cfg(storekit)]
    {
        macos::price(plan.id()).await
    }
    #[cfg(not(storekit))]
    {
        let _ = plan;
        none()
    }
}

/// Subscribes to `plan`, as the App Store asks the user. From another plan, the App
/// Store moves the subscription; it is never held twice.
pub async fn buy(plan: Plan) -> Result<Bought, String> {
    #[cfg(storekit)]
    {
        macos::buy(plan.id()).await
    }
    #[cfg(not(storekit))]
    {
        let _ = plan;
        none()
    }
}

/// Restore Purchases: asks the App Store for what the user bought (it may ask them to
/// sign in), then whether they hold Pro.
pub async fn restore() -> Result<bool, String> {
    #[cfg(storekit)]
    {
        macos::restore(&every_id()).await
    }
    #[cfg(not(storekit))]
    {
        none()
    }
}

/// Calls `changed` with whether the user holds Pro each time a transaction comes, for as
/// long as packwing runs: bought on another Mac, renewed, approved, ended, refunded. Call
/// it once.
pub fn watch(changed: impl Fn(bool) + Send + Sync + 'static) {
    #[cfg(storekit)]
    macos::watch(&every_id(), changed);
    #[cfg(not(storekit))]
    let _ = changed;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plans_are_two_products() {
        assert_eq!(
            every_id(),
            "com.hivepaas.packwing.pro.monthly,com.hivepaas.packwing.pro.yearly"
        );
    }

    /// The Swift library is built and linked, and answers: outside the App Store (a
    /// test is not an app it sold), the product is not there, or StoreKit says why.
    #[test]
    fn the_app_store_answers_or_says_why_not() {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(futures::executor::block_on(async {
                (price(Plan::Yearly).await, holds().await)
            }));
        });
        let (price, holds) = rx
            .recv_timeout(std::time::Duration::from_secs(60))
            .expect("StoreKit answers within a minute");
        if cfg!(storekit) {
            assert!(matches!(price, Ok(None) | Err(_)), "{price:?}");
            assert!(matches!(holds, Ok(false) | Err(_)), "{holds:?}");
        } else {
            assert!(price.is_err() && holds.is_err());
        }
    }
}
