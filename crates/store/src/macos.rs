//! The Swift library's C functions (swift/Sources/PackwingStore/Store.swift): each takes
//! the product's id, a context and a function it calls with the context and its answer,
//! once, on a thread of its own; `packwing_store_watch` calls it again and again.
#![allow(unsafe_code)]

use std::ffi::{CStr, CString, c_char, c_void};

use futures_channel::oneshot;

use crate::Bought;

type Done = extern "C" fn(*mut c_void, i32, *const c_char);

unsafe extern "C" {
    fn packwing_store_holds(id: *const c_char, context: *mut c_void, done: Done);
    fn packwing_store_price(id: *const c_char, context: *mut c_void, done: Done);
    fn packwing_store_buy(id: *const c_char, context: *mut c_void, done: Done);
    fn packwing_store_restore(id: *const c_char, context: *mut c_void, done: Done);
    fn packwing_store_watch(id: *const c_char, context: *mut c_void, done: Done);
}

/// The Swift code's statuses.
const NO: i32 = 0;
const YES: i32 = 1;
const PENDING: i32 = 2;
const CANCELLED: i32 = 3;

type Answer = (i32, Option<String>);

/// The text Swift passes, copied: it lasts for the call alone.
fn text_of(text: *const c_char) -> Option<String> {
    // SAFETY: Swift passes null or a C string that lasts for the call.
    (!text.is_null()).then(|| {
        unsafe { CStr::from_ptr(text) }
            .to_string_lossy()
            .into_owned()
    })
}

extern "C" fn answered(context: *mut c_void, status: i32, text: *const c_char) {
    // SAFETY: the context is the Box `ask` leaked, and Swift calls this once with it.
    let answer = unsafe { Box::from_raw(context.cast::<oneshot::Sender<Answer>>()) };
    let _ = answer.send((status, text_of(text)));
}

/// Calls `f` with `id`, and waits for its answer.
async fn ask(
    f: unsafe extern "C" fn(*const c_char, *mut c_void, Done),
    id: &str,
) -> Result<Answer, String> {
    let id = CString::new(id).map_err(|e| e.to_string())?;
    let (answer, answered_rx) = oneshot::channel::<Answer>();
    let context = Box::into_raw(Box::new(answer)).cast::<c_void>();
    // SAFETY: the id lasts for the call (Swift copies it); `answered` takes the context
    // back, once.
    unsafe { f(id.as_ptr(), context, answered) };
    answered_rx
        .await
        .map_err(|_| "the App Store did not answer".to_owned())
}

fn failed((status, text): Answer) -> String {
    text.unwrap_or_else(|| format!("the App Store answered {status}"))
}

pub async fn holds(id: &str) -> Result<bool, String> {
    match ask(packwing_store_holds, id).await? {
        (YES, _) => Ok(true),
        (NO, _) => Ok(false),
        other => Err(failed(other)),
    }
}

pub async fn price(id: &str) -> Result<Option<String>, String> {
    match ask(packwing_store_price, id).await? {
        (YES, Some(price)) => Ok(Some(price)),
        (NO, _) => Ok(None),
        other => Err(failed(other)),
    }
}

pub async fn buy(id: &str) -> Result<Bought, String> {
    match ask(packwing_store_buy, id).await? {
        (YES, _) => Ok(Bought::Yes),
        (PENDING, _) => Ok(Bought::Pending),
        (CANCELLED, _) => Ok(Bought::Cancelled),
        (NO, _) => Err("the App Store has no such product".to_owned()),
        other => Err(failed(other)),
    }
}

pub async fn restore(id: &str) -> Result<bool, String> {
    match ask(packwing_store_restore, id).await? {
        (YES, _) => Ok(true),
        (NO, _) => Ok(false),
        other => Err(failed(other)),
    }
}

type Changed = Box<dyn Fn(bool) + Send + Sync>;

extern "C" fn changed(context: *mut c_void, status: i32, _: *const c_char) {
    // SAFETY: the context is the Box `watch` leaked, which is never freed.
    let changed = unsafe { &*context.cast::<Changed>() };
    changed(status == YES);
}

pub fn watch(id: &str, f: impl Fn(bool) + Send + Sync + 'static) {
    let Ok(id) = CString::new(id) else {
        return;
    };
    let f: Changed = Box::new(f);
    // Kept for as long as packwing runs, as Swift keeps calling it.
    let context = Box::into_raw(Box::new(f)).cast::<c_void>();
    // SAFETY: the id lasts for the call (Swift copies it); the context is never freed.
    unsafe { packwing_store_watch(id.as_ptr(), context, changed) };
}
