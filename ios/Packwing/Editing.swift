import Foundation
import PhotosUI
import SwiftUI

/// Changes to an open archive: files added to it, deleted from it, renamed in
/// it, by packwing-core, as the desktop makes them.
///
/// A change writes the archive again beside it and puts it in its place, so
/// one that fails or is stopped leaves it as it was. Where packwing may write
/// beside it (its own folder, a copy Mail gave), that is done where it is.
/// Elsewhere iOS gives packwing the file it opened, not its folder: a copy of
/// it is changed, in packwing's temporary folder, and written over the file.
extension AppModel {
    /// Changes `opened` by `work` (given the archive, and the path of the file
    /// to change: its own, or its copy's), showing `title` while it does, and
    /// reads it again once it is changed.
    func change(
        _ opened: OpenedArchive,
        _ title: String,
        work: @escaping @Sendable (Archive, String, Job) throws -> Void
    ) {
        guard changing == nil else { return }
        let job = Job.quiet()
        changeJob = job
        changing = (title, opened.info.name)
        Task {
            defer {
                changing = nil
                changeJob = nil
            }
            let url = opened.url
            let archive = opened.archive
            let result = await Task.detached(priority: .userInitiated) {
                Result { () throws -> Archive in
                    try Self.changeFile(url) { file in try work(archive, file, job) }
                    return try archive.reopen(job: Job.quiet())
                }
            }.value
            switch result {
            case .success(let now):
                opened.changed(to: now)
                archivesChanged += 1
                remember(url, opened.info, locked: opened.info.encryption != nil)
            case .failure(let error):
                if case .Cancelled = error as? MobileError { return }
                failure = (error as? MobileError).map(reason) ?? error.localizedDescription
            }
        }
    }

    /// Stops the change being made: the archive stays as it was.
    func stopChanging() {
        changeJob?.cancel()
    }

    /// Runs `change` on the file at `url`, or on a copy of it written over it
    /// after, where packwing may not write beside it.
    nonisolated private static func changeFile(_ url: URL, change: (String) throws -> Void) throws {
        let folder = url.deletingLastPathComponent()
        var beside = FileManager.default.isWritableFile(atPath: folder.path(percentEncoded: false))
        #if DEBUG
        // The Simulator lets packwing write beside a file of Files, which an
        // iPhone may not: `defaults write com.hivepaas.packwing changeThroughCopy
        // -bool YES` takes the way an iPhone may take.
        if UserDefaults.standard.bool(forKey: "changeThroughCopy") { beside = false }
        #endif
        if beside {
            try change(url.path(percentEncoded: false))
            return
        }
        let work = URL.temporaryDirectory.appending(path: "change-\(UUID().uuidString)", directoryHint: .isDirectory)
        try FileManager.default.createDirectory(at: work, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: work) }
        let copy = work.appending(path: url.lastPathComponent)
        try coordinated(reading: url) { try FileManager.default.copyItem(at: $0, to: copy) }
        try change(copy.path(percentEncoded: false))
        try coordinated(writing: url) { try overwrite($0, with: copy) }
    }

    /// `file`'s bytes made `with`'s, in place: all an app may do to a file of
    /// Files it was given without its folder.
    nonisolated private static func overwrite(_ file: URL, with new: URL) throws {
        let input = try FileHandle(forReadingFrom: new)
        defer { try? input.close() }
        let output = try FileHandle(forWritingTo: file)
        defer { try? output.close() }
        try output.truncate(atOffset: 0)
        while let chunk = try input.read(upToCount: 1 << 20), !chunk.isEmpty {
            try output.write(contentsOf: chunk)
        }
        try output.synchronize()
    }

    nonisolated private static func coordinated(reading url: URL, _ body: (URL) throws -> Void) throws {
        var failed: Error?
        var coordination: NSError?
        NSFileCoordinator().coordinate(readingItemAt: url, options: [], error: &coordination) { at in
            do { try body(at) } catch { failed = error }
        }
        if let error = coordination ?? failed { throw error }
    }

    nonisolated private static func coordinated(writing url: URL, _ body: (URL) throws -> Void) throws {
        var failed: Error?
        var coordination: NSError?
        NSFileCoordinator().coordinate(writingItemAt: url, options: .forReplacing, error: &coordination) { at in
            do { try body(at) } catch { failed = error }
        }
        if let error = coordination ?? failed { throw error }
    }

    // MARK: The changes

    func delete(_ picked: [[String]], from opened: OpenedArchive) {
        change(opened, tr("Deleting…")) { archive, file, job in
            try archive.delete(file: file, picked: picked, job: job)
        }
    }

    func rename(_ path: [String], to name: String, in opened: OpenedArchive) {
        change(opened, tr("Renaming…")) { archive, file, job in
            try archive.rename(file: file, from: path, to: name, job: job)
        }
    }

    /// Adds `items` to `folder` of `opened`; a file whose name is there goes
    /// as `onExisting` says. They are kept until it is done: they hold access
    /// to the files.
    func add(_ items: [NewItem], to folder: [String], of opened: OpenedArchive, onExisting: OnExisting) {
        let paths = items.map { $0.url.path(percentEncoded: false) }
        change(opened, tr("Adding…")) { archive, file, job in
            try withExtendedLifetime(items) {
                try archive.add(file: file, paths: paths, folder: folder, onExisting: onExisting, job: job)
            }
        }
    }
}

/// Whether `opened` has a signature beside it, which a change leaves behind.
func signedBeside(_ opened: OpenedArchive) -> Bool {
    let signature = opened.url.deletingLastPathComponent().appending(path: opened.url.lastPathComponent + ".sig")
    return FileManager.default.fileExists(atPath: signature.path(percentEncoded: false))
}

/// The photos `picked` in Photos, as copies of their files; those that do not
/// come are left out.
func photoItems(_ picked: [PhotosPickerItem]) async -> [NewItem] {
    var came: [NewItem] = []
    for item in picked {
        if let photo = try? await item.loadTransferable(type: PhotoFile.self) {
            came.append(NewItem(url: photo.url, scoped: false, photo: true))
        }
    }
    return came
}

/// What an archive shown is asked to change, as its screen asks it.
struct ArchiveChanges: ViewModifier {
    @Environment(AppModel.self) private var model
    let opened: OpenedArchive
    let folder: [String]
    /// The names in the folder, to tell a file added of one of them.
    let names: Set<String>
    @Binding var deleting: [[String]]?
    @Binding var renaming: [String]?
    @Binding var adding: NewSource?
    /// Items whose names are in the folder already, until it is said what of them.
    @State private var clashing: [NewItem]?

    func body(content: Content) -> some View {
        content
            .confirmationDialog(deleteTitle, isPresented: Binding { deleting != nil } set: { if !$0 { deleting = nil } },
                                titleVisibility: .visible, presenting: deleting) { picked in
                Button(tr("Delete"), role: .destructive) { model.delete(picked, from: opened) }
            } message: { _ in
                Text(signedBeside(opened)
                     ? tr("It goes from the archive for good, folders with all they hold. This can’t be undone.") + "\n\n"
                        + tr("This archive is signed. After this change it will no longer match its signature, until it is signed again.")
                     : tr("It goes from the archive for good, folders with all they hold. This can’t be undone."))
            }
            // A sheet, not an alert: an alert's text field and buttons lost
            // their first tap now and then, shown from a context menu.
            .sheet(item: Binding { renaming.map(RenamePath.init) } set: { renaming = $0?.path }) { what in
                RenameSheet(path: what.path, signed: signedBeside(opened)) { name in
                    model.rename(what.path, to: name, in: opened)
                }
            }
            .choosingNewItems($adding) { items in
                take(items)
            } photos: { picked in
                Task { take(await photoItems(picked)) }
            }
            .confirmationDialog(tr("If a file of that name is there"),
                                isPresented: Binding { clashing != nil } set: { if !$0 { clashing = nil } },
                                titleVisibility: .visible, presenting: clashing) { items in
                Button(tr("Replace it")) { model.add(items, to: folder, of: opened, onExisting: .replace) }
                Button(tr("Skip it")) { model.add(items, to: folder, of: opened, onExisting: .skip) }
                Button(tr("Cancel"), role: .cancel) {}
            } message: { items in
                Text(items.filter { names.contains($0.name) }.map(\.name).joined(separator: ", "))
            }
    }

    private var deleteTitle: String {
        guard let deleting else { return "" }
        let what = deleting.count == 1
            ? deleting[0].last ?? ""
            : tr("{many} items", ["many": "\(deleting.count)"])
        return tr("Delete {what}?", ["what": what])
    }

    /// Adds `items`, or asks first what of those whose names are there.
    private func take(_ items: [NewItem]) {
        guard !items.isEmpty else { return }
        if items.contains(where: { names.contains($0.name) }) {
            clashing = items
        } else {
            model.add(items, to: folder, of: opened, onExisting: .stop)
        }
    }
}

/// What the Rename sheet renames.
private struct RenamePath: Identifiable {
    let path: [String]
    var id: [String] { path }
}

/// A new name for a file or folder of an archive.
private struct RenameSheet: View {
    @Environment(\.dismiss) private var dismiss
    let path: [String]
    /// Whether the archive has a signature beside it, which the change leaves behind.
    let signed: Bool
    let rename: (String) -> Void
    @State private var name: String
    @FocusState private var typing: Bool

    init(path: [String], signed: Bool, rename: @escaping (String) -> Void) {
        self.path = path
        self.signed = signed
        self.rename = rename
        _name = State(initialValue: path.last ?? "")
    }

    private var newName: String { name.trimmingCharacters(in: .whitespacesAndNewlines) }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    // A file's name: as typed, not corrected.
                    TextField(tr("New name"), text: $name)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)
                        .focused($typing)
                        .submitLabel(.done)
                        .onSubmit(done)
                        .accessibilityIdentifier("new-name")
                } footer: {
                    if signed {
                        Text(tr("This archive is signed. After this change it will no longer match its signature, until it is signed again."))
                    }
                }
                .listRowBackground(Tonal.surface)
            }
            .scrollContentBackground(.hidden)
            .background(Tonal.page)
            .navigationTitle(tr("Rename {what}", ["what": path.last ?? ""]))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(role: .close) { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(role: .confirm, action: done)
                        .disabled(newName.isEmpty || newName == path.last)
                        .accessibilityLabel(tr("Rename"))
                }
            }
            .onAppear { typing = true }
        }
        .presentationDetents([.medium])
    }

    private func done() {
        guard !newName.isEmpty, newName != path.last else { return }
        rename(newName)
        dismiss()
    }
}

