import Foundation
import Observation
import StoreKit
import SwiftUI

/// packwing Pro, bought once in the App Store: the same purchase as the Mac
/// App Store's (one app record, Universal Purchase), so Pro bought on a Mac is
/// Pro here too. Only a transaction the App Store signed counts.
@MainActor
@Observable
final class Store {
    static let pro = "com.hivepaas.packwing.pro"

    /// Whether the user holds Pro: bought or restored, and not refunded.
    private(set) var holds = false
    /// Its price, as the App Store shows it in the user's country.
    private(set) var price: String?
    /// Whether the App Store sells it in the user's country; nothing until it said.
    private(set) var sold: Bool?
    /// A purchase or a restore going on.
    private(set) var busy = false
    /// What the last of them came to, when it is to be said.
    private(set) var said: Said?

    enum Said: Equatable {
        /// Someone must approve it first (Ask to Buy).
        case pending
        case nothingToRestore
        case failed(String)
    }

    /// A store of no App Store, for the UI tests (`-store fake`): Buy and
    /// Restore answer at once, and what was bought is kept in the settings.
    private let fake: Bool

    init() {
        #if DEBUG
        fake = UserDefaults.standard.string(forKey: "store") == "fake"
        #else
        fake = false
        #endif
        if fake {
            holds = UserDefaults.standard.bool(forKey: "fakePro")
            price = "$9.99"
            sold = true
            return
        }
        Task { await load() }
        // Bought on another device, approved, refunded: as it comes.
        Task.detached { [weak self] in
            for await update in Transaction.updates {
                if case .verified(let transaction) = update {
                    await transaction.finish()
                }
                await self?.refresh()
            }
        }
    }

    private func load() async {
        await refresh()
        do {
            let products = try await Product.products(for: [Self.pro])
            price = products.first?.displayPrice
            sold = !products.isEmpty
        } catch {
            // Asked again when Buy is pressed.
        }
    }

    private func refresh() async {
        var held = false
        for await result in Transaction.currentEntitlements {
            if case .verified(let transaction) = result, transaction.productID == Self.pro,
               transaction.revocationDate == nil {
                held = true
            }
        }
        holds = held
    }

    func buy() {
        guard !busy else { return }
        said = nil
        if fake {
            holds = true
            UserDefaults.standard.set(true, forKey: "fakePro")
            return
        }
        busy = true
        Task {
            defer { busy = false }
            do {
                guard let product = try await Product.products(for: [Self.pro]).first else {
                    sold = false
                    return
                }
                price = product.displayPrice
                sold = true
                switch try await product.purchase() {
                case .success(.verified(let transaction)):
                    await transaction.finish()
                    holds = true
                case .success(.unverified(_, let why)):
                    said = .failed(why.localizedDescription)
                case .pending:
                    said = .pending
                case .userCancelled:
                    break
                @unknown default:
                    break
                }
            } catch {
                said = .failed(error.localizedDescription)
            }
        }
    }

    /// Restore Purchases: what the user bought with their Apple Account (it may
    /// ask them to sign in).
    func restore() {
        guard !busy else { return }
        said = nil
        if fake {
            holds = UserDefaults.standard.bool(forKey: "fakePro")
            if !holds { said = .nothingToRestore }
            return
        }
        busy = true
        Task {
            defer { busy = false }
            do {
                try await AppStore.sync()
                await refresh()
                if !holds { said = .nothingToRestore }
            } catch {
                said = .failed(error.localizedDescription)
            }
        }
    }

    /// Buy Pro, with its price once the App Store said it.
    var buyLabel: String {
        price.map { tr("Buy Pro · {price}", ["price": $0]) } ?? tr("Buy Pro")
    }
}

/// Settings' License: what the plan gives, and Pro bought, or to buy.
struct LicenseSection: View {
    @Environment(AppModel.self) private var model

    var body: some View {
        let store = model.store
        Section {
            VStack(alignment: .leading, spacing: 4) {
                Text(store.holds ? "packwing Pro" : "packwing Free")
                    .font(.nunito(17, .bold, relativeTo: .headline))
                    .accessibilityIdentifier("store-plan")
                Text(store.holds
                     ? tr("Every feature, and any number of storages.")
                     : tr("Every feature, and one storage across a network."))
                    .font(.nunito(14, relativeTo: .subheadline))
                    .foregroundStyle(Tonal.muted)
            }
            if store.holds {
                Label(tr("Bought"), systemImage: "checkmark.seal.fill")
                    .foregroundStyle(Tonal.accent)
                    .accessibilityIdentifier("store-held")
            } else {
                if store.sold != false {
                    Button(store.buyLabel) { store.buy() }
                        .disabled(store.busy)
                        .accessibilityIdentifier("store-buy")
                }
                Button(tr("Restore Purchases")) { store.restore() }
                    .disabled(store.busy)
                    .accessibilityIdentifier("store-restore")
                if store.sold == false {
                    Text(tr("The App Store does not sell Pro in your country or region. Pro bought with this Apple Account in another one can still be restored."))
                        .font(.nunito(14, relativeTo: .subheadline))
                        .foregroundStyle(Tonal.warnText)
                }
            }
            if let said = store.said {
                Text(words(said))
                    .font(.nunito(14, relativeTo: .subheadline))
                    .foregroundStyle(said == .pending ? Tonal.onTonal : Tonal.warnText)
                    .accessibilityIdentifier("store-said")
            }
        } header: {
            Text(tr("License"))
        } footer: {
            Text(tr("Pro, bought once in the App Store: yours on every iPhone, iPad and Mac with your Apple Account."))
        }
        .listRowBackground(Tonal.surface)
    }

    private func words(_ said: Store.Said) -> String {
        switch said {
        case .pending: tr("Waiting for approval: packwing turns Pro once it is given.")
        case .nothingToRestore: tr("The App Store found nothing bought with this Apple Account.")
        case .failed(let why): tr("The App Store could not do it: {why}", ["why": why])
        }
    }
}
