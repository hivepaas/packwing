import SwiftUI

/// Asks for an archive's password, or an age file's passphrase, and opens it
/// with it.
struct PasswordSheet: View {
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    let ask: PasswordAsk
    @State private var secret = ""
    @State private var shown = false
    @FocusState private var focused: Bool

    var body: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: 18) {
                Text(ask.passphrase
                    ? tr("{name} is encrypted with a passphrase. Ask the person who sent it for it.", ["name": ask.name])
                    : tr("{name} is locked. Ask the person who sent it for its password.", ["name": ask.name]))
                    .font(.nunito(15, relativeTo: .subheadline))
                    .foregroundStyle(Tonal.muted)
                    .fixedSize(horizontal: false, vertical: true)

                VStack(alignment: .leading, spacing: 8) {
                    Text(ask.passphrase ? tr("Passphrase") : tr("Password"))
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
                        .accessibilityLabel(ask.passphrase ? tr("Passphrase") : tr("Password"))
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
            .padding(20)
            .navigationTitle(ask.passphrase ? tr("Enter the passphrase") : tr("Enter the password"))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(role: .close) { dismiss() }
                }
            }
        }
        .presentationDetents([.medium, .large])
        .onAppear { focused = true }
    }

    private func unlock() {
        guard !secret.isEmpty else { return }
        model.unlock(ask, with: secret)
    }
}
