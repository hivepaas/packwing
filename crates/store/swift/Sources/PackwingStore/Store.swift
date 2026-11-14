// StoreKit 2, which only Swift can call, as C functions for crates/store (src/macos.rs).
// Each takes the product's id, a context and a function it calls once with the context
// and its answer, on a thread of its own; a text it passes lasts for that call alone.
// Only a transaction the App Store signed (StoreKit's `.verified`) counts.

import Foundation
import StoreKit

/// `done(context, status, text)`.
public typealias Done = @convention(c) (UnsafeMutableRawPointer?, Int32, UnsafePointer<CChar>?) -> Void

/// What `done` is told.
private enum Status: Int32 {
    case no = 0
    case yes = 1
    case pending = 2
    case cancelled = 3
    case failed = -1
}

private func tell(_ done: Done, _ context: UnsafeMutableRawPointer?, _ status: Status, _ text: String? = nil) {
    if let text {
        text.withCString { done(context, status.rawValue, $0) }
    } else {
        done(context, status.rawValue, nil)
    }
}

/// Whether the user holds `id`: bought, or restored, and not refunded.
private func holds(_ id: String) async -> Bool {
    for await result in Transaction.currentEntitlements {
        if case .verified(let transaction) = result,
           transaction.productID == id,
           transaction.revocationDate == nil
        {
            return true
        }
    }
    return false
}

private func product(_ id: String) async throws -> Product? {
    try await Product.products(for: [id]).first
}

/// Whether the user holds the product: `yes` or `no`.
@_cdecl("packwing_store_holds")
public func packwingStoreHolds(_ id: UnsafePointer<CChar>, _ context: UnsafeMutableRawPointer?, _ done: Done) {
    let id = String(cString: id)
    Task.detached {
        tell(done, context, await holds(id) ? .yes : .no)
    }
}

/// The product's price, as the App Store shows it in the user's country: `yes` and the
/// price, `no` when the store has no such product, or `failed` and why.
@_cdecl("packwing_store_price")
public func packwingStorePrice(_ id: UnsafePointer<CChar>, _ context: UnsafeMutableRawPointer?, _ done: Done) {
    let id = String(cString: id)
    Task.detached {
        do {
            if let product = try await product(id) {
                tell(done, context, .yes, product.displayPrice)
            } else {
                tell(done, context, .no)
            }
        } catch {
            tell(done, context, .failed, error.localizedDescription)
        }
    }
}

/// Buys the product, as the App Store asks the user: `yes` once it is theirs, `pending`
/// while someone must approve it (Ask to Buy), `cancelled`, `no` when the store has no
/// such product, or `failed` and why.
@_cdecl("packwing_store_buy")
public func packwingStoreBuy(_ id: UnsafePointer<CChar>, _ context: UnsafeMutableRawPointer?, _ done: Done) {
    let id = String(cString: id)
    Task.detached {
        do {
            guard let product = try await product(id) else {
                tell(done, context, .no)
                return
            }
            switch try await product.purchase() {
            case .success(.verified(let transaction)):
                await transaction.finish()
                tell(done, context, .yes)
            case .success(.unverified(_, let why)):
                tell(done, context, .failed, why.localizedDescription)
            case .pending:
                tell(done, context, .pending)
            case .userCancelled:
                tell(done, context, .cancelled)
            @unknown default:
                tell(done, context, .failed, "unknown")
            }
        } catch {
            tell(done, context, .failed, error.localizedDescription)
        }
    }
}

/// Asks the App Store for what the user bought, as Restore Purchases does (it may ask
/// them to sign in): then whether they hold the product, `yes` or `no`, or `failed`.
@_cdecl("packwing_store_restore")
public func packwingStoreRestore(_ id: UnsafePointer<CChar>, _ context: UnsafeMutableRawPointer?, _ done: Done) {
    let id = String(cString: id)
    Task.detached {
        do {
            try await AppStore.sync()
            tell(done, context, await holds(id) ? .yes : .no)
        } catch {
            tell(done, context, .failed, error.localizedDescription)
        }
    }
}

/// From now on, for as long as packwing runs, tells `done` whether the user holds the
/// product each time a transaction comes: bought on another Mac, approved, refunded.
/// `done` is called many times, with the same context.
@_cdecl("packwing_store_watch")
public func packwingStoreWatch(_ id: UnsafePointer<CChar>, _ context: UnsafeMutableRawPointer?, _ done: Done) {
    let id = String(cString: id)
    Task.detached {
        for await update in Transaction.updates {
            if case .verified(let transaction) = update {
                await transaction.finish()
            }
            tell(done, context, await holds(id) ? .yes : .no)
        }
    }
}
