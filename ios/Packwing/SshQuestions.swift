import SwiftUI
import UIKit
import UniformTypeIdentifiers

/// What SFTP asks as it connects: whether to trust a server packwing does not
/// know yet, a password, a key's passphrase, a code. The core asks on the
/// worker thread a storage's call runs on, and waits: the question shows in a
/// window of its own, over whatever shows (a sheet, a job's screen), and its
/// answer goes back to that thread.
final class SshAsker: SshQuestions, @unchecked Sendable {
    func hostKey(host: String, port: UInt16, algorithm: String, fingerprint: String) -> Bool {
        if case .trust(let yes) = ask(.hostKey(host: host, port: port, algorithm: algorithm, fingerprint: fingerprint)) {
            return yes
        }
        return false
    }

    func password(target: String, retry: Bool) -> TypedSecret? {
        if case .secret(let typed) = ask(.password(target: target, retry: retry)) { return typed }
        return nil
    }

    func passphrase(key: String, retry: Bool) -> TypedSecret? {
        if case .secret(let typed) = ask(.passphrase(key: key, retry: retry)) { return typed }
        return nil
    }

    func code(name: String, instructions: String, text: String, echo: Bool) -> String? {
        if case .code(let answer) = ask(.code(name: name, instructions: instructions, text: text, echo: echo)) {
            return answer
        }
        return nil
    }

    /// Shows `question`, and waits for its answer. Never on the main thread,
    /// which the answer comes from.
    private func ask(_ question: SshQuestion) -> SshAnswer {
        guard !Thread.isMainThread else { return .none }
        let answered = DispatchSemaphore(value: 0)
        var answer = SshAnswer.none
        DispatchQueue.main.async {
            QuestionWindow.shared.show(question) { said in
                answer = said
                answered.signal()
            }
        }
        answered.wait()
        return answer
    }
}

enum SshQuestion {
    case hostKey(host: String, port: UInt16, algorithm: String, fingerprint: String)
    case password(target: String, retry: Bool)
    case passphrase(key: String, retry: Bool)
    case code(name: String, instructions: String, text: String, echo: Bool)
}

enum SshAnswer {
    case none
    case trust(Bool)
    case secret(TypedSecret?)
    case code(String?)
}

/// A window over every other, for a question asked from anywhere.
@MainActor
final class QuestionWindow {
    static let shared = QuestionWindow()
    private var window: UIWindow?

    func show(_ question: SshQuestion, answer: @escaping (SshAnswer) -> Void) {
        guard let scene = UIApplication.shared.connectedScenes
            .compactMap({ $0 as? UIWindowScene })
            .first(where: { $0.activationState == .foregroundActive }) ?? UIApplication.shared.connectedScenes
            .compactMap({ $0 as? UIWindowScene }).first
        else {
            answer(.none)
            return
        }
        let window = UIWindow(windowScene: scene)
        window.windowLevel = .alert + 1
        let view = SshQuestionView(question: question) { [weak self] said in
            self?.window?.isHidden = true
            self?.window = nil
            answer(said)
        }
        .tint(Tonal.accent)
        .font(.nunito(17))
        let host = UIHostingController(rootView: view)
        host.view.backgroundColor = .clear
        window.rootViewController = host
        window.backgroundColor = .clear
        self.window = window
        window.makeKeyAndVisible()
    }
}

/// A question, as a card over the screen dimmed: the desktop's, in its words.
struct SshQuestionView: View {
    let question: SshQuestion
    let answer: (SshAnswer) -> Void
    @State private var secret = ""
    @State private var keep = false
    @FocusState private var typing: Bool

    var body: some View {
        ZStack {
            Color.black.opacity(0.35).ignoresSafeArea()
            VStack(alignment: .leading, spacing: 16) {
                HStack(spacing: 12) {
                    Image(systemName: icon)
                        .font(.system(size: 20, weight: .semibold))
                        .foregroundStyle(Tonal.onTonal)
                        .frame(width: 44, height: 44)
                        .background(Tonal.tonal, in: .rect(cornerRadius: 14))
                    VStack(alignment: .leading, spacing: 2) {
                        Text(title)
                            .font(.nunito(19, .bold, relativeTo: .title3))
                        if !detail.isEmpty {
                            Text(detail)
                                .font(.nunito(14, relativeTo: .subheadline))
                                .foregroundStyle(Tonal.muted)
                                .lineLimit(2)
                        }
                    }
                }
                content
                HStack(spacing: 10) {
                    Button { answer(no) } label: {
                        Text(noLabel).frame(maxWidth: .infinity, minHeight: 44)
                    }
                    .buttonStyle(.bordered)
                    Button { answer(yes) } label: {
                        Text(yesLabel).font(.nunito(17, .bold, relativeTo: .headline))
                            .frame(maxWidth: .infinity, minHeight: 44)
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(needsSecret && secret.isEmpty)
                    .accessibilityIdentifier("ssh-yes")
                }
            }
            .padding(22)
            .background(Tonal.page, in: .rect(cornerRadius: 28))
            .frame(maxWidth: 460)
            .padding(20)
        }
        .onAppear { typing = needsSecret || isCode }
    }

    @ViewBuilder
    private var content: some View {
        switch question {
        case .hostKey(_, _, let algorithm, let fingerprint):
            Text(tr("packwing has not connected to this server before. Trust it only if this is its key: whoever runs the server can tell you its fingerprint."))
                .font(.nunito(15, relativeTo: .body))
                .foregroundStyle(Tonal.muted)
            VStack(alignment: .leading, spacing: 6) {
                Text(tr("{algorithm} key fingerprint", ["algorithm": algorithm]))
                    .font(.nunito(13, .semibold, relativeTo: .footnote))
                    .foregroundStyle(Tonal.muted)
                // Whole, on as many lines as it takes: it is compared, digit by digit.
                Text(breakable(fingerprint))
                    .font(.system(.callout, design: .monospaced).weight(.medium))
                    .fixedSize(horizontal: false, vertical: true)
                    .padding(14)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Tonal.field, in: .rect(cornerRadius: 16))
                    .accessibilityLabel(fingerprint)
                    .accessibilityIdentifier("ssh-fingerprint")
            }
        case .password(_, let retry), .passphrase(_, let retry):
            if retry {
                Label(isPassword ? tr("The server did not take that password.") : tr("That passphrase does not open the key."),
                      systemImage: "exclamationmark.triangle")
                    .font(.nunito(14, relativeTo: .subheadline))
                    .foregroundStyle(Tonal.warnText)
            }
            SecureField(isPassword ? tr("Password") : tr("Passphrase"), text: $secret)
                .textContentType(.oneTimeCode)
                .focused($typing)
                .onSubmit { if !secret.isEmpty { answer(yes) } }
                .padding(14)
                .background(Tonal.field, in: .rect(cornerRadius: 16))
                .accessibilityIdentifier("ssh-secret")
            Toggle(tr("Keep it in the system’s keychain"), isOn: $keep)
                .font(.nunito(15, relativeTo: .body))
        case .code(_, _, let text, let echo):
            let label = text.trimmingCharacters(in: .whitespacesAndNewlines)
            Group {
                if echo {
                    TextField(label, text: $secret)
                } else {
                    SecureField(label, text: $secret)
                }
            }
            .textContentType(.oneTimeCode)
            .autocorrectionDisabled()
            .textInputAutocapitalization(.never)
            .focused($typing)
            .padding(14)
            .background(Tonal.field, in: .rect(cornerRadius: 16))
        }
    }

    private var isPassword: Bool {
        if case .password = question { return true }
        return false
    }

    private var isCode: Bool {
        if case .code = question { return true }
        return false
    }

    private var needsSecret: Bool {
        switch question {
        case .password, .passphrase: true
        default: false
        }
    }

    private var icon: String {
        if case .hostKey = question { return "checkmark.shield" }
        return "lock"
    }

    private var title: String {
        switch question {
        case .hostKey: tr("Trust this server?")
        case .password: tr("Sign in to the server")
        case .passphrase: tr("Enter the key’s passphrase")
        case .code(let name, _, _, _):
            name.trimmingCharacters(in: .whitespaces).isEmpty ? tr("The server asks") : name
        }
    }

    private var detail: String {
        switch question {
        case .hostKey(let host, let port, _, _): port == 22 ? host : "\(host):\(port)"
        case .password(let target, _): target
        case .passphrase(let key, _): key
        case .code(_, let instructions, _, _): instructions
        }
    }

    private var yesLabel: String {
        switch question {
        case .hostKey: tr("Trust and connect")
        case .password: tr("Sign in")
        case .passphrase: tr("Open")
        case .code: tr("Send")
        }
    }

    private var noLabel: String {
        if case .hostKey = question { return tr("Don’t connect") }
        return tr("Cancel")
    }

    private var yes: SshAnswer {
        switch question {
        case .hostKey: .trust(true)
        case .password, .passphrase: .secret(TypedSecret(secret: secret, keep: keep))
        case .code: .code(secret)
        }
    }

    private var no: SshAnswer {
        switch question {
        case .hostKey: .trust(false)
        case .password, .passphrase: .secret(nil)
        case .code: .code(nil)
        }
    }
}

// MARK: Keys kept

/// The key a storage signs in with, chosen from those kept, or any of them;
/// and Add a key file….
struct SshKeyPicker: View {
    @Binding var selection: String
    @State private var keys: [SshKey] = (try? sshKeys()) ?? []
    @State private var importing = false

    var body: some View {
        Section {
            Picker(tr("Key"), selection: $selection) {
                Text(tr("Any key")).tag("")
                ForEach(keys, id: \.id) { key in
                    Text(key.name).tag(key.id)
                }
            }
            Button(tr("Add a key file…"), systemImage: "key") { importing = true }
        } header: {
            Text(tr("Key"))
        }
        .listRowBackground(Tonal.surface)
        .sheet(isPresented: $importing) {
            ImportKeySheet { id in
                keys = (try? sshKeys()) ?? keys
                selection = id
            }
        }
    }
}

/// A private key to keep: pasted or from its file, opened with its passphrase
/// if it has one.
struct ImportKeySheet: View {
    @Environment(\.dismiss) private var dismiss
    /// Keeps the key typed or read, as named, with its passphrase if one was
    /// given, and says its id: an SSH key by default, a signing key for
    /// Settings' Signing keys.
    var keep: (_ name: String, _ text: String, _ passphrase: String?) throws -> String = { name, text, passphrase in
        try importSshKey(name: name, text: text, passphrase: passphrase).id
    }
    /// Told the id of the key kept.
    let kept: (String) -> Void
    @State private var name = ""
    @State private var text = ""
    @State private var passphrase = ""
    @State private var asksPassphrase = false
    @State private var problem: String?
    @State private var choosing = false

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("-----BEGIN OPENSSH PRIVATE KEY-----", text: $text, axis: .vertical)
                        .lineLimit(4...8)
                        .font(.system(.footnote, design: .monospaced))
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)
                        .textContentType(.oneTimeCode)
                        .accessibilityIdentifier("key-text")
                    Button(tr("Choose a file…"), systemImage: "doc") { choosing = true }
                } header: {
                    Text(tr("Private key"))
                }
                .listRowBackground(Tonal.surface)
                Section(tr("Name")) {
                    TextField(tr("Name"), text: $name)
                        .textContentType(.oneTimeCode)
                }
                .listRowBackground(Tonal.surface)
                if asksPassphrase {
                    Section(tr("Passphrase")) {
                        SecureField(tr("Passphrase"), text: $passphrase)
                            .textContentType(.oneTimeCode)
                    }
                    .listRowBackground(Tonal.surface)
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
            .navigationTitle(tr("Private key"))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(role: .close) { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(role: .confirm, action: add)
                        .disabled(text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                        .accessibilityLabel(tr("Add"))
                }
            }
            .fileImporter(isPresented: $choosing, allowedContentTypes: [.item]) { result in
                guard case .success(let url) = result else { return }
                let scoped = url.startAccessingSecurityScopedResource()
                defer { if scoped { url.stopAccessingSecurityScopedResource() } }
                // A key file is small; anything much bigger is not one.
                if let data = try? Data(contentsOf: url), data.count < 64 * 1024 {
                    text = String(decoding: data, as: UTF8.self)
                    if name.isEmpty { name = url.lastPathComponent }
                }
            }
        }
    }

    private func add() {
        let named = name.trimmingCharacters(in: .whitespacesAndNewlines)
        do {
            kept(try keep(named.isEmpty ? tr("Key") : named, text, passphrase.isEmpty ? nil : passphrase))
            dismiss()
        } catch MobileError.Locked(let why, _, _) {
            asksPassphrase = true
            problem = why
        } catch {
            problem = (error as? MobileError).map(reason) ?? error.localizedDescription
        }
    }
}

/// Settings' SSH keys: those kept, removed by a swipe.
struct SshKeysSection: View {
    @State private var keys: [SshKey] = (try? sshKeys()) ?? []
    @State private var importing = false

    var body: some View {
        Section {
            ForEach(keys, id: \.id) { key in
                VStack(alignment: .leading, spacing: 2) {
                    Text(key.name).font(.nunito(16, .semibold, relativeTo: .body))
                    Text("\(key.kind) · \(key.fingerprint)")
                        .font(.system(.caption, design: .monospaced))
                        .foregroundStyle(Tonal.muted)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }
                .contextMenu {
                    Button(tr("Copy"), systemImage: "doc.on.doc") { UIPasteboard.general.string = key.public }
                }
            }
            .onDelete { offsets in
                for i in offsets { try? removeSshKey(id: keys[i].id) }
                keys = (try? sshKeys()) ?? []
            }
            Button(tr("Add a key file…"), systemImage: "key") { importing = true }
        } header: {
            Text(tr("SSH keys"))
        }
        .listRowBackground(Tonal.surface)
        .sheet(isPresented: $importing) {
            ImportKeySheet { _ in keys = (try? sshKeys()) ?? keys }
        }
        .onAppear { keys = (try? sshKeys()) ?? keys }
    }
}
