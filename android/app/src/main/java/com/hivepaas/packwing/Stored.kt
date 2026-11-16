package com.hivepaas.packwing

import android.net.Uri

/**
 * An archive on a storage, as a URI of no file: `packwing-storage://<id>/<path>?size=<bytes>`.
 * An archive opened from there is told by it (Recent, a password asked for),
 * opened again through its storage, and no file is looked for beside it.
 */
data class Stored(val id: String, val path: List<String>, val size: ULong) {
    val uri: Uri
        get() = Uri.Builder().scheme(SCHEME).authority(id)
            .apply { path.forEach { appendPath(it) } }
            .appendQueryParameter("size", size.toString())
            .build()

    companion object {
        const val SCHEME = "packwing-storage"

        /** What `uri` names, if it is an archive on a storage. */
        fun of(uri: Uri): Stored? {
            if (uri.scheme != SCHEME) return null
            val id = uri.authority ?: return null
            val path = uri.pathSegments.takeIf { it.isNotEmpty() } ?: return null
            return Stored(id, path, uri.getQueryParameter("size")?.toULongOrNull() ?: 0UL)
        }
    }
}
