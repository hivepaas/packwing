package com.hivepaas.packwing.ui

import android.graphics.BitmapFactory
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.outlined.GridView
import androidx.compose.material.icons.outlined.Image
import androidx.compose.material.icons.outlined.Search
import androidx.compose.material.icons.automirrored.outlined.ViewList
import androidx.compose.material3.BasicAlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Screen
import com.hivepaas.packwing.Sheet
import com.hivepaas.packwing.tr
import com.hivepaas.packwing.trN
import uniffi.packwing_mobile.EntryRow
import uniffi.packwing_mobile.pictureFits

/** What is in the archive, folder by folder, and what a search found. */
@Composable
fun ArchiveScreen(model: AppModel, screen: Screen.Folder) {
    val c = tonal()
    val opened = model.opened ?: return
    var preview by remember { mutableStateOf<List<String>?>(null) }
    var grid by remember(screen.path) { mutableStateOf(screen.photos) }

    Box(Modifier.fillMaxSize()) {
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
                Column(Modifier.weight(1f)) {
                    Text(
                        screen.path.lastOrNull() ?: opened.info.name,
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Bold,
                        color = c.text,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        listOf(
                            trN(
                                opened.info.files,
                                "{n} file",
                                "{n} files",
                                "n" to opened.info.files.toString(),
                            ),
                            sizeText(opened.info.packed),
                        ).joinToString(" · "),
                        fontSize = 13.sp,
                        color = c.muted,
                    )
                }
                IconButton(onClick = { grid = !grid }) {
                    Icon(
                        if (grid) Icons.AutoMirrored.Outlined.ViewList else Icons.Outlined.GridView,
                        if (grid) tr("Show a list") else tr("Show a grid"),
                        tint = c.text,
                    )
                }
            }

            SearchField(model.query, c) { model.search(it) }

            if (model.query.isBlank() && screen.path.isNotEmpty()) {
                Text(
                    (listOf(opened.info.name.substringBeforeLast('.')) + screen.path).joinToString("  ›  "),
                    modifier = Modifier.padding(horizontal = 24.dp, vertical = 6.dp),
                    fontSize = 13.sp,
                    color = c.muted,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }

            val rows = if (model.query.isBlank()) model.rows else model.found.map { it.entry }
            val paths: (EntryRow) -> List<String> = { row ->
                if (model.query.isBlank()) {
                    screen.path + row.name
                } else {
                    model.found.first { it.entry.name == row.name }.path + row.name
                }
            }

            if (grid) {
                LazyVerticalGrid(
                    columns = GridCells.Fixed(3),
                    modifier = Modifier.weight(1f),
                    contentPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 4.dp, bottom = 96.dp),
                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                    verticalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    items(rows, key = { it.name }) { row ->
                        Tile(model, paths(row), row) {
                            if (row.folder) model.go(Screen.Folder(screen.path + row.name, photos = true))
                            else preview = paths(row)
                        }
                    }
                }
            } else {
                LazyColumn(
                    modifier = Modifier.weight(1f),
                    contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 96.dp),
                ) {
                    item {
                        Box(Modifier.height(8.dp))
                    }
                    items(rows, key = { it.name }) { row ->
                        Column(
                            Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(0.dp))
                                .background(c.surface),
                        ) {
                            EntryLine(row) {
                                if (row.folder) model.go(Screen.Folder(screen.path + row.name))
                                else preview = paths(row)
                            }
                            HorizontalDivider(color = c.field, thickness = 1.dp)
                        }
                    }
                }
            }
        }

        ExtendedFloatingActionButton(
            onClick = { model.sheet = Sheet.Extract() },
            modifier = Modifier.align(Alignment.BottomEnd).padding(16.dp),
            containerColor = c.accent,
            contentColor = c.onAccent,
            shape = RoundedCornerShape(20.dp),
            icon = { Icon(Icons.Outlined.Image, null) },
            text = { Text(tr("Extract all"), fontWeight = FontWeight.SemiBold) },
        )
    }

    preview?.let { path ->
        PreviewDialog(model, path) { preview = null }
    }
}

@Composable
private fun SearchField(query: String, c: Tonal, onChange: (String) -> Unit) {
    Row(
        modifier = Modifier
            .padding(horizontal = 16.dp, vertical = 4.dp)
            .fillMaxWidth()
            .height(56.dp)
            .clip(RoundedCornerShape(28.dp))
            .background(c.field)
            .padding(horizontal = 18.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Icon(Icons.Outlined.Search, null, tint = c.muted, modifier = Modifier.size(20.dp))
        Box(Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
            if (query.isEmpty()) {
                Text(tr("Search in this archive"), fontSize = 16.sp, color = c.faint)
            }
            BasicTextField(
                value = query,
                onValueChange = onChange,
                singleLine = true,
                textStyle = TextStyle(fontSize = 16.sp, color = c.text),
                cursorBrush = androidx.compose.ui.graphics.SolidColor(c.accent),
                modifier = Modifier.fillMaxWidth(),
            )
        }
    }
}

@Composable
private fun EntryLine(row: EntryRow, open: () -> Unit) {
    val c = tonal()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 64.dp)
            .clickable(onClick = open)
            .padding(horizontal = 16.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        KindTile(row.name, FileKind.of(row.name, row.folder, row.link), size = 40.dp)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(1.dp)) {
            Text(
                row.name,
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold,
                color = c.text,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                if (row.folder) {
                    trN(row.files, "{n} file", "{n} files", "n" to row.files.toString())
                } else {
                    sizeText(row.size)
                },
                fontSize = 13.sp,
                color = c.muted,
            )
        }
    }
}

/** How many pixels across a picture in the grid is, at the least. */
private const val TILE_PIXELS = 320

@Composable
private fun Tile(model: AppModel, path: List<String>, row: EntryRow, open: () -> Unit) {
    val c = tonal()
    val kind = FileKind.of(row.name, row.folder, row.link)
    // The archive holds no thumbnail, so a picture is read whole to show a
    // tile of it: only one the grid can show, and only once (AppModel keeps
    // what it made).
    val picture = !row.folder && !row.link &&
        AppModel.isPicture(row.name) &&
        row.size <= AppModel.MOST_PICTURE
    var small by remember(path) { mutableStateOf<ImageBitmap?>(null) }
    if (picture) {
        LaunchedEffect(path) { small = model.picture(path, TILE_PIXELS) }
    }
    Box(
        modifier = Modifier
            .aspectRatio(1f)
            .clip(RoundedCornerShape(12.dp))
            .background(kind.tint(isDark()).ground)
            .clickable(onClick = open),
        contentAlignment = Alignment.Center,
    ) {
        val made = small
        if (made != null) {
            Image(
                bitmap = made,
                contentDescription = row.name,
                contentScale = ContentScale.Crop,
                modifier = Modifier.fillMaxSize(),
            )
        } else if (row.folder) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                KindTile(row.name, kind, size = 34.dp)
                Text(
                    row.name,
                    fontSize = 11.sp,
                    color = c.text,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(horizontal = 6.dp, vertical = 4.dp),
                )
            }
        } else {
            Text(
                kind.tag(row.name),
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold,
                color = kind.tint(isDark()).ink,
            )
        }
    }
}

/** A picture or a text of the archive, shown without extracting it. */
@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
private fun PreviewDialog(model: AppModel, path: List<String>, close: () -> Unit) {
    val c = tonal()
    var bytes by remember(path) { mutableStateOf<ByteArray?>(null) }
    var missing by remember(path) { mutableStateOf(false) }
    LaunchedEffect(path) {
        val read = model.preview(path)
        if (read == null) missing = true else bytes = read
    }
    BasicAlertDialog(onDismissRequest = close) {
        Column(
            Modifier
                .clip(RoundedCornerShape(28.dp))
                .background(c.surface)
                .padding(20.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(
                path.last(),
                fontSize = 17.sp,
                fontWeight = FontWeight.Bold,
                color = c.text,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
            val read = bytes
            when {
                missing -> Text(tr("packwing cannot show this file."), fontSize = 14.sp, color = c.muted)
                read == null -> Box(Modifier.fillMaxWidth().height(120.dp), Alignment.Center) {
                    CircularProgressIndicator(color = c.accent)
                }
                AppModel.isPicture(path.last()) && pictureFits(read) -> {
                    val bitmap = remember(read) { BitmapFactory.decodeByteArray(read, 0, read.size) }
                    if (bitmap != null) {
                        androidx.compose.foundation.Image(
                            bitmap.asImageBitmap(),
                            path.last(),
                            modifier = Modifier.fillMaxWidth().heightIn(max = 420.dp),
                            contentScale = ContentScale.Fit,
                        )
                    } else {
                        Text(tr("packwing cannot show this file."), fontSize = 14.sp, color = c.muted)
                    }
                }
                else -> Text(
                    read.decodeToString().take(20_000),
                    modifier = Modifier.heightIn(max = 420.dp).verticalScroll(rememberScrollState()),
                    fontSize = 13.sp,
                    fontFamily = FontFamily.Monospace,
                    color = c.text,
                )
            }
            TextButton(onClick = close, modifier = Modifier.align(Alignment.End)) {
                Text(tr("Close"), color = c.accent, fontWeight = FontWeight.SemiBold)
            }
        }
    }
}
