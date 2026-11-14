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
                    // `-new <path>|<path>…`: New archive, of those.
                    if let paths = UserDefaults.standard.string(forKey: "new") {
                        model.drafting = NewArchiveDraft(
                            paths.split(separator: "|").map { NewItem(url: URL(filePath: String($0)), scoped: false) })
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
            // Named here: iOS's own name for it is in the system's language.
            Tab(tr("Search"), systemImage: "magnifyingglass", value: AppTab.search, role: .search) {
                SearchView()
            }
        }
        .sheet(item: $model.asking) { ask in
            PasswordSheet(ask: ask)
        }
        .sheet(item: $model.drafting, onDismiss: model.sheetGone) { draft in
            CreateSheet(draft: draft)
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
        .fullScreenCover(isPresented: $model.showingJob) {
            if let job = model.job { JobScreen(job: job) }
        }
        .overlay(alignment: .bottom) {
            VStack(spacing: 10) {
                if let job = model.job, job.state == .running, !model.showingJob {
                    JobBanner(job: job)
                }
                if let name = model.opening {
                    BusyBanner(title: tr("Opening"), name: name) { model.stopOpening() }
                }
            }
            .padding(.bottom, 96)
            .transition(.move(edge: .bottom).combined(with: .opacity))
        }
        .animation(.default, value: model.opening)
    }
}

/// Said while something is read, as a big one takes a moment: with a way to stop.
struct BusyBanner: View {
    let title: String
    let name: String
    let stop: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            ProgressView()
            VStack(alignment: .leading, spacing: 1) {
                Text(title)
                    .font(.nunito(13, .medium, relativeTo: .caption))
                    .foregroundStyle(Tonal.muted)
                Text(name)
                    .font(.nunito(15, .semibold, relativeTo: .subheadline))
                    .lineLimit(1)
            }
            Spacer(minLength: 8)
            Button(tr("Stop"), action: stop)
                .buttonStyle(.bordered)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .glassEffect(.regular, in: .capsule)
        .padding(.horizontal, 20)
    }
}
