import Network
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

    /// Taps `element` once a screen or sheet going has gone: a tap while one
    /// still slides goes nowhere. (XCTest's `isHittable` says no to SwiftUI's
    /// rows and buttons after a sheet, though they take a tap.)
    private func tapWhenReady(_ element: XCUIElement, timeout: TimeInterval = 20) {
        XCTAssert(element.waitForExistence(timeout: timeout), "no \(element)")
        sleep(1)
        element.tap()
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

        // The format, from its menu.
        app.buttons["new-format"].tap()
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
        // Asked what to read: its own file.
        let itself = app.buttons.matching(NSPredicate(format: "label BEGINSWITH 'The archive itself'")).firstMatch
        XCTAssert(itself.waitForExistence(timeout: 5))
        itself.tap()
        app.buttons["checksum-read"].tap()

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
        // Under License, which may fill the screen: a list's rows below it are
        // not made until they are scrolled to.
        let kept = app.staticTexts["Tester"]
        for _ in 0..<4 where !kept.waitForExistence(timeout: 2) {
            app.swipeUp()
        }
        XCTAssert(kept.waitForExistence(timeout: 5))
        XCTAssert(app.staticTexts[fingerprint].exists)
        keep(app, "keys-trusted")
    }

    func testFilesInAnArchiveAreRenamedDeletedAndAdded() {
        // Given as Mail gives one: packwing keeps it in a folder of its own,
        // where the archive is changed where it is.
        let sample = Bundle(for: Self.self).url(forResource: "report.zip", withExtension: nil)!
        let app = XCUIApplication()
        app.launchArguments = ["-receive", sample.path, "-AppleLanguages", "(en)", "-language", "",
                               "-trustedKeys", ""]
        app.launch()
        let folder = app.staticTexts["Báo cáo quý 3"]
        XCTAssert(folder.waitForExistence(timeout: 15))
        folder.tap()

        // Renamed.
        let notes = app.staticTexts["Ghi chú.txt"]
        XCTAssert(notes.waitForExistence(timeout: 10))
        notes.press(forDuration: 1.2)
        app.buttons["Rename"].tap()
        let name = app.textFields["new-name"]
        XCTAssert(name.waitForExistence(timeout: 5))
        name.typeText(String(repeating: XCUIKeyboardKey.delete.rawValue, count: 20) + "Notes.txt")
        app.buttons["name-confirm"].tap()
        XCTAssert(app.staticTexts["Notes.txt"].waitForExistence(timeout: 15))
        XCTAssertFalse(notes.exists)

        // Deleted, once it is said that it is for good.
        let sales = app.staticTexts["Doanh thu.csv"]
        sales.press(forDuration: 1.2)
        app.buttons["Delete…"].tap()
        XCTAssert(app.staticTexts["Delete Doanh thu.csv?"].waitForExistence(timeout: 5))
        keep(app, "delete-asked")
        app.buttons.matching(NSPredicate(format: "label == 'Delete'")).firstMatch.tap()
        let gone = XCTNSPredicateExpectation(predicate: NSPredicate(format: "exists == false"), object: sales)
        XCTAssertEqual(XCTWaiter.wait(for: [gone], timeout: 15), .completed)

        // Added, from Photos.
        app.buttons["Add files"].tap()
        app.buttons["Photos"].tap()
        let photo = app.images.matching(identifier: "PXGGridLayout-Info").firstMatch
        XCTAssert(photo.waitForExistence(timeout: 40))
        photo.tap()
        app.buttons["Done"].tap()
        let added = app.staticTexts.matching(NSPredicate(format: "label MATCHES[c] '.*\\\\.(jpe?g|heic|png)'")).firstMatch
        XCTAssert(added.waitForExistence(timeout: 30))
        XCTAssert(app.staticTexts["Notes.txt"].exists)
        keep(app, "changed")
    }

    /// Against a WebDAV server on this computer (`make webdav-start`): skipped
    /// without one.
    func testAnArchiveOnAStorageIsOpenedWhereItIs() throws {
        let server = "http://127.0.0.1:18335"
        let folder = "ui-\(UUID().uuidString.prefix(8))"
        let auth = "Basic " + Data("packwing:packwing".utf8).base64EncodedString()
        func send(_ method: String, _ path: String, _ body: Data? = nil) -> Int {
            var request = URLRequest(url: URL(string: "\(server)/\(path)")!)
            request.httpMethod = method
            request.setValue(auth, forHTTPHeaderField: "Authorization")
            request.httpBody = body
            var status = 0
            let done = expectation(description: method)
            URLSession.shared.dataTask(with: request) { _, response, _ in
                status = (response as? HTTPURLResponse)?.statusCode ?? 0
                done.fulfill()
            }.resume()
            wait(for: [done], timeout: 10)
            return status
        }
        guard send("MKCOL", folder) / 100 == 2 else { throw XCTSkip("no WebDAV server at \(server)") }
        let sample = Bundle(for: Self.self).url(forResource: "report.zip", withExtension: nil)!
        XCTAssertEqual(send("PUT", "\(folder)/report.zip", try Data(contentsOf: sample)) / 100, 2)

        let app = XCUIApplication()
        app.launchArguments = ["-AppleLanguages", "(en)", "-language", "", "-storages", ""]
        app.launch()
        let add = app.buttons["storage-new"]
        XCTAssert(add.waitForExistence(timeout: 15))
        add.tap()
        app.buttons["WebDAV folder"].tap()
        // In the order they are shown: the password last, as its keyboard is iOS's own.
        let fields = [("storage-address", "\(server)/\(folder)"), ("storage-user", "packwing"),
                      ("storage-password", "packwing")]
        for (id, text) in fields {
            let field = id == "storage-password" ? app.secureTextFields[id] : app.textFields[id]
            XCTAssert(field.waitForExistence(timeout: 5), id)
            field.tap()
            field.typeText(text)
        }
        keep(app, "storage-add")
        app.buttons["storage-add"].tap()

        // Added, once it answered; then browsed.
        let row = app.buttons["storage-row"]
        if !row.waitForExistence(timeout: 20) {
            keep(app, "storage-not-added")
            XCTFail("not added: \(app.staticTexts.allElementsBoundByIndex.map(\.label).filter { !$0.isEmpty })")
            return
        }

        // packwing Free keeps one.
        tapWhenReady(add)
        XCTAssert(app.staticTexts["packwing Free keeps one storage across a network. Pro lifts this."].waitForExistence(timeout: 5))
        app.alerts.buttons["OK"].tap()
        let gone = XCTNSPredicateExpectation(predicate: NSPredicate(format: "exists == false"), object: app.alerts.firstMatch)
        XCTAssertEqual(XCTWaiter.wait(for: [gone], timeout: 5), .completed)
        row.tap()
        let zip = app.buttons.matching(identifier: "stored-file")
            .matching(NSPredicate(format: "label BEGINSWITH 'report.zip'")).firstMatch
        XCTAssert(zip.waitForExistence(timeout: 20))
        keep(app, "storage-folder")
        // Once the folder has slid in: a list's row is never "hittable" to XCTest.
        sleep(1)
        zip.tap()

        // Opened where it is.
        XCTAssert(app.staticTexts["Báo cáo quý 3"].waitForExistence(timeout: 20))
        XCTAssert(app.buttons["Extract all"].exists)
        keep(app, "storage-archive")

        _ = send("DELETE", folder)
    }

    /// Whether something answers on `port` of this computer, within a second or two.
    private func listening(_ port: UInt16) -> Bool {
        let connection = NWConnection(host: "127.0.0.1", port: NWEndpoint.Port(rawValue: port)!, using: .tcp)
        let ready = expectation(description: "port \(port)")
        ready.assertForOverFulfill = false
        var up = false
        connection.stateUpdateHandler = { state in
            switch state {
            case .ready: up = true; ready.fulfill()
            case .failed, .waiting: ready.fulfill()
            default: break
            }
        }
        connection.start(queue: .global())
        wait(for: [ready], timeout: 3)
        connection.cancel()
        return up
    }

    /// Against the SFTP server of `make sftp-start` (its Docker container, on port
    /// 18322): skipped without one.
    func testAnSftpServerIsAskedAboutOnceTrustedAndBrowsed() throws {
        guard listening(18322) else { throw XCTSkip("no SFTP server on port 18322") }
        let app = XCUIApplication()
        app.launchArguments = ["-AppleLanguages", "(en)", "-language", "", "-storages", "", "-forgetServers", "YES"]
        app.launch()
        tapWhenReady(app.buttons["storage-new"])
        app.buttons["SFTP"].tap()
        let host = app.textFields["storage-host"]
        XCTAssert(host.waitForExistence(timeout: 5))
        host.tap()
        host.typeText("tester@127.0.0.1:18322")
        let password = app.secureTextFields["storage-password"]
        password.tap()
        password.typeText("packwing")
        app.buttons["storage-add"].tap()

        // A server not known yet: asked about, with its fingerprint.
        XCTAssert(app.staticTexts["Trust this server?"].waitForExistence(timeout: 20))
        XCTAssert(app.staticTexts["127.0.0.1:18322"].exists)
        let fingerprint = app.staticTexts["ssh-fingerprint"]
        XCTAssert(fingerprint.exists)
        XCTAssert(fingerprint.label.hasPrefix("SHA256:"))
        // Whole: none of it cut off.
        XCTAssertGreaterThan(fingerprint.frame.height, 30)
        keep(app, "ssh-question")
        app.buttons["Trust and connect"].tap()

        // Added, and browsed: the tester's home.
        let row = app.buttons["storage-row"]
        XCTAssert(row.waitForExistence(timeout: 20))
        tapWhenReady(row)
        XCTAssert(app.staticTexts["logs"].waitForExistence(timeout: 20))
        keep(app, "sftp-folder")
    }

    func testTheFilesInAnArchiveHaveTheirChecksumsListedAndKeptBesideIt() {
        // Ghi chú.txt's, as `unzip -p … | shasum -a 256` says it.
        let notes = "a96b0aa7a284474b68cdea5ab0da0fb0251c340809610dd147c6329a634d09a7"
        // Given as Mail gives one: packwing may write beside it.
        let sample = Bundle(for: Self.self).url(forResource: "report.zip", withExtension: nil)!
        let app = XCUIApplication()
        app.launchArguments = ["-receive", sample.path, "-AppleLanguages", "(en)", "-language", "",
                               "-checksums", "sha256 sha1"]
        app.launch()
        let folder = app.staticTexts["Báo cáo quý 3"]
        XCTAssert(folder.waitForExistence(timeout: 15))

        // All of its files, in one read.
        app.buttons["More"].tap()
        app.buttons["Checksums"].tap()
        let all = app.buttons.matching(NSPredicate(format: "label BEGINSWITH 'All 3 files'")).firstMatch
        XCTAssert(all.waitForExistence(timeout: 5))
        all.tap()
        app.buttons["checksum-read"].tap()
        let files = app.buttons.matching(identifier: "checksum-file")
        XCTAssert(files.firstMatch.waitForExistence(timeout: 15))
        XCTAssertEqual(files.count, 3)
        keep(app, "checksums-files")
        // One checksum alone, pasted: the file it is of.
        let pasted = app.textFields["checksums-pasted"]
        pasted.tap()
        pasted.typeText(notes)
        XCTAssert(app.staticTexts["It matches the SHA-256 checksum of Báo cáo quý 3/Ghi chú.txt."].waitForExistence(timeout: 5))
        keep(app, "checksums-compared")
        // The keyboard away: Save shows again.
        app.buttons["checksums-pasted-done"].tap()
        tapWhenReady(app.buttons["checksum-save"])
        XCTAssert(app.descendants(matching: .any)["checksum-saved"].waitForExistence(timeout: 5))
        XCTAssert(app.staticTexts.matching(NSPredicate(format: "label CONTAINS 'report.zip.contents.sha256'")).firstMatch.exists)
        keep(app, "checksums-saved")
        app.buttons["Close"].firstMatch.tap()

        // One file, from its menu: its checksums at once.
        tapWhenReady(folder)
        let file = app.staticTexts["Ghi chú.txt"]
        XCTAssert(file.waitForExistence(timeout: 10))
        file.press(forDuration: 1.2)
        app.buttons["Checksums"].tap()
        let sum = app.staticTexts["checksum-sha256"]
        XCTAssert(sum.waitForExistence(timeout: 15))
        XCTAssertEqual(sum.label, notes)
        keep(app, "checksums-one")
    }

    /// Pro by subscription, as `-store fake` plays the App Store: a year's,
    /// held after a restart too, and Free again once it ends.
    func testProIsSubscribedToInSettingsAndHeldAfterARestart() {
        let app = XCUIApplication()
        app.launchArguments = ["-AppleLanguages", "(en)", "-language", "", "-store", "fake", "-fakePro", "NO",
                               "-fakeNoMonthly", "NO"]
        app.launch()
        tapWhenReady(app.buttons.matching(NSPredicate(format: "label == 'Settings'")).firstMatch)
        let plan = app.staticTexts["store-plan"]
        XCTAssert(plan.waitForExistence(timeout: 10))
        XCTAssertEqual(plan.label, "packwing Free")
        let yearly = app.buttons["store-buy-yearly"]
        XCTAssertEqual(yearly.label, "$8.99 a year")
        XCTAssertEqual(app.buttons["store-buy-monthly"].label, "$0.99 a month")
        XCTAssert(app.descendants(matching: .any)["store-terms"].exists)
        XCTAssert(app.descendants(matching: .any)["store-privacy"].exists)
        keep(app, "store-free")
        yearly.tap()
        XCTAssert(app.descendants(matching: .any)["store-held"].waitForExistence(timeout: 5))
        XCTAssertEqual(plan.label, "packwing Pro")
        app.terminate()

        // Kept.
        app.launchArguments = ["-AppleLanguages", "(en)", "-language", "", "-store", "fake"]
        app.launch()
        tapWhenReady(app.buttons.matching(NSPredicate(format: "label == 'Settings'")).firstMatch)
        XCTAssert(app.descendants(matching: .any)["store-held"].waitForExistence(timeout: 10))
        keep(app, "store-pro")

        // Ended: Free, and the plans again.
        app.buttons["store-manage"].tap()
        XCTAssert(app.buttons["store-buy-yearly"].waitForExistence(timeout: 5))
        XCTAssertEqual(plan.label, "packwing Free")
    }

    /// A price the App Store left out: the plan's name on its button.
    func testAPlanWithNoPriceShowsItsName() {
        let app = XCUIApplication()
        app.launchArguments = ["-AppleLanguages", "(en)", "-language", "", "-store", "fake", "-fakePro", "NO",
                               "-fakeNoMonthly", "YES"]
        app.launch()
        tapWhenReady(app.buttons.matching(NSPredicate(format: "label == 'Settings'")).firstMatch)
        let monthly = app.buttons["store-buy-monthly"]
        XCTAssert(monthly.waitForExistence(timeout: 10))
        XCTAssertEqual(monthly.label, "Monthly")
        XCTAssertEqual(app.buttons["store-buy-yearly"].label, "$8.99 a year")
    }

    /// Against the WebDAV server of `make webdav-start`: skipped without one.
    func testAStorageIsWrittenToFromThePhone() throws {
        let server = "http://127.0.0.1:18335"
        let folder = "ui-\(UUID().uuidString.prefix(8))"
        let auth = "Basic " + Data("packwing:packwing".utf8).base64EncodedString()
        func send(_ method: String, _ path: String) -> Int {
            var request = URLRequest(url: URL(string: "\(server)/\(path)")!)
            request.httpMethod = method
            request.setValue(auth, forHTTPHeaderField: "Authorization")
            var status = 0
            let done = expectation(description: method)
            URLSession.shared.dataTask(with: request) { _, response, _ in
                status = (response as? HTTPURLResponse)?.statusCode ?? 0
                done.fulfill()
            }.resume()
            wait(for: [done], timeout: 10)
            return status
        }
        guard send("MKCOL", folder) / 100 == 2 else { throw XCTSkip("no WebDAV server at \(server)") }

        let app = XCUIApplication()
        app.launchArguments = ["-AppleLanguages", "(en)", "-language", "", "-storages", "", "-store", "fake",
                               "-fakePro", "NO"]
        app.launch()
        tapWhenReady(app.buttons["storage-new"])
        app.buttons["WebDAV folder"].tap()
        for (id, text) in [("storage-address", "\(server)/\(folder)"), ("storage-user", "packwing"),
                           ("storage-password", "packwing")] {
            let field = id == "storage-password" ? app.secureTextFields[id] : app.textFields[id]
            XCTAssert(field.waitForExistence(timeout: 5), id)
            field.tap()
            field.typeText(text)
        }
        app.buttons["storage-add"].tap()
        tapWhenReady(app.buttons["storage-row"])

        // A folder made, and deleted.
        tapWhenReady(app.buttons["More"])
        app.buttons["New folder"].tap()
        let name = app.textFields["new-name"]
        XCTAssert(name.waitForExistence(timeout: 5))
        name.typeText("Drafts")
        app.buttons["name-confirm"].tap()
        let drafts = app.staticTexts["Drafts"]
        XCTAssert(drafts.waitForExistence(timeout: 15))
        keep(app, "storage-folder-made")
        sleep(1)
        drafts.press(forDuration: 1.2)
        app.buttons["Delete…"].tap()
        app.buttons.matching(NSPredicate(format: "label == 'Delete'")).firstMatch.tap()
        let gone = XCTNSPredicateExpectation(predicate: NSPredicate(format: "exists == false"), object: drafts)
        XCTAssertEqual(XCTWaiter.wait(for: [gone], timeout: 15), .completed)
        app.terminate()

        // A new archive made there.
        let sample = Bundle(for: Self.self).url(forResource: "report.zip", withExtension: nil)!
        app.launchArguments = ["-new", sample.path, "-AppleLanguages", "(en)", "-language", "",
                               "-newFormat", "zip", "-newCompression", "medium"]
        app.launch()
        let change = app.buttons["Change"]
        XCTAssert(change.waitForExistence(timeout: 15))
        change.tap()
        app.buttons["Storages"].tap()
        let entry = app.buttons.matching(NSPredicate(format: "label CONTAINS %@", folder)).firstMatch
        XCTAssert(entry.waitForExistence(timeout: 10))
        entry.tap()
        tapWhenReady(app.buttons["picker-here"])
        XCTAssert(app.staticTexts.matching(NSPredicate(format: "label CONTAINS %@", folder)).firstMatch.waitForExistence(timeout: 5))
        tapWhenReady(app.buttons["Create"].firstMatch)
        XCTAssert(app.staticTexts["Done"].waitForExistence(timeout: 30))
        keep(app, "storage-created")
        XCTAssertEqual(send("HEAD", "\(folder)/report.zip") / 100, 2, "on the server")
        app.buttons.matching(NSPredicate(format: "label == 'Done'")).firstMatch.tap()

        // Renamed there, and deleted.
        tapWhenReady(app.buttons["storage-row"])
        let made = app.buttons.matching(identifier: "stored-file")
            .matching(NSPredicate(format: "label BEGINSWITH 'report.zip'")).firstMatch
        XCTAssert(made.waitForExistence(timeout: 15))
        sleep(1)
        made.press(forDuration: 1.2)
        app.buttons["Rename"].tap()
        XCTAssert(name.waitForExistence(timeout: 5))
        name.typeText(String(repeating: XCUIKeyboardKey.delete.rawValue, count: 12) + "kept.zip")
        app.buttons["name-confirm"].tap()
        let kept = app.buttons.matching(identifier: "stored-file")
            .matching(NSPredicate(format: "label BEGINSWITH 'kept.zip'")).firstMatch
        XCTAssert(kept.waitForExistence(timeout: 15))
        XCTAssertEqual(send("HEAD", "\(folder)/kept.zip") / 100, 2, "renamed on the server")
        keep(app, "storage-renamed")
        _ = send("DELETE", folder)
    }

    func testOneFileIsCompressedByItselfAndEncryptedWithAge() {
        // Not launch(making:), whose -newFormat would hold the format: it is chosen here.
        let sample = Bundle(for: Self.self).url(forResource: "report.zip", withExtension: nil)!
        let app = XCUIApplication()
        app.launchArguments = ["-new", sample.path, "-AppleLanguages", "(en)", "-language", ""]
        app.launch()
        let format = app.buttons["new-format"]
        XCTAssert(format.waitForExistence(timeout: 15))
        format.tap()
        // One file by itself: the formats for one file are offered too.
        let gz = app.buttons["gz"]
        XCTAssert(gz.waitForExistence(timeout: 5))
        gz.tap()
        XCTAssert(app.staticTexts[".gz"].waitForExistence(timeout: 5))

        // Not ZIP's AES: age, to a passphrase.
        let age = app.switches["Encrypt with age"]
        XCTAssert(age.exists)
        XCTAssertFalse(app.switches.matching(NSPredicate(format: "label BEGINSWITH 'Hide the file names'")).firstMatch.exists)
        age.switches.firstMatch.tap()
        let passphrase = app.secureTextFields["new-password"]
        XCTAssert(passphrase.waitForExistence(timeout: 5))
        XCTAssertEqual(passphrase.placeholderValue, "Passphrase")
        passphrase.tap()
        passphrase.typeText("sample-password")
        keep(app, "create-gz")
        app.buttons["Create"].firstMatch.tap()

        XCTAssert(app.staticTexts["Done"].waitForExistence(timeout: 30))
        XCTAssert(text(app, starting: "report.gz").exists)
        keep(app, "created-gz")
    }
}
