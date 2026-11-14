package com.hivepaas.packwing

import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.ParcelFileDescriptor
import android.provider.DocumentsContract
import android.provider.MediaStore
import android.provider.OpenableColumns
import android.system.Os
import android.system.OsConstants
import android.util.Log
import androidx.documentfile.provider.DocumentFile
import java.io.Closeable
import java.io.File
import java.util.UUID

/**
 * Where the app's files come from and go: Android hands out `content://` URIs,
 * and packwing-core works on paths, so each way in and out is bridged here.
 *
 * Reading an archive needs no copy: the file descriptor the system opens is a
 * path of its own, `/proc/self/fd/N`, for as long as it is held. Writing does
 * need one, because a document tree cannot be written to by path: the core
 * extracts into the app's own folder, and what it wrote is then copied where
 * the user pointed. See android/README.md › What costs a second pass.
 */
object Documents {

    /** What was picked: its URI, and what the system says about it. */
    data class Picked(val uri: Uri, val name: String, val size: Long)

    /**
     * An archive's file as a path the core can open again and again, with
     * what keeps it reachable. Closed when the archive is let go.
     */
    class OpenFile(val path: String, private val holds: Closeable?, private val remove: File?) : Closeable {
        override fun close() {
            runCatching { holds?.close() }
            remove?.deleteRecursively()
        }
    }

    fun picked(context: Context, uri: Uri): Picked {
        var name = uri.lastPathSegment?.substringAfterLast('/') ?: "archive"
        var size = -1L
        runCatching {
            context.contentResolver.query(uri, null, null, null, null)?.use { row ->
                val nameAt = row.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                val sizeAt = row.getColumnIndex(OpenableColumns.SIZE)
                if (row.moveToFirst()) {
                    if (nameAt >= 0 && !row.isNull(nameAt)) name = row.getString(nameAt)
                    if (sizeAt >= 0 && !row.isNull(sizeAt)) size = row.getLong(sizeAt)
                }
            }
        }
        return Picked(uri, name, size)
    }

    /**
     * Opens `uri` as a path. A file on this device is its own path; anything
     * else is read through its descriptor, and only a provider that hands out
     * a pipe (which cannot be read twice, nor seeked) is copied to the cache
     * first.
     */
    fun open(context: Context, uri: Uri): OpenFile {
        if (uri.scheme == "file") {
            uri.path?.let { return OpenFile(it, null, null) }
        }
        val fd = runCatching { context.contentResolver.openFileDescriptor(uri, "r") }.getOrNull()
        if (fd != null && seekable(fd) && readableAgain(fd)) {
            Log.i("packwing", "reading $uri through its descriptor, with no copy")
            return OpenFile("/proc/self/fd/${fd.fd}", fd, null)
        }
        Log.i("packwing", "copying $uri to the cache: its descriptor cannot be opened again")
        runCatching { fd?.close() }
        // Under a folder of its own, so the copy keeps the name the archive
        // came with: that name is what the app shows for it.
        val into = File(cache(context, "opened"), UUID.randomUUID().toString()).apply { mkdirs() }
        val copy = File(into, picked(context, uri).name)
        context.contentResolver.openInputStream(uri)?.use { from ->
            copy.outputStream().use { to -> from.copyTo(to, DEFAULT_BUFFER_SIZE) }
        } ?: error("cannot read the file")
        return OpenFile(copy.path, null, into)
    }

    /** Whether the descriptor is a file that can be read again from any point. */
    private fun seekable(fd: ParcelFileDescriptor): Boolean = runCatching {
        Os.lseek(fd.fileDescriptor, 0, OsConstants.SEEK_SET)
        fd.statSize >= 0
    }.getOrDefault(false)

    /**
     * Whether `/proc/self/fd/N` opens again. Opening that path is a fresh
     * open of the file underneath, which is checked against the app's own
     * rights, not against the grant that handed us the descriptor: for a file
     * the document picker or MediaStore gave, the app has no rights of its
     * own and the open is refused. The core opens an archive's file once per
     * call, so a descriptor that will not open again is no use to it.
     */
    private fun readableAgain(fd: ParcelFileDescriptor): Boolean = runCatching {
        File("/proc/self/fd/${fd.fd}").inputStream().use { it.read() }
        true
    }.getOrDefault(false)

    /** The app's own folder for `what`, made if it is not there. */
    fun cache(context: Context, what: String): File =
        File(context.cacheDir, what).apply { mkdirs() }

    /**
     * Where the core writes before what it wrote is copied out: the app's own
     * folder, so nothing outside it is touched if a job fails.
     */
    fun staging(context: Context, name: String): File =
        File(cache(context, "extract"), "${UUID.randomUUID()}").apply { mkdirs() }

    /**
     * Copies everything under `from` into the document tree `tree`, keeping
     * its folders, under a folder named `folder`. Says how many files landed.
     */
    fun exportToTree(context: Context, from: File, tree: Uri, folder: String): Int {
        val root = DocumentFile.fromTreeUri(context, tree) ?: error("that folder is not reachable")
        val top = root.findFile(folder)?.takeIf { it.isDirectory }
            ?: root.createDirectory(folder)
            ?: error("cannot make a folder there")
        return copyInto(context, from, top)
    }

    private fun copyInto(context: Context, from: File, into: DocumentFile): Int {
        var written = 0
        for (child in from.listFiles().orEmpty()) {
            if (child.isDirectory) {
                val sub = into.findFile(child.name)?.takeIf { it.isDirectory }
                    ?: into.createDirectory(child.name) ?: continue
                written += copyInto(context, child, sub)
            } else {
                val existing = into.findFile(child.name)
                val file = existing?.takeIf { it.isFile }
                    ?: into.createFile(mimeOf(child.name), child.name) ?: continue
                context.contentResolver.openOutputStream(file.uri, "wt")?.use { to ->
                    child.inputStream().use { source -> source.copyTo(to, DEFAULT_BUFFER_SIZE) }
                    written++
                }
            }
        }
        return written
    }

    /**
     * Puts the pictures and videos under `from` in the gallery, through
     * MediaStore, under Pictures/packwing/`album`. Says how many landed.
     */
    fun saveToGallery(context: Context, from: File, album: String): Int {
        var saved = 0
        for (file in from.walkTopDown().filter { it.isFile }) {
            val mime = mimeOf(file.name)
            val video = mime.startsWith("video/")
            if (!video && !mime.startsWith("image/")) continue
            val folder = if (video) "Movies/packwing/$album" else "Pictures/packwing/$album"
            val into = if (video) MediaStore.Video.Media.EXTERNAL_CONTENT_URI
            else MediaStore.Images.Media.EXTERNAL_CONTENT_URI
            val values = ContentValues().apply {
                put(MediaStore.MediaColumns.DISPLAY_NAME, file.name)
                put(MediaStore.MediaColumns.MIME_TYPE, mime)
                put(MediaStore.MediaColumns.RELATIVE_PATH, folder)
                put(MediaStore.MediaColumns.IS_PENDING, 1)
            }
            val uri = runCatching { context.contentResolver.insert(into, values) }.getOrNull() ?: continue
            runCatching {
                context.contentResolver.openOutputStream(uri)?.use { to ->
                    file.inputStream().use { source -> source.copyTo(to, DEFAULT_BUFFER_SIZE) }
                }
                context.contentResolver.update(
                    uri,
                    ContentValues().apply { put(MediaStore.MediaColumns.IS_PENDING, 0) },
                    null, null,
                )
                saved++
            }.onFailure {
                Log.w("packwing", "the gallery would not take ${file.name}", it)
                runCatching { context.contentResolver.delete(uri, null, null) }
            }
        }
        return saved
    }

    /**
     * Copies what the picker gave into the app's own folder, under their own
     * names, and says where they landed: making an archive reads paths, and a
     * `content://` file has none. Call this off the main thread.
     */
    fun copyIn(context: Context, uris: List<Uri>): List<File> {
        val into = File(cache(context, "making"), UUID.randomUUID().toString()).apply { mkdirs() }
        return uris.mapNotNull { uri ->
            val name = picked(context, uri).name
            val file = File(into, name)
            runCatching {
                context.contentResolver.openInputStream(uri)?.use { from ->
                    file.outputStream().use { to -> from.copyTo(to, DEFAULT_BUFFER_SIZE) }
                } ?: return@mapNotNull null
                file
            }.getOrNull()
        }
    }

    /** What a name says the file is, for the providers that ask. */
    fun mimeOf(name: String): String = when (name.substringAfterLast('.', "").lowercase()) {
        "jpg", "jpeg" -> "image/jpeg"
        "png" -> "image/png"
        "gif" -> "image/gif"
        "webp" -> "image/webp"
        "heic" -> "image/heic"
        "bmp" -> "image/bmp"
        "tif", "tiff" -> "image/tiff"
        "svg" -> "image/svg+xml"
        "mp4", "m4v" -> "video/mp4"
        "mkv" -> "video/x-matroska"
        "mov" -> "video/quicktime"
        "webm" -> "video/webm"
        "avi" -> "video/x-msvideo"
        "pdf" -> "application/pdf"
        "txt", "md", "log", "csv" -> "text/plain"
        "json" -> "application/json"
        "xml" -> "text/xml"
        "html", "htm" -> "text/html"
        "zip" -> "application/zip"
        "7z" -> "application/x-7z-compressed"
        "rar" -> "application/vnd.rar"
        else -> "application/octet-stream"
    }

    /** Keeps reaching `uri` after the app is closed and opened again. */
    fun keepAccess(context: Context, uri: Uri, write: Boolean = false) {
        val flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or
            if (write) Intent.FLAG_GRANT_WRITE_URI_PERMISSION else 0
        runCatching { context.contentResolver.takePersistableUriPermission(uri, flags) }
            .onFailure { Log.i("packwing", "no lasting access to $uri", it) }
    }

    /** What the document tree is called, to show which folder was chosen. */
    fun treeName(context: Context, tree: Uri): String =
        DocumentFile.fromTreeUri(context, tree)?.name
            ?: DocumentsContract.getTreeDocumentId(tree).substringAfterLast(':').ifEmpty { "chosen folder" }

    /** The kinds of file the picker offers, as the manifest lists them. */
    val archiveMimeTypes = arrayOf(
        "application/zip",
        "application/x-zip-compressed",
        "application/x-7z-compressed",
        "application/vnd.rar",
        "application/x-rar-compressed",
        "application/x-tar",
        "application/gzip",
        "application/x-gzip",
        "application/x-bzip2",
        "application/x-xz",
        "application/zstd",
        "application/vnd.comicbook+zip",
        "application/vnd.debian.binary-package",
        "application/x-iso9660-image",
        "application/octet-stream",
    )
}
