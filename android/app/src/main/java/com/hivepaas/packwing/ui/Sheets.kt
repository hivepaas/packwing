package com.hivepaas.packwing.ui

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Check
import androidx.compose.material.icons.outlined.Folder
import androidx.compose.material.icons.outlined.Image
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ModalBottomSheet
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
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Destination
import com.hivepaas.packwing.Documents
import com.hivepaas.packwing.Sheet
import com.hivepaas.packwing.tr
import com.hivepaas.packwing.trN
import uniffi.packwing_mobile.Secrets

/** What an encrypted archive needs, asked without leaving the screen. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PasswordSheet(model: AppModel, ask: Sheet.Password) {
    val c = tonal()
    val state = rememberModalBottomSheetState()
    var secret by remember(ask.uri) { mutableStateOf("") }
    var shown by remember { mutableStateOf(false) }

    ModalBottomSheet(
        onDismissRequest = { model.sheet = null },
        sheetState = state,
        containerColor = c.page,
        contentColor = c.text,
    ) {
        Column(
            Modifier
                .verticalScroll(rememberScrollState())
                .padding(start = 20.dp, end = 20.dp, bottom = 28.dp),
            verticalArrangement = Arrangement.spacedBy(18.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
                Box(
                    Modifier.size(48.dp).clip(RoundedCornerShape(16.dp)).background(c.tonal),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(Icons.Outlined.Lock, null, tint = c.onTonal, modifier = Modifier.size(24.dp))
                }
                Column {
                    Text(
                        if (ask.passphrase) tr("Enter the passphrase") else tr("Enter the password"),
                        fontSize = 20.sp,
                        fontWeight = FontWeight.Bold,
                        color = c.text,
                    )
                    Text(
                        ask.name,
                        fontSize = 14.sp,
                        color = c.muted,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }

            OutlinedTextField(
                value = secret,
                onValueChange = { secret = it },
                label = { Text(if (ask.passphrase) tr("Passphrase") else tr("Password")) },
                singleLine = true,
                isError = ask.problem != null,
                supportingText = {
                    Text(
                        ask.problem ?: tr("The password is never saved"),
                        color = if (ask.problem != null) c.badText else c.muted,
                    )
                },
                visualTransformation = if (shown) VisualTransformation.None else PasswordVisualTransformation(),
                trailingIcon = {
                    IconButton(onClick = { shown = !shown }) {
                        Icon(
                            if (shown) Icons.Outlined.VisibilityOff else Icons.Outlined.Visibility,
                            if (shown) tr("Hide the password") else tr("Show the password"),
                            tint = c.muted,
                        )
                    }
                },
                shape = RoundedCornerShape(14.dp),
                colors = TextFieldDefaults.colors(
                    focusedContainerColor = c.page,
                    unfocusedContainerColor = c.page,
                    focusedIndicatorColor = c.accent,
                    unfocusedIndicatorColor = c.outline,
                    focusedLabelColor = c.accent,
                    unfocusedLabelColor = c.muted,
                    cursorColor = c.accent,
                    focusedTextColor = c.text,
                    unfocusedTextColor = c.text,
                ),
                modifier = Modifier.fillMaxWidth(),
            )

            Button(
                onClick = {
                    model.open(
                        ask.uri,
                        if (ask.passphrase) Secrets(null, secret) else Secrets(secret, null),
                    )
                },
                enabled = secret.isNotEmpty(),
                shape = RoundedCornerShape(28.dp),
                colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
                modifier = Modifier.fillMaxWidth().height(56.dp),
            ) {
                Text(tr("Unlock"), fontSize = 17.sp, fontWeight = FontWeight.Bold)
            }
            TextButton(onClick = { model.sheet = null }, modifier = Modifier.fillMaxWidth()) {
                Text(tr("Not now"), color = c.accent, fontWeight = FontWeight.SemiBold)
            }
        }
    }
}

/** Where what is extracted goes, and what happens to a file already there. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ExtractSheet(model: AppModel, ask: Sheet.Extract) {
    val c = tonal()
    val state = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val opened = model.opened ?: return
    var toGallery by remember { mutableStateOf(false) }
    var folder by remember { mutableStateOf(model.settings.destination) }
    var overwrite by remember { mutableStateOf(model.settings.overwrite) }
    val context = androidx.compose.ui.platform.LocalContext.current

    val pickFolder = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { picked ->
        if (picked != null) {
            Documents.keepAccess(context, picked, write = true)
            model.settings.destination = picked
            folder = picked
        }
    }

    ModalBottomSheet(
        onDismissRequest = { model.sheet = null },
        sheetState = state,
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
                tr("Extract"),
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
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        tr(
                            "{files} · {size} once extracted",
                            "files" to trN(
                                opened.info.files,
                                "{n} file",
                                "{n} files",
                                "n" to opened.info.files.toString(),
                            ),
                            "size" to sizeText(opened.info.unpacked),
                        ),
                        fontSize = 13.sp,
                        color = c.onTonal,
                    )
                }
            }

            Text(
                tr("Save to"),
                modifier = Modifier.padding(start = 6.dp),
                fontSize = 14.sp,
                fontWeight = FontWeight.Bold,
                color = c.accent,
            )
            Row(Modifier.fillMaxWidth()) {
                Segment(
                    label = tr("A folder"),
                    icon = Icons.Outlined.Folder,
                    picked = !toGallery,
                    start = true,
                    modifier = Modifier.weight(1f),
                ) { toGallery = false }
                Segment(
                    label = tr("The gallery"),
                    icon = Icons.Outlined.Image,
                    picked = toGallery,
                    start = false,
                    modifier = Modifier.weight(1f),
                ) { toGallery = true }
            }

            if (!toGallery) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(min = 64.dp)
                        .clip(RoundedCornerShape(20.dp))
                        .background(c.surface)
                        .padding(start = 16.dp, end = 10.dp, top = 10.dp, bottom = 10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Icon(Icons.Outlined.Folder, null, tint = c.accent, modifier = Modifier.size(24.dp))
                    Column(Modifier.weight(1f)) {
                        Text(tr("Where files go"), fontSize = 13.sp, color = c.muted)
                        Text(
                            folder?.let { Documents.treeName(context, it) } ?: tr("No folder chosen yet"),
                            fontSize = 15.sp,
                            fontWeight = FontWeight.SemiBold,
                            color = c.text,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                    TextButton(onClick = { pickFolder.launch(folder) }) {
                        Text(
                            if (folder == null) tr("Choose") else tr("Change"),
                            color = c.accent,
                            fontWeight = FontWeight.SemiBold,
                        )
                    }
                }
                Text(
                    tr("Android asks for this folder once. packwing keeps no other access to your storage."),
                    modifier = Modifier.padding(horizontal = 8.dp),
                    fontSize = 13.sp,
                    lineHeight = 19.sp,
                    color = c.muted,
                )
            } else {
                Text(
                    tr("The pictures and videos go to your gallery. The other files stay in the archive."),
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(20.dp))
                        .background(c.surface)
                        .padding(16.dp),
                    fontSize = 14.sp,
                    lineHeight = 20.sp,
                    color = c.muted,
                )
            }

            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(20.dp))
                    .background(c.surface)
                    .padding(horizontal = 16.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(tr("Replace a file already there"), Modifier.weight(1f), fontSize = 16.sp, color = c.text)
                Switch(
                    checked = overwrite,
                    onCheckedChange = {
                        overwrite = it
                        model.settings.overwrite = it
                    },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = c.surface,
                        checkedTrackColor = c.accent,
                        uncheckedThumbColor = c.outline,
                        uncheckedTrackColor = c.field,
                        uncheckedBorderColor = c.outline,
                    ),
                )
            }

            Text(
                if (toGallery) {
                    tr("Nothing is written through links, and an archive bomb stops at its size limit.")
                } else {
                    tr("Nothing is written outside that folder or through links, and an archive bomb stops at its size limit.")
                },
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(16.dp))
                    .background(c.field)
                    .padding(14.dp),
                fontSize = 13.sp,
                lineHeight = 19.sp,
                color = c.muted,
            )

            Button(
                onClick = {
                    val where = if (toGallery) Destination.Gallery else Destination.Folder(folder ?: return@Button)
                    model.extract(ask.picked, where)
                },
                enabled = toGallery || folder != null,
                shape = RoundedCornerShape(28.dp),
                colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
                modifier = Modifier.fillMaxWidth().height(56.dp),
            ) {
                Text(
                    // How many go to the gallery is not known until they are
                    // read: what is neither picture nor video stays behind.
                    if (toGallery) {
                        tr("Extract the pictures")
                    } else {
                        trN(
                            opened.info.files,
                            "Extract {n} file",
                            "Extract {n} files",
                            "n" to opened.info.files.toString(),
                        )
                    },
                    fontSize = 17.sp,
                    fontWeight = FontWeight.Bold,
                )
            }
        }
    }
}

/** One of two buttons joined into a pair, as Material draws a segmented one. */
@Composable
internal fun Segment(
    label: String,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    picked: Boolean,
    start: Boolean,
    modifier: Modifier = Modifier,
    pick: () -> Unit,
) {
    val c = tonal()
    val shape = if (start) {
        RoundedCornerShape(topStart = 22.dp, bottomStart = 22.dp)
    } else {
        RoundedCornerShape(topEnd = 22.dp, bottomEnd = 22.dp)
    }
    Row(
        modifier = modifier
            .height(44.dp)
            .clip(shape)
            .background(if (picked) c.tonal else c.page)
            .border(1.dp, c.accent, shape)
            .clickable(onClick = pick),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.Center,
    ) {
        if (picked) {
            Icon(Icons.Outlined.Check, null, tint = c.onTonal, modifier = Modifier.size(18.dp))
            Box(Modifier.size(6.dp))
        }
        Text(
            label,
            fontSize = 15.sp,
            fontWeight = FontWeight.SemiBold,
            color = if (picked) c.onTonal else c.text,
        )
    }
}
