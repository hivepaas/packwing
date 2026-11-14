package com.hivepaas.packwing

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.util.Log
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.packwing_mobile.Archive
import uniffi.packwing_mobile.ArchiveFormat
import uniffi.packwing_mobile.ArchiveInfo
import uniffi.packwing_mobile.CompressionLevel
import uniffi.packwing_mobile.EntryRow
import uniffi.packwing_mobile.ExtractHow
import uniffi.packwing_mobile.FoundRow
import uniffi.packwing_mobile.Job
import uniffi.packwing_mobile.JobListener
import uniffi.packwing_mobile.JobProgress
import uniffi.packwing_mobile.MobileException
import uniffi.packwing_mobile.NewArchiveHow
import uniffi.packwing_mobile.Secrets
import uniffi.packwing_mobile.createArchive
import java.io.Closeable
import java.io.File
import java.io.FileNotFoundException

/** An archive open: the core's, and what keeps its file reachable. */
class Opened(val uri: Uri, private val held: Documents.Opening, val archive: Archive) : Closeable {
    val info: ArchiveInfo = archive.info()

    override fun close() {
        runCatching { archive.close() }
        runCatching { held.close() }
    }
}

/**
 * What went wrong, in the user's language. Each kind of [MobileException]
 * holds its own `reason`; what `message` gives is the whole error written
 * out, for a log rather than for a person.
 */
val MobileException.said: String
    get() = when (this) {
        is MobileException.NotAnArchive -> reason
        is MobileException.Locked -> reason
        is MobileException.WrongPassword -> reason
        is MobileException.Cancelled -> reason
        is MobileException.Failed -> reason
    }

/** Where the app is. */
sealed interface Screen {
    data object Home : Screen
    data object Settings : Screen
    data object Create : Screen
    data object Running : Screen

    /** A folder of the open archive; `photos` shows its pictures as a grid. */
    data class Folder(val path: List<String>, val photos: Boolean = false) : Screen
}

/** What is asked over the screen. */
sealed interface Sheet {
    /** An archive that needs a password, or an age file that needs a passphrase. */
    data class Password(
        val uri: Uri,
        val name: String,
        val passphrase: Boolean,
        val problem: String? = null,
    ) : Sheet

    /** Extracting: all of it, or the paths picked. */
    data class Extract(val picked: List<List<String>> = emptyList()) : Sheet
}

/** Where what is extracted ends up. */
sealed interface Destination {
    /** A folder the user pointed at with the document picker. */
    data class Folder(val uri: Uri) : Destination

    /** The gallery, through MediaStore: pictures and videos only. */
    data object Gallery : Destination
}

/** A job going on, and how far it has got. */
class Running(val what: What, val name: String, val job: Job) {
    enum class What { Extract, Create }

    private val told = MutableStateFlow(JobProgress(0UL, 0UL, 0UL, 0UL))
    val progress: StateFlow<JobProgress> = told

    /** Told by the core, off the main thread. */
    val listener = object : JobListener {
        override fun progress(progress: JobProgress) {
            told.value = progress
        }
    }

    var done by mutableStateOf<String?>(null)
    var failed by mutableStateOf<String?>(null)
}

/**
 * What the app shows and does, in one place: the archive open, where the user
 * is in it, the jobs going on, and what was opened before. It belongs to the
 * process, not the window, because a job goes on while the app is not on
 * screen (ExtractService).
 */
class AppModel(private val context: Context) {
    val settings = Settings(context)
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)

    /** The screens the user came through; the last one is on top. */
    val stack = mutableStateListOf<Screen>(Screen.Home)
    val screen: Screen get() = stack.last()

    var opened by mutableStateOf<Opened?>(null)
        private set

    /** The name of the archive being opened, while it is. */
    var opening by mutableStateOf<String?>(null)
        private set

    var failure by mutableStateOf<String?>(null)

    /** What went well, said in passing: a job that is through. */
    var note by mutableStateOf<String?>(null)
    var sheet by mutableStateOf<Sheet?>(null)
    var running by mutableStateOf<Running?>(null)
        private set

    var recent by mutableStateOf(settings.recent)
        private set

    /** What the open folder holds, and what a search found. */
    var rows by mutableStateOf<List<EntryRow>>(emptyList())
        private set
    var query by mutableStateOf("")
        private set
    var found by mutableStateOf<List<FoundRow>>(emptyList())
        private set

    // MARK: Going about

    fun go(where: Screen) {
        stack.add(where)
        if (where is Screen.Folder) list(where.path)
    }

    /** Back a screen; false when there is nowhere to go, so the system takes it. */
    fun back(): Boolean {
        if (sheet != null) {
            sheet = null
            return true
        }
        if (stack.size <= 1) return false
        stack.removeAt(stack.size - 1)
        (screen as? Screen.Folder)?.let { list(it.path) }
        if (stack.none { it is Screen.Folder }) letGo()
        return true
    }

    fun home() {
        while (stack.size > 1) stack.removeAt(stack.size - 1)
        letGo()
    }

    private fun letGo() {
        opened?.close()
        opened = null
        rows = emptyList()
        found = emptyList()
        query = ""
    }

    // MARK: Opening

    /** Opens what another app handed us, or what the picker gave. */
    fun open(uri: Uri, secrets: Secrets = Secrets(null, null)) {
        val picked = Documents.picked(context, uri)
        Documents.keepAccess(context, uri)
        opening = picked.name
        failure = null
        scope.launch {
            val result = withContext(Dispatchers.IO) {
                runCatching {
                    val held = Documents.open(context, uri)
                    try {
                        Opened(uri, held, read(held, secrets))
                    } catch (e: Throwable) {
                        held.close()
                        throw e
                    }
                }
            }
            opening = null
            result.onSuccess { archive ->
                opened?.close()
                opened = archive
                sheet = null
                remember(archive)
                stack.add(Screen.Folder(emptyList(), photos = mostlyPictures(archive)))
                list(emptyList())
            }.onFailure { e -> failed(uri, picked.name, e) }
        }
    }

    /**
     * The core's archive, read however what was opened can be read: where it
     * asks, through the system's descriptor, or from the copy made for a
     * provider whose descriptor cannot be read at an offset.
     */
    private fun read(held: Documents.Opening, secrets: Secrets): Archive = when (held) {
        is Documents.Opening.AsDocument ->
            Archive.openDocument(held.document, secrets, Job.quiet())
        is Documents.Opening.AsFile ->
            Archive.open(held.path, secrets, Job.quiet())
    }

    /** What to say, or ask, when an archive would not open. */
    private fun failed(uri: Uri, name: String, e: Throwable) {
        Log.w("packwing", "could not open $name", e)
        when (e) {
            is MobileException.Locked ->
                sheet = Sheet.Password(uri, name, e.passphrase)
            is MobileException.WrongPassword -> {
                val asking = sheet as? Sheet.Password
                sheet = Sheet.Password(uri, name, asking?.passphrase ?: false, e.said)
            }
            is MobileException -> failure = e.said
            // One that was opened before and cannot be now: the system has
            // taken its grant back, or its file is gone. Tapping it again
            // would only fail again, so it does not stay in the list.
            is SecurityException, is FileNotFoundException -> {
                forget(uri)
                failure = tr(
                    "packwing can no longer reach {name}. Open it again to look inside.",
                    "name" to name,
                )
            }
            else -> failure = tr("packwing could not open {name}.", "name" to name)
        }
    }

    private fun remember(archive: Opened) {
        settings.remember(
            Settings.Recent(
                uri = archive.uri,
                name = archive.info.name,
                size = archive.info.packed,
                files = archive.info.files,
                locked = archive.info.encryption != null,
                opened = System.currentTimeMillis(),
            ),
        )
        recent = settings.recent
    }

    fun forget(uri: Uri) {
        settings.forget(uri)
        recent = settings.recent
    }

    /** Whether to open on the grid: an archive that is pictures, mostly. */
    private fun mostlyPictures(archive: Opened): Boolean {
        val top = runCatching { archive.archive.children(emptyList()) }.getOrDefault(emptyList())
        val pictures = top.count { !it.folder && isPicture(it.name) }
        return top.isNotEmpty() && pictures * 2 > top.size
    }

    // MARK: What is inside

    private fun list(folder: List<String>) {
        val archive = opened ?: return
        query = ""
        found = emptyList()
        scope.launch {
            rows = withContext(Dispatchers.IO) {
                runCatching { archive.archive.children(folder) }.getOrDefault(emptyList())
            }
        }
    }

    fun search(text: String) {
        query = text
        val archive = opened ?: return
        if (text.isBlank()) {
            found = emptyList()
            return
        }
        scope.launch {
            found = withContext(Dispatchers.IO) {
                runCatching { archive.archive.search(text) }.getOrDefault(emptyList())
            }
        }
    }

    /** What the file at `path` holds, at most `most` bytes, to show it. */
    suspend fun preview(path: List<String>, most: ULong = 32UL * 1024UL * 1024UL): ByteArray? {
        val archive = opened ?: return null
        return withContext(Dispatchers.IO) {
            runCatching { archive.archive.preview(path, most, Job.quiet()) }
                .onFailure { Log.i("packwing", "no preview for ${path.lastOrNull()}", it) }
                .getOrNull()
        }
    }

    // MARK: Extracting

    /**
     * Extracts `picked` (all of it when empty) into `destination`. The core
     * hands each entry over as it reads it and it is made where the user
     * asked for it there and then, so nothing is written twice.
     */
    fun extract(picked: List<List<String>>, destination: Destination) {
        val archive = opened ?: return
        if (running != null) return
        val name = archive.info.name.substringBeforeLast('.')
        val job = Running(Running.What.Extract, archive.info.name, Job.quiet())
        val held = Job(job.listener)
        running = job
        sheet = null
        go(Screen.Running)
        ExtractService.start(context)
        scope.launch {
            val result = withContext(Dispatchers.IO) {
                runCatching {
                    val into: Made = when (destination) {
                        is Destination.Folder ->
                            DocumentTree.under(context, destination.uri, name, settings.overwrite)
                        Destination.Gallery -> GalleryTree(context, name)
                    }
                    val done = try {
                        archive.archive
                            .extractInto(into, picked, ExtractHow(settings.overwrite), held)
                    } catch (e: Throwable) {
                        // Stopped, or it would not write: nothing is left behind.
                        into.undo()
                        throw e
                    }
                    // What the gallery took, not what the archive held.
                    if (into is GalleryTree) into.saved.toLong() else done.files.toLong()
                }
            }
            result.onSuccess { files ->
                job.done = trN(
                    files,
                    "{n} file is ready",
                    "{n} files are ready",
                    "n" to files.toString(),
                )
            }.onFailure { e ->
                job.failed = (e as? MobileException)?.said
                    ?: tr("packwing could not extract {name}.", "name" to archive.info.name)
            }
            finished(job)
        }
    }

    /** Makes an archive of `paths` (files the picker gave) under `name`. */
    fun create(paths: List<File>, name: String, how: NewArchiveHow, destination: Destination) {
        if (running != null) return
        val job = Running(Running.What.Create, name, Job.quiet())
        val held = Job(job.listener)
        running = job
        go(Screen.Running)
        ExtractService.start(context)
        scope.launch {
            val result = withContext(Dispatchers.IO) {
                val staging = Documents.staging(context, name)
                val wanted = File(staging, name + if (how.format == ArchiveFormat.ZIP) ".zip" else ".7z")
                runCatching {
                    createArchive(paths.map { it.path }, wanted.path, how, held)
                    when (destination) {
                        is Destination.Folder ->
                            Documents.exportToTree(context, staging, destination.uri, name)
                        Destination.Gallery -> 0
                    }
                }.also { staging.deleteRecursively() }
            }
            result.onSuccess { job.done = tr("{name} is ready", "name" to name) }
                .onFailure { e ->
                    job.failed = (e as? MobileException)?.said
                        ?: tr("packwing could not make {name}.", "name" to name)
                }
            finished(job)
        }
    }

    private fun finished(job: Running) {
        ExtractService.stop(context)
        val said = job.done
        if (running === job && screen is Screen.Running && job.failed == null) {
            // The progress screen stays only while there is progress to show,
            // so what it would have said is said in passing instead.
            note = said
            back()
        }
        if (said != null && settings.tellWhenDone) {
            ExtractService.tell(context, job.name, said)
        }
    }

    fun cancel() {
        running?.job?.cancel()
    }

    /** The job is over and its word has been read: the screen can forget it. */
    fun clearJob() {
        running = null
    }

    fun setLanguage(tag: String?) {
        settings.language = tag
        uniffi.packwing_mobile.setLanguage(tag)
    }

    companion object {
        fun isPicture(name: String): Boolean =
            Documents.mimeOf(name).startsWith("image/")

        fun isPictureOrVideo(name: String): Boolean =
            Documents.mimeOf(name).let { it.startsWith("image/") || it.startsWith("video/") }
    }
}
