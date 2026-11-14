package com.hivepaas.packwing

import android.app.Notification
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import androidx.lifecycle.LifecycleService
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch

/**
 * Keeps a job going while packwing is not on screen, which is what Android
 * asks of work the user started and is waiting for: the notification says how
 * far it has got, and stops it.
 *
 * The work itself is the model's, in this same process; the service only holds
 * the process up and draws the notification.
 */
class ExtractService : LifecycleService() {

    override fun onCreate() {
        super.onCreate()
        val model = PackwingApp.model()
        lifecycleScope.launch {
            // One notification, told how far the job is ten times a second at
            // most, as the core tells it.
            model.running?.progress?.collectLatest { progress ->
                val running = model.running ?: return@collectLatest
                notify(running, progress.bytes, progress.totalBytes)
            }
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        super.onStartCommand(intent, flags, startId)
        if (intent?.action == STOP) {
            PackwingApp.model().cancel()
            stopSelf()
            return START_NOT_STICKY
        }
        val running = PackwingApp.model().running
        if (running == null) {
            stopSelf()
            return START_NOT_STICKY
        }
        ServiceCompat.startForeground(
            this,
            ONGOING,
            notification(running, 0UL, 0UL),
            if (Build.VERSION.SDK_INT >= 29) ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC else 0,
        )
        return START_NOT_STICKY
    }

    private fun notify(running: Running, bytes: ULong, total: ULong) {
        val manager = getSystemService(android.app.NotificationManager::class.java)
        manager.notify(ONGOING, notification(running, bytes, total))
    }

    private fun notification(running: Running, bytes: ULong, total: ULong): Notification {
        val open = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        val stop = PendingIntent.getService(
            this,
            1,
            Intent(this, ExtractService::class.java).setAction(STOP),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        val title = when (running.what) {
            Running.What.Extract -> tr("Extracting {name}", "name" to running.name)
            Running.What.Create -> tr("Making {name}", "name" to running.name)
        }
        val percent = if (total > 0UL) (bytes * 100UL / total).toInt() else 0
        return NotificationCompat.Builder(this, PackwingApp.JOB_CHANNEL)
            .setContentTitle(title)
            .setContentText(
                if (total > 0UL) {
                    tr(
                        "{done} of {all}",
                        "done" to com.hivepaas.packwing.ui.sizeText(bytes),
                        "all" to com.hivepaas.packwing.ui.sizeText(total),
                    )
                } else {
                    tr("Getting ready")
                },
            )
            .setSmallIcon(R.drawable.ic_notification)
            .setContentIntent(open)
            .setProgress(100, percent, total == 0UL)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .addAction(0, tr("Cancel"), stop)
            .setForegroundServiceBehavior(NotificationCompat.FOREGROUND_SERVICE_IMMEDIATE)
            .build()
    }

    companion object {
        private const val ONGOING = 1
        private const val STOP = "com.hivepaas.packwing.STOP"

        fun start(context: Context) {
            context.startForegroundService(Intent(context, ExtractService::class.java))
        }

        fun stop(context: Context) {
            context.stopService(Intent(context, ExtractService::class.java))
        }
    }
}
