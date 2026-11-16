package com.hivepaas.packwing

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.ParcelFileDescriptor
import android.provider.DocumentsContract
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
 * answers them from the descriptor the system opened. Extracting costs no
 * copy either, and none of it is here: the core hands over each entry as it
 * reads it, and `Trees.kt` makes it where the user asked. What still costs a
 * copy is making an archive, because the core reads what goes in by path,
 * and reading from a provider whose descriptor cannot be read at an offset.
 * See android/README.md › How Android's storage shapes it.
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
     * Where a new archive is written before it is copied where the user
     * asked: the app's own folder, so nothing outside it is touched if the
     * job fails. Extracting needs none of this — it writes where the user
     * asked as it goes (`Trees.kt`).
     */
    fun staging(context: Context): File =
        File(cache(context, "making"), "${UUID.randomUUID()}").apply { mkdirs() }

    /**
     * Copies one file into the tree the user pointed at, under its own name,
     * and says what it ended up called: a new archive goes in that folder
     * itself, not in a folder of its own. A name already taken is not
     * written over — the provider gives the next one that is free, which is
     * why the name it chose is what comes back.
     */
    fun exportFileToTree(context: Context, from: File, tree: Uri): String {
        val root = DocumentFile.fromTreeUri(context, tree) ?: error("that folder is not reachable")
        val made = root.createFile(saveMime(from.name), from.name)
            ?: error("cannot write there")
        context.contentResolver.openOutputStream(made.uri, "wt")?.use { to ->
            from.inputStream().use { source -> source.copyTo(to, DEFAULT_BUFFER_SIZE) }
        } ?: error("cannot write there")
        return made.name ?: from.name
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
    fun saveMime(name: String): String =
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

    /**
     * Whether packwing may write over `uri`: a file the picker gave with
     * leave to, or one of its own. Another app's, handed over to be looked
     * at, is read only. Nothing is written to find out.
     */
    fun writable(context: Context, uri: Uri): Boolean = runCatching {
        context.contentResolver.openFileDescriptor(uri, "rw")?.use { true } ?: false
    }.getOrDefault(false)

    /** `uri`'s bytes, copied to `to`, which is made. Call this off the main thread. */
    fun copyOut(context: Context, uri: Uri, to: File) {
        context.contentResolver.openInputStream(uri)?.use { from ->
            to.outputStream().use { into -> from.copyTo(into, COPY_BUFFER) }
        } ?: error(tr("packwing could not read {name}.", "name" to to.name))
    }

    /**
     * `uri`'s bytes made `from`'s, in place: all an app may do to a file it
     * was given without its folder. Call this off the main thread.
     */
    fun writeOver(context: Context, uri: Uri, from: File) {
        context.contentResolver.openOutputStream(uri, "wt")?.use { into ->
            from.inputStream().use { it.copyTo(into, COPY_BUFFER) }
        } ?: error(tr("packwing could not write {name}.", "name" to from.name))
    }

    /**
     * The folder the user pointed at, copied into the app's own folder with
     * all it holds, under its own name: adding to an archive reads paths.
     * Call this off the main thread.
     */
    fun copyTreeIn(context: Context, tree: Uri): File {
        val root = DocumentFile.fromTreeUri(context, tree) ?: error(tr("packwing can no longer reach that folder."))
        val into = File(cache(context, "making"), UUID.randomUUID().toString()).apply { mkdirs() }
        fun copy(from: DocumentFile, to: File) {
            if (from.isDirectory) {
                to.mkdirs()
                from.listFiles().forEach { child -> child.name?.let { copy(child, File(to, it)) } }
            } else {
                context.contentResolver.openInputStream(from.uri)?.use { input ->
                    to.outputStream().use { input.copyTo(it, COPY_BUFFER) }
                }
            }
        }
        val folder = File(into, root.name ?: treeName(context, tree))
        copy(root, folder)
        return folder
    }

    /**
     * `from` (a file, or a folder with all it holds) copied into the tree the
     * user pointed at, under its own name; one there already of that name is
     * left as it is, and the provider gives the copy the next name free.
     * Call this off the main thread.
     */
    fun copyToTree(context: Context, from: File, tree: Uri) {
        val root = DocumentFile.fromTreeUri(context, tree) ?: error(tr("packwing can no longer reach that folder."))
        fun copy(file: File, into: DocumentFile) {
            if (file.isDirectory) {
                val made = into.findFile(file.name)?.takeIf { it.isDirectory } ?: into.createDirectory(file.name)
                    ?: error(tr("packwing could not make the folder {name}.", "name" to file.name))
                file.listFiles()?.forEach { copy(it, made) }
            } else {
                val made = into.createFile(saveMime(file.name), file.name)
                    ?: error(tr("packwing could not write {name}.", "name" to file.name))
                context.contentResolver.openOutputStream(made.uri, "wt")?.use { out ->
                    file.inputStream().use { it.copyTo(out, COPY_BUFFER) }
                } ?: error(tr("packwing could not write {name}.", "name" to file.name))
            }
        }
        copy(from, root)
    }

    /** A megabyte at a time: an archive is copied whole. */
    private const val COPY_BUFFER = 1 shl 20

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
