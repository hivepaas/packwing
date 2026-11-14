import Foundation
import Observation
import SwiftUI

/// The app's tabs.
enum AppTab: Hashable {
    case archives, settings, search
}

/// An archive open: the core's, and access to its file for as long as it is
/// kept, as reading a file of it or extracting reads the file again.
final class OpenedArchive: Hashable, @unchecked Sendable {
    let url: URL
    let archive: Archive
    let info: ArchiveInfo
    private let scoped: Bool

    init(url: URL, archive: Archive, scoped: Bool) {
        self.url = url
        self.archive = archive
        self.info = archive.info()
        self.scoped = scoped
    }

    deinit {
        if scoped { url.stopAccessingSecurityScopedResource() }
    }

    static func == (a: OpenedArchive, b: OpenedArchive) -> Bool { a === b }
    func hash(into h: inout Hasher) { h.combine(ObjectIdentifier(self)) }
}

/// A folder of an open archive, as the navigation shows it.
struct Place: Hashable {
    let archive: OpenedArchive
    /// Its path in the archive; empty for the top.
    let folder: [String]
}

/// An archive opened before, as Recent lists it: a bookmark to its file,
/// which reaches it again, wherever it is in Files.
struct RecentArchive: Codable, Identifiable, Hashable {
    var id = UUID()
    var bookmark: Data
    var name: String
    var size: UInt64
    var files: UInt64
    var opened: Date
    var locked: Bool
}

/// What the password sheet asks for.
struct PasswordAsk: Identifiable {
    let id = UUID()
    let url: URL
    let name: String
    /// An age file's passphrase, rather than an archive's password.
    let passphrase: Bool
    /// What was wrong with the one given, if one was.
    let problem: String?
}

@MainActor
@Observable
final class AppModel {
    var tab: AppTab = .archives
    var path: [Place] = []
    var recent: [RecentArchive] = []
    var asking: PasswordAsk?
    /// The name of the archive being opened, while it is.
    var opening: String?
    var failure: String?
    /// The language in use: the screens are drawn again when it changes.
    var language = Packwing.language()

    /// The long job running or last run, and whether its screen shows.
    var job: RunningJob?
    var showingJob = false
    /// What the New archive sheet is making an archive of, while it shows.
    var drafting: NewArchiveDraft?
    /// The copy of a file Quick Look shows.
    var previewing: URL?
    /// The name of the file being read to preview it, while it is.
    var reading: String?
    /// Counts the folders chosen to save in, so that what shows it is drawn again.
    var foldersChanged = 0

    @ObservationIgnored private var openJob: Job?
    @ObservationIgnored var readJob: Job?
    /// What a sheet starts once it is gone: a job's screen shown while a
    /// sheet is still going is not shown at all.
    @ObservationIgnored var afterSheet: (() -> Void)?

    /// Starts what the sheet just gone left to start.
    func sheetGone() {
        let then = afterSheet
        afterSheet = nil
        then?()
    }

    init() {
        start(supportFolder: Self.support.path(percentEncoded: false),
              language: UserDefaults.standard.string(forKey: "language"))
        language = Packwing.language()
        recent = Self.loadRecent()
    }

    /// The app's own folder for what it keeps: Application Support.
    static let support: URL = {
        let url = URL.applicationSupportDirectory
        try? FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }()

    // MARK: Opening

    func open(_ url: URL) {
        Task { await open(url, secrets: Secrets(password: nil, passphrase: nil)) }
    }

    /// Opens `url` with `secrets`: shows it, or asks for what it needs.
    func open(_ url: URL, secrets: Secrets) async {
        openJob?.cancel()
        let job = Job.quiet()
        openJob = job
        let scoped = url.startAccessingSecurityScopedResource()
        let name = url.lastPathComponent
        let file = url.path(percentEncoded: false)
        opening = name
        let result = await Task.detached(priority: .userInitiated) {
            Result { try Archive.open(path: file, secrets: secrets, job: job) }
        }.value
        if openJob === job {
            opening = nil
            openJob = nil
        }
        switch result {
        case .success(let archive):
            asking = nil
            let opened = OpenedArchive(url: url, archive: archive, scoped: scoped)
            remember(url, opened.info, locked: secrets.password != nil || secrets.passphrase != nil)
            tab = .archives
            path = [Place(archive: opened, folder: [])]
        case .failure(let error):
            if scoped { url.stopAccessingSecurityScopedResource() }
            switch error as? MobileError {
            case .Locked(_, let passphrase):
                asking = PasswordAsk(url: url, name: name, passphrase: passphrase, problem: nil)
            case .WrongPassword(let message):
                asking = PasswordAsk(url: url, name: name, passphrase: asking?.passphrase ?? false, problem: message)
            case .Cancelled:
                break
            case .NotAnArchive(let message), .Failed(let message):
                asking = nil
                failure = message
            case nil:
                asking = nil
                failure = error.localizedDescription
            }
        }
    }

    /// Stops opening the archive being opened.
    func stopOpening() {
        openJob?.cancel()
        openJob = nil
        opening = nil
    }

    /// Opens what the password sheet asked for, with `secret`.
    func unlock(_ ask: PasswordAsk, with secret: String) {
        let secrets = ask.passphrase
            ? Secrets(password: nil, passphrase: secret)
            : Secrets(password: secret, passphrase: nil)
        Task { await open(ask.url, secrets: secrets) }
    }

    func open(_ item: RecentArchive) {
        var stale = false
        guard let url = try? URL(resolvingBookmarkData: item.bookmark, bookmarkDataIsStale: &stale) else {
            failure = tr("This archive is not where it was any more.")
            return
        }
        open(url)
    }

    // MARK: Recent

    private static var recentFile: URL { support.appending(path: "recent.json") }

    private static func loadRecent() -> [RecentArchive] {
        guard let data = try? Data(contentsOf: recentFile) else { return [] }
        return (try? JSONDecoder().decode([RecentArchive].self, from: data)) ?? []
    }

    private func saveRecent() {
        guard let data = try? JSONEncoder().encode(recent) else { return }
        try? data.write(to: Self.recentFile, options: .atomic)
    }

    /// Puts `url` first in Recent.
    private func remember(_ url: URL, _ info: ArchiveInfo, locked: Bool) {
        guard let bookmark = try? url.bookmarkData() else { return }
        var stale = false
        recent.removeAll { item in
            (try? URL(resolvingBookmarkData: item.bookmark, bookmarkDataIsStale: &stale))?.standardizedFileURL
                == url.standardizedFileURL
        }
        recent.insert(
            RecentArchive(bookmark: bookmark, name: info.name, size: info.packed, files: info.files,
                          opened: .now, locked: locked || info.encryption != nil),
            at: 0)
        recent = Array(recent.prefix(50))
        saveRecent()
    }

    func forget(_ item: RecentArchive) {
        recent.removeAll { $0.id == item.id }
        saveRecent()
    }

    // MARK: Settings

    /// Speaks `tag`, or the system's language for none.
    func speak(_ tag: String?) {
        UserDefaults.standard.set(tag, forKey: "language")
        setLanguage(choice: tag)
        language = Packwing.language()
    }
}
