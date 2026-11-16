import SwiftUI

struct SettingsView: View {
    @Environment(AppModel.self) private var model
    @AppStorage("language") private var chosen: String = ""
    @AppStorage("newFormat") private var formatKey = ArchiveFormat.zip.key
    @AppStorage("newCompression") private var levelKey = CompressionLevel.medium.key
    @AppStorage("storeCompressed") private var storeCompressed = true
    @AppStorage("skipMacFiles") private var skipMacFiles = true
    @AppStorage("extractLimit") private var extractLimit = ExtractLimit.standard.key

    var body: some View {
        NavigationStack {
            Form {
                Section(tr("General")) {
                    Picker(tr("Language"), selection: $chosen) {
                        Text(tr("As the system")).tag("")
                        ForEach(languages(), id: \.tag) { lang in
                            Text(lang.name).tag(lang.tag)
                        }
                    }
                    .onChange(of: chosen) { _, now in
                        model.speak(now.isEmpty ? nil : now)
                    }
                }
                .listRowBackground(Tonal.surface)
                Section(tr("New archives")) {
                    // What an earlier version kept, a format for one file, shows as ZIP.
                    Picker(tr("Format"), selection: Binding { ArchiveFormat.forMany(key: formatKey).key } set: { formatKey = $0 }) {
                        ForEach(ArchiveFormat.all, id: \.key) { Text($0.label).tag($0.key) }
                    }
                    .accessibilityIdentifier("settings-format")
                    Picker(tr("Compression"), selection: $levelKey) {
                        ForEach(CompressionLevel.all, id: \.key) { Text($0.label).tag($0.key) }
                    }
                    Toggle(isOn: $storeCompressed) {
                        Explained(tr("Don’t compress what already is"),
                                  tr("For ZIP and 7z: pictures, video, sound and archives go in as they are, sooner and no larger"))
                    }
                    .accessibilityIdentifier("settings-store-compressed")
                    Toggle(isOn: $skipMacFiles) {
                        Explained(tr("Leave out Mac-only files"),
                                  tr(".DS_Store, __MACOSX and resource forks, clutter anywhere but a Mac"))
                    }
                    .accessibilityIdentifier("settings-skip-mac-files")
                }
                .listRowBackground(Tonal.surface)
                Section {
                    // The choice beside its name: what it does goes under, or
                    // the two lines leave no room for it.
                    Picker(tr("Largest an archive may unpack to"), selection: $extractLimit) {
                        ForEach(ExtractLimit.all, id: \.key) { Text($0.label).tag($0.key) }
                    }
                    .accessibilityIdentifier("settings-extract-limit")
                } header: {
                    Text(tr("Extracting"))
                } footer: {
                    Text(tr("Stops archive bombs: small files that unpack to fill the disk"))
                }
                .listRowBackground(Tonal.surface)
                LicenseSection()
                TrustedKeysSection()
                SigningKeysSection()
                SshKeysSection()
                Section {
                    NavigationLink {
                        AboutView()
                    } label: {
                        HStack(spacing: 12) {
                            PackwingLogo(size: 36)
                            Text(tr("About"))
                            Spacer(minLength: 8)
                            Text(version).foregroundStyle(Tonal.muted)
                        }
                    }
                    .accessibilityIdentifier("settings-about")
                }
                .listRowBackground(Tonal.surface)
            }
            .scrollContentBackground(.hidden)
            .background(Tonal.page)
            .navigationTitle(tr("Settings"))
        }
    }

    private var version: String {
        Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? ""
    }
}

/// A setting's name, and what it does under it.
private struct Explained: View {
    let title: String
    let detail: String

    init(_ title: String, _ detail: String) {
        self.title = title
        self.detail = detail
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
            Text(detail)
                .font(.nunito(12, relativeTo: .caption))
                .foregroundStyle(Tonal.muted)
        }
    }
}

/// The most an archive may unpack to, as the desktop offers it; 0 for no
/// limit. Kept by its key, as the other choices are.
struct ExtractLimit {
    let key: String
    let bytes: UInt64
    let label: String

    static var standard: ExtractLimit { all[1] }

    // Said again each time: in the language in use.
    static var all: [ExtractLimit] { [
        ExtractLimit(key: "10gb", bytes: 10_000_000_000, label: "10 GB"),
        ExtractLimit(key: "100gb", bytes: 100_000_000_000, label: "100 GB"),
        ExtractLimit(key: "1tb", bytes: 1_000_000_000_000, label: "1 TB"),
        ExtractLimit(key: "none", bytes: 0, label: tr("No limit")),
    ] }

    /// The one kept as `key`; 100 GB if there is none.
    init(key: String) {
        self = Self.all.first { $0.key == key } ?? Self.standard
    }

    private init(key: String, bytes: UInt64, label: String) {
        self.key = key
        self.bytes = bytes
        self.label = label
    }
}
