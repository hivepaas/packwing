package com.hivepaas.packwing.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.hivepaas.packwing.AppModel
import com.hivepaas.packwing.Running
import com.hivepaas.packwing.tr
import com.hivepaas.packwing.trN

/**
 * How far the job has got. The same numbers are in the notification, so the
 * app may be left; what the core cannot do is pause, only stop, so Cancel is
 * the only button while it runs.
 */
@Composable
fun RunningScreen(model: AppModel) {
    val c = tonal()
    val running = model.running ?: return
    val progress by running.progress.collectAsStateWithLifecycle()
    val share = if (progress.totalBytes > 0UL) {
        progress.bytes.toDouble() / progress.totalBytes.toDouble()
    } else {
        0.0
    }
    val over = running.done ?: running.failed

    Column(
        Modifier
            .fillMaxSize()
            .padding(top = WindowInsets.statusBars.asPaddingValues().calculateTopPadding())
            .padding(horizontal = 20.dp),
        verticalArrangement = Arrangement.spacedBy(20.dp),
    ) {
        Box(Modifier.height(32.dp))

        Row(verticalAlignment = Alignment.Bottom, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Text(
                "${(share * 100).toInt()}%",
                fontSize = 56.sp,
                fontWeight = FontWeight.Bold,
                color = c.onTonal,
            )
        }
        LinearProgressIndicator(
            progress = { share.toFloat() },
            modifier = Modifier.fillMaxWidth().height(8.dp),
            color = c.accent,
            trackColor = c.tonal,
        )

        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(
                when (running.what) {
                    Running.What.Extract -> tr("Extracting")
                    Running.What.Create -> tr("Making an archive")
                },
                fontSize = 24.sp,
                fontWeight = FontWeight.Bold,
                color = c.text,
            )
            Text(running.name, fontSize = 15.sp, color = c.muted)
        }

        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Count(
                Modifier.weight(1f),
                "${progress.entries} / ${progress.totalEntries}",
                trN(progress.totalEntries, "file", "files"),
            )
            Count(
                Modifier.weight(1f),
                sizeText(progress.bytes),
                tr("of {all}", "all" to sizeText(progress.totalBytes)),
            )
        }

        if (over == null) {
            Text(
                tr("You can leave the app. A notification shows how far it got."),
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(20.dp))
                    .background(c.tonal)
                    .padding(16.dp),
                fontSize = 14.sp,
                lineHeight = 20.sp,
                color = c.onTonal,
            )
            OutlinedButton(
                onClick = { model.cancel() },
                shape = RoundedCornerShape(28.dp),
                modifier = Modifier.fillMaxWidth().height(56.dp),
            ) {
                Text(tr("Cancel"), fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = c.text)
            }
        } else {
            Text(
                over,
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(20.dp))
                    .background(if (running.failed != null) c.badBg else c.tonal)
                    .padding(16.dp),
                fontSize = 15.sp,
                lineHeight = 21.sp,
                color = if (running.failed != null) c.badText else c.onTonal,
            )
            Button(
                onClick = {
                    model.clearJob()
                    model.back()
                },
                shape = RoundedCornerShape(28.dp),
                colors = ButtonDefaults.buttonColors(containerColor = c.accent, contentColor = c.onAccent),
                modifier = Modifier.fillMaxWidth().height(56.dp),
            ) {
                Text(tr("Done"), fontSize = 17.sp, fontWeight = FontWeight.Bold)
            }
        }
    }
}

@Composable
private fun Count(modifier: Modifier, big: String, small: String) {
    val c = tonal()
    Column(
        modifier
            .clip(RoundedCornerShape(20.dp))
            .background(c.surface)
            .padding(horizontal = 16.dp, vertical = 14.dp),
    ) {
        Text(big, fontSize = 20.sp, fontWeight = FontWeight.Bold, color = c.text)
        Text(small, fontSize = 13.sp, color = c.muted)
    }
}
