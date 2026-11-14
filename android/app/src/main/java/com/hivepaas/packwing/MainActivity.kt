package com.hivepaas.packwing

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.core.content.IntentCompat
import com.hivepaas.packwing.ui.AppRoot
import com.hivepaas.packwing.ui.PackwingTheme

/**
 * The one window. An archive comes in three ways: the launcher, with nothing
 * open; a VIEW intent, from Files or a chat; and Share › packwing, from any
 * app. The window draws behind the system's bars (edge to edge), and the
 * system's back gesture goes back a screen.
 */
class MainActivity : ComponentActivity() {
    private val model: AppModel get() = PackwingApp.model()

    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        handed(intent)?.let(model::open)
        setContent {
            PackwingTheme {
                AppRoot(model)
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handed(intent)?.let(model::open)
    }

    /** The archive another app handed us, if it did. */
    private fun handed(intent: Intent): Uri? = when (intent.action) {
        Intent.ACTION_VIEW -> intent.data
        Intent.ACTION_SEND -> IntentCompat.getParcelableExtra(intent, Intent.EXTRA_STREAM, Uri::class.java)
        else -> null
    }
}
