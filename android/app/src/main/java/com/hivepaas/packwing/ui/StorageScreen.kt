package com.hivepaas.packwing.ui

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.material.icons.automirrored.outlined.DriveFileMove
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material.icons.outlined.CreateNewFolder
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.Download
import androidx.compose.material.icons.outlined.DriveFileRenameOutline
import androidx.compose.material.icons.outlined.Folder
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.ui.platform.LocalContext
import com.hivepaas.packwing.Documents
import java.io.File
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.Inventory2
import androidx.compose.material.icons.outlined.MoreVert
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.WarningAmber
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CheckboxDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextFieldDefaults
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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Screen
import com.hivepaas.packwing.Settings
import com.hivepaas.packwing.said
import com.hivepaas.packwing.tr
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.packwing_mobile.Job
import uniffi.packwing_mobile.MobileException
import uniffi.packwing_mobile.RemoteStorage
import uniffi.packwing_mobile.StoredItem
import uniffi.packwing_mobile.describeStorage
import uniffi.packwing_mobile.storageKinds
import java.text.DateFormat
import java.util.Date
import java.util.UUID

/** The most of a file of a storage read to show it: what the archive's preview reads. */
private val MOST_PREVIEW: ULong = 32UL * 1024UL * 1024UL

/** What went wrong, as the core says it. */
private fun said(e: Throwable): String = (e as? MobileException)?.said ?: e.message ?: e.toString()

// MARK: The Archives tab's storages

/** The storages added, and Add storage: on the Archives tab. */
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun StoragesGroup(model: AppModel) {
    val c = tonal()
    var removing by remember { mutableStateOf<Settings.StorageEntry?>(null) }
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(24.dp)).background(c.surface)) {
        model.storages.forEach { entry ->
            val kind = storageKinds().firstOrNull { it.id == entry.kind }
            Row(
                Modifier
                    .fillMaxWidth()
                    .combinedClickable(
                        onClick = { model.go(Screen.Storage(entry.id)) },
                        onLongClick = { removing = entry },
                    )
                    .padding(start = 16.dp, end = 4.dp, top = 10.dp, bottom = 10.dp)
                    .testTag("storage-row"),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                val tint = FileKind.Image.tint(isDark())
                Box(
                    Modifier.size(42.dp).clip(RoundedCornerShape(12.dp)).background(tint.ground),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(kind?.tag ?: "", fontSize = 13.sp, fontWeight = FontWeight.Bold, color = tint.ink)
                }
                Column(Modifier.weight(1f)) {
                    Text(
                        describeStorage(entry.kind, entry.values),
                        fontSize = 16.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = c.text,
                        maxLines = 1,
                        overflow = TextOverflow.MiddleEllipsis,
                    )
                    Text(kind?.name ?: "", fontSize = 13.sp, color = c.muted)
                }
                IconButton(onClick = { removing = entry }) {
                    Icon(Icons.Outlined.MoreVert, tr("More"), tint = c.muted)
                }
            }
            HorizontalDivider(color = c.field)
        }
        Row(
            Modifier
                .fillMaxWidth()
                .heightIn(min = 56.dp)
                .clickable { model.go(Screen.AddStorage) }
                .padding(horizontal = 18.dp)
                .testTag("storage-new"),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Icon(Icons.Outlined.Add, null, tint = c.accent)
            Text(tr("Add storage"), fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = c.accent)
        }
    }

    removing?.let { entry ->
        AlertDialog(
            onDismissRequest = { removing = null },
            containerColor = c.surface,
            title = {
                Text(tr("Remove “{name}” from the list?", "name" to describeStorage(entry.kind, entry.values)), color = c.text)
            },
            text = {
                Text(
                    tr("Its files stay where they are. What packwing signs in to it with is taken out of the system’s keychain."),
                    color = c.muted,
                )
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        removing = null
                        model.removeStorage(entry)
                    },
                    modifier = Modifier.testTag("storage-remove"),
                ) {
                    Text(tr("Remove from this list"), color = c.badText, fontWeight = FontWeight.SemiBold)
                }
            },
            dismissButton = { TextButton(onClick = { removing = null }) { Text(tr("Cancel"), color = c.accent) } },
        )
    }
}

// MARK: A storage's folder

/** A folder of a storage: its files and folders, an archive opened where it is, a file previewed. */
@Composable
fun StorageScreen(model: AppModel, screen: Screen.Storage) {
    val c = tonal()
    val entry = model.storages.firstOrNull { it.id == screen.id }
    if (entry == null) {
        LaunchedEffect(Unit) { model.back() }
        return
    }
    var items by remember(screen) { mutableStateOf<List<StoredItem>?>(null) }
    var problem by remember(screen) { mutableStateOf<String?>(null) }
    var hidden by remember { mutableStateOf(model.settings.storageHidden) }
    var preview by remember { mutableStateOf<StoredItem?>(null) }
    var tries by remember { mutableStateOf(0) }
    // What is asked before the storage is changed.
    var renaming by remember { mutableStateOf<String?>(null) }
    var deleting by remember { mutableStateOf<List<String>?>(null) }
    var naming by remember { mutableStateOf(false) }
    var sending by remember { mutableStateOf<Pair<List<String>, Boolean>?>(null) }
    var downloading by remember { mutableStateOf<List<String>?>(null) }
    val context = LocalContext.current
    val work = rememberCoroutineScope()
    val up: (suspend () -> List<File>) -> Unit = { copy ->
        work.launch {
            val files = withContext(Dispatchers.IO) { runCatching { copy() }.getOrDefault(emptyList()) }
            model.upload(files, entry, screen.folder)
        }
    }
    val addFiles = rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris ->
        if (uris.isNotEmpty()) up { Documents.copyIn(context, uris) }
    }
    val addFolder = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { tree ->
        if (tree != null) up { listOf(Documents.copyTreeIn(context, tree)) }
    }
    val addPhotos = rememberLauncherForActivityResult(ActivityResultContracts.PickMultipleVisualMedia()) { uris ->
        if (uris.isNotEmpty()) up { Documents.copyIn(context, uris) }
    }
    val downloadInto = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { tree ->
        val names = downloading
        downloading = null
        if (tree != null && names != null) {
            Documents.keepAccess(context, tree, write = true)
            model.download(names, entry, screen.folder, tree)
        }
    }

    // Read again once something here, or anywhere, changed.
    LaunchedEffect(screen, tries, model.storagesChanged) {
        problem = null
        withContext(Dispatchers.IO) {
            runCatching { model.storage(entry).list(screen.folder, Job.quiet()) }
        }.onSuccess { items = it }.onFailure { problem = said(it) }
    }

    val shown = items.orEmpty().filter { hidden || !it.hidden }
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
            Text(
                screen.folder.lastOrNull() ?: describeStorage(entry.kind, entry.values),
                modifier = Modifier.weight(1f),
                fontSize = 18.sp,
                fontWeight = FontWeight.Bold,
                color = c.text,
                maxLines = 1,
                overflow = TextOverflow.MiddleEllipsis,
            )
            // Up from the phone: files, a folder, photos.
            AddMenu(
                files = { addFiles.launch(arrayOf("*/*")) },
                folder = { addFolder.launch(null) },
                photos = { addPhotos.launch(PickVisualMediaRequest(ActivityResultContracts.PickVisualMedia.ImageAndVideo)) },
            )
            var open by remember { mutableStateOf(false) }
            Box {
                IconButton(onClick = { open = true }, Modifier.testTag("storage-menu")) {
                    Icon(Icons.Outlined.MoreVert, tr("More"), tint = c.text)
                }
                DropdownMenu(expanded = open, onDismissRequest = { open = false }, containerColor = c.surface) {
                    DropdownMenuItem(
                        text = { Text(tr("New folder"), color = c.text) },
                        leadingIcon = { Icon(Icons.Outlined.CreateNewFolder, null, tint = c.muted) },
                        onClick = {
                            open = false
                            naming = true
                        },
                        modifier = Modifier.testTag("storage-new-folder"),
                    )
                    DropdownMenuItem(
                        text = { Text(tr("New archive"), color = c.text) },
                        leadingIcon = { Icon(Icons.Outlined.Inventory2, null, tint = c.muted) },
                        onClick = {
                            open = false
                            model.go(Screen.Create(on = screen))
                        },
                    )
                    DropdownMenuItem(
                        text = { Text(tr("Show hidden files"), color = c.text) },
                        leadingIcon = {
                            Checkbox(
                                checked = hidden,
                                onCheckedChange = null,
                                colors = CheckboxDefaults.colors(checkedColor = c.accent, uncheckedColor = c.outline),
                            )
                        },
                        onClick = {
                            hidden = !hidden
                            model.settings.storageHidden = hidden
                        },
                    )
                }
            }
        }

        model.changing?.let { title -> Busy(title, model.changingName ?: "") { model.stopChanging() } }

        when {
            problem != null -> Column(
                Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Row(
                    Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(16.dp))
                        .background(c.warnBg)
                        .padding(14.dp),
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    Icon(Icons.Outlined.WarningAmber, null, tint = c.warnText)
                    Text(problem!!, fontSize = 14.sp, color = c.warnText)
                }
                TextButton(onClick = { tries++ }) { Text(tr("Try again"), color = c.accent, fontWeight = FontWeight.SemiBold) }
            }
            items == null -> Row(
                Modifier.padding(24.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                CircularProgressIndicator(Modifier.size(22.dp), color = c.accent, strokeWidth = 3.dp)
                Text(tr("Checking…"), fontSize = 15.sp, color = c.text)
            }
            shown.isEmpty() -> Text(
                tr("This folder is empty."),
                modifier = Modifier.padding(24.dp),
                fontSize = 15.sp,
                color = c.muted,
            )
            else -> LazyColumn(
                modifier = Modifier.weight(1f).testTag("stored-list"),
                contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 24.dp),
            ) {
                items(shown, key = { it.name }) { item ->
                    val path = screen.folder + item.name
                    val kind = FileKind.of(item.name, item.folder, item.link)
                    StoredRow(
                        item,
                        kind,
                        open = {
                            when {
                                item.folder -> model.go(Screen.Storage(entry.id, path))
                                item.link -> Unit
                                kind == FileKind.Archive -> model.openStored(entry, path, item.size)
                                else -> preview = item
                            }
                        },
                        menu = {
                            StoredMenu(
                                item,
                                openIt = { model.openStored(entry, path, item.size) },
                                previewIt = { preview = item },
                                download = {
                                    downloading = listOf(item.name)
                                    downloadInto.launch(null)
                                },
                                send = { moving -> sending = listOf(item.name) to moving },
                                rename = { renaming = item.name },
                                delete = { deleting = listOf(item.name) },
                            )
                        },
                    )
                    HorizontalDivider(color = c.field)
                }
            }
        }
    }

    model.opening?.let { name ->
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.BottomCenter) {
            Row(
                Modifier
                    .padding(24.dp)
                    .clip(RoundedCornerShape(20.dp))
                    .background(c.tonal)
                    .padding(16.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                CircularProgressIndicator(Modifier.size(20.dp), color = c.onTonal, strokeWidth = 2.5.dp)
                Text(tr("Opening {name}", "name" to name), fontSize = 15.sp, color = c.onTonal)
            }
        }
    }

    renaming?.let { name ->
        NameDialog(tr("Rename {what}", "what" to name), name, tr("Rename"), close = { renaming = null }) { to ->
            renaming = null
            model.onStorage(tr("Renaming…"), name) { model.storage(entry).rename(screen.folder, name, to) }
        }
    }
    if (naming) {
        NameDialog(tr("New folder"), null, tr("Create"), close = { naming = false }) { name ->
            naming = false
            model.onStorage(tr("New folder"), name) { model.storage(entry).makeFolder(screen.folder, name) }
        }
    }
    deleting?.let { names ->
        val what = names.singleOrNull() ?: tr("{many} items", "many" to names.size.toString())
        AlertDialog(
            onDismissRequest = { deleting = null },
            containerColor = c.surface,
            title = { Text(tr("Delete {what}?", "what" to what), color = c.text) },
            text = { Text(tr("This cannot be undone from packwing."), color = c.muted) },
            confirmButton = {
                TextButton(
                    onClick = {
                        deleting = null
                        model.onStorage(tr("Deleting…"), what) { model.storage(entry).delete(screen.folder, names) }
                    },
                    modifier = Modifier.testTag("delete-confirm"),
                ) { Text(tr("Delete"), color = c.badText, fontWeight = FontWeight.SemiBold) }
            },
            dismissButton = { TextButton(onClick = { deleting = null }) { Text(tr("Cancel"), color = c.accent) } },
        )
    }
    sending?.let { (names, moving) ->
        StorageFolderPicker(
            model,
            title = if (moving) tr("Move to…") else tr("Copy to…"),
            confirm = if (moving) tr("Move here") else tr("Copy here"),
            close = { sending = null },
        ) { to ->
            sending = null
            model.transfer(names, entry, screen.folder, to, moving)
        }
    }

    preview?.let { item ->
        PreviewDialog(
            item.name,
            {
                withContext(Dispatchers.IO) {
                    runCatching { model.storage(entry).read(screen.folder + item.name, MOST_PREVIEW) }.getOrNull()
                }
            },
        ) { preview = null }
    }
}

@Composable
private fun StoredRow(item: StoredItem, kind: FileKind, open: () -> Unit, menu: (@Composable () -> Unit)?) {
    val c = tonal()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 64.dp)
            .background(c.surface)
            .clickable(onClick = open)
            .padding(start = 16.dp, end = 4.dp, top = 8.dp, bottom = 8.dp)
            .testTag("stored-${item.name}"),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        KindTile(item.name, kind, size = 40.dp)
        Column(Modifier.weight(1f)) {
            Text(
                item.name,
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold,
                color = c.text,
                maxLines = 1,
                overflow = TextOverflow.MiddleEllipsis,
            )
            val meta = listOfNotNull(
                if (item.folder) null else sizeText(item.size),
                item.modified?.let { DateFormat.getDateInstance(DateFormat.MEDIUM).format(Date(it * 1000)) },
            ).joinToString(" · ")
            if (meta.isNotEmpty()) Text(meta, fontSize = 13.sp, color = c.muted, maxLines = 1)
        }
        if (menu != null) menu() else Box(Modifier.size(12.dp))
    }
}

/**
 * What a file's or folder's ⋮ offers: to open a file as an archive, whatever
 * its name, or look at it; to download it, copy or move it, rename or delete it.
 */
@Composable
private fun StoredMenu(
    item: StoredItem,
    openIt: () -> Unit,
    previewIt: () -> Unit,
    download: () -> Unit,
    send: (moving: Boolean) -> Unit,
    rename: () -> Unit,
    delete: () -> Unit,
) {
    val c = tonal()
    var open by remember { mutableStateOf(false) }
    val entries = buildList {
        if (!item.folder && !item.link) {
            // packwing tells an archive by what is in it, whatever its name.
            add(Triple(tr("Open"), Icons.Outlined.Inventory2, openIt))
            add(Triple(tr("Preview"), Icons.Outlined.Visibility, previewIt))
        }
        add(Triple(tr("Download"), Icons.Outlined.Download, download))
        add(Triple(tr("Copy to…"), Icons.Outlined.ContentCopy) { send(false) })
        add(Triple(tr("Move to…"), Icons.AutoMirrored.Outlined.DriveFileMove) { send(true) })
        add(Triple(tr("Rename"), Icons.Outlined.DriveFileRenameOutline, rename))
    }
    Box {
        IconButton(onClick = { open = true }, Modifier.testTag("stored-menu-${item.name}")) {
            Icon(Icons.Outlined.MoreVert, tr("More"), tint = c.muted)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }, containerColor = c.surface) {
            entries.forEach { (label, icon, act) ->
                DropdownMenuItem(
                    text = { Text(label, color = c.text) },
                    leadingIcon = { Icon(icon, null, tint = c.muted) },
                    onClick = {
                        open = false
                        act()
                    },
                )
            }
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

/**
 * A folder of a storage to copy or move to: the storages added, then their
 * folders, and `confirm` for the one shown.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun StorageFolderPicker(
    model: AppModel,
    title: String,
    confirm: String,
    close: () -> Unit,
    chosen: (Screen.Storage) -> Unit,
) {
    val c = tonal()
    var at by remember { mutableStateOf<Screen.Storage?>(null) }
    var folders by remember(at) { mutableStateOf<List<StoredItem>?>(null) }
    var problem by remember(at) { mutableStateOf<String?>(null) }
    val here = at
    LaunchedEffect(here) {
        val entry = here?.let { spot -> model.storages.firstOrNull { it.id == spot.id } } ?: return@LaunchedEffect
        withContext(Dispatchers.IO) { runCatching { model.storage(entry).list(here.folder, Job.quiet()) } }
            .onSuccess { items -> folders = items.filter { it.folder && !it.hidden } }
            .onFailure { problem = said(it) }
    }
    val shownTitle = here?.let { spot ->
        spot.folder.lastOrNull()
            ?: model.storages.firstOrNull { it.id == spot.id }?.let { describeStorage(it.kind, it.values) }
    } ?: title
    ModalBottomSheet(
        onDismissRequest = close,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        containerColor = c.page,
        contentColor = c.text,
    ) {
        Column(
            Modifier.padding(start = 16.dp, end = 16.dp, bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                if (here != null) {
                    IconButton(onClick = { at = if (here.folder.isEmpty()) null else here.copy(folder = here.folder.dropLast(1)) }) {
                        Icon(Icons.AutoMirrored.Outlined.ArrowBack, tr("Back"), tint = c.text)
                    }
                }
                Text(
                    shownTitle,
                    Modifier.padding(start = 4.dp).weight(1f),
                    fontSize = 20.sp,
                    fontWeight = FontWeight.Bold,
                    color = c.text,
                    maxLines = 1,
                    overflow = TextOverflow.MiddleEllipsis,
                )
            }
            Column(
                Modifier
                    .fillMaxWidth()
                    .heightIn(max = 420.dp)
                    .clip(RoundedCornerShape(20.dp))
                    .background(c.surface)
                    .verticalScroll(rememberScrollState()),
            ) {
                when {
                    here == null -> model.storages.forEach { entry ->
                        PickerRow(describeStorage(entry.kind, entry.values), "picker-storage") { at = Screen.Storage(entry.id) }
                    }
                    problem != null -> Text(problem!!, Modifier.padding(16.dp), fontSize = 14.sp, color = c.warnText)
                    folders == null -> CircularProgressIndicator(
                        Modifier.padding(16.dp).size(22.dp),
                        color = c.accent,
                        strokeWidth = 3.dp,
                    )
                    else -> folders!!.forEach { folder ->
                        PickerRow(folder.name, "picker-folder-${folder.name}") { at = here.copy(folder = here.folder + folder.name) }
                    }
                }
            }
            if (here != null) {
                Button(
                    onClick = { chosen(here) },
                    shape = RoundedCornerShape(28.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
                    modifier = Modifier.fillMaxWidth().height(52.dp).testTag("picker-here"),
                ) { Text(confirm, fontSize = 17.sp, fontWeight = FontWeight.Bold) }
            }
        }
    }
}

@Composable
private fun PickerRow(label: String, tag: String, go: () -> Unit) {
    val c = tonal()
    Row(
        Modifier
            .fillMaxWidth()
            .heightIn(min = 52.dp)
            .clickable(onClick = go)
            .padding(horizontal = 16.dp)
            .testTag(tag),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Icon(Icons.Outlined.Folder, null, tint = c.accent)
        Text(label, fontSize = 16.sp, color = c.text, maxLines = 1, overflow = TextOverflow.MiddleEllipsis)
    }
    HorizontalDivider(color = c.field)
}

// MARK: Adding one

/** A storage's kind and settings, tried before it is kept; its secrets go to the vault. */
@Composable
fun AddStorageScreen(model: AppModel) {
    val c = tonal()
    val work = rememberCoroutineScope()
    val kinds = remember { storageKinds() }
    var kindId by remember { mutableStateOf(kinds.first().id) }
    val values = remember { androidx.compose.runtime.mutableStateMapOf<String, String>() }
    var checking by remember { mutableStateOf(false) }
    var problem by remember { mutableStateOf<String?>(null) }
    val kind = kinds.first { it.id == kindId }

    fun add() {
        val secretKeys = kind.fields.filter { it.secret }.map { it.key }.toSet()
        val settings = values
            .filter { (key, value) ->
                key !in secretKeys && (kind.fields.any { it.key == key } || key == kind.keyValue) && value.isNotBlank()
            }
            .mapValues { it.value.trim() }
        val secrets = values.filter { it.key in secretKeys }.toMap()
        val entry = Settings.StorageEntry(UUID.randomUUID().toString(), kind.id, settings)
        checking = true
        problem = null
        work.launch {
            val result = withContext(Dispatchers.IO) {
                runCatching {
                    RemoteStorage.open(entry.kind, entry.id, entry.values, secrets).list(emptyList(), Job.quiet())
                    model.addStorage(entry, secrets)
                }
            }
            checking = false
            result.onSuccess {
                // Kept: no password is left in the form as it goes.
                secretKeys.forEach { values.remove(it) }
                model.back()
            }.onFailure { problem = said(it) }
        }
    }

    Column(
        Modifier
            .fillMaxSize()
            .padding(top = WindowInsets.statusBars.asPaddingValues().calculateTopPadding())
            // Its button above the keyboard, not under it.
            .imePadding(),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth().height(64.dp).padding(horizontal = 4.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(onClick = { model.back() }) {
                Icon(Icons.AutoMirrored.Outlined.ArrowBack, tr("Back"), tint = c.text)
            }
            Text(tr("Add storage"), fontSize = 18.sp, fontWeight = FontWeight.Bold, color = c.text)
        }
        Column(
            Modifier
                .weight(1f)
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 16.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            Row(Modifier.fillMaxWidth()) {
                kinds.forEachIndexed { at, row ->
                    KindSegment(
                        row.name,
                        picked = row.id == kindId,
                        first = at == 0,
                        last = at == kinds.lastIndex,
                        modifier = Modifier.weight(1f).testTag("storage-kind-${row.id}"),
                    ) { kindId = row.id }
                }
            }
            kind.fields.forEach { field ->
                OutlinedTextField(
                    value = values[field.key] ?: "",
                    onValueChange = { values[field.key] = it },
                    label = { Text(field.label) },
                    placeholder = { Text(field.hint, color = c.faint) },
                    singleLine = true,
                    visualTransformation = if (field.secret) PasswordVisualTransformation() else androidx.compose.ui.text.input.VisualTransformation.None,
                    colors = TextFieldDefaults.colors(
                        focusedContainerColor = c.surface,
                        unfocusedContainerColor = c.surface,
                        focusedIndicatorColor = c.accent,
                        unfocusedIndicatorColor = c.outline,
                        focusedLabelColor = c.accent,
                        unfocusedLabelColor = c.muted,
                        cursorColor = c.accent,
                        focusedTextColor = c.text,
                        unfocusedTextColor = c.text,
                    ),
                    modifier = Modifier.fillMaxWidth().testTag("storage-${field.key}"),
                )
            }
            kind.keyValue?.let { setting ->
                SshKeyPicker(values[setting] ?: "") { values[setting] = it }
            }
            if (kind.fields.any { it.secret }) {
                Text(
                    tr("Secrets go in the system’s keychain, not with the settings."),
                    modifier = Modifier.padding(horizontal = 6.dp),
                    fontSize = 13.sp,
                    color = c.muted,
                )
            }
            problem?.let {
                Row(
                    Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(16.dp))
                        .background(c.warnBg)
                        .padding(14.dp),
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    Icon(Icons.Outlined.WarningAmber, null, tint = c.warnText)
                    Text(it, fontSize = 14.sp, color = c.warnText)
                }
            }
        }
        Button(
            onClick = { add() },
            enabled = !checking,
            shape = RoundedCornerShape(28.dp),
            colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
                .height(56.dp)
                .testTag("storage-add"),
        ) {
            if (checking) {
                CircularProgressIndicator(Modifier.size(20.dp), color = c.onAccent, strokeWidth = 2.5.dp)
                Box(Modifier.size(10.dp))
            }
            Text(if (checking) tr("Checking…") else tr("Add storage"), fontSize = 17.sp, fontWeight = FontWeight.Bold)
        }
    }
}

/** One of a row of buttons joined, as Material draws a segmented one. */
@Composable
private fun KindSegment(label: String, picked: Boolean, first: Boolean, last: Boolean, modifier: Modifier, pick: () -> Unit) {
    val c = tonal()
    val shape = RoundedCornerShape(
        topStart = if (first) 22.dp else 0.dp,
        bottomStart = if (first) 22.dp else 0.dp,
        topEnd = if (last) 22.dp else 0.dp,
        bottomEnd = if (last) 22.dp else 0.dp,
    )
    Box(
        modifier
            .height(44.dp)
            .clip(shape)
            .background(if (picked) c.tonal else c.page)
            .border(1.dp, c.accent, shape)
            .clickable(onClick = pick),
        contentAlignment = Alignment.Center,
    ) {
        // A kind's whole name ("WebDAV folder"): three to a phone's width.
        Text(
            label,
            modifier = Modifier.padding(horizontal = 6.dp),
            fontSize = 13.sp,
            fontWeight = FontWeight.SemiBold,
            color = if (picked) c.onTonal else c.text,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
