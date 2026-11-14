import QuickLook
import SwiftUI

/// A folder of an open archive: its folders and files, a search through all
/// of it, a file previewed, and Extract.
struct ArchiveView: View {
    @Environment(AppModel.self) private var model
    let place: Place
    @State private var query = ""
    @State private var entries: [EntryRow] = []
    @State private var found: [FoundRow] = []
    @State private var selecting = false
    @State private var picked = Set<[String]>()
    @State private var extracting: [[String]]?

    private var info: ArchiveInfo { place.archive.info }

    var body: some View {
        @Bindable var model = model
        List(selection: $picked) {
            if query.isEmpty {
                Section {
                    ForEach(entries, id: \.name) { entry in
                        row(entry, at: place.folder + [entry.name], showing: nil)
                    }
                } footer: {
                    if entries.isEmpty {
                        Text(tr("This folder is empty."))
                    } else if !selecting {
                        Text(tr("Tap a file to preview it."))
                    }
                }
            } else {
                Section {
                    ForEach(found, id: \.path) { hit in
                        row(hit.entry, at: hit.path, showing: hit.path.dropLast().joined(separator: " › "))
                    }
                } footer: {
                    Text(tr("Case and accents don’t matter: bao cao finds Báo cáo."))
                }
            }
        }
        .listStyle(.insetGrouped)
        .scrollContentBackground(.hidden)
        .background(Tonal.page)
        .environment(\.editMode, .constant(selecting ? .active : .inactive))
        // Its own bar at the bottom, Select and Extract, in the tabs' place.
        .toolbar(.hidden, for: .tabBar)
        .navigationTitle(place.folder.last ?? info.name)
        .navigationSubtitle(subtitle)
        .navigationBarTitleDisplayMode(.inline)
        .searchable(text: $query, prompt: trN(Int(info.files), "Search its {n} file", "Search all {n} files"))
        .toolbar {
            ToolbarItem(placement: .bottomBar) {
                Button(selecting ? tr("Done") : tr("Select")) {
                    selecting.toggle()
                    picked = []
                }
            }
            ToolbarSpacer(.flexible, placement: .bottomBar)
            ToolbarItem(placement: .bottomBar) {
                Button {
                    extracting = selecting ? Array(picked) : []
                } label: {
                    // Its words too, not only its icon, as a toolbar shows a Label.
                    HStack(spacing: 8) {
                        Image(systemName: "square.and.arrow.down")
                        Text(extractLabel).font(.nunito(16, .bold, relativeTo: .headline))
                    }
                    .padding(.horizontal, 8)
                }
                .buttonStyle(.glassProminent)
                .disabled(selecting && picked.isEmpty)
            }
        }
        .sheet(item: Binding(get: { extracting.map(Picked.init) }, set: { extracting = $0?.paths }),
               onDismiss: model.sheetGone) { what in
            ExtractSheet(archive: place.archive, picked: what.paths)
        }
        .quickLookPreview($model.previewing)
        .overlay(alignment: .bottom) {
            if let name = model.reading {
                BusyBanner(title: tr("Opening"), name: name) { model.stopReading() }
                    .padding(.bottom, 80)
            }
        }
        .task { entries = place.archive.archive.children(folder: place.folder) }
        .onChange(of: query) { _, now in
            found = now.isEmpty ? [] : place.archive.archive.search(query: now)
        }
    }

    private var extractLabel: String {
        selecting ? trN(picked.count, "Extract {n}", "Extract {n}") : tr("Extract all")
    }

    private var subtitle: String {
        let files = trN(Int(info.files), "{n} file", "{n} files")
        return "\(files) · \(sizeText(info.packed))"
    }

    @ViewBuilder
    private func row(_ entry: EntryRow, at path: [String], showing location: String?) -> some View {
        let kind = FileKind.of(name: entry.name, folder: entry.folder, link: entry.link)
        let label = EntryLabel(entry: entry, kind: kind, location: location)
        Group {
            if entry.folder && !selecting {
                NavigationLink(value: Place(archive: place.archive, folder: path)) { label }
            } else if selecting || entry.link {
                label
            } else {
                Button { model.preview(place.archive, path) } label: {
                    label.contentShape(.rect)
                }
                .buttonStyle(.plain)
            }
        }
        .tag(path)
        .listRowBackground(Tonal.surface)
        .contextMenu {
            if !entry.folder && !entry.link {
                Button(tr("Preview"), systemImage: "eye") { model.preview(place.archive, path) }
            }
            Button(tr("Extract"), systemImage: "square.and.arrow.down") { extracting = [path] }
        }
    }
}

/// What the Extract sheet is for: paths, none for all.
private struct Picked: Identifiable {
    let paths: [[String]]
    var id: [[String]] { paths }
}

struct EntryLabel: View {
    let entry: EntryRow
    let kind: FileKind
    /// Where it is in the archive, for what a search found.
    let location: String?

    var body: some View {
        HStack(spacing: 14) {
            KindTile(name: entry.name, kind: kind, size: 38)
            VStack(alignment: .leading, spacing: 1) {
                Text(entry.name)
                    .font(.nunito(16, .semibold, relativeTo: .body))
                    .foregroundStyle(Tonal.text)
                    .lineLimit(1)
                    .truncationMode(.middle)
                Text(meta)
                    .font(.nunito(13, relativeTo: .footnote))
                    .foregroundStyle(Tonal.muted)
                    .lineLimit(1)
            }
            Spacer(minLength: 0)
        }
        .padding(.vertical, 2)
    }

    private var meta: String {
        var parts: [String] = []
        if let location, !location.isEmpty { parts.append(location) }
        if entry.folder {
            parts.append(trN(Int(entry.files), "{n} file", "{n} files"))
        } else {
            parts.append(sizeText(entry.size))
        }
        if let modified = entry.modified {
            parts.append(Date(timeIntervalSince1970: TimeInterval(modified)).formatted(date: .abbreviated, time: .omitted))
        }
        return parts.joined(separator: " · ")
    }
}
