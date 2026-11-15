import SwiftUI
import UniformTypeIdentifiers

/// Asks for an archive's password, an age file's passphrase, or the key file
/// it was encrypted to and that file's passphrase, and opens it with it.
struct PasswordSheet: View {
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    let ask: PasswordAsk
    @State private var secret = ""
    @State private var shown = false
    @FocusState private var focused: Bool
    @State private var choosingKey = false
    /// Why the key file chosen or pasted is not one, if it is not.
    @State private var keyProblem: String?

    var body: some View {
        NavigationStack {
            Group {
                if ask.asks == .keyFile { keyFile } else { typed }
            }
            .padding(20)
            .navigationTitle(title)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(role: .close) { dismiss() }
                }
            }
        }
        .presentationDetents([.medium, .large])
        .onAppear { focused = ask.asks != .keyFile }
    }

    private var title: String {
        switch ask.asks {
        case .password: tr("Enter the password")
        case .passphrase: tr("Enter the passphrase")
        case .keyFile: tr("Choose the key file")
        case .keyPassphrase: tr("Enter the key file’s passphrase")
        }
    }

    private var label: String {
        ask.asks == .password ? tr("Password") : tr("Passphrase")
    }

    private var said: String {
        switch ask.asks {
        case .password: tr("{name} is locked. Ask the person who sent it for its password.", ["name": ask.name])
        case .passphrase: tr("{name} is encrypted with a passphrase. Ask the person who sent it for it.", ["name": ask.name])
        case .keyFile: tr("This file is encrypted with age to a key: an age key file or an SSH key. packwing keeps what it reads from it only while the archive is open, and never writes it down.")
        case .keyPassphrase: tr("The key file is itself protected. packwing keeps its passphrase only while the archive is open, and never writes it down.")
        }
    }

    /// The key file: chosen in Files, or its text pasted.
    private var keyFile: some View {
        VStack(alignment: .leading, spacing: 18) {
            Text(said)
                .font(.nunito(15, relativeTo: .subheadline))
                .foregroundStyle(Tonal.muted)
                .fixedSize(horizontal: false, vertical: true)
            if let problem = keyProblem ?? ask.problem {
                Label(problem, systemImage: "exclamationmark.circle")
                    .font(.nunito(14, .medium, relativeTo: .subheadline))
                    .foregroundStyle(Tonal.badText)
                    .fixedSize(horizontal: false, vertical: true)
                    .accessibilityIdentifier("key-problem")
            }
            Spacer(minLength: 0)
            // A key kept as text (in Notes, or a password manager): pasted.
            PasteButton(payloadType: String.self) { texts in
                guard let text = texts.first else { return }
                Task { @MainActor in use(Data(text.utf8)) }
            }
            .frame(maxWidth: .infinity)
            .disabled(model.opening != nil)
            Button { choosingKey = true } label: {
                Text(tr("Choose…"))
                    .font(.nunito(17, .bold, relativeTo: .headline))
                    .frame(maxWidth: .infinity, minHeight: 52)
            }
            .buttonStyle(.glassProminent)
            .disabled(model.opening != nil)
            .accessibilityIdentifier("key-choose")
        }
        .fileImporter(isPresented: $choosingKey, allowedContentTypes: [.item]) { result in
            guard case .success(let url) = result else { return }
            let scoped = url.startAccessingSecurityScopedResource()
            defer { if scoped { url.stopAccessingSecurityScopedResource() } }
            do {
                use(try Self.read(url))
            } catch {
                keyProblem = error.localizedDescription
            }
        }
    }

    /// The most a key file is read of: more than any key file is, and a file
    /// chosen by mistake may be any size.
    private static let keyFileLimit = 16 << 20

    private static func read(_ url: URL) throws -> Data {
        let handle = try FileHandle(forReadingFrom: url)
        defer { try? handle.close() }
        return try handle.read(upToCount: keyFileLimit) ?? Data()
    }

    /// Opens with `key`, if it is a key file packwing can use.
    private func use(_ key: Data) {
        do {
            try checkKeyFile(contents: key)
            keyProblem = nil
            model.unlock(ask, key: key)
        } catch {
            keyProblem = (error as? MobileError).map(reason) ?? error.localizedDescription
        }
    }

    /// A password or passphrase, typed.
    private var typed: some View {
            VStack(alignment: .leading, spacing: 18) {
                Text(said)
                    .font(.nunito(15, relativeTo: .subheadline))
                    .foregroundStyle(Tonal.muted)
                    .fixedSize(horizontal: false, vertical: true)

                VStack(alignment: .leading, spacing: 8) {
                    Text(label)
                        .font(.nunito(14, .semibold, relativeTo: .subheadline))
                    HStack {
                        Group {
                            if shown {
                                TextField("", text: $secret)
                            } else {
                                SecureField("", text: $secret)
                            }
                        }
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .textContentType(.password)
                        .focused($focused)
                        .submitLabel(.go)
                        .onSubmit(unlock)
                        .accessibilityLabel(label)
                        Button {
                            shown.toggle()
                        } label: {
                            Image(systemName: shown ? "eye.slash" : "eye")
                                .frame(width: 44, height: 44)
                        }
                        .foregroundStyle(Tonal.muted)
                        .accessibilityLabel(shown ? tr("Hide password") : tr("Show password"))
                    }
                    .padding(.leading, 18)
                    .padding(.trailing, 6)
                    .frame(minHeight: 56)
                    .background(Tonal.field, in: .rect(cornerRadius: 20))
                    .overlay {
                        RoundedRectangle(cornerRadius: 20)
                            .strokeBorder(ask.problem == nil ? Tonal.accent : Tonal.badText, lineWidth: focused ? 2 : 0)
                    }
                    if let problem = ask.problem {
                        Label(problem, systemImage: "exclamationmark.circle")
                            .font(.nunito(14, .medium, relativeTo: .subheadline))
                            .foregroundStyle(Tonal.badText)
                    }
                    Button(tr("Paste"), systemImage: "doc.on.clipboard") {
                        if let text = UIPasteboard.general.string { secret = text }
                    }
                    .font(.nunito(15, .semibold, relativeTo: .subheadline))
                }

                Label(tr("Kept in memory only, never saved, and forgotten when the archive is closed."),
                      systemImage: "checkmark.shield")
                    .font(.nunito(13, relativeTo: .footnote))
                    .foregroundStyle(Tonal.muted)
                    .fixedSize(horizontal: false, vertical: true)

                Spacer(minLength: 0)

                Button(action: unlock) {
                    Text(tr("Unlock"))
                        .font(.nunito(17, .bold, relativeTo: .headline))
                        .frame(maxWidth: .infinity, minHeight: 52)
                }
                .buttonStyle(.glassProminent)
                .disabled(secret.isEmpty || model.opening != nil)
            }
    }

    private func unlock() {
        guard !secret.isEmpty else { return }
        model.unlock(ask, with: secret)
    }
}
