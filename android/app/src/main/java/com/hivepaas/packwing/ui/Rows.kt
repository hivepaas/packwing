package com.hivepaas.packwing.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ExpandMore
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

// The rows a group of choices is made of, in New archive and in Settings.

/**
 * A choice of a few, as a row that says the one chosen and opens the others;
 * `detail` under its name says what the one chosen does.
 */
@Composable
internal fun Choice(
    title: String,
    chosen: String,
    options: List<Pair<String, String>>,
    tag: String,
    detail: String? = null,
    enabled: Boolean = true,
    choose: (String) -> Unit,
) {
    val c = tonal()
    var open by remember { mutableStateOf(false) }
    Box {
        Row(
            Modifier
                .fillMaxWidth()
                .heightIn(min = 56.dp)
                .clickable(enabled = enabled) { open = true }
                .padding(horizontal = 16.dp, vertical = 6.dp)
                .testTag(tag),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f)) {
                Text(title, fontSize = 16.sp, color = if (enabled) c.text else c.faint)
                detail?.let { Text(it, fontSize = 12.sp, color = c.muted) }
            }
            Text(chosen, fontSize = 15.sp, color = if (enabled) c.muted else c.faint)
            Icon(Icons.Outlined.ExpandMore, null, tint = if (enabled) c.muted else c.faint)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }, containerColor = c.surface) {
            options.forEach { (label, key) ->
                DropdownMenuItem(
                    text = { Text(label, color = c.text) },
                    onClick = {
                        open = false
                        choose(key)
                    },
                    modifier = Modifier.testTag("$tag-$key"),
                )
            }
        }
    }
}

/** A setting that is on or off; `detail` under its name says what it does. */
@Composable
internal fun Toggle(
    title: String,
    on: Boolean,
    tag: String,
    detail: String? = null,
    enabled: Boolean = true,
    set: (Boolean) -> Unit,
) {
    val c = tonal()
    Row(
        Modifier.fillMaxWidth().heightIn(min = 56.dp).padding(horizontal = 16.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f)) {
            Text(title, fontSize = 16.sp, color = if (enabled) c.text else c.faint)
            detail?.let { Text(it, fontSize = 12.sp, color = c.muted) }
        }
        Switch(
            checked = on,
            onCheckedChange = set,
            enabled = enabled,
            colors = SwitchDefaults.colors(
                checkedThumbColor = c.surface,
                checkedTrackColor = c.accent,
                uncheckedThumbColor = c.outline,
                uncheckedTrackColor = c.field,
                uncheckedBorderColor = c.outline,
            ),
            modifier = Modifier.testTag(tag),
        )
    }
}
