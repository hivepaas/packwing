import CoreTransferable
import Foundation
import Observation
import PhotosUI
import SwiftUI
import UniformTypeIdentifiers

/// A file or folder going into a new archive. It keeps what reaching it
/// needs for as long as it is kept: access to it, chosen in Files; or, for a
/// photo, the copy of it Photos gave, which goes with it.
final class NewItem: Identifiable, @unchecked Sendable {
    let id = UUID()
    let url: URL
    let folder: Bool
    /// From Photos, as a copy of its own.
    let photo: Bool
    private let scoped: Bool

    init(url: URL, scoped: Bool, photo: Bool = false) {
        self.url = url
        self.scoped = scoped
        self.photo = photo
        folder = (try? url.resourceValues(forKeys: [.isDirectoryKey]).isDirectory) ?? false
    }

    /// A file or folder chosen in Files.
    convenience init(chosen url: URL) {
        self.init(url: url, scoped: url.startAccessingSecurityScopedResource())
    }

    deinit {
        if scoped { url.stopAccessingSecurityScopedResource() }
        // A photo's copy is in a folder of its own.
        if photo { try? FileManager.default.removeItem(at: url.deletingLastPathComponent()) }
    }

    var name: String { url.lastPathComponent }

    /// How much it holds, its folders' files and all.
    var size: UInt64 {
        func of(_ url: URL) -> UInt64 {
            UInt64((try? url.resourceValues(forKeys: [.fileSizeKey]).fileSize) ?? 0)
        }
        guard folder else { return of(url) }
        var total: UInt64 = 0
        let files = FileManager.default.enumerator(at: url, includingPropertiesForKeys: [.fileSizeKey])
        while let file = files?.nextObject() as? URL { total += of(file) }
        return total
    }
}

/// What the New archive sheet makes an archive of, as it is chosen.
@MainActor
@Observable
final class NewArchiveDraft: Identifiable {
    let id = UUID()
    private(set) var items: [NewItem] = []
    /// How much they hold, once counted.
    private(set) var size: UInt64?
    /// How many photos are still coming from Photos.
    private(set) var fetching = 0
    /// Why a photo did not come, if one did not.
    var problem: String?

    init(_ items: [NewItem] = []) {
        add(items)
    }

    func add(_ new: [NewItem]) {
        let had = Set(items.map { $0.url.standardizedFileURL })
        items += new.filter { !had.contains($0.url.standardizedFileURL) }
        count()
    }

    func remove(at offsets: IndexSet) {
        items.remove(atOffsets: offsets)
        count()
    }

    /// Takes the photos `picked` in Photos, as copies of their files.
    func add(_ picked: [PhotosPickerItem]) {
        guard !picked.isEmpty else { return }
        fetching += picked.count
        Task {
            var came: [NewItem] = []
            for item in picked {
                do {
                    if let photo = try await item.loadTransferable(type: PhotoFile.self) {
                        came.append(NewItem(url: photo.url, scoped: false, photo: true))
                    }
                } catch {
                    problem = tr("A photo could not be read: {error}", ["error": error.localizedDescription])
                }
                fetching -= 1
            }
            add(came)
        }
    }

    private func count() {
        size = nil
        let items = items
        Task {
            let total = await Task.detached(priority: .utility) {
                items.reduce(0) { $0 + $1.size }
            }.value
            // Only if nothing changed meanwhile.
            if items.map(\.id) == self.items.map(\.id) { size = total }
        }
    }

    /// What they are, as the sheet says it: "12 photos and 1 folder".
    var summary: String {
        let photos = items.count { $0.photo }
        let folders = items.count { $0.folder }
        let files = items.count - photos - folders
        var said: [String] = []
        if photos > 0 { said.append(trN(photos, "{n} photo", "{n} photos")) }
        if files > 0 { said.append(trN(files, "{n} file", "{n} files")) }
        if folders > 0 { said.append(trN(folders, "{n} folder", "{n} folders")) }
        guard !said.isEmpty else { return tr("Nothing yet") }
        // Joined as the language packwing speaks joins a list: its own commas, its own "and".
        let list = ListFormatter()
        list.locale = appLocale
        return list.string(from: said) ?? said.joined(separator: ", ")
    }

    /// What the archive is called unless it is told otherwise: after what it
    /// holds, or with several, after their folder.
    var suggestedName: String {
        switch items.count {
        case 0:
            return tr("Archive")
        case 1:
            let one = items[0]
            return one.folder ? one.name : one.url.deletingPathExtension().lastPathComponent
        default:
            if items.allSatisfy(\.photo) { return tr("Photos") }
            let parents = Set(items.map { $0.url.deletingLastPathComponent().standardizedFileURL })
            guard parents.count == 1, let parent = parents.first, !items[0].photo,
                  parent != AppModel.ownFolder.standardizedFileURL
            else { return tr("Archive") }
            return parent.lastPathComponent
        }
    }
}

/// A photo or video from Photos, as the file it is, copied to a folder of its
/// own (Photos takes its copy back once loading ends).
struct PhotoFile: Transferable {
    let url: URL

    static var transferRepresentation: some TransferRepresentation {
        FileRepresentation(importedContentType: .movie) { try copy($0) }
        FileRepresentation(importedContentType: .image) { try copy($0) }
    }

    private static func copy(_ received: ReceivedTransferredFile) throws -> PhotoFile {
        let folder = URL.temporaryDirectory.appending(path: "new-\(UUID().uuidString)", directoryHint: .isDirectory)
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        let url = folder.appending(path: received.file.lastPathComponent)
        try FileManager.default.copyItem(at: received.file, to: url)
        return PhotoFile(url: url)
    }
}

// MARK: How new archives are made, kept between them

extension ArchiveFormat {
    static let all: [ArchiveFormat] = [.zip, .sevenZip]

    var key: String { self == .zip ? "zip" : "7z" }
    var ext: String { self == .zip ? ".zip" : ".7z" }
    var label: String { self == .zip ? "ZIP" : "7z" }

    init(key: String) {
        self = key == "7z" ? .sevenZip : .zip
    }
}

extension CompressionLevel {
    static let all: [CompressionLevel] = [.store, .lowest, .low, .medium, .high, .highest]

    var key: String {
        switch self {
        case .store: "store"
        case .lowest: "lowest"
        case .low: "low"
        case .medium: "medium"
        case .high: "high"
        case .highest: "highest"
        }
    }

    var label: String {
        switch self {
        case .store: tr("Store")
        case .lowest: tr("Lowest")
        case .low: tr("Low")
        case .medium: tr("Medium")
        case .high: tr("High")
        case .highest: tr("Highest")
        }
    }

    init(key: String) {
        self = Self.all.first { $0.key == key } ?? .medium
    }
}

extension AppModel {
    // MARK: Where new archives go

    /// Where new archives are made: the folder chosen last, else packwing's own.
    var createFolder: URL { keptFolder("createIn") }

    func createIn(_ folder: URL) { keep(folder, as: "createIn") }

    // MARK: Making one

    /// Makes an archive of `draft`'s items, `named` (without its extension),
    /// in `folder`, showing how far it is.
    func create(_ draft: NewArchiveDraft, named name: String, how: NewArchiveHow, in folder: URL) {
        if let job, job.state == .running {
            showingJob = true
            return
        }
        let file = name + how.format.ext
        let running = RunningJob(title: tr("Compressing"), name: file)
        job = running
        showingJob = true
        running.continueInBackground(subtitle: file)
        // Kept until it is done: they hold access to the files.
        let items = draft.items
        Task {
            let scoped = folder.startAccessingSecurityScopedResource()
            defer { if scoped { folder.stopAccessingSecurityScopedResource() } }
            let paths = items.map { $0.url.path(percentEncoded: false) }
            let wanted = folder.appending(path: file).path(percentEncoded: false)
            let job = running.job
            let result = await Task.detached(priority: .userInitiated) {
                Result { try createArchive(paths: paths, wanted: wanted, how: how, job: job) }
            }.value
            withExtendedLifetime(items) {}
            switch result {
            case .success(let made):
                let made = URL(filePath: made)
                running.folder = folder
                running.made = made
                running.finish(.done(detail: tr("{name} in {folder}",
                                                ["name": made.lastPathComponent, "folder": shown(folder)])))
            case .failure(let error):
                running.finish(Self.ended(by: error))
            }
        }
    }
}
