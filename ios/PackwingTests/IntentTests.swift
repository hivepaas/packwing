import AppIntents
import UniformTypeIdentifiers
import XCTest
@testable import Packwing

/// Shortcuts' actions, run as Shortcuts runs them: in packwing, on files given.
final class IntentTests: XCTestCase {
    private var folder: URL!

    override func setUpWithError() throws {
        folder = URL.temporaryDirectory.appending(path: "intent-tests-\(UUID().uuidString)", directoryHint: .isDirectory)
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws {
        try? FileManager.default.removeItem(at: folder)
    }

    /// A sample, as Files gives it: a file where it is.
    private func given(_ name: String) throws -> IntentFile {
        let sample = try XCTUnwrap(Bundle(for: Self.self).url(forResource: name, withExtension: nil))
        let copy = folder.appending(path: name)
        try FileManager.default.copyItem(at: sample, to: copy)
        return IntentFile(fileURL: copy, filename: name, type: UTType(filenameExtension: copy.pathExtension))
    }

    /// The paths the archive `file` holds, opened with `password`.
    private func contents(of file: IntentFile, password: String? = nil) async throws -> [String] {
        var intent = ArchiveContentsIntent()
        intent.archive = file
        intent.password = password
        let result = try await intent.perform()
        return try XCTUnwrap(result.value)
    }

    func testFilesAreCompressedIntoAZipOfTheNameGiven() async throws {
        var intent = CompressIntent()
        intent.files = [try given("report.zip"), try given("locked.7z")]
        intent.format = .zip
        intent.name = "Bundle"
        let result = try await intent.perform()
        let made = try XCTUnwrap(result.value)
        XCTAssertEqual(made.filename, "Bundle.zip")
        let listed = try await contents(of: made)
        XCTAssertEqual(listed.sorted(), ["locked.7z", "report.zip"])
    }

    /// Named after the one file in it, as New archive names it.
    func testAFileIsCompressedUnderItsOwnName() async throws {
        var intent = CompressIntent()
        intent.files = [try given("report.zip.sig")]
        intent.format = .tarZst
        let result = try await intent.perform()
        let made = try XCTUnwrap(result.value)
        XCTAssertEqual(made.filename, "report.zip.tar.zst")
    }

    /// With a password: a 7z whose names are hidden, which lists only with it,
    /// and Shortcuts asks for it when none is given.
    func testA7zWithAPasswordAsksForItToBeRead() async throws {
        var intent = CompressIntent()
        intent.files = [try given("report.zip")]
        intent.format = .sevenZip
        intent.password = "secret words"
        let result = try await intent.perform()
        let made = try XCTUnwrap(result.value)
        XCTAssertEqual(made.filename, "report.7z")

        do {
            _ = try await contents(of: made)
            XCTFail("listed without its password")
        } catch {
            // Asked for, not said as packwing's own problem.
            XCTAssertFalse(error is IntentProblem, "\(error)")
        }
        do {
            _ = try await contents(of: made, password: "not it")
            XCTFail("listed with a wrong password")
        } catch let problem as IntentProblem {
            XCTAssert(problem.message.contains("wrong"), problem.message)
        }
        let listed = try await contents(of: made, password: "secret words")
        XCTAssertEqual(listed, ["report.zip"])
    }

    func testAnArchiveIsExtractedIntoItsFilesAndFolders() async throws {
        var intent = ExtractIntent()
        intent.archive = try given("locked.7z")
        intent.password = "sample-password"
        let result = try await intent.perform()
        let files = try XCTUnwrap(result.value)
        XCTAssertEqual(files.map(\.filename), ["Báo cáo quý 3"])
        let top = try XCTUnwrap(files.first?.fileURL)
        let notes = top.appending(path: "Ghi chú.txt")
        XCTAssert(FileManager.default.fileExists(atPath: notes.path(percentEncoded: false)))
    }

    func testTheContentsOfAnArchiveAreItsFilesPaths() async throws {
        let sample = try given("report.zip")
        let paths = try await contents(of: sample)
        XCTAssertEqual(paths.count, 3)
        XCTAssert(paths.contains("Báo cáo quý 3/Hình ảnh/logo.png"), "\(paths)")
    }

    func testWhatIsNoArchiveIsSaidSo() async throws {
        do {
            let key = try given("test-age.key")
            _ = try await contents(of: key)
            XCTFail("a key file listed as an archive")
        } catch let problem as IntentProblem {
            XCTAssertFalse(problem.message.isEmpty)
        }
    }
}
