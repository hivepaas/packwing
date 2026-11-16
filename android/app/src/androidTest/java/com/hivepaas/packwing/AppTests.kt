package com.hivepaas.packwing

import android.Manifest
import android.net.Uri
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.junit4.AndroidComposeTestRule
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.rules.ActivityScenarioRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.GrantPermissionRule
import org.junit.After
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
    fun startAtHome() = onMain { model.home() }

    @After
    fun leave() = onMain {
        model.sheet = null
        model.home()
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
    }
}

/** Waits for a node showing `text`: the core answers off the main thread. */
@OptIn(ExperimentalTestApi::class)
fun AndroidComposeTestRule<ActivityScenarioRule<MainActivity>, MainActivity>.waitForText(
    text: String,
    substring: Boolean = false,
    timeout: Long = 15_000L,
) = waitUntilAtLeastOneExists(hasText(text, substring = substring), timeout)
