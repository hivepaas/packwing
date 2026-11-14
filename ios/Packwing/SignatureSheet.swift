import SwiftUI
import UniformTypeIdentifiers

/// A public key the user trusts, by the name they gave it.
struct TrustedKey: Codable, Hashable, Identifiable {
    var name: String
    /// As `read_key` gives it: its kind and its base64.
    var line: String
    var id: String { line }

    var fingerprint: String { (try? readKey(text: line))?.fingerprint ?? "" }
}

extension AppModel {
    // MARK: Keys trusted

    private static let keysKey = "trustedKeys"

    static func loadKeys() -> [TrustedKey] {
        guard let data = UserDefaults.standard.data(forKey: keysKey) else { return [] }
        return (try? JSONDecoder().decode([TrustedKey].self, from: data)) ?? []
    }

    private func saveKeys() {
        UserDefaults.standard.set(try? JSONEncoder().encode(trustedKeys), forKey: Self.keysKey)
    }

    /// Trusts `key`, under `name`, or the start of its fingerprint if that
    /// says nothing. One trusted already keeps the name it has.
    func trust(_ key: KeyRead, named name: String) {
        guard !trustedKeys.contains(where: { $0.line == key.line }) else { return }
        let name = name.trimmingCharacters(in: .whitespacesAndNewlines)
        trustedKeys.append(TrustedKey(name: name.isEmpty ? String(key.fingerprint.prefix(19)) : name,
                                      line: key.line))
        saveKeys()
    }

    func distrust(at offsets: IndexSet) {
        trustedKeys.remove(atOffsets: offsets)
        saveKeys()
    }

    /// What the user calls the key of `fingerprint`, or the fingerprint.
    func signer(_ fingerprint: String) -> String {
        trustedKeys.first { $0.fingerprint == fingerprint }?.name ?? fingerprint
    }
}

/// Whether an archive is what a key the user trusts signed: by its signature
/// beside it, `name.sig`, if packwing can reach that, or one chosen in Files.
struct SignatureSheet: View {
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    let archive: OpenedArchive

    private enum Look {
        /// None beside it to reach: one is chosen.
        case none
        case checking
        case done(SignatureState)
    }

    @State private var look = Look.checking
    /// The signature last checked, to check it again once its key is trusted.
    @State private var signature: URL?
    @State private var job: Job?
    @State private var choosing = false
    @State private var naming: KeyRead?
    @State private var name = ""

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    HStack(spacing: 14) {
                        KindTile(name: archive.info.name, kind: .of(name: archive.info.name),
                                 tint: Tint(ground: Tonal.surface, ink: Tonal.onTonal))
                        VStack(alignment: .leading, spacing: 2) {
                            Text(archive.info.name)
                                .font(.nunito(16, .bold, relativeTo: .body))
                                .lineLimit(1)
                                .truncationMode(.middle)
                            if let signature {
                                Text(signature.lastPathComponent)
                                    .font(.nunito(13, relativeTo: .footnote))
                                    .lineLimit(1)
                                    .truncationMode(.middle)
                            }
                        }
                    }
                    .foregroundStyle(Tonal.onTonal)
                    .listRowBackground(Tonal.tonal)
                }

                Section {
                    said
                    Button(tr("Choose its signature…"), systemImage: "doc.badge.ellipsis") { choosing = true }
                        .listRowBackground(Tonal.surface)
                } footer: {
                    Text(tr("A signature says who signed a file only by a key you trust: Settings › Keys you trust. One by any other key proves nothing, whatever it matches."))
                        .font(.nunito(13, relativeTo: .footnote))
                }
            }
            .scrollContentBackground(.hidden)
            .background(Tonal.page)
            .navigationTitle(tr("Signature"))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(role: .close) { dismiss() }
                }
            }
            .fileImporter(isPresented: $choosing, allowedContentTypes: [.item]) { result in
                if case .success(let url) = result { check(url) }
            }
            .alert(tr("Trust this key"), isPresented: Binding { naming != nil } set: { if !$0 { naming = nil } },
                   presenting: naming) { key in
                TextField(tr("Name"), text: $name)
                Button(tr("Cancel"), role: .cancel) {}
                Button(tr("Trust this key")) {
                    model.trust(key, named: name)
                    if let signature { check(signature) }
                }
            } message: { key in
                Text(key.fingerprint)
            }
        }
        .presentationDetents([.medium, .large])
        .task { lookBeside() }
        .onDisappear { job?.cancel() }
    }

    /// What the signature says, in a row of its standing's colors.
    @ViewBuilder
    private var said: some View {
        switch look {
        case .none:
            row(tr("There is no signature beside it."), "signature", Tonal.muted, Tonal.surface)
        case .checking:
            HStack(spacing: 12) {
                ProgressView()
                Text(tr("Checking its signature…"))
                    .font(.nunito(15, .semibold, relativeTo: .body))
            }
            .listRowBackground(Tonal.surface)
        case .done(let state):
            switch state {
            case .good(let fingerprint):
                row(tr("Signed by {signer}", ["signer": model.signer(fingerprint)]),
                    "checkmark.seal.fill", Tonal.onAccent, Tonal.accent, detail: fingerprint, code: true)
                    .accessibilityIdentifier("signature-good")
            case .stranger(let fingerprint, let key):
                row(tr("Signed by a key you do not trust: {fingerprint}", ["fingerprint": fingerprint]),
                    "exclamationmark.shield.fill", Tonal.warnText, Tonal.warnBg)
                    .accessibilityIdentifier("signature-stranger")
                if let key = try? readKey(text: key) {
                    Button(tr("Trust this key"), systemImage: "person.badge.shield.checkmark") {
                        name = ""
                        naming = key
                    }
                    .listRowBackground(Tonal.surface)
                }
            case .mismatch(let fingerprint):
                row(tr("It does not match its signature"), "xmark.seal.fill", Tonal.badText, Tonal.badBg,
                    detail: model.signer(fingerprint))
                    .accessibilityIdentifier("signature-mismatch")
            case .unreadable(let why):
                row(tr("Its signature cannot be read"), "exclamationmark.triangle.fill",
                    Tonal.warnText, Tonal.warnBg, detail: why)
            case .failed(let why):
                row(tr("Its signature could not be checked"), "exclamationmark.triangle.fill",
                    Tonal.warnText, Tonal.warnBg, detail: why)
            }
        }
    }

    /// A row saying `text`, with `detail` under it: a fingerprint (`code`),
    /// or why.
    private func row(_ text: String, _ icon: String, _ ink: Color, _ ground: Color,
                     detail: String? = nil, code: Bool = false) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 12) {
            Image(systemName: icon)
            VStack(alignment: .leading, spacing: 4) {
                Text(text)
                    .font(.nunito(15, .bold, relativeTo: .body))
                if let detail {
                    Text(code ? breakable(detail) : detail)
                        .font(code ? .system(.caption, design: .monospaced) : .nunito(13, relativeTo: .footnote))
                        .opacity(0.85)
                        .accessibilityLabel(detail)
                }
            }
        }
        .foregroundStyle(ink)
        .padding(.vertical, 4)
        .listRowBackground(ground)
    }

    /// The signature beside it, if packwing can reach it: in packwing's own
    /// folder, or one Files gave access to all of.
    private func lookBeside() {
        let beside = archive.url.deletingLastPathComponent()
            .appending(path: archive.url.lastPathComponent + ".sig")
        if FileManager.default.isReadableFile(atPath: beside.path(percentEncoded: false)) {
            check(beside)
        } else {
            look = .none
        }
    }

    private func check(_ url: URL) {
        job?.cancel()
        signature = url
        look = .checking
        let job = Job.quiet()
        self.job = job
        let archive = archive.archive
        let trusted = model.trustedKeys.map(\.line)
        Task {
            let scoped = url.startAccessingSecurityScopedResource()
            defer { if scoped { url.stopAccessingSecurityScopedResource() } }
            let path = url.path(percentEncoded: false)
            let result = await Task.detached(priority: .userInitiated) {
                Result { try archive.checkSignature(signature: path, trusted: trusted, job: job) }
            }.value
            guard self.job === job else { return }
            self.job = nil
            switch result {
            case .success(let state):
                look = .done(state)
            case .failure(let error):
                if case .Cancelled = error as? MobileError { return }
                look = .done(.failed(reason: (error as? MobileError).map(reason) ?? error.localizedDescription))
            }
        }
    }
}

/// Why a call failed, as the core says it, in the user's language.
func reason(_ error: MobileError) -> String {
    switch error {
    case .NotAnArchive(let why), .WrongPassword(let why), .Cancelled(let why), .Failed(let why), .Locked(let why, _):
        why
    }
}

/// Settings' keys the user trusts: a signature by one says who signed a file.
struct TrustedKeysSection: View {
    @Environment(AppModel.self) private var model
    @State private var adding = false

    var body: some View {
        Section {
            ForEach(model.trustedKeys) { key in
                VStack(alignment: .leading, spacing: 2) {
                    Text(key.name)
                        .font(.nunito(16, .semibold, relativeTo: .body))
                    Text(key.fingerprint)
                        .font(.system(.caption, design: .monospaced))
                        .foregroundStyle(Tonal.muted)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }
            }
            .onDelete { model.distrust(at: $0) }
            Button(tr("Paste a public key"), systemImage: "plus") { adding = true }
        } header: {
            Text(tr("Keys you trust"))
        } footer: {
            Text(tr("A signature by one of these says who signed a file. One by any other key proves nothing, whatever it matches."))
        }
        .listRowBackground(Tonal.surface)
        .sheet(isPresented: $adding) { AddKeySheet() }
    }
}

/// A public key to trust, pasted or from a file, and its name.
private struct AddKeySheet: View {
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    @State private var text = ""
    @State private var name = ""
    @State private var choosing = false
    @State private var problem: String?

    private var key: KeyRead? { try? readKey(text: text) }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField(tr("Type a public key, or choose a file of them."), text: $text, axis: .vertical)
                        .lineLimit(3...6)
                        .font(.system(.footnote, design: .monospaced))
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)
                        .accessibilityIdentifier("key-text")
                    Button(tr("Choose a file…"), systemImage: "doc") { choosing = true }
                } header: {
                    Text(tr("Public key"))
                } footer: {
                    if let key {
                        Text(key.fingerprint).font(.system(.caption, design: .monospaced))
                    } else if !text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                        Text(problemOf(text)).foregroundStyle(Tonal.badText)
                    }
                }
                .listRowBackground(Tonal.surface)
                Section(tr("Name")) {
                    TextField(tr("Name"), text: $name)
                        .accessibilityIdentifier("key-name")
                }
                .listRowBackground(Tonal.surface)
            }
            .scrollContentBackground(.hidden)
            .background(Tonal.page)
            .navigationTitle(tr("Paste a public key"))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(role: .close) { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(role: .confirm) {
                        if let key {
                            model.trust(key, named: name)
                            dismiss()
                        }
                    }
                    .disabled(key == nil)
                    .accessibilityLabel(tr("Trust this key"))
                }
            }
            .fileImporter(isPresented: $choosing, allowedContentTypes: [.item]) { result in
                guard case .success(let url) = result else { return }
                let scoped = url.startAccessingSecurityScopedResource()
                defer { if scoped { url.stopAccessingSecurityScopedResource() } }
                // A key file is a line; anything much bigger is not one.
                if let data = try? Data(contentsOf: url), data.count < 64 * 1024 {
                    text = String(decoding: data, as: UTF8.self)
                }
            }
        }
    }

    private func problemOf(_ text: String) -> String {
        do {
            _ = try readKey(text: text)
            return ""
        } catch let error as MobileError {
            return reason(error)
        } catch {
            return error.localizedDescription
        }
    }
}
