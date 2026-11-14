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
        app.launchArguments = ["-open", sample!.path, "-AppleLanguages", "(en)", "-language", "",
                               "-trustedKeys", ""]
        app.launch()
        return app
    }

    /// The app, showing New archive of the samples `names` as it starts.
    private func launch(making names: [String]) -> XCUIApplication {
        let samples = names.map { name in
            let sample = Bundle(for: Self.self).url(forResource: name, withExtension: nil)
            XCTAssertNotNil(sample, "no sample \(name)")
            return sample!.path
        }
        let app = XCUIApplication()
        app.launchArguments = ["-new", samples.joined(separator: "|"), "-AppleLanguages", "(en)", "-language", "",
                               "-newFormat", "zip", "-newCompression", "medium"]
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

    func testFilesChosenAreCompressedIntoAZipOfTheNameGiven() {
        let app = launch(making: ["report.zip", "locked.7z"])
        XCTAssert(app.staticTexts["2 files"].waitForExistence(timeout: 15))
        let name = app.textFields["new-name"]
        name.tap()
        name.typeText(String(repeating: XCUIKeyboardKey.delete.rawValue, count: 40) + "Bundle")
        keep(app, "create-sheet")
        app.buttons["Create"].firstMatch.tap()

        XCTAssert(app.staticTexts["Done"].waitForExistence(timeout: 30))
        XCTAssert(text(app, starting: "Bundle").exists)
        XCTAssert(app.buttons["Share"].exists)
        keep(app, "created")
    }

    func testANewArchiveIsLockedWithItsNamesHidden() {
        let app = launch(making: ["report.zip"])
        let name = app.textFields["new-name"]
        XCTAssert(name.waitForExistence(timeout: 15))
        // Named after the one file in it.
        XCTAssertEqual(name.value as? String, "report")

        app.buttons["7z"].tap()
        app.switches["Protect with a password"].switches.firstMatch.tap()
        // Before the keyboard covers it.
        let hide = app.switches.matching(NSPredicate(format: "label BEGINSWITH %@", "Hide the file names too")).firstMatch
        XCTAssert(hide.waitForExistence(timeout: 5))
        hide.switches.firstMatch.tap()
        XCTAssertEqual(hide.value as? String, "1")
        let password = app.secureTextFields["new-password"]
        password.tap()
        password.typeText("sample-password")
        // Shown, it is the one typed.
        app.buttons["Show the password"].tap()
        XCTAssertEqual(app.textFields["new-password"].value as? String, "sample-password")
        keep(app, "create-locked")
        app.buttons["Create"].firstMatch.tap()

        XCTAssert(app.staticTexts["Done"].waitForExistence(timeout: 30))
        XCTAssert(text(app, starting: "report").exists)
    }

    func testPhotosChosenGoIntoANewArchive() {
        let app = XCUIApplication()
        app.launchArguments = ["-AppleLanguages", "(en)", "-language", "", "-newFormat", "zip"]
        app.launch()
        let add = app.buttons["New archive"]
        XCTAssert(add.waitForExistence(timeout: 15))
        add.tap()
        app.buttons["Photos"].tap()

        // The picker is iOS's own: the Simulator's sample photos.
        let photo = app.images.matching(identifier: "PXGGridLayout-Info").firstMatch
        XCTAssert(photo.waitForExistence(timeout: 40))
        photo.tap()
        app.buttons["Done"].tap()

        XCTAssert(app.staticTexts["1 photo"].waitForExistence(timeout: 30))
        keep(app, "create-photo")
        app.buttons["Create"].firstMatch.tap()
        XCTAssert(app.staticTexts["Done"].waitForExistence(timeout: 30))
        XCTAssert(app.buttons["Share"].exists)
    }

    func testAnArchiveOfPicturesShowsAsAGridOfThem() {
        let app = launch(opening: "photos.zip")
        // Its pictures as themselves, a HEIC too, once read from it.
        for name in ["IMG_0001.jpg", "IMG_0002.jpg", "IMG_0003.heic", "IMG_0004.png"] {
            let tile = app.buttons.matching(NSPredicate(format: "label == %@ AND identifier == 'picture'", name)).firstMatch
            XCTAssert(tile.waitForExistence(timeout: 20), "no picture of \(name)")
        }
        // A file that is not a picture, as its kind.
        XCTAssertEqual(app.buttons["Ghi chú.txt"].identifier, "tile")
        keep(app, "gallery")

        // Chosen in the grid, then extracted.
        app.buttons["Select"].tap()
        app.buttons["IMG_0002.jpg"].tap()
        app.buttons["IMG_0003.heic"].tap()
        XCTAssert(app.buttons["Extract 2"].waitForExistence(timeout: 5))
        keep(app, "gallery-selecting")
        app.buttons["Done"].tap()

        // All of it to Photos, from the grid.
        app.buttons["Save all to Photos"].tap()
        XCTAssert(app.buttons["Save to Photos"].waitForExistence(timeout: 5))
        app.buttons["Close"].firstMatch.tap()

        // As a list, when asked.
        app.buttons["Show a list"].tap()
        XCTAssert(app.staticTexts["IMG_0001.jpg"].waitForExistence(timeout: 5))
        XCTAssert(app.buttons["Extract all"].exists)
    }

    func testAnArchiveSharedFromMailIsSeenWithoutBeingSavedInFiles() {
        // Given as Mail gives one: a copy in packwing's Documents/Inbox.
        let sample = Bundle(for: Self.self).url(forResource: "report.zip", withExtension: nil)!
        let app = XCUIApplication()
        app.launchArguments = ["-receive", sample.path, "-AppleLanguages", "(en)", "-language", ""]
        app.launch()
        XCTAssert(app.buttons["Extract all"].waitForExistence(timeout: 15))
        app.terminate()

        // Opened again from Recent, where packwing keeps it now.
        app.launchArguments = ["-AppleLanguages", "(en)", "-language", ""]
        app.launch()
        let recent = app.buttons.matching(NSPredicate(format: "label BEGINSWITH 'report.zip'")).firstMatch
        XCTAssert(recent.waitForExistence(timeout: 10))
        recent.tap()
        XCTAssert(app.buttons["Extract all"].waitForExistence(timeout: 15))

        // And not in packwing's folder in Files (On My iPhone, or iPad): no Inbox there.
        let files = XCUIApplication(bundleIdentifier: "com.apple.DocumentsApp")
        files.launch()
        // A tab bar's on an iPhone, buttons at the top on an iPad.
        let browse = files.buttons["Browse"].firstMatch
        if browse.waitForExistence(timeout: 5) { browse.tap() }
        let onPhone = files.descendants(matching: .any).matching(NSPredicate(format: "label BEGINSWITH 'On My i'")).firstMatch
        if onPhone.waitForExistence(timeout: 3) { onPhone.tap() }
        let own = files.cells.matching(NSPredicate(format: "identifier BEGINSWITH 'packwing'")).firstMatch
        XCTAssert(own.waitForExistence(timeout: 10))
        own.tap()
        // In it: what it holds, and no longer packwing's own folder among them.
        let inside = XCTNSPredicateExpectation(predicate: NSPredicate(format: "exists == false"), object: own)
        XCTAssertEqual(XCTWaiter.wait(for: [inside], timeout: 10), .completed)
        XCTAssert(files.cells.firstMatch.waitForExistence(timeout: 10))
        keep(files, "files-packwing")
        XCTAssertFalse(files.cells.matching(NSPredicate(format: "identifier BEGINSWITH 'Inbox'")).firstMatch.exists)
    }

    func testAnArchiveSChecksumIsComparedWithTheOneGiven() {
        // Its SHA-256 and MD5, as `shasum -a 256` and `md5` say them.
        let sha256 = "ff5b00b6e9f828d6a94e05f8f4487dd8aa6c0132a3674008ef14dd50a82b6e5d"
        let md5 = "4af0f880e412e4c0abdca3ecbb0e8367"
        let app = launch(opening: "report.zip")
        XCTAssert(app.buttons["Extract all"].waitForExistence(timeout: 15))
        app.buttons["More"].tap()
        app.buttons["Checksums"].tap()

        let made = app.staticTexts["checksum-sha256"]
        XCTAssert(made.waitForExistence(timeout: 10))
        let read = XCTNSPredicateExpectation(predicate: NSPredicate(format: "label == %@", sha256), object: made)
        XCTAssertEqual(XCTWaiter.wait(for: [read], timeout: 15), .completed)

        // As sha256sum writes it, in capitals.
        let field = app.textFields["checksum-pasted"]
        field.tap()
        field.typeText(sha256.uppercased() + "  report.zip")
        XCTAssert(app.staticTexts["It matches the file’s SHA-256 checksum."].waitForExistence(timeout: 5))
        keep(app, "checksum-matches")

        // An MD5: not made until asked.
        field.press(forDuration: 1.2)
        if app.menuItems["Select All"].waitForExistence(timeout: 3) { app.menuItems["Select All"].tap() }
        field.typeText(md5)
        let makeIt = app.buttons["Make it"]
        XCTAssert(makeIt.waitForExistence(timeout: 5))
        makeIt.tap()
        XCTAssert(app.staticTexts["It matches the file’s MD5 checksum."].waitForExistence(timeout: 15))
        XCTAssert(app.staticTexts["checksum-md5"].exists)
        keep(app, "checksum-md5")
    }

    func testAnArchiveSSignatureSaysWhoSignedItOnceItsKeyIsTrusted() {
        // Samples/report.zip.sig: report.zip signed with `ssh-keygen -Y sign -n file`.
        let fingerprint = "SHA256:4mHGiWUeHRWTUW56vDGyBZfe9CwbQ32QEVMD86pJVto"
        let app = launch(opening: "report.zip")
        XCTAssert(app.buttons["Extract all"].waitForExistence(timeout: 15))
        app.buttons["More"].tap()
        app.buttons["Signature"].tap()

        // Found beside it; by a key not trusted yet, which proves nothing.
        let stranger = app.staticTexts["Signed by a key you do not trust: \(fingerprint)"]
        XCTAssert(stranger.waitForExistence(timeout: 15))
        keep(app, "signature-stranger")
        app.buttons["Trust this key"].tap()
        let name = app.alerts.textFields.firstMatch
        XCTAssert(name.waitForExistence(timeout: 5))
        name.typeText("Tester")
        app.alerts.buttons["Trust this key"].tap()

        // Checked again, against all of it.
        XCTAssert(app.staticTexts["Signed by Tester"].waitForExistence(timeout: 15))
        keep(app, "signature-good")
        app.buttons["Close"].firstMatch.tap()

        // Kept in Settings.
        app.navigationBars.buttons.element(boundBy: 0).tap()
        // A tab bar's on an iPhone, buttons at the top on an iPad: by its label.
        let settings = app.buttons.matching(NSPredicate(format: "label == 'Settings'")).firstMatch
        XCTAssert(settings.waitForExistence(timeout: 5))
        settings.tap()
        let kept = app.staticTexts["Tester"]
        XCTAssert(kept.waitForExistence(timeout: 5))
        XCTAssert(app.staticTexts[fingerprint].exists)
        keep(app, "keys-trusted")
    }
}
