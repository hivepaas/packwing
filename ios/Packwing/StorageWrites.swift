import PhotosUI
import SwiftUI

/// What is changed on a storage from the phone: folders made, files renamed,
/// deleted, copied and moved (between storages, and with Files), sent up from
/// the phone, and new archives made there. As on the desktop; a storage across
/// the network has no Trash, so a delete is for good.
extension AppModel {
    /// Does `work` on a storage, showing `title` and `name` while it does, with
    /// Stop; then each storage's folder shown reads itself again.
    func onStorage(_ title: String, _ name: String, work: @escaping @Sendable (Job) throws -> Void) {
        guard changing == nil else { return }
        let job = Job.quiet()
        changeJob = job
        changing = (title, name)
        Task {
            defer {
                changing = nil
                changeJob = nil
                storagesChanged += 1
            }
            let result = await Task.detached(priority: .userInitiated) { Result { try work(job) } }.value
            if case .failure(let error) = result {
                if case .Cancelled = error as? MobileError { return }
                failure = (error as? MobileError).map(reason) ?? error.localizedDescription
            }
        }
    }

    /// Sends `items` (of Files, or Photos' copies) up into `folder` of `handle`.
    /// They are kept until it is done: they hold access to the files.
    func upload(_ items: [NewItem], to folder: [String], of handle: StorageHandle) {
        guard let first = items.first else { return }
        onStorage(tr("Uploading…"), items.count == 1 ? first.name : tr("{many} items", ["many": "\(items.count)"])) { job in
            try withExtendedLifetime(items) {
                for item in items {
                    let scoped = item.url.startAccessingSecurityScopedResource()
                    defer { if scoped { item.url.stopAccessingSecurityScopedResource() } }
                    _ = try transferItems(
                        from: RemoteStorage.local(path: item.url.deletingLastPathComponent().path(percentEncoded: false)),
                        fromFolder: [], names: [item.name],
                        to: handle.storage, toFolder: folder, moving: false, onExisting: .stop, job: job)
                }
            }
        }
    }

    /// Copies, or moves, `names` of `folder` of `handle` into `to`.
    func transfer(_ names: [String], in folder: [String], of handle: StorageHandle, to: StorageSpot, moving: Bool) {
        onStorage(moving ? tr("Moving…") : tr("Copying…"), names.count == 1 ? names[0] : tr("{many} items", ["many": "\(names.count)"])) { job in
            _ = try transferItems(from: handle.storage, fromFolder: folder, names: names,
                                  to: to.handle.storage, toFolder: to.folder, moving: moving,
                                  onExisting: .stop, job: job)
        }
    }

    /// Copies `names` of `folder` of `handle` down into `local`, a folder of Files.
    func download(_ names: [String], in folder: [String], of handle: StorageHandle, into local: URL) {
        onStorage(tr("Download"), names.count == 1 ? names[0] : tr("{many} items", ["many": "\(names.count)"])) { job in
            let scoped = local.startAccessingSecurityScopedResource()
            defer { if scoped { local.stopAccessingSecurityScopedResource() } }
            _ = try transferItems(from: handle.storage, fromFolder: folder, names: names,
                                  to: RemoteStorage.local(path: local.path(percentEncoded: false)), toFolder: [],
                                  moving: false, onExisting: .stop, job: job)
        }
    }

    /// Makes an archive of `draft`'s items, `named` (with its extension), in
    /// `folder` of `handle`, showing how far it is.
    func create(_ draft: NewArchiveDraft, named file: String, how: NewArchiveHow, on handle: StorageHandle, in folder: [String]) {
        if let job, job.state == .running {
            showingJob = true
            return
        }
        let running = RunningJob(title: tr("Compressing"), name: file)
        job = running
        showingJob = true
        running.continueInBackground(subtitle: file)
        let items = draft.items
        Task {
            let paths = items.map { $0.url.path(percentEncoded: false) }
            let job = running.job
            let result = await Task.detached(priority: .userInitiated) {
                Result { try handle.storage.createArchive(paths: paths, folder: folder, name: file, how: how, job: job) }
            }.value
            withExtendedLifetime(items) {}
            storagesChanged += 1
            switch result {
            case .success(let made):
                running.finish(.done(detail: tr("{name} in {folder}",
                                                ["name": made, "folder": handle.storage.showPath(path: folder)])))
            case .failure(let error):
                running.finish(Self.ended(by: error))
            }
        }
    }
}

/// A folder of a storage to copy or move to, or make an archive in: the
/// storages added, then their folders.
struct StorageFolderPicker: View {
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    let title: String
    let confirm: String
    let chosen: (StorageSpot) -> Void
    @State private var path: [StorageSpot] = []

    var body: some View {
        NavigationStack(path: $path) {
            List {
                ForEach(model.storages) { entry in
                    Button {
                        if let handle = try? model.handle(entry) {
                            path.append(StorageSpot(handle: handle, folder: []))
                        }
                    } label: {
                        Label(entry.described, systemImage: "externaldrive.connected.to.line.below")
                            .lineLimit(1)
                            .truncationMode(.middle)
                    }
                }
            }
            .scrollContentBackground(.hidden)
            .background(Tonal.page)
            .navigationTitle(title)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(role: .close) { dismiss() }
                }
            }
            .navigationDestination(for: StorageSpot.self) { spot in
                PickerFolder(spot: spot, confirm: confirm) { chosen($0); dismiss() }
            }
        }
    }
}

/// A folder of a storage, in the picker: its folders, and here.
private struct PickerFolder: View {
    let spot: StorageSpot
    let confirm: String
    let here: (StorageSpot) -> Void
    @State private var folders: [StoredItem]?
    @State private var problem: String?

    var body: some View {
        List {
            if let problem {
                Label(problem, systemImage: "exclamationmark.triangle").foregroundStyle(Tonal.warnText)
            } else if let folders {
                ForEach(folders, id: \.name) { folder in
                    NavigationLink(value: StorageSpot(handle: spot.handle, folder: spot.folder + [folder.name])) {
                        Label(folder.name, systemImage: "folder")
                    }
                }
            } else {
                ProgressView()
            }
        }
        .scrollContentBackground(.hidden)
        .background(Tonal.page)
        .navigationTitle(spot.folder.last ?? spot.handle.entry.described)
        .navigationBarTitleDisplayMode(.inline)
        .safeAreaInset(edge: .bottom) {
            Button { here(spot) } label: {
                Text(confirm)
                    .font(.nunito(17, .bold, relativeTo: .headline))
                    .frame(maxWidth: .infinity, minHeight: 52)
            }
            .buttonStyle(.glassProminent)
            .padding(.horizontal, 20)
            .padding(.bottom, 8)
            .accessibilityIdentifier("picker-here")
        }
        .task {
            let storage = spot.handle.storage
            let folder = spot.folder
            let result = await Task.detached { Result { try storage.list(folder: folder, job: Job.quiet()) } }.value
            switch result {
            case .success(let items): folders = items.filter { $0.folder && !$0.hidden }
            case .failure(let error): problem = (error as? MobileError).map(reason) ?? error.localizedDescription
            }
        }
    }
}

/// What a storage's folder is asked to change, as its screen asks it.
struct StorageChanges: ViewModifier {
    @Environment(AppModel.self) private var model
    let spot: StorageSpot
    @Binding var renaming: String?
    @Binding var deleting: [String]?
    @Binding var naming: Bool
    @Binding var adding: NewSource?
    /// Copied (false) or moved (true), and what.
    @Binding var sending: (names: [String], moving: Bool)?
    @Binding var downloading: [String]?

    func body(content: Content) -> some View {
        content
            .sheet(item: Binding { renaming.map(Named.init) } set: { renaming = $0?.name }) { what in
                NameSheet(title: tr("Rename {what}", ["what": what.name]), current: what.name, confirm: tr("Rename")) { name in
                    model.onStorage(tr("Renaming…"), what.name) { _ in
                        try spot.handle.storage.rename(folder: spot.folder, from: what.name, to: name)
                    }
                }
            }
            .sheet(isPresented: $naming) {
                NameSheet(title: tr("New folder"), current: nil, confirm: tr("Create")) { name in
                    model.onStorage(tr("New folder"), name) { _ in
                        try spot.handle.storage.makeFolder(folder: spot.folder, name: name)
                    }
                }
            }
            .confirmationDialog(deleteTitle, isPresented: Binding { deleting != nil } set: { if !$0 { deleting = nil } },
                                titleVisibility: .visible, presenting: deleting) { names in
                Button(tr("Delete"), role: .destructive) {
                    model.onStorage(tr("Deleting…"), names.count == 1 ? names[0] : tr("{many} items", ["many": "\(names.count)"])) { _ in
                        try spot.handle.storage.delete(folder: spot.folder, names: names)
                    }
                }
            } message: { _ in
                Text(tr("This cannot be undone from packwing."))
            }
            .sheet(item: Binding { sending.map { Sending(names: $0.names, moving: $0.moving) } } set: { if $0 == nil { sending = nil } }) { what in
                StorageFolderPicker(title: what.moving ? tr("Move to…") : tr("Copy to…"),
                                    confirm: what.moving ? tr("Move here") : tr("Copy here")) { to in
                    model.transfer(what.names, in: spot.folder, of: spot.handle, to: to, moving: what.moving)
                }
            }
            .fileImporter(isPresented: Binding { downloading != nil } set: { if !$0 { downloading = nil } },
                          allowedContentTypes: [.folder]) { result in
                if case .success(let folder) = result, let names = downloading {
                    model.download(names, in: spot.folder, of: spot.handle, into: folder)
                }
            }
            .choosingNewItems($adding) { items in
                model.upload(items, to: spot.folder, of: spot.handle)
            } photos: { picked in
                Task { model.upload(await photoItems(picked), to: spot.folder, of: spot.handle) }
            }
    }

    private var deleteTitle: String {
        guard let deleting else { return "" }
        let what = deleting.count == 1 ? deleting[0] : tr("{many} items", ["many": "\(deleting.count)"])
        return tr("Delete {what}?", ["what": what])
    }
}

private struct Named: Identifiable {
    let name: String
    var id: String { name }
}

private struct Sending: Identifiable {
    let names: [String]
    let moving: Bool
    let id = UUID()
}
