import SwiftUI

struct SettingsView: View {
    @Environment(AppModel.self) private var model
    @AppStorage("language") private var chosen: String = ""

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
