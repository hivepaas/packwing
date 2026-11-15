import PhotosUI
import SwiftUI
import UniformTypeIdentifiers

/// Where new items are chosen from.
enum NewSource: Identifiable {
    case files, folder, photos
    var id: Self { self }
}

/// The choices of where to add from: Files, a folder, Photos.
struct AddMenu<Label: View>: View {
    @Binding var source: NewSource?
    @ViewBuilder let label: () -> Label

    var body: some View {
        Menu {
            Button(tr("Files"), systemImage: "doc") { source = .files }
            Button(tr("A folder"), systemImage: "folder") { source = .folder }
            Button(tr("Photos"), systemImage: "photo.on.rectangle") { source = .photos }
        } label: {
            label()
        }
    }
}

extension View {
    /// Shows the picker for `source`, and hands on what is chosen in it.
    func choosingNewItems(
        _ source: Binding<NewSource?>,
        chosen: @escaping ([NewItem]) -> Void,
        photos: @escaping ([PhotosPickerItem]) -> Void
    ) -> some View {
        modifier(NewItemPickers(source: source, chosen: chosen, photos: photos))
    }
}

private struct NewItemPickers: ViewModifier {
    @Binding var source: NewSource?
    let chosen: ([NewItem]) -> Void
    let photos: ([PhotosPickerItem]) -> Void
    @State private var picked: [PhotosPickerItem] = []

    func body(content: Content) -> some View {
        content
            .fileImporter(
                isPresented: Binding(get: { source == .files || source == .folder },
                                     set: { if !$0 { source = nil } }),
                allowedContentTypes: source == .folder ? [.folder] : [.item],
                allowsMultipleSelection: true
            ) { result in
                if case .success(let urls) = result { chosen(urls.map(NewItem.init(chosen:))) }
            }
            .photosPicker(
                isPresented: Binding(get: { source == .photos }, set: { if !$0 { source = nil } }),
                selection: $picked,
                matching: .any(of: [.images, .videos]),
                // Their files as they are: HEIC stays HEIC.
                preferredItemEncoding: .current
            )
            .onChange(of: picked) { _, now in
                guard !now.isEmpty else { return }
                photos(now)
                picked = []
            }
    }
}

/// What New archive asks before it starts: what goes in, its name, and how.
struct CreateSheet: View {
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    let draft: NewArchiveDraft
    @AppStorage("newFormat") private var formatKey = ArchiveFormat.zip.key
    @AppStorage("newCompression") private var levelKey = CompressionLevel.medium.key
    @State private var name = ""
    /// Whether the name is the person's own, rather than the one suggested.
    @State private var named = false
    @State private var locked = false
    @State private var password = ""
    @State private var showingPassword = false
    @State private var hideNames = false
    /// The bytes of each part, if it is cut in parts.
    @State private var partSize: UInt64?
    @State private var adding: NewSource?
    @State private var choosingFolder = false
    /// A storage's folder to make it in, in place of one of Files.
    @State private var toStorage: StorageSpot?
    @State private var pickingStorage = false

    var body: some View {
        // Read, so that a folder chosen draws it again.
        let _ = model.foldersChanged
        let folder = model.createFolder
        let format = chosenFormat
        NavigationStack {
            Form {
                Section {
                    NavigationLink {
                        NewItemsList(draft: draft)
                    } label: {
                        ItemsSummary(draft: draft)
                    }
                    AddMenu(source: $adding) {
                        Label(tr("Add more"), systemImage: "plus")
                    }
                }
                .listRowBackground(Tonal.surface)

                Section(tr("Name")) {
                    HStack(spacing: 4) {
                        TextField(tr("Name"), text: $name)
                            .font(.nunito(17, relativeTo: .body))
                            .textInputAutocapitalization(.never)
                            .autocorrectionDisabled()
                            .submitLabel(.done)
                            .onChange(of: name) { _, now in
                                if now != draft.suggestedName { named = true }
                            }
                            .accessibilityIdentifier("new-name")
                        Text(format.ext)
                            .foregroundStyle(Tonal.muted)
                    }
                }
                .listRowBackground(Tonal.surface)

                Section {
                    // ZIP and 7z first, as most people want one; then the desktop's others.
                    Picker(tr("Format"), selection: Binding { format.key } set: { formatKey = $0 }) {
                        ForEach(ArchiveFormat.offered(oneFile: oneFile), id: \.key) { Text($0.label).tag($0.key) }
                    }
                    .accessibilityIdentifier("new-format")
                } footer: {
                    if format == .zip || format == .sevenZip {
                        Text(format == .zip
                             ? tr("Opens everywhere. With a password, AES-256: Windows’ own Explorer can’t open it, 7-Zip and packwing can.")
                             : tr("Smaller. Opens with packwing, 7-Zip and most archivers."))
                            .font(.nunito(13, relativeTo: .footnote))
                    }
                }
                .listRowBackground(Tonal.surface)

                Section {
                    Picker(tr("Compression"), selection: Binding { level.key } set: { levelKey = $0 }) {
                        ForEach(CompressionLevel.all.filter { $0 != .store || format.stores }, id: \.key) {
                            Text($0.label).tag($0.key)
                        }
                    }
                    Picker(tr("Split into parts"), selection: $partSize) {
                        Text(tr("Don’t split")).tag(UInt64?.none)
                        Text(tr("25 MB (email)")).tag(UInt64?.some(25_000_000))
                        Text(tr("4 GB (USB stick)")).tag(UInt64?.some(4_000_000_000))
                    }
                    Toggle(format.usesAge ? tr("Encrypt with age") : tr("Protect with a password"), isOn: $locked.animation())
                    if locked {
                        HStack(spacing: 8) {
                            Group {
                                if showingPassword {
                                    TextField(format.usesAge ? tr("Passphrase") : tr("Password"), text: $password)
                                } else {
                                    SecureField(format.usesAge ? tr("Passphrase") : tr("Password"), text: $password)
                                }
                            }
                            // Not an account's password: as one (.newPassword, or
                            // none, by the name above it), iOS puts in a strong
                            // password of its own, which no one knows, and asks
                            // to keep it in Passwords as the name's.
                            .textContentType(.oneTimeCode)
                            .textInputAutocapitalization(.never)
                            .autocorrectionDisabled()
                            .accessibilityIdentifier("new-password")
                            Button {
                                showingPassword.toggle()
                            } label: {
                                Image(systemName: showingPassword ? "eye.slash" : "eye")
                                    .foregroundStyle(Tonal.muted)
                            }
                            .buttonStyle(.borderless)
                            .accessibilityLabel(showingPassword ? tr("Hide the password") : tr("Show the password"))
                        }
                        // age hides it all, names too: only ZIP and 7z have a choice.
                        if !format.usesAge {
                            Toggle(isOn: $hideNames) {
                                VStack(alignment: .leading, spacing: 1) {
                                    Text(tr("Hide the file names too"))
                                    Text(format == .sevenZip
                                         ? tr("No one sees what is inside without the password.")
                                         : tr("7z only: ZIP always shows its names."))
                                        .font(.nunito(12, relativeTo: .caption))
                                        .foregroundStyle(Tonal.muted)
                                }
                            }
                            .disabled(format == .zip)
                        }
                    }
                }
                .listRowBackground(Tonal.surface)

                Section(tr("Save to")) {
                    HStack(spacing: 12) {
                        Image(systemName: "folder")
                            .foregroundStyle(Tonal.accent)
                            .font(.system(size: 20))
                        Text(toStorage.map { $0.handle.storage.showPath(path: $0.folder) } ?? model.shown(folder))
                            .font(.nunito(15, .semibold, relativeTo: .subheadline))
                            .lineLimit(2)
                            .truncationMode(.middle)
                        Spacer(minLength: 8)
                        if model.storages.isEmpty {
                            Button(tr("Change")) { choosingFolder = true }
                                .buttonStyle(.bordered)
                        } else {
                            // A folder of Files, or one of a storage.
                            Menu(tr("Change")) {
                                Button(tr("Files"), systemImage: "folder") { choosingFolder = true }
                                Button(tr("Storages"), systemImage: "externaldrive.connected.to.line.below") {
                                    pickingStorage = true
                                }
                            }
                            .buttonStyle(.bordered)
                        }
                    }
                }
                .listRowBackground(Tonal.surface)

                if let problem = draft.problem {
                    Section {
                        Label(problem, systemImage: "exclamationmark.triangle")
                            .font(.nunito(14, relativeTo: .subheadline))
                            .foregroundStyle(Tonal.badText)
                    }
                    .listRowBackground(Tonal.badBg)
                }
            }
            .scrollContentBackground(.hidden)
            .background(Tonal.page)
            .navigationTitle(tr("New archive"))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(role: .close) { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(role: .confirm) { start(folder) }
                        .disabled(!ready)
                        .accessibilityLabel(tr("Create"))
                }
            }
            .safeAreaInset(edge: .bottom) {
                Button { start(folder) } label: {
                    Group {
                        if draft.fetching > 0 {
                            HStack(spacing: 10) {
                                ProgressView()
                                Text(trN(draft.fetching, "Getting {n} photo", "Getting {n} photos"))
                            }
                        } else {
                            Text(tr("Create"))
                        }
                    }
                    .font(.nunito(17, .bold, relativeTo: .headline))
                    .frame(maxWidth: .infinity, minHeight: 52)
                }
                .buttonStyle(.glassProminent)
                .disabled(!ready)
                .padding(.horizontal, 20)
                .padding(.bottom, 8)
            }
            .choosingNewItems($adding, chosen: { draft.add($0) }, photos: { draft.add($0) })
            .fileImporter(isPresented: $choosingFolder, allowedContentTypes: [.folder]) { result in
                if case .success(let url) = result {
                    model.createIn(url)
                    toStorage = nil
                }
            }
            .sheet(isPresented: $pickingStorage) {
                StorageFolderPicker(title: tr("Save to"), confirm: tr("Save here")) { toStorage = $0 }
            }
        }
        .presentationDetents([.large])
        // On an iPad, a page: all of it shows, the keyboard up or not.
        .presentationSizing(.page)
        .onAppear { name = draft.suggestedName }
        .onChange(of: draft.suggestedName) { _, now in
            if !named { name = now }
        }
        .onChange(of: formatKey) { _, now in
            if now == ArchiveFormat.zip.key { hideNames = false }
        }
    }

    /// The name as a file can be called: no folders in it, nor its extension twice.
    private var fileName: String {
        var said = name.trimmingCharacters(in: .whitespacesAndNewlines)
            .replacingOccurrences(of: "/", with: "-")
            .replacingOccurrences(of: ":", with: "-")
        let ext = chosenFormat.ext
        if said.lowercased().hasSuffix(ext) { said.removeLast(ext.count) }
        return said.trimmingCharacters(in: .whitespaces)
    }

    /// One file by itself, not a folder: the formats for one file are offered too.
    private var oneFile: Bool {
        draft.items.count == 1 && !draft.items[0].folder
    }

    /// The format chosen, if it is one for what is chosen; ZIP otherwise.
    private var chosenFormat: ArchiveFormat {
        let chosen = ArchiveFormat(key: formatKey)
        return chosen.oneFile && !oneFile ? .zip : chosen
    }

    /// The level chosen; Store is ZIP's and 7z's, Lowest for the others.
    private var level: CompressionLevel {
        let chosen = CompressionLevel(key: levelKey)
        return chosen == .store && !chosenFormat.stores ? .lowest : chosen
    }

    private var ready: Bool {
        !draft.items.isEmpty && draft.fetching == 0 && !fileName.isEmpty && !(locked && password.isEmpty)
    }

    private func start(_ folder: URL) {
        guard ready else { return }
        let format = chosenFormat
        let how = NewArchiveHow(
            format: format,
            compression: level,
            password: locked ? password : nil,
            hideNames: locked && hideNames && format == .sevenZip,
            partSize: partSize)
        let name = fileName
        if let toStorage {
            model.afterSheet = { [model, draft] in
                model.create(draft, named: name + format.ext, how: how, on: toStorage.handle, in: toStorage.folder)
            }
        } else {
            model.afterSheet = { [model, draft] in model.create(draft, named: name, how: how, in: folder) }
        }
        dismiss()
    }
}

/// What goes in, in short: the first few as tiles, how many and how big.
private struct ItemsSummary: View {
    let draft: NewArchiveDraft

    var body: some View {
        HStack(spacing: 12) {
            HStack(spacing: 4) {
                ForEach(draft.items.prefix(3)) { item in
                    KindTile(name: item.name, kind: .of(name: item.name, folder: item.folder), size: 36)
                }
            }
            VStack(alignment: .leading, spacing: 1) {
                Text(draft.summary)
                    .font(.nunito(15, .semibold, relativeTo: .subheadline))
                if let size = draft.size {
                    Text(sizeText(size))
                        .font(.nunito(13, relativeTo: .footnote))
                        .foregroundStyle(Tonal.muted)
                }
            }
        }
        .accessibilityElement(children: .combine)
    }
}

/// All that goes in, one by one: a swipe takes one out.
private struct NewItemsList: View {
    let draft: NewArchiveDraft

    var body: some View {
        List {
            ForEach(draft.items) { item in
                HStack(spacing: 14) {
                    KindTile(name: item.name, kind: .of(name: item.name, folder: item.folder))
                    Text(item.name)
                        .font(.nunito(16, relativeTo: .body))
                        .lineLimit(1)
                        .truncationMode(.middle)
                }
            }
            .onDelete { draft.remove(at: $0) }
            .listRowBackground(Tonal.surface)
        }
        .scrollContentBackground(.hidden)
        .background(Tonal.page)
        .navigationTitle(draft.summary)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar { EditButton() }
    }
}
