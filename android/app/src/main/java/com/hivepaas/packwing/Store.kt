package com.hivepaas.packwing

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.util.Base64
import android.util.Log
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.android.billingclient.api.AcknowledgePurchaseParams
import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.BillingClient.BillingResponseCode
import com.android.billingclient.api.BillingClient.ProductType
import com.android.billingclient.api.BillingClientStateListener
import com.android.billingclient.api.BillingFlowParams
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.PendingPurchasesParams
import com.android.billingclient.api.ProductDetails
import com.android.billingclient.api.Purchase
import com.android.billingclient.api.PurchasesUpdatedListener
import com.android.billingclient.api.QueryProductDetailsParams
import com.android.billingclient.api.QueryPurchasesParams
import com.android.billingclient.api.acknowledgePurchase
import com.android.billingclient.api.queryProductDetails
import com.android.billingclient.api.queryPurchasesAsync
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.security.KeyFactory
import java.security.Signature
import java.security.spec.X509EncodedKeySpec

/**
 * packwing Pro by subscription in Google Play, a month or a year: one
 * subscription, `pro`, of two base plans. Only a purchase Google Play signed
 * with packwing's licence key counts, and only while the subscription is
 * current; on an Android phone, Pro is Google Play's alone, apart from the
 * App Store's.
 */
abstract class Store {
    /** The plans: base plans of the one subscription. */
    enum class Plan(val id: String) {
        Monthly("monthly"),
        Yearly("yearly"),
    }

    /** What the last purchase or restore came to, when it is to be said. */
    sealed interface Said {
        /** Paid in a way that takes time (cash, a bank): Pro once it is through. */
        data object Pending : Said
        data object NothingToRestore : Said
        data class Failed(val why: String) : Said
    }

    /** Whether the user holds Pro: subscribed, renewed or in the grace period, and not refunded. */
    var holds by mutableStateOf(false)
        protected set

    /** The plans' prices, as Google Play shows them in the user's country. */
    var prices by mutableStateOf(emptyMap<Plan, String>())
        protected set

    /** Whether Google Play sells Pro here, on this phone; nothing until it said. */
    var sold by mutableStateOf<Boolean?>(null)
        protected set

    /** A purchase or a restore going on. */
    var busy by mutableStateOf(false)
        protected set

    var said by mutableStateOf<Said?>(null)
        protected set

    /** Whether Pro is held, asked of Google Play again: when packwing comes back to the screen. */
    abstract fun refresh()

    /** Google Play's own sheet, over `activity`, to subscribe to `plan`. */
    abstract fun buy(activity: Activity, plan: Plan)

    /** Restore Purchases: a subscription of the Google Account in use on this phone. */
    abstract fun restore()

    /** Manage Subscription: Google Play's page for packwing's subscription. */
    abstract fun manage(context: Context)

    /** A plan's button: its price for its period, or its name until Google Play said it. */
    fun label(plan: Plan): String {
        val price = prices[plan]
        return when (plan) {
            Plan.Monthly -> if (price != null) tr("{price} a month", "price" to price) else tr("Monthly")
            Plan.Yearly -> if (price != null) tr("{price} a year", "price" to price) else tr("Yearly")
        }
    }

    companion object {
        /** The subscription's product id, in the Play Console. */
        const val PRODUCT = "pro"

        /** How many storages packwing Free keeps. */
        const val FREE_STORAGES = 1
    }
}

/**
 * The store of Google Play, through the Play Billing Library. A purchase is
 * checked against the Play Console's licence key, which a build is given as
 * `PACKWING_PLAY_LICENSE_KEY`; a build without it holds Pro by no purchase.
 */
class PlayStore(private val context: Context, private val licenceKey: String) : Store(), PurchasesUpdatedListener {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private val client = BillingClient.newBuilder(context)
        .setListener(this)
        .enablePendingPurchases(PendingPurchasesParams.newBuilder().enableOneTimeProducts().build())
        .enableAutoServiceReconnection()
        .build()

    /** The subscription as Google Play describes it, once it has: what a purchase is made of. */
    private var product: ProductDetails? = null

    init {
        client.startConnection(object : BillingClientStateListener {
            override fun onBillingSetupFinished(result: BillingResult) {
                scope.launch {
                    when (result.responseCode) {
                        BillingResponseCode.OK -> {
                            refresh()
                            loadPrices()
                        }
                        // No Google Play on this phone, or none that bills.
                        BillingResponseCode.BILLING_UNAVAILABLE, BillingResponseCode.FEATURE_NOT_SUPPORTED -> sold = false
                        else -> Log.w(TAG, "billing: ${result.responseCode} ${result.debugMessage}")
                    }
                }
            }

            // Connected again by the library itself (enableAutoServiceReconnection).
            override fun onBillingServiceDisconnected() {}
        })
    }

    /** The plans' prices; asked again a few times while one is left out. */
    private suspend fun loadPrices() {
        repeat(PRICE_ATTEMPTS) { attempt ->
            if (attempt > 0) delay(PRICE_AGAIN_MS)
            if (details() != null && prices.size == Plan.entries.size) return
        }
    }

    /** The subscription's details, asked of Google Play if they are not yet; its prices with them. */
    private suspend fun details(): ProductDetails? {
        product?.let { return it }
        val params = QueryProductDetailsParams.newBuilder()
            .setProductList(
                listOf(
                    QueryProductDetailsParams.Product.newBuilder()
                        .setProductId(PRODUCT)
                        .setProductType(ProductType.SUBS)
                        .build(),
                ),
            )
            .build()
        val answer = client.queryProductDetails(params)
        if (answer.billingResult.responseCode != BillingResponseCode.OK) {
            if (answer.billingResult.responseCode == BillingResponseCode.BILLING_UNAVAILABLE) sold = false
            return null
        }
        val found = answer.productDetailsList?.firstOrNull { it.productId == PRODUCT }
        if (found == null) {
            // Not sold in the user's country, or not on sale yet.
            if (prices.isEmpty()) sold = false
            return null
        }
        val got = Plan.entries.mapNotNull { plan ->
            offer(found, plan)?.pricingPhases?.pricingPhaseList?.lastOrNull()?.formattedPrice?.let { plan to it }
        }.toMap()
        prices = got
        sold = true
        // Kept once every plan is in it; until then, asked again.
        if (got.size == Plan.entries.size) product = found
        return found
    }

    /** A plan's base offer: its price, with no free trial or introductory offer the Console may add. */
    private fun offer(details: ProductDetails, plan: Plan): ProductDetails.SubscriptionOfferDetails? =
        details.subscriptionOfferDetails?.firstOrNull { it.basePlanId == plan.id && it.offerId == null }

    override fun refresh() {
        scope.launch { refreshNow() }
    }

    private suspend fun refreshNow() {
        if (!client.isReady) return
        val answer = client.queryPurchasesAsync(QueryPurchasesParams.newBuilder().setProductType(ProductType.SUBS).build())
        if (answer.billingResult.responseCode != BillingResponseCode.OK) return
        holds = answer.purchasesList.any { take(it) }
    }

    /**
     * Whether `purchase` is Pro held: Pro's, paid, and signed by Google Play;
     * acknowledged if it is not yet, as Google Play refunds one that is not
     * within three days.
     */
    private suspend fun take(purchase: Purchase): Boolean {
        if (PRODUCT !in purchase.products) return false
        if (purchase.purchaseState != Purchase.PurchaseState.PURCHASED) return false
        if (!signedByPlay(purchase)) {
            Log.w(TAG, "a purchase not signed with packwing's licence key")
            return false
        }
        if (!purchase.isAcknowledged) {
            val params = AcknowledgePurchaseParams.newBuilder().setPurchaseToken(purchase.purchaseToken).build()
            val result = client.acknowledgePurchase(params)
            if (result.responseCode != BillingResponseCode.OK) Log.w(TAG, "acknowledge: ${result.debugMessage}")
        }
        return true
    }

    /** Google Play's signature of the purchase, by the key the Play Console gives packwing. */
    private fun signedByPlay(purchase: Purchase): Boolean {
        if (licenceKey.isBlank()) return false
        return runCatching {
            val key = KeyFactory.getInstance("RSA").generatePublic(X509EncodedKeySpec(Base64.decode(licenceKey, Base64.DEFAULT)))
            Signature.getInstance("SHA1withRSA").run {
                initVerify(key)
                update(purchase.originalJson.toByteArray())
                verify(Base64.decode(purchase.signature, Base64.DEFAULT))
            }
        }.getOrDefault(false)
    }

    override fun buy(activity: Activity, plan: Plan) {
        if (busy) return
        said = null
        busy = true
        scope.launch {
            val details = details()
            val token = details?.let { offer(it, plan) }?.offerToken
            if (details == null || token == null) {
                busy = false
                return@launch
            }
            val params = BillingFlowParams.newBuilder()
                .setProductDetailsParamsList(
                    listOf(
                        BillingFlowParams.ProductDetailsParams.newBuilder()
                            .setProductDetails(details)
                            .setOfferToken(token)
                            .build(),
                    ),
                )
                .build()
            val result = client.launchBillingFlow(activity, params)
            // What came of it is onPurchasesUpdated's, once the sheet is through.
            if (result.responseCode != BillingResponseCode.OK) {
                busy = false
                answered(result)
            }
        }
    }

    override fun onPurchasesUpdated(result: BillingResult, purchases: MutableList<Purchase>?) {
        scope.launch {
            busy = false
            when (result.responseCode) {
                BillingResponseCode.OK -> {
                    val all = purchases.orEmpty().filter { PRODUCT in it.products }
                    if (all.any { take(it) }) {
                        holds = true
                    } else if (all.any { it.purchaseState == Purchase.PurchaseState.PENDING }) {
                        said = Said.Pending
                    }
                }
                // Subscribed already, on this Google Account: held.
                BillingResponseCode.ITEM_ALREADY_OWNED -> refreshNow()
                else -> answered(result)
            }
        }
    }

    /** What a purchase that did not go through says; nothing when the user stopped it. */
    private fun answered(result: BillingResult) {
        if (result.responseCode == BillingResponseCode.USER_CANCELED) return
        said = Said.Failed(result.debugMessage.ifBlank { "${result.responseCode}" })
    }

    override fun restore() {
        if (busy) return
        said = null
        busy = true
        scope.launch {
            refreshNow()
            busy = false
            if (!holds) said = Said.NothingToRestore
        }
    }

    override fun manage(context: Context) {
        val page = Uri.parse("https://play.google.com/store/account/subscriptions")
            .buildUpon()
            .appendQueryParameter("sku", PRODUCT)
            .appendQueryParameter("package", context.packageName)
            .build()
        context.startActivity(Intent(Intent.ACTION_VIEW, page).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
    }

    private companion object {
        const val TAG = "packwing-store"
        const val PRICE_ATTEMPTS = 5
        const val PRICE_AGAIN_MS = 3000L
    }
}

/**
 * A store of no Google Play, for the UI tests: a purchase and Restore answer
 * at once, what was bought is kept in the settings, and Manage Subscription
 * ends it. `noMonthly` leaves the monthly price out, as a store may.
 */
class FakeStore(private val settings: Settings, noMonthly: Boolean = false) : Store() {
    init {
        holds = settings.fakePro
        prices = mapOf(Plan.Monthly to "$0.99", Plan.Yearly to "$8.99").filterKeys { !(noMonthly && it == Plan.Monthly) }
        sold = true
    }

    override fun refresh() {}

    override fun buy(activity: Activity, plan: Plan) {
        said = null
        holds = true
        settings.fakePro = true
    }

    override fun restore() {
        said = null
        holds = settings.fakePro
        if (!holds) said = Said.NothingToRestore
    }

    override fun manage(context: Context) {
        holds = false
        settings.fakePro = false
    }
}
