package com.hivepaas.packwing

import android.Manifest
import android.net.Uri
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.isEnabled
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.AndroidComposeTestRule
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTouchInput
import androidx.test.ext.junit.rules.ActivityScenarioRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.GrantPermissionRule
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Assume.assumeTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File

/**
 * The app driven as a person would, on a device or the emulator
 * (make android-test), over the archives the iOS app's tests open
 * (ios/PackwingUITests/Samples): opened, looked through, searched, previewed,
 * unlocked, shown as pictures and extracted.
 */
@OptIn(ExperimentalTestApi::class)
@RunWith(AndroidJUnit4::class)
class AppTests {
    /**
     * A job asks for notifications the first time; granted, the system's
     * question does not come over the app mid-test.
     */
    @get:Rule(order = 0)
    val notifications: GrantPermissionRule = GrantPermissionRule.grant(Manifest.permission.POST_NOTIFICATIONS)

    @get:Rule(order = 1)
    val ui = createAndroidComposeRule<MainActivity>()

    private val model: AppModel get() = PackwingApp.model()

    @Before
    fun startAtHome() = onMain {
        model.home()
        // The last test's keyboard, if it typed, is not this one's.
        val window = ui.activity.window
        androidx.core.view.WindowCompat.getInsetsController(window, window.decorView)
            .hide(androidx.core.view.WindowInsetsCompat.Type.ime())
    }

    @After
    fun leave() = onMain {
        model.sheet = null
        model.home()
        // A job that is through is forgotten, as its screen forgets it once read.
        model.clearJob()
    }

    @Test
    fun anArchiveIsLookedThroughSearchedAndPreviewed() {
        open("report.zip")
        ui.waitForText("Báo cáo quý 3")
        ui.onNodeWithText("Báo cáo quý 3").performClick()
        ui.waitForText("Doanh thu.csv")
        ui.onNodeWithText("Ghi chú.txt").performClick()
        // A text is shown as itself, with no extracting.
        ui.waitUntilAtLeastOneExists(hasText("Close"), TIMEOUT)
        ui.onNodeWithText("Close").performClick()

        // Without its accents, as a person types on a phone.
        ui.onNode(hasSetTextAction()).performTextInput("doanh thu")
        ui.waitForText("Doanh thu.csv")
    }

    @Test
    fun aLockedArchiveAsksForItsPassword() {
        open("locked.7z")
        ui.waitForText("Enter the password")
        typePassword("not it")
        ui.waitForText("The password is wrong", substring = true)

        ui.onNode(hasSetTextAction()).performTextClearance()
        typePassword("sample-password")
        // Its names were hidden: there now.
        until { model.opened != null && model.sheet == null }
        ui.waitForText("Báo cáo quý 3")
    }

    @Test
    fun anArchiveOfPicturesIsAGridAndGoesToTheGallery() {
        open("photos.zip")
        // A grid shows a picture as itself; its name is its description.
        ui.waitUntilAtLeastOneExists(
            androidx.compose.ui.test.hasContentDescription("IMG_0001.jpg"),
            TIMEOUT,
        )
        onMain { model.extract(emptyList(), Destination.Gallery) }
        // The text is what the gallery took: the four pictures, not the note.
        ui.waitForText("4 files are ready")
    }

    @Test
    fun picturesPickedGoToTheGalleryAlone() {
        open("photos.zip")
        ui.waitUntilAtLeastOneExists(hasTestTag("entry-IMG_0001.jpg"), TIMEOUT)
        // A long press starts picking; a tap then picks another.
        ui.onNodeWithTag("entry-IMG_0001.jpg").performTouchInput { longClick() }
        ui.onNodeWithTag("entry-IMG_0002.jpg").performClick()
        ui.waitForText("2 selected")
        ui.onNodeWithTag("extract").performClick()
        ui.waitForText("Extract 2 files")
        ui.onNodeWithText("The gallery").performClick()
        ui.onNodeWithText("Extract the pictures").performClick()
        ui.waitForText("2 files are ready")
    }

    @Test
    fun anArchiveSChecksumIsComparedWithTheOneGiven() {
        open("report.zip")
        ui.waitForText("Báo cáo quý 3")
        ui.onNodeWithTag("archive-menu").performClick()
        ui.onNodeWithText("Checksums").performClick()
        ui.onNodeWithTag("checksum-scope-file").performClick()
        ui.onNodeWithTag("checksum-read").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("checksum-sha256") and hasText(REPORT_SHA256), TIMEOUT)

        ui.onNodeWithTag("checksum-pasted").performTextInput(REPORT_SHA256.uppercase())
        ui.waitUntilAtLeastOneExists(hasTestTag("checksum-matches"), TIMEOUT)

        // One not made yet is made when asked for.
        ui.onNodeWithTag("checksum-pasted").performTextClearance()
        ui.onNodeWithTag("checksum-pasted").performTextInput(REPORT_MD5)
        ui.waitForText("Make it")
        ui.onNodeWithText("Make it").performClick()
        ui.waitForText("It matches the file’s MD5 checksum.")
    }

    @Test
    fun theFilesInAnArchiveHaveTheirChecksumsListedAndCompared() {
        open("report.zip")
        ui.waitForText("Báo cáo quý 3")
        ui.onNodeWithTag("archive-menu").performClick()
        ui.onNodeWithText("Checksums").performClick()
        // Nothing picked: all of its files.
        ui.onNodeWithTag("checksum-read").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("checksum-file-2"), TIMEOUT)

        // One checksum alone: the file it is of.
        val pasted = ui.onNodeWithTag("checksums-pasted")
        pasted.performTextInput(NOTE_SHA256)
        ui.waitForText("It matches the SHA-256 checksum of Báo cáo quý 3/Ghi chú.txt.")

        // A list as sha256sum writes one, with a line that is not right.
        pasted.performTextClearance()
        pasted.performTextInput(REPORT_LIST.replace(NOTE_SHA256, "0".repeat(64)))
        ui.waitUntilAtLeastOneExists(hasTestTag("checksum-differs"), TIMEOUT)
        pasted.performTextClearance()
        pasted.performTextInput(REPORT_LIST)
        ui.waitUntilAtLeastOneExists(hasTestTag("checksum-all-match"), TIMEOUT)

        // Copy all: the list of the first checksum shown, as sha256sum writes it.
        ui.onNodeWithTag("checksum-copy-all").performClick()
        var copied = ""
        onMain {
            copied = app.getSystemService(android.content.ClipboardManager::class.java)
                .primaryClip!!.getItemAt(0).text.toString()
        }
        assertEquals(REPORT_LIST, copied)
    }

    @Test
    fun aFileSChecksumsAreReadFromItsMenu() {
        open("report.zip")
        ui.waitForText("Báo cáo quý 3")
        ui.onNodeWithText("Báo cáo quý 3").performClick()
        ui.waitForText("Ghi chú.txt")
        ui.onNodeWithTag("menu-Ghi chú.txt").performClick()
        ui.onNodeWithText("Checksums").performClick()
        // Read at once, not asked what.
        ui.waitUntilAtLeastOneExists(hasTestTag("checksum-sha256") and hasText(NOTE_SHA256), TIMEOUT)
    }

    @Test
    fun anArchiveSSignatureSaysWhoSignedItOnceItsKeyIsTrusted() {
        forgetKeys()
        // report.zip.sig: report.zip signed with `ssh-keygen -Y sign -n file`, beside it.
        sample("report.zip.sig")
        open("report.zip")
        ui.waitForText("Báo cáo quý 3")
        ui.onNodeWithTag("archive-menu").performClick()
        ui.onNodeWithText("Signature").performClick()

        // By a key not trusted yet, which proves nothing.
        ui.waitForText("Signed by a key you do not trust: $SIGNER")
        ui.onNodeWithTag("trust-key").performClick()
        ui.onNodeWithTag("key-name").performTextInput("Tester")
        ui.onNodeWithTag("key-trust").performClick()
        // Checked again, against all of it.
        ui.waitForText("Signed by Tester")

        // Kept in Settings.
        onMain {
            model.sheet = null
            model.home()
            model.go(Screen.Settings)
        }
        ui.waitForText("Tester")
        ui.waitForText(SIGNER)

        // Beside an archive it is not of: by a key trusted, and it does not match.
        sample("report.zip.sig", "photos.zip.sig")
        open("photos.zip")
        ui.waitUntilAtLeastOneExists(hasTestTag("archive-menu"), TIMEOUT)
        ui.onNodeWithTag("archive-menu").performClick()
        ui.onNodeWithText("Signature").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("signature-mismatch"), TIMEOUT)
        forgetKeys()
    }

    @Test
    fun aPublicKeyPastedInSettingsIsTrusted() {
        forgetKeys()
        onMain { model.go(Screen.Settings) }
        ui.waitUntilAtLeastOneExists(hasTestTag("add-key"), TIMEOUT)
        ui.onNodeWithTag("add-key").performClick()
        ui.onNodeWithTag("key-text").performTextInput("not a key")
        ui.onNodeWithTag("key-text").performTextClearance()
        ui.onNodeWithTag("key-text").performTextInput("$SIGNER_KEY tester@example")
        // Its fingerprint, as ssh-keygen -l says it, once it reads as a key.
        ui.waitForText(SIGNER)
        ui.onNodeWithTag("key-name").performTextInput("Pasted")
        ui.onNodeWithTag("key-trust").performClick()
        ui.waitForText("Pasted")
        until { model.trustedKeys.singleOrNull()?.fingerprint == SIGNER }
        forgetKeys()
    }

    @Test
    fun filesInAnArchiveAreRenamedDeletedAndAdded() {
        val copy = open("report.zip")
        ui.waitForText("Báo cáo quý 3")
        ui.onNodeWithText("Báo cáo quý 3").performClick()
        ui.waitForText("Ghi chú.txt")

        ui.onNodeWithTag("menu-Ghi chú.txt").performClick()
        ui.onNodeWithText("Rename").performClick()
        ui.onNodeWithTag("new-name").performTextClearance()
        ui.onNodeWithTag("new-name").performTextInput("Ghi chú mới.txt")
        ui.onNodeWithTag("name-confirm").performClick()
        ui.waitForText("Ghi chú mới.txt")
        until { model.changing == null }

        // Picked, then deleted.
        ui.onNodeWithTag("entry-Doanh thu.csv").performTouchInput { longClick() }
        ui.onNodeWithTag("delete-picked").performClick()
        ui.onNodeWithTag("delete-confirm").performClick()
        until { model.changing == null && model.rows.none { it.name == "Doanh thu.csv" } && model.rows.isNotEmpty() }

        // A file added, as the picker's copy of it is.
        val added = File(File(app.cacheDir, "making/added").apply { mkdirs() }, "Thêm.txt").apply { writeText("thêm") }
        onMain { model.add(listOf(added), listOf("Báo cáo quý 3"), uniffi.packwing_mobile.OnExisting.STOP) }
        ui.waitForText("Thêm.txt")
        until { model.changing == null }

        // Written over the file it was opened from.
        val now = uniffi.packwing_mobile.Archive.open(copy.path, uniffi.packwing_mobile.Secrets(null, null, null, null), uniffi.packwing_mobile.Job.quiet())
        assertEquals(
            setOf("Hình ảnh", "Ghi chú mới.txt", "Thêm.txt"),
            now.children(listOf("Báo cáo quý 3")).map { it.name }.toSet(),
        )
        now.close()
    }

    @Test
    fun anArchivePackwingMayNotWriteIsNotChanged() {
        sample("report.zip").setReadOnly()
        onMain { model.open(Uri.fromFile(File(app.cacheDir, "samples/report.zip"))) }
        ui.waitForText("Báo cáo quý 3")
        until { model.opened?.canChange == false }
        ui.onNodeWithTag("add-menu").assertDoesNotExist()
        File(app.cacheDir, "samples/report.zip").setWritable(true)
    }

    /**
     * Against the WebDAV server of `make webdav-start`, which the emulator
     * reaches at 10.0.2.2: skipped without one.
     */
    @Test
    fun anArchiveOnAStorageIsOpenedWhereItIsAndItsPasswordKept() {
        assumeTrue("no WebDAV server: make webdav-start", webdav("PROPFIND", "") in 200..299)
        val folder = "ui-${java.util.UUID.randomUUID().toString().take(8)}"
        assertTrue(webdav("MKCOL", folder) in 200..299)
        val zip = InstrumentationRegistry.getInstrumentation().context.assets.open("report.zip").use { it.readBytes() }
        assertTrue(webdav("PUT", "$folder/report.zip", zip) in 200..299)
        onMain { model.storages.forEach { model.removeStorage(it) } }

        onMain { model.go(Screen.AddStorage) }
        ui.onNodeWithTag("storage-kind-webdav").performClick()
        ui.onNodeWithTag("storage-address").performTextInput("http://$HOST:$WEBDAV_PORT")
        ui.onNodeWithTag("storage-user").performTextInput("packwing")
        ui.onNodeWithTag("storage-password").performTextInput("packwing")
        ui.onNodeWithTag("storage-add").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("storage-row"), TIMEOUT)

        // Its password is in the vault, sealed by the Keystore, not with its settings.
        var entry: Settings.StorageEntry? = null
        onMain { entry = model.storages.single() }
        assertEquals("packwing", uniffi.packwing_mobile.keptStorageSecrets(entry!!.kind, entry!!.id)["password"])
        assertFalse(entry!!.values.containsKey("password"))

        ui.onNodeWithTag("storage-row").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("stored-$folder"), TIMEOUT)
        ui.onNodeWithTag("stored-$folder").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("stored-report.zip"), TIMEOUT)
        ui.onNodeWithTag("stored-report.zip").performClick()
        // Read where it is, its folders listed as a file's are.
        ui.waitForText("Báo cáo quý 3")
        until { model.recent.firstOrNull()?.uri?.scheme == Stored.SCHEME }

        // Removed: its password goes with it.
        onMain { model.removeStorage(entry!!) }
        assertEquals("", uniffi.packwing_mobile.keptStorageSecrets(entry!!.kind, entry!!.id)["password"])
        webdav("DELETE", "$folder/")
    }

    /**
     * Against the SFTP server of `make sftp-start` (its Docker container, on
     * port 18322 of this computer, 10.0.2.2 to the emulator): skipped without one.
     */
    @Test
    fun anSftpServerIsAskedAboutOnceTrustedAndBrowsed() {
        assumeTrue("no SFTP server: make sftp-start", listening(SFTP_PORT))
        // No server trusted yet, as on a new phone.
        File(app.filesDir, "ssh/known_hosts").delete()
        onMain { model.storages.forEach { model.removeStorage(it) } }

        onMain { model.go(Screen.AddStorage) }
        ui.onNodeWithTag("storage-kind-sftp").performClick()
        ui.onNodeWithTag("storage-host").performTextInput("tester@$HOST:$SFTP_PORT")
        ui.onNodeWithTag("storage-password").performTextInput("packwing")
        ui.onNodeWithTag("storage-add").performClick()

        // A server not known yet: asked about, with its fingerprint, whole.
        ui.waitForText("Trust this server?", timeout = 30_000)
        ui.waitForText("$HOST:$SFTP_PORT")
        ui.waitUntilAtLeastOneExists(hasTestTag("ssh-fingerprint") and hasText("SHA256:", substring = true), TIMEOUT)
        ui.onNodeWithTag("ssh-yes").performClick()

        // Added, and browsed: the tester's home.
        ui.waitUntilAtLeastOneExists(hasTestTag("storage-row"), 30_000)
        assertTrue(File(app.filesDir, "ssh/known_hosts").readText().contains("[$HOST]:$SFTP_PORT"))
        ui.onNodeWithTag("storage-row").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("stored-logs"), 30_000)
        onMain { model.storages.forEach { model.removeStorage(it) } }
    }

    /** Against the WebDAV server of `make webdav-start`: skipped without one. */
    @Test
    fun aStorageIsWrittenToFromThePhone() {
        assumeTrue("no WebDAV server: make webdav-start", webdav("PROPFIND", "") in 200..299)
        val folder = "ui-${java.util.UUID.randomUUID().toString().take(8)}"
        assertTrue(webdav("MKCOL", folder) in 200..299)
        val entry = Settings.StorageEntry(
            java.util.UUID.randomUUID().toString(),
            "webdav",
            mapOf("address" to "http://$HOST:$WEBDAV_PORT", "user" to "packwing"),
        )
        onMain {
            model.storages.forEach { model.removeStorage(it) }
            model.addStorage(entry, mapOf("password" to "packwing"))
            model.go(Screen.Storage(entry.id, listOf(folder)))
        }
        ui.waitForText("This folder is empty.")

        // A folder made.
        ui.onNodeWithTag("storage-menu").performClick()
        ui.onNodeWithTag("storage-new-folder").performClick()
        ui.onNodeWithTag("new-name").performTextInput("Mới")
        ui.onNodeWithTag("name-confirm").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("stored-Mới"), TIMEOUT)

        // A file sent up, as the picker's copy of it is; then renamed.
        val note = File(File(app.cacheDir, "making/up").apply { mkdirs() }, "Ghi chú.txt").apply { writeText("lên") }
        onMain { model.upload(listOf(note), entry, listOf(folder)) }
        ui.waitUntilAtLeastOneExists(hasTestTag("stored-Ghi chú.txt"), TIMEOUT)
        until { model.changing == null }
        ui.onNodeWithTag("stored-menu-Ghi chú.txt").performClick()
        ui.onNodeWithText("Rename").performClick()
        ui.onNodeWithTag("new-name").performTextClearance()
        ui.onNodeWithTag("new-name").performTextInput("Đổi.txt")
        ui.onNodeWithTag("name-confirm").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("stored-Đổi.txt"), TIMEOUT)
        until { model.changing == null }

        // Copied into the folder made, through the picker of storages and folders.
        ui.onNodeWithTag("stored-menu-Đổi.txt").performClick()
        ui.onNodeWithText("Copy to…").performClick()
        ui.onNodeWithTag("picker-storage").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("picker-folder-$folder"), TIMEOUT)
        ui.onNodeWithTag("picker-folder-$folder").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("picker-folder-Mới"), TIMEOUT)
        ui.onNodeWithTag("picker-folder-Mới").performClick()
        ui.onNodeWithTag("picker-here").performClick()
        until { model.changing == null }
        eventually { webdav("PROPFIND", "$folder/Mới/Đổi.txt") in 200..299 }

        // Deleted, for good: a storage has no Trash.
        ui.onNodeWithTag("stored-menu-Đổi.txt").performClick()
        ui.onNodeWithText("Delete…").performClick()
        ui.onNodeWithTag("delete-confirm").performClick()
        until { model.changing == null }
        eventually { webdav("PROPFIND", "$folder/Đổi.txt") == 404 }

        // A new archive made there.
        val inside = File(File(app.cacheDir, "making/zip").apply { mkdirs() }, "a.txt").apply { writeText("a") }
        onMain {
            model.create(
                listOf(inside),
                "made.zip",
                uniffi.packwing_mobile.NewArchiveHow(
                    uniffi.packwing_mobile.ArchiveFormat.ZIP,
                    uniffi.packwing_mobile.CompressionLevel.MEDIUM,
                ),
                entry,
                listOf(folder),
            )
        }
        eventually { webdav("PROPFIND", "$folder/made.zip") in 200..299 }

        onMain {
            model.clearJob()
            model.removeStorage(entry)
        }
        webdav("DELETE", "$folder/")
    }

    @Test
    fun aPrivateKeyIsKeptAndOneWithAPassphraseAsksForIt() {
        onMain { uniffi.packwing_mobile.sshKeys().forEach { uniffi.packwing_mobile.removeSshKey(it.id) } }
        onMain { model.go(Screen.Settings) }
        ui.waitUntilAtLeastOneExists(hasTestTag("ssh-key-add"), TIMEOUT)

        ui.onNodeWithTag("ssh-key-add").performClick()
        ui.onNodeWithTag("private-key-text").performTextInput(TEST_KEY)
        ui.onNodeWithTag("private-key-name").performTextInput("Plain")
        ui.onNodeWithTag("private-key-add").performClick()
        ui.waitForText("ED25519 · $TEST_KEY_FINGERPRINT")

        // One with a passphrase asks for it, and is kept once it is given.
        ui.onNodeWithTag("ssh-key-add").performClick()
        ui.onNodeWithTag("private-key-text").performTextInput(TEST_KEY_LOCKED)
        ui.onNodeWithTag("private-key-name").performTextInput("Locked")
        ui.onNodeWithTag("private-key-add").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("private-key-passphrase"), TIMEOUT)
        ui.onNodeWithTag("private-key-passphrase").performScrollTo().performTextInput("sample-password")
        // Below the field now, with the keyboard up: in sight first, as a thumb would bring it.
        ui.onNodeWithTag("private-key-add").performScrollTo().performClick()
        // Listed by its fingerprint: the name alone is in the field typed too.
        ui.waitForText("ED25519 · $TEST_KEY_LOCKED_FINGERPRINT")

        // Kept in the vault, with no passphrase of its own.
        var kept = emptyList<String>()
        onMain { kept = uniffi.packwing_mobile.sshKeys().map { it.name } }
        assertEquals(setOf("Plain", "Locked"), kept.toSet())
        onMain { uniffi.packwing_mobile.sshKeys().forEach { uniffi.packwing_mobile.removeSshKey(it.id) } }
    }

    // MARK: Helpers

    /** Waits for `done`, asked on the test's own thread: one that goes to the network may not be on the main one. */
    private fun eventually(done: () -> Boolean) {
        val by = System.currentTimeMillis() + TIMEOUT
        while (System.currentTimeMillis() < by) {
            if (done()) return
            Thread.sleep(200)
        }
        throw AssertionError("it did not come to be in ${TIMEOUT / 1000} s")
    }

    /** Whether something on this computer answers on `port`. */
    private fun listening(port: Int): Boolean = runCatching {
        java.net.Socket().use { it.connect(java.net.InetSocketAddress(HOST, port), 2_000) }
        true
    }.getOrDefault(false)

    /**
     * Types `password` in the sheet asking for one, and Unlock once it takes
     * it: a sheet still coming up may not have, and Unlock is off while the
     * field is empty.
     */
    private fun typePassword(password: String) {
        ui.onNode(hasSetTextAction()).performTextInput(password)
        ui.waitUntilAtLeastOneExists(hasText("Unlock") and isEnabled(), TIMEOUT)
        ui.onNodeWithText("Unlock").performClick()
    }

    /**
     * A WebDAV request to the test server, its status; 0 if it cannot be
     * reached. A socket of its own: Android keeps its HTTP clients off plain
     * HTTP, and the server is the test's, not the app's.
     */
    private fun webdav(method: String, path: String, body: ByteArray = ByteArray(0)): Int = runCatching {
        java.net.Socket().use { socket ->
            socket.connect(java.net.InetSocketAddress(HOST, WEBDAV_PORT), 2_000)
            socket.soTimeout = 10_000
            val auth = android.util.Base64.encodeToString("packwing:packwing".toByteArray(), android.util.Base64.NO_WRAP)
            val head = "$method /${Uri.encode(path, "/")} HTTP/1.1\r\nHost: $HOST:$WEBDAV_PORT\r\n" +
                "Authorization: Basic $auth\r\nDepth: 0\r\nContent-Length: ${body.size}\r\nConnection: close\r\n\r\n"
            socket.getOutputStream().apply {
                write(head.toByteArray())
                write(body)
                flush()
            }
            val status = socket.getInputStream().bufferedReader().readLine() ?: ""
            status.split(' ').getOrNull(1)?.toIntOrNull() ?: 0
        }
    }.getOrDefault(0)

    private fun forgetKeys() = onMain { model.trustedKeys.forEach { model.distrust(it.key.line) } }

    /** Opens the sample `name`, from a copy in the app's cache, as Files would hand it over. */
    private fun open(name: String, password: String? = null): File {
        val copy = sample(name)
        onMain {
            model.open(
                Uri.fromFile(copy),
                uniffi.packwing_mobile.Secrets(password, null, null, null),
            )
        }
        return copy
    }

    /** The sample `name`, copied as `called` into one folder: a signature goes beside its archive. */
    private fun sample(name: String, called: String = name): File {
        val into = File(app.cacheDir, "samples").apply { mkdirs() }
        val file = File(into, called).apply { if (exists()) setWritable(true) }
        InstrumentationRegistry.getInstrumentation().context.assets.open(name).use { from ->
            file.outputStream().use { from.copyTo(it) }
        }
        return file
    }

    private val app get() = InstrumentationRegistry.getInstrumentation().targetContext

    /** Waits for the model to come to `done`, which the core reaches off the main thread. */
    private fun until(done: () -> Boolean) {
        val by = System.currentTimeMillis() + TIMEOUT
        while (System.currentTimeMillis() < by) {
            var now = false
            onMain { now = done() }
            if (now) return
            Thread.sleep(100)
        }
        throw AssertionError("the app did not get there in ${TIMEOUT / 1000} s")
    }

    private fun onMain(work: () -> Unit) =
        InstrumentationRegistry.getInstrumentation().runOnMainSync(work)

    private companion object {
        /** Long enough for the emulator's first read of an archive. */
        const val TIMEOUT = 15_000L

        /** This computer, as the emulator reaches it, and the port of `make webdav-start`. */
        const val HOST = "10.0.2.2"
        const val WEBDAV_PORT = 18335
        const val SFTP_PORT = 18322

        /**
         * Two ED25519 keys made for these tests alone, with no server that
         * takes them: one plain, one sealed with the passphrase `sample-password`.
         */
        val TEST_KEY = """
            -----BEGIN OPENSSH PRIVATE KEY-----
            b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
            QyNTUxOQAAACD7Ouyap78Jl6K09mfXOAnT/R3eKn2dG0HhLzqSrDQaYgAAAJixnJxMsZyc
            TAAAAAtzc2gtZWQyNTUxOQAAACD7Ouyap78Jl6K09mfXOAnT/R3eKn2dG0HhLzqSrDQaYg
            AAAED6C22MQMMjnwPVAiUXZMpXrss20WMYqoOIAOB4e/X1S/s67JqnvwmXorT2Z9c4CdP9
            Hd4qfZ0bQeEvOpKsNBpiAAAAFXBhY2t3aW5nLWFuZHJvaWQtdGVzdA==
            -----END OPENSSH PRIVATE KEY-----
        """.trimIndent() + "\n"
        const val TEST_KEY_FINGERPRINT = "SHA256:RjrGEhAM//nEvxizpMphtF9tYTkMjHLr39GgCeqkFWQ"
        const val TEST_KEY_LOCKED_FINGERPRINT = "SHA256:R64XpmaMW/a9nxshuBLEJ2yuN1bmbXMWmuLjjd6CTic"
        val TEST_KEY_LOCKED = """
            -----BEGIN OPENSSH PRIVATE KEY-----
            b3BlbnNzaC1rZXktdjEAAAAACmFlczI1Ni1jdHIAAAAGYmNyeXB0AAAAGAAAABAqSMScA9
            HsVpMm8OIdCD1wAAAAGAAAAAEAAAAzAAAAC3NzaC1lZDI1NTE5AAAAIAVOV+Vn3JdHBGVA
            ZO9AVvimcyfIdNQvQzAKxLOgMTJ+AAAAoCWL/3URMpRr3dZzEr+7RPHzM78ZlCmXCbjoQm
            PYaC2gXfk0SehaxOyao74kzPlKDad4+ZThP6GyssA4TYh3IZzhpq7/a97YwLLqASG92lQ4
            cWpUTomyinnPlky8FylH8e5TxaT3TULjTEkMMfp1NqwqSXnKz+rCGABx6FAT2+TJmqjI5F
            EdVTyoSTPEITBwvH55mU045sYOoi8lOME9E9E=
            -----END OPENSSH PRIVATE KEY-----
        """.trimIndent() + "\n"

        /** report.zip's own, and its Ghi chú.txt's. */
        const val REPORT_SHA256 = "ff5b00b6e9f828d6a94e05f8f4487dd8aa6c0132a3674008ef14dd50a82b6e5d"
        const val REPORT_MD5 = "4af0f880e412e4c0abdca3ecbb0e8367"
        /** The key report.zip.sig was made with. */
        const val SIGNER = "SHA256:4mHGiWUeHRWTUW56vDGyBZfe9CwbQ32QEVMD86pJVto"
        const val SIGNER_KEY = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIP94XxTfx8LLpUBb2U8OiCshCNyBWeea1VO3AJVc6Qvv"

        const val NOTE_SHA256 = "a96b0aa7a284474b68cdea5ab0da0fb0251c340809610dd147c6329a634d09a7"

        /** What `sha256sum` writes of report.zip's files, extracted. */
        val REPORT_LIST = """
            3fe8418c5a4625da1bb37012472c52c4fb561b427dd7eb2aaedfbe12de0a44c9  Báo cáo quý 3/Doanh thu.csv
            a96b0aa7a284474b68cdea5ab0da0fb0251c340809610dd147c6329a634d09a7  Báo cáo quý 3/Ghi chú.txt
            96cf32f73c233837833264efb15d90dd6935339b49d7ab42d80f12e2e1b404e1  Báo cáo quý 3/Hình ảnh/logo.png

        """.trimIndent()
    }
}

/** Waits for a node showing `text`: the core answers off the main thread. */
@OptIn(ExperimentalTestApi::class)
fun AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>.waitForText(
    text: String,
    substring: Boolean = false,
    timeout: Long = 15_000L,
) = waitUntilAtLeastOneExists(hasText(text, substring = substring), timeout)
