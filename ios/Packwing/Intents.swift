import AppIntents
import Foundation
import UniformTypeIdentifiers

// packwing's actions in Shortcuts, Siri and Spotlight: compress files,
// extract an archive, list what one holds. Free, as the app is. They run in
// packwing without showing it, made as Settings › New archives and
// Extracting say.
//
// Every string here a person reads is a literal Xcode takes out at build
// time: scripts/i18n.py gathers them for translation as it does tr()'s, and
// writes Localizable.xcstrings and AppShortcuts.xcstrings from the catalogs.

/// The kinds of archive Compress makes: those for any files.
enum IntentFormat: String, AppEnum {
    case zip
    case sevenZip = "7z"
    case tarZst = "tar.zst"
    case tarGz = "tar.gz"
    case tarXz = "tar.xz"
    case tarBz2 = "tar.bz2"
    case tarLz4 = "tar.lz4"
    case tar

    static let typeDisplayRepresentation = TypeDisplayRepresentation(name: "Format")

    // Their names, which no language says otherwise.
    static let caseDisplayRepresentations: [IntentFormat: DisplayRepresentation] = [
        .zip: "ZIP", .sevenZip: "7z", .tarZst: "tar.zst", .tarGz: "tar.gz", .tarXz: "tar.xz",
        .tarBz2: "tar.bz2", .tarLz4: "tar.lz4", .tar: "tar",
    ]

    var format: ArchiveFormat { ArchiveFormat(key: rawValue) }
}

struct CompressIntent: AppIntent {
    static let title: LocalizedStringResource = "Compress Files"
    static let description = IntentDescription("Makes an archive of the files given, as Settings › New archives says.")

    @Parameter(title: "Files", supportedContentTypes: [.item])
    var files: [IntentFile]

    @Parameter(title: "Format", default: .zip)
    var format: IntentFormat

    @Parameter(title: "Name", description: "Without its extension. After what it holds if none is given.")
    var name: String?

    @Parameter(title: "Password", description: "Kept in the shortcut as it is typed. Leave it empty for none.")
    var password: String?

    static var parameterSummary: some ParameterSummary {
        Summary("Compress \(\.$files) as \(\.$format)") {
            \.$name
            \.$password
        }
    }

    func perform() async throws -> some IntentResult & ReturnsValue<IntentFile> {
        Core.ready()
        guard !files.isEmpty else { throw IntentProblem(tr("Nothing to compress.")) }
        let work = try IntentWork()
        let inputs = try files.map { try work.take($0) }
        defer { inputs.forEach { $0.letGo() } }
        let password = password.flatMap { $0.isEmpty ? nil : $0 }
        let format = format.format
        let defaults = UserDefaults.standard
        let how = NewArchiveHow(
            format: format,
            compression: CompressionLevel(key: defaults.string(forKey: "newCompression") ?? ""),
            password: password,
            hideNames: password != nil && format == .sevenZip && defaults.object(forKey: "hideNames") as? Bool ?? true,
            partSize: nil,
            ageKeys: [],
            storeCompressed: defaults.object(forKey: "storeCompressed") as? Bool ?? true,
            skipMacFiles: defaults.object(forKey: "skipMacFiles") as? Bool ?? true,
            wholeWithAge: false,
            storedExtensions: StoredExtensions.chosen,
            streamAbove: nil)
        let given = name?.trimmingCharacters(in: .whitespacesAndNewlines).replacingOccurrences(of: "/", with: "-")
        let base = given.flatMap { $0.isEmpty ? nil : $0 } ?? Self.suggestedName(inputs.map(\.url))
        let wanted = try work.folder("out").appending(path: base + how.ext)
        let paths = inputs.map { $0.url.path(percentEncoded: false) }
        let made = try await Task.detached(priority: .userInitiated) {
            try IntentProblem.said {
                try createArchive(paths: paths, wanted: wanted.path(percentEncoded: false), how: how, job: Job.quiet())
            }
        }.value
        let url = URL(filePath: made)
        return .result(value: IntentFile(fileURL: url, filename: url.lastPathComponent, type: UTType(filenameExtension: url.pathExtension)))
    }

    /// After the one file or folder, else "Archive", as New archive names it.
    private static func suggestedName(_ urls: [URL]) -> String {
        guard urls.count == 1, let one = urls.first else { return tr("Archive") }
        let folder = (try? one.resourceValues(forKeys: [.isDirectoryKey]).isDirectory) ?? false
        return folder ? one.lastPathComponent : one.deletingPathExtension().lastPathComponent
    }
}

struct ExtractIntent: AppIntent {
    static let title: LocalizedStringResource = "Extract Archive"
    static let description = IntentDescription("Gives the files and folders an archive holds: ZIP, 7z, RAR, tar and the rest packwing opens.")

    @Parameter(title: "Archive", supportedContentTypes: [.item])
    var archive: IntentFile

    @Parameter(title: "Password", description: "For an encrypted archive: asked for when it is needed and not given.")
    var password: String?

    static var parameterSummary: some ParameterSummary {
        Summary("Extract \(\.$archive)") {
            \.$password
        }
    }

    func perform() async throws -> some IntentResult & ReturnsValue<[IntentFile]> {
        Core.ready()
        let work = try IntentWork()
        let input = try work.take(archive)
        defer { input.letGo() }
        let opened = try await IntentProblem.opening(input.url, password: password, asking: $password)
        let out = try work.folder("out")
        let how = AppModel.extractHow
        _ = try await Task.detached(priority: .userInitiated) {
            try IntentProblem.said {
                try opened.extract(destination: out.path(percentEncoded: false), picked: [], how: how, job: Job.quiet())
            }
        }.value
        let items = (try? FileManager.default.contentsOfDirectory(at: out, includingPropertiesForKeys: nil)) ?? []
        return .result(value: items.sorted { $0.lastPathComponent < $1.lastPathComponent }.map { url in
            IntentFile(fileURL: url, filename: url.lastPathComponent, type: UTType(filenameExtension: url.pathExtension))
        })
    }
}

struct ArchiveContentsIntent: AppIntent {
    static let title: LocalizedStringResource = "Get Archive Contents"
    static let description = IntentDescription("Lists the paths of the files an archive holds, without extracting them.")

    @Parameter(title: "Archive", supportedContentTypes: [.item])
    var archive: IntentFile

    @Parameter(title: "Password", description: "For an encrypted archive: asked for when it is needed and not given.")
    var password: String?

    static var parameterSummary: some ParameterSummary {
        Summary("Get the contents of \(\.$archive)") {
            \.$password
        }
    }

    func perform() async throws -> some IntentResult & ReturnsValue<[String]> {
        Core.ready()
        let work = try IntentWork()
        let input = try work.take(archive)
        defer { input.letGo() }
        let opened = try await IntentProblem.opening(input.url, password: password, asking: $password)
        var paths: [String] = []
        func walk(_ folder: [String]) {
            for entry in opened.children(folder: folder) {
                let path = folder + [entry.name]
                if entry.folder {
                    walk(path)
                } else {
                    paths.append(path.joined(separator: "/"))
                }
            }
        }
        walk([])
        return .result(value: paths)
    }
}

struct PackwingShortcuts: AppShortcutsProvider {
    static var appShortcuts: [AppShortcut] {
        AppShortcut(intent: CompressIntent(),
                    phrases: ["Compress files with \(.applicationName)", "Make an archive with \(.applicationName)"],
                    shortTitle: "Compress Files", systemImageName: "archivebox")
        AppShortcut(intent: ExtractIntent(),
                    phrases: ["Extract an archive with \(.applicationName)"],
                    shortTitle: "Extract Archive", systemImageName: "square.and.arrow.down")
        AppShortcut(intent: ArchiveContentsIntent(),
                    phrases: ["Show what is in an archive with \(.applicationName)"],
                    shortTitle: "Get Archive Contents", systemImageName: "list.bullet")
    }
}

// MARK: Running them

/// packwing-core, started once, whether the app shows or an action runs
/// without it.
enum Core {
    private static let started: Void = {
        let support = URL.applicationSupportDirectory
        try? FileManager.default.createDirectory(at: support, withIntermediateDirectories: true)
        start(supportFolder: support.path(percentEncoded: false),
              language: UserDefaults.standard.string(forKey: "language"))
    }()

    static func ready() { _ = started }
}

/// What an action said went wrong, in the person's words.
struct IntentProblem: Error, CustomLocalizedStringResourceConvertible {
    let message: String

    init(_ message: String) { self.message = message }

    var localizedStringResource: LocalizedStringResource { "\(message)" }

    /// `work`, its error said as packwing says it.
    static func said<T>(_ work: () throws -> T) throws -> T {
        do {
            return try work()
        } catch {
            throw IntentProblem(AppModel.said(error))
        }
    }

    /// The archive at `url`, opened with `password`; asked for in Shortcuts
    /// if it needs one and none was given.
    static func opening(
        _ url: URL, password: String?, asking: IntentParameter<String?>
    ) async throws -> Archive {
        let given = password.flatMap { $0.isEmpty ? nil : $0 }
        // A password is an archive's, or an age file's passphrase.
        let secrets = Secrets(password: given, passphrase: given)
        let path = url.path(percentEncoded: false)
        do {
            return try await Task.detached(priority: .userInitiated) {
                try Archive.open(path: path, secrets: secrets, job: Job.quiet())
            }.value
        } catch MobileError.Locked(let message, _, let asks) {
            if given == nil, asks == .password || asks == .passphrase {
                throw asking.needsValueError("This archive is encrypted. What is its password?")
            }
            throw IntentProblem(message)
        } catch {
            throw IntentProblem(AppModel.said(error))
        }
    }
}

/// A folder of its own in packwing's temporary folder for one action: what it
/// is given that comes as bytes, and what it makes. Let go of when packwing
/// next starts: what it gives back is read after the action returns.
struct IntentWork {
    static let prefix = "intent-"
    let root: URL

    init() throws {
        root = URL.temporaryDirectory.appending(path: Self.prefix + UUID().uuidString, directoryHint: .isDirectory)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
    }

    /// A new folder in it, named `name`.
    func folder(_ name: String) throws -> URL {
        let url = root.appending(path: name, directoryHint: .isDirectory)
        try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }

    /// A file given, where packwing reads it: where it is, or written here.
    func take(_ file: IntentFile) throws -> Taken {
        if let url = file.fileURL {
            let scoped = url.startAccessingSecurityScopedResource()
            if FileManager.default.isReadableFile(atPath: url.path(percentEncoded: false)) {
                return Taken(url: url, scoped: scoped)
            }
            if scoped { url.stopAccessingSecurityScopedResource() }
        }
        let folder = try folder("in-\(UUID().uuidString)")
        let name = file.filename.isEmpty ? "file" : file.filename.replacingOccurrences(of: "/", with: "-")
        let url = folder.appending(path: name)
        try file.data.write(to: url)
        return Taken(url: url, scoped: false)
    }

    /// Lets go of what earlier actions left, an hour old or more: none runs that long.
    static func sweep() {
        let files = FileManager.default
        let old = Date.now.addingTimeInterval(-3600)
        let left = (try? files.contentsOfDirectory(at: URL.temporaryDirectory,
                                                   includingPropertiesForKeys: [.creationDateKey])) ?? []
        for url in left where url.lastPathComponent.hasPrefix(prefix) {
            let made = (try? url.resourceValues(forKeys: [.creationDateKey]).creationDate) ?? .distantPast
            if made < old { try? files.removeItem(at: url) }
        }
    }
}

/// A file an action reads, and whether it holds access to it.
struct Taken {
    let url: URL
    let scoped: Bool

    func letGo() {
        if scoped { url.stopAccessingSecurityScopedResource() }
    }
}
