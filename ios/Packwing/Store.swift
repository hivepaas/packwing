import Foundation
import Observation
import StoreKit
import SwiftUI

/// packwing Pro by subscription in the App Store, a month or a year: the same
/// subscriptions as the Mac App Store's (one app record, Universal Purchase),
/// so Pro subscribed to on a Mac is Pro here too. Only a transaction the App
/// Store signed counts, and only while the subscription is current.
@MainActor
@Observable
final class Store {
    /// The plans, of one subscription group.
    enum Plan: CaseIterable {
        case monthly, yearly

        var id: String {
            switch self {
            case .monthly: "com.hivepaas.packwing.pro.monthly"
            case .yearly: "com.hivepaas.packwing.pro.yearly"
            }
        }

        static let ids = Set(allCases.map(\.id))
    }

    /// Whether the user holds Pro: subscribed (renewed, or in the billing
    /// grace period), and not refunded.
    private(set) var holds = false
    /// The plans' prices, as the App Store shows them in the user's country.
    private(set) var prices: [Plan: String] = [:]
    /// Whether the App Store sells Pro in the user's country; nothing until it said.
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

    /// A store of no App Store, for the UI tests (`-store fake`): a purchase
    /// and Restore answer at once, and what was bought is kept in the settings;
    /// Manage Subscription ends it. `-fakeNoMonthly YES` leaves the monthly
    /// price out, as the sandbox did now and then.
    private let fake: Bool

    /// How often the App Store is asked for a price it left out, and how far apart.
    private static let priceAttempts = 5
    private static let priceAgain: Duration = .seconds(3)

    init() {
        #if DEBUG
        fake = UserDefaults.standard.string(forKey: "store") == "fake"
        #else
        fake = false
        #endif
        if fake {
            holds = UserDefaults.standard.bool(forKey: "fakePro")
            prices = [.monthly: "$0.99", .yearly: "$8.99"]
            if UserDefaults.standard.bool(forKey: "fakeNoMonthly") { prices[.monthly] = nil }
            sold = true
            return
        }
        Task { await refresh() }
        Task { await loadPrices() }
        // Subscribed on another device, renewed, approved, refunded: as it comes.
        Task.detached { [weak self] in
            for await update in Transaction.updates {
                if case .verified(let transaction) = update {
                    await transaction.finish()
                }
                await self?.refresh()
            }
        }
        // A subscription that ends sends no transaction: asked again now and then.
        Task { [weak self] in
            while true {
                try? await Task.sleep(for: .seconds(3600))
                await self?.refresh()
            }
        }
    }

    /// The plans' prices; one left out is asked for again a few times.
    private func loadPrices() async {
        for attempt in 0..<Self.priceAttempts {
            if attempt > 0 { try? await Task.sleep(for: Self.priceAgain) }
            do {
                let products = try await Product.products(for: Plan.ids)
                for product in products {
                    if let plan = Plan.allCases.first(where: { $0.id == product.id }) {
                        prices[plan] = product.displayPrice
                    }
                }
                if !products.isEmpty { sold = true } else if prices.isEmpty { sold = false }
            } catch {
                // Asked again below, and when a plan is pressed.
            }
            if prices.count == Plan.allCases.count { return }
        }
    }

    /// Whether Pro is held, asked of the App Store again: when packwing comes
    /// back to the screen, and now and then.
    func refresh() async {
        guard !fake else { return }
        var held = false
        for await result in Transaction.currentEntitlements {
            if case .verified(let transaction) = result, Plan.ids.contains(transaction.productID),
               transaction.revocationDate == nil {
                held = true
            }
        }
        holds = held
    }

    func buy(_ plan: Plan) {
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
                guard let product = try await Product.products(for: [plan.id]).first else {
                    if prices.isEmpty { sold = false }
                    return
                }
                prices[plan] = product.displayPrice
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

    /// Restore Purchases: a subscription of the user's Apple Account (it may
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

    /// The fake store's Manage Subscription: the subscription ends.
    var endsWhenManaged: Bool { fake }

    func endFake() {
        guard fake else { return }
        holds = false
        UserDefaults.standard.set(false, forKey: "fakePro")
    }

    /// A plan's button: its price for its period, or its name until the App Store said it.
    func label(_ plan: Plan) -> String {
        switch (plan, prices[plan]) {
        case (.monthly, let price?): tr("{price} a month", ["price": price])
        case (.monthly, nil): tr("Monthly")
        case (.yearly, let price?): tr("{price} a year", ["price": price])
        case (.yearly, nil): tr("Yearly")
        }
    }
}

/// The App Store's standard license agreement, which packwing's subscriptions
/// are sold under, and packwing's privacy policy: the App Store asks for both
/// where they are sold.
private let terms = URL(string: "https://www.apple.com/legal/internet-services/itunes/dev/stdeula/")!
private let privacy = URL(string: "https://packwing.app/privacy")!

/// Settings' License: what the plan gives, and Pro subscribed to, or the plans
/// to subscribe to with what the App Store asks to be said beside them.
struct LicenseSection: View {
    @Environment(AppModel.self) private var model
    @State private var managing = false

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
                Label(tr("Subscribed"), systemImage: "checkmark.seal.fill")
                    .foregroundStyle(Tonal.accent)
                    .accessibilityIdentifier("store-held")
                Button(tr("Manage Subscription")) {
                    if store.endsWhenManaged { store.endFake() } else { managing = true }
                }
                .accessibilityIdentifier("store-manage")
            } else {
                if store.sold != false {
                    plans(store)
                }
                Button(tr("Restore Purchases")) { store.restore() }
                    .disabled(store.busy)
                    .accessibilityIdentifier("store-restore")
                if store.sold == false {
                    Text(tr("The App Store does not sell Pro in your country or region. A subscription of this Apple Account from another one can still be restored."))
                        .font(.nunito(14, relativeTo: .subheadline))
                        .foregroundStyle(Tonal.warnText)
                        .accessibilityIdentifier("store-not-sold")
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
            Text(tr("Pro by subscription, on every Mac, iPhone and iPad with your Apple Account."))
        }
        .listRowBackground(Tonal.surface)
        .manageSubscriptionsSheet(isPresented: $managing)
        // Ended, or renewed, while it was shown.
        .onChange(of: managing) { _, shown in
            if !shown { Task { await store.refresh() } }
        }
    }

    /// A button for each plan, the year's first, and how a subscription renews.
    private func plans(_ store: Store) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            Button { store.buy(.yearly) } label: {
                Text(store.label(.yearly)).frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent)
            .accessibilityIdentifier("store-buy-yearly")
            Button { store.buy(.monthly) } label: {
                Text(store.label(.monthly)).frame(maxWidth: .infinity)
            }
            .buttonStyle(.bordered)
            .accessibilityIdentifier("store-buy-monthly")
            Text(tr("Payment is charged to your Apple Account. A subscription renews automatically for the same period and price unless you cancel it at least 24 hours before it ends; manage or cancel it in your App Store account settings."))
                .font(.nunito(13, relativeTo: .footnote))
                .foregroundStyle(Tonal.muted)
            HStack(spacing: 16) {
                Link(tr("Terms of Use"), destination: terms)
                    .accessibilityIdentifier("store-terms")
                Link(tr("Privacy Policy"), destination: privacy)
                    .accessibilityIdentifier("store-privacy")
            }
            .font(.nunito(14, .semibold, relativeTo: .subheadline))
            // Each its own target in a row of the list.
            .buttonStyle(.borderless)
        }
        .disabled(store.busy)
        .padding(.vertical, 4)
    }

    private func words(_ said: Store.Said) -> String {
        switch said {
        case .pending: tr("Waiting for approval: packwing turns Pro once it is given.")
        case .nothingToRestore: tr("The App Store found no subscription of this Apple Account.")
        case .failed(let why): tr("The App Store could not do it: {why}", ["why": why])
        }
    }
}
