package com.hivepaas.packwing

import android.content.Context
import android.content.Intent
import android.graphics.BitmapFactory
import android.net.Uri
import android.util.Log
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
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
import uniffi.packwing_mobile.KeyRead
import uniffi.packwing_mobile.MobileException
import uniffi.packwing_mobile.NewArchiveHow
import uniffi.packwing_mobile.OnExisting
import uniffi.packwing_mobile.Secrets
import uniffi.packwing_mobile.createArchive
import uniffi.packwing_mobile.RemoteStorage
import uniffi.packwing_mobile.transferItems
import uniffi.packwing_mobile.forgetStorageSecrets
import uniffi.packwing_mobile.keepStorageSecrets
import uniffi.packwing_mobile.pictureFits
import uniffi.packwing_mobile.readKey
import java.io.Closeable
import java.io.File
import java.io.FileNotFoundException
import java.util.UUID

/**
 * An archive open: the core's, what keeps its file reachable, the secrets it
 * was opened with, and whether packwing may write over its file.
 */
class Opened(
    val uri: Uri,
    /** What keeps its file reachable: a document's descriptor, a copy; nothing on a storage. */
    private val held: Closeable,
    val archive: Archive,
    val secrets: Secrets,
    private val writable: Boolean,
) : Closeable {
    val info: ArchiveInfo = archive.info()

    /** A ZIP or 7z of one file, which packwing may write over: files added, deleted, renamed. */
    val canChange: Boolean = writable && archive.canChange()

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
    /** A new archive: in a folder of the phone's, or `on` a storage's folder. */
    data class Create(val on: Storage? = null) : Screen
    data object Running : Screen

    /** A folder of the storage `id`: its path, empty for the top. */
    data class Storage(val id: String, val folder: List<String> = emptyList()) : Screen

    /** A storage to add: its kind and settings, tried before it is kept. */
    data object AddStorage : Screen

    /** A folder of the open archive; `photos` shows its pictures as a grid. */
    data class Folder(val path: List<String>, val photos: Boolean = false) : Screen

    /**
     * Checksums of the open archive: of `picked` (paths in it), of all of its
     * files, or of its own file, as asked first; `atOnce` reads those picked
     * without asking, for one file asked for alone.
     */
    data class Checksums(val picked: Picked = Picked(), val atOnce: Boolean = false) : Screen
}

/** Paths in the open archive, picked, and what they hold: files, and their bytes. */
data class Picked(
    val paths: List<List<String>> = emptyList(),
    val files: ULong = 0UL,
    val bytes: ULong = 0UL,
)

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
    data class Extract(val picked: Picked = Picked()) : Sheet

    /** Whether the open archive is what a key the user trusts signed. */
    data object Signature : Sheet

    /** A public key to trust: Settings › Keys you trust. */
    data object AddKey : Sheet
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

    /** Picking files and folders in the archive, and those picked, by their paths. */
    var selecting by mutableStateOf(false)
        private set
    var picked by mutableStateOf<Map<List<String>, EntryRow>>(emptyMap())
        private set

    /** The checksums shown, by their keys (`sha256 sha1`): Checksums' menu sets them. */
    var checksumKeys by mutableStateOf(settings.checksums)
        private set

    fun showChecksums(keys: String) {
        checksumKeys = keys
        settings.checksums = keys
    }

    /** The keys a signature is trusted by, each with its fingerprint. */
    var trustedKeys by mutableStateOf(settings.trustedKeys.map(::Trusted))
        private set

    /** A key trusted, and how it is told from others. */
    data class Trusted(val key: Settings.TrustedKey) {
        val fingerprint: String = runCatching { readKey(key.line).fingerprint }.getOrDefault("")
    }

    /**
     * Trusts `key`, under `name`, or the start of its fingerprint if that says
     * nothing. One trusted already keeps the name it has.
     */
    fun trust(key: KeyRead, name: String) {
        if (trustedKeys.any { it.key.line == key.line }) return
        val called = name.trim().ifEmpty { key.fingerprint.take(19) }
        trustedKeys = trustedKeys + Trusted(Settings.TrustedKey(called, key.line))
        settings.trustedKeys = trustedKeys.map { it.key }
    }

    fun distrust(line: String) {
        trustedKeys = trustedKeys.filter { it.key.line != line }
        settings.trustedKeys = trustedKeys.map { it.key }
    }

    /** What the user calls the key of `fingerprint`, or the fingerprint. */
    fun signer(fingerprint: String): String =
        trustedKeys.firstOrNull { it.fingerprint == fingerprint }?.key?.name ?: fingerprint

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
        if (where is Screen.Folder) {
            stopPicking()
            list(where.path)
        }
    }

    /** Back a screen; false when there is nowhere to go, so the system takes it. */
    fun back(): Boolean {
        if (sheet != null) {
            sheet = null
            return true
        }
        if (stack.size <= 1) return false
        if (selecting && screen is Screen.Folder) {
            stopPicking()
            return true
        }
        val left = stack.removeAt(stack.size - 1)
        // Back from a folder to the one above; not from Checksums, whose
        // folder is as it was, with what was picked in it.
        if (left is Screen.Folder) {
            stopPicking()
            (screen as? Screen.Folder)?.let { list(it.path) }
        }
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
        synchronized(pictures) { pictures.clear() }
        rows = emptyList()
        found = emptyList()
        query = ""
        stopPicking()
    }

    // MARK: Storages

    /** SFTP's questions waiting for the user, the first one shown (SshAsker). */
    val questions = mutableStateListOf<Asked>()

    /** Answers `asked`, and shows the next one. */
    fun answer(asked: Asked, said: SshAnswer) {
        questions.remove(asked)
        asked.answer(said)
    }

    /** The storages added: their kinds and settings. */
    var storages by mutableStateOf(settings.storages)
        private set

    /** The storages open, by id: a connection is kept for what follows. */
    private val connected = mutableMapOf<String, Pair<Settings.StorageEntry, RemoteStorage>>()

    /** `entry` open, the one open already if it is; it touches the network only once used. */
    fun storage(entry: Settings.StorageEntry): RemoteStorage = synchronized(connected) {
        connected[entry.id]?.takeIf { it.first == entry }?.second
            ?: RemoteStorage.open(entry.kind, entry.id, entry.values, null).also { connected[entry.id] = entry to it }
    }

    /** Keeps `entry`, its `secrets` in the vault. */
    fun addStorage(entry: Settings.StorageEntry, secrets: Map<String, String>) {
        keepStorageSecrets(entry.kind, entry.id, secrets)
        storages = storages + entry
        settings.storages = storages
    }

    /** Forgets `entry` and its secrets; its files stay where they are. */
    fun removeStorage(entry: Settings.StorageEntry) {
        forgetStorageSecrets(entry.kind, entry.id)
        storages = storages.filter { it.id != entry.id }
        settings.storages = storages
        synchronized(connected) { connected.remove(entry.id) }
        val gone = recent.filter { Stored.of(it.uri)?.id == entry.id }
        gone.forEach { forget(it.uri) }
    }

    /** Bumped once a storage was changed from here: its folders shown read themselves again. */
    var storagesChanged by mutableStateOf(0)
        private set

    /**
     * Does `work` on a storage, saying `title` and `name` while it does, with
     * Stop; then each storage's folder shown reads itself again. A storage
     * across the network has no Trash: a delete there is for good.
     */
    fun onStorage(title: String, name: String, after: () -> Unit = {}, work: (Job) -> Unit) {
        if (changing != null) return
        val job = Job.quiet()
        changeJob = job
        changing = title
        changingName = name
        scope.launch {
            val result = withContext(Dispatchers.IO) { runCatching { work(job) }.also { after() } }
            changing = null
            changingName = null
            changeJob = null
            storagesChanged++
            result.onFailure { e ->
                if (e is MobileException.Cancelled) return@onFailure
                Log.w("packwing", "could not do $title on a storage", e)
                failure = (e as? MobileException)?.said ?: e.message ?: e.toString()
            }
        }
    }

    /** What is changed, by name, while `changing` says how; the archive's name for one. */
    var changingName by mutableStateOf<String?>(null)
        private set

    /** One name, or how many there are. */
    private fun named(names: List<String>): String =
        names.singleOrNull() ?: tr("{many} items", "many" to names.size.toString())

    /**
     * Sends `files` (copies in the app's own folder, each with what is under
     * it) up into `folder` of the storage `entry`; the copies go once it is done.
     */
    fun upload(files: List<File>, entry: Settings.StorageEntry, folder: List<String>) {
        if (files.isEmpty()) return
        onStorage(tr("Uploading…"), named(files.map { it.name }), after = { files.forEach { it.parentFile?.deleteRecursively() } }) { job ->
            val to = storage(entry)
            files.groupBy { it.parentFile!! }.forEach { (from, here) ->
                transferItems(RemoteStorage.local(from.path), emptyList(), here.map { it.name }, to, folder, false, OnExisting.STOP, job)
            }
        }
    }

    /** Copies, or moves, `names` of `folder` of `entry` into `to`, a folder of a storage. */
    fun transfer(names: List<String>, entry: Settings.StorageEntry, folder: List<String>, to: Screen.Storage, moving: Boolean) {
        val target = storages.firstOrNull { it.id == to.id } ?: return
        onStorage(if (moving) tr("Moving…") else tr("Copying…"), named(names)) { job ->
            transferItems(storage(entry), folder, names, storage(target), to.folder, moving, OnExisting.STOP, job)
        }
    }

    /**
     * Copies `names` of `folder` of `entry` down into `tree`, a folder the
     * user pointed at: through the app's own folder, as a document tree has
     * no path to write to.
     */
    fun download(names: List<String>, entry: Settings.StorageEntry, folder: List<String>, tree: Uri) {
        val staging = Documents.staging(context)
        onStorage(tr("Download"), named(names), after = { staging.deleteRecursively() }) { job ->
            transferItems(storage(entry), folder, names, RemoteStorage.local(staging.path), emptyList(), false, OnExisting.STOP, job)
            names.forEach { Documents.copyToTree(context, File(staging, it), tree) }
        }
    }

    /** Opens the archive at `path` of the storage `entry`, of `size` bytes, where it is. */
    fun openStored(entry: Settings.StorageEntry, path: List<String>, size: ULong) =
        open(Stored(entry.id, path, size).uri)

    // MARK: Picking

    /** Picks `row`, at `path`, or lets it go; the first one picked starts picking. */
    fun pick(path: List<String>, row: EntryRow) {
        selecting = true
        // A path is a list, so `-` would take its parts for keys.
        picked = if (path in picked) picked.filterKeys { it != path } else picked + (path to row)
    }

    /** Picks every row shown, or lets them all go when they all are. */
    fun pickAll(shown: List<Pair<List<String>, EntryRow>>) {
        selecting = true
        picked = if (shown.all { it.first in picked }) emptyMap() else picked + shown
    }

    fun stopPicking() {
        selecting = false
        picked = emptyMap()
    }

    /** Those picked, with what they hold. */
    fun pickedNow(): Picked = picked.toPicked()

    // MARK: Opening

    /** Opens what another app handed us, or what the picker gave, or an archive on a storage. */
    fun open(uri: Uri, secrets: Secrets = Secrets(null, null)) {
        if (uri.scheme == Stored.SCHEME) return openStored(uri, secrets)
        val picked = Documents.picked(context, uri)
        Documents.keepAccess(context, uri)
        opening = picked.name
        failure = null
        scope.launch {
            val result = withContext(Dispatchers.IO) {
                runCatching { opening(uri, secrets) }
            }
            opening = null
            result.onSuccess { shown(it) }.onFailure { e -> failed(uri, picked.name, e) }
        }
    }

    /**
     * The archive on a storage `uri` names, opened where it is with `secrets`:
     * the core reads only what it needs of it. Off the main thread, as
     * opening a file is.
     */
    private fun openStored(uri: Uri, secrets: Secrets) {
        val stored = Stored.of(uri)
        val entry = stored?.let { s -> storages.firstOrNull { it.id == s.id } }
        if (stored == null || entry == null) {
            forget(uri)
            failure = tr("packwing can no longer reach {name}. Open it again to look inside.", "name" to (uri.lastPathSegment ?: ""))
            return
        }
        val name = stored.path.last()
        opening = name
        failure = null
        scope.launch {
            val result = withContext(Dispatchers.IO) {
                runCatching {
                    val archive = storage(entry).openArchive(stored.path, stored.size, secrets, Job.quiet())
                    Opened(uri, {}, archive, secrets, writable = false)
                }
            }
            opening = null
            result.onSuccess { shown(it) }.onFailure { e -> failed(uri, name, e) }
        }
    }

    /** `uri` opened with `secrets`, for the core to read. Call this off the main thread. */
    private fun opening(uri: Uri, secrets: Secrets): Opened {
        val held = Documents.open(context, uri)
        try {
            return Opened(uri, held, read(held, secrets), secrets, Documents.writable(context, uri))
        } catch (e: Throwable) {
            held.close()
            throw e
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

    /** An archive opened, shown: its top folder, as a grid if it is pictures mostly. */
    private fun shown(archive: Opened) {
        opened?.close()
        opened = archive
        sheet = null
        remember(archive)
        stack.add(Screen.Folder(emptyList(), photos = mostlyPictures(archive)))
        list(emptyList())
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

    /**
     * The pictures the grid has shown, so that scrolling back to one does not
     * read it out of the archive again. The oldest goes once there are more
     * than a screenful or two; a name that is not a picture is kept as none,
     * so it is not tried twice.
     */
    private val pictures = object : LinkedHashMap<String, ImageBitmap?>(16, 0.75f, true) {
        override fun removeEldestEntry(eldest: Map.Entry<String, ImageBitmap?>) = size > 120
    }

    /** How many pictures are read and decoded at once. */
    private val decoding = Semaphore(2)

    /**
     * A picture of what is at `path`, no smaller than `edge` across, for the
     * grid; none if it is not a picture, or is one too big to decode. The
     * whole entry is read to make it, as an archive holds no thumbnail of
     * its own.
     */
    suspend fun picture(path: List<String>, edge: Int): ImageBitmap? {
        val key = path.joinToString("/")
        synchronized(pictures) { if (pictures.containsKey(key)) return pictures[key] }
        val archive = opened ?: return null
        val made = decoding.withPermit {
            withContext(Dispatchers.IO) {
                runCatching {
                    val bytes = archive.archive.preview(path, MOST_PICTURE, Job.quiet())
                    if (pictureFits(bytes)) smallerBy(bytes, edge) else null
                }.onFailure { Log.i("packwing", "no picture of ${path.lastOrNull()}", it) }
                    .getOrNull()
            }
        }
        synchronized(pictures) { pictures[key] = made }
        return made
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
        stopPicking()
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

    /**
     * Makes an archive of `paths` (files the picker gave) called `name`, in
     * the folder `into`. The core writes it in the app's own folder first,
     * which a new archive still needs: it is written as it is made, in no
     * order, and a document tree takes a file only from its start.
     */
    /**
     * Makes an archive of `paths` called `name` (with its extension, as the
     * storage gets it) in `folder` of the storage `entry`: written there, or
     * made here first and sent, as the core sees fit.
     */
    fun create(paths: List<File>, name: String, how: NewArchiveHow, entry: Settings.StorageEntry, folder: List<String>) {
        if (running != null) return
        val job = Running(Running.What.Create, name, Job.quiet())
        val held = Job(job.listener)
        running = job
        go(Screen.Running)
        ExtractService.start(context)
        scope.launch {
            val result = withContext(Dispatchers.IO) {
                runCatching {
                    val made = storage(entry).createArchive(paths.map { it.path }, folder, name, how, held)
                    made to storage(entry).showPath(folder)
                }.also { paths.forEach { it.parentFile?.deleteRecursively() } }
            }
            storagesChanged++
            result.onSuccess { (called, where) -> job.done = tr("{name} in {folder}", "name" to called, "folder" to where) }
                .onFailure { e ->
                    job.failed = (e as? MobileException)?.said
                        ?: tr("packwing could not make {name}.", "name" to name)
                }
            finished(job)
        }
    }

    fun create(paths: List<File>, name: String, how: NewArchiveHow, into: Uri) {
        if (running != null) return
        val job = Running(Running.What.Create, name, Job.quiet())
        val held = Job(job.listener)
        running = job
        go(Screen.Running)
        ExtractService.start(context)
        scope.launch {
            val result = withContext(Dispatchers.IO) {
                val staging = Documents.staging(context)
                val wanted = File(staging, name + if (how.format == ArchiveFormat.ZIP) ".zip" else ".7z")
                runCatching {
                    val made = createArchive(paths.map { it.path }, wanted.path, how, held)
                    Documents.exportFileToTree(context, File(made), into)
                }.also { staging.deleteRecursively() }
            }
            result.onSuccess { called -> job.done = tr("{name} is ready", "name" to called) }
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

    // MARK: Changing

    /** What the archive is being changed by, as it says it: Adding…, Deleting…. */
    var changing by mutableStateOf<String?>(null)
        private set
    private var changeJob: Job? = null

    /**
     * Changes the open archive by `work` (given the archive, and the path of
     * the file to change), saying `title` while it does, and reads it again
     * once it is changed. Android gives no path to change it at, so a copy of
     * it is changed, in the app's own folder, and written over it: a change
     * that fails or is stopped leaves the copy, never the archive, as it was.
     * `after` is run either way, off the main thread: what was added is let go.
     */
    private fun change(title: String, after: () -> Unit = {}, work: (Archive, String, Job) -> Unit) {
        val now = opened ?: return
        if (changing != null || !now.canChange) return
        val job = Job.quiet()
        changeJob = job
        changing = title
        scope.launch {
            val result = withContext(Dispatchers.IO) {
                val dir = File(Documents.cache(context, "changing"), UUID.randomUUID().toString()).apply { mkdirs() }
                runCatching {
                    val copy = File(dir, now.info.name)
                    Documents.copyOut(context, now.uri, copy)
                    work(now.archive, copy.path, job)
                    Documents.writeOver(context, now.uri, copy)
                    opening(now.uri, now.secrets)
                }.also {
                    dir.deleteRecursively()
                    after()
                }
            }
            changing = null
            changeJob = null
            result.onSuccess { fresh ->
                if (opened !== now) {
                    fresh.close()
                    return@onSuccess
                }
                now.close()
                opened = fresh
                stopPicking()
                remember(fresh)
                (screen as? Screen.Folder)?.let { list(it.path) }
            }.onFailure { e ->
                if (e is MobileException.Cancelled) return@onFailure
                Log.w("packwing", "could not change ${now.info.name}", e)
                failure = (e as? MobileException)?.said ?: tr("packwing could not write {name}.", "name" to now.info.name)
            }
        }
    }

    /** Stops the change being made: the archive stays as it was. */
    fun stopChanging() {
        changeJob?.cancel()
    }

    /** Deletes `picked` from the archive, each with what is under it. */
    fun delete(picked: List<List<String>>) =
        change(tr("Deleting…")) { archive, file, job -> archive.delete(file, picked, job) }

    fun rename(path: List<String>, name: String) =
        change(tr("Renaming…")) { archive, file, job -> archive.rename(file, path, name, job) }

    /**
     * Adds `files` (copies in the app's own folder, each with what is under
     * it) to `folder` of the archive; one whose name is there goes as
     * `onExisting` says. The copies go once it is done.
     */
    fun add(files: List<File>, folder: List<String>, onExisting: OnExisting) =
        change(tr("Adding…"), after = { files.forEach { it.parentFile?.deleteRecursively() } }) { archive, file, job ->
            archive.add(file, files.map { it.path }, folder, onExisting, job)
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
        /** `rows`, by path, as picked: a folder's files and bytes are what is under it. */
        fun Map<List<String>, EntryRow>.toPicked(): Picked = Picked(
            paths = keys.toList(),
            files = values.sumOf { if (it.folder) it.files else 1UL },
            bytes = values.sumOf { it.size },
        )

        /** The most of an entry read to make a picture of it: 64 MiB. */
        val MOST_PICTURE: ULong = 64UL * 1024UL * 1024UL

        /**
         * `bytes` decoded no smaller than `edge` across, halving as many
         * times as it can on the way: a picture for a tile is a fraction of
         * the one in the archive, and decoding it whole only to shrink it
         * would be the whole of it in memory.
         */
        private fun smallerBy(bytes: ByteArray, edge: Int): ImageBitmap? {
            val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
            BitmapFactory.decodeByteArray(bytes, 0, bytes.size, bounds)
            if (bounds.outWidth <= 0 || bounds.outHeight <= 0) return null
            var by = 1
            while (
                bounds.outWidth / (by * 2) >= edge && bounds.outHeight / (by * 2) >= edge
            ) {
                by *= 2
            }
            val how = BitmapFactory.Options().apply { inSampleSize = by }
            return BitmapFactory.decodeByteArray(bytes, 0, bytes.size, how)?.asImageBitmap()
        }

        fun isPicture(name: String): Boolean =
            Documents.mimeOf(name).startsWith("image/")

        fun isPictureOrVideo(name: String): Boolean =
            Documents.mimeOf(name).let { it.startsWith("image/") || it.startsWith("video/") }
    }
}
