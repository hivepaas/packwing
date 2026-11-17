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

    /** The checksums shown, by their keys, apart by spaces: the desktop's first two. */
    var checksums: String
        get() = prefs.getString(CHECKSUMS, null) ?: "sha256 sha1"
        set(value) = prefs.edit().putString(CHECKSUMS, value).apply()

    /** A public key the user trusts, by the name they gave it; `line` as `readKey` gives it. */
    data class TrustedKey(val name: String, val line: String)

    /** The keys a signature is trusted by: Settings › Keys you trust. */
    var trustedKeys: List<TrustedKey>
        get() = runCatching {
            val rows = JSONArray(prefs.getString(TRUSTED_KEYS, "[]"))
            (0 until rows.length()).map { at ->
                val row = rows.getJSONObject(at)
                TrustedKey(row.getString("name"), row.getString("line"))
            }
        }.getOrDefault(emptyList())
        set(value) {
            val rows = JSONArray()
            value.forEach { rows.put(JSONObject().put("name", it.name).put("line", it.line)) }
            prefs.edit().putString(TRUSTED_KEYS, rows.toString()).apply()
        }

    /** A storage across the network: its kind and settings; its secrets are in the vault, under its id. */
    data class StorageEntry(val id: String, val kind: String, val values: Map<String, String>)

    var storages: List<StorageEntry>
        get() = runCatching {
            val rows = JSONArray(prefs.getString(STORAGES, "[]"))
            (0 until rows.length()).map { at ->
                val row = rows.getJSONObject(at)
                val values = row.getJSONObject("values")
                StorageEntry(
                    row.getString("id"),
                    row.getString("kind"),
                    values.keys().asSequence().associateWith { values.getString(it) },
                )
            }
        }.getOrDefault(emptyList())
        set(value) {
            val rows = JSONArray()
            value.forEach {
                rows.put(JSONObject().put("id", it.id).put("kind", it.kind).put("values", JSONObject(it.values)))
            }
            prefs.edit().putString(STORAGES, rows.toString()).apply()
        }

    /** Whether a storage's folders show the files whose names start with a dot. */
    var storageHidden: Boolean
        get() = prefs.getBoolean(STORAGE_HIDDEN, false)
        set(value) = prefs.edit().putBoolean(STORAGE_HIDDEN, value).apply()

    /** The format of new archives, by its key (`zip`, `tar.zst`): one for many files. */
    var newFormat: String
        get() = prefs.getString(NEW_FORMAT, null) ?: "zip"
        set(value) = prefs.edit().putString(NEW_FORMAT, value).apply()

    /** How hard new archives are compressed, by its key (`medium`). */
    var newCompression: String
        get() = prefs.getString(NEW_COMPRESSION, null) ?: "medium"
        set(value) = prefs.edit().putString(NEW_COMPRESSION, value).apply()

    /** A 7z's names hidden with its password, as the desktop does unless told. */
    var hideNames: Boolean
        get() = prefs.getBoolean(HIDE_NAMES, true)
        set(value) = prefs.edit().putBoolean(HIDE_NAMES, value).apply()

    /** What is compressed already goes into a ZIP or 7z as it is. */
    var storeCompressed: Boolean
        get() = prefs.getBoolean(STORE_COMPRESSED, true)
        set(value) = prefs.edit().putBoolean(STORE_COMPRESSED, value).apply()

    /** What only a Mac uses is left out of new archives. */
    var skipMacFiles: Boolean
        get() = prefs.getBoolean(SKIP_MAC_FILES, true)
        set(value) = prefs.edit().putBoolean(SKIP_MAC_FILES, value).apply()

    /**
     * What goes in as it is, with [storeCompressed]: extensions apart by
     * spaces; empty for the desktop's list, which it then follows.
     */
    var storedExtensions: String
        get() = prefs.getString(STORED_EXTENSIONS, null) ?: ""
        set(value) = prefs.edit().putString(STORED_EXTENSIONS, value).apply()

    /** Above how many bytes an archive bound for a storage is written straight there, by its key. */
    var streamAbove: String
        get() = prefs.getString(STREAM_ABOVE, null) ?: StreamAbove.standard.key
        set(value) = prefs.edit().putString(STREAM_ABOVE, value).apply()

    /** Where an archive's files go in the folder chosen, by its key ([ExtractPlace]). */
    var extractTo: String
        get() = prefs.getString(EXTRACT_TO, null) ?: ExtractPlace.Smart.key
        set(value) = prefs.edit().putString(EXTRACT_TO, value).apply()

    /** The most an archive may unpack to, by its key ([ExtractLimit]). */
    var extractLimit: String
        get() = prefs.getString(EXTRACT_LIMIT, null) ?: ExtractLimit.standard.key
        set(value) = prefs.edit().putString(EXTRACT_LIMIT, value).apply()

    /** Light, dark, or as the system is, by its key ([Appearance]). */
    var appearance: String
        get() = prefs.getString(APPEARANCE, null) ?: Appearance.System.key
        set(value) = prefs.edit().putString(APPEARANCE, value).apply()

    /** The signing key signed with last, by its id, to start with. */
    var signingKey: String
        get() = prefs.getString(SIGNING_KEY, null) ?: ""
        set(value) = prefs.edit().putString(SIGNING_KEY, value).apply()

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
        const val CHECKSUMS = "checksums"
        const val TRUSTED_KEYS = "trustedKeys"
        const val STORAGES = "storages"
        const val NEW_FORMAT = "newFormat"
        const val SIGNING_KEY = "signingKey"
        const val NEW_COMPRESSION = "newCompression"
        const val HIDE_NAMES = "hideNames"
        const val STORE_COMPRESSED = "storeCompressed"
        const val SKIP_MAC_FILES = "skipMacFiles"
        const val STORED_EXTENSIONS = "storedExtensions"
        const val STREAM_ABOVE = "streamAbove"
        const val EXTRACT_TO = "extractTo"
        const val EXTRACT_LIMIT = "extractLimit"
        const val APPEARANCE = "appearance"
        const val STORAGE_HIDDEN = "storageHidden"
        const val RECENT = "recent"
        const val MOST_RECENT = 40
    }
}
