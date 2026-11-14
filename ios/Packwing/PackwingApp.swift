import SwiftUI

@main
struct PackwingApp: App {
    @State private var model = AppModel()

    init() {
        useNunitoInBars()
    }

    var body: some Scene {
        WindowGroup {
            RootView()
                .environment(model)
                .tint(Tonal.accent)
                .font(.nunito(17))
                // Drawn again in a language chosen in Settings.
                .id(model.language)
                // An archive opened in packwing from Files, Mail or another
                // app's Share sheet: read where it is.
                .onOpenURL { model.open($0) }
                #if DEBUG
                // `-open <path>` at launch: opens that archive at once.
                .task {
                    if let path = UserDefaults.standard.string(forKey: "open") {
                        model.open(URL(filePath: path))
                    }
                }
                #endif
        }
    }
}

struct RootView: View {
    @Environment(AppModel.self) private var model

    var body: some View {
        @Bindable var model = model
        TabView(selection: $model.tab) {
            Tab(tr("Archives"), systemImage: "archivebox", value: AppTab.archives) {
                HomeView()
            }
            Tab(tr("Settings"), systemImage: "slider.horizontal.3", value: AppTab.settings) {
                SettingsView()
            }
            Tab(value: AppTab.search, role: .search) {
                SearchView()
            }
        }
        .sheet(item: $model.asking) { ask in
            PasswordSheet(ask: ask)
        }
        .alert(
            tr("Can’t open it"),
            isPresented: Binding(get: { model.failure != nil }, set: { if !$0 { model.failure = nil } }),
            presenting: model.failure
        ) { _ in
            Button(tr("OK"), role: .cancel) {}
        } message: { failure in
            Text(failure)
        }
        .overlay(alignment: .bottom) {
            if let name = model.opening {
                OpeningBanner(name: name)
                    .padding(.bottom, 96)
                    .transition(.move(edge: .bottom).combined(with: .opacity))
            }
        }
        .animation(.default, value: model.opening)
    }
}

/// Said while an archive opens, as a big one takes a moment: with a way to stop.
struct OpeningBanner: View {
    @Environment(AppModel.self) private var model
    let name: String

    var body: some View {
        HStack(spacing: 12) {
            ProgressView()
            VStack(alignment: .leading, spacing: 1) {
                Text(tr("Opening"))
                    .font(.nunito(13, .medium, relativeTo: .caption))
                    .foregroundStyle(Tonal.muted)
                Text(name)
                    .font(.nunito(15, .semibold, relativeTo: .subheadline))
                    .lineLimit(1)
            }
            Spacer(minLength: 8)
            Button(tr("Stop")) { model.stopOpening() }
                .buttonStyle(.bordered)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .glassEffect(.regular, in: .capsule)
        .padding(.horizontal, 20)
    }
}
