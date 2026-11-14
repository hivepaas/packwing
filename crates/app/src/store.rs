//! Pro bought in the Mac App Store, in its build (the `app-store` feature), through
//! packwing-store: whether the user holds it, its price, buying it and Restore
//! Purchases. Bought there, it is Pro for good, as a key bought outright is for the
//! versions of its year; a key bought elsewhere still counts (licensing.rs), as the
//! App Store's rules let an app take what was bought for it on other systems.
//!
//! In any other build the store is not there: [`there`] is false, and nothing is asked.

use gpui::{App, AsyncApp, Global};

/// What the App Store said.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Store {
    /// The user holds Pro, bought or restored here.
    pub holds: bool,
    /// Pro's price in the user's country; none until the App Store answers, or if it has
    /// no Pro.
    pub price: Option<String>,
    /// Whether the App Store sells Pro in the user's country (its storefront): none until
    /// it answers. Pro is not sold everywhere the app is.
    pub sold: Option<bool>,
    /// A purchase or a restore is under way.
    pub busy: bool,
    /// What the last purchase or restore came to, when the user needs to know.
    pub said: Option<Said>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Said {
    /// Someone must approve it first (Ask to Buy).
    Pending,
    /// Restore found nothing bought.
    NothingToRestore,
    Failed(String),
}

impl Global for Store {}

/// This build sells Pro in the App Store: its own, or a test's.
pub fn there() -> bool {
    #[cfg(all(test, feature = "test-hooks"))]
    if fake::on() {
        return true;
    }
    cfg!(feature = "app-store")
}

pub fn read(cx: &App) -> Store {
    cx.try_global::<Store>().cloned().unwrap_or_default()
}

/// Whether the user holds Pro from the App Store.
pub fn holds(cx: &App) -> bool {
    cx.try_global::<Store>().is_some_and(|s| s.holds)
}

fn change(cx: &mut App, f: impl FnOnce(&mut Store)) {
    let before = holds(cx);
    f(cx.default_global::<Store>());
    if holds(cx) != before {
        crate::audit_log::apply(cx, crate::audit_log::By::License);
    }
    cx.refresh_windows();
}

fn change_async(cx: &mut AsyncApp, f: impl FnOnce(&mut Store)) {
    cx.update(|cx| change(cx, f));
}

/// Asks the App Store whether the user holds Pro and what it costs, and hears from it
/// from now on: bought on another Mac, approved, refunded.
pub fn start(cx: &mut App) {
    if !there() {
        return;
    }
    let (changed_tx, mut changed_rx) = futures_channel::mpsc::unbounded::<bool>();
    watch(move |holds| {
        let _ = changed_tx.unbounded_send(holds);
    });
    cx.spawn(async move |cx| {
        if let Ok(holds) = api::holds().await {
            change_async(cx, |s| s.holds = holds);
        }
        if let Ok(price) = api::price().await {
            change_async(cx, |s| {
                s.sold = Some(price.is_some());
                s.price = price;
            });
        }
        use futures::StreamExt as _;
        while let Some(holds) = changed_rx.next().await {
            change_async(cx, |s| {
                s.holds = holds;
                if holds {
                    s.said = None;
                }
            });
        }
    })
    .detach();
}

/// Buys Pro, as the App Store asks the user.
pub fn buy(cx: &mut App) {
    if read(cx).busy {
        return;
    }
    change(cx, |s| {
        s.busy = true;
        s.said = None;
    });
    cx.spawn(async move |cx| {
        let bought = api::buy().await;
        change_async(cx, |s| {
            s.busy = false;
            match bought {
                Ok(packwing_store::Bought::Yes) => s.holds = true,
                Ok(packwing_store::Bought::Pending) => s.said = Some(Said::Pending),
                Ok(packwing_store::Bought::Cancelled) => {}
                Err(why) => s.said = Some(Said::Failed(why)),
            }
        });
    })
    .detach();
}

/// Restore Purchases.
pub fn restore(cx: &mut App) {
    if read(cx).busy {
        return;
    }
    change(cx, |s| {
        s.busy = true;
        s.said = None;
    });
    cx.spawn(async move |cx| {
        let restored = api::restore().await;
        change_async(cx, |s| {
            s.busy = false;
            match restored {
                Ok(true) => s.holds = true,
                Ok(false) => s.said = Some(Said::NothingToRestore),
                Err(why) => s.said = Some(Said::Failed(why)),
            }
        });
    })
    .detach();
}

fn watch(changed: impl Fn(bool) + Send + Sync + 'static) {
    #[cfg(all(test, feature = "test-hooks"))]
    if fake::on() {
        return fake::watch(changed);
    }
    packwing_store::watch(changed);
}

/// The App Store, or the test's.
mod api {
    use packwing_store::Bought;

    // Without the app-store feature, packwing-store has no StoreKit, and says so.
    macro_rules! call {
        ($name:ident) => {{
            #[cfg(all(test, feature = "test-hooks"))]
            if super::fake::on() {
                return super::fake::$name();
            }
            packwing_store::$name().await
        }};
    }

    pub async fn holds() -> Result<bool, String> {
        call!(holds)
    }

    pub async fn price() -> Result<Option<String>, String> {
        call!(price)
    }

    pub async fn buy() -> Result<Bought, String> {
        call!(buy)
    }

    pub async fn restore() -> Result<bool, String> {
        call!(restore)
    }
}

/// An App Store for the UI scenarios: what it answers, and a purchase made elsewhere.
#[cfg(all(test, feature = "test-hooks"))]
pub mod fake {
    use std::sync::{Arc, Mutex};

    use packwing_store::Bought;

    #[derive(Default)]
    struct Fake {
        holds: bool,
        price: Option<String>,
        buy: Option<Result<Bought, String>>,
        restore: Option<Result<bool, String>>,
        watching: Option<Arc<dyn Fn(bool) + Send + Sync>>,
    }

    static FAKE: Mutex<Option<Fake>> = Mutex::new(None);

    fn with<R>(f: impl FnOnce(&mut Fake) -> R) -> Option<R> {
        let mut fake = FAKE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        fake.as_mut().map(f)
    }

    /// From now on the App Store sells Pro at `price`, or, with none, has no Pro in the
    /// user's country; the user holds it if `holds`.
    pub fn start(price: Option<&str>, holds: bool) {
        *FAKE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Fake {
            holds,
            price: price.map(str::to_owned),
            ..Fake::default()
        });
    }

    pub fn on() -> bool {
        with(|_| ()).is_some()
    }

    /// What the next purchase comes to: bought, it is held.
    pub fn answer_buy(answer: Result<Bought, String>) {
        with(|f| f.buy = Some(answer));
    }

    /// What the next restore finds.
    pub fn answer_restore(answer: Result<bool, String>) {
        with(|f| f.restore = Some(answer));
    }

    /// A transaction comes: bought on another Mac, approved, refunded.
    pub fn transaction(holds: bool) {
        let watching = with(|f| {
            f.holds = holds;
            f.watching.clone()
        })
        .flatten();
        if let Some(watching) = watching {
            watching(holds);
        }
    }

    pub fn watch(changed: impl Fn(bool) + Send + Sync + 'static) {
        with(|f| f.watching = Some(Arc::new(changed)));
    }

    pub fn holds() -> Result<bool, String> {
        Ok(with(|f| f.holds).unwrap_or(false))
    }

    pub fn price() -> Result<Option<String>, String> {
        Ok(with(|f| f.price.clone()).flatten())
    }

    pub fn buy() -> Result<Bought, String> {
        let answer = with(|f| f.buy.take())
            .flatten()
            .unwrap_or(Ok(Bought::Cancelled));
        if answer == Ok(Bought::Yes) {
            with(|f| f.holds = true);
        }
        answer
    }

    pub fn restore() -> Result<bool, String> {
        with(|f| f.restore.take()).flatten().unwrap_or(Ok(false))
    }
}
