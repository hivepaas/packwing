package com.hivepaas.packwing.ui

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.Folder
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Documents
import com.hivepaas.packwing.Screen
import com.hivepaas.packwing.Settings
import com.hivepaas.packwing.tr
import com.hivepaas.packwing.trN
import java.util.concurrent.TimeUnit

/** The screen the app opens on: a way in, and the archives opened before. */
@Composable
fun HomeScreen(model: AppModel) {
    val c = tonal()
    val pick = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        uri?.let(model::open)
    }
    Box(Modifier.fillMaxSize()) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(
                top = WindowInsets.statusBars.asPaddingValues().calculateTopPadding() + 8.dp,
                bottom = 96.dp,
                start = 16.dp,
                end = 16.dp,
            ),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            item {
                Text(
                    tr("Archives"),
                    modifier = Modifier.padding(start = 4.dp, top = 8.dp, bottom = 2.dp),
                    fontSize = 30.sp,
                    fontWeight = FontWeight.Bold,
                    color = c.text,
                )
            }

            item {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(28.dp))
                        .background(c.tonal)
                        .clickable { pick.launch(Documents.archiveMimeTypes) }
                        .padding(18.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(16.dp),
                ) {
                    Box(
                        Modifier.size(54.dp).clip(RoundedCornerShape(18.dp)).background(c.accent),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(Icons.Outlined.Folder, null, tint = c.onAccent, modifier = Modifier.size(26.dp))
                    }
                    Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                        Text(
                            tr("Open an archive"),
                            fontSize = 19.sp,
                            fontWeight = FontWeight.Bold,
                            color = c.onTonal,
                        )
                        Text(
                            tr("From the Files app, a USB drive or Drive"),
                            fontSize = 14.sp,
                            color = c.onTonal,
                        )
                    }
                }
            }

            item {
                Text(
                    tr("A ZIP or RAR in a chat or an email? Tap Share, then packwing, to look inside without saving it."),
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(20.dp))
                        .background(c.warnBg)
                        .padding(horizontal = 18.dp, vertical = 14.dp),
                    fontSize = 14.sp,
                    lineHeight = 21.sp,
                    color = c.warnText,
                )
            }

            if (model.opening != null) {
                item {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(20.dp))
                            .background(c.surface)
                            .padding(16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(14.dp),
                    ) {
                        CircularProgressIndicator(Modifier.size(22.dp), color = c.accent, strokeWidth = 3.dp)
                        Text(
                            tr("Opening {name}", "name" to (model.opening ?: "")),
                            fontSize = 15.sp,
                            color = c.text,
                        )
                    }
                }
            }

            if (model.recent.isNotEmpty()) {
                item {
                    Text(
                        tr("Recent"),
                        modifier = Modifier.padding(start = 4.dp, top = 4.dp),
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Bold,
                        color = c.accent,
                    )
                }
                item {
                    Column(
                        Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(24.dp))
                            .background(c.surface),
                    ) {
                        model.recent.forEachIndexed { at, row ->
                            if (at > 0) {
                                HorizontalDivider(color = c.field, thickness = 1.dp)
                            }
                            RecentRow(row) { model.open(row.uri) }
                        }
                    }
                }
            } else {
                item {
                    Text(
                        tr("Nothing opened yet."),
                        modifier = Modifier.padding(start = 4.dp, top = 8.dp),
                        fontSize = 14.sp,
                        color = c.muted,
                    )
                }
            }
        }

        ExtendedFloatingActionButton(
            onClick = { model.go(Screen.Create) },
            modifier = Modifier.align(Alignment.BottomEnd).padding(16.dp),
            containerColor = c.accent,
            contentColor = c.onAccent,
            shape = RoundedCornerShape(20.dp),
            icon = { Icon(Icons.Outlined.Add, null) },
            text = { Text(tr("New archive"), fontWeight = FontWeight.SemiBold) },
        )
    }
}

@Composable
private fun RecentRow(row: Settings.Recent, open: () -> Unit) {
    val c = tonal()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 68.dp)
            .clickable(onClick = open)
            .padding(horizontal = 16.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        KindTile(row.name, FileKind.of(row.name))
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(
                row.name,
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold,
                color = c.text,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                if (row.locked) {
                    Icon(Icons.Outlined.Lock, null, tint = c.muted, modifier = Modifier.size(13.dp))
                }
                Text(said(row), fontSize = 13.sp, color = c.muted, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
    }
}

/** What a recent row says under its name: what it holds, and when it was opened. */
@Composable
private fun said(row: Settings.Recent): String {
    val parts = buildList {
        if (row.locked) add(tr("Locked"))
        if (row.files > 0UL) {
            add(trN(row.files, "{n} file", "{n} files", "n" to row.files.toString()))
        }
        if (row.size > 0UL) add(sizeText(row.size))
        add(whenText(row.opened))
    }
    return parts.joinToString(" · ")
}

@Composable
private fun whenText(at: Long): String {
    val days = TimeUnit.MILLISECONDS.toDays(System.currentTimeMillis() - at)
    return when {
        days <= 0 -> tr("Today")
        days == 1L -> tr("Yesterday")
        days < 30 -> trN(days, "{n} day ago", "{n} days ago", "n" to days.toString())
        else -> trN(days / 30, "{n} month ago", "{n} months ago", "n" to (days / 30).toString())
    }
}
