import SwiftUI

/// What Extract asks before it starts: where to, and what becomes of it.
struct ExtractSheet: View {
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    let archive: OpenedArchive
    /// Paths in it; none for all of it.
    let picked: [[String]]
    @State private var toPhotos: Bool

    init(archive: OpenedArchive, picked: [[String]], toPhotos: Bool = false) {
        self.archive = archive
        self.picked = picked
        _toPhotos = State(initialValue: toPhotos)
    }
    @State private var choosing = false
    @AppStorage("showInFiles") private var showInFiles = false

    var body: some View {
        // Read, so that a folder chosen draws it again.
        let _ = model.foldersChanged
        let folder = model.extractFolder
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
                            Text(summary)
                                .font(.nunito(13, relativeTo: .footnote))
                        }
                    }
                    .foregroundStyle(Tonal.onTonal)
                    .listRowBackground(Tonal.tonal)
                }

                Section {
                    Picker(tr("Save to"), selection: $toPhotos) {
                        Text(tr("Files")).tag(false)
                        Text(tr("Photos")).tag(true)
                    }
                    .pickerStyle(.segmented)
                    .listRowBackground(Color.clear)
                    .listRowInsets(EdgeInsets())
                    if toPhotos {
                        Label(tr("The pictures and videos go to Photos. The other files stay in the archive."),
                              systemImage: "photo.on.rectangle")
                            .font(.nunito(14, relativeTo: .subheadline))
                            .foregroundStyle(Tonal.muted)
                    } else {
                        HStack(spacing: 12) {
                            Image(systemName: "folder")
                                .foregroundStyle(Tonal.accent)
                                .font(.system(size: 20))
                            VStack(alignment: .leading, spacing: 2) {
                                Text(model.shown(folder))
                                    .font(.nunito(15, .semibold, relativeTo: .subheadline))
                                Text(tr("In a new folder, {name}",
                                        ["name": AppModel.freeFolder(in: folder, named: stem(name: archive.info.name)).lastPathComponent]))
                                    .font(.nunito(13, relativeTo: .footnote))
                                    .foregroundStyle(Tonal.muted)
                            }
                            Spacer(minLength: 8)
                            Button(tr("Change")) { choosing = true }
                                .buttonStyle(.bordered)
                        }
                    }
                } header: {
                    Text(tr("Save to"))
                }
                .listRowBackground(Tonal.surface)

                if !toPhotos {
                    Section {
                        Toggle(tr("Show in Files when done"), isOn: $showInFiles)
                    } header: {
                        Text(tr("Options"))
                    } footer: {
                        Label(tr("Nothing is written outside this folder or through links, and an archive bomb stops at its size limit."),
                              systemImage: "checkmark.shield")
                            .font(.nunito(13, relativeTo: .footnote))
                    }
                    .listRowBackground(Tonal.surface)
                }
            }
            .scrollContentBackground(.hidden)
            .background(Tonal.page)
            .navigationTitle(tr("Extract"))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(role: .close) { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button(role: .confirm) { start(folder) }
                        .accessibilityLabel(tr("Extract now"))
                }
            }
            .safeAreaInset(edge: .bottom) {
                Button { start(folder) } label: {
                    Text(startLabel)
                        .font(.nunito(17, .bold, relativeTo: .headline))
                        .frame(maxWidth: .infinity, minHeight: 52)
                }
                .buttonStyle(.glassProminent)
                .padding(.horizontal, 20)
                .padding(.bottom, 8)
            }
            .fileImporter(isPresented: $choosing, allowedContentTypes: [.folder]) { result in
                if case .success(let url) = result { model.extractTo(url) }
            }
        }
        .presentationDetents([.large])
    }

    private var summary: String {
        if picked.isEmpty {
            let files = trN(Int(archive.info.files), "{n} file", "{n} files")
            return tr("{files} · {size} once extracted",
                      ["files": files, "size": sizeText(archive.info.unpacked)])
        }
        return trN(picked.count, "{n} item chosen", "{n} items chosen")
    }

    private var startLabel: String {
        if toPhotos { return tr("Save to Photos") }
        return picked.isEmpty
            ? trN(Int(archive.info.files), "Extract {n} file", "Extract {n} files")
            : trN(picked.count, "Extract {n} item", "Extract {n} items")
    }

    private func start(_ folder: URL) {
        let target: ExtractTarget = toPhotos ? .photos : .files(folder)
        model.afterSheet = { [model, archive, picked] in model.extract(archive, picked: picked, to: target) }
        dismiss()
    }
}
