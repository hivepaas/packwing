package com.hivepaas.packwing.ui

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
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
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material.icons.outlined.Folder
import androidx.compose.material.icons.outlined.Inventory2
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Destination
import com.hivepaas.packwing.Documents
import com.hivepaas.packwing.tr
import com.hivepaas.packwing.trN
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.packwing_mobile.ArchiveFormat
import uniffi.packwing_mobile.CompressionLevel
import uniffi.packwing_mobile.NewArchiveHow
import java.io.File

/** A new ZIP or 7z of files the user picked, with AES-256 if they ask. */
@Composable
fun CreateScreen(model: AppModel) {
    val c = tonal()
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var name by remember { mutableStateOf("") }
    var sevenZip by remember { mutableStateOf(false) }
    var locked by remember { mutableStateOf(false) }
    var password by remember { mutableStateOf("") }
    var level by remember { mutableStateOf(3f) }
    var files by remember { mutableStateOf<List<File>>(emptyList()) }
    var copying by remember { mutableStateOf(false) }
    var folder by remember { mutableStateOf(model.settings.destination) }

    val pickFiles = rememberLauncherForActivityResult(
        ActivityResultContracts.OpenMultipleDocuments(),
    ) { uris ->
        if (uris.isNotEmpty()) {
            copying = true
            scope.launch {
                val copied = withContext(Dispatchers.IO) { Documents.copyIn(context, uris) }
                files = files + copied
                if (name.isBlank()) {
                    name = copied.firstOrNull()?.name?.substringBeforeLast('.') ?: "archive"
                }
                copying = false
            }
        }
    }
    val pickFolder = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { picked ->
        if (picked != null) {
            Documents.keepAccess(context, picked, write = true)
            model.settings.destination = picked
            folder = picked
        }
    }

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
                Icon(Icons.Outlined.Close, tr("Cancel"), tint = c.text)
            }
            Text(
                tr("New archive"),
                Modifier.weight(1f),
                fontSize = 20.sp,
                fontWeight = FontWeight.Bold,
                color = c.text,
            )
        }

        LazyColumn(
            modifier = Modifier.weight(1f),
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, bottom = 16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            item {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text(tr("Name")) },
                    singleLine = true,
                    shape = RoundedCornerShape(14.dp),
                    colors = fieldColors(c),
                    modifier = Modifier.fillMaxWidth(),
                )
            }

            item {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        tr("Format"),
                        modifier = Modifier.padding(start = 6.dp),
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Bold,
                        color = c.accent,
                    )
                    Row(Modifier.fillMaxWidth()) {
                        Segment(
                            label = "ZIP",
                            icon = Icons.Outlined.Inventory2,
                            picked = !sevenZip,
                            start = true,
                            modifier = Modifier.weight(1f),
                        ) { sevenZip = false }
                        Segment(
                            label = "7z",
                            icon = Icons.Outlined.Inventory2,
                            picked = sevenZip,
                            start = false,
                            modifier = Modifier.weight(1f),
                        ) { sevenZip = true }
                    }
                }
            }

            item {
                Column(
                    Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(20.dp))
                        .background(c.surface),
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth().heightIn(min = 56.dp).padding(horizontal = 16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(tr("Protect with a password"), Modifier.weight(1f), fontSize = 16.sp, color = c.text)
                        Switch(
                            checked = locked,
                            onCheckedChange = { locked = it },
                            colors = SwitchDefaults.colors(
                                checkedThumbColor = c.surface,
                                checkedTrackColor = c.accent,
                                uncheckedThumbColor = c.outline,
                                uncheckedTrackColor = c.field,
                                uncheckedBorderColor = c.outline,
                            ),
                        )
                    }
                    if (locked) {
                        Column(
                            Modifier.padding(start = 16.dp, end = 16.dp, bottom = 16.dp),
                            verticalArrangement = Arrangement.spacedBy(8.dp),
                        ) {
                            OutlinedTextField(
                                value = password,
                                onValueChange = { password = it },
                                label = { Text(tr("Password")) },
                                singleLine = true,
                                visualTransformation = PasswordVisualTransformation(),
                                shape = RoundedCornerShape(14.dp),
                                colors = fieldColors(c),
                                modifier = Modifier.fillMaxWidth(),
                            )
                            Text(
                                if (sevenZip) {
                                    tr("AES-256. In 7z the names are hidden too.")
                                } else {
                                    tr("AES-256.")
                                },
                                fontSize = 13.sp,
                                color = c.muted,
                            )
                        }
                    }
                }
            }

            item {
                Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    Row(Modifier.fillMaxWidth().padding(horizontal = 6.dp)) {
                        Text(
                            tr("Compression"),
                            Modifier.weight(1f),
                            fontSize = 14.sp,
                            fontWeight = FontWeight.Bold,
                            color = c.accent,
                        )
                        Text(levelName(level), fontSize = 14.sp, color = c.muted)
                    }
                    Slider(
                        value = level,
                        onValueChange = { level = it },
                        valueRange = 0f..5f,
                        steps = 4,
                        colors = SliderDefaults.colors(
                            thumbColor = c.accent,
                            activeTrackColor = c.accent,
                            inactiveTrackColor = c.tonal,
                        ),
                    )
                }
            }

            item {
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
                    Box(
                        Modifier.size(42.dp).clip(RoundedCornerShape(14.dp)).background(c.tonal),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(Icons.Outlined.Add, null, tint = c.onTonal)
                    }
                    Column(Modifier.weight(1f)) {
                        Text(
                            if (files.isEmpty()) {
                                tr("No files yet")
                            } else {
                                trN(files.size, "{n} file", "{n} files", "n" to files.size.toString())
                            },
                            fontSize = 15.sp,
                            fontWeight = FontWeight.SemiBold,
                            color = c.text,
                        )
                        Text(
                            if (copying) {
                                tr("Copying them in…")
                            } else {
                                sizeText(files.sumOf { it.length() }.toULong())
                            },
                            fontSize = 13.sp,
                            color = c.muted,
                        )
                    }
                    TextButton(onClick = { pickFiles.launch(arrayOf("*/*")) }) {
                        Text(tr("Add"), color = c.accent, fontWeight = FontWeight.SemiBold)
                    }
                }
            }

            item {
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
                        Text(tr("Where it goes"), fontSize = 13.sp, color = c.muted)
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
            }
        }

        Box(Modifier.padding(16.dp)) {
            Button(
                onClick = {
                    model.create(
                        paths = files,
                        name = name.ifBlank { "archive" },
                        how = NewArchiveHow(
                            format = if (sevenZip) ArchiveFormat.SEVEN_ZIP else ArchiveFormat.ZIP,
                            compression = levelOf(level),
                            password = password.takeIf { locked && it.isNotEmpty() },
                            hideNames = sevenZip && locked,
                        ),
                        destination = Destination.Folder(folder ?: return@Button),
                    )
                },
                enabled = files.isNotEmpty() && !copying && folder != null && (!locked || password.isNotEmpty()),
                shape = RoundedCornerShape(28.dp),
                colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
                modifier = Modifier.fillMaxWidth().height(56.dp),
            ) {
                Text(
                    tr("Make {kind}", "kind" to if (sevenZip) "7z" else "ZIP"),
                    fontSize = 17.sp,
                    fontWeight = FontWeight.Bold,
                )
            }
        }
    }
}

@Composable
private fun fieldColors(c: Tonal) = TextFieldDefaults.colors(
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

private fun levelOf(value: Float): CompressionLevel = when (value.toInt()) {
    0 -> CompressionLevel.STORE
    1 -> CompressionLevel.LOWEST
    2 -> CompressionLevel.LOW
    3 -> CompressionLevel.MEDIUM
    4 -> CompressionLevel.HIGH
    else -> CompressionLevel.HIGHEST
}

@Composable
private fun levelName(value: Float): String = when (value.toInt()) {
    0 -> tr("Store, fastest")
    1 -> tr("Lowest")
    2 -> tr("Low")
    3 -> tr("Normal")
    4 -> tr("High")
    else -> tr("Smallest")
}
