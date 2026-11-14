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
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Check
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.BuildConfig
import com.hivepaas.packwing.Documents
import com.hivepaas.packwing.tr
import uniffi.packwing_mobile.languages

/** What the user chooses: where files go, what to be told, and the language. */
@Composable
fun SettingsScreen(model: AppModel) {
    val c = tonal()
    val context = LocalContext.current
    var destination by remember { mutableStateOf(model.settings.destination) }
    var overwrite by remember { mutableStateOf(model.settings.overwrite) }
    var tell by remember { mutableStateOf(model.settings.tellWhenDone) }
    var language by remember { mutableStateOf(model.settings.language) }
    var pickingLanguage by remember { mutableStateOf(false) }

    val pickFolder = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { picked ->
        if (picked != null) {
            Documents.keepAccess(context, picked, write = true)
            model.settings.destination = picked
            destination = picked
        }
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
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

        item { Heading(tr("Extracting")) }
        item {
            Group {
                Line(
                    tr("Where files go"),
                    destination?.let { Documents.treeName(context, it) } ?: tr("Ask every time"),
                ) { pickFolder.launch(destination) }
                HorizontalDivider(color = c.field)
                Toggle(tr("Replace a file already there"), overwrite) {
                    overwrite = it
                    model.settings.overwrite = it
                }
            }
        }

        item { Heading(tr("While it works")) }
        item {
            Group {
                Toggle(tr("Tell me when a job is through"), tell) {
                    tell = it
                    model.settings.tellWhenDone = it
                }
            }
        }

        item { Heading(tr("How it looks")) }
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
            }
        }

        item { Heading(tr("About")) }
        item {
            Group {
                Line("packwing", BuildConfig.VERSION_NAME, null)
                HorizontalDivider(color = c.field)
                Line(tr("Licence"), "GPL-3.0-or-later", null)
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

@Composable
private fun Group(content: @Composable () -> Unit) = Column(
    Modifier
        .fillMaxWidth()
        .clip(RoundedCornerShape(20.dp))
        .background(tonal().surface),
) { content() }

@Composable
private fun Line(title: String, value: String, open: (() -> Unit)? = {}) {
    val c = tonal()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 60.dp)
            .let { if (open != null) it.clickable(onClick = open) else it }
            .padding(horizontal = 16.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Text(title, Modifier.weight(1f), fontSize = 16.sp, color = c.text)
        Text(
            value,
            fontSize = 15.sp,
            color = c.muted,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun Toggle(title: String, on: Boolean, set: (Boolean) -> Unit) {
    val c = tonal()
    Row(
        modifier = Modifier.fillMaxWidth().heightIn(min = 60.dp).padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(title, Modifier.weight(1f), fontSize = 16.sp, color = c.text)
        Switch(
            checked = on,
            onCheckedChange = set,
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
