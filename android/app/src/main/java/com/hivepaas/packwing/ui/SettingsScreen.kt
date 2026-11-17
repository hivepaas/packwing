package com.hivepaas.packwing.ui

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Check
import androidx.compose.material.icons.outlined.ChevronRight
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Appearance
import com.hivepaas.packwing.BuildConfig
import com.hivepaas.packwing.Documents
import com.hivepaas.packwing.ExtractLimit
import com.hivepaas.packwing.ExtractPlace
import com.hivepaas.packwing.Screen
import com.hivepaas.packwing.StreamAbove
import com.hivepaas.packwing.tr
import uniffi.packwing_mobile.ArchiveFormat
import uniffi.packwing_mobile.CompressionLevel
import uniffi.packwing_mobile.checksumKinds
import uniffi.packwing_mobile.defaultStoredExtensions
import uniffi.packwing_mobile.extensionsIn
import uniffi.packwing_mobile.languages

/**
 * What the user chooses, as the desktop's Settings and the iOS app's: the
 * language and how it looks, what new archives are made with, where extracted
 * files go, the keys, the checksums shown, and what to be told.
 */
@Composable
fun SettingsScreen(model: AppModel) {
    val c = tonal()
    val context = LocalContext.current
    val s = model.settings
    var destination by remember { mutableStateOf(s.destination) }
    var tell by remember { mutableStateOf(s.tellWhenDone) }
    var language by remember { mutableStateOf(s.language) }
    var pickingLanguage by remember { mutableStateOf(false) }
    var formatKey by remember { mutableStateOf(s.newFormat) }
    var levelKey by remember { mutableStateOf(s.newCompression) }
    var hideNames by remember { mutableStateOf(s.hideNames) }
    var storeCompressed by remember { mutableStateOf(s.storeCompressed) }
    var skipMacFiles by remember { mutableStateOf(s.skipMacFiles) }
    var streamAbove by remember { mutableStateOf(StreamAbove.of(s.streamAbove)) }
    var editingExtensions by remember { mutableStateOf(false) }
    var place by remember { mutableStateOf(ExtractPlace.of(s.extractTo)) }
    var overwrite by remember { mutableStateOf(s.overwrite) }
    var limit by remember { mutableStateOf(ExtractLimit.of(s.extractLimit)) }

    val pickFolder = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { picked ->
        if (picked != null) {
            Documents.keepAccess(context, picked, write = true)
            s.destination = picked
            destination = picked
        }
    }

    // What an earlier version kept, a format for one file, shows as ZIP.
    val manyFiles = formats.filter { !it.oneFile }
    val format = manyFiles.firstOrNull { it.key == formatKey } ?: manyFiles.first { it.format == ArchiveFormat.ZIP }
    val offeredLevels = levels.filter { it != CompressionLevel.STORE || format.stores }
    val level = offeredLevels.firstOrNull { it.key == levelKey } ?: CompressionLevel.MEDIUM

    LazyColumn(
        modifier = Modifier.fillMaxSize().testTag("settings-list"),
        contentPadding = PaddingValues(
            top = WindowInsets.statusBars.asPaddingValues().calculateTopPadding() + 8.dp,
            bottom = 24.dp,
            start = 16.dp,
            end = 16.dp,
        ),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item {
            Text(
                tr("Settings"),
                modifier = Modifier.padding(start = 4.dp, top = 8.dp, bottom = 2.dp),
                fontSize = 30.sp,
                fontWeight = FontWeight.Bold,
                color = c.text,
            )
        }

        item { Heading(tr("General")) }
        item {
            Group {
                Line(
                    tr("Language"),
                    languages().firstOrNull { it.tag == language }?.name ?: tr("Follow the system"),
                ) { pickingLanguage = !pickingLanguage }
                if (pickingLanguage) {
                    HorizontalDivider(color = c.field)
                    LanguageLine(tr("Follow the system"), language == null) {
                        language = null
                        model.setLanguage(null)
                    }
                    languages().forEach { row ->
                        LanguageLine(row.name, language == row.tag) {
                            language = row.tag
                            model.setLanguage(row.tag)
                        }
                    }
                }
                HorizontalDivider(color = c.field)
                Choice(
                    tr("Appearance"),
                    model.appearance.label,
                    Appearance.entries.map { it.label to it.key },
                    tag = "settings-appearance",
                ) { model.chooseAppearance(Appearance.of(it)) }
            }
        }

        item { Heading(tr("New archives")) }
        item {
            Group {
                Choice(tr("Format"), format.label, manyFiles.map { it.label to it.key }, tag = "settings-format") {
                    formatKey = it
                    s.newFormat = it
                }
                HorizontalDivider(color = c.field)
                Choice(tr("Compression"), level.label, offeredLevels.map { it.label to it.key }, tag = "settings-compression") {
                    levelKey = it
                    s.newCompression = it
                }
                HorizontalDivider(color = c.field)
                Toggle(
                    tr("Hide file names behind the password"),
                    hideNames,
                    "settings-hide-names",
                    detail = tr("For 7z: nobody sees what is inside without it"),
                ) {
                    hideNames = it
                    s.hideNames = it
                }
                HorizontalDivider(color = c.field)
                Toggle(
                    tr("Don’t compress what already is"),
                    storeCompressed,
                    "settings-store-compressed",
                    detail = tr("For ZIP and 7z: pictures, video, sound and archives go in as they are, sooner and no larger"),
                ) {
                    storeCompressed = it
                    s.storeCompressed = it
                }
                if (storeCompressed) {
                    HorizontalDivider(color = c.field)
                    Line(tr("What goes in as it is"), "", tag = "settings-stored-extensions") { editingExtensions = true }
                }
                HorizontalDivider(color = c.field)
                Toggle(
                    tr("Leave out Mac-only files"),
                    skipMacFiles,
                    "settings-skip-mac-files",
                    detail = tr(".DS_Store, __MACOSX and resource forks, clutter anywhere but a Mac"),
                ) {
                    skipMacFiles = it
                    s.skipMacFiles = it
                }
                HorizontalDivider(color = c.field)
                Choice(
                    tr("Write straight to a storage above"),
                    streamAbove.label,
                    StreamAbove.all.map { it.label to it.key },
                    tag = "settings-stream-above",
                ) {
                    streamAbove = StreamAbove.of(it)
                    s.streamAbove = it
                }
            }
        }
        item {
            Note(tr("Above it, an archive bound for a storage takes no room on this device, but a lost connection means compressing again."))
        }

        item { Heading(tr("Extracting")) }
        item {
            Group {
                Line(
                    tr("Folder"),
                    destination?.let { Documents.treeName(context, it) } ?: tr("Ask every time"),
                    tag = "settings-folder",
                ) { pickFolder.launch(destination) }
                HorizontalDivider(color = c.field)
                Choice(
                    tr("Where the files go"),
                    place.label,
                    ExtractPlace.entries.map { it.label to it.key },
                    tag = "settings-extract-to",
                    detail = place.detail,
                ) {
                    place = ExtractPlace.of(it)
                    s.extractTo = it
                }
                HorizontalDivider(color = c.field)
                // A new folder holds nothing yet: nothing to stop at or replace.
                Choice(
                    tr("When a file is already there"),
                    if (overwrite) tr("Replace it") else tr("Stop and say so"),
                    listOf(tr("Stop and say so") to "stop", tr("Replace it") to "replace"),
                    tag = "settings-extract-existing",
                    enabled = place != ExtractPlace.NewFolder,
                ) {
                    overwrite = it == "replace"
                    s.overwrite = overwrite
                }
                HorizontalDivider(color = c.field)
                Choice(
                    tr("Largest an archive may unpack to"),
                    limit.label,
                    ExtractLimit.all.map { it.label to it.key },
                    tag = "settings-extract-limit",
                ) {
                    limit = ExtractLimit.of(it)
                    s.extractLimit = it
                }
            }
        }
        item { Note(tr("Stops archive bombs: small files that unpack to fill the disk")) }

        item { Heading(tr("Keys you trust")) }
        item { TrustedKeys(model) }
        item {
            Note(tr("A signature by one of these says who signed a file. One by any other key proves nothing, whatever it matches."))
        }

        item { Heading(tr("Signing keys")) }
        item { SigningKeys(model) }
        item { Note(tr("Kept in this device’s keychain. Anyone can check what a key signs with its public half.")) }

        item { Heading(tr("SSH keys")) }
        item { SshKeys() }

        item { Heading(tr("Checksums")) }
        item { ChecksumKinds(model) }

        item { Heading(tr("While it works")) }
        item {
            Group {
                Toggle(tr("Tell me when a job is through"), tell, "settings-tell") {
                    tell = it
                    s.tellWhenDone = it
                }
            }
        }

        item {
            Group {
                Row(
                    Modifier
                        .fillMaxWidth()
                        .heightIn(min = 60.dp)
                        .clickable { model.go(Screen.About) }
                        .padding(horizontal = 16.dp, vertical = 10.dp)
                        .testTag("settings-about"),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    PackwingLogo(36.dp)
                    Text(tr("About"), Modifier.weight(1f), fontSize = 16.sp, color = c.text)
                    Text(BuildConfig.VERSION_NAME, fontSize = 15.sp, color = c.muted)
                    Icon(Icons.Outlined.ChevronRight, null, tint = c.muted)
                }
            }
        }
    }

    if (editingExtensions) StoredExtensionsSheet(model) { editingExtensions = false }
}

/**
 * The kinds a file's or an archive's checksums show, as the Checksums
 * screen's own menu chooses them; one at least.
 */
@Composable
private fun ChecksumKinds(model: AppModel) {
    val c = tonal()
    val kinds = remember { checksumKinds() }
    val shown = shownChecksums(model.checksumKeys).map { it.key }.toSet()
    Group {
        kinds.forEachIndexed { at, row ->
            if (at > 0) HorizontalDivider(color = c.field)
            Toggle(
                if (row.weak) tr("{name} (weak)", "name" to row.name) else row.name,
                row.key in shown,
                "settings-checksum-${row.key}",
            ) { on ->
                val keys = if (on) shown + row.key else if (shown.size > 1) shown - row.key else shown
                // In the order they are shown.
                model.showChecksums(kinds.map { it.key }.filter { it in keys }.joinToString(" "))
            }
        }
    }
}

/**
 * Settings › What goes in as it is: the extensions typed, apart by spaces,
 * and the way back to the desktop's list, which is kept as none and followed.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun StoredExtensionsSheet(model: AppModel, close: () -> Unit) {
    val c = tonal()
    val standard = remember { defaultStoredExtensions() }
    var text by remember {
        val kept = model.settings.storedExtensions
        mutableStateOf((if (kept.isBlank()) standard else extensionsIn(kept)).joinToString(" "))
    }
    var kept by remember { mutableStateOf(model.settings.storedExtensions) }
    ModalBottomSheet(
        onDismissRequest = close,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        containerColor = c.page,
        contentColor = c.text,
    ) {
        Column(
            Modifier.padding(start = 16.dp, end = 16.dp, bottom = 24.dp).imePadding(),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(tr("What goes in as it is"), fontSize = 20.sp, fontWeight = FontWeight.Bold, color = c.text)
            OutlinedTextField(
                value = text,
                onValueChange = { now ->
                    text = now
                    val read = extensionsIn(now)
                    // The desktop's list again is kept as none: it follows the desktop's.
                    kept = if (read == standard || read.isEmpty()) "" else read.joinToString(" ")
                    model.settings.storedExtensions = kept
                },
                placeholder = { Text(tr("Extensions, apart by spaces")) },
                minLines = 4,
                maxLines = 12,
                keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.None, autoCorrectEnabled = false),
                colors = OutlinedTextFieldDefaults.colors(
                    focusedBorderColor = c.accent,
                    unfocusedBorderColor = c.outline,
                    focusedContainerColor = c.surface,
                    unfocusedContainerColor = c.surface,
                ),
                modifier = Modifier.fillMaxWidth().testTag("settings-extensions"),
            )
            Note(tr("Files whose names end in these, whatever their case"))
            TextButton(
                onClick = {
                    text = standard.joinToString(" ")
                    kept = ""
                    model.settings.storedExtensions = ""
                },
                enabled = kept.isNotEmpty(),
                modifier = Modifier.testTag("settings-extensions-reset"),
            ) {
                Text(
                    tr("Reset to default"),
                    color = if (kept.isNotEmpty()) c.accent else c.faint,
                    fontWeight = FontWeight.SemiBold,
                )
            }
        }
    }
}

@Composable
private fun Heading(text: String) = Text(
    text,
    modifier = Modifier.padding(start = 6.dp, top = 4.dp),
    fontSize = 14.sp,
    fontWeight = FontWeight.Bold,
    color = tonal().accent,
)

/** What a group says of itself, under it. */
@Composable
private fun Note(text: String) = Text(
    text,
    modifier = Modifier.padding(horizontal = 8.dp),
    fontSize = 13.sp,
    lineHeight = 19.sp,
    color = tonal().muted,
)

@Composable
private fun Group(content: @Composable () -> Unit) = Column(
    Modifier
        .fillMaxWidth()
        .clip(RoundedCornerShape(20.dp))
        .background(tonal().surface),
) { content() }

/** A row that says what is chosen and opens what chooses it; with no `open`, one that only says. */
@Composable
private fun Line(title: String, value: String, tag: String? = null, open: (() -> Unit)? = {}) {
    val c = tonal()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 60.dp)
            .let { if (open != null) it.clickable(onClick = open) else it }
            .let { if (tag != null) it.testTag(tag) else it }
            .padding(horizontal = 16.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Text(title, Modifier.weight(1f), fontSize = 16.sp, color = c.text)
        if (value.isNotEmpty()) {
            Text(
                value,
                fontSize = 15.sp,
                color = c.muted,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        } else if (open != null) {
            Icon(Icons.Outlined.ChevronRight, null, tint = c.muted)
        }
    }
}

@Composable
private fun LanguageLine(name: String, picked: Boolean, pick: () -> Unit) {
    val c = tonal()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 52.dp)
            .clickable(onClick = pick)
            .padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(name, Modifier.weight(1f), fontSize = 15.sp, color = c.text)
        if (picked) Icon(Icons.Outlined.Check, null, tint = c.accent)
    }
}
