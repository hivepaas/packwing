package com.hivepaas.packwing.ui

import android.Manifest
import android.os.Build
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Inventory2
import androidx.compose.material.icons.outlined.Tune
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Screen
import com.hivepaas.packwing.Sheet
import com.hivepaas.packwing.tr
import kotlinx.coroutines.flow.filterNotNull

/**
 * Everything on screen: the screen the user is on, the navigation bar on the
 * two that have it, and what is asked over them.
 */
@Composable
fun AppRoot(model: AppModel) {
    val screen = model.screen
    val snackbar = remember { SnackbarHostState() }

    // Android asks before an app may post a notification; the job's progress
    // is shown in one, so it is asked for the first time a job starts.
    val askNotifications = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestPermission(),
    ) { }
    LaunchedEffect(model.running != null) {
        if (model.running != null && Build.VERSION.SDK_INT >= 33) {
            askNotifications.launch(Manifest.permission.POST_NOTIFICATIONS)
        }
    }

    // Each is said once, and taken as said as it starts: the window may go
    // before it has been on screen its while, and the next one would say it
    // again, over what is there by then.
    LaunchedEffect(Unit) {
        snapshotFlow { model.failure }.filterNotNull().collect { said ->
            model.failure = null
            snackbar.showSnackbar(said, duration = SnackbarDuration.Long)
        }
    }
    LaunchedEffect(Unit) {
        snapshotFlow { model.note }.filterNotNull().collect { said ->
            model.note = null
            snackbar.showSnackbar(said, duration = SnackbarDuration.Short)
        }
    }

    BackHandler(enabled = model.stack.size > 1 || model.sheet != null || model.selecting) { model.back() }

    Scaffold(
        containerColor = tonal().page,
        snackbarHost = { SnackbarHost(snackbar) },
        bottomBar = {
            if (screen is Screen.Home || screen is Screen.Settings) {
                BottomBar(model, screen)
            }
        },
        // The Scaffold's, so that what is said in passing goes above it, not over it.
        floatingActionButton = {
            if (screen is Screen.Folder && model.opened != null) ExtractButton(model)
            if (screen is Screen.Home) NewArchiveButton(model)
        },
    ) { insets ->
        Box(Modifier.fillMaxSize().padding(bottom = insets.calculateBottomPadding())) {
            when (screen) {
                is Screen.Home -> HomeScreen(model)
                is Screen.Settings -> SettingsScreen(model)
                is Screen.Folder -> ArchiveScreen(model, screen)
                is Screen.Create -> CreateScreen(model)
                is Screen.Running -> RunningScreen(model)
                is Screen.Checksums -> ChecksumsScreen(model, screen)
                is Screen.Storage -> StorageScreen(model, screen)
                Screen.AddStorage -> AddStorageScreen(model)
            }
        }
    }

    when (val sheet = model.sheet) {
        is Sheet.Password -> PasswordSheet(model, sheet)
        is Sheet.Extract -> ExtractSheet(model, sheet)
        Sheet.Signature -> SignatureSheet(model)
        Sheet.AddKey -> AddKeySheet(model)
        null -> Unit
    }
}

@Composable
private fun BottomBar(model: AppModel, screen: Screen) {
    val c = tonal()
    NavigationBar(containerColor = c.chrome, tonalElevation = androidx.compose.ui.unit.Dp.Hairline) {
        val colors = NavigationBarItemDefaults.colors(
            selectedIconColor = c.onTonal,
            selectedTextColor = c.onTonal,
            indicatorColor = c.tonal,
            unselectedIconColor = c.muted,
            unselectedTextColor = c.muted,
        )
        NavigationBarItem(
            selected = screen is Screen.Home,
            onClick = { model.home() },
            icon = { Icon(Icons.Outlined.Inventory2, null) },
            label = { Text(tr("Archives"), style = MaterialTheme.typography.labelMedium) },
            colors = colors,
        )
        NavigationBarItem(
            selected = screen is Screen.Settings,
            onClick = { if (screen !is Screen.Settings) model.go(Screen.Settings) },
            icon = { Icon(Icons.Outlined.Tune, null) },
            label = { Text(tr("Settings"), style = MaterialTheme.typography.labelMedium) },
            colors = colors,
        )
    }
}

/** The ground a card sits on, as the design has it. */
internal val cardColor: Color
    @Composable get() = tonal().surface
