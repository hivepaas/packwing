package com.hivepaas.packwing.ui

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.automirrored.outlined.HelpOutline
import androidx.compose.material.icons.outlined.Cancel
import androidx.compose.material.icons.outlined.Check
import androidx.compose.material.icons.outlined.CheckCircle
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material.icons.outlined.ContentPaste
import androidx.compose.material.icons.outlined.Download
import androidx.compose.material.icons.outlined.RadioButtonUnchecked
import androidx.compose.material.icons.outlined.RemoveCircleOutline
import androidx.compose.material.icons.outlined.Tune
import androidx.compose.material.icons.outlined.Verified
import androidx.compose.material.icons.outlined.WarningAmber
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CheckboxDefaults
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.documentfile.provider.DocumentFile
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Documents
import com.hivepaas.packwing.Opened
import com.hivepaas.packwing.Picked
import com.hivepaas.packwing.Screen
import com.hivepaas.packwing.said
import com.hivepaas.packwing.tr
import com.hivepaas.packwing.trN
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.packwing_mobile.Checksum
import uniffi.packwing_mobile.ChecksumKind
import uniffi.packwing_mobile.ChecksumKindRow
import uniffi.packwing_mobile.Compared
import uniffi.packwing_mobile.EntryChecksum
import uniffi.packwing_mobile.EntryChecksums
import uniffi.packwing_mobile.FileVerdict
import uniffi.packwing_mobile.Job
import uniffi.packwing_mobile.JobListener
import uniffi.packwing_mobile.JobProgress
import uniffi.packwing_mobile.MobileException
import uniffi.packwing_mobile.PastedCompared
import uniffi.packwing_mobile.checksumKinds
import uniffi.packwing_mobile.checksumList
import uniffi.packwing_mobile.checksumListName
import uniffi.packwing_mobile.compareChecksum
import uniffi.packwing_mobile.comparePasted

/**
 * Checksums of the open archive: of the files picked in it, of all of its
 * files, or of its own file, as asked first (the desktop's Checksums…). Of
 * its files, each one's are made in one read of the archive, and listed as
 * `sha256sum` writes them, to copy or keep, where `sha256sum -c` checks what
 * is extracted with them. A checksum says a file is the one it was made of,
 * not who made it.
 */
@Composable
fun ChecksumsScreen(model: AppModel, screen: Screen.Checksums) {
    val c = tonal()
    val opened = model.opened ?: return
    var scope by remember(screen) { mutableStateOf(if (screen.picked.paths.isEmpty()) Scope.All else Scope.Picked) }
    var reading by remember(screen) { mutableStateOf(if (screen.atOnce) Scope.Picked else null) }
    // The file asked about alone, as its name; the archive's otherwise.
    val title = if (screen.atOnce) screen.picked.paths.firstOrNull()?.lastOrNull() else null

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
                Text(tr("Checksums"), fontSize = 18.sp, fontWeight = FontWeight.Bold, color = c.text)
                Text(
                    title ?: opened.info.name,
                    fontSize = 13.sp,
                    color = c.muted,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            ShownMenu(model)
        }

        when (reading) {
            Scope.File -> ArchiveFileSums(model, opened)
            Scope.Picked -> FilesSums(model, opened, screen.picked.paths)
            Scope.All -> FilesSums(model, opened, emptyList())
            null -> Choosing(opened, screen.picked, scope, { scope = it }) { reading = scope }
        }
    }
}

/** What Checksums reads: what is picked, all of its files, or its own file. */
private enum class Scope { Picked, All, File }

// MARK: What to read

@Composable
private fun Choosing(opened: Opened, picked: Picked, scope: Scope, choose: (Scope) -> Unit, read: () -> Unit) {
    val c = tonal()
    val files = { n: ULong -> trN(n, "{count} file", "{count} files", "count" to n.toString()) }
    val pickedTitle = if (picked.paths.size == 1) {
        picked.paths[0].lastOrNull() ?: ""
    } else {
        tr("{picked} selected", "picked" to picked.paths.size.toString())
    }
    val pickedDetail = if (picked.paths.isEmpty()) {
        tr("Select files or folders in it first.")
    } else {
        "${files(picked.files)} · ${sizeText(picked.bytes)}"
    }
    Column(Modifier.fillMaxSize()) {
        LazyColumn(
            modifier = Modifier.weight(1f),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            item { Heading(tr("What should it read?")) }
            item {
                Group {
                    Option(pickedTitle, pickedDetail, scope == Scope.Picked, enabled = picked.paths.isNotEmpty(), tag = "checksum-scope-picked") {
                        choose(Scope.Picked)
                    }
                    Option(
                        tr("All {counted}", "counted" to files(opened.info.files)),
                        sizeText(opened.info.unpacked),
                        scope == Scope.All,
                        tag = "checksum-scope-all",
                    ) { choose(Scope.All) }
                    Option(
                        tr("The archive itself"),
                        "${opened.info.name} · ${sizeText(opened.info.packed)}",
                        scope == Scope.File,
                        tag = "checksum-scope-file",
                    ) { choose(Scope.File) }
                }
            }
        }
        Button(
            onClick = read,
            shape = RoundedCornerShape(28.dp),
            colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
            modifier = Modifier
                .fillMaxWidth()
                .padding(start = 16.dp, end = 16.dp, bottom = 16.dp + navigationBottom())
                .height(56.dp)
                .testTag("checksum-read"),
        ) {
            Text(tr("Read it"), fontSize = 17.sp, fontWeight = FontWeight.Bold)
        }
    }
}

@Composable
private fun Option(title: String, detail: String, chosen: Boolean, enabled: Boolean = true, tag: String, choose: () -> Unit) {
    val c = tonal()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 64.dp)
            .let { if (enabled) it.clickable(onClick = choose) else it }
            .padding(horizontal = 16.dp, vertical = 10.dp)
            .testTag(tag),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Column(Modifier.weight(1f)) {
            Text(
                title,
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold,
                color = if (enabled) c.text else c.faint,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(detail, fontSize = 13.sp, color = c.muted, maxLines = 2)
        }
        Icon(
            if (chosen) Icons.Outlined.CheckCircle else Icons.Outlined.RadioButtonUnchecked,
            null,
            tint = if (chosen) c.accent else c.faint,
        )
    }
}

// MARK: The archive's own file

/** Its own checksums, all of its file read once for those shown. */
@Composable
private fun ArchiveFileSums(model: AppModel, opened: Opened) {
    val c = tonal()
    val work = rememberCoroutineScope()
    val shown = shownChecksums(model.checksumKeys)
    var made by remember { mutableStateOf<List<Checksum>>(emptyList()) }
    var making by remember { mutableStateOf<List<ChecksumKind>>(emptyList()) }
    var read by remember { mutableStateOf(0UL) }
    var job by remember { mutableStateOf<Job?>(null) }
    var problem by remember { mutableStateOf<String?>(null) }
    var pasted by remember { mutableStateOf("") }

    // Those not made yet, unless some are being made: the rest are made after them.
    fun make(kinds: List<ChecksumKind>) {
        if (job != null) return
        val wanted = kinds.filter { kind -> made.none { it.kind == kind } }
        if (wanted.isEmpty()) return
        problem = null
        making = wanted
        read = 0UL
        val now = Job(Told { bytes, _ -> read = bytes })
        job = now
        work.launch {
            val result = withContext(Dispatchers.IO) { runCatching { opened.archive.checksums(wanted, now) } }
            job = null
            making = emptyList()
            result.onSuccess { sums ->
                val all = made + sums
                made = allKinds.mapNotNull { row -> all.firstOrNull { it.kind == row.kind } }
                make(shownChecksums(model.checksumKeys).map { it.kind })
            }.onFailure { e ->
                if (e !is MobileException.Cancelled) {
                    problem = tr("The file could not be read: {why}", "why" to ((e as? MobileException)?.said ?: e.toString()))
                }
            }
        }
    }

    LaunchedEffect(model.checksumKeys) { make(shown.map { it.kind }) }
    DisposableEffect(Unit) { onDispose { job?.cancel() } }

    val rows = allKinds.filter { row -> shown.any { it.kind == row.kind } || made.any { it.kind == row.kind } }
    val compared = compareChecksum(pasted, opened.info.name, made, making)

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 24.dp + navigationBottom()),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        item { Heading(tr("Compare with")) }
        item {
            Group {
                PasteField(pasted, tr("Paste the checksum given with the file"), "checksum-pasted") { pasted = it }
                compared?.let { Said(it) { kinds -> make(kinds) } }
            }
        }
        item {
            Group {
                rows.forEach { row -> SumRow(row, made.firstOrNull { it.kind == row.kind }?.sum) }
            }
        }
        item {
            Column(Modifier.padding(horizontal = 6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                if (job != null) {
                    val size = opened.info.packed
                    LinearProgressIndicator(
                        progress = { if (size > 0UL) (minOf(read, size).toDouble() / size.toDouble()).toFloat() else 0f },
                        modifier = Modifier.fillMaxWidth(),
                        color = c.accent,
                        trackColor = c.field,
                    )
                    Small(
                        if (size > 0UL) {
                            tr("Reading… {size} of {size2}", "size" to sizeText(read), "size2" to sizeText(size))
                        } else {
                            tr("Reading… {size}", "size" to sizeText(read))
                        },
                    )
                }
                weakNote(rows)?.let { Small(it) }
                problem?.let { Note(it, Icons.Outlined.WarningAmber, c.warnText, c.warnBg) }
            }
        }
    }
}

/** What the checksum pasted says of the archive's file. */
@Composable
private fun Said(compared: Compared, make: (List<ChecksumKind>) -> Unit) {
    val c = tonal()
    when (compared) {
        Compared.NotOne -> Note(tr("There is no checksum in this."), Icons.Outlined.WarningAmber, c.warnText, c.warnBg)
        is Compared.Unknown -> Note(
            tr("This is not a checksum packwing makes: it has {digits} digits.", "digits" to compared.digits.toString()),
            Icons.Outlined.WarningAmber,
            c.warnText,
            c.warnBg,
        )
        is Compared.Matches -> Note(
            tr("It matches the file’s {name} checksum.", "name" to checksumNames(listOf(compared.kind))),
            Icons.Outlined.Verified,
            c.onTonal,
            c.tonal,
            "checksum-matches",
        )
        Compared.Waiting -> Small(tr("It is compared once the file is read."), Modifier.padding(16.dp))
        is Compared.Differs -> {
            if (compared.made.isNotEmpty()) {
                Note(
                    tr(
                        "It does not match the file’s {name} checksum: this is not the file it was made of, or the file changed on the way.",
                        "name" to checksumNames(compared.made),
                    ),
                    Icons.Outlined.Cancel,
                    c.badText,
                    c.badBg,
                    "checksum-differs",
                )
            }
            if (compared.notMade.isNotEmpty()) MakeIt(compared.notMade, make)
        }
    }
}

// MARK: The archive's files

/** The checksums of the archive's files, all or those picked, in one read of it. */
@Composable
private fun FilesSums(model: AppModel, opened: Opened, picked: List<List<String>>) {
    val c = tonal()
    val context = LocalContext.current
    val work = rememberCoroutineScope()
    val shown = shownChecksums(model.checksumKeys)
    var made by remember { mutableStateOf<EntryChecksums?>(null) }
    var read by remember { mutableStateOf(0UL) }
    var total by remember { mutableStateOf(0UL) }
    var job by remember { mutableStateOf<Job?>(null) }
    var problem by remember { mutableStateOf<String?>(null) }
    var saved by remember { mutableStateOf<List<String>>(emptyList()) }
    var copiedAll by remember { mutableStateOf(false) }
    var pasted by remember { mutableStateOf("") }
    var onlyMismatched by remember { mutableStateOf(false) }
    var looking by remember { mutableStateOf<EntryChecksum?>(null) }

    // Read again whenever the checksums shown change: all of them in one read.
    LaunchedEffect(model.checksumKeys) {
        job?.cancel()
        made = null
        problem = null
        saved = emptyList()
        read = 0UL
        val now = Job(Told { bytes, all ->
            read = bytes
            total = all
        })
        job = now
        val kinds = shownChecksums(model.checksumKeys).map { it.kind }
        val result = withContext(Dispatchers.IO) { runCatching { opened.archive.entryChecksums(picked, kinds, now) } }
        if (job !== now) return@LaunchedEffect
        job = null
        result.onSuccess { made = it }.onFailure { e ->
            if (e !is MobileException.Cancelled) {
                problem = tr("The file could not be read: {why}", "why" to ((e as? MobileException)?.said ?: e.toString()))
            }
        }
    }
    DisposableEffect(Unit) { onDispose { job?.cancel() } }

    val saveIn = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { folder ->
        val now = made ?: return@rememberLauncherForActivityResult
        if (folder == null) return@rememberLauncherForActivityResult
        Documents.keepAccess(context, folder, write = true)
        work.launch {
            val result = withContext(Dispatchers.IO) {
                runCatching { saveLists(context, folder, opened.info.name, now.files, shownChecksums(model.checksumKeys)) }
            }
            result.onSuccess { saved = it }.onFailure { e ->
                problem = tr("They could not be saved: {why}", "why" to (e.message ?: e.toString()))
            }
        }
    }

    looking?.let { file ->
        EntrySums(file, shown) { looking = null }
        return
    }

    val files = made
    val compared = files?.let { comparePasted(pasted, it.files, shown.map { row -> row.kind }) }
    val verdict = { i: Int ->
        when (compared) {
            is PastedCompared.Listed -> compared.verdicts.getOrNull(i)
            is PastedCompared.One -> compared.found.firstOrNull { it.index.toInt() == i }?.let { FileVerdict.Matches(it.kind) }
            else -> null
        }
    }

    Column(Modifier.fillMaxSize()) {
        LazyColumn(
            modifier = Modifier.weight(1f),
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            when {
                files != null && files.files.isNotEmpty() -> {
                    item { Heading(tr("Compare with")) }
                    item {
                        Group {
                            PasteField(
                                pasted,
                                tr("Paste the checksums given with the files, or one of them"),
                                "checksums-pasted",
                            ) { pasted = it }
                            compared?.let {
                                PastedSaid(it, files, shown, onlyMismatched, { on -> onlyMismatched = on }) { kinds ->
                                    show(model, shown, kinds)
                                }
                            }
                        }
                    }
                    if (files.files.size == 1) {
                        val file = files.files.first()
                        item { Heading("${file.name} · ${sizeText(file.size)}") }
                        item {
                            Group { shown.forEach { row -> SumRow(row, file.sums.firstOrNull { it.kind == row.kind }?.sum) } }
                        }
                    } else {
                        fileRows(files, onlyMismatched, verdict) { looking = it }
                        item { Small(tr("Tap a file to see its checksums."), Modifier.padding(horizontal = 6.dp)) }
                    }
                    item { Notes(files, shown, saved) }
                }
                files != null -> item {
                    Group { Small(tr("There are no files in it to read."), Modifier.padding(16.dp)) }
                }
                problem != null -> item { Note(problem!!, Icons.Outlined.WarningAmber, c.warnText, c.warnBg) }
                else -> item {
                    Column(Modifier.padding(6.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        LinearProgressIndicator(
                            progress = { if (total > 0UL) (minOf(read, total).toDouble() / total.toDouble()).toFloat() else 0f },
                            modifier = Modifier.fillMaxWidth(),
                            color = c.accent,
                            trackColor = c.field,
                        )
                        Small(
                            if (total > 0UL) {
                                tr("Reading… {size} of {size2}", "size" to sizeText(read), "size2" to sizeText(total))
                            } else {
                                tr("Reading… {size}", "size" to sizeText(read))
                            },
                        )
                    }
                }
            }
            if (files != null && problem != null) {
                item { Note(problem!!, Icons.Outlined.WarningAmber, c.warnText, c.warnBg) }
            }
        }

        if (files != null && files.files.isNotEmpty()) {
            Row(
                Modifier
                    .fillMaxWidth()
                    .padding(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 12.dp + navigationBottom()),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                OutlinedButton(
                    onClick = {
                        val kind = shown.first().kind
                        copy(context, checksumList(files.files, kind))
                        copiedAll = true
                        work.launch {
                            delay(2000)
                            copiedAll = false
                        }
                    },
                    shape = RoundedCornerShape(24.dp),
                    modifier = Modifier.weight(1f).height(52.dp).testTag("checksum-copy-all"),
                ) {
                    Icon(if (copiedAll) Icons.Outlined.Check else Icons.Outlined.ContentCopy, null, tint = c.accent)
                    Box(Modifier.size(8.dp))
                    Text(if (copiedAll) tr("Copied") else tr("Copy all"), color = c.accent, fontWeight = FontWeight.SemiBold)
                }
                Button(
                    onClick = { saveIn.launch(model.settings.destination) },
                    shape = RoundedCornerShape(24.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
                    modifier = Modifier.weight(1f).height(52.dp).testTag("checksum-save"),
                ) {
                    Icon(Icons.Outlined.Download, null)
                    Box(Modifier.size(8.dp))
                    Text(tr("Save in…"), fontWeight = FontWeight.SemiBold)
                }
            }
        }
    }
}

/** The files, each with what the list pasted says of it; tapped, all of its checksums. */
private fun LazyListScope.fileRows(
    files: EntryChecksums,
    onlyMismatched: Boolean,
    verdict: (Int) -> FileVerdict?,
    look: (EntryChecksum) -> Unit,
) {
    itemsIndexed(files.files, key = { _, file -> file.name }) { i, file ->
        val said = verdict(i)
        if (onlyMismatched && said is FileVerdict.Matches) return@itemsIndexed
        val c = tonal()
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(16.dp))
                .background(c.surface)
                .clickable { look(file) }
                .padding(horizontal = 16.dp, vertical = 12.dp)
                .testTag("checksum-file-$i"),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            said?.let { Mark(it, i) }
            Column(Modifier.weight(1f)) {
                Text(
                    file.name,
                    fontSize = 15.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = c.text,
                    maxLines = 1,
                    overflow = TextOverflow.MiddleEllipsis,
                )
                Text(sizeText(file.size), fontSize = 13.sp, color = c.muted)
            }
        }
    }
}

/** What the list says of a file, as a mark beside it. */
@Composable
private fun Mark(verdict: FileVerdict, i: Int) {
    val c = tonal()
    val (icon, tint, said) = when {
        verdict is FileVerdict.Matches -> Triple(Icons.Outlined.CheckCircle, c.accent, tr("Matches"))
        verdict is FileVerdict.Differs && verdict.made.isNotEmpty() -> Triple(Icons.Outlined.Cancel, c.badText, tr("Differs"))
        verdict is FileVerdict.NotListed -> Triple(Icons.Outlined.RemoveCircleOutline, c.faint, tr("The list does not name it."))
        else -> Triple(Icons.AutoMirrored.Outlined.HelpOutline, c.muted, null)
    }
    Icon(icon, said, tint = tint, modifier = Modifier.size(24.dp).testTag("checksum-mark-$i"))
}

/** What checksums pasted say of the archive's files. */
@Composable
private fun PastedSaid(
    compared: PastedCompared,
    made: EntryChecksums,
    shown: List<ChecksumKindRow>,
    onlyMismatched: Boolean,
    setOnly: (Boolean) -> Unit,
    show: (List<ChecksumKind>) -> Unit,
) {
    val c = tonal()
    when (compared) {
        PastedCompared.NotOne -> Note(tr("There is no checksum in this."), Icons.Outlined.WarningAmber, c.warnText, c.warnBg)
        is PastedCompared.One -> {
            val found = compared.found
            when {
                allKinds.none { DIGITS[it.kind] == compared.digits } -> Note(
                    tr("This is not a checksum packwing makes: it has {digits} digits.", "digits" to compared.digits.toString()),
                    Icons.Outlined.WarningAmber,
                    c.warnText,
                    c.warnBg,
                )
                found.size == 1 -> Note(
                    tr(
                        "It matches the {name} checksum of {file}.",
                        "name" to checksumNames(listOf(found[0].kind)),
                        "file" to made.files[found[0].index.toInt()].name,
                    ),
                    Icons.Outlined.Verified,
                    c.onTonal,
                    c.tonal,
                    "checksum-one-matches",
                )
                found.isNotEmpty() -> Note(
                    trN(
                        found.size,
                        "It matches {count} file: {names}",
                        "It matches {count} files: {names}",
                        "count" to found.size.toString(),
                        "names" to found.take(3).joinToString(", ") { made.files[it.index.toInt()].name },
                    ),
                    Icons.Outlined.Verified,
                    c.onTonal,
                    c.tonal,
                )
                shown.any { DIGITS[it.kind] == compared.digits } -> Note(
                    tr("No file in it has this checksum."),
                    Icons.Outlined.Cancel,
                    c.badText,
                    c.badBg,
                    "checksum-none-match",
                )
            }
            if (found.isEmpty() && compared.notMade.isNotEmpty()) MakeIt(compared.notMade, show)
        }
        is PastedCompared.Listed -> {
            val verdicts = compared.verdicts
            val matching = verdicts.count { it is FileVerdict.Matches }
            val differing = verdicts.count { it is FileVerdict.Differs && it.made.isNotEmpty() }
            val unlisted = verdicts.count { it is FileVerdict.NotListed }
            if (compared.notFound.isEmpty() && matching == verdicts.size) {
                Note(tr("Every file matches the list."), Icons.Outlined.Verified, c.onTonal, c.tonal, "checksum-all-match")
                return
            }
            val line = buildList {
                add(trN(matching, "{count} file matches", "{count} files match", "count" to matching.toString()))
                if (differing > 0) add(trN(differing, "{count} file differs", "{count} files differ", "count" to differing.toString()))
                if (unlisted > 0) {
                    add(trN(unlisted, "{count} file is not in the list", "{count} files are not in the list", "count" to unlisted.toString()))
                }
            }.joinToString(" · ")
            if (differing > 0) {
                Note(line, Icons.Outlined.Cancel, c.badText, c.badBg, "checksum-differs")
            } else {
                Small(line, Modifier.padding(horizontal = 16.dp, vertical = 10.dp))
            }
            if (compared.notFound.isNotEmpty()) {
                Small(
                    trN(
                        compared.notFound.size,
                        "{count} line names a file it does not hold: {names}",
                        "{count} lines name files it does not hold: {names}",
                        "count" to compared.notFound.size.toString(),
                        "names" to compared.notFound.take(3).joinToString(", "),
                    ),
                    Modifier.padding(horizontal = 16.dp, vertical = 6.dp),
                )
            }
            Row(
                Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(tr("Show only those that do not match"), Modifier.weight(1f), fontSize = 15.sp, color = c.text)
                Switch(checked = onlyMismatched, onCheckedChange = setOnly, colors = switchColors())
            }
        }
    }
}

/** What else is to know of what was read: damaged files, links, weak checksums, lists saved. */
@Composable
private fun Notes(made: EntryChecksums, shown: List<ChecksumKindRow>, saved: List<String>) {
    val c = tonal()
    Column(Modifier.padding(horizontal = 6.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (made.damaged.isNotEmpty()) {
            Text(
                trN(
                    made.damaged.size,
                    "{count} file is damaged, and has none: {names}",
                    "{count} files are damaged, and have none: {names}",
                    "count" to made.damaged.size.toString(),
                    "names" to made.damaged.joinToString(", "),
                ),
                fontSize = 13.sp,
                color = c.warnText,
            )
        }
        if (made.links > 0UL) Small(tr("Links have none of their own: a checking tool reads what they lead to."))
        weakNote(shown)?.let { Small(it) }
        if (saved.isNotEmpty()) {
            Note(
                tr(
                    "Saved as {files}. Where they are extracted, sha256sum -c and the like check them with it, without packwing.",
                    "files" to saved.joinToString(", "),
                ),
                Icons.Outlined.CheckCircle,
                c.onTonal,
                c.tonal,
                "checksum-saved",
            )
        }
    }
}

/** All of a file's checksums, each to copy. */
@Composable
private fun EntrySums(file: EntryChecksum, shown: List<ChecksumKindRow>, close: () -> Unit) {
    androidx.activity.compose.BackHandler(onBack = close)
    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 24.dp + navigationBottom()),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        item { Heading("${file.name} · ${sizeText(file.size)}") }
        item { Group { shown.forEach { row -> SumRow(row, file.sums.firstOrNull { it.kind == row.kind }?.sum) } } }
    }
}

// MARK: Shared by both

/** Every way to make a checksum, in the order they are shown. */
private val allKinds: List<ChecksumKindRow> by lazy { checksumKinds() }

/** How many hex digits each makes. */
private val DIGITS = mapOf(
    ChecksumKind.MD5 to 32U,
    ChecksumKind.SHA1 to 40U,
    ChecksumKind.SHA256 to 64U,
    ChecksumKind.BLAKE3 to 64U,
    ChecksumKind.SHA512 to 128U,
)

/** The checksums shown, by the keys `keys` keeps; never none. */
fun shownChecksums(keys: String): List<ChecksumKindRow> {
    val kept = keys.split(' ').filter { it.isNotEmpty() }.toSet()
    val chosen = allKinds.filter { it.key in kept }
    return chosen.ifEmpty { listOf(allKinds.first()) }
}

/** Their names, apart by slashes: names, not words to translate. */
fun checksumNames(kinds: List<ChecksumKind>): String =
    kinds.mapNotNull { kind -> allKinds.firstOrNull { it.kind == kind }?.name }.joinToString(" / ")

/** Shows `kinds` too, and so reads them. */
private fun show(model: AppModel, shown: List<ChecksumKindRow>, kinds: List<ChecksumKind>) {
    val keys = shown.map { it.key } + allKinds.filter { it.kind in kinds }.map { it.key }
    model.showChecksums(allKinds.map { it.key }.filter { it in keys }.joinToString(" "))
}

private fun weakNote(rows: List<ChecksumKindRow>): String? {
    val weak = rows.filter { it.weak }
    if (weak.isEmpty()) return null
    return tr(
        "A weak checksum ({names}) still tells a damaged file, but not one changed on purpose, which can be made to match.",
        "names" to checksumNames(weak.map { it.kind }),
    )
}

/** The checksums shown, each on or off; the last one stays. */
@Composable
private fun ShownMenu(model: AppModel) {
    val c = tonal()
    var open by remember { mutableStateOf(false) }
    Box {
        IconButton(onClick = { open = true }, Modifier.testTag("checksums-shown")) {
            Icon(Icons.Outlined.Tune, tr("Choose the checksums shown"), tint = c.text)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }, containerColor = c.surface) {
            val shown = shownChecksums(model.checksumKeys)
            allKinds.forEach { row ->
                val on = shown.any { it.kind == row.kind }
                DropdownMenuItem(
                    text = { Text(if (row.weak) tr("{name} (weak)", "name" to row.name) else row.name, color = c.text) },
                    leadingIcon = {
                        Checkbox(
                            checked = on,
                            onCheckedChange = null,
                            colors = CheckboxDefaults.colors(checkedColor = c.accent, uncheckedColor = c.outline),
                        )
                    },
                    onClick = {
                        val keys = shown.map { it.key }.toMutableList()
                        if (!on) keys += row.key else if (keys.size > 1) keys -= row.key
                        model.showChecksums(allKinds.map { it.key }.filter { it in keys }.joinToString(" "))
                    },
                    modifier = Modifier.testTag("checksum-kind-${row.key}"),
                )
            }
        }
    }
}

/** A checksum's row: its name, its digits, and Copy. */
@Composable
private fun SumRow(row: ChecksumKindRow, sum: String?) {
    val c = tonal()
    val context = LocalContext.current
    val work = rememberCoroutineScope()
    var copied by remember { mutableStateOf(false) }
    Column(
        Modifier.fillMaxWidth().padding(start = 16.dp, end = 4.dp, top = 8.dp, bottom = 12.dp),
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(row.name, fontSize = 15.sp, fontWeight = FontWeight.Bold, color = c.text)
            if (row.weak) {
                Text(
                    tr("weak"),
                    modifier = Modifier
                        .clip(RoundedCornerShape(10.dp))
                        .background(c.warnBg)
                        .padding(horizontal = 8.dp, vertical = 2.dp),
                    fontSize = 12.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = c.warnText,
                )
            }
            Box(Modifier.weight(1f))
            if (sum != null) {
                TextButton(onClick = {
                    copy(context, sum)
                    copied = true
                    work.launch {
                        delay(2000)
                        copied = false
                    }
                }) {
                    Icon(if (copied) Icons.Outlined.Check else Icons.Outlined.ContentCopy, null, tint = c.accent, modifier = Modifier.size(18.dp))
                    Box(Modifier.size(6.dp))
                    Text(if (copied) tr("Copied") else tr("Copy"), color = c.accent, fontWeight = FontWeight.SemiBold)
                }
            } else {
                Box(Modifier.height(48.dp))
            }
        }
        Text(
            sum ?: tr("Reading…"),
            modifier = Modifier.padding(end = 12.dp).testTag("checksum-${row.key}"),
            fontSize = 13.sp,
            fontFamily = FontFamily.Monospace,
            color = if (sum == null) c.muted else c.text,
        )
    }
}

/** Where checksums given elsewhere are pasted: typed, or Paste. */
@Composable
private fun PasteField(text: String, hint: String, tag: String, change: (String) -> Unit) {
    val c = tonal()
    val context = LocalContext.current
    OutlinedTextField(
        value = text,
        onValueChange = change,
        placeholder = { Text(hint, fontSize = 14.sp, color = c.faint) },
        minLines = 2,
        maxLines = 5,
        textStyle = androidx.compose.ui.text.TextStyle(fontSize = 13.sp, fontFamily = FontFamily.Monospace, color = c.text),
        trailingIcon = {
            IconButton(onClick = { pasteFrom(context)?.let(change) }) {
                Icon(Icons.Outlined.ContentPaste, tr("Paste"), tint = c.accent)
            }
        },
        shape = RoundedCornerShape(14.dp),
        colors = TextFieldDefaults.colors(
            focusedContainerColor = c.surface,
            unfocusedContainerColor = c.surface,
            focusedIndicatorColor = c.accent,
            unfocusedIndicatorColor = c.outline,
            cursorColor = c.accent,
            focusedTextColor = c.text,
            unfocusedTextColor = c.text,
        ),
        modifier = Modifier.fillMaxWidth().padding(12.dp).testTag(tag),
    )
}

@Composable
private fun MakeIt(kinds: List<ChecksumKind>, make: (List<ChecksumKind>) -> Unit) {
    val c = tonal()
    Row(
        Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text(
            tr("It may be a {name} checksum, which is not made yet.", "name" to checksumNames(kinds)),
            Modifier.weight(1f),
            fontSize = 14.sp,
            color = c.muted,
        )
        OutlinedButton(onClick = { make(kinds) }, shape = RoundedCornerShape(20.dp)) {
            Text(tr("Make it"), color = c.accent, fontWeight = FontWeight.SemiBold)
        }
    }
}

/** A word on a ground of its own: what matches, what does not, what is wrong. */
@Composable
private fun Note(text: String, icon: ImageVector, ink: Color, ground: Color, tag: String? = null) {
    Row(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .background(ground)
            .padding(14.dp)
            .let { if (tag != null) it.testTag(tag) else it },
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Icon(icon, null, tint = ink, modifier = Modifier.size(22.dp))
        Text(text, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = ink, lineHeight = 20.sp)
    }
}

@Composable
private fun Small(text: String, modifier: Modifier = Modifier) =
    Text(text, modifier, fontSize = 13.sp, lineHeight = 19.sp, color = tonal().muted)

@Composable
private fun Heading(text: String) = Text(
    text,
    modifier = Modifier.padding(start = 6.dp, top = 6.dp),
    fontSize = 14.sp,
    fontWeight = FontWeight.Bold,
    color = tonal().accent,
)

@Composable
private fun Group(content: @Composable () -> Unit) = Column(
    Modifier
        .fillMaxWidth()
        .clip(RoundedCornerShape(20.dp))
        .background(tonal().surface),
) { content() }

@Composable
private fun switchColors() = tonal().let { c ->
    SwitchDefaults.colors(
        checkedThumbColor = c.surface,
        checkedTrackColor = c.accent,
        uncheckedThumbColor = c.outline,
        uncheckedTrackColor = c.field,
        uncheckedBorderColor = c.outline,
    )
}

@Composable
private fun navigationBottom() = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()

/** Told by the core how much is read, and of how much, off the main thread. */
private class Told(val told: (ULong, ULong) -> Unit) : JobListener {
    private val main = android.os.Handler(android.os.Looper.getMainLooper())

    override fun progress(progress: JobProgress) {
        main.post { told(progress.bytes, progress.totalBytes) }
    }
}

private fun copy(context: Context, text: String) {
    context.getSystemService(ClipboardManager::class.java)
        .setPrimaryClip(ClipData.newPlainText("checksum", text))
}

private fun pasteFrom(context: Context): String? =
    context.getSystemService(ClipboardManager::class.java).primaryClip
        ?.takeIf { it.itemCount > 0 }
        ?.getItemAt(0)
        ?.coerceToText(context)
        ?.toString()

/**
 * A list for each checksum shown, in `folder`: `a.zip.contents.sha256`, as
 * `sha256sum` writes one. One there already is written over: it is the list
 * of the same archive.
 */
private fun saveLists(
    context: Context,
    folder: Uri,
    archive: String,
    files: List<EntryChecksum>,
    shown: List<ChecksumKindRow>,
): List<String> {
    val root = DocumentFile.fromTreeUri(context, folder) ?: error(tr("packwing can no longer reach that folder."))
    return shown.map { row ->
        val name = checksumListName(archive, row.kind)
        val unwritten = { tr("packwing could not write {name}.", "name" to name) }
        val file = root.findFile(name) ?: root.createFile(Documents.saveMime(name), name) ?: error(unwritten())
        context.contentResolver.openOutputStream(file.uri, "wt")?.use { out ->
            out.write(checksumList(files, row.kind).toByteArray())
        } ?: error(unwritten())
        file.name ?: name
    }
}
