//! In-app purchases for the Mac App Store's packwing: Pro, bought once in the App Store
//! ([`PRO`]), through StoreKit 2 (swift/, which only Swift can call). Only what the App
//! Store signed counts; StoreKit checks it on this Mac.
//!
//! Elsewhere (another system, or a build without the `storekit` feature) every call
//! says there is no App Store.

#[cfg(storekit)]
mod macos;

/// The product: packwing Pro, a non-consumable in App Store Connect.
pub const PRO: &str = "com.hivepaas.packwing.pro";

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

/// Whether the user holds [`PRO`]: bought or restored, and not refunded.
pub async fn holds() -> Result<bool, String> {
    #[cfg(storekit)]
    {
        macos::holds(PRO).await
    }
    #[cfg(not(storekit))]
    {
        none()
    }
}

/// [`PRO`]'s price, as the App Store shows it in the user's country; none when it has
/// no such product.
pub async fn price() -> Result<Option<String>, String> {
    #[cfg(storekit)]
    {
        macos::price(PRO).await
    }
    #[cfg(not(storekit))]
    {
        none()
    }
}

/// Buys [`PRO`], as the App Store asks the user.
pub async fn buy() -> Result<Bought, String> {
    #[cfg(storekit)]
    {
        macos::buy(PRO).await
    }
    #[cfg(not(storekit))]
    {
        none()
    }
}

/// Restore Purchases: asks the App Store for what the user bought (it may ask them to
/// sign in), then whether they hold [`PRO`].
pub async fn restore() -> Result<bool, String> {
    #[cfg(storekit)]
    {
        macos::restore(PRO).await
    }
    #[cfg(not(storekit))]
    {
        none()
    }
}

/// Calls `changed` with whether the user holds [`PRO`] each time a transaction comes, for
/// as long as packwing runs: bought on another Mac, approved, refunded. Call it once.
pub fn watch(changed: impl Fn(bool) + Send + Sync + 'static) {
    #[cfg(storekit)]
    macos::watch(PRO, changed);
    #[cfg(not(storekit))]
    let _ = changed;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Swift library is built and linked, and answers: outside the App Store (a
    /// test is not an app it sold), the product is not there, or StoreKit says why.
    #[test]
    fn the_app_store_answers_or_says_why_not() {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(futures::executor::block_on(async {
                (price().await, holds().await)
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
