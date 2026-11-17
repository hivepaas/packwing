package com.hivepaas.packwing.ui

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Intent
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
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.Download
import androidx.compose.material.icons.outlined.Draw
import androidx.compose.material.icons.outlined.ExpandMore
import androidx.compose.material.icons.outlined.Key
import androidx.compose.material.icons.outlined.MoreVert
import androidx.compose.material.icons.outlined.Share
import androidx.compose.material.icons.outlined.Verified
import androidx.compose.material.icons.outlined.WarningAmber
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
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
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Screen
import com.hivepaas.packwing.Stored
import com.hivepaas.packwing.said
import com.hivepaas.packwing.tr
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.packwing_mobile.Job
import uniffi.packwing_mobile.MobileException
import uniffi.packwing_mobile.SigningKeyRow
import uniffi.packwing_mobile.exportSigningKey
import uniffi.packwing_mobile.importSigningKey
import uniffi.packwing_mobile.makeSigningKey
import uniffi.packwing_mobile.maySign
import uniffi.packwing_mobile.readKey
import uniffi.packwing_mobile.removeSigningKey
import uniffi.packwing_mobile.signingKeys
import java.io.File

// Signing files, as the desktop signs them, with keys kept in Android's
// keychain (the vault): given as a key file (an SSH Ed25519 key, or one
// packwing made), or made here. The signature is `name.sig`, beside the file
// where packwing may write there (a storage, a file of the app's own); a file
// Android handed over has no folder packwing may write in, so its signature
// is saved where the user says.

/** The keys kept, read again: none if the vault cannot be reached. */
private fun keptSigningKeys(): List<SigningKeyRow> = runCatching { signingKeys() }.getOrDefault(emptyList())

/** Keeps a key file's text as a signing key: what ImportKeySheet is given to keep one. */
private val keepSigningKey: (String, String, String?) -> String = { name, text, passphrase ->
    importSigningKey(name, text.toByteArray(), passphrase).id
}

/** Where a signature is while it is made. */
private sealed interface SignState {
    data object Ready : SignState
    data object Signing : SignState

    /** Signed: the signature is where it says. */
    data class Done(val name: String) : SignState
    data class Failed(val why: String) : SignState
}

/**
 * Signs the open archive's file with a key kept: its signature beside it
 * where packwing may write there, or saved where the user says.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SignSheet(model: AppModel) {
    val c = tonal()
    val context = LocalContext.current
    val work = rememberCoroutineScope()
    val opened = model.opened ?: return
    val name = opened.info.name
    var keys by remember { mutableStateOf(keptSigningKeys()) }
    var chosen by remember { mutableStateOf(model.settings.signingKey) }
    var state by remember { mutableStateOf<SignState>(SignState.Ready) }
    var making by remember { mutableStateOf(false) }
    var importing by remember { mutableStateOf(false) }
    var text by remember { mutableStateOf<String?>(null) }
    val key = keys.firstOrNull { it.id == chosen } ?: keys.firstOrNull()
    // On a storage, the core writes it beside the file. Of the phone's, the
    // core reads the file through Android's document, and makes the
    // signature; it goes beside the file where packwing reaches that by its
    // path and may write there, and where the user says otherwise.
    val onStorage = opened.uri.scheme == Stored.SCHEME
    val besideFile = opened.uri.takeIf { it.scheme == "file" }?.path
        ?.let { File("$it.sig") }?.takeIf { it.parentFile?.canWrite() == true }

    val saveIn = rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("application/octet-stream")) { uri ->
        val signature = text
        text = null
        if (uri == null || signature == null) return@rememberLauncherForActivityResult
        work.launch {
            val written = withContext(Dispatchers.IO) {
                runCatching { context.contentResolver.openOutputStream(uri, "wt")!!.use { it.write(signature.toByteArray()) } }
            }
            state = written.fold(
                onSuccess = { SignState.Done(com.hivepaas.packwing.Documents.picked(context, uri).name) },
                onFailure = { SignState.Failed(it.message ?: it.toString()) },
            )
        }
    }

    fun sign() {
        if (!model.store.holds) return
        val with = key ?: return
        chosen = with.id
        model.settings.signingKey = with.id
        state = SignState.Signing
        val job = Job.quiet()
        work.launch {
            val result = withContext(Dispatchers.IO) {
                runCatching {
                    if (onStorage) {
                        opened.archive.signBeside(with.id, job)
                        null
                    } else {
                        val signature = opened.archive.signature(with.id, job)
                        // Written beside it, in place of one there; or handed on, to be saved.
                        if (besideFile != null) {
                            besideFile.writeText(signature)
                            null
                        } else {
                            signature
                        }
                    }
                }
            }
            result.onSuccess { signature ->
                // Whoever holds the key trusts it: what they sign shows as signed by them.
                runCatching { readKey(with.public) }.getOrNull()?.let { model.trust(it, with.name) }
                if (signature != null) {
                    text = signature
                    state = SignState.Ready
                    saveIn.launch("$name.sig")
                } else {
                    state = SignState.Done("$name.sig")
                }
            }.onFailure { e ->
                state = if (e is MobileException.Cancelled) SignState.Ready else SignState.Failed((e as? MobileException)?.said ?: e.toString())
            }
        }
    }

    ModalBottomSheet(
        onDismissRequest = { model.sheet = null },
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        containerColor = c.page,
        contentColor = c.text,
    ) {
        Column(
            Modifier.verticalScroll(rememberScrollState()).padding(start = 16.dp, end = 16.dp, bottom = 28.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            Text(
                tr("Sign {name}", "name" to name),
                Modifier.padding(start = 4.dp),
                fontSize = 22.sp,
                fontWeight = FontWeight.Bold,
                color = c.text,
                maxLines = 2,
                overflow = TextOverflow.MiddleEllipsis,
            )
            if (!model.store.holds) {
                // Checking one is free: the Signature sheet, not this one.
                Column(
                    Modifier.fillMaxWidth().clip(RoundedCornerShape(20.dp)).background(c.surface).padding(16.dp).testTag("sign-pro"),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        Icon(Icons.Outlined.Draw, null, tint = c.accent)
                        Text(
                            tr("Signing takes packwing Pro: Settings › License. Checking a signature is free."),
                            fontSize = 15.sp,
                            lineHeight = 20.sp,
                            color = c.text,
                        )
                    }
                    TextButton(
                        onClick = {
                            model.sheet = null
                            model.go(Screen.Settings)
                        },
                        modifier = Modifier.align(Alignment.End).testTag("sign-pro-settings"),
                    ) {
                        Text(tr("Settings"), color = c.accent, fontWeight = FontWeight.SemiBold)
                    }
                }
            } else if (!maySign(name)) {
                Warn(tr("This is one part of a file cut in several. A signature covers one file, and would seem to cover them all, so packwing does not sign parts."))
            } else {
                Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(20.dp)).background(c.surface)) {
                    if (key == null) {
                        Text(tr("No key chosen"), Modifier.padding(16.dp), fontSize = 16.sp, color = c.muted)
                    } else {
                        KeyChoice(keys, key) { chosen = it.id }
                    }
                    HorizontalDivider(color = c.field)
                    Action(tr("New key…"), Icons.Outlined.Add, "sign-new-key") { making = true }
                    HorizontalDivider(color = c.field)
                    Action(tr("Add a key file…"), Icons.Outlined.Key, "sign-add-key") { importing = true }
                }
                Text(
                    tr("Anyone can check it, with the key’s public half"),
                    Modifier.padding(horizontal = 8.dp),
                    fontSize = 13.sp,
                    color = c.muted,
                )
                when (val now = state) {
                    SignState.Signing -> Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        CircularProgressIndicator(Modifier.size(22.dp), color = c.accent, strokeWidth = 3.dp)
                        Text(tr("Signing…"), fontSize = 15.sp, color = c.text)
                    }
                    is SignState.Done -> Row(
                        Modifier.fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(c.tonal).padding(14.dp).testTag("signed"),
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Icon(Icons.Outlined.Verified, null, tint = c.onTonal)
                        Text(tr("Signed: {name}", "name" to now.name), fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = c.onTonal)
                    }
                    is SignState.Failed -> Warn(now.why)
                    SignState.Ready -> Unit
                }
                Button(
                    onClick = { sign() },
                    enabled = key != null && state != SignState.Signing,
                    shape = RoundedCornerShape(28.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
                    modifier = Modifier.fillMaxWidth().height(56.dp).testTag("sign-confirm"),
                ) {
                    Text(tr("Sign"), fontSize = 17.sp, fontWeight = FontWeight.Bold)
                }
            }
        }
    }

    if (making) {
        NameDialog(tr("New key…"), null, tr("Make the key…"), close = { making = false }) { called ->
            making = false
            runCatching { makeSigningKey(called) }
                .onSuccess { chosen = it.id }
                .onFailure { e -> state = SignState.Failed((e as? MobileException)?.said ?: e.toString()) }
            keys = keptSigningKeys()
        }
    }
    if (importing) {
        ImportKeySheet(close = { importing = false }, keep = keepSigningKey) { id ->
            keys = keptSigningKeys()
            chosen = id
        }
    }
}

/** The key to sign with, chosen from those kept. */
@Composable
private fun KeyChoice(keys: List<SigningKeyRow>, key: SigningKeyRow, choose: (SigningKeyRow) -> Unit) {
    val c = tonal()
    var open by remember { mutableStateOf(false) }
    Box {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 60.dp).clickable { open = true }.padding(horizontal = 16.dp, vertical = 8.dp).testTag("sign-key"),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            KeyLabel(key, Modifier.weight(1f))
            Icon(Icons.Outlined.ExpandMore, null, tint = c.muted)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }, containerColor = c.surface) {
            keys.forEach { row ->
                DropdownMenuItem(
                    text = { KeyLabel(row) },
                    onClick = {
                        open = false
                        choose(row)
                    },
                )
            }
        }
    }
}

@Composable
private fun KeyLabel(key: SigningKeyRow, modifier: Modifier = Modifier) {
    val c = tonal()
    Column(modifier) {
        Text(key.name, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = c.text)
        Text(
            key.fingerprint,
            fontSize = 12.sp,
            fontFamily = FontFamily.Monospace,
            color = c.muted,
            maxLines = 1,
            overflow = TextOverflow.MiddleEllipsis,
        )
    }
}

@Composable
private fun Action(label: String, icon: androidx.compose.ui.graphics.vector.ImageVector, tag: String, act: () -> Unit) {
    val c = tonal()
    Row(
        Modifier.fillMaxWidth().heightIn(min = 52.dp).clickable(onClick = act).padding(horizontal = 16.dp).testTag(tag),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Icon(icon, null, tint = c.accent)
        Text(label, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = c.accent)
    }
}

@Composable
private fun Warn(text: String) {
    val c = tonal()
    Row(
        Modifier.fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(c.warnBg).padding(14.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Icon(Icons.Outlined.WarningAmber, null, tint = c.warnText)
        Text(text, fontSize = 14.sp, color = c.warnText)
    }
}

// MARK: Settings › Signing keys

/**
 * Settings' signing keys: those kept, their public halves shared or copied,
 * a copy of one made here saved, removed; a new one made, or one added.
 */
@Composable
fun SigningKeys(model: AppModel) {
    val c = tonal()
    val context = LocalContext.current
    var keys by remember { mutableStateOf(keptSigningKeys()) }
    var making by remember { mutableStateOf(false) }
    var importing by remember { mutableStateOf(false) }
    var copying by remember { mutableStateOf<SigningKeyRow?>(null) }
    var removing by remember { mutableStateOf<SigningKeyRow?>(null) }
    var problem by remember { mutableStateOf<String?>(null) }

    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(20.dp)).background(c.surface)) {
        keys.forEach { key ->
            Row(
                Modifier.fillMaxWidth().heightIn(min = 60.dp).padding(start = 16.dp, end = 4.dp, top = 8.dp, bottom = 8.dp).testTag("signing-key"),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                KeyLabel(key, Modifier.weight(1f))
                var open by remember { mutableStateOf(false) }
                Box {
                    IconButton(onClick = { open = true }, Modifier.testTag("signing-key-menu-${key.name}")) {
                        Icon(Icons.Outlined.MoreVert, tr("More"), tint = c.muted)
                    }
                    DropdownMenu(expanded = open, onDismissRequest = { open = false }, containerColor = c.surface) {
                        DropdownMenuItem(
                            text = { Text(tr("Share the public key"), color = c.text) },
                            leadingIcon = { Icon(Icons.Outlined.Share, null, tint = c.muted) },
                            onClick = {
                                open = false
                                val send = Intent(Intent.ACTION_SEND).setType("text/plain").putExtra(Intent.EXTRA_TEXT, key.public)
                                context.startActivity(Intent.createChooser(send, null))
                            },
                        )
                        DropdownMenuItem(
                            text = { Text(tr("Copy"), color = c.text) },
                            leadingIcon = { Icon(Icons.Outlined.ContentCopy, null, tint = c.muted) },
                            onClick = {
                                open = false
                                context.getSystemService(ClipboardManager::class.java)
                                    .setPrimaryClip(ClipData.newPlainText("public key", key.public))
                            },
                        )
                        if (key.madeHere) {
                            DropdownMenuItem(
                                text = { Text(tr("Save a copy…"), color = c.text) },
                                leadingIcon = { Icon(Icons.Outlined.Download, null, tint = c.muted) },
                                onClick = {
                                    open = false
                                    copying = key
                                },
                            )
                        }
                        DropdownMenuItem(
                            text = { Text(tr("Remove"), color = c.badText) },
                            leadingIcon = { Icon(Icons.Outlined.Delete, null, tint = c.badText) },
                            onClick = {
                                open = false
                                removing = key
                            },
                        )
                    }
                }
            }
            HorizontalDivider(color = c.field)
        }
        Action(tr("New key…"), Icons.Outlined.Add, "signing-new-key") { making = true }
        HorizontalDivider(color = c.field)
        Action(tr("Add a key file…"), Icons.Outlined.Key, "signing-add-key") { importing = true }
    }
    problem?.let { Warn(it) }

    if (making) {
        NameDialog(tr("New key…"), null, tr("Make the key…"), close = { making = false }) { called ->
            making = false
            problem = runCatching { makeSigningKey(called) }.exceptionOrNull()?.let { (it as? MobileException)?.said ?: it.toString() }
            keys = keptSigningKeys()
        }
    }
    if (importing) {
        ImportKeySheet(close = { importing = false }, keep = keepSigningKey) { keys = keptSigningKeys() }
    }
    copying?.let { key -> SaveKeyCopy(key) { copying = null } }
    removing?.let { key ->
        AlertDialog(
            onDismissRequest = { removing = null },
            containerColor = c.surface,
            title = { Text(tr("Remove “{name}”?", "name" to key.name), color = c.text) },
            text = {
                if (key.madeHere) {
                    Text(
                        tr("This key was made here, and this device keeps the only copy of it unless one was saved. What it signed can still be checked, but nothing more can be signed with it."),
                        color = c.muted,
                    )
                }
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        removing = null
                        runCatching { removeSigningKey(key.id) }
                        keys = keptSigningKeys()
                    },
                    modifier = Modifier.testTag("signing-remove"),
                ) { Text(tr("Remove"), color = c.badText, fontWeight = FontWeight.SemiBold) }
            },
            dismissButton = { TextButton(onClick = { removing = null }) { Text(tr("Cancel"), color = c.accent) } },
        )
    }
}

/**
 * A copy of a key made here, encrypted with a passphrase typed twice, saved
 * where the user says: the desktop and `pkw` sign with it too.
 */
@Composable
private fun SaveKeyCopy(key: SigningKeyRow, close: () -> Unit) {
    val c = tonal()
    val context = LocalContext.current
    val work = rememberCoroutineScope()
    var passphrase by remember { mutableStateOf("") }
    var again by remember { mutableStateOf("") }
    var problem by remember { mutableStateOf<String?>(null) }
    var bytes by remember { mutableStateOf<ByteArray?>(null) }
    val save = rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("application/octet-stream")) { uri ->
        val copy = bytes
        bytes = null
        if (uri == null || copy == null) return@rememberLauncherForActivityResult
        work.launch {
            withContext(Dispatchers.IO) {
                runCatching { context.contentResolver.openOutputStream(uri, "wt")!!.use { it.write(copy) } }
            }.onSuccess { close() }.onFailure { problem = it.message ?: it.toString() }
        }
    }
    val same = passphrase == again
    AlertDialog(
        onDismissRequest = close,
        containerColor = c.surface,
        title = { Text(tr("Save a copy…"), color = c.text) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                listOf(tr("Passphrase") to passphrase, tr("Again") to again).forEachIndexed { at, (label, value) ->
                    OutlinedTextField(
                        value = value,
                        onValueChange = { if (at == 0) passphrase = it else again = it },
                        label = { Text(label) },
                        singleLine = true,
                        visualTransformation = PasswordVisualTransformation(),
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password, autoCorrectEnabled = false),
                        colors = TextFieldDefaults.colors(
                            focusedContainerColor = c.surface,
                            unfocusedContainerColor = c.surface,
                            focusedIndicatorColor = c.accent,
                            unfocusedIndicatorColor = c.outline,
                            cursorColor = c.accent,
                            focusedTextColor = c.text,
                            unfocusedTextColor = c.text,
                        ),
                        modifier = Modifier.fillMaxWidth().testTag(if (at == 0) "copy-passphrase" else "copy-again"),
                    )
                }
                if (again.isNotEmpty() && !same) {
                    Text(tr("The two passphrases are not the same."), fontSize = 13.sp, color = c.warnText)
                } else {
                    Text(tr("The copy is encrypted with this passphrase. packwing on a computer signs with it too."), fontSize = 13.sp, color = c.muted)
                }
                problem?.let { Text(it, fontSize = 13.sp, color = c.warnText) }
            }
        },
        confirmButton = {
            TextButton(
                onClick = {
                    runCatching { exportSigningKey(key.id, passphrase) }
                        .onSuccess {
                            bytes = it
                            save.launch(key.name)
                        }
                        .onFailure { e -> problem = (e as? MobileException)?.said ?: e.toString() }
                },
                enabled = passphrase.isNotEmpty() && same,
            ) { Text(tr("Save a copy…"), color = c.accent, fontWeight = FontWeight.SemiBold) }
        },
        dismissButton = { TextButton(onClick = close) { Text(tr("Cancel"), color = c.accent) } },
    )
}
