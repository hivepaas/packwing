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
    /// What it holds, read again once it is changed.
    private(set) var archive: Archive
    private(set) var info: ArchiveInfo
    /// How many times it was changed, so that what was read of it before is not taken for it.
    private(set) var version = 0
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

    /// It, as `archive` reads it now it is changed.
    func changed(to archive: Archive) {
        self.archive = archive
        info = archive.info()
        version += 1
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

/// Where an archive is opened from: a file (of Files, or packwing's own), or
/// a file of a storage across the network, read where it is.
enum OpenSpot {
    case file(URL)
    /// Its storage, its path there, and its size.
    case stored(StorageHandle, [String], UInt64)

    var name: String {
        switch self {
        case .file(let url): url.lastPathComponent
        case .stored(_, let path, _): path.last ?? ""
        }
    }
}

/// What the password sheet asks for.
struct PasswordAsk: Identifiable {
    let id = UUID()
    let spot: OpenSpot
    let name: String
    /// An archive's password, an age file's passphrase, the key file it was
    /// encrypted to, or that key file's passphrase.
    let asks: Asks
    /// What was given before, and is given again with what is asked: the key
    /// file, when its passphrase is asked.
    let given: Secrets
    /// What was wrong with the one given, if one was.
    let problem: String?
}

@MainActor
@Observable
final class AppModel {
    var tab: AppTab = .archives
    /// What the Archives tab shows over its first screen: storages' folders, an
    /// archive's.
    var path = NavigationPath()
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
    /// Counts the changes made to archives, so that what shows one is drawn again.
    var archivesChanged = 0
    /// Counts the changes made on storages, so that their folders shown read themselves again.
    var storagesChanged = 0
    /// What is being done to the archive shown, and its name, while it is.
    var changing: (title: String, name: String)?
    @ObservationIgnored var changeJob: Job?
    /// The public keys whose signatures say who signed a file.
    var trustedKeys = AppModel.loadKeys()
    /// The storages across the network added, in the order they were.
    var storages = AppModel.loadStorages()
    /// packwing Pro, bought in the App Store, or to buy.
    let store = Store()
    /// Each storage open, by its id: its connection is kept for what follows.
    @ObservationIgnored var handles: [String: StorageHandle] = [:]

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
        // Started already if a Shortcuts action ran before packwing showed.
        Core.ready()
        language = Packwing.language()
        // SFTP's questions, asked in a window of their own.
        askSshWith(questions: SshAsker())
        #if DEBUG
        // `-forgetServers YES`: no server trusted yet, as on a new phone.
        if UserDefaults.standard.bool(forKey: "forgetServers") {
            try? FileManager.default.removeItem(at: Self.support.appending(path: "ssh/known_hosts"))
        }
        #endif
        recent = Self.loadRecent()
        sweepReceived()
        DroppedFile.sweep()
        IntentWork.sweep()
    }

    /// The app's own folder for what it keeps: Application Support.
    static let support: URL = {
        let url = URL.applicationSupportDirectory
        try? FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }()

    // MARK: Opening

    func open(_ url: URL) {
        let url = Self.receive(url)
        Task { await open(.file(url), secrets: Secrets()) }
    }

    /// Opens the archive at `spot` with `secrets`: shows it, or asks for what it needs.
    func open(_ spot: OpenSpot, secrets: Secrets) async {
        openJob?.cancel()
        let job = Job.quiet()
        openJob = job
        let url: URL
        let scoped: Bool
        switch spot {
        case .file(let file):
            url = file
            scoped = file.startAccessingSecurityScopedResource()
        case .stored(let handle, let path, _):
            url = handle.url(of: path)
            scoped = false
        }
        let name = spot.name
        opening = name
        let result = await Task.detached(priority: .userInitiated) {
            Result { () throws -> Archive in
                switch spot {
                case .file(let file):
                    try Archive.open(path: file.path(percentEncoded: false), secrets: secrets, job: job)
                case .stored(let handle, let path, let size):
                    try handle.storage.openArchive(path: path, size: size, secrets: secrets, job: job)
                }
            }
        }.value
        if openJob === job {
            opening = nil
            openJob = nil
        }
        switch result {
        case .success(let archive):
            asking = nil
            let opened = OpenedArchive(url: url, archive: archive, scoped: scoped)
            tab = .archives
            if case .stored = spot {
                // Over the storage's folder it is in, to go back to.
                path.append(Place(archive: opened, folder: []))
            } else {
                remember(url, opened.info, locked: secrets != Secrets())
                path = NavigationPath([Place(archive: opened, folder: [])])
            }
        case .failure(let error):
            if scoped { url.stopAccessingSecurityScopedResource() }
            switch error as? MobileError {
            case .Locked(let message, _, let asks):
                // A key file asked for again, one given: it was not the key.
                let again = asks == .keyFile && secrets.keyFile != nil
                asking = PasswordAsk(spot: spot, name: name, asks: asks, given: secrets,
                                     problem: again ? message : nil)
            case .WrongPassword(let message):
                asking = PasswordAsk(spot: spot, name: name, asks: asking?.asks ?? .password,
                                     given: asking?.given ?? Secrets(), problem: message)
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

    /// Opens what the password sheet asked for, with `secret` typed, and what
    /// was given before.
    func unlock(_ ask: PasswordAsk, with secret: String) {
        var secrets = ask.given
        switch ask.asks {
        case .password: secrets.password = secret
        case .passphrase: secrets.passphrase = secret
        case .keyPassphrase: secrets.keyPassphrase = secret
        case .keyFile: return
        }
        Task { await open(ask.spot, secrets: secrets) }
    }

    /// Opens what the password sheet asked for with the key file `key`, and
    /// what was given before but a passphrase of another key file.
    func unlock(_ ask: PasswordAsk, key: Data) {
        var secrets = ask.given
        secrets.keyFile = key
        secrets.keyPassphrase = nil
        Task { await open(ask.spot, secrets: secrets) }
    }

    func open(_ item: RecentArchive) {
        guard let url = Self.resolve(item) else {
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
    func remember(_ url: URL, _ info: ArchiveInfo, locked: Bool) {
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
        // A copy another app gave goes with it.
        if let url = Self.resolve(item), let folder = Self.receivedFolder(of: url) {
            try? FileManager.default.removeItem(at: folder)
        }
    }

    private static func resolve(_ item: RecentArchive) -> URL? {
        var stale = false
        return try? URL(resolvingBookmarkData: item.bookmark, bookmarkDataIsStale: &stale)
    }

    // MARK: Received

    // An archive shared from Files is read where it is; one from Mail or
    // Messages comes as a copy, in an Inbox of packwing's own: Documents/Inbox,
    // which Files shows as a folder of packwing's, or tmp/<its id>-Inbox, which
    // iOS empties when it likes. Moved out of either into a folder of its own,
    // it is seen without being saved anywhere the person keeps files, and
    // Recent opens it again. It is kept while Recent lists it, at most
    // `receivedFor` from when it was last opened.

    /// Where the copies other apps gave are kept, out of Files.
    static let received: URL = support.appending(path: "Received", directoryHint: .isDirectory)
    /// How long a copy another app gave is kept, from when it was last opened.
    static let receivedFor: TimeInterval = 30 * 24 * 60 * 60

    /// `url`, or for a copy another app gave in packwing's Inbox, where it is
    /// kept now.
    static func receive(_ url: URL) -> URL {
        let file = url.resolvingSymlinksInPath()
        let home = URL.homeDirectory.resolvingSymlinksInPath().pathComponents
        let parts = file.pathComponents
        guard parts.starts(with: home), parts.count > home.count + 1,
              let inbox = parts[home.count...].dropLast().firstIndex(where: { $0 == "Inbox" || $0.hasSuffix("-Inbox") })
        else { return url }
        guard let kept = keep(given: file) else { return url }
        // The Inbox goes once it is empty, and with it its folder in Files.
        let inboxFolder = URL(filePath: NSString.path(withComponents: Array(parts[...inbox])), directoryHint: .isDirectory)
        if (try? FileManager.default.contentsOfDirectory(atPath: inboxFolder.path(percentEncoded: false)))?.isEmpty == true {
            try? FileManager.default.removeItem(at: inboxFolder)
        }
        return kept
    }

    /// Where the copy `file` another app gave is kept now, moved out of where
    /// it was given: in Received, with the same one kept before if there is.
    static func keep(given file: URL) -> URL? {
        let files = FileManager.default
        if let same = keptAlready(file) {
            // The same attachment shared again: the copy kept the first time.
            try? files.removeItem(at: file)
            return same
        }
        let folder = received.appending(path: UUID().uuidString, directoryHint: .isDirectory)
        let kept = folder.appending(path: file.lastPathComponent)
        do {
            try files.createDirectory(at: folder, withIntermediateDirectories: true)
            do {
                try files.moveItem(at: file, to: kept)
            } catch {
                try files.copyItem(at: file, to: kept)
                try? files.removeItem(at: file)
            }
        } catch {
            try? files.removeItem(at: folder)
            return nil
        }
        return kept
    }

    /// A copy kept before of the same name and the same bytes as `file`.
    private static func keptAlready(_ file: URL) -> URL? {
        let files = FileManager.default
        let folders = (try? files.contentsOfDirectory(at: received, includingPropertiesForKeys: nil)) ?? []
        return folders.lazy
            .map { $0.appending(path: file.lastPathComponent) }
            .first { files.contentsEqual(atPath: $0.path(percentEncoded: false), andPath: file.path(percentEncoded: false)) }
    }

    /// The folder of the copy `url` is, if it is one another app gave.
    private static func receivedFolder(of url: URL) -> URL? {
        let base = received.resolvingSymlinksInPath().pathComponents
        let parts = url.resolvingSymlinksInPath().pathComponents
        guard parts.starts(with: base), parts.count > base.count + 1 else { return nil }
        return URL(filePath: NSString.path(withComponents: Array(parts[...base.count])), directoryHint: .isDirectory)
    }

    /// Lets go of the copies Recent no longer lists, or not opened for long.
    private func sweepReceived() {
        let old = Date.now.addingTimeInterval(-Self.receivedFor)
        var kept = Set<String>()
        let before = recent.count
        recent.removeAll { item in
            guard let url = Self.resolve(item), let folder = Self.receivedFolder(of: url) else { return false }
            if item.opened < old || !FileManager.default.fileExists(atPath: url.path(percentEncoded: false)) {
                return true
            }
            kept.insert(folder.lastPathComponent)
            return false
        }
        if recent.count != before { saveRecent() }
        let folders = (try? FileManager.default.contentsOfDirectory(at: Self.received, includingPropertiesForKeys: nil)) ?? []
        for folder in folders where !kept.contains(folder.lastPathComponent) {
            try? FileManager.default.removeItem(at: folder)
        }
    }

    // MARK: Settings

    /// Speaks `tag`, or the system's language for none.
    func speak(_ tag: String?) {
        UserDefaults.standard.set(tag, forKey: "language")
        setLanguage(choice: tag)
        language = Packwing.language()
    }
}
