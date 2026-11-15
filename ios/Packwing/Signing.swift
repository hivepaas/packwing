import SwiftUI
import UniformTypeIdentifiers

// Signing files, as the desktop signs them, with keys kept in this device's
// keychain: given as a key file (an SSH Ed25519 key, or one packwing made), or
// made here. The signature is `name.sig`, beside the file where packwing may
// write there; elsewhere it is saved where the user says. Signing takes Pro, as
// on the desktop; checking a signature is free.

/// Bytes saved as a file where the user says: a signature, a key's copy.
struct BytesDocument: FileDocument {
    static let readableContentTypes: [UTType] = [.data]
    let bytes: Data

    init(_ bytes: Data) { self.bytes = bytes }

    init(configuration: ReadConfiguration) throws {
        bytes = configuration.file.regularFileContents ?? Data()
    }

    func fileWrapper(configuration: WriteConfiguration) throws -> FileWrapper {
        FileWrapper(regularFileWithContents: bytes)
    }
}

extension SigningKeyRow: Identifiable {}

/// A file to save, and the name it is offered under.
struct FileToSave: Identifiable {
    let id = UUID()
    let document: BytesDocument
    let name: String
}

/// Settings' Signing keys: those kept, made here or given; the public half of
/// each to share, a copy of one made here to save, and one removed.
struct SigningKeysSection: View {
    @State private var keys: [SigningKeyRow] = (try? signingKeys()) ?? []
    @State private var making = false
    @State private var importing = false
    @State private var copying: SigningKeyRow?
    @State private var removing: SigningKeyRow?
    @State private var problem: String?

    var body: some View {
        Section {
            ForEach(keys, id: \.id) { key in
                SigningKeyLabel(key: key)
                    .contextMenu {
                        ShareLink(item: key.public + "\n",
                                  preview: SharePreview(key.name + ".pub")) {
                            Label(tr("Share the public key"), systemImage: "square.and.arrow.up")
                        }
                        Button(tr("Copy"), systemImage: "doc.on.doc") { UIPasteboard.general.string = key.public }
                        if key.madeHere {
                            Button(tr("Save a copy…"), systemImage: "square.and.arrow.down") { copying = key }
                        }
                        Button(tr("Remove"), systemImage: "trash", role: .destructive) { removing = key }
                    }
                    .swipeActions {
                        Button(tr("Remove"), role: .destructive) { removing = key }
                    }
            }
            Button(tr("New key…"), systemImage: "plus") { making = true }
                .accessibilityIdentifier("signing-new")
            Button(tr("Add a key file…"), systemImage: "key") { importing = true }
            if let problem {
                Label(problem, systemImage: "exclamationmark.triangle")
                    .foregroundStyle(Tonal.warnText)
            }
        } header: {
            Text(tr("Signing keys"))
        } footer: {
            Text(tr("Kept in this device’s keychain. Anyone can check what a key signs with its public half."))
        }
        .listRowBackground(Tonal.surface)
        .sheet(isPresented: $making) {
            NameSheet(title: tr("New key…"), current: nil, confirm: tr("Make the key…")) { name in
                do {
                    _ = try makeSigningKey(name: name)
                    problem = nil
                } catch {
                    problem = (error as? MobileError).map(reason) ?? error.localizedDescription
                }
                refresh()
            }
        }
        .sheet(isPresented: $importing) {
            ImportKeySheet(keep: { name, text, passphrase in
                try importSigningKey(name: name, contents: Data(text.utf8), passphrase: passphrase).id
            }) { _ in refresh() }
        }
        .sheet(item: $copying) { key in
            SaveKeyCopySheet(key: key)
        }
        .confirmationDialog(tr("Remove “{name}”?", ["name": removing?.name ?? ""]),
                            isPresented: Binding { removing != nil } set: { if !$0 { removing = nil } },
                            titleVisibility: .visible, presenting: removing) { key in
            Button(tr("Remove"), role: .destructive) {
                try? removeSigningKey(id: key.id)
                refresh()
            }
        } message: { key in
            if key.madeHere {
                Text(tr("This key was made here, and this device keeps the only copy of it unless one was saved. What it signed can still be checked, but nothing more can be signed with it."))
            }
        }
        .onAppear(perform: refresh)
    }

    private func refresh() {
        keys = (try? signingKeys()) ?? keys
    }
}

/// A signing key in a list: its name, and its fingerprint.
private struct SigningKeyLabel: View {
    let key: SigningKeyRow

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(key.name).font(.nunito(16, .semibold, relativeTo: .body))
            Text(key.fingerprint)
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(Tonal.muted)
                .lineLimit(1)
                .truncationMode(.middle)
        }
        .accessibilityElement(children: .combine)
        .accessibilityIdentifier("signing-key")
    }
}

/// A copy of a key made here, encrypted with a passphrase typed twice, saved
/// where the user says: the desktop and `pkw` sign with it too.
private struct SaveKeyCopySheet: View {
    @Environment(\.dismiss) private var dismiss
    let key: SigningKeyRow
    @State private var passphrase = ""
    @State private var again = ""
    @State private var saving: FileToSave?
    @State private var problem: String?

    private var ready: Bool { !passphrase.isEmpty && passphrase == again }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    SecureField(tr("Passphrase"), text: $passphrase)
                        .textContentType(.oneTimeCode)
                    SecureField(tr("Again"), text: $again)
                        .textContentType(.oneTimeCode)
                } footer: {
                    if !again.isEmpty && passphrase != again {
                        Text(tr("The two passphrases are not the same.")).foregroundStyle(Tonal.warnText)
                    } else {
                        Text(tr("The copy is encrypted with this passphrase. packwing on a computer signs with it too."))
                    }
                }
                .listRowBackground(Tonal.surface)
                if let problem {
                    Section {
                        Label(problem, systemImage: "exclamationmark.triangle").foregroundStyle(Tonal.warnText)
                    }
                    .listRowBackground(Tonal.warnBg)
                }
            }
            .scrollContentBackground(.hidden)
            .background(Tonal.page)
            .navigationTitle(tr("Save a copy…"))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(role: .close) { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(role: .confirm, action: save)
                        .disabled(!ready)
                        .accessibilityLabel(tr("Save a copy…"))
                }
            }
            .fileExporter(isPresented: Binding { saving != nil } set: { if !$0 { saving = nil } },
                          document: saving?.document, contentType: .data,
                          defaultFilename: saving?.name) { result in
                if case .success = result { dismiss() }
            }
        }
        .presentationDetents([.medium])
    }

    private func save() {
        do {
            let bytes = try exportSigningKey(id: key.id, passphrase: passphrase)
            saving = FileToSave(document: BytesDocument(bytes), name: key.name)
            passphrase = ""
            again = ""
        } catch {
            problem = (error as? MobileError).map(reason) ?? error.localizedDescription
        }
    }
}

/// Signs an archive's file with a key kept: its signature beside it, where
/// packwing may write there (a storage, packwing's own folder), or saved where
/// the user says.
struct SignSheet: View {
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    let archive: OpenedArchive
    /// The key signed with last, to start with.
    @AppStorage("signingKey") private var chosen = ""
    @State private var keys: [SigningKeyRow] = (try? signingKeys()) ?? []
    @State private var making = false
    @State private var importing = false
    @State private var state: SignState = .ready
    @State private var saving: FileToSave?

    private enum SignState {
        case ready
        case signing(Job)
        /// Signed: the signature is where it says.
        case done(String)
        case failed(String)
    }

    private var name: String { archive.info.name }

    private var key: SigningKeyRow? {
        keys.first { $0.id == chosen } ?? keys.first
    }

    private var signing: Bool {
        if case .signing = state { return true }
        return false
    }

    var body: some View {
        NavigationStack {
            Form {
                if !model.store.holds {
                    Section {
                        Label(tr("Signing takes packwing Pro: Settings › License. Checking a signature is free."),
                              systemImage: "signature")
                            .accessibilityIdentifier("sign-pro")
                        Button(tr("Settings")) {
                            dismiss()
                            model.tab = .settings
                        }
                    }
                    .listRowBackground(Tonal.surface)
                } else if !maySign(name: name) {
                    Section {
                        Label(tr("This is one part of a file cut in several. A signature covers one file, and would seem to cover them all, so packwing does not sign parts."),
                              systemImage: "exclamationmark.triangle")
                            .foregroundStyle(Tonal.warnText)
                    }
                    .listRowBackground(Tonal.warnBg)
                } else {
                    // First: what came of it shows above the keys, not under the button.
                    stateSection
                    Section {
                        if keys.isEmpty {
                            Text(tr("No key chosen")).foregroundStyle(Tonal.muted)
                        } else {
                            Picker(tr("Key"), selection: Binding { key?.id ?? "" } set: { chosen = $0 }) {
                                ForEach(keys, id: \.id) { Text($0.name).tag($0.id) }
                            }
                            .accessibilityIdentifier("sign-key")
                        }
                        Button(tr("New key…"), systemImage: "plus") { making = true }
                            .accessibilityIdentifier("sign-new-key")
                        Button(tr("Add a key file…"), systemImage: "key") { importing = true }
                    } footer: {
                        Text(tr("Anyone can check it, with the key’s public half"))
                    }
                    .listRowBackground(Tonal.surface)
                    .disabled(signing)
                }
            }
            .scrollContentBackground(.hidden)
            .background(Tonal.page)
            .navigationTitle(tr("Sign {name}", ["name": name]))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(role: .close) {
                        if case .signing(let job) = state { job.cancel() }
                        dismiss()
                    }
                }
            }
            .safeAreaInset(edge: .bottom) {
                if model.store.holds && maySign(name: name) {
                    Button(action: sign) {
                        Text(tr("Sign"))
                            .font(.nunito(17, .bold, relativeTo: .headline))
                            .frame(maxWidth: .infinity, minHeight: 52)
                    }
                    .buttonStyle(.glassProminent)
                    .disabled(key == nil || signing)
                    .accessibilityIdentifier("sign-confirm")
                    .padding(.horizontal, 20)
                    .padding(.bottom, 8)
                }
            }
            .sheet(isPresented: $making) {
                NameSheet(title: tr("New key…"), current: nil, confirm: tr("Make the key…")) { name in
                    do {
                        chosen = try makeSigningKey(name: name).id
                    } catch {
                        state = .failed((error as? MobileError).map(reason) ?? error.localizedDescription)
                    }
                    keys = (try? signingKeys()) ?? keys
                }
            }
            .sheet(isPresented: $importing) {
                ImportKeySheet(keep: { name, text, passphrase in
                    try importSigningKey(name: name, contents: Data(text.utf8), passphrase: passphrase).id
                }) { id in
                    keys = (try? signingKeys()) ?? keys
                    chosen = id
                }
            }
            .fileExporter(isPresented: Binding { saving != nil } set: { if !$0 { saving = nil } },
                          document: saving?.document, contentType: .data,
                          defaultFilename: saving?.name) { result in
                switch result {
                case .success(let url): state = .done(url.lastPathComponent)
                case .failure(let error): state = .failed(error.localizedDescription)
                }
            }
        }
        .presentationDetents([.medium, .large])
    }

    @ViewBuilder private var stateSection: some View {
        switch state {
        case .ready:
            EmptyView()
        case .signing:
            Section {
                HStack(spacing: 10) {
                    ProgressView()
                    Text(tr("Signing…"))
                }
            }
            .listRowBackground(Tonal.surface)
        case .done(let saved):
            Section {
                Label(tr("Signed: {name}", ["name": saved]), systemImage: "checkmark.seal.fill")
                    .foregroundStyle(Tonal.accent)
                    .accessibilityIdentifier("sign-done")
            }
            .listRowBackground(Tonal.surface)
        case .failed(let why):
            Section {
                Label(why, systemImage: "exclamationmark.triangle").foregroundStyle(Tonal.warnText)
            }
            .listRowBackground(Tonal.warnBg)
        }
    }

    /// Whether packwing may write beside the archive's file: always on a
    /// storage; on this device, where its folder lets it.
    private var besideIt: Bool {
        if archive.archive.stored() { return true }
        let folder = archive.url.deletingLastPathComponent().path(percentEncoded: false)
        var writable = FileManager.default.isWritableFile(atPath: folder)
        #if DEBUG
        // As for a change: the way an iPhone may take, which may not write beside a file of Files.
        if UserDefaults.standard.bool(forKey: "changeThroughCopy") { writable = false }
        #endif
        return writable
    }

    private func sign() {
        guard let key else { return }
        chosen = key.id
        let job = Job.quiet()
        state = .signing(job)
        let target = archive.archive
        let beside = besideIt
        let sigName = name + ".sig"
        Task {
            let result = await Task.detached(priority: .userInitiated) {
                Result { () throws -> String? in
                    if beside {
                        try target.signBeside(keyId: key.id, job: job)
                        return nil
                    }
                    return try target.signature(keyId: key.id, job: job)
                }
            }.value
            switch result {
            case .success(let text):
                // Whoever holds the key trusts it: what they sign shows as signed by them.
                if let read = try? readKey(text: key.public) { model.trust(read, named: key.name) }
                if let text {
                    saving = FileToSave(document: BytesDocument(Data(text.utf8)), name: sigName)
                    state = .ready
                } else {
                    state = .done(sigName)
                }
            case .failure(let error):
                if case .Cancelled = error as? MobileError {
                    state = .ready
                } else {
                    state = .failed((error as? MobileError).map(reason) ?? error.localizedDescription)
                }
            }
        }
    }
}
