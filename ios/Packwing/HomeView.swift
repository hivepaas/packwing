import SwiftUI
import UniformTypeIdentifiers

/// The Archives tab: opening one, and those opened before.
struct HomeView: View {
    @Environment(AppModel.self) private var model
    @State private var importing = false
    @State private var adding: NewSource?

    var body: some View {
        @Bindable var model = model
        NavigationStack(path: $model.path) {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    OpenCard { importing = true }
                    if model.recent.isEmpty {
                        ShareTip()
                    } else {
                        RecentSection(items: model.recent)
                    }
                }
                .padding(.horizontal, 16)
                .padding(.bottom, 24)
            }
            .background(Tonal.page)
            .navigationTitle(tr("Archives"))
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    AddMenu(source: $adding) {
                        Image(systemName: "plus")
                    }
                    .accessibilityLabel(tr("New archive"))
                }
            }
            .choosingNewItems(
                $adding,
                chosen: { model.drafting = NewArchiveDraft($0) },
                photos: { picked in
                    let draft = NewArchiveDraft()
                    draft.add(picked)
                    model.drafting = draft
                })
            .navigationDestination(for: Place.self) { place in
                ArchiveView(place: place)
            }
        }
        // Any file: packwing tells an archive by what is in it, not its name.
        .fileImporter(isPresented: $importing, allowedContentTypes: [.data]) { result in
            if case .success(let url) = result { model.open(url) }
        }
    }
}

/// The big button: an archive from Files.
struct OpenCard: View {
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 16) {
                Image(systemName: "folder")
                    .font(.system(size: 24, weight: .semibold))
                    .foregroundStyle(Tonal.onAccent)
                    .frame(width: 56, height: 56)
                    .background(Tonal.accent, in: .rect(cornerRadius: 18))
                VStack(alignment: .leading, spacing: 2) {
                    Text(tr("Open an archive"))
                        .font(.nunito(19, .bold, relativeTo: .title3))
                    Text(tr("From Files, iCloud Drive or a USB drive"))
                        .font(.nunito(14, relativeTo: .subheadline))
                }
                Spacer(minLength: 0)
            }
            .foregroundStyle(Tonal.onTonal)
            .padding(20)
            .background(Tonal.tonal, in: .rect(cornerRadius: 28))
        }
        .buttonStyle(.plain)
    }
}

/// How to open an attachment without saving it.
struct ShareTip: View {
    var body: some View {
        Text(tr("A ZIP or RAR in Mail or Messages? Tap Share, then packwing, to look inside without saving it."))
            .font(.nunito(14, relativeTo: .subheadline))
            .foregroundStyle(Tonal.warnText)
            .padding(.horizontal, 18)
            .padding(.vertical, 14)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Tonal.warnBg, in: .rect(cornerRadius: 22))
    }
}

struct RecentSection: View {
    @Environment(AppModel.self) private var model
    let items: [RecentArchive]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(tr("Recent"))
                .font(.nunito(20, .bold, relativeTo: .title3))
                .padding(.horizontal, 4)
                .accessibilityAddTraits(.isHeader)
            VStack(spacing: 0) {
                ForEach(Array(items.enumerated()), id: \.element.id) { i, item in
                    if i > 0 {
                        Divider().overlay(Tonal.field).padding(.leading, 72)
                    }
                    RecentRow(item: item)
                }
            }
            .background(Tonal.surface, in: .rect(cornerRadius: 26))
        }
    }
}

struct RecentRow: View {
    @Environment(AppModel.self) private var model
    let item: RecentArchive

    var body: some View {
        Button {
            model.open(item)
        } label: {
            HStack(spacing: 14) {
                KindTile(name: item.name, kind: .of(name: item.name))
                VStack(alignment: .leading, spacing: 2) {
                    Text(item.name)
                        .font(.nunito(16, .semibold, relativeTo: .body))
                        .lineLimit(1)
                        .truncationMode(.middle)
                    HStack(spacing: 5) {
                        if item.locked {
                            Image(systemName: "lock.fill").font(.system(size: 11))
                                .accessibilityLabel(tr("Locked"))
                        }
                        Text(meta)
                    }
                    .font(.nunito(13, relativeTo: .footnote))
                    .foregroundStyle(Tonal.muted)
                }
                Spacer(minLength: 0)
                Image(systemName: "chevron.right")
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(Tonal.faint)
            }
            .foregroundStyle(Tonal.text)
            .padding(.vertical, 12)
            .padding(.leading, 16)
            .padding(.trailing, 14)
            .contentShape(.rect)
        }
        .buttonStyle(.plain)
        .contextMenu {
            Button(tr("Remove from Recent"), systemImage: "minus.circle", role: .destructive) {
                model.forget(item)
            }
        }
    }

    private var meta: String {
        let files = trN(Int(item.files), "{n} file", "{n} files")
        let when = item.opened.formatted(.relative(presentation: .named).locale(appLocale))
        return "\(files) · \(sizeText(item.size)) · \(when)"
    }
}

/// The Search tab: the archives opened before, by name.
struct SearchView: View {
    @Environment(AppModel.self) private var model
    @State private var query = ""

    var body: some View {
        NavigationStack {
            ScrollView {
                let found = model.recent.filter {
                    query.isEmpty || $0.name.localizedStandardContains(query)
                }
                if found.isEmpty {
                    ContentUnavailableView.search(text: query)
                        .padding(.top, 80)
                } else {
                    RecentSection(items: found).padding(16)
                }
            }
            .background(Tonal.page)
            .navigationTitle(tr("Search"))
            .searchable(text: $query, prompt: tr("Archives opened before"))
        }
    }
}
