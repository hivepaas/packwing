package com.hivepaas.packwing

import android.content.Intent
import android.graphics.Color
import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.SystemBarStyle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.DisposableEffect
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
            val dark = when (model.appearance) {
                Appearance.Light -> false
                Appearance.Dark -> true
                Appearance.System -> isSystemInDarkTheme()
            }
            // The bars' icons as the app is drawn, not as the system is.
            DisposableEffect(dark) {
                enableEdgeToEdge(
                    statusBarStyle = SystemBarStyle.auto(Color.TRANSPARENT, Color.TRANSPARENT) { dark },
                    navigationBarStyle = SystemBarStyle.auto(LIGHT_SCRIM, DARK_SCRIM) { dark },
                )
                onDispose {}
            }
            PackwingTheme(darkTheme = dark) {
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

    private companion object {
        // What enableEdgeToEdge puts behind the buttons' bar, as it does by itself.
        val LIGHT_SCRIM = Color.argb(0xe6, 0xff, 0xff, 0xff)
        val DARK_SCRIM = Color.argb(0x80, 0x1b, 0x1b, 0x1b)
    }
}
