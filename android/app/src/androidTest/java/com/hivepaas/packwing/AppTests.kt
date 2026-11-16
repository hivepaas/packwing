package com.hivepaas.packwing

import android.Manifest
import android.net.Uri
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.AndroidComposeTestRule
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTouchInput
import androidx.test.ext.junit.rules.ActivityScenarioRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.GrantPermissionRule
import org.junit.After
import org.junit.Assert.assertEquals
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
        ui.onNode(hasSetTextAction()).performTextInput("not it")
        ui.onNodeWithText("Unlock").performClick()
        ui.waitForText("The password is wrong", substring = true)

        ui.onNode(hasSetTextAction()).performTextClearance()
        ui.onNode(hasSetTextAction()).performTextInput("sample-password")
        ui.onNodeWithText("Unlock").performClick()
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

    // MARK: Helpers

    /** Opens the sample `name`, from a copy in the app's cache, as Files would hand it over. */
    private fun open(name: String, password: String? = null) {
        val copy = sample(name)
        onMain {
            model.open(
                Uri.fromFile(copy),
                uniffi.packwing_mobile.Secrets(password, null, null, null),
            )
        }
    }

    private fun sample(name: String): File {
        val into = File(app.cacheDir, "samples").apply { mkdirs() }
        val file = File(into, name)
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

        /** report.zip's own, and its Ghi chú.txt's. */
        const val REPORT_SHA256 = "ff5b00b6e9f828d6a94e05f8f4487dd8aa6c0132a3674008ef14dd50a82b6e5d"
        const val REPORT_MD5 = "4af0f880e412e4c0abdca3ecbb0e8367"
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
