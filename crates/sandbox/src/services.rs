//! macOS Services: AppKit calls the provider's method an Info.plist's NSServices entry
//! names (its NSMessage, then `:userData:error:`) with the pasteboard holding the files
//! and folders chosen in Finder.
#![allow(unsafe_code)]

use std::cell::RefCell;
use std::path::PathBuf;

use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{NSApplication, NSPasteboard, NSPasteboardTypeFileURL};
use objc2_foundation::{NSString, NSURL};

type Each = Box<dyn Fn(&str, Vec<PathBuf>)>;

thread_local! {
    /// What a service does, and the provider AppKit holds only weakly: both for as long
    /// as packwing runs, on the main thread.
    static SERVING: RefCell<Option<(Each, Retained<Provider>)>> = const { RefCell::new(None) };
}

/// The files and folders on the pasteboard, as paths: Finder puts one item each, with
/// its file URL (a file reference URL, whose path is where the file is now).
fn paths(board: &NSPasteboard) -> Vec<PathBuf> {
    let Some(items) = board.pasteboardItems() else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            // SAFETY: an AppKit constant, there for as long as the program runs.
            let url = item.stringForType(unsafe { NSPasteboardTypeFileURL })?;
            let url = NSURL::URLWithString(&url)?;
            Some(PathBuf::from(url.path()?.to_string()))
        })
        .collect()
}

fn serve_one(message: &str, board: &NSPasteboard) {
    let chosen = paths(board);
    if chosen.is_empty() {
        return;
    }
    SERVING.with_borrow(|serving| {
        if let Some((each, _)) = serving {
            each(message, chosen);
        }
    });
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements, and Provider has no Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "PackwingServicesProvider"]
    struct Provider;

    impl Provider {
        #[unsafe(method(extractHere:userData:error:))]
        fn extract_here(&self, board: &NSPasteboard, _: Option<&NSString>, _: *mut *mut NSString) {
            serve_one("extractHere", board);
        }

        #[unsafe(method(extractToFolder:userData:error:))]
        fn extract_to_folder(
            &self,
            board: &NSPasteboard,
            _: Option<&NSString>,
            _: *mut *mut NSString,
        ) {
            serve_one("extractToFolder", board);
        }

        #[unsafe(method(extract:userData:error:))]
        fn extract(&self, board: &NSPasteboard, _: Option<&NSString>, _: *mut *mut NSString) {
            serve_one("extract", board);
        }

        #[unsafe(method(compressZip:userData:error:))]
        fn compress_zip(&self, board: &NSPasteboard, _: Option<&NSString>, _: *mut *mut NSString) {
            serve_one("compressZip", board);
        }

        #[unsafe(method(compress:userData:error:))]
        fn compress(&self, board: &NSPasteboard, _: Option<&NSString>, _: *mut *mut NSString) {
            serve_one("compress", board);
        }
    }
);

/// Makes packwing the provider of its Services, on the main thread.
pub fn serve(each: Each) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    // SAFETY: Provider has no instance variables, so NSObject's init initialises it.
    let provider: Retained<Provider> = unsafe { msg_send![Provider::alloc(mtm), init] };
    let app = NSApplication::sharedApplication(mtm);
    // SAFETY: the provider answers the messages Info.plist names; SERVING keeps it.
    unsafe { app.setServicesProvider(Some(&provider)) };
    SERVING.with_borrow_mut(|serving| *serving = Some((each, provider)));
}
