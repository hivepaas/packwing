package com.hivepaas.packwing.ui

import android.icu.text.ListFormatter
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material.icons.outlined.ExpandMore
import androidx.compose.material.icons.outlined.Folder
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material.icons.outlined.WarningAmber
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Documents
import com.hivepaas.packwing.Screen
import com.hivepaas.packwing.StreamAbove
import com.hivepaas.packwing.said
import com.hivepaas.packwing.tr
import com.hivepaas.packwing.trN
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.packwing_mobile.ArchiveFormat
import uniffi.packwing_mobile.ArchiveFormatRow
import uniffi.packwing_mobile.CompressionLevel
import uniffi.packwing_mobile.MobileException
import uniffi.packwing_mobile.NewArchiveHow
import uniffi.packwing_mobile.ageKeyIsPostQuantum
import uniffi.packwing_mobile.ageKeyProblem
import uniffi.packwing_mobile.ageKeysIn
import uniffi.packwing_mobile.archiveFormats
import uniffi.packwing_mobile.extensionsIn
import java.io.File
import java.util.Locale

/** A file, folder or photo to go in: its copy in the app's own folder. */
private data class NewItem(val file: File, val photo: Boolean = false) {
    val name: String get() = file.name
    val folder: Boolean get() = file.isDirectory
}

/** How a new archive is cut in parts. */
private enum class Split { None, Email, Usb, Custom }

/** A file of public keys age encrypts to. */
private data class KeysFile(val name: String, val keys: List<String>, val leftOut: ULong)

/** The formats, those for one file by itself too, with what each can do. */
internal val formats: List<ArchiveFormatRow> by lazy { archiveFormats(true) }

internal val ArchiveFormatRow.key: String get() = ext.removePrefix(".")

/** As it is called: ZIP and 7z by their names, the others by what follows a name. */
internal val ArchiveFormatRow.label: String
    get() = when (format) {
        ArchiveFormat.ZIP -> "ZIP"
        ArchiveFormat.SEVEN_ZIP -> "7z"
        else -> key
    }

internal val levels = listOf(
    CompressionLevel.STORE,
    CompressionLevel.LOWEST,
    CompressionLevel.LOW,
    CompressionLevel.MEDIUM,
    CompressionLevel.HIGH,
    CompressionLevel.HIGHEST,
)

internal val CompressionLevel.key: String get() = name.lowercase(Locale.ROOT)

internal val CompressionLevel.label: String
    get() = when (this) {
        CompressionLevel.STORE -> tr("Store")
        CompressionLevel.LOWEST -> tr("Lowest")
        CompressionLevel.LOW -> tr("Low")
        CompressionLevel.MEDIUM -> tr("Medium")
        CompressionLevel.HIGH -> tr("High")
        CompressionLevel.HIGHEST -> tr("Highest")
    }

/** The smallest part: a ZIP's parts are 64 KB or more, and parts this small help nobody. */
private const val SMALLEST_PART = 1_000_000.0

/** A size typed in, in bytes: a number of `unit`s, a comma as the decimal point, of 1 MB or more. */
private fun partBytes(typed: String, unit: ULong): ULong? {
    val n = typed.trim().replace(',', '.').toDoubleOrNull() ?: return null
    val bytes = n * unit.toDouble()
    return if (bytes.isFinite() && bytes >= SMALLEST_PART) Math.round(bytes).toULong() else null
}

/**
 * What New archive asks before it starts: what goes in, its name, and how,
 * as the desktop and the iOS app ask it. In a folder of the phone's, or in a
 * storage's folder (`screen.on`, or one chosen).
 */
@Composable
fun CreateScreen(model: AppModel, screen: Screen.Create) {
    val c = tonal()
    val context = LocalContext.current
    val work = rememberCoroutineScope()
    var items by remember { mutableStateOf(screen.files.map { NewItem(it) }) }
    var copying by remember { mutableStateOf(0) }
    var problem by remember { mutableStateOf<String?>(null) }
    var showingItems by remember { mutableStateOf(false) }
    var name by remember { mutableStateOf("") }
    var named by remember { mutableStateOf(false) }
    var formatKey by remember { mutableStateOf(model.settings.newFormat) }
    // A format for one file (`.gz`) is this archive's alone: not kept for those after it.
    var oneFileKey by remember { mutableStateOf<String?>(null) }
    var levelKey by remember { mutableStateOf(model.settings.newCompression) }
    var split by remember { mutableStateOf(Split.None) }
    var customSize by remember { mutableStateOf("") }
    var customUnit by remember { mutableStateOf(1_000_000UL) }
    var locked by remember { mutableStateOf(false) }
    var wholeWithAge by remember { mutableStateOf(false) }
    var toKeys by remember { mutableStateOf(false) }
    var password by remember { mutableStateOf("") }
    var showingPassword by remember { mutableStateOf(false) }
    var hideNames by remember { mutableStateOf(model.settings.hideNames) }
    var key by remember { mutableStateOf("") }
    var keysFile by remember { mutableStateOf<KeysFile?>(null) }
    var keysProblem by remember { mutableStateOf<String?>(null) }
    var folder by remember { mutableStateOf(model.settings.destination) }
    var onStorage by remember { mutableStateOf(screen.on) }
    var pickingStorage by remember { mutableStateOf(false) }

    // What comes from a picker is copied in: the core reads paths.
    fun take(photo: Boolean, copy: suspend () -> List<File>) {
        copying++
        work.launch {
            val came = withContext(Dispatchers.IO) { runCatching { copy() } }
            copying--
            came.onSuccess { files -> items = items + files.map { NewItem(it, photo) } }
                .onFailure { problem = it.message ?: it.toString() }
        }
    }
    val addFiles = rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris ->
        if (uris.isNotEmpty()) take(false) { Documents.copyIn(context, uris) }
    }
    val addFolder = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { tree ->
        if (tree != null) take(false) { listOf(Documents.copyTreeIn(context, tree)) }
    }
    val addPhotos = rememberLauncherForActivityResult(ActivityResultContracts.PickMultipleVisualMedia()) { uris ->
        if (uris.isNotEmpty()) take(true) { Documents.copyIn(context, uris) }
    }
    val pickFolder = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { picked ->
        if (picked != null) {
            Documents.keepAccess(context, picked, write = true)
            model.settings.destination = picked
            folder = picked
            onStorage = null
        }
    }
    val chooseKeys = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri != null) {
            work.launch {
                val read = withContext(Dispatchers.IO) {
                    runCatching {
                        val bytes = context.contentResolver.openInputStream(uri)?.use { it.upTo(16 shl 20) } ?: ByteArray(0)
                        Documents.picked(context, uri).name to ageKeysIn(bytes)
                    }
                }
                read.onSuccess { (called, file) ->
                    keysFile = KeysFile(called, file.keys, file.leftOut)
                    keysProblem = null
                }.onFailure { e ->
                    keysFile = null
                    keysProblem = (e as? MobileException)?.said ?: e.toString()
                }
            }
        }
    }

    // One file by itself, not a folder: the formats for one file are offered too.
    val oneFile = items.size == 1 && !items[0].folder
    val format = formats.firstOrNull { oneFile && it.key == oneFileKey }
        ?: formats.firstOrNull { it.key == formatKey && !it.oneFile }
        ?: formats.first()
    val offered = formats.filter { oneFile || !it.oneFile }
    // Store is ZIP's and 7z's; Lowest for the others.
    val level = levels.firstOrNull { it.key == levelKey }
        ?.let { if (it == CompressionLevel.STORE && !format.stores) CompressionLevel.LOWEST else it }
        ?: CompressionLevel.MEDIUM
    // age encrypts it: a format age is around, or ZIP and 7z when chosen in place of their AES-256.
    val withAge = format.usesAge || wholeWithAge
    val encryptsToKeys = locked && toKeys && withAge
    val typedKey = key.trim()
    val ageKeys = when {
        !encryptsToKeys -> emptyList()
        keysFile != null -> keysFile!!.keys
        typedKey.isNotEmpty() && ageKeyProblem(typedKey) == null -> listOf(typedKey)
        else -> emptyList()
    }
    val partSize = when (split) {
        Split.None -> null
        Split.Email -> 25_000_000UL
        Split.Usb -> 4_000_000_000UL
        Split.Custom -> partBytes(customSize, customUnit)
    }
    val savedExt = format.ext + if (locked && withAge) ".age" else ""
    val suggested = items.singleOrNull()?.let { if (it.folder) it.name else it.name.substringBeforeLast('.') }
        ?: tr("Archive")
    val shownName = if (named) name else suggested
    // The name as a file can be called: no folders in it, nor its extension twice.
    val fileName = shownName.trim().replace('/', '-').replace(':', '-')
        .let { if (it.lowercase(Locale.ROOT).endsWith(savedExt)) it.dropLast(savedExt.length) else it }.trim()
    val storageEntry = onStorage?.let { spot -> model.storages.firstOrNull { it.id == spot.id } }
    val ready = items.isNotEmpty() && copying == 0 && fileName.isNotEmpty() &&
        (split != Split.Custom || partSize != null) &&
        (storageEntry != null || folder != null) &&
        (!locked || if (encryptsToKeys) ageKeys.isNotEmpty() else password.isNotEmpty())

    fun start() {
        if (!ready) return
        val how = NewArchiveHow(
            format = format.format,
            compression = level,
            password = if (locked && !encryptsToKeys) password else null,
            hideNames = locked && hideNames && format.format == ArchiveFormat.SEVEN_ZIP && !withAge,
            partSize = partSize,
            ageKeys = ageKeys,
            storeCompressed = model.settings.storeCompressed,
            skipMacFiles = model.settings.skipMacFiles,
            wholeWithAge = locked && wholeWithAge && !format.usesAge,
            // Typed in Settings, or the desktop's list until any are.
            storedExtensions = model.settings.storedExtensions.takeIf { it.isNotBlank() }?.let(::extensionsIn),
            streamAbove = StreamAbove.of(model.settings.streamAbove).bytes,
        )
        val paths = items.map { it.file }
        val spot = onStorage
        if (spot != null && storageEntry != null) {
            model.create(paths, fileName + savedExt, how, storageEntry, spot.folder)
        } else {
            model.create(paths = paths, name = fileName + savedExt, how = how, into = folder ?: return)
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
            Text(tr("New archive"), fontSize = 18.sp, fontWeight = FontWeight.Bold, color = c.text)
        }

        LazyColumn(
            modifier = Modifier.weight(1f),
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            // What goes in: in short, or one by one; and more.
            item {
                Card {
                    Row(
                        Modifier
                            .fillMaxWidth()
                            .heightIn(min = 64.dp)
                            .clickable(enabled = items.isNotEmpty()) { showingItems = !showingItems }
                            .padding(horizontal = 16.dp, vertical = 10.dp)
                            .testTag("new-items"),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                            items.take(3).forEach { KindTile(it.name, FileKind.of(it.name, it.folder), size = 36.dp) }
                        }
                        Column(Modifier.weight(1f)) {
                            Text(summary(items), fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = c.text)
                            if (copying > 0) Text(tr("Reading…"), fontSize = 13.sp, color = c.muted)
                        }
                        if (items.isNotEmpty()) Icon(Icons.Outlined.ExpandMore, null, tint = c.muted)
                    }
                    if (showingItems) {
                        items.forEach { item ->
                            HorizontalDivider(color = c.field)
                            Row(
                                Modifier.fillMaxWidth().padding(start = 16.dp, end = 4.dp, top = 6.dp, bottom = 6.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(12.dp),
                            ) {
                                KindTile(item.name, FileKind.of(item.name, item.folder), size = 32.dp)
                                Text(
                                    item.name,
                                    Modifier.weight(1f),
                                    fontSize = 15.sp,
                                    color = c.text,
                                    maxLines = 1,
                                    overflow = TextOverflow.MiddleEllipsis,
                                )
                                IconButton(onClick = {
                                    items = items - item
                                    work.launch(Dispatchers.IO) { item.file.parentFile?.deleteRecursively() }
                                }) {
                                    Icon(Icons.Outlined.Close, tr("Remove"), tint = c.muted)
                                }
                            }
                        }
                    }
                    HorizontalDivider(color = c.field)
                    Row(Modifier.fillMaxWidth().padding(start = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                        AddMenu(
                            files = { addFiles.launch(arrayOf("*/*")) },
                            folder = { addFolder.launch(null) },
                            photos = { addPhotos.launch(PickVisualMediaRequest(ActivityResultContracts.PickVisualMedia.ImageAndVideo)) },
                        )
                        Text(tr("Add more"), fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = c.accent)
                    }
                }
            }

            item { Heading(tr("Name")) }
            item {
                OutlinedTextField(
                    value = shownName,
                    onValueChange = {
                        name = it
                        named = true
                    },
                    singleLine = true,
                    suffix = { Text(savedExt, color = c.muted) },
                    keyboardOptions = KeyboardOptions(autoCorrectEnabled = false),
                    colors = fieldColors(),
                    modifier = Modifier.fillMaxWidth().testTag("new-name"),
                )
            }

            item {
                Card {
                    Choice(tr("Format"), format.label, offered.map { it.label to it.key }, tag = "new-format") { chosen ->
                        val row = formats.first { it.key == chosen }
                        if (row.oneFile) {
                            oneFileKey = chosen
                        } else {
                            oneFileKey = null
                            formatKey = chosen
                            model.settings.newFormat = chosen
                        }
                    }
                    if (format.format == ArchiveFormat.ZIP || format.format == ArchiveFormat.SEVEN_ZIP) {
                        Note(
                            if (format.format == ArchiveFormat.ZIP) {
                                tr("Opens everywhere. With a password, AES-256: Windows’ own Explorer can’t open it, 7-Zip and packwing can.")
                            } else {
                                tr("Smaller. Opens with packwing, 7-Zip and most archivers.")
                            },
                        )
                    }
                    HorizontalDivider(color = c.field)
                    Choice(
                        tr("Compression"),
                        level.label,
                        levels.filter { it != CompressionLevel.STORE || format.stores }.map { it.label to it.key },
                        tag = "new-compression",
                    ) {
                        levelKey = it
                        model.settings.newCompression = it
                    }
                    HorizontalDivider(color = c.field)
                    Choice(tr("Split into parts"), splitLabel(split), Split.entries.map { splitLabel(it) to it.name }, tag = "new-split") {
                        split = Split.valueOf(it)
                    }
                    if (split == Split.Custom) {
                        Row(
                            Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            Text(tr("Each part"), Modifier.weight(1f), fontSize = 16.sp, color = c.text)
                            OutlinedTextField(
                                value = customSize,
                                onValueChange = { customSize = it },
                                placeholder = { Text("100", color = c.faint) },
                                singleLine = true,
                                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                                colors = fieldColors(),
                                modifier = Modifier.width(110.dp).testTag("new-part-size"),
                            )
                            val units = listOf("MB" to 1_000_000UL, "GB" to 1_000_000_000UL)
                            Row {
                                units.forEachIndexed { at, (label, bytes) ->
                                    Pill(label, customUnit == bytes, first = at == 0, last = at == units.lastIndex, tag = "new-part-unit-$label") {
                                        customUnit = bytes
                                    }
                                }
                            }
                        }
                        if (customSize.isNotEmpty() && partSize == null) {
                            Warning(tr("A part takes a size of 1 MB or more, such as 100 MB."))
                        }
                    }
                    if (split != Split.None) {
                        Note(tr("Parts open in 7-Zip and archivers like it, not in the tools built into Windows and macOS."))
                    }
                    HorizontalDivider(color = c.field)
                    Toggle(if (withAge) tr("Encrypt with age") else tr("Protect with a password"), locked, "new-locked") { locked = it }
                    // ZIP and 7z have their own AES-256, or age around them, as the others have.
                    if (locked && !format.usesAge) {
                        Pills(listOf("${format.label.uppercase(Locale.ROOT)} AES-256" to false, "age" to true), wholeWithAge, "new-encryption") {
                            wholeWithAge = it
                        }
                    }
                    if (locked && withAge) {
                        Pills(listOf(tr("Passphrase") to false, tr("Public key") to true), toKeys, "new-age-to") { toKeys = it }
                    }
                    if (encryptsToKeys) {
                        val file = keysFile
                        if (file != null) {
                            Row(
                                Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                Text(
                                    trN(file.keys.size, "{n} key from {name}", "{n} keys from {name}", "n" to file.keys.size.toString(), "name" to file.name),
                                    Modifier.weight(1f).testTag("new-age-keys-file"),
                                    fontSize = 15.sp,
                                    color = c.text,
                                    maxLines = 1,
                                    overflow = TextOverflow.MiddleEllipsis,
                                )
                                TextButton(onClick = { keysFile = null }) { Text(tr("Remove"), color = c.accent) }
                            }
                            if (file.leftOut > 0UL) {
                                Warning(
                                    trN(
                                        file.leftOut,
                                        "{n} key in it was left out, of a kind age does not encrypt to.",
                                        "{n} keys in it were left out, of a kind age does not encrypt to.",
                                        "n" to file.leftOut.toString(),
                                    ),
                                )
                            }
                        } else {
                            Row(
                                Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(8.dp),
                            ) {
                                OutlinedTextField(
                                    value = key,
                                    onValueChange = { key = it },
                                    placeholder = { Text(tr("age1…, age1pq1… or ssh-ed25519 …"), color = c.faint, fontSize = 13.sp) },
                                    singleLine = true,
                                    textStyle = androidx.compose.ui.text.TextStyle(fontSize = 13.sp, fontFamily = FontFamily.Monospace),
                                    keyboardOptions = KeyboardOptions(autoCorrectEnabled = false, keyboardType = KeyboardType.Ascii),
                                    colors = fieldColors(),
                                    modifier = Modifier.weight(1f).testTag("new-age-key"),
                                )
                                TextButton(onClick = { chooseKeys.launch(arrayOf("*/*")) }) {
                                    Text(tr("From a file…"), color = c.accent, fontWeight = FontWeight.SemiBold)
                                }
                            }
                            if (typedKey.isNotEmpty()) ageKeyProblem(typedKey)?.let { Warning(it) }
                        }
                        keysProblem?.let { Warning(it) }
                        if (ageKeys.any { ageKeyIsPostQuantum(it) }) {
                            Warning(tr("A post-quantum key: to open this, the age tool must be version 1.3 or newer."))
                        }
                        Warning(tr("Only the private key that goes with it opens this, and nothing can if that key is lost."))
                    } else if (locked) {
                        OutlinedTextField(
                            value = password,
                            onValueChange = { password = it },
                            label = { Text(if (withAge) tr("Passphrase") else tr("Password")) },
                            singleLine = true,
                            visualTransformation = if (showingPassword) VisualTransformation.None else PasswordVisualTransformation(),
                            keyboardOptions = KeyboardOptions(autoCorrectEnabled = false, keyboardType = KeyboardType.Password),
                            trailingIcon = {
                                IconButton(onClick = { showingPassword = !showingPassword }) {
                                    Icon(
                                        if (showingPassword) Icons.Outlined.VisibilityOff else Icons.Outlined.Visibility,
                                        if (showingPassword) tr("Hide the password") else tr("Show the password"),
                                        tint = c.muted,
                                    )
                                }
                            },
                            colors = fieldColors(),
                            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp).testTag("new-password"),
                        )
                        // age hides it all, names too: only ZIP's and 7z's own AES have a choice.
                        if (!withAge) {
                            val sevenZip = format.format == ArchiveFormat.SEVEN_ZIP
                            Toggle(
                                tr("Hide the file names too"),
                                hideNames && sevenZip,
                                "new-hide-names",
                                detail = if (sevenZip) {
                                    tr("No one sees what is inside without the password.")
                                } else {
                                    tr("7z only: ZIP always shows its names.")
                                },
                                enabled = sevenZip,
                            ) { hideNames = it }
                        }
                    }
                }
            }

            item { Heading(tr("Save to")) }
            item {
                Card {
                    Row(
                        Modifier.fillMaxWidth().heightIn(min = 64.dp).padding(start = 16.dp, end = 8.dp, top = 8.dp, bottom = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        Icon(Icons.Outlined.Folder, null, tint = c.accent, modifier = Modifier.size(24.dp))
                        val spot = onStorage
                        Text(
                            if (spot != null && storageEntry != null) {
                                model.storage(storageEntry).showPath(spot.folder)
                            } else {
                                folder?.let { Documents.treeName(context, it) } ?: tr("No folder chosen yet")
                            },
                            Modifier.weight(1f).testTag("new-save-to-shown"),
                            fontSize = 15.sp,
                            fontWeight = FontWeight.SemiBold,
                            color = c.text,
                            maxLines = 2,
                            overflow = TextOverflow.MiddleEllipsis,
                        )
                        SaveTo(
                            storages = model.storages.isNotEmpty(),
                            files = { pickFolder.launch(folder) },
                            storage = { pickingStorage = true },
                        )
                    }
                }
            }

            problem?.let { item { Warning(it) } }
        }

        Button(
            onClick = { start() },
            enabled = ready,
            shape = RoundedCornerShape(28.dp),
            colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
            modifier = Modifier
                .fillMaxWidth()
                .padding(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 12.dp + WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding())
                .height(56.dp)
                .testTag("create-make"),
        ) {
            if (copying > 0) {
                CircularProgressIndicator(Modifier.size(20.dp), color = c.onAccent, strokeWidth = 2.5.dp)
                Box(Modifier.size(10.dp))
            }
            Text(tr("Create"), fontSize = 17.sp, fontWeight = FontWeight.Bold)
        }
    }

    if (pickingStorage) {
        StorageFolderPicker(model, tr("Save to"), tr("Save here"), close = { pickingStorage = false }) {
            pickingStorage = false
            onStorage = it
        }
    }
}

/** What they are, as the screen says it: "12 photos and 1 folder", in the language spoken. */
private fun summary(items: List<NewItem>): String {
    val photos = items.count { it.photo }
    val folders = items.count { it.folder }
    val files = items.size - photos - folders
    val said = buildList {
        if (photos > 0) add(trN(photos, "{n} photo", "{n} photos", "n" to photos.toString()))
        if (files > 0) add(trN(files, "{n} file", "{n} files", "n" to files.toString()))
        if (folders > 0) add(trN(folders, "{n} folder", "{n} folders", "n" to folders.toString()))
    }
    if (said.isEmpty()) return tr("Nothing yet")
    // Joined as the language packwing speaks joins a list: its own commas, its own "and".
    val language = uniffi.packwing_mobile.language()
    return runCatching { ListFormatter.getInstance(Locale.forLanguageTag(language)).format(said) }
        .getOrDefault(said.joinToString(", "))
}

private fun splitLabel(split: Split) = when (split) {
    Split.None -> tr("Don’t split")
    Split.Email -> tr("25 MB (email)")
    Split.Usb -> tr("4 GB (USB stick)")
    Split.Custom -> tr("Custom size…")
}

/** Change: a folder of the phone's, or one of a storage, when there are storages. */
@Composable
private fun SaveTo(storages: Boolean, files: () -> Unit, storage: () -> Unit) {
    val c = tonal()
    var open by remember { mutableStateOf(false) }
    Box {
        TextButton(onClick = { if (storages) open = true else files() }, modifier = Modifier.testTag("new-save-to")) {
            Text(tr("Change"), color = c.accent, fontWeight = FontWeight.SemiBold)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }, containerColor = c.surface) {
            DropdownMenuItem(
                text = { Text(tr("Files"), color = c.text) },
                onClick = {
                    open = false
                    files()
                },
            )
            DropdownMenuItem(
                text = { Text(tr("Storages"), color = c.text) },
                onClick = {
                    open = false
                    storage()
                },
                modifier = Modifier.testTag("new-save-to-storage"),
            )
        }
    }
}

@Composable
private fun Card(content: @Composable () -> Unit) = Column(
    Modifier.fillMaxWidth().clip(RoundedCornerShape(20.dp)).background(tonal().surface),
) { content() }

@Composable
private fun Heading(text: String) = Text(
    text,
    modifier = Modifier.padding(start = 6.dp, top = 4.dp),
    fontSize = 14.sp,
    fontWeight = FontWeight.Bold,
    color = tonal().accent,
)

/** Two choices side by side, one picked. */
@Composable
private fun <T> Pills(options: List<Pair<String, T>>, chosen: T, tag: String, choose: (T) -> Unit) {
    Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp)) {
        options.forEachIndexed { at, (label, value) ->
            Pill(label, value == chosen, first = at == 0, last = at == options.lastIndex, tag = "$tag-$at", modifier = Modifier.weight(1f)) {
                choose(value)
            }
        }
    }
}

@Composable
private fun Pill(
    label: String,
    picked: Boolean,
    first: Boolean,
    last: Boolean,
    tag: String,
    modifier: Modifier = Modifier,
    pick: () -> Unit,
) {
    val c = tonal()
    val shape = RoundedCornerShape(
        topStart = if (first) 20.dp else 0.dp,
        bottomStart = if (first) 20.dp else 0.dp,
        topEnd = if (last) 20.dp else 0.dp,
        bottomEnd = if (last) 20.dp else 0.dp,
    )
    Box(
        modifier
            .height(40.dp)
            .clip(shape)
            .background(if (picked) c.tonal else c.page)
            .border(1.dp, c.accent, shape)
            .clickable(onClick = pick)
            .padding(horizontal = 12.dp)
            .testTag(tag),
        contentAlignment = Alignment.Center,
    ) {
        Text(label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = if (picked) c.onTonal else c.text, maxLines = 1)
    }
}

@Composable
private fun Note(text: String) =
    Text(text, Modifier.padding(start = 16.dp, end = 16.dp, bottom = 10.dp), fontSize = 13.sp, lineHeight = 18.sp, color = tonal().muted)

@Composable
private fun Warning(text: String) {
    val c = tonal()
    Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        Icon(Icons.Outlined.WarningAmber, null, tint = c.warnText, modifier = Modifier.size(18.dp))
        Text(text, fontSize = 13.sp, lineHeight = 18.sp, color = c.warnText)
    }
}

@Composable
private fun fieldColors() = tonal().let { c ->
    TextFieldDefaults.colors(
        focusedContainerColor = c.surface,
        unfocusedContainerColor = c.surface,
        focusedIndicatorColor = c.accent,
        unfocusedIndicatorColor = c.outline,
        focusedLabelColor = c.accent,
        unfocusedLabelColor = c.muted,
        cursorColor = c.accent,
        focusedTextColor = c.text,
        unfocusedTextColor = c.text,
    )
}
