package com.hivepaas.packwing

import android.app.Application
import android.app.NotificationChannel
import android.app.NotificationManager
import uniffi.packwing_mobile.start

/**
 * The app's one process: gets the core ready before anything asks it
 * something, and keeps the model, which outlives the window because a job
 * goes on while the app is not on screen.
 */
class PackwingApp : Application() {
    lateinit var model: AppModel
        private set

    override fun onCreate() {
        super.onCreate()
        app = this
        // The journal of extractions lives in the app's own folder, so what a
        // job left behind when the app was ended is cleaned up next time.
        start(filesDir.absolutePath, Settings(this).language)
        model = AppModel(this)
        notifications()
    }

    private fun notifications() {
        val manager = getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(
            NotificationChannel(
                JOB_CHANNEL,
                tr("Extracting and making archives"),
                NotificationManager.IMPORTANCE_LOW,
            ).apply {
                description = tr("How far a job has got, while packwing is not on screen")
                setShowBadge(false)
            },
        )
        manager.createNotificationChannel(
            NotificationChannel(
                DONE_CHANNEL,
                tr("Finished jobs"),
                NotificationManager.IMPORTANCE_DEFAULT,
            ).apply { description = tr("When a job is through, or could not finish") },
        )
    }

    companion object {
        const val JOB_CHANNEL = "job"
        const val DONE_CHANNEL = "done"

        private lateinit var app: PackwingApp

        /** The model, from anywhere in the process. */
        fun model(): AppModel = app.model
    }
}
