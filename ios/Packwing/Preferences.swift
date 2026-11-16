import SwiftUI

// The desktop's settings, as the phone keeps them (UserDefaults), each by the
// key the desktop names it by: how packwing looks, where extracted files go,
// and what a new archive is made with.

/// Light, dark, or as the system is.
enum Appearance: String, CaseIterable {
    case system, light, dark

    var label: String {
        switch self {
        case .system: tr("System")
        case .light: tr("Light")
        case .dark: tr("Dark")
        }
    }

    /// What SwiftUI is told: none to follow the system.
    var scheme: ColorScheme? {
        switch self {
        case .system: nil
        case .light: .light
        case .dark: .dark
        }
    }
}

/// Where the files of an archive go, in the folder Extract saves to.
enum ExtractPlace: String, CaseIterable {
    /// Right in the folder if what goes is one thing, else a new folder.
    case smart
    /// A new folder, named after the archive.
    case newFolder = "new-folder"
    /// Straight into the folder.
    case here

    var label: String {
        switch self {
        case .smart: tr("Smart")
        case .newFolder: tr("In a new folder")
        case .here: tr("Straight into the folder")
        }
    }

    var detail: String {
        switch self {
        case .smart: tr("One item right here, more in a new folder")
        case .newFolder: tr("Named after the archive")
        case .here: tr("No folder of its own")
        }
    }

    /// As Settings › Extracting says; Smart, as on the desktop, until it says.
    static var chosen: ExtractPlace {
        UserDefaults.standard.string(forKey: "extractTo").flatMap(ExtractPlace.init) ?? .smart
    }

    /// Whether `going` items go in a new folder.
    func newFolder(going: Int) -> Bool {
        switch self {
        case .smart: going != 1
        case .newFolder: true
        case .here: false
        }
    }
}

/// Above how many bytes a new archive bound for a storage is written straight
/// there, as the desktop offers it; kept by its key.
struct StreamAbove {
    let key: String
    let bytes: UInt64

    static var all: [StreamAbove] {
        [
            StreamAbove(key: "always", bytes: 0),
            StreamAbove(key: "100mb", bytes: 100_000_000),
            StreamAbove(key: "500mb", bytes: 500_000_000),
            StreamAbove(key: "1gb", bytes: 1_000_000_000),
            StreamAbove(key: "10gb", bytes: 10_000_000_000),
        ]
    }

    /// 500 MB, as on the desktop.
    static var standard: StreamAbove { all[2] }

    var label: String { bytes == 0 ? tr("Always") : sizeText(bytes) }

    /// The one kept as `key`; 500 MB if there is none.
    init(key: String) {
        self = Self.all.first { $0.key == key } ?? Self.standard
    }

    private init(key: String, bytes: UInt64) {
        self.key = key
        self.bytes = bytes
    }

    static var chosen: StreamAbove { StreamAbove(key: UserDefaults.standard.string(forKey: "streamAbove") ?? "") }
}

/// The extensions of what goes in as it is: those typed in Settings, or the
/// desktop's list until any are.
enum StoredExtensions {
    static let key = "storedExtensions"

    /// What is kept: none for the desktop's list.
    static var chosen: [String]? {
        guard let text = UserDefaults.standard.string(forKey: key), !text.isEmpty else { return nil }
        return extensionsIn(text: text)
    }
}

/// Settings › What goes in as it is: the extensions typed, apart by spaces,
/// and the way back to the desktop's list.
struct StoredExtensionsView: View {
    @AppStorage(StoredExtensions.key) private var kept = ""
    @State private var text = ""

    var body: some View {
        Form {
            Section {
                TextField(tr("Extensions, apart by spaces"), text: $text, axis: .vertical)
                    .lineLimit(4...12)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .textContentType(.oneTimeCode)
                    .accessibilityIdentifier("settings-extensions")
                    .onChange(of: text) { _, now in
                        let read = extensionsIn(text: now)
                        // The desktop's list again is kept as none: it follows the desktop's.
                        kept = read == defaultStoredExtensions() || read.isEmpty ? "" : read.joined(separator: " ")
                    }
                Button(tr("Reset to default")) {
                    text = defaultStoredExtensions().joined(separator: " ")
                }
                .disabled(kept.isEmpty)
                .accessibilityIdentifier("settings-extensions-reset")
            } footer: {
                Text(tr("Files whose names end in these, whatever their case"))
            }
            .listRowBackground(Tonal.surface)
        }
        .scrollContentBackground(.hidden)
        .background(Tonal.page)
        .navigationTitle(tr("What goes in as it is"))
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            text = (StoredExtensions.chosen ?? defaultStoredExtensions()).joined(separator: " ")
        }
    }
}

/// Settings › Checksums: the kinds a file's or an archive's checksums show,
/// as the Checksums sheet's own menu chooses them.
struct ChecksumsSection: View {
    @AppStorage("checksums") private var shownKeys = "sha256 sha1"
    private let kinds = checksumKinds()

    var body: some View {
        Section {
            ForEach(kinds, id: \.key) { row in
                Toggle(row.weak ? tr("{name} (weak)", ["name": row.name]) : row.name, isOn: chosen(row))
                    .accessibilityIdentifier("settings-checksum-\(row.key)")
            }
        } header: {
            Text(tr("Checksums"))
        }
        .listRowBackground(Tonal.surface)
    }

    private func chosen(_ row: ChecksumKindRow) -> Binding<Bool> {
        Binding {
            shownChecksums(shownKeys).contains { $0.kind == row.kind }
        } set: { on in
            var keys = shownChecksums(shownKeys).map(\.key)
            if on {
                keys.append(row.key)
            } else if keys.count > 1 {
                keys.removeAll { $0 == row.key }
            }
            // In the order they are shown.
            shownKeys = kinds.map(\.key).filter(keys.contains).joined(separator: " ")
        }
    }
}
