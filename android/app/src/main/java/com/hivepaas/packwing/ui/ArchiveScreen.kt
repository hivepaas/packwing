package com.hivepaas.packwing.ui

import android.graphics.BitmapFactory
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.combinedClickable
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
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.Check
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.Description
import androidx.compose.material.icons.outlined.Download
import androidx.compose.material.icons.outlined.DriveFileRenameOutline
import androidx.compose.material.icons.outlined.Folder
import androidx.compose.material.icons.outlined.Draw
import androidx.compose.material.icons.outlined.GridView
import androidx.compose.material.icons.outlined.Image
import androidx.compose.material.icons.outlined.MoreVert
import androidx.compose.material.icons.outlined.SelectAll
import androidx.compose.material.icons.outlined.Tag
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.Search
import androidx.compose.material.icons.automirrored.outlined.ViewList
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.BasicAlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Documents
import com.hivepaas.packwing.Picked
import com.hivepaas.packwing.Screen
import com.hivepaas.packwing.Sheet
import com.hivepaas.packwing.tr
import com.hivepaas.packwing.trN
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.packwing_mobile.EntryRow
import uniffi.packwing_mobile.OnExisting
import java.io.File
import uniffi.packwing_mobile.pictureFits

/** What is in the archive, folder by folder, and what a search found. */
@Composable
fun ArchiveScreen(model: AppModel, screen: Screen.Folder) {
    val c = tonal()
    val opened = model.opened ?: return
    var preview by remember { mutableStateOf<List<String>?>(null) }
    var grid by remember(screen.path) { mutableStateOf(screen.photos) }

    val rows = if (model.query.isBlank()) model.rows else model.found.map { it.entry }
    val paths: (EntryRow) -> List<String> = { row ->
        if (model.query.isBlank()) {
            screen.path + row.name
        } else {
            model.found.first { it.entry.name == row.name }.path + row.name
        }
    }
    val shown = rows.map { paths(it) to it }
    // What is asked before the archive is changed.
    var deleting by remember { mutableStateOf<List<List<String>>?>(null) }
    var renaming by remember { mutableStateOf<List<String>?>(null) }
    var clashing by remember { mutableStateOf<List<File>?>(null) }
    val context = LocalContext.current
    val work = rememberCoroutineScope()
    // Adds what came, or asks first what of those whose names are there.
    val take: (List<File>) -> Unit = { files ->
        val names = model.rows.map { it.name }.toSet()
        when {
            files.isEmpty() -> Unit
            files.any { it.name in names } -> clashing = files
            else -> model.add(files, screen.path, OnExisting.STOP)
        }
    }
    val copied: (suspend () -> List<File>) -> Unit = { copy ->
        work.launch { take(withContext(Dispatchers.IO) { runCatching { copy() }.getOrDefault(emptyList()) }) }
    }
    val addFiles = rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris ->
        if (uris.isNotEmpty()) copied { Documents.copyIn(context, uris) }
    }
    val addFolder = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { tree ->
        if (tree != null) copied { listOf(Documents.copyTreeIn(context, tree)) }
    }
    val addPhotos = rememberLauncherForActivityResult(ActivityResultContracts.PickMultipleVisualMedia()) { uris ->
        if (uris.isNotEmpty()) copied { Documents.copyIn(context, uris) }
    }
    // A tap picks while picking; otherwise it opens a folder, or previews a file.
    val tap: (EntryRow, Boolean) -> Unit = { row, photos ->
        when {
            model.selecting -> model.pick(paths(row), row)
            row.folder -> model.go(Screen.Folder(screen.path + row.name, photos = photos))
            !row.link -> preview = paths(row)
        }
    }

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
                if (model.selecting) {
                    IconButton(onClick = { model.stopPicking() }, Modifier.testTag("picking-done")) {
                        Icon(Icons.Outlined.Close, tr("Done"), tint = c.text)
                    }
                    Text(
                        tr("{picked} selected", "picked" to model.picked.size.toString()),
                        modifier = Modifier.weight(1f),
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Bold,
                        color = c.text,
                    )
                    IconButton(onClick = { model.pickAll(shown) }, Modifier.testTag("pick-all")) {
                        Icon(Icons.Outlined.SelectAll, tr("Select all"), tint = c.text)
                    }
                    if (opened.canChange && model.picked.isNotEmpty()) {
                        IconButton(onClick = { deleting = model.picked.keys.toList() }, Modifier.testTag("delete-picked")) {
                            Icon(Icons.Outlined.Delete, tr("Delete"), tint = c.text)
                        }
                    }
                    ArchiveMenu(model)
                    return@Row
                }
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
                if (opened.canChange) {
                    AddMenu(
                        files = { addFiles.launch(arrayOf("*/*")) },
                        folder = { addFolder.launch(null) },
                        photos = {
                            addPhotos.launch(PickVisualMediaRequest(ActivityResultContracts.PickVisualMedia.ImageAndVideo))
                        },
                    )
                }
                ArchiveMenu(model)
            }

            model.changing?.let { title -> Busy(title, opened.info.name) { model.stopChanging() } }

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

            if (grid) {
                LazyVerticalGrid(
                    columns = GridCells.Fixed(3),
                    modifier = Modifier.weight(1f),
                    contentPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 4.dp, bottom = 96.dp),
                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                    verticalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    items(rows, key = { it.name }) { row ->
                        Tile(
                            model,
                            paths(row),
                            row,
                            picked = paths(row) in model.picked,
                            open = { tap(row, true) },
                            pick = { model.pick(paths(row), row) },
                        )
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
                        Column(Modifier.fillMaxWidth()) {
                            EntryLine(
                                row,
                                picking = model.selecting,
                                picked = paths(row) in model.picked,
                                open = { tap(row, false) },
                                pick = { model.pick(paths(row), row) },
                            ) {
                                EntryMenu(
                                    model,
                                    paths(row),
                                    row,
                                    preview = { preview = paths(row) },
                                    rename = { renaming = paths(row) },
                                    delete = { deleting = listOf(paths(row)) },
                                )
                            }
                            HorizontalDivider(color = c.field, thickness = 1.dp)
                        }
                    }
                }
            }
        }
    }

    preview?.let { path ->
        PreviewDialog(path.last(), { model.preview(path) }) { preview = null }
    }

    val signed = signedBeside(opened.uri)
    deleting?.let { picked ->
        DeleteDialog(picked, signed, close = { deleting = null }) {
            deleting = null
            model.delete(picked)
        }
    }
    renaming?.let { path ->
        NameDialog(
            tr("Rename {what}", "what" to path.last()),
            path.last(),
            tr("Rename"),
            note = if (signed) UNSIGNED else null,
            close = { renaming = null },
        ) { name ->
            renaming = null
            model.rename(path, name)
        }
    }
    clashing?.let { files ->
        val names = model.rows.map { it.name }.toSet()
        ClashDialog(
            files.map { it.name }.filter { it in names },
            close = {
                clashing = null
                work.launch(Dispatchers.IO) { files.forEach { it.parentFile?.deleteRecursively() } }
            },
        ) { onExisting ->
            clashing = null
            model.add(files, screen.path, onExisting)
        }
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

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun EntryLine(
    row: EntryRow,
    picking: Boolean,
    picked: Boolean,
    open: () -> Unit,
    pick: () -> Unit,
    menu: @Composable () -> Unit,
) {
    val c = tonal()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 64.dp)
            .background(if (picked) c.tonal else c.surface)
            .combinedClickable(onClick = open, onLongClick = pick)
            .padding(start = 16.dp, end = 4.dp, top = 8.dp, bottom = 8.dp)
            .testTag("entry-${row.name}"),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        if (picked) {
            Box(
                Modifier.size(40.dp).clip(RoundedCornerShape(12.dp)).background(c.accent),
                contentAlignment = Alignment.Center,
            ) {
                Icon(Icons.Outlined.Check, tr("Selected"), tint = c.onAccent)
            }
        } else {
            KindTile(row.name, FileKind.of(row.name, row.folder, row.link), size = 40.dp)
        }
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
        if (!picking) menu() else Box(Modifier.size(12.dp))
    }
}

/** What the archive's ⋮ offers: its checksums, of what is picked or asked about. */
@Composable
private fun ArchiveMenu(model: AppModel) {
    val c = tonal()
    var open by remember { mutableStateOf(false) }
    Box {
        IconButton(onClick = { open = true }, Modifier.testTag("archive-menu")) {
            Icon(Icons.Outlined.MoreVert, tr("More"), tint = c.text)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }, containerColor = c.surface) {
            DropdownMenuItem(
                text = { Text(tr("Checksums"), color = c.text) },
                leadingIcon = { Icon(Icons.Outlined.Tag, null, tint = c.muted) },
                onClick = {
                    open = false
                    model.go(Screen.Checksums(if (model.selecting) model.pickedNow() else Picked()))
                },
            )
            DropdownMenuItem(
                text = { Text(tr("Signature"), color = c.text) },
                leadingIcon = { Icon(Icons.Outlined.Draw, null, tint = c.muted) },
                onClick = {
                    open = false
                    model.sheet = Sheet.Signature
                },
            )
        }
    }
}

/** What a file or folder's ⋮ offers: to look at it, extract it, or read its checksums. */
@Composable
private fun EntryMenu(
    model: AppModel,
    path: List<String>,
    row: EntryRow,
    preview: () -> Unit,
    rename: () -> Unit,
    delete: () -> Unit,
) {
    val c = tonal()
    var open by remember { mutableStateOf(false) }
    val alone = { mapOf(path to row).let { with(AppModel) { it.toPicked() } } }
    Box {
        IconButton(onClick = { open = true }, Modifier.testTag("menu-${row.name}")) {
            Icon(Icons.Outlined.MoreVert, tr("More"), tint = c.muted)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }, containerColor = c.surface) {
            if (!row.folder && !row.link) {
                DropdownMenuItem(
                    text = { Text(tr("Preview"), color = c.text) },
                    leadingIcon = { Icon(Icons.Outlined.Visibility, null, tint = c.muted) },
                    onClick = {
                        open = false
                        preview()
                    },
                )
            }
            DropdownMenuItem(
                text = { Text(tr("Extract"), color = c.text) },
                leadingIcon = { Icon(Icons.Outlined.Download, null, tint = c.muted) },
                onClick = {
                    open = false
                    model.sheet = Sheet.Extract(alone())
                },
            )
            DropdownMenuItem(
                text = { Text(tr("Checksums"), color = c.text) },
                leadingIcon = { Icon(Icons.Outlined.Tag, null, tint = c.muted) },
                onClick = {
                    open = false
                    model.go(Screen.Checksums(alone(), atOnce = true))
                },
            )
            if (model.opened?.canChange == true) {
                DropdownMenuItem(
                    text = { Text(tr("Rename"), color = c.text) },
                    leadingIcon = { Icon(Icons.Outlined.DriveFileRenameOutline, null, tint = c.muted) },
                    onClick = {
                        open = false
                        rename()
                    },
                )
                DropdownMenuItem(
                    text = { Text(tr("Delete…"), color = c.badText) },
                    leadingIcon = { Icon(Icons.Outlined.Delete, null, tint = c.badText) },
                    onClick = {
                        open = false
                        delete()
                    },
                )
            }
        }
    }
}

/** How many pixels across a picture in the grid is, at the least. */
private const val TILE_PIXELS = 320

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun Tile(
    model: AppModel,
    path: List<String>,
    row: EntryRow,
    picked: Boolean,
    open: () -> Unit,
    pick: () -> Unit,
) {
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
            .combinedClickable(onClick = open, onLongClick = pick)
            .testTag("entry-${row.name}"),
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
        if (picked) {
            Box(
                Modifier
                    .align(Alignment.TopEnd)
                    .padding(6.dp)
                    .size(26.dp)
                    .clip(RoundedCornerShape(13.dp))
                    .background(c.accent),
                contentAlignment = Alignment.Center,
            ) {
                Icon(Icons.Outlined.Check, tr("Selected"), tint = c.onAccent, modifier = Modifier.size(18.dp))
            }
        }
    }
}

/** A picture or a text of the archive, shown without extracting it. */
@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
internal fun PreviewDialog(name: String, load: suspend () -> ByteArray?, close: () -> Unit) {
    val c = tonal()
    var bytes by remember(name) { mutableStateOf<ByteArray?>(null) }
    var missing by remember(name) { mutableStateOf(false) }
    LaunchedEffect(name) {
        val read = load()
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
                name,
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
                AppModel.isPicture(name) && pictureFits(read) -> {
                    val bitmap = remember(read) { BitmapFactory.decodeByteArray(read, 0, read.size) }
                    if (bitmap != null) {
                        androidx.compose.foundation.Image(
                            bitmap.asImageBitmap(),
                            name,
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

/** Extract all of the archive, or what is picked; nothing while nothing is. */
@Composable
fun ExtractButton(model: AppModel) {
    val c = tonal()
    if (model.selecting && model.picked.isEmpty()) return
    ExtendedFloatingActionButton(
        onClick = { model.sheet = Sheet.Extract(if (model.selecting) model.pickedNow() else Picked()) },
        modifier = Modifier.testTag("extract"),
        containerColor = c.accent,
        contentColor = c.onAccent,
        shape = RoundedCornerShape(20.dp),
        icon = { Icon(Icons.Outlined.Download, null) },
        text = {
            Text(
                if (model.selecting) {
                    trN(model.picked.size, "Extract {n}", "Extract {n}", "n" to model.picked.size.toString())
                } else {
                    tr("Extract all")
                },
                fontWeight = FontWeight.SemiBold,
            )
        },
    )
}

/** Files, a folder or photos to add to the archive: Android's own pickers. */
@Composable
internal fun AddMenu(files: () -> Unit, folder: () -> Unit, photos: () -> Unit) {
    val c = tonal()
    var open by remember { mutableStateOf(false) }
    Box {
        IconButton(onClick = { open = true }, Modifier.testTag("add-menu")) {
            Icon(Icons.Outlined.Add, tr("Add files"), tint = c.text)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }, containerColor = c.surface) {
            listOf(
                Triple(tr("Files"), Icons.Outlined.Description, files),
                Triple(tr("A folder"), Icons.Outlined.Folder, folder),
                Triple(tr("Photos"), Icons.Outlined.Image, photos),
            ).forEach { (label, icon, pick) ->
                DropdownMenuItem(
                    text = { Text(label, color = c.text) },
                    leadingIcon = { Icon(icon, null, tint = c.muted) },
                    onClick = {
                        open = false
                        pick()
                    },
                )
            }
        }
    }
}

/** What the archive is being changed by, and Stop. */
@Composable
internal fun Busy(title: String, name: String, stop: () -> Unit) {
    val c = tonal()
    Row(
        Modifier
            .padding(horizontal = 16.dp, vertical = 4.dp)
            .fillMaxWidth()
            .clip(RoundedCornerShape(20.dp))
            .background(c.tonal)
            .padding(start = 16.dp, end = 6.dp, top = 8.dp, bottom = 8.dp)
            .testTag("changing"),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        CircularProgressIndicator(Modifier.size(20.dp), color = c.onTonal, strokeWidth = 2.5.dp)
        Column(Modifier.weight(1f)) {
            Text(title, fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = c.onTonal)
            Text(name, fontSize = 13.sp, color = c.onTonal, maxLines = 1, overflow = TextOverflow.MiddleEllipsis)
        }
        TextButton(onClick = stop) { Text(tr("Stop"), color = c.onTonal, fontWeight = FontWeight.SemiBold) }
    }
}

private val GONE_FOR_GOOD get() = tr("It goes from the archive for good, folders with all they hold. This can’t be undone.")
private val UNSIGNED get() =
    tr("This archive is signed. After this change it will no longer match its signature, until it is signed again.")

@Composable
private fun DeleteDialog(picked: List<List<String>>, signed: Boolean, close: () -> Unit, delete: () -> Unit) {
    val c = tonal()
    val what = if (picked.size == 1) picked[0].last() else tr("{many} items", "many" to picked.size.toString())
    AlertDialog(
        onDismissRequest = close,
        containerColor = c.surface,
        title = { Text(tr("Delete {what}?", "what" to what), color = c.text) },
        text = { Text(if (signed) GONE_FOR_GOOD + "\n\n" + UNSIGNED else GONE_FOR_GOOD, color = c.muted) },
        confirmButton = {
            TextButton(onClick = delete, modifier = Modifier.testTag("delete-confirm")) {
                Text(tr("Delete"), color = c.badText, fontWeight = FontWeight.SemiBold)
            }
        },
        dismissButton = { TextButton(onClick = close) { Text(tr("Cancel"), color = c.accent) } },
    )
}

/**
 * A name to give: a file's or folder's new one (not the one it has), a new
 * folder's (`current` none); never one with a slash.
 */
@Composable
internal fun NameDialog(
    title: String,
    current: String?,
    confirm: String,
    note: String? = null,
    close: () -> Unit,
    named: (String) -> Unit,
) {
    val c = tonal()
    var name by remember { mutableStateOf(current ?: "") }
    val wanted = name.trim()
    val ready = wanted.isNotEmpty() && wanted != current && '/' !in wanted
    AlertDialog(
        onDismissRequest = close,
        containerColor = c.surface,
        title = { Text(title, color = c.text) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text(tr("Name")) },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth().testTag("new-name"),
                )
                note?.let { Text(it, fontSize = 13.sp, color = c.muted) }
            }
        },
        confirmButton = {
            TextButton(onClick = { named(wanted) }, enabled = ready, modifier = Modifier.testTag("name-confirm")) {
                Text(confirm, color = if (ready) c.accent else c.faint, fontWeight = FontWeight.SemiBold)
            }
        },
        dismissButton = { TextButton(onClick = close) { Text(tr("Cancel"), color = c.accent) } },
    )
}

/** Files added whose names the folder holds: replace them, or skip them. */
@Composable
private fun ClashDialog(names: List<String>, close: () -> Unit, add: (OnExisting) -> Unit) {
    val c = tonal()
    AlertDialog(
        onDismissRequest = close,
        containerColor = c.surface,
        title = { Text(tr("If a file of that name is there"), color = c.text) },
        text = { Text(names.joinToString(", "), color = c.muted) },
        confirmButton = {
            Row {
                TextButton(onClick = { add(OnExisting.SKIP) }) { Text(tr("Skip it"), color = c.accent) }
                TextButton(onClick = { add(OnExisting.REPLACE) }, modifier = Modifier.testTag("replace-it")) {
                    Text(tr("Replace it"), color = c.accent, fontWeight = FontWeight.SemiBold)
                }
            }
        },
        dismissButton = { TextButton(onClick = close) { Text(tr("Cancel"), color = c.accent) } },
    )
}

/**
 * Whether the archive has a signature beside it, which a change leaves
 * behind: only to be told where packwing can look beside it.
 */
private fun signedBeside(uri: Uri): Boolean =
    uri.scheme == "file" && uri.path?.let { File("$it.sig").exists() } == true
