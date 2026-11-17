package com.hivepaas.packwing.ui

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Build
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.BuildConfig
import com.hivepaas.packwing.R
import com.hivepaas.packwing.tr
import kotlinx.coroutines.delay

private const val SOURCE = "https://github.com/hivepaas/packwing"
private const val LICENSE = "https://github.com/hivepaas/packwing/blob/main/LICENSE"
private const val ISSUES = "https://github.com/hivepaas/packwing/issues/new"
private const val RELEASES = "https://github.com/hivepaas/packwing/releases"

/** The year in the copyright line. */
private const val YEAR = "2026"

/** The system, as a problem report names it: `Android 16`. */
private val system: String get() = "Android ${Build.VERSION.RELEASE}"

/** The processor this build runs on, as the desktop names it: `arm64`, `x86_64`. */
private val arch: String
    get() = when (val abi = Build.SUPPORTED_ABIS.firstOrNull() ?: "") {
        "arm64-v8a" -> "arm64"
        "armeabi-v7a" -> "arm"
        else -> abi
    }

/** What a problem report needs to say about this build, as the desktop says it. */
internal val versionInfo: String get() = "packwing ${BuildConfig.VERSION_NAME} ($system $arch)"

/**
 * Settings › About, as the desktop and the iOS app have it: what packwing is
 * and which build this is, where its code is, and the other apps from HivePaaS.
 */
@Composable
fun AboutScreen(model: AppModel) {
    val c = tonal()
    val context = LocalContext.current
    var copied by remember { mutableStateOf(false) }
    LaunchedEffect(copied) {
        if (copied) {
            delay(2000)
            copied = false
        }
    }

    Column(
        Modifier
            .fillMaxSize()
            .padding(top = WindowInsets.statusBars.asPaddingValues().calculateTopPadding()),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth().height(64.dp).padding(horizontal = 4.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(onClick = { model.back() }) {
                Icon(Icons.AutoMirrored.Outlined.ArrowBack, tr("Back"), tint = c.text)
            }
            Text(tr("About"), fontSize = 18.sp, fontWeight = FontWeight.Bold, color = c.text)
        }
        Column(
            Modifier
                .weight(1f)
                .verticalScroll(rememberScrollState())
                .testTag("about-list")
                .padding(start = 16.dp, end = 16.dp, bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            Card {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                        PackwingLogo(72.dp)
                        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                            Text("packwing", fontSize = 28.sp, fontWeight = FontWeight.Bold, color = c.text)
                            Text(
                                tr(
                                    "Version {version} · {os} {arch}",
                                    "version" to BuildConfig.VERSION_NAME,
                                    "os" to system,
                                    "arch" to arch,
                                ),
                                fontSize = 14.sp,
                                color = c.muted,
                                modifier = Modifier.testTag("about-version"),
                            )
                        }
                    }
                    Text(
                        tr("A file archiver that puts every core to work and opens archives safely."),
                        fontSize = 15.sp,
                        lineHeight = 21.sp,
                        color = c.text,
                    )
                    OutlinedButton(
                        onClick = {
                            val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                            clipboard.setPrimaryClip(ClipData.newPlainText("version", versionInfo))
                            copied = true
                        },
                        shape = RoundedCornerShape(20.dp),
                        modifier = Modifier.testTag("about-copy-version"),
                    ) {
                        Icon(Icons.Outlined.ContentCopy, null, tint = c.accent, modifier = Modifier.size(18.dp))
                        Text(
                            if (copied) tr("Copied") else tr("Copy version info"),
                            Modifier.padding(start = 8.dp),
                            color = c.accent,
                            fontWeight = FontWeight.SemiBold,
                        )
                    }
                }
            }

            Card {
                LinkRow(
                    tr("Free software"),
                    tr("Under the GNU General Public License, version 3 or later"),
                    tr("View license"),
                    LICENSE,
                    "about-license",
                )
                HorizontalDivider(color = c.field)
                LinkRow(
                    tr("Made by HivePaaS"),
                    tr("On hivepack, our library for reading and writing archives"),
                    tr("Source code"),
                    SOURCE,
                    "about-source",
                )
                HorizontalDivider(color = c.field)
                LinkRow(tr("Something wrong?"), tr("Tell us, with the version above"), tr("Report a problem"), ISSUES, "about-issues")
                HorizontalDivider(color = c.field)
                LinkRow(tr("What changed"), tr("Every release, and what is new in it"), tr("Release notes"), RELEASES, "about-releases")
            }

            Text(
                tr("More from HivePaaS"),
                modifier = Modifier.padding(start = 6.dp, top = 4.dp),
                fontSize = 14.sp,
                fontWeight = FontWeight.Bold,
                color = c.accent,
            )
            Card {
                otherApps().forEachIndexed { at, app ->
                    if (at > 0) HorizontalDivider(color = c.field)
                    OtherAppCard(app)
                }
            }
            Text(
                tr("Free and open source, like packwing."),
                modifier = Modifier.padding(horizontal = 8.dp),
                fontSize = 13.sp,
                color = c.muted,
            )
            Text(
                tr("© {year} HivePaaS. The packwing name and logo are trademarks of HivePaaS.", "year" to YEAR),
                modifier = Modifier.padding(horizontal = 8.dp, vertical = 8.dp),
                fontSize = 12.sp,
                lineHeight = 17.sp,
                color = c.faint,
            )
        }
    }
}

@Composable
private fun Card(content: @Composable () -> Unit) = Column(
    Modifier
        .fillMaxWidth()
        .clip(RoundedCornerShape(20.dp))
        .background(tonal().surface),
) { content() }

/** A row that says what something is, and opens its page. */
@Composable
private fun LinkRow(title: String, detail: String, action: String, url: String, tag: String) {
    val c = tonal()
    val open = LocalUriHandler.current
    Row(
        Modifier
            .fillMaxWidth()
            .heightIn(min = 64.dp)
            .clickable { runCatching { open.openUri(url) } }
            .padding(horizontal = 16.dp, vertical = 10.dp)
            .testTag(tag),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = c.text)
            Text(detail, fontSize = 13.sp, lineHeight = 18.sp, color = c.muted)
        }
        Text(action, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = c.accent)
    }
}

/**
 * packwing's logo on its accent tile, as the desktop draws it: the zipper in
 * the tile's ink, and the feather in its own colour.
 */
@Composable
fun PackwingLogo(size: Dp) {
    val c = tonal()
    Box(
        Modifier.size(size).clip(RoundedCornerShape(size / 3)).background(c.accent),
        contentAlignment = Alignment.Center,
    ) {
        Icon(painterResource(R.drawable.logo_zipper), null, tint = c.onAccent, modifier = Modifier.size(size / 2))
        Icon(
            painterResource(R.drawable.logo_feather),
            null,
            tint = if (isDark()) rgb(0x55391f) else rgb(0xffe1cc),
            modifier = Modifier.size(size / 2),
        )
    }
}

/** An app from HivePaaS: its mark, name, what it is, its strong points, and where to see it. */
private class OtherApp(val id: String, val what: String, val points: List<String>, val site: Pair<String, String>, val source: String)

// Said again each time: in the language in use.
private fun otherApps() = listOf(
    OtherApp(
        "shellwing",
        tr("A fast, native SSH client for Linux, macOS and Windows."),
        listOf(tr("SSH and SFTP"), tr("Follows your ssh_config"), tr("A real terminal"), tr("GPL-3.0-or-later")),
        tr("Visit shellwing.app") to "https://shellwing.app",
        "https://github.com/hivepaas/shellwing",
    ),
    OtherApp(
        "tracewing",
        tr("A native HTTP(S) debugging proxy that keeps up with big sessions."),
        listOf(
            tr("HTTP/2, WebSocket and SSE"),
            tr("A million requests, smooth"),
            tr("Breakpoints and rewrites"),
            tr("Phones and emulators"),
            tr("GPL-3.0-or-later"),
        ),
        tr("Visit tracewing.app") to "https://tracewing.app",
        "https://github.com/hivepaas/tracewing",
    ),
    OtherApp(
        "HivePaaS",
        tr("A self-hosted platform for your apps, built on Docker Swarm: an open-source alternative to Heroku, Render and Coolify."),
        listOf(
            tr("300+ one-click apps"),
            tr("Deploy on push"),
            tr("Pull request previews"),
            tr("Automatic TLS"),
            tr("Live logs and metrics"),
            tr("Apache-2.0"),
        ),
        tr("Visit hivepaas.com") to "https://hivepaas.com",
        "https://github.com/hivepaas/hivepaas",
    ),
)

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun OtherAppCard(app: OtherApp) {
    val c = tonal()
    val open = LocalUriHandler.current
    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
            AppMark(app.id)
            Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text(app.id, fontSize = 18.sp, fontWeight = FontWeight.Bold, color = c.text)
                Text(app.what, fontSize = 13.sp, lineHeight = 18.sp, color = c.muted)
            }
        }
        FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            app.points.forEach { point ->
                Text(
                    point,
                    Modifier.clip(CircleShape).background(c.field).padding(horizontal = 12.dp, vertical = 6.dp),
                    fontSize = 13.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = c.text,
                )
            }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(
                onClick = { runCatching { open.openUri(app.site.second) } },
                shape = RoundedCornerShape(20.dp),
                colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
                modifier = Modifier.testTag("about-${app.id.lowercase()}"),
            ) { Text(app.site.first, fontSize = 14.sp, fontWeight = FontWeight.SemiBold) }
            OutlinedButton(
                onClick = { runCatching { open.openUri(app.source) } },
                shape = RoundedCornerShape(20.dp),
            ) { Text("GitHub", fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = c.accent) }
        }
    }
}

/** Each app's mark on its own tile, as the desktop draws them, in either look. */
@Composable
private fun AppMark(id: String) {
    val tile = Modifier.size(56.dp).clip(RoundedCornerShape(16.dp))
    when (id) {
        "shellwing" -> Box(tile.background(rgb(0xeef1f5)), contentAlignment = Alignment.Center) {
            Icon(painterResource(R.drawable.mark_shellwing), null, tint = rgb(0x0284c7), modifier = Modifier.size(36.dp))
        }
        "tracewing" -> Box(tile.background(rgb(0x2f55b4)), contentAlignment = Alignment.Center) {
            Icon(painterResource(R.drawable.mark_tracewing), null, tint = Color.White, modifier = Modifier.size(30.dp))
        }
        else -> Box(tile.background(rgb(0xfbf7f0)), contentAlignment = Alignment.Center) { HiveMark() }
    }
}

/** HivePaaS's mark: seven cells of a hive in its brown and amber. */
@Composable
private fun HiveMark() {
    val brown = rgb(0x543d2b)
    val amber = rgb(0xf59e0b)
    // Centers in the mark's 36-unit box, and whether each is brown.
    val cells = listOf(
        Triple(12.98f, 9.30f, true), Triple(23.02f, 9.30f, false),
        Triple(7.95f, 18.0f, true), Triple(18.0f, 18.0f, false), Triple(28.05f, 18.0f, false),
        Triple(12.98f, 26.70f, true), Triple(23.02f, 26.70f, false),
    )
    Canvas(Modifier.size(36.dp)) {
        val unit = size.width / 36f
        cells.forEach { (x, y, isBrown) ->
            drawCircle(if (isBrown) brown else amber, radius = 4.3f * unit, center = Offset(x * unit, y * unit))
        }
    }
}
