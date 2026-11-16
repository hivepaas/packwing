package com.hivepaas.packwing.ui

import android.content.Context
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.Description
import androidx.compose.material.icons.outlined.Draw
import androidx.compose.material.icons.outlined.GppBad
import androidx.compose.material.icons.outlined.GppMaybe
import androidx.compose.material.icons.outlined.Verified
import androidx.compose.material.icons.outlined.WarningAmber
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.material3.rememberModalBottomSheetState
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
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Documents
import com.hivepaas.packwing.Sheet
import com.hivepaas.packwing.said
import com.hivepaas.packwing.tr
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.packwing_mobile.Job
import uniffi.packwing_mobile.KeyRead
import uniffi.packwing_mobile.MobileException
import uniffi.packwing_mobile.SignatureState
import uniffi.packwing_mobile.readKey
import java.io.File
import java.util.UUID

/** Where checking a signature is. */
private sealed interface Look {
    /** None beside it to reach: one is chosen. */
    data object None : Look
    data object Checking : Look
    data class Done(val state: SignatureState) : Look
}

/**
 * Whether the open archive is what a key the user trusts signed: by its
 * signature beside it, `name.sig`, where packwing can reach that, or one
 * chosen with the document picker. A file Android handed over alone has no
 * folder packwing may look in, so its signature is chosen.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SignatureSheet(model: AppModel) {
    val c = tonal()
    val context = LocalContext.current
    val work = rememberCoroutineScope()
    val opened = model.opened ?: return
    var look by remember { mutableStateOf<Look>(Look.Checking) }
    // The signature last checked, to check it again once its key is trusted.
    var signature by remember { mutableStateOf<Pair<String, File>?>(null) }
    var job by remember { mutableStateOf<Job?>(null) }
    var naming by remember { mutableStateOf<KeyRead?>(null) }

    fun check(called: String, file: File) {
        job?.cancel()
        signature = called to file
        look = Look.Checking
        val now = Job.quiet()
        job = now
        val trusted = model.trustedKeys.map { it.key.line }
        work.launch {
            val result = withContext(Dispatchers.IO) {
                runCatching { opened.archive.checkSignature(file.path, trusted, now) }
            }
            if (job !== now) return@launch
            job = null
            result.onSuccess { look = Look.Done(it) }.onFailure { e ->
                if (e is MobileException.Cancelled) return@onFailure
                look = Look.Done(SignatureState.Failed((e as? MobileException)?.said ?: e.toString()))
            }
        }
    }

    val choose = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri == null) return@rememberLauncherForActivityResult
        work.launch {
            val copied = withContext(Dispatchers.IO) { runCatching { copyIn(context, uri) } }
            copied.onSuccess { (called, file) -> check(called, file) }.onFailure {
                look = Look.Done(SignatureState.Unreadable(it.message ?: it.toString()))
            }
        }
    }

    LaunchedEffect(Unit) {
        val beside = beside(opened.uri)
        if (beside != null) check(beside.name, beside) else look = Look.None
    }
    DisposableEffect(Unit) { onDispose { job?.cancel() } }

    ModalBottomSheet(
        onDismissRequest = { model.sheet = null },
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        containerColor = c.page,
        contentColor = c.text,
    ) {
        Column(
            Modifier
                .verticalScroll(rememberScrollState())
                .padding(start = 16.dp, end = 16.dp, bottom = 28.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            Text(
                tr("Signature"),
                modifier = Modifier.padding(start = 4.dp),
                fontSize = 22.sp,
                fontWeight = FontWeight.Bold,
                color = c.text,
            )
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(24.dp))
                    .background(c.tonal)
                    .padding(14.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                KindTile(opened.info.name, FileKind.of(opened.info.name))
                Column {
                    Text(
                        opened.info.name,
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Bold,
                        color = c.onTonal,
                        maxLines = 1,
                        overflow = TextOverflow.MiddleEllipsis,
                    )
                    signature?.let { (called, _) ->
                        Text(called, fontSize = 13.sp, color = c.onTonal, maxLines = 1, overflow = TextOverflow.MiddleEllipsis)
                    }
                }
            }

            when (val now = look) {
                Look.None -> Said(tr("There is no signature beside it."), Icons.Outlined.Draw, c.muted, c.surface)
                Look.Checking -> Row(
                    Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(16.dp))
                        .background(c.surface)
                        .padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    CircularProgressIndicator(Modifier.size(22.dp), color = c.accent, strokeWidth = 3.dp)
                    Text(tr("Checking its signature…"), fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = c.text)
                }
                is Look.Done -> when (val state = now.state) {
                    is SignatureState.Good -> Said(
                        tr("Signed by {signer}", "signer" to model.signer(state.fingerprint)),
                        Icons.Outlined.Verified,
                        c.onAccent,
                        c.accent,
                        detail = state.fingerprint,
                        code = true,
                        tag = "signature-good",
                    )
                    is SignatureState.Stranger -> {
                        Said(
                            tr("Signed by a key you do not trust: {fingerprint}", "fingerprint" to state.fingerprint),
                            Icons.Outlined.GppMaybe,
                            c.warnText,
                            c.warnBg,
                            tag = "signature-stranger",
                        )
                        runCatching { readKey(state.key) }.getOrNull()?.let { key ->
                            Button(
                                onClick = { naming = key },
                                shape = RoundedCornerShape(24.dp),
                                colors = ButtonDefaults.buttonColors(containerColor = c.tonal, contentColor = c.onTonal),
                                modifier = Modifier.fillMaxWidth().height(52.dp).testTag("trust-key"),
                            ) {
                                Text(tr("Trust this key"), fontWeight = FontWeight.SemiBold)
                            }
                        }
                    }
                    is SignatureState.Mismatch -> Said(
                        tr("It does not match its signature"),
                        Icons.Outlined.GppBad,
                        c.badText,
                        c.badBg,
                        detail = model.signer(state.fingerprint),
                        tag = "signature-mismatch",
                    )
                    is SignatureState.Unreadable -> Said(
                        tr("Its signature cannot be read"),
                        Icons.Outlined.WarningAmber,
                        c.warnText,
                        c.warnBg,
                        detail = state.reason,
                    )
                    is SignatureState.Failed -> Said(
                        tr("Its signature could not be checked"),
                        Icons.Outlined.WarningAmber,
                        c.warnText,
                        c.warnBg,
                        detail = state.reason,
                    )
                }
            }

            TextButton(onClick = { choose.launch(arrayOf("*/*")) }, modifier = Modifier.fillMaxWidth()) {
                Icon(Icons.Outlined.Description, null, tint = c.accent)
                Box(Modifier.size(8.dp))
                Text(tr("Choose its signature…"), color = c.accent, fontWeight = FontWeight.SemiBold)
            }
            Text(
                tr("A signature says who signed a file only by a key you trust: Settings › Keys you trust. One by any other key proves nothing, whatever it matches."),
                modifier = Modifier.padding(horizontal = 8.dp),
                fontSize = 13.sp,
                lineHeight = 19.sp,
                color = c.muted,
            )
        }
    }

    naming?.let { key ->
        NameKey(key, close = { naming = null }) { name ->
            model.trust(key, name)
            naming = null
            signature?.let { (called, file) -> check(called, file) }
        }
    }
}

/** What the signature says, on a ground of its standing's colours, with a fingerprint or why under it. */
@Composable
private fun Said(
    text: String,
    icon: ImageVector,
    ink: Color,
    ground: Color,
    detail: String? = null,
    code: Boolean = false,
    tag: String? = null,
) {
    Row(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .background(ground)
            .padding(16.dp)
            .let { if (tag != null) it.testTag(tag) else it },
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Icon(icon, null, tint = ink, modifier = Modifier.size(24.dp))
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(text, fontSize = 15.sp, fontWeight = FontWeight.Bold, color = ink)
            detail?.let {
                Text(
                    it,
                    fontSize = 13.sp,
                    fontFamily = if (code) FontFamily.Monospace else null,
                    color = ink.copy(alpha = 0.85f),
                )
            }
        }
    }
}

/** The name a key is trusted under: the start of its fingerprint, left empty. */
@Composable
private fun NameKey(key: KeyRead, close: () -> Unit, trust: (String) -> Unit) {
    val c = tonal()
    var name by remember { mutableStateOf("") }
    AlertDialog(
        onDismissRequest = close,
        containerColor = c.surface,
        title = { Text(tr("Trust this key"), color = c.text) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text(key.fingerprint, fontSize = 13.sp, fontFamily = FontFamily.Monospace, color = c.muted)
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text(tr("Name")) },
                    singleLine = true,
                    colors = fieldColors(),
                    modifier = Modifier.fillMaxWidth().testTag("key-name"),
                )
            }
        },
        confirmButton = {
            TextButton(onClick = { trust(name) }, modifier = Modifier.testTag("key-trust")) {
                Text(tr("Trust this key"), color = c.accent, fontWeight = FontWeight.SemiBold)
            }
        },
        dismissButton = {
            TextButton(onClick = close) { Text(tr("Cancel"), color = c.accent) }
        },
    )
}

/** Settings' keys the user trusts: a signature by one says who signed a file. */
@Composable
fun TrustedKeys(model: AppModel) {
    val c = tonal()
    Column(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(20.dp))
            .background(c.surface),
    ) {
        model.trustedKeys.forEach { trusted ->
            Row(
                Modifier.fillMaxWidth().heightIn(min = 60.dp).padding(start = 16.dp, end = 4.dp, top = 8.dp, bottom = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(Modifier.weight(1f)) {
                    Text(trusted.key.name, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = c.text)
                    Text(
                        trusted.fingerprint,
                        fontSize = 12.sp,
                        fontFamily = FontFamily.Monospace,
                        color = c.muted,
                        maxLines = 1,
                        overflow = TextOverflow.MiddleEllipsis,
                    )
                }
                IconButton(onClick = { model.distrust(trusted.key.line) }) {
                    Icon(Icons.Outlined.Delete, tr("Delete"), tint = c.muted)
                }
            }
            HorizontalDivider(color = c.field)
        }
        Row(
            Modifier
                .fillMaxWidth()
                .heightIn(min = 56.dp)
                .clickable { model.sheet = Sheet.AddKey }
                .padding(horizontal = 16.dp)
                .testTag("add-key"),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Icon(Icons.Outlined.Add, null, tint = c.accent)
            Text(tr("Paste a public key"), fontSize = 16.sp, color = c.accent, fontWeight = FontWeight.SemiBold)
        }
    }
}

/** A public key to trust, pasted or from a file, and its name. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddKeySheet(model: AppModel) {
    val c = tonal()
    val context = LocalContext.current
    val work = rememberCoroutineScope()
    var text by remember { mutableStateOf("") }
    var name by remember { mutableStateOf("") }
    val read = runCatching { readKey(text) }
    val key = read.getOrNull()

    val choose = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri == null) return@rememberLauncherForActivityResult
        work.launch {
            // A key file is a line; anything much bigger is not one.
            withContext(Dispatchers.IO) {
                runCatching {
                    context.contentResolver.openInputStream(uri)?.use { it.upTo(MOST_KEY_FILE) }
                }.getOrNull()
            }?.let { text = it.decodeToString() }
        }
    }

    ModalBottomSheet(
        onDismissRequest = { model.sheet = null },
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        containerColor = c.page,
        contentColor = c.text,
    ) {
        Column(
            Modifier
                .verticalScroll(rememberScrollState())
                .padding(start = 16.dp, end = 16.dp, bottom = 28.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            Text(
                tr("Paste a public key"),
                modifier = Modifier.padding(start = 4.dp),
                fontSize = 22.sp,
                fontWeight = FontWeight.Bold,
                color = c.text,
            )
            OutlinedTextField(
                value = text,
                onValueChange = { text = it },
                label = { Text(tr("Public key")) },
                placeholder = { Text(tr("Type a public key, or choose a file of them."), color = c.faint) },
                minLines = 3,
                maxLines = 6,
                textStyle = androidx.compose.ui.text.TextStyle(fontSize = 13.sp, fontFamily = FontFamily.Monospace),
                supportingText = {
                    when {
                        key != null -> Text(key.fingerprint, fontFamily = FontFamily.Monospace, color = c.muted)
                        text.isNotBlank() -> Text(
                            (read.exceptionOrNull() as? MobileException)?.said ?: "",
                            color = c.badText,
                        )
                    }
                },
                colors = fieldColors(),
                modifier = Modifier.fillMaxWidth().testTag("key-text"),
            )
            TextButton(onClick = { choose.launch(arrayOf("*/*")) }) {
                Icon(Icons.Outlined.Description, null, tint = c.accent)
                Box(Modifier.size(8.dp))
                Text(tr("Choose a file…"), color = c.accent, fontWeight = FontWeight.SemiBold)
            }
            OutlinedTextField(
                value = name,
                onValueChange = { name = it },
                label = { Text(tr("Name")) },
                singleLine = true,
                colors = fieldColors(),
                modifier = Modifier.fillMaxWidth().testTag("key-name"),
            )
            Button(
                onClick = {
                    key?.let {
                        model.trust(it, name)
                        model.sheet = null
                    }
                },
                enabled = key != null,
                shape = RoundedCornerShape(28.dp),
                colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
                modifier = Modifier.fillMaxWidth().height(56.dp).testTag("key-trust"),
            ) {
                Text(tr("Trust this key"), fontSize = 17.sp, fontWeight = FontWeight.Bold)
            }
        }
    }
}

@Composable
private fun fieldColors() = tonal().let { c ->
    TextFieldDefaults.colors(
        focusedContainerColor = c.page,
        unfocusedContainerColor = c.page,
        focusedIndicatorColor = c.accent,
        unfocusedIndicatorColor = c.outline,
        focusedLabelColor = c.accent,
        unfocusedLabelColor = c.muted,
        cursorColor = c.accent,
        focusedTextColor = c.text,
        unfocusedTextColor = c.text,
    )
}

/** The most of a key file, or of a signature, read: far more than either is. */
private const val MOST_KEY_FILE = 64 * 1024

/**
 * The signature beside the archive at `uri`, `name.sig`, where packwing can
 * look beside it: a file it reaches by its path. A document Android handed
 * over alone has no folder packwing may read.
 */
private fun beside(uri: Uri): File? {
    if (uri.scheme != "file") return null
    val path = uri.path ?: return null
    return File("$path.sig").takeIf { it.canRead() }
}

/**
 * A signature chosen, copied into the app's own folder: the core reads one
 * by its path, which a `content://` file has none of. Its name and the copy.
 */
private fun copyIn(context: Context, uri: Uri): Pair<String, File> {
    val name = Documents.picked(context, uri).name
    // Only the last one is wanted: the one before is checked no more.
    val kept = Documents.cache(context, "signatures").apply { deleteRecursively() }
    val into = File(kept, UUID.randomUUID().toString()).apply { mkdirs() }
    val file = File(into, name)
    context.contentResolver.openInputStream(uri)?.use { from ->
        file.writeBytes(from.upTo(MOST_KEY_FILE))
    } ?: error(tr("packwing could not read {name}.", "name" to name))
    return name to file
}

/** At most `most` bytes of what is left: `readNBytes` is Android 13's. */
internal fun java.io.InputStream.upTo(most: Int): ByteArray {
    val read = java.io.ByteArrayOutputStream()
    val buffer = ByteArray(8192)
    while (read.size() < most) {
        val got = read(buffer, 0, minOf(buffer.size, most - read.size()))
        if (got < 0) break
        read.write(buffer, 0, got)
    }
    return read.toByteArray()
}
