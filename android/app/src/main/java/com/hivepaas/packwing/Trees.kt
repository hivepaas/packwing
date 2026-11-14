package com.hivepaas.packwing

import android.content.ContentValues
import android.content.Context
import android.net.Uri
import android.provider.MediaStore
import android.util.Log
import androidx.documentfile.provider.DocumentFile
import uniffi.packwing_mobile.MobileException
import uniffi.packwing_mobile.Tree
import java.io.IOException
import java.io.OutputStream

/**
 * Where extracting writes. packwing-core hands over each entry as it reads
 * it, and it is made where the user asked for it there and then: nothing is
 * written twice, as it was when the core wrote into a folder of the app's own
 * and the app copied what it wrote across.
 *
 * One entry at a time, off the main thread. A folder on the way to a file
 * comes only if the archive holds an entry for it, so the ones between are
 * made here.
 *
 * Each keeps what it made, so that [undo] can take it away again: a job that
 * fails or is stopped leaves nothing, as it does on the desktop.
 */
interface Made : Tree {
    /** Takes away what this wrote, for a job that did not finish. */
    fun undo()
}
private fun fail(said: String): Nothing = throw MobileException.Failed(said)

/** The folder a destination writes into, named after the archive. */
private fun named(name: String) = name.ifBlank { "packwing" }

/** Into a folder the user pointed at with the document picker. */
class DocumentTree(
    private val context: Context,
    private val top: DocumentFile,
    private val overwrite: Boolean,
) : Made {

    private val folders = HashMap<String, DocumentFile>()
    private var writing: OutputStream? = null
    private var made: DocumentFile? = null

    /** What was not there before, newest last. */
    private val wrote = ArrayList<DocumentFile>()

    override fun `directory`(path: List<String>) {
        folder(path)
    }

    override fun `startFile`(path: List<String>, size: ULong?) {
        val name = path.lastOrNull() ?: fail(tr("packwing could not write a file with no name."))
        val into = folder(path.dropLast(1))
        into.findFile(name)?.takeIf { it.isFile }?.let { there ->
            if (!overwrite) fail(tr("{name} is already in the folder.", "name" to name))
            there.delete()
        }
        val file = into.createFile(Documents.saveMime(name), name)
            ?: fail(tr("packwing could not write {name}.", "name" to name))
        wrote.add(file)
        made = file
        writing = runCatching { context.contentResolver.openOutputStream(file.uri, "wt") }
            .getOrNull()
            ?: fail(tr("packwing could not write {name}.", "name" to name))
    }

    override fun `write`(data: ByteArray) {
        val out = writing ?: return
        try {
            out.write(data)
        } catch (e: IOException) {
            Log.w("packwing", "writing ${made?.name} stopped", e)
            fail(tr("packwing could not write {name}.", "name" to (made?.name ?: "")))
        }
    }

    override fun `finishFile`() {
        val out = writing ?: return
        writing = null
        made = null
        try {
            out.close()
        } catch (e: IOException) {
            Log.w("packwing", "closing a file stopped", e)
            fail(tr("packwing could not write what it extracted."))
        }
    }

    override fun `discardFile`() {
        runCatching { writing?.close() }
        runCatching { made?.delete() }
        writing = null
        made = null
    }

    /** The folder at `path`, and the ones above it, made as they are needed. */
    private fun folder(path: List<String>): DocumentFile {
        if (path.isEmpty()) return top
        val key = path.joinToString("/")
        folders[key]?.let { return it }
        val parent = folder(path.dropLast(1))
        val name = path.last()
        val there = parent.findFile(name)?.takeIf { it.isDirectory }
        val here = there
            ?: parent.createDirectory(name)
            ?: fail(tr("packwing could not make the folder {name}.", "name" to name))
        if (there == null) wrote.add(here)
        folders[key] = here
        return here
    }

    override fun undo() {
        runCatching { writing?.close() }
        writing = null
        made = null
        // Newest first: a file before the folder it is in.
        for (file in wrote.asReversed()) runCatching { file.delete() }
        wrote.clear()
        folders.clear()
    }

    companion object {
        /**
         * A folder called `name` in the tree the user pointed at, made if it
         * is not there, to write an archive's files into.
         */
        fun under(context: Context, tree: Uri, name: String, overwrite: Boolean): DocumentTree {
            val root = DocumentFile.fromTreeUri(context, tree)
                ?: fail(tr("packwing can no longer reach that folder."))
            val folder = named(name)
            val top = root.findFile(folder)?.takeIf { it.isDirectory }
                ?: root.createDirectory(folder)
                ?: fail(tr("packwing could not make the folder {name}.", "name" to folder))
            return DocumentTree(context, top, overwrite)
        }
    }
}

/**
 * Into the gallery, through MediaStore. Only the pictures and videos of an
 * archive go in, under `Pictures/packwing/<name>` and `Movies/…`, keeping the
 * folders the archive has them in; the gallery has nowhere to put the rest,
 * so the rest is left out.
 */
class GalleryTree(private val context: Context, private val name: String) : Made {

    private var writing: OutputStream? = null
    private var made: Uri? = null

    /** What was put in the gallery, to take out again if the job fails. */
    private val wrote = ArrayList<Uri>()

    /** How many landed: what the user is told, not what the core wrote. */
    var saved = 0
        private set

    override fun `directory`(path: List<String>) = Unit

    override fun `startFile`(path: List<String>, size: ULong?) {
        val file = path.lastOrNull() ?: return
        val mime = Documents.mimeOf(file)
        val video = mime.startsWith("video/")
        if (!video && !mime.startsWith("image/")) return
        val where = (listOf(if (video) "Movies" else "Pictures", "packwing", named(name)) +
            path.dropLast(1)).joinToString("/")
        val into = if (video) MediaStore.Video.Media.EXTERNAL_CONTENT_URI
        else MediaStore.Images.Media.EXTERNAL_CONTENT_URI
        val values = ContentValues().apply {
            put(MediaStore.MediaColumns.DISPLAY_NAME, file)
            put(MediaStore.MediaColumns.MIME_TYPE, mime)
            put(MediaStore.MediaColumns.RELATIVE_PATH, where)
            put(MediaStore.MediaColumns.IS_PENDING, 1)
        }
        val uri = runCatching { context.contentResolver.insert(into, values) }.getOrNull()
            ?: fail(tr("The gallery would not take {name}.", "name" to file))
        wrote.add(uri)
        made = uri
        writing = runCatching { context.contentResolver.openOutputStream(uri) }.getOrNull()
            ?: fail(tr("The gallery would not take {name}.", "name" to file))
    }

    override fun `write`(data: ByteArray) {
        val out = writing ?: return
        try {
            out.write(data)
        } catch (e: IOException) {
            Log.w("packwing", "writing to the gallery stopped", e)
            fail(tr("packwing could not write what it extracted."))
        }
    }

    override fun `finishFile`() {
        val out = writing ?: return
        val uri = made
        writing = null
        made = null
        runCatching { out.close() }
        if (uri == null) return
        runCatching {
            context.contentResolver.update(
                uri,
                ContentValues().apply { put(MediaStore.MediaColumns.IS_PENDING, 0) },
                null,
                null,
            )
        }
        saved++
    }

    override fun `discardFile`() {
        runCatching { writing?.close() }
        made?.let { uri ->
            wrote.remove(uri)
            runCatching { context.contentResolver.delete(uri, null, null) }
        }
        writing = null
        made = null
    }

    override fun undo() {
        runCatching { writing?.close() }
        writing = null
        made = null
        for (uri in wrote.asReversed()) {
            runCatching { context.contentResolver.delete(uri, null, null) }
        }
        wrote.clear()
        saved = 0
    }
}
