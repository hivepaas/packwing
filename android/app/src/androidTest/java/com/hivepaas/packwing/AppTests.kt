package com.hivepaas.packwing

import android.Manifest
import android.net.Uri
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assertTextContains
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
import androidx.compose.ui.test.performScrollToNode
import androidx.compose.ui.test.performSemanticsAction
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
    fun startAtHome() {
        onMain {
            model.home()
            // Google Play's store is no test's: Pro held, for what signs or adds a storage.
            model.fakeStore(pro = true)
        }
        // The last test's keyboard, if it typed, is not this one's.
        hideKeyboard()
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
        // All of its file read, by a debug build, the emulator busy with others: give it time.
        ui.waitUntilAtLeastOneExists(hasTestTag("checksum-sha256") and hasText(REPORT_SHA256), 2 * TIMEOUT)

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
        inSettings(hasText("Tester"))
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
    fun aFileIsSignedWithAKeyMadeHereAndItsSignatureChecks() {
        forgetKeys()
        onMain { uniffi.packwing_mobile.signingKeys().forEach { uniffi.packwing_mobile.removeSigningKey(it.id) } }
        val copy = sample("report.zip")
        File(copy.path + ".sig").delete()
        open("report.zip")
        ui.waitForText("Báo cáo quý 3")

        // A key made where it signs, and the signature beside the file.
        ui.onNodeWithTag("archive-menu").performClick()
        ui.onNodeWithTag("archive-sign").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("sign-new-key"), TIMEOUT)
        ui.onNodeWithTag("sign-new-key").performClick()
        ui.onNodeWithTag("new-name").performTextInput("Tester")
        ui.onNodeWithTag("name-confirm").performClick()
        ui.waitForText("Tester")
        ui.onNodeWithTag("sign-confirm").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("signed"), 2 * TIMEOUT)
        assertTrue(File(copy.path + ".sig").readText().startsWith("-----BEGIN SSH SIGNATURE-----"))

        // Whoever holds the key trusts it: the signature says who signed it.
        // As a person does: the Sign sheet closed, then Signature from the ⋮.
        onMain { model.sheet = null }
        ui.waitForIdle()
        ui.onNodeWithTag("archive-menu").performClick()
        ui.onNodeWithText("Signature").performClick()
        ui.waitForText("Signed by Tester", timeout = 2 * TIMEOUT)

        // Kept in Settings, and removed there.
        onMain {
            model.sheet = null
            model.home()
            model.go(Screen.Settings)
        }
        inSettings(hasTestTag("signing-key-menu-Tester"))
        ui.onNodeWithTag("signing-key-menu-Tester").performClick()
        ui.onNodeWithText("Remove").performClick()
        ui.onNodeWithTag("signing-remove").performClick()
        until { uniffi.packwing_mobile.signingKeys().isEmpty() }

        forgetKeys()
        File(copy.path + ".sig").delete()
    }

    @Test
    fun aPublicKeyPastedInSettingsIsTrusted() {
        forgetKeys()
        onMain { model.go(Screen.Settings) }
        inSettings(hasTestTag("add-key"))
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
        // Among the folders other runs left, maybe below what is in sight.
        ui.waitUntilAtLeastOneExists(hasTestTag("stored-list"), TIMEOUT)
        ui.onNodeWithTag("stored-list").performScrollToNode(hasTestTag("stored-$folder"))
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

    /** Pro by subscription, as the fake store plays Google Play: a year's, kept, and Free again once it ends. */
    @Test
    fun proIsSubscribedToInSettingsAndKeptAfterARestart() {
        onMain {
            model.fakeStore(pro = false)
            model.go(Screen.Settings)
        }
        inSettings(hasTestTag("store-plan"))
        ui.onNodeWithTag("store-plan").assertTextContains("packwing Free")
        // The terms are the last of the plans: all of them are there then.
        inSettings(hasTestTag("store-privacy"))
        ui.onNodeWithTag("store-buy-yearly").assertTextContains("$8.99 a year")
        ui.onNodeWithTag("store-buy-monthly").assertTextContains("$0.99 a month")
        ui.onNodeWithTag("store-terms").assertExists()
        ui.onNodeWithTag("store-buy-yearly").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("store-held"), TIMEOUT)
        ui.onNodeWithTag("store-plan").assertTextContains("packwing Pro")

        // Kept, as packwing starts again.
        onMain { model.restartFakeStore() }
        inSettings(hasTestTag("store-held"))
        ui.onNodeWithTag("store-plan").assertTextContains("packwing Pro")

        // Ended: Free, and the plans again.
        ui.onNodeWithTag("store-manage").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("store-buy-yearly"), TIMEOUT)
        ui.onNodeWithTag("store-plan").assertTextContains("packwing Free")

        // Restored: nothing of this account's to restore, and it says so.
        inSettings(hasTestTag("store-restore"))
        ui.onNodeWithTag("store-restore").performClick()
        inSettings(hasTestTag("store-said"))
        ui.onNodeWithTag("store-said").assertTextContains("found no subscription", substring = true)
    }

    /** A price Google Play left out: the plan's name on its button. */
    @Test
    fun aPlanWithNoPriceShowsItsName() {
        onMain {
            model.fakeStore(pro = false, noMonthly = true)
            model.go(Screen.Settings)
        }
        inSettings(hasTestTag("store-buy-monthly"))
        ui.onNodeWithTag("store-buy-monthly").assertTextContains("Monthly")
        ui.onNodeWithTag("store-buy-yearly").assertTextContains("$8.99 a year")
    }

    /** Signing takes Pro: packwing Free says so, and where to get it. */
    @Test
    fun signingTakesPro() {
        onMain { model.fakeStore(pro = false) }
        open("report.zip")
        ui.waitForText("Báo cáo quý 3")
        ui.onNodeWithTag("archive-menu").performClick()
        ui.onNodeWithTag("archive-sign").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("sign-pro"), TIMEOUT)
        ui.onNodeWithTag("sign-confirm").assertDoesNotExist()
        ui.onNodeWithTag("sign-new-key").assertDoesNotExist()
        // Settings, where the plans are.
        ui.onNodeWithTag("sign-pro-settings").performClick()
        inSettings(hasTestTag("store-plan"))
        ui.onNodeWithTag("store-plan").assertTextContains("packwing Free")
    }

    /** packwing Free keeps one storage: a second is Pro's, and it says so. */
    @Test
    fun packwingFreeKeepsOneStorage() {
        val entry = Settings.StorageEntry(
            java.util.UUID.randomUUID().toString(),
            "webdav",
            mapOf("address" to "http://$HOST:$WEBDAV_PORT", "user" to "packwing"),
        )
        onMain {
            model.storages.forEach { model.removeStorage(it) }
            model.fakeStore(pro = false)
        }
        // None yet: one may be added.
        ui.waitUntilAtLeastOneExists(hasTestTag("storage-new"), TIMEOUT)
        ui.onNodeWithTag("storage-new").performClick()
        until { model.screen == Screen.AddStorage }
        onMain {
            model.home()
            model.addStorage(entry, mapOf("password" to "packwing"))
        }
        // One kept: no other.
        ui.waitUntilAtLeastOneExists(hasTestTag("storage-row"), TIMEOUT)
        ui.onNodeWithTag("storage-new").performClick()
        ui.waitForText("packwing Free keeps one storage across a network. Pro lifts this.")
        ui.onNodeWithTag("storage-limit-ok").performClick()
        assertEquals(Screen.Home, model.screen)

        // With Pro, another.
        onMain { model.fakeStore(pro = true) }
        ui.onNodeWithTag("storage-new").performClick()
        until { model.screen == Screen.AddStorage }
        onMain {
            model.home()
            model.removeStorage(entry)
        }
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
        // Tapped as what they are, not where: among the folders other runs left,
        // a row below the sheet's fold is tapped on the scrim, and the sheet goes.
        ui.waitUntilAtLeastOneExists(hasTestTag("picker-storage"), TIMEOUT)
        ui.onNodeWithTag("picker-storage").performSemanticsAction(SemanticsActions.OnClick)
        ui.waitUntilAtLeastOneExists(hasTestTag("picker-folder-$folder"), TIMEOUT)
        ui.onNodeWithTag("picker-folder-$folder").performSemanticsAction(SemanticsActions.OnClick)
        ui.waitUntilAtLeastOneExists(hasTestTag("picker-folder-Mới"), TIMEOUT)
        ui.onNodeWithTag("picker-folder-Mới").performSemanticsAction(SemanticsActions.OnClick)
        ui.onNodeWithTag("picker-here").performSemanticsAction(SemanticsActions.OnClick)
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

    /**
     * New archives in the formats and ways the desktop makes them, made on the
     * WebDAV server of `make webdav-start`, where they can be looked for:
     * skipped without one.
     */
    @Test
    fun newArchivesAreMadeInTheFormatAndWayChosen() {
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
            model.settings.newFormat = "zip"
        }
        val spot = Screen.Storage(entry.id, listOf(folder))
        fun twoFiles(): List<File> {
            val into = File(app.cacheDir, "making/${java.util.UUID.randomUUID()}").apply { mkdirs() }
            return listOf(File(into, "a.txt").apply { writeText("a") }, File(into, "b.txt").apply { writeText("b") })
        }

        // tar.zst, encrypted whole with age to a passphrase: Archive.tar.zst.age.
        onMain { model.go(Screen.Create(spot, twoFiles())) }
        ui.waitUntilAtLeastOneExists(hasTestTag("new-format"), TIMEOUT)
        ui.onNodeWithTag("new-format").performClick()
        ui.onNodeWithTag("new-format-tar.zst").performClick()
        ui.waitForText(".tar.zst")
        ui.onNodeWithTag("new-locked").performClick()
        ui.waitForText(".tar.zst.age")
        ui.onNodeWithTag("new-password").performScrollTo().performTextInput("sample-password")
        makeIt()
        madeWithoutFailing()
        eventually { webdav("PROPFIND", "$folder/Archive.tar.zst.age") in 200..299 }

        // Opened where it is, with its passphrase: what went in is there.
        val size = Regex("<D:getcontentlength>(\\d+)").find(webdavBody("$folder/Archive.tar.zst.age"))?.groupValues?.get(1)?.toULong() ?: 0UL
        onMain { model.home() }
        onMain {
            model.open(
                Stored(entry.id, listOf(folder, "Archive.tar.zst.age"), size).uri,
                uniffi.packwing_mobile.Secrets(null, "sample-password", null, null),
            )
        }
        ui.waitForText("a.txt")
        ui.waitForText("b.txt")

        // ZIP, in parts of 1 MB: Archive.zip.001, .002.
        val big = File(File(app.cacheDir, "making/${java.util.UUID.randomUUID()}").apply { mkdirs() }, "big.bin")
            .apply { writeBytes(java.util.Random(7).let { r -> ByteArray(2_500_000).also { r.nextBytes(it) } }) }
        onMain {
            model.home()
            model.go(Screen.Create(spot, listOf(big)))
        }
        ui.waitUntilAtLeastOneExists(hasTestTag("new-split"), TIMEOUT)
        ui.onNodeWithTag("new-format").performClick()
        ui.onNodeWithTag("new-format-zip").performClick()
        ui.onNodeWithTag("new-split").performClick()
        ui.onNodeWithTag("new-split-Custom").performClick()
        ui.onNodeWithTag("new-part-size").performScrollTo().performTextInput("1")
        makeIt()
        madeWithoutFailing()
        eventually { webdav("PROPFIND", "$folder/big.zip.003") in 200..299 }
        assertTrue(webdav("PROPFIND", "$folder/big.zip.001") in 200..299)

        // 7z, encrypted whole with age to a public key (an SSH one): Archive.7z.age.
        onMain {
            model.home()
            model.go(Screen.Create(spot, twoFiles()))
        }
        ui.waitUntilAtLeastOneExists(hasTestTag("new-format"), TIMEOUT)
        ui.onNodeWithTag("new-format").performClick()
        ui.onNodeWithTag("new-format-7z").performClick()
        ui.onNodeWithTag("new-locked").performClick()
        ui.onNodeWithTag("new-encryption-1").performScrollTo().performClick()
        ui.onNodeWithTag("new-age-to-1").performScrollTo().performClick()
        ui.onNodeWithTag("new-age-key").performScrollTo().performTextInput(SIGNER_KEY)
        makeIt()
        madeWithoutFailing()
        eventually { webdav("PROPFIND", "$folder/Archive.7z.age") in 200..299 }

        onMain {
            model.clearJob()
            model.home()
            model.removeStorage(entry)
            model.settings.newFormat = "zip"
        }
        webdav("DELETE", "$folder/")
    }

    @Test
    fun aPrivateKeyIsKeptAndOneWithAPassphraseAsksForIt() {
        onMain { uniffi.packwing_mobile.sshKeys().forEach { uniffi.packwing_mobile.removeSshKey(it.id) } }
        onMain { model.go(Screen.Settings) }
        // Below the signing keys: in sight first, clear of the bar at the foot.
        inSettings(hasTestTag("ssh-key-add"))
        ui.onNodeWithTag("ssh-key-add").performClick()
        ui.onNodeWithTag("private-key-text").performTextInput(TEST_KEY)
        ui.onNodeWithTag("private-key-name").performTextInput("Plain")
        ui.onNodeWithTag("private-key-add").performClick()
        ui.waitForText("ED25519 · $TEST_KEY_FINGERPRINT")

        // One with a passphrase asks for it, and is kept once it is given.
        ui.onNodeWithTag("settings-list").performScrollToNode(hasTestTag("ssh-key-add"))
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

    @Test
    fun settingsSayHowArchivesAreMadeAndWhereTheirFilesGo() {
        onMain {
            model.settings.extractTo = ExtractPlace.Smart.key
            model.settings.overwrite = false
            model.settings.storedExtensions = ""
            model.showChecksums("sha256 sha1")
            model.go(Screen.Settings)
        }
        ui.waitUntilAtLeastOneExists(hasTestTag("settings-list"), TIMEOUT)

        // Dark, whatever the system is; then as it is again.
        choose("settings-appearance", Appearance.Dark.key)
        until { model.appearance == Appearance.Dark && model.settings.appearance == "dark" }
        choose("settings-appearance", Appearance.System.key)
        until { model.appearance == Appearance.System }

        // New archives: the format and what goes in as it is.
        choose("settings-format", "7z")
        assertEquals("7z", model.settings.newFormat)
        choose("settings-format", "zip")
        ui.onNodeWithTag("settings-list").performScrollToNode(hasTestTag("settings-stored-extensions"))
        ui.onNodeWithTag("settings-stored-extensions").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("settings-extensions"), TIMEOUT)
        ui.onNodeWithTag("settings-extensions").performTextClearance()
        ui.onNodeWithTag("settings-extensions").performTextInput("ABC .def")
        until { model.settings.storedExtensions == "abc def" }
        ui.onNodeWithTag("settings-extensions-reset").performClick()
        until { model.settings.storedExtensions == "" }
        hideKeyboard()
        onMain { model.home() }
        onMain { model.go(Screen.Settings) }
        ui.waitUntilAtLeastOneExists(hasTestTag("settings-list"), TIMEOUT)

        // A checksum shown too, as Checksums' menu would.
        ui.onNodeWithTag("settings-list").performScrollToNode(hasTestTag("settings-checksum-md5"))
        ui.onNodeWithTag("settings-checksum-md5").performClick()
        until { "md5" in model.checksumKeys.split(' ') }

        // Extracting: in a new folder, named after the archive, and another beside it.
        choose("settings-extract-to", ExtractPlace.NewFolder.key)
        choose("settings-extract-limit", "none")
        assertEquals("none", model.settings.extractLimit)
        val folder = File(app.cacheDir, "extracted-${java.util.UUID.randomUUID()}").apply { mkdirs() }
        open("report.zip")
        ui.waitForText("Báo cáo quý 3")
        assertEquals(null, extractInto(folder))
        assertEquals(null, extractInto(folder))
        assertEquals(setOf("report", "report 2"), folder.list()!!.toSet())
        assertTrue(folder.walk().joinToString { it.relativeTo(folder).path }, File(folder, "report 2/Báo cáo quý 3/Ghi chú.txt").isFile)

        // Straight into the folder; and again, which stops, a file being there.
        onMain { model.settings.extractTo = ExtractPlace.Here.key }
        val straight = File(folder, "straight").apply { mkdirs() }
        assertEquals(null, extractInto(straight))
        assertTrue(File(straight, "Báo cáo quý 3/Doanh thu.csv").isFile)
        assertTrue(extractInto(straight) != null)

        // Smart: one folder in it, so straight in, as Here.
        onMain { model.settings.extractTo = ExtractPlace.Smart.key }
        val smart = File(folder, "smart").apply { mkdirs() }
        assertEquals(null, extractInto(smart))
        assertEquals(setOf("Báo cáo quý 3"), smart.list()!!.toSet())

        folder.deleteRecursively()
        onMain {
            model.settings.extractLimit = ExtractLimit.standard.key
            model.showChecksums("sha256 sha1")
        }
    }

    @Test
    fun aboutSaysWhichBuildThisIsAndCopiesIt() {
        onMain { model.go(Screen.Settings) }
        inSettings(hasTestTag("settings-about"))
        ui.onNodeWithTag("settings-about").performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("about-version"), TIMEOUT)
        ui.onNodeWithTag("about-version").assertTextContains(BuildConfig.VERSION_NAME, substring = true)

        // Copied as a problem report wants it, as the desktop copies it.
        ui.onNodeWithTag("about-copy-version").performClick()
        ui.waitForText("Copied")
        var copied: String? = null
        onMain {
            val clipboard = app.getSystemService(android.content.ClipboardManager::class.java)
            copied = clipboard.primaryClip?.getItemAt(0)?.text?.toString()
        }
        assertEquals(com.hivepaas.packwing.ui.versionInfo, copied)
        assertTrue(copied!!.startsWith("packwing ${BuildConfig.VERSION_NAME} (Android "))

        // The other apps, at its foot.
        ui.onNodeWithTag("about-list").performScrollToNode(hasTestTag("about-hivepaas"))
        ui.onNodeWithTag("about-hivepaas").assertExists()

        onMain { model.back() }
        ui.waitUntilAtLeastOneExists(hasTestTag("settings-list"), TIMEOUT)
    }

    // MARK: Helpers

    /** Brings what `matches` into sight in Settings: a row far down is not there until it is. */
    private fun inSettings(matches: SemanticsMatcher) {
        ui.waitUntilAtLeastOneExists(hasTestTag("settings-list"), TIMEOUT)
        ui.onNodeWithTag("settings-list").performScrollToNode(matches)
    }

    /** Picks `key` in the Settings row `tag`, scrolled to first. */
    private fun choose(tag: String, key: String) {
        inSettings(hasTestTag(tag))
        ui.onNodeWithTag(tag).performClick()
        ui.waitUntilAtLeastOneExists(hasTestTag("$tag-$key"), TIMEOUT)
        ui.onNodeWithTag("$tag-$key").performClick()
        ui.waitUntilDoesNotExist(hasTestTag("$tag-$key"), TIMEOUT)
    }

    /** Extracts all of the archive open into `folder`, by its path; what it said if it failed. */
    private fun extractInto(folder: File): String? {
        onMain {
            model.clearJob()
            model.extract(emptyList(), Destination.Folder(Uri.fromFile(folder)))
        }
        until(60_000) { model.running?.let { it.done != null || it.failed != null } == true }
        var said: String? = null
        onMain {
            said = model.running?.failed
            model.clearJob()
        }
        return said
    }

    /**
     * Waits for the archive being made to be made, and says why if it was
     * not: compressed by a debug build and sent, with others running, it may
     * take more than a few seconds. Then forgets the job, as its screen does.
     */
    private fun madeWithoutFailing() {
        until(90_000) { model.running?.let { it.done != null || it.failed != null } == true }
        var said: String? = null
        var done: String? = null
        onMain {
            said = model.running?.failed
            done = model.running?.done
            model.clearJob()
        }
        assertEquals(null, said)
        // What it says of it, in passing, gone: it lies over what is tapped next.
        done?.let { ui.waitUntilDoesNotExist(hasText(it), 20_000) }
    }

    /** Create, once it is on: it is off while what it needs is still being typed. */
    private fun makeIt() {
        ui.waitUntilAtLeastOneExists(hasTestTag("create-make") and isEnabled(), TIMEOUT)
        // Done typing, as a thumb says it: the keyboard goes. Create sits
        // above it and slides down as it goes, by the system's animation,
        // which waitForIdle does not wait for: a tap where it was may miss.
        hideKeyboard()
        ui.waitForIdle()
        ui.onNodeWithTag("create-make").performSemanticsAction(SemanticsActions.OnClick)
        until { model.running != null }
    }

    private fun hideKeyboard() = onMain {
        val window = ui.activity.window
        androidx.core.view.WindowCompat.getInsetsController(window, window.decorView)
            .hide(androidx.core.view.WindowInsetsCompat.Type.ime())
    }

    /** What the test server says of `path` (Depth 0), as text: its size, for one. */
    private fun webdavBody(path: String): String = runCatching {
        java.net.Socket().use { socket ->
            socket.connect(java.net.InetSocketAddress(HOST, WEBDAV_PORT), 2_000)
            socket.soTimeout = 10_000
            val auth = android.util.Base64.encodeToString("packwing:packwing".toByteArray(), android.util.Base64.NO_WRAP)
            val head = "PROPFIND /${Uri.encode(path, "/")} HTTP/1.1\r\nHost: $HOST:$WEBDAV_PORT\r\n" +
                "Authorization: Basic $auth\r\nDepth: 0\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            socket.getOutputStream().apply {
                write(head.toByteArray())
                flush()
            }
            socket.getInputStream().bufferedReader().readText()
        }
    }.getOrDefault("")

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
    private fun until(timeout: Long = TIMEOUT, done: () -> Boolean) {
        val by = System.currentTimeMillis() + timeout
        while (System.currentTimeMillis() < by) {
            var now = false
            onMain { now = done() }
            if (now) return
            Thread.sleep(100)
        }
        throw AssertionError("the app did not get there in ${timeout / 1000} s")
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
