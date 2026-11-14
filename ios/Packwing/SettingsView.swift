import SwiftUI

struct SettingsView: View {
    @Environment(AppModel.self) private var model
    @AppStorage("language") private var chosen: String = ""
    @AppStorage("newFormat") private var formatKey = ArchiveFormat.zip.key
    @AppStorage("newCompression") private var levelKey = CompressionLevel.medium.key

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
                    Picker(tr("Format"), selection: $formatKey) {
                        ForEach(ArchiveFormat.all, id: \.key) { Text($0.label).tag($0.key) }
                    }
                    Picker(tr("Compression"), selection: $levelKey) {
                        ForEach(CompressionLevel.all, id: \.key) { Text($0.label).tag($0.key) }
                    }
                }
                .listRowBackground(Tonal.surface)
                TrustedKeysSection()
                Section(tr("About")) {
                    LabeledContent("packwing", value: version)
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
