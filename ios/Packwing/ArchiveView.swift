import SwiftUI

/// A folder of an open archive: its folders and files, and a search through
/// all of it.
struct ArchiveView: View {
    let place: Place
    @State private var query = ""
    @State private var entries: [EntryRow] = []
    @State private var found: [FoundRow] = []

    private var info: ArchiveInfo { place.archive.info }

    var body: some View {
        List {
            if query.isEmpty {
                Section {
                    ForEach(entries, id: \.name) { entry in
                        row(entry, at: place.folder + [entry.name], showing: nil)
                    }
                } footer: {
                    if entries.isEmpty {
                        Text(tr("This folder is empty."))
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
        .navigationTitle(place.folder.last ?? info.name)
        .navigationSubtitle(subtitle)
        .navigationBarTitleDisplayMode(.inline)
        .searchable(text: $query, prompt: trN(Int(info.files), "Search its {n} file", "Search all {n} files"))
        .task { entries = place.archive.archive.children(folder: place.folder) }
        .onChange(of: query) { _, now in
            found = now.isEmpty ? [] : place.archive.archive.search(query: now)
        }
    }

    private var subtitle: String {
        let files = trN(Int(info.files), "{n} file", "{n} files")
        return "\(files) · \(sizeText(info.packed))"
    }

    @ViewBuilder
    private func row(_ entry: EntryRow, at path: [String], showing location: String?) -> some View {
        let kind = FileKind.of(name: entry.name, folder: entry.folder, link: entry.link)
        let label = EntryLabel(entry: entry, kind: kind, location: location)
        if entry.folder {
            NavigationLink(value: Place(archive: place.archive, folder: path)) { label }
                .listRowBackground(Tonal.surface)
        } else {
            label.listRowBackground(Tonal.surface)
        }
    }
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
                    .lineLimit(1)
                    .truncationMode(.middle)
                Text(meta)
                    .font(.nunito(13, relativeTo: .footnote))
                    .foregroundStyle(Tonal.muted)
                    .lineLimit(1)
            }
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
