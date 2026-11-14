import XCTest

/// The app as a person uses it, on the Simulator: an archive opened from the
/// launch argument `-open` (debug builds), as Files would open it.
final class AppTests: XCTestCase {
    override func setUp() {
        continueAfterFailure = false
    }

    /// The app, opening the sample `name` (in Samples/) as it starts.
    private func launch(opening name: String) -> XCUIApplication {
        let sample = Bundle(for: Self.self).url(forResource: name, withExtension: nil)
        XCTAssertNotNil(sample, "no sample \(name)")
        let app = XCUIApplication()
        app.launchArguments = ["-open", sample!.path, "-AppleLanguages", "(en)", "-language", ""]
        app.launch()
        return app
    }

    /// Keeps what the screen shows in the test's results, to look at.
    private func keep(_ app: XCUIApplication, _ name: String) {
        let shot = XCTAttachment(screenshot: app.screenshot())
        shot.name = name
        shot.lifetime = .keepAlways
        add(shot)
    }

    private func text(_ app: XCUIApplication, starting prefix: String) -> XCUIElement {
        app.staticTexts.matching(NSPredicate(format: "label BEGINSWITH %@", prefix)).firstMatch
    }

    func testAnArchiveIsExtractedIntoAFolderOfItsName() {
        let app = launch(opening: "report.zip")
        let extractAll = app.buttons["Extract all"]
        XCTAssert(extractAll.waitForExistence(timeout: 15))
        keep(app, "archive")
        extractAll.tap()

        let start = app.buttons["Extract 3 files"]
        XCTAssert(start.waitForExistence(timeout: 5))
        keep(app, "extract-sheet")
        start.tap()

        XCTAssert(app.staticTexts["Done"].waitForExistence(timeout: 30))
        XCTAssert(text(app, starting: "3 files in report").exists)
        keep(app, "extracted")
    }

    func testALockedArchiveAsksForItsPasswordAgainWhenWrong() {
        let app = launch(opening: "locked.7z")
        var field = app.secureTextFields["Password"]
        XCTAssert(field.waitForExistence(timeout: 15))
        field.tap()
        field.typeText("not it\n")

        XCTAssert(text(app, starting: "The password is wrong").waitForExistence(timeout: 15))
        field = app.secureTextFields["Password"]
        field.tap()
        field.typeText("sample-password\n")

        // Its names were hidden: there now.
        XCTAssert(app.staticTexts["Báo cáo quý 3"].waitForExistence(timeout: 15))
    }

    func testAFileTappedIsPreviewed() {
        let app = launch(opening: "report.zip")
        let folder = app.staticTexts["Báo cáo quý 3"]
        XCTAssert(folder.waitForExistence(timeout: 15))
        folder.tap()

        let file = app.staticTexts["Ghi chú.txt"]
        XCTAssert(file.waitForExistence(timeout: 5))
        file.tap()

        let shown = app.descendants(matching: .any)
            .matching(NSPredicate(format: "label CONTAINS %@", "Doanh thu quý 3 tăng"))
            .firstMatch
        XCTAssert(shown.waitForExistence(timeout: 15))
    }

    func testThePicturesOfAnArchiveGoToPhotos() {
        let app = launch(opening: "report.zip")
        let extractAll = app.buttons["Extract all"]
        XCTAssert(extractAll.waitForExistence(timeout: 15))
        extractAll.tap()

        let photos = app.buttons["Photos"]
        XCTAssert(photos.waitForExistence(timeout: 5))
        photos.tap()
        app.buttons["Save to Photos"].tap()

        // iOS asks once whether packwing may add to Photos.
        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")
        let allow = springboard.buttons["Allow"]
        if allow.waitForExistence(timeout: 10) { allow.tap() }

        XCTAssert(app.staticTexts["Done"].waitForExistence(timeout: 30))
        XCTAssert(text(app, starting: "1 picture or video added to Photos").exists)
    }
}
