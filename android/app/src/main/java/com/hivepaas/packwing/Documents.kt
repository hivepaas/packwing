package com.hivepaas.packwing

import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.ParcelFileDescriptor
import android.provider.DocumentsContract
import android.provider.MediaStore
import android.provider.OpenableColumns
import android.system.ErrnoException
import android.system.Os
import android.util.Log
import android.webkit.MimeTypeMap
import androidx.documentfile.provider.DocumentFile
import uniffi.packwing_mobile.Document
import uniffi.packwing_mobile.MobileException
import java.io.Closeable
import java.io.File
import java.util.UUID

/**
 * Where the app's files come from and go: Android hands out `content://` URIs,
 * so each way in and out is bridged here.
 *
 * An archive is read with no copy. The core asks for the ranges it wants
 * (`Document`, which crates/mobile exports for Android alone), and `pread`
 * answers them from the descriptor the system opened. Writing still costs a
 * copy, because a document tree cannot be written to by path: the core
 * extracts into the app's own folder, and what it wrote is then copied where
 * the user pointed. See android/README.md › How Android's storage shapes it.
 */
object Documents {

    /** What was picked: its URI, and what the system says about it. */
    data class Picked(val uri: Uri, val name: String, val size: Long)

    /** An archive's file, however it has to be read, until it is let go. */
    sealed interface Opening : Closeable {
        /**
         * Read where the core asks, through the descriptor the system
         * opened: nothing is copied.
         */
        class AsDocument(val document: OpenDocument) : Opening {
            override fun close() = document.close()
        }

        /**
         * A copy in the app's own folder, for a provider whose descriptor
         * cannot be read at an offset: a stream's cannot.
         */
        class AsFile(val path: String, private val remove: File?) : Opening {
            override fun close() {
                remove?.deleteRecursively()
            }
        }
    }

    /**
     * An archive the core reads where it asks. `pread` reads at an offset
     * without moving the descriptor's position, so reads can come in any
     * order and from more than one thread, and it asks no rights of its own
     * — unlike opening `/proc/self/fd/N`, which is a fresh open of the file
     * underneath, checked against the app's own rights rather than against
     * the grant that handed over the descriptor.
     */
    class OpenDocument(
        private val called: String,
        private val fd: ParcelFileDescriptor,
        private val length: Long,
    ) : Document, Closeable {

        override fun `name`(): String = called

        override fun `size`(): ULong = length.toULong()

        override fun `readAt`(offset: ULong, length: UInt): ByteArray {
            val at = offset.toLong()
            if (at < 0 || at >= this.length) return ByteArray(0)
            val want = minOf(length.toLong(), this.length - at).toInt()
            val buffer = ByteArray(want)
            var done = 0
            while (done < want) {
                val got = try {
                    Os.pread(fd.fileDescriptor, buffer, done, want - done, at + done)
                } catch (e: ErrnoException) {
                    Log.w("packwing", "could not read $called at ${at + done}", e)
                    throw MobileException.Failed(
                        tr("packwing could not read {name}.", "name" to called),
                    )
                }
                if (got <= 0) break
                done += got
            }
            return if (done == want) buffer else buffer.copyOf(done)
        }

        override fun close() {
            runCatching { fd.close() }
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
     * Opens `uri` for the core to read: through the system's descriptor where
     * that can be read at an offset, and otherwise from a copy in the app's
     * own folder. Call this off the main thread.
     */
    fun open(context: Context, uri: Uri): Opening {
        val name = picked(context, uri).name
        val fd = runCatching { context.contentResolver.openFileDescriptor(uri, "r") }.getOrNull()
        val length = fd?.let { readableAt(it) }
        if (fd != null && length != null) {
            Log.i("packwing", "reading $name where the core asks, with no copy")
            return Opening.AsDocument(OpenDocument(name, fd, length))
        }
        runCatching { fd?.close() }
        Log.i("packwing", "copying $uri in: its descriptor cannot be read at an offset")
        // Under a folder of its own, so the copy keeps the name the archive
        // came with: that name is what the app shows for it.
        val into = File(cache(context, "opened"), UUID.randomUUID().toString()).apply { mkdirs() }
        val copy = File(into, name)
        context.contentResolver.openInputStream(uri)?.use { from ->
            copy.outputStream().use { to -> from.copyTo(to, DEFAULT_BUFFER_SIZE) }
        } ?: error("cannot read the file")
        return Opening.AsFile(copy.path, into)
    }

    /**
     * How long the file behind `fd` is, if it can be read at an offset; none
     * if it cannot, as a pipe cannot — which is what a provider that makes
     * its answer up as it is read hands over.
     */
    private fun readableAt(fd: ParcelFileDescriptor): Long? = runCatching {
        val length = fd.statSize
        require(length >= 0)
        if (length > 0) Os.pread(fd.fileDescriptor, ByteArray(1), 0, 1, 0)
        length
    }.getOrNull()

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
                    ?: into.createFile(saveMime(child.name), child.name) ?: continue
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

    /**
     * What to make a file as, so that the folder it is written to keeps the
     * name it came with. A document provider checks the name's extension
     * against the kind it is handed and adds an extension of its own when the
     * two disagree, so the answer has to come from the table it asks:
     * `ghi chú.md`, made as `text/plain`, was written as `ghi chú.md.txt`.
     * A kind it does not know adds nothing.
     */
    private fun saveMime(name: String): String =
        MimeTypeMap.getSingleton()
            .getMimeTypeFromExtension(name.substringAfterLast('.', "").lowercase())
            ?: "application/octet-stream"

    /** What a name says the file is: what goes in the gallery, and as what. */
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
