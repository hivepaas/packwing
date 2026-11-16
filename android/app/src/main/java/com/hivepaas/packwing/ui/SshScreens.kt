package com.hivepaas.packwing.ui

import android.content.ClipData
import android.content.ClipboardManager
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
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.Description
import androidx.compose.material.icons.outlined.Key
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.material.icons.outlined.Shield
import androidx.compose.material.icons.outlined.WarningAmber
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
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
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Asked
import com.hivepaas.packwing.SshAnswer
import com.hivepaas.packwing.SshQuestion
import com.hivepaas.packwing.said
import com.hivepaas.packwing.tr
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.packwing_mobile.MobileException
import uniffi.packwing_mobile.SshKey
import uniffi.packwing_mobile.TypedSecret
import uniffi.packwing_mobile.importSshKey
import uniffi.packwing_mobile.removeSshKey
import uniffi.packwing_mobile.sshKeys

// MARK: What SFTP asks

/**
 * The first of SFTP's questions waiting, as a card over the screen dimmed: the
 * desktop's, in its words. It is answered, never just closed: Back says no.
 */
@Composable
fun SshQuestionDialog(model: AppModel) {
    val asked = model.questions.firstOrNull() ?: return
    // A new card for each: nothing typed for one is left in the next.
    androidx.compose.runtime.key(asked) { QuestionCard(asked) { said -> model.answer(asked, said) } }
}

@Composable
private fun QuestionCard(asked: Asked, answer: (SshAnswer) -> Unit) {
    val c = tonal()
    val question = asked.question
    var secret by remember { mutableStateOf("") }
    var keep by remember { mutableStateOf(false) }
    val needsSecret = question is SshQuestion.Password || question is SshQuestion.Passphrase
    val yes: SshAnswer = when (question) {
        is SshQuestion.HostKey -> SshAnswer.Trust(true)
        is SshQuestion.Password, is SshQuestion.Passphrase -> SshAnswer.Secret(TypedSecret(secret, keep))
        is SshQuestion.Code -> SshAnswer.Code(secret)
    }
    val no: SshAnswer = when (question) {
        is SshQuestion.HostKey -> SshAnswer.Trust(false)
        is SshQuestion.Password, is SshQuestion.Passphrase -> SshAnswer.Secret(null)
        is SshQuestion.Code -> SshAnswer.Code(null)
    }
    val title = when (question) {
        is SshQuestion.HostKey -> tr("Trust this server?")
        is SshQuestion.Password -> tr("Sign in to the server")
        is SshQuestion.Passphrase -> tr("Enter the key’s passphrase")
        is SshQuestion.Code -> question.name.trim().ifEmpty { tr("The server asks") }
    }
    val detail = when (question) {
        is SshQuestion.HostKey -> if (question.port.toInt() == 22) question.host else "${question.host}:${question.port}"
        is SshQuestion.Password -> question.target
        is SshQuestion.Passphrase -> question.key
        is SshQuestion.Code -> question.instructions
    }
    val yesLabel = when (question) {
        is SshQuestion.HostKey -> tr("Trust and connect")
        is SshQuestion.Password -> tr("Sign in")
        is SshQuestion.Passphrase -> tr("Open")
        is SshQuestion.Code -> tr("Send")
    }
    val ready = !needsSecret || secret.isNotEmpty()

    Dialog(
        onDismissRequest = { answer(no) },
        properties = DialogProperties(dismissOnClickOutside = false, usePlatformDefaultWidth = false),
    ) {
        Column(
            Modifier
                .padding(20.dp)
                .widthIn(max = 460.dp)
                .clip(RoundedCornerShape(28.dp))
                .background(c.page)
                .verticalScroll(rememberScrollState())
                .padding(22.dp)
                .testTag("ssh-question"),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Box(
                    Modifier.size(44.dp).clip(RoundedCornerShape(14.dp)).background(c.tonal),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        if (question is SshQuestion.HostKey) Icons.Outlined.Shield else Icons.Outlined.Lock,
                        null,
                        tint = c.onTonal,
                    )
                }
                Column {
                    Text(title, fontSize = 19.sp, fontWeight = FontWeight.Bold, color = c.text)
                    if (detail.isNotEmpty()) {
                        Text(detail, fontSize = 14.sp, color = c.muted, maxLines = 2, overflow = TextOverflow.Ellipsis)
                    }
                }
            }

            when (question) {
                is SshQuestion.HostKey -> {
                    Text(
                        tr("packwing has not connected to this server before. Trust it only if this is its key: whoever runs the server can tell you its fingerprint."),
                        fontSize = 15.sp,
                        lineHeight = 21.sp,
                        color = c.muted,
                    )
                    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        Text(
                            tr("{algorithm} key fingerprint", "algorithm" to question.algorithm),
                            fontSize = 13.sp,
                            fontWeight = FontWeight.SemiBold,
                            color = c.muted,
                        )
                        // Whole, on as many lines as it takes: it is compared, digit by digit.
                        Text(
                            question.fingerprint,
                            modifier = Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(16.dp))
                                .background(c.field)
                                .padding(14.dp)
                                .testTag("ssh-fingerprint"),
                            fontSize = 15.sp,
                            fontFamily = FontFamily.Monospace,
                            fontWeight = FontWeight.Medium,
                            color = c.text,
                        )
                    }
                }
                is SshQuestion.Password, is SshQuestion.Passphrase -> {
                    val password = question is SshQuestion.Password
                    val retry = (question as? SshQuestion.Password)?.retry ?: (question as SshQuestion.Passphrase).retry
                    if (retry) {
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            Icon(Icons.Outlined.WarningAmber, null, tint = c.warnText, modifier = Modifier.size(20.dp))
                            Text(
                                if (password) tr("The server did not take that password.") else tr("That passphrase does not open the key."),
                                fontSize = 14.sp,
                                color = c.warnText,
                            )
                        }
                    }
                    SecretField(
                        secret,
                        if (password) tr("Password") else tr("Passphrase"),
                        hidden = true,
                        tag = "ssh-secret",
                        done = { if (ready) answer(yes) },
                    ) { secret = it }
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(tr("Keep it in the system’s keychain"), Modifier.weight(1f), fontSize = 15.sp, color = c.text)
                        Switch(
                            checked = keep,
                            onCheckedChange = { keep = it },
                            colors = SwitchDefaults.colors(
                                checkedThumbColor = c.surface,
                                checkedTrackColor = c.accent,
                                uncheckedThumbColor = c.outline,
                                uncheckedTrackColor = c.field,
                                uncheckedBorderColor = c.outline,
                            ),
                        )
                    }
                }
                is SshQuestion.Code -> SecretField(
                    secret,
                    question.text.trim(),
                    hidden = !question.echo,
                    tag = "ssh-secret",
                    done = { answer(yes) },
                ) { secret = it }
            }

            Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                OutlinedButton(
                    onClick = { answer(no) },
                    shape = RoundedCornerShape(24.dp),
                    modifier = Modifier.weight(1f).heightIn(min = 48.dp),
                ) {
                    Text(
                        if (question is SshQuestion.HostKey) tr("Don’t connect") else tr("Cancel"),
                        color = c.accent,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
                Button(
                    onClick = { answer(yes) },
                    enabled = ready,
                    shape = RoundedCornerShape(24.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
                    modifier = Modifier.weight(1f).heightIn(min = 48.dp).testTag("ssh-yes"),
                ) {
                    Text(yesLabel, fontWeight = FontWeight.Bold)
                }
            }
        }
    }
}

@Composable
private fun SecretField(
    text: String,
    label: String,
    hidden: Boolean,
    tag: String,
    done: () -> Unit,
    change: (String) -> Unit,
) {
    OutlinedTextField(
        value = text,
        onValueChange = change,
        label = { Text(label) },
        singleLine = true,
        visualTransformation = if (hidden) PasswordVisualTransformation() else VisualTransformation.None,
        keyboardOptions = KeyboardOptions(
            keyboardType = if (hidden) KeyboardType.Password else KeyboardType.Ascii,
            autoCorrectEnabled = false,
            imeAction = ImeAction.Done,
        ),
        keyboardActions = KeyboardActions(onDone = { done() }),
        colors = sshFieldColors(),
        modifier = Modifier.fillMaxWidth().testTag(tag),
    )
}

@Composable
private fun sshFieldColors() = tonal().let { c ->
    TextFieldDefaults.colors(
        focusedContainerColor = c.field,
        unfocusedContainerColor = c.field,
        focusedIndicatorColor = c.accent,
        unfocusedIndicatorColor = c.outline,
        focusedLabelColor = c.accent,
        unfocusedLabelColor = c.muted,
        cursorColor = c.accent,
        focusedTextColor = c.text,
        unfocusedTextColor = c.text,
    )
}

// MARK: Keys kept

/** The keys kept, read again: none if the keychain cannot be reached. */
private fun keptKeys(): List<SshKey> = runCatching { sshKeys() }.getOrDefault(emptyList())

/**
 * The key a storage signs in with (`selection`, its id; empty for any of
 * them), chosen from those kept; and Add a key file….
 */
@Composable
fun SshKeyPicker(selection: String, choose: (String) -> Unit) {
    val c = tonal()
    var keys by remember { mutableStateOf(keptKeys()) }
    var open by remember { mutableStateOf(false) }
    var importing by remember { mutableStateOf(false) }
    val chosen = keys.firstOrNull { it.id == selection }?.name ?: tr("Any key")
    Column(
        Modifier.fillMaxWidth().clip(RoundedCornerShape(20.dp)).background(c.surface),
    ) {
        Box {
            Row(
                Modifier
                    .fillMaxWidth()
                    .heightIn(min = 56.dp)
                    .clickable { open = true }
                    .padding(horizontal = 16.dp)
                    .testTag("storage-key"),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(tr("Key"), Modifier.weight(1f), fontSize = 16.sp, color = c.text)
                Text(chosen, fontSize = 15.sp, color = c.muted, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            DropdownMenu(expanded = open, onDismissRequest = { open = false }, containerColor = c.surface) {
                DropdownMenuItem(
                    text = { Text(tr("Any key"), color = c.text) },
                    onClick = {
                        open = false
                        choose("")
                    },
                )
                keys.forEach { key ->
                    DropdownMenuItem(
                        text = { Text(key.name, color = c.text) },
                        onClick = {
                            open = false
                            choose(key.id)
                        },
                    )
                }
            }
        }
        HorizontalDivider(color = c.field)
        Row(
            Modifier
                .fillMaxWidth()
                .heightIn(min = 52.dp)
                .clickable { importing = true }
                .padding(horizontal = 16.dp)
                .testTag("key-add"),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Icon(Icons.Outlined.Key, null, tint = c.accent)
            Text(tr("Add a key file…"), fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = c.accent)
        }
    }
    if (importing) {
        ImportKeySheet(close = { importing = false }) { id ->
            keys = keptKeys()
            choose(id)
        }
    }
}

/**
 * A private key to keep, pasted or from its file, opened with its passphrase
 * if it has one: an SSH key by default, kept by `keep`, which says its id.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ImportKeySheet(
    close: () -> Unit,
    keep: (name: String, text: String, passphrase: String?) -> String = { name, text, passphrase ->
        importSshKey(name, text, passphrase).id
    },
    kept: (String) -> Unit,
) {
    val c = tonal()
    val context = LocalContext.current
    val work = rememberCoroutineScope()
    var name by remember { mutableStateOf("") }
    var text by remember { mutableStateOf("") }
    var passphrase by remember { mutableStateOf("") }
    var asksPassphrase by remember { mutableStateOf(false) }
    var problem by remember { mutableStateOf<String?>(null) }

    val choose = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri == null) return@rememberLauncherForActivityResult
        work.launch {
            withContext(Dispatchers.IO) {
                runCatching {
                    val called = com.hivepaas.packwing.Documents.picked(context, uri).name
                    // A key file is small; anything much bigger is not one.
                    val bytes = context.contentResolver.openInputStream(uri)?.use { it.upTo(MOST_KEY_FILE + 1) }
                    called to bytes?.takeIf { it.size <= MOST_KEY_FILE }
                }.getOrNull()
            }?.let { (called, bytes) ->
                if (bytes != null) {
                    text = bytes.decodeToString()
                    if (name.isEmpty()) name = called
                }
            }
        }
    }

    fun add() {
        val named = name.trim().ifEmpty { tr("Key") }
        val given = passphrase.ifEmpty { null }
        work.launch {
            val result = withContext(Dispatchers.IO) { runCatching { keep(named, text, given) } }
            result.onSuccess { id ->
                kept(id)
                close()
            }.onFailure { e ->
                if (e is MobileException.Locked) asksPassphrase = true
                problem = (e as? MobileException)?.said ?: e.toString()
            }
        }
    }

    ModalBottomSheet(
        onDismissRequest = close,
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
            Text(tr("Private key"), Modifier.padding(start = 4.dp), fontSize = 22.sp, fontWeight = FontWeight.Bold, color = c.text)
            OutlinedTextField(
                value = text,
                onValueChange = { text = it },
                placeholder = { Text("-----BEGIN OPENSSH PRIVATE KEY-----", color = c.faint, fontSize = 13.sp) },
                minLines = 4,
                maxLines = 8,
                textStyle = androidx.compose.ui.text.TextStyle(fontSize = 12.sp, fontFamily = FontFamily.Monospace),
                keyboardOptions = KeyboardOptions(autoCorrectEnabled = false, keyboardType = KeyboardType.Ascii),
                colors = sshFieldColors(),
                modifier = Modifier.fillMaxWidth().testTag("private-key-text"),
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
                colors = sshFieldColors(),
                modifier = Modifier.fillMaxWidth().testTag("private-key-name"),
            )
            if (asksPassphrase) {
                SecretField(passphrase, tr("Passphrase"), hidden = true, tag = "private-key-passphrase", done = { add() }) {
                    passphrase = it
                }
            }
            problem?.let {
                Row(
                    Modifier.fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(c.warnBg).padding(14.dp),
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    Icon(Icons.Outlined.WarningAmber, null, tint = c.warnText)
                    Text(it, fontSize = 14.sp, color = c.warnText)
                }
            }
            Button(
                onClick = { add() },
                enabled = text.isNotBlank(),
                shape = RoundedCornerShape(28.dp),
                colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
                modifier = Modifier.fillMaxWidth().height(56.dp).testTag("private-key-add"),
            ) {
                Text(tr("Add"), fontSize = 17.sp, fontWeight = FontWeight.Bold)
            }
        }
    }
}

/** Settings' SSH keys: those kept, their public halves copied, removed. */
@Composable
fun SshKeys() {
    val c = tonal()
    val context = LocalContext.current
    var keys by remember { mutableStateOf(keptKeys()) }
    var importing by remember { mutableStateOf(false) }
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(20.dp)).background(c.surface)) {
        keys.forEach { key ->
            Row(
                Modifier.fillMaxWidth().heightIn(min = 60.dp).padding(start = 16.dp, end = 4.dp, top = 8.dp, bottom = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(Modifier.weight(1f)) {
                    Text(key.name, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = c.text)
                    Text(
                        "${key.kind} · ${key.fingerprint}",
                        fontSize = 12.sp,
                        fontFamily = FontFamily.Monospace,
                        color = c.muted,
                        maxLines = 1,
                        overflow = TextOverflow.MiddleEllipsis,
                    )
                }
                // Its public half, for a server's authorized_keys.
                IconButton(onClick = {
                    context.getSystemService(ClipboardManager::class.java)
                        .setPrimaryClip(ClipData.newPlainText("public key", key.public))
                }) {
                    Icon(Icons.Outlined.ContentCopy, tr("Copy"), tint = c.muted)
                }
                IconButton(onClick = {
                    runCatching { removeSshKey(key.id) }
                    keys = keptKeys()
                }) {
                    Icon(Icons.Outlined.Delete, tr("Delete"), tint = c.muted)
                }
            }
            HorizontalDivider(color = c.field)
        }
        Row(
            Modifier
                .fillMaxWidth()
                .heightIn(min = 56.dp)
                .clickable { importing = true }
                .padding(horizontal = 16.dp)
                .testTag("ssh-key-add"),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Icon(Icons.Outlined.Key, null, tint = c.accent)
            Text(tr("Add a key file…"), fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = c.accent)
        }
    }
    if (importing) {
        ImportKeySheet(close = { importing = false }) { keys = keptKeys() }
    }
}

/** A key file is a few kilobytes; anything much bigger is not one. */
private const val MOST_KEY_FILE = 64 * 1024
