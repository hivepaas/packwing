import SwiftUI
import UniformTypeIdentifiers

// Drag and drop, as an iPad has it, a phone too: a file or folder of an
// archive dragged out to Files, Mail or Notes, extracted only once it is
// dropped; files dropped on packwing opened, made an archive of, or added to
// the archive shown.

// MARK: Out

extension OpenedArchive {
    /// The tag every drag out of this archive carries, to tell it dropped on
    /// it again: the same files, added to where they are.
    var dragTag: String { "com.hivepaas.packwing.entry.\(ObjectIdentifier(self).hashValue)" }

    /// What a drag of the file or folder at `path` gives: it, extracted into
    /// a folder of packwing's own once it is dropped, with what is under it.
    func dragged(_ path: [String], folder: Bool) -> NSItemProvider {
        let provider = NSItemProvider()
        let name = path.last ?? ""
        provider.suggestedName = name
        let type = folder ? UTType.folder : UTType(filenameExtension: (name as NSString).pathExtension) ?? .data
        provider.registerDataRepresentation(forTypeIdentifier: dragTag, visibility: .ownProcess) { done in
            done(Data(), nil)
            return nil
        }
        // Held, with its access to the file, until it is extracted.
        let opened = self
        provider.registerFileRepresentation(for: type, visibility: .all, openInPlace: false) { done in
            let progress = Progress(totalUnitCount: 100)
            let job = Job(listener: DragProgress(progress))
            progress.cancellationHandler = { job.cancel() }
            let temp = URL.temporaryDirectory.appending(path: "drag-\(UUID().uuidString)", directoryHint: .isDirectory)
            let how = AppModel.extractHow
            Task.detached(priority: .userInitiated) {
                do {
                    try FileManager.default.createDirectory(at: temp, withIntermediateDirectories: true)
                    _ = try opened.archive.extract(
                        destination: temp.path(percentEncoded: false), picked: [path], how: how, job: job)
                    progress.completedUnitCount = progress.totalUnitCount
                    done(temp.appending(path: name, directoryHint: folder ? .isDirectory : .notDirectory), false, nil)
                } catch {
                    try? FileManager.default.removeItem(at: temp)
                    done(nil, false, error)
                }
            }
            return progress
        }
        return provider
    }
}

/// Tells a drag's progress how far its extraction is.
private final class DragProgress: JobListener, @unchecked Sendable {
    private let told: Progress

    init(_ told: Progress) { self.told = told }

    func progress(progress: JobProgress) {
        guard progress.totalBytes > 0 else { return }
        told.totalUnitCount = Int64(clamping: progress.totalBytes)
        told.completedUnitCount = Int64(clamping: progress.bytes)
    }
}

// MARK: In

/// A file or folder dropped on packwing: read where it is when the app it
/// came from lets it be, else a copy in packwing's own temporary folder.
struct DroppedFile {
    let url: URL
    /// A copy of packwing's: kept or let go of as packwing likes.
    let copy: Bool

    var name: String { url.lastPathComponent }

    /// It, as what a new archive is made of, or an archive is added: a copy
    /// goes once it is done with.
    var item: NewItem {
        copy ? NewItem(url: url, scoped: false, copied: true) : NewItem(chosen: url)
    }

    /// Takes the files `providers` give, but not when they are the drags
    /// `refusing` tags: whether it takes them.
    @MainActor
    static func accept(_ providers: [NSItemProvider], refusing: String?, take: @escaping ([DroppedFile]) -> Void) -> Bool {
        if let refusing, providers.contains(where: { $0.registeredTypeIdentifiers.contains(refusing) }) {
            return false
        }
        Task { @MainActor in
            let files = await load(providers)
            if !files.isEmpty { take(files) }
        }
        return true
    }

    /// Where copies of what was dropped are made.
    static let prefix = "dropped-"

    /// The files `providers` give, in their order; those that give none (a
    /// piece of text, a link) are left out.
    static func load(_ providers: [NSItemProvider]) async -> [DroppedFile] {
        var files: [DroppedFile] = []
        for provider in providers {
            if let file = await load(provider) { files.append(file) }
        }
        return files
    }

    private static func load(_ provider: NSItemProvider) async -> DroppedFile? {
        let types = provider.registeredTypeIdentifiers
        // A folder as one, not the archive of it some apps give besides.
        let id = types.first { UTType($0)?.conforms(to: .folder) == true }
            ?? types.first { UTType($0).map { $0.conforms(to: .data) || $0.conforms(to: .directory) } == true }
        guard let id else { return nil }
        return await withCheckedContinuation { told in
            _ = provider.loadInPlaceFileRepresentation(forTypeIdentifier: id) { url, inPlace, _ in
                guard let url else { return told.resume(returning: nil) }
                if inPlace { return told.resume(returning: DroppedFile(url: url, copy: false)) }
                // Gone once this returns: copied in a folder of its own, under
                // the name the app it came from gave it.
                let folder = URL.temporaryDirectory.appending(path: prefix + UUID().uuidString, directoryHint: .isDirectory)
                let copy = folder.appending(path: name(of: provider, at: url, type: id))
                do {
                    try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
                    try FileManager.default.copyItem(at: url, to: copy)
                    told.resume(returning: DroppedFile(url: copy, copy: true))
                } catch {
                    try? FileManager.default.removeItem(at: folder)
                    told.resume(returning: nil)
                }
            }
        }
    }

    /// The name the app it came from gave it, with its extension; else its
    /// file's, or, for a hidden one of the system's, what kind of file it is.
    private static func name(of provider: NSItemProvider, at url: URL, type: String) -> String {
        let ext = url.pathExtension
        var name = provider.suggestedName?.replacingOccurrences(of: "/", with: "-") ?? ""
        if name.isEmpty {
            guard url.lastPathComponent.hasPrefix(".") else { return url.lastPathComponent }
            name = UTType(type)?.localizedDescription ?? "file"
        }
        if (name as NSString).pathExtension.isEmpty, !ext.isEmpty {
            name += ".\(ext)"
        }
        return name
    }

    /// Lets go of what drags in and out left in the temporary folder: nothing
    /// is using it when packwing starts.
    static func sweep() {
        let files = FileManager.default
        let left = (try? files.contentsOfDirectory(at: URL.temporaryDirectory, includingPropertiesForKeys: nil)) ?? []
        for url in left where url.lastPathComponent.hasPrefix(prefix) || url.lastPathComponent.hasPrefix("drag-") {
            try? files.removeItem(at: url)
        }
    }
}

extension AppModel {
    /// Files dropped on packwing, but on an archive or New archive: one
    /// archive is opened; anything else is what a new archive is made of.
    func take(_ dropped: [DroppedFile]) {
        guard !dropped.isEmpty else { return }
        if dropped.count == 1, let file = dropped.first, FileKind.of(name: file.name) == .archive {
            // A copy is kept as one another app gave, for Recent to open again.
            open(file.copy ? Self.keep(given: file.url) ?? file.url : file.url)
            return
        }
        if let drafting {
            drafting.add(dropped.map(\.item))
        } else {
            drafting = NewArchiveDraft(dropped.map(\.item))
        }
    }
}

/// Where files may be dropped: said over it while they are held over it.
struct DropZone: ViewModifier {
    /// What is said while files are held over it.
    let hint: String
    /// The tag of drags it refuses: an archive's own files.
    var refusing: String?
    let take: ([DroppedFile]) -> Void
    @State private var over = false

    func body(content: Content) -> some View {
        content
            .onDrop(of: [.item], isTargeted: $over) { providers in
                DroppedFile.accept(providers, refusing: refusing, take: take)
            }
            .overlay {
                if over {
                    DropHint(text: hint)
                        .transition(.opacity)
                }
            }
            .animation(.easeOut(duration: 0.15), value: over)

    }
}

extension View {
    /// Takes files dropped on it, saying `hint` while they are held over it.
    func dropZone(_ hint: String, refusing: String? = nil, take: @escaping ([DroppedFile]) -> Void) -> some View {
        modifier(DropZone(hint: hint, refusing: refusing, take: take))
    }
}

/// What a drop zone says while files are held over it: framed, over what it covers.
private struct DropHint: View {
    let text: String

    var body: some View {
        RoundedRectangle(cornerRadius: 24, style: .continuous)
            .strokeBorder(Tonal.accent, style: StrokeStyle(lineWidth: 3, dash: [10, 8]))
            .background(Tonal.accent.opacity(0.08), in: .rect(cornerRadius: 24, style: .continuous))
            .overlay {
                Label(text, systemImage: "arrow.down.doc")
                    .font(.nunito(17, .semibold, relativeTo: .headline))
                    .foregroundStyle(Tonal.accent)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, 20)
                    .padding(.vertical, 12)
                    .glassEffect(.regular, in: .capsule)
                    .padding(24)
            }
            .padding(8)
            .allowsHitTesting(false)
            .accessibilityIdentifier("drop-hint")
    }
}
