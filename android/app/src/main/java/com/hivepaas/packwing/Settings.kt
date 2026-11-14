package com.hivepaas.packwing

import android.content.Context
import android.net.Uri
import org.json.JSONArray
import org.json.JSONObject

/**
 * What the user chose, kept between runs: the language, where files go, and
 * the archives opened before.
 */
class Settings(context: Context) {
    private val prefs = context.getSharedPreferences("packwing", Context.MODE_PRIVATE)

    /** The language chosen, by tag; none for the system's. */
    var language: String?
        get() = prefs.getString(LANGUAGE, null)
        set(value) = prefs.edit().apply { if (value == null) remove(LANGUAGE) else putString(LANGUAGE, value) }.apply()

    /** The folder extraction last wrote to, chosen through the document picker. */
    var destination: Uri?
        get() = prefs.getString(DESTINATION, null)?.let(Uri::parse)
        set(value) = prefs.edit().apply {
            if (value == null) remove(DESTINATION) else putString(DESTINATION, value.toString())
        }.apply()

    /** Keeps the archive's own folders, rather than every file in one. */
    var keepFolders: Boolean
        get() = prefs.getBoolean(KEEP_FOLDERS, true)
        set(value) = prefs.edit().putBoolean(KEEP_FOLDERS, value).apply()

    /** Replaces a file that is already there, instead of stopping to ask. */
    var overwrite: Boolean
        get() = prefs.getBoolean(OVERWRITE, false)
        set(value) = prefs.edit().putBoolean(OVERWRITE, value).apply()

    var tellWhenDone: Boolean
        get() = prefs.getBoolean(TELL_WHEN_DONE, true)
        set(value) = prefs.edit().putBoolean(TELL_WHEN_DONE, value).apply()

    /** An archive opened before, as Recent lists it. */
    data class Recent(
        val uri: Uri,
        val name: String,
        val size: ULong,
        val files: ULong,
        val locked: Boolean,
        val opened: Long,
    )

    var recent: List<Recent>
        get() = runCatching {
            val rows = JSONArray(prefs.getString(RECENT, "[]"))
            (0 until rows.length()).map { at ->
                val row = rows.getJSONObject(at)
                Recent(
                    uri = Uri.parse(row.getString("uri")),
                    name = row.getString("name"),
                    size = row.optLong("size", 0).toULong(),
                    files = row.optLong("files", 0).toULong(),
                    locked = row.optBoolean("locked", false),
                    opened = row.optLong("opened", 0),
                )
            }
        }.getOrDefault(emptyList())
        set(value) {
            val rows = JSONArray()
            for (row in value.take(MOST_RECENT)) {
                rows.put(
                    JSONObject().apply {
                        put("uri", row.uri.toString())
                        put("name", row.name)
                        put("size", row.size.toLong())
                        put("files", row.files.toLong())
                        put("locked", row.locked)
                        put("opened", row.opened)
                    },
                )
            }
            prefs.edit().putString(RECENT, rows.toString()).apply()
        }

    /** Puts `row` at the top of Recent, in place of an older look at the same file. */
    fun remember(row: Recent) {
        recent = listOf(row) + recent.filter { it.uri != row.uri }
    }

    fun forget(uri: Uri) {
        recent = recent.filter { it.uri != uri }
    }

    private companion object {
        const val LANGUAGE = "language"
        const val DESTINATION = "destination"
        const val KEEP_FOLDERS = "keepFolders"
        const val OVERWRITE = "overwrite"
        const val TELL_WHEN_DONE = "tellWhenDone"
        const val RECENT = "recent"
        const val MOST_RECENT = 40
    }
}
