package com.hivepaas.packwing.ui

import android.content.Intent
import android.net.Uri
import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Verified
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Store
import com.hivepaas.packwing.tr

/** Google Play's terms, which a subscription in it is sold under, and packwing's privacy policy. */
private const val TERMS = "https://play.google.com/about/play-terms/"
private const val PRIVACY = "https://packwing.app/privacy"

/** A text button's words in line with the text above them, not set in by its padding. */
private val FLUSH = PaddingValues(horizontal = 0.dp)

/**
 * Settings › License: what the plan gives, and Pro subscribed to, or the
 * plans to subscribe to, with what renews and how to stop it beside them.
 */
@Composable
internal fun License(model: AppModel) {
    val c = tonal()
    val context = LocalContext.current
    val activity = LocalActivity.current
    val store = model.store
    Column(
        Modifier.fillMaxWidth().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(
                if (store.holds) "packwing Pro" else "packwing Free",
                Modifier.testTag("store-plan"),
                fontSize = 17.sp,
                fontWeight = FontWeight.Bold,
                color = c.text,
            )
            Text(
                if (store.holds) {
                    tr("Every feature, and any number of storages.")
                } else {
                    tr("Every feature but signing files, and one storage across a network.")
                },
                fontSize = 14.sp,
                lineHeight = 19.sp,
                color = c.muted,
            )
        }
        if (store.holds) {
            Row(
                Modifier.testTag("store-held"),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Icon(Icons.Outlined.Verified, null, tint = c.accent)
                Text(tr("Subscribed"), fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = c.accent)
            }
            TextButton(onClick = { store.manage(context) }, contentPadding = FLUSH, modifier = Modifier.testTag("store-manage")) {
                Text(tr("Manage Subscription"), color = c.accent, fontWeight = FontWeight.SemiBold)
            }
        } else {
            if (store.sold != false) Plans(store) { plan -> activity?.let { store.buy(it, plan) } }
            TextButton(
                onClick = { store.restore() },
                enabled = !store.busy,
                contentPadding = FLUSH,
                modifier = Modifier.testTag("store-restore"),
            ) {
                Text(tr("Restore Purchases"), color = if (store.busy) c.faint else c.accent, fontWeight = FontWeight.SemiBold)
            }
            if (store.sold == false) {
                Text(
                    tr("Google Play does not sell Pro in your country or region, or not on this device. A subscription of this Google Account can still be restored."),
                    Modifier.testTag("store-not-sold"),
                    fontSize = 14.sp,
                    lineHeight = 19.sp,
                    color = c.warnText,
                )
            }
        }
        store.said?.let { said ->
            Text(
                words(said),
                Modifier.testTag("store-said"),
                fontSize = 14.sp,
                lineHeight = 19.sp,
                color = if (said == Store.Said.Pending) c.onTonal else c.warnText,
            )
        }
    }
}

/** A button for each plan, the year's first, and how a subscription renews. */
@Composable
private fun Plans(store: Store, buy: (Store.Plan) -> Unit) {
    val c = tonal()
    val context = LocalContext.current
    fun open(link: String) = context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(link)))
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Button(
            onClick = { buy(Store.Plan.Yearly) },
            enabled = !store.busy,
            shape = RoundedCornerShape(26.dp),
            colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
            modifier = Modifier.fillMaxWidth().height(52.dp).testTag("store-buy-yearly"),
        ) {
            Text(store.label(Store.Plan.Yearly), fontSize = 16.sp, fontWeight = FontWeight.Bold)
        }
        OutlinedButton(
            onClick = { buy(Store.Plan.Monthly) },
            enabled = !store.busy,
            shape = RoundedCornerShape(26.dp),
            modifier = Modifier.fillMaxWidth().height(52.dp).testTag("store-buy-monthly"),
        ) {
            Text(store.label(Store.Plan.Monthly), fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = c.accent)
        }
        Text(
            tr("Payment is charged to your Google Account. A subscription renews automatically for the same period and price until you cancel it; manage or cancel it in your Google Play subscriptions."),
            fontSize = 13.sp,
            lineHeight = 18.sp,
            color = c.muted,
        )
        Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
            TextButton(onClick = { open(TERMS) }, contentPadding = FLUSH, modifier = Modifier.testTag("store-terms")) {
                Text(tr("Terms of Use"), color = c.accent, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
            }
            TextButton(onClick = { open(PRIVACY) }, contentPadding = FLUSH, modifier = Modifier.testTag("store-privacy")) {
                Text(tr("Privacy Policy"), color = c.accent, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
            }
        }
    }
}

private fun words(said: Store.Said): String = when (said) {
    Store.Said.Pending -> tr("Waiting for the payment: packwing turns Pro once it is through.")
    Store.Said.NothingToRestore -> tr("Google Play found no subscription of this Google Account.")
    is Store.Said.Failed -> tr("Google Play could not do it: {why}", "why" to said.why)
}
