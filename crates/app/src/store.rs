//! Pro by subscription in the Mac App Store, in its build (the `app-store` feature),
//! through packwing-store: whether the user holds it, each plan's price, subscribing and
//! Restore Purchases. It is Pro while the subscription is current, and Free once it ends;
//! a key bought elsewhere still counts (licensing.rs), as the App Store's rules let an
//! app take what was bought for it on other systems.
//!
//! In any other build the store is not there: [`there`] is false, and nothing is asked.

use gpui::{App, AsyncApp, Global};
pub use packwing_store::Plan;

/// Where the user manages or cancels their subscriptions: the App Store's account page.
pub const MANAGE: &str = "https://apps.apple.com/account/subscriptions";

/// What the App Store said.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Store {
    /// The user holds Pro: a subscription of theirs is current.
    pub holds: bool,
    /// The monthly and yearly plans' prices in the user's country; none until the App
    /// Store answers, or if it has no such plan there.
    pub monthly: Option<String>,
    pub yearly: Option<String>,
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

impl Store {
    /// `plan`'s price, as the App Store said it.
    pub fn price(&self, plan: Plan) -> Option<&String> {
        match plan {
            Plan::Monthly => self.monthly.as_ref(),
            Plan::Yearly => self.yearly.as_ref(),
        }
    }
}

/// Asks the App Store whether the user holds Pro and what each plan costs, and hears
/// from it from now on: bought on another Mac, renewed, approved, ended, refunded.
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
        if let (Ok(monthly), Ok(yearly)) = (
            api::price(Plan::Monthly).await,
            api::price(Plan::Yearly).await,
        ) {
            change_async(cx, |s| {
                s.sold = Some(monthly.is_some() || yearly.is_some());
                s.monthly = monthly;
                s.yearly = yearly;
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

/// Subscribes to `plan`, as the App Store asks the user.
pub fn buy(plan: Plan, cx: &mut App) {
    if read(cx).busy {
        return;
    }
    change(cx, |s| {
        s.busy = true;
        s.said = None;
    });
    cx.spawn(async move |cx| {
        let bought = api::buy(plan).await;
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
    use packwing_store::{Bought, Plan};

    // Without the app-store feature, packwing-store has no StoreKit, and says so.
    macro_rules! call {
        ($name:ident $(, $arg:expr)?) => {{
            #[cfg(all(test, feature = "test-hooks"))]
            if super::fake::on() {
                return super::fake::$name($($arg)?);
            }
            packwing_store::$name($($arg)?).await
        }};
    }

    pub async fn holds() -> Result<bool, String> {
        call!(holds)
    }

    pub async fn price(plan: Plan) -> Result<Option<String>, String> {
        call!(price, plan)
    }

    pub async fn buy(plan: Plan) -> Result<Bought, String> {
        call!(buy, plan)
    }

    pub async fn restore() -> Result<bool, String> {
        call!(restore)
    }
}

/// An App Store for the UI scenarios: what it answers, and a purchase made elsewhere.
#[cfg(all(test, feature = "test-hooks"))]
pub mod fake {
    use std::sync::{Arc, Mutex};

    use packwing_store::{Bought, Plan};

    #[derive(Default)]
    struct Fake {
        holds: bool,
        /// The monthly and yearly prices.
        prices: Option<(String, String)>,
        /// The plan the last purchase was of.
        bought: Option<Plan>,
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

    /// From now on the App Store sells Pro at `prices`, a month's and a year's, or, with
    /// none, has no Pro in the user's country; the user holds it if `holds`.
    pub fn start(prices: Option<(&str, &str)>, holds: bool) {
        *FAKE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Fake {
            holds,
            prices: prices.map(|(m, y)| (m.to_owned(), y.to_owned())),
            ..Fake::default()
        });
    }

    /// The plan the last purchase was of.
    pub fn bought() -> Option<Plan> {
        with(|f| f.bought).flatten()
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

    pub fn price(plan: Plan) -> Result<Option<String>, String> {
        Ok(with(|f| {
            f.prices.clone().map(|(m, y)| match plan {
                Plan::Monthly => m,
                Plan::Yearly => y,
            })
        })
        .flatten())
    }

    pub fn buy(plan: Plan) -> Result<Bought, String> {
        with(|f| f.bought = Some(plan));
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
