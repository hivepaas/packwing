import SwiftUI

/// A storage across the network the user added: its kind and settings. Its
/// secrets are in the keychain, under its id.
struct StorageEntry: Codable, Hashable, Identifiable {
    var id: String
    /// As `storageKinds` gives it: `s3`, `webdav`…
    var kind: String
    var values: [String: String]

    var kindRow: StorageKindRow? { storageKinds().first { $0.id == kind } }

    /// Where it is, as its row says it.
    var described: String { describeStorage(kind: kind, values: values) }
}

/// A storage open, as long as it is shown or an archive of it is: its
/// connection is kept for what follows.
final class StorageHandle: Hashable, @unchecked Sendable {
    let entry: StorageEntry
    let storage: RemoteStorage

    init(_ entry: StorageEntry, _ storage: RemoteStorage) {
        self.entry = entry
        self.storage = storage
    }

    /// Where `path` is, as a URL of no file: an archive opened from there is
    /// told by it, and no file is looked for beside it.
    func url(of path: [String]) -> URL {
        path.reduce(URL(string: "packwing-storage://\(entry.id)/")!) { $0.appending(path: $1) }
    }

    static func == (a: StorageHandle, b: StorageHandle) -> Bool { a === b }
    func hash(into h: inout Hasher) { h.combine(ObjectIdentifier(self)) }
}

/// A folder of a storage, as the navigation shows it.
struct StorageSpot: Hashable {
    let handle: StorageHandle
    /// Its path; empty for the top.
    let folder: [String]
}

extension AppModel {
    // MARK: The storages kept

    /// How many storages packwing Free keeps.
    static let freeStorages = 1

    private static let storagesKey = "storages"

    static func loadStorages() -> [StorageEntry] {
        guard let data = UserDefaults.standard.data(forKey: storagesKey) else { return [] }
        return (try? JSONDecoder().decode([StorageEntry].self, from: data)) ?? []
    }

    private func saveStorages() {
        UserDefaults.standard.set(try? JSONEncoder().encode(storages), forKey: Self.storagesKey)
    }

    /// Whether another storage can be added.
    var mayAddStorage: Bool { store.holds || storages.count < Self.freeStorages }

    /// Keeps `entry`, its `secrets` in the keychain.
    func add(_ entry: StorageEntry, secrets: [String: String]) throws {
        try keepStorageSecrets(kind: entry.kind, id: entry.id, secrets: secrets)
        storages.append(entry)
        saveStorages()
    }

    func remove(_ entry: StorageEntry) {
        forgetStorageSecrets(kind: entry.kind, id: entry.id)
        storages.removeAll { $0.id == entry.id }
        handles[entry.id] = nil
        saveStorages()
    }

    /// `entry` open, the one already open if it is.
    func handle(_ entry: StorageEntry) throws -> StorageHandle {
        if let open = handles[entry.id], open.entry == entry { return open }
        let storage = try RemoteStorage.open(kind: entry.kind, id: entry.id, values: entry.values, typed: nil)
        let open = StorageHandle(entry, storage)
        handles[entry.id] = open
        return open
    }

    // MARK: Its files

    /// Opens the archive `item` of `folder` of `handle`, where it is.
    func open(_ item: StoredItem, in folder: [String], of handle: StorageHandle) {
        let spot = OpenSpot.stored(handle, folder + [item.name], item.size)
        Task { await open(spot, secrets: Secrets()) }
    }

    /// Shows `item` of `folder` of `handle` in Quick Look, read whole first.
    func preview(_ item: StoredItem, in folder: [String], of handle: StorageHandle) {
        guard reading == nil else { return }
        guard item.size <= Thumbnails.most else {
            failure = tr("packwing cannot show this file.")
            return
        }
        reading = item.name
        let path = folder + [item.name]
        Task {
            defer { reading = nil }
            let result = await Task.detached(priority: .userInitiated) {
                Result { () throws -> URL in
                    let bytes = try handle.storage.read(path: path, most: Thumbnails.most)
                    let at = URL.temporaryDirectory.appending(path: "stored-\(UUID().uuidString)", directoryHint: .isDirectory)
                    try FileManager.default.createDirectory(at: at, withIntermediateDirectories: true)
                    let file = at.appending(path: item.name)
                    try bytes.write(to: file)
                    return file
                }
            }.value
            switch result {
            case .success(let file): previewing = file
            case .failure(let error): failure = (error as? MobileError).map(reason) ?? error.localizedDescription
            }
        }
    }
}

// MARK: The Archives tab's storages

/// The storages added, and Add storage.
struct StoragesSection: View {
    @Environment(AppModel.self) private var model
    @State private var adding = false
    @State private var limited = false
    @State private var removing: StorageEntry?

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(tr("Storages"))
                .font(.nunito(20, .bold, relativeTo: .title3))
                .padding(.horizontal, 4)
                .accessibilityAddTraits(.isHeader)
            VStack(spacing: 0) {
                ForEach(model.storages) { entry in
                    row(entry)
                    Divider().overlay(Tonal.field).padding(.leading, 72)
                }
                Button {
                    if model.mayAddStorage { adding = true } else { limited = true }
                } label: {
                    Label(tr("Add storage"), systemImage: "plus")
                        .font(.nunito(16, .semibold, relativeTo: .body))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.vertical, 16)
                        .padding(.horizontal, 18)
                        .contentShape(.rect)
                }
                .buttonStyle(.plain)
                .foregroundStyle(Tonal.accent)
                .accessibilityIdentifier("storage-new")
                // On the button, apart from the sheet: on an iPhone, an alert
                // beside a sheet that was shown no longer showed.
                .alert(tr("Add storage"), isPresented: $limited) {
                    // The plans are in Settings, with what the App Store asks
                    // to be said beside them.
                    if model.store.sold != false {
                        Button(tr("Settings")) { model.tab = .settings }
                    }
                    Button(tr("OK"), role: .cancel) {}
                } message: {
                    Text(tr("packwing Free keeps one storage across a network. Pro lifts this."))
                }
            }
            .background(Tonal.surface, in: .rect(cornerRadius: 26))
        }
        .sheet(isPresented: $adding) { AddStorageSheet() }
        .confirmationDialog(tr("Remove “{name}” from the list?", ["name": removing?.described ?? ""]),
                            isPresented: Binding { removing != nil } set: { if !$0 { removing = nil } },
                            titleVisibility: .visible, presenting: removing) { entry in
            Button(tr("Remove from this list"), role: .destructive) { model.remove(entry) }
        }
    }

    private func row(_ entry: StorageEntry) -> some View {
        Button {
            do {
                model.path.append(StorageSpot(handle: try model.handle(entry), folder: []))
            } catch {
                model.failure = (error as? MobileError).map(reason) ?? error.localizedDescription
            }
        } label: {
            HStack(spacing: 14) {
                Text(entry.kindRow?.tag ?? "")
                    .font(.nunito(13, .bold, relativeTo: .caption))
                    .foregroundStyle(Tint.blue.ink)
                    .frame(width: 42, height: 42)
                    .background(Tint.blue.ground, in: .rect(cornerRadius: 12))
                VStack(alignment: .leading, spacing: 2) {
                    Text(entry.described)
                        .font(.nunito(16, .semibold, relativeTo: .body))
                        .lineLimit(1)
                        .truncationMode(.middle)
                    Text(entry.kindRow?.name ?? "")
                        .font(.nunito(13, relativeTo: .footnote))
                        .foregroundStyle(Tonal.muted)
                }
                Spacer(minLength: 0)
                Image(systemName: "chevron.right")
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(Tonal.faint)
            }
            .foregroundStyle(Tonal.text)
            .padding(.vertical, 12)
            .padding(.leading, 16)
            .padding(.trailing, 14)
            .contentShape(.rect)
        }
        .buttonStyle(.plain)
        .accessibilityIdentifier("storage-row")
        .contextMenu {
            Button(tr("Remove from this list"), systemImage: "minus.circle", role: .destructive) { removing = entry }
        }
    }
}

// MARK: A storage's folder

struct StorageFolderView: View {
    @Environment(AppModel.self) private var model
    let spot: StorageSpot
    @State private var items: [StoredItem]?
    @State private var problem: String?
    @State private var renaming: String?
    @State private var deleting: [String]?
    @State private var naming = false
    @State private var adding: NewSource?
    @State private var sending: (names: [String], moving: Bool)?
    @State private var downloading: [String]?
    @AppStorage("storageHidden") private var showingHidden = false

    private var shown: [StoredItem] {
        (items ?? []).filter { showingHidden || !$0.hidden }
    }

    var body: some View {
        @Bindable var model = model
        List {
            if let problem {
                Section {
                    Label(problem, systemImage: "exclamationmark.triangle")
                        .foregroundStyle(Tonal.warnText)
                        .listRowBackground(Tonal.warnBg)
                    Button(tr("Try again")) { Task { await load() } }
                        .listRowBackground(Tonal.surface)
                }
            } else if items == nil {
                HStack(spacing: 12) {
                    ProgressView()
                    Text(tr("Checking…"))
                }
                .listRowBackground(Tonal.surface)
            } else {
                Section {
                    ForEach(shown, id: \.name) { item in
                        row(item)
                    }
                } footer: {
                    if shown.isEmpty { Text(tr("This folder is empty.")) }
                }
            }
        }
        .listStyle(.insetGrouped)
        .scrollContentBackground(.hidden)
        .background(Tonal.page)
        .navigationTitle(spot.folder.last ?? spot.handle.entry.described)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                // Up from the phone: Files, a folder, Photos.
                AddMenu(source: $adding) {
                    Image(systemName: "plus")
                }
                .accessibilityLabel(tr("Add files"))
            }
            ToolbarItem(placement: .topBarTrailing) {
                Menu {
                    Button(tr("New folder"), systemImage: "folder.badge.plus") { naming = true }
                    Toggle(tr("Show hidden files"), isOn: $showingHidden)
                } label: {
                    Image(systemName: "ellipsis")
                }
                .accessibilityLabel(tr("More"))
            }
        }
        .modifier(StorageChanges(spot: spot, renaming: $renaming, deleting: $deleting, naming: $naming,
                                 adding: $adding, sending: $sending, downloading: $downloading))
        .quickLookPreview($model.previewing)
        .overlay(alignment: .bottom) {
            if let name = model.reading {
                BusyBanner(title: tr("Opening"), name: name) {}
                    .padding(.bottom, 24)
            } else if let changing = model.changing {
                BusyBanner(title: changing.title, name: changing.name) { model.stopChanging() }
                    .padding(.bottom, 24)
            }
        }
        .refreshable { await load() }
        .task { if items == nil { await load() } }
        // Read again once something here, or anywhere, changed.
        .onChange(of: model.storagesChanged) { Task { await load() } }
    }

    private func row(_ item: StoredItem) -> some View {
        let kind = FileKind.of(name: item.name, folder: item.folder, link: item.link)
        let label = StoredLabel(item: item, kind: kind)
        return Group {
            if item.folder {
                NavigationLink(value: StorageSpot(handle: spot.handle, folder: spot.folder + [item.name])) { label }
            } else if item.link {
                label
            } else {
                Button {
                    if kind == .archive {
                        model.open(item, in: spot.folder, of: spot.handle)
                    } else {
                        model.preview(item, in: spot.folder, of: spot.handle)
                    }
                } label: {
                    label.contentShape(.rect)
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier("stored-file")
            }
        }
        .listRowBackground(Tonal.surface)
        .contextMenu {
            if !item.folder && !item.link {
                // packwing tells an archive by what is in it, whatever its name.
                Button(tr("Open"), systemImage: "archivebox") { model.open(item, in: spot.folder, of: spot.handle) }
                Button(tr("Preview"), systemImage: "eye") { model.preview(item, in: spot.folder, of: spot.handle) }
            }
            Button(tr("Download"), systemImage: "square.and.arrow.down") { downloading = [item.name] }
            Button(tr("Copy to…"), systemImage: "doc.on.doc") { sending = ([item.name], false) }
            Button(tr("Move to…"), systemImage: "folder") { sending = ([item.name], true) }
            Button(tr("Rename"), systemImage: "pencil") { renaming = item.name }
            Button(tr("Delete…"), systemImage: "trash", role: .destructive) { deleting = [item.name] }
        }
    }

    private func load() async {
        problem = nil
        let storage = spot.handle.storage
        let folder = spot.folder
        let result = await Task.detached(priority: .userInitiated) {
            Result { try storage.list(folder: folder, job: Job.quiet()) }
        }.value
        switch result {
        case .success(let now): items = now
        case .failure(let error): problem = (error as? MobileError).map(reason) ?? error.localizedDescription
        }
    }
}

private struct StoredLabel: View {
    let item: StoredItem
    let kind: FileKind

    var body: some View {
        HStack(spacing: 14) {
            KindTile(name: item.name, kind: kind, size: 38)
            VStack(alignment: .leading, spacing: 1) {
                Text(item.name)
                    .font(.nunito(16, .semibold, relativeTo: .body))
                    .foregroundStyle(Tonal.text)
                    .lineLimit(1)
                    .truncationMode(.middle)
                Text(meta)
                    .font(.nunito(13, relativeTo: .footnote))
                    .foregroundStyle(Tonal.muted)
                    .lineLimit(1)
            }
            Spacer(minLength: 0)
        }
        .padding(.vertical, 2)
    }

    private var meta: String {
        var parts: [String] = []
        if !item.folder { parts.append(sizeText(item.size)) }
        if let modified = item.modified {
            parts.append(Date(timeIntervalSince1970: TimeInterval(modified))
                .formatted(Date.FormatStyle(date: .abbreviated, time: .omitted).locale(appLocale)))
        }
        return parts.joined(separator: " · ")
    }
}

// MARK: Adding one

/// A storage's kind and settings, tried before it is kept.
struct AddStorageSheet: View {
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    private let kinds = storageKinds()
    @State private var kindId = ""
    @State private var values: [String: String] = [:]
    @State private var checking = false
    @State private var problem: String?

    private var kind: StorageKindRow? { kinds.first { $0.id == kindId } ?? kinds.first }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    Picker(tr("Kind"), selection: $kindId) {
                        ForEach(kinds, id: \.id) { Text($0.name).tag($0.id) }
                    }
                    .pickerStyle(.segmented)
                    .listRowBackground(Color.clear)
                    .listRowInsets(EdgeInsets())
                }
                if let kind {
                    ForEach(kind.fields, id: \.key) { field in
                        Section {
                            Group {
                                // Not a login, to iOS: it would offer to save the
                                // password to Passwords as the sheet goes, over packwing,
                                // which keeps it in its own keychain.
                                if field.secret {
                                    SecureField(field.hint, text: binding(field.key))
                                        .textContentType(.oneTimeCode)
                                } else {
                                    TextField(field.hint, text: binding(field.key))
                                        .textContentType(.oneTimeCode)
                                }
                            }
                            .autocorrectionDisabled()
                            .textInputAutocapitalization(.never)
                            .accessibilityIdentifier("storage-\(field.key)")
                        } header: {
                            Text(field.label)
                        }
                        .listRowBackground(Tonal.surface)
                    }
                }
                if let keyValue = kind?.keyValue {
                    SshKeyPicker(selection: binding(keyValue))
                }
                if let problem {
                    Section {
                        Label(problem, systemImage: "exclamationmark.triangle")
                            .foregroundStyle(Tonal.warnText)
                            .listRowBackground(Tonal.warnBg)
                    }
                }
            }
            .scrollContentBackground(.hidden)
            .background(Tonal.page)
            .navigationTitle(tr("Add storage"))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(role: .close) { dismiss() }
                }
            }
            .safeAreaInset(edge: .bottom) {
                Button { add() } label: {
                    HStack(spacing: 10) {
                        if checking { ProgressView().tint(Tonal.onAccent) }
                        Text(checking ? tr("Checking…") : tr("Add storage"))
                            .font(.nunito(17, .bold, relativeTo: .headline))
                    }
                    .frame(maxWidth: .infinity, minHeight: 52)
                }
                .buttonStyle(.glassProminent)
                .disabled(checking)
                .accessibilityIdentifier("storage-add")
                .padding(.horizontal, 20)
                .padding(.bottom, 8)
            }
            .onAppear { if kindId.isEmpty { kindId = kinds.first?.id ?? "" } }
        }
        .presentationDetents([.large])
        .presentationSizing(.page)
    }

    private func binding(_ key: String) -> Binding<String> {
        Binding { values[key] ?? "" } set: { values[key] = $0 }
    }

    /// Tries it, and keeps it if it answers: its settings, and its secrets in
    /// the keychain.
    private func add() {
        guard let kind else { return }
        let secretKeys = Set(kind.fields.filter(\.secret).map(\.key))
        let settings = values.filter { key, value in
            !secretKeys.contains(key) && (kind.fields.contains { $0.key == key } || key == kind.keyValue)
                && !value.isEmpty
        }.mapValues { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
        let secrets = values.filter { secretKeys.contains($0.key) }
        let entry = StorageEntry(id: UUID().uuidString, kind: kind.id, values: settings)
        checking = true
        problem = nil
        Task {
            let result = await Task.detached(priority: .userInitiated) {
                Result {
                    let storage = try RemoteStorage.open(kind: entry.kind, id: entry.id, values: entry.values,
                                                         typed: secrets)
                    _ = try storage.list(folder: [], job: Job.quiet())
                }
            }.value
            checking = false
            do {
                try result.get()
                try model.add(entry, secrets: secrets)
                // Kept: no password is left in the form as it goes.
                for key in secretKeys { values[key] = nil }
                dismiss()
            } catch {
                problem = (error as? MobileError).map(reason) ?? error.localizedDescription
            }
        }
    }
}
