import SwiftUI
import UIKit

/// Checksums of an archive: of the files picked in it, of all of its files, or
/// of its own file, as asked first (the desktop's Checksums…). Of its files,
/// each one's are made in one read of the archive, and listed as `sha256sum`
/// writes them, to copy or keep beside it, where `sha256sum -c` checks what is
/// extracted with them.
struct ChecksumSheet: View {
    @Environment(\.dismiss) private var dismiss
    let archive: OpenedArchive
    /// Paths in it, picked: none, to ask what to read.
    let picked: [[String]]
    /// What the picked hold, to say it: files, and their bytes.
    let pickedFiles: UInt64
    let pickedBytes: UInt64?
    /// Of a file of it asked for alone: read at once, not asked.
    let atOnce: Bool

    private enum Scope: Hashable { case picked, all, file }

    @State private var scope: Scope
    @State private var reading: Scope?

    init(archive: OpenedArchive, picked: [[String]] = [], pickedFiles: UInt64 = 0, pickedBytes: UInt64? = nil,
         atOnce: Bool = false) {
        self.archive = archive
        self.picked = picked
        self.pickedFiles = pickedFiles
        self.pickedBytes = pickedBytes
        self.atOnce = atOnce
        _scope = State(initialValue: picked.isEmpty ? .all : .picked)
        _reading = State(initialValue: atOnce ? .picked : nil)
    }

    var body: some View {
        NavigationStack {
            Group {
                switch reading {
                case .file: ArchiveFileChecksums(archive: archive)
                case .picked: FilesChecksums(archive: archive, picked: picked)
                case .all: FilesChecksums(archive: archive, picked: [])
                case nil: choosing
                }
            }
            .navigationTitle(tr("Checksums"))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(role: .close) { dismiss() }
                }
                ToolbarItem(placement: .topBarTrailing) {
                    ChecksumsShownMenu()
                }
            }
        }
        .presentationDetents([.large])
        .presentationSizing(.page)
    }

    // MARK: What to read

    private var choosing: some View {
        let files = { (n: UInt64) in trN(Int(n), "{count} file", "{count} files", ["count": "\(n)"]) }
        let pickedTitle = picked.count == 1
            ? picked[0].last ?? ""
            : tr("{picked} selected", ["picked": "\(picked.count)"])
        let pickedDetail = picked.isEmpty
            ? tr("Select files or folders in it first.")
            : [files(pickedFiles), pickedBytes.map(sizeText)].compactMap { $0 }.joined(separator: " · ")
        return Form {
            Section {
                option(.picked, pickedTitle, pickedDetail, enabled: !picked.isEmpty)
                option(.all, tr("All {counted}", ["counted": files(archive.info.files)]),
                       sizeText(archive.info.unpacked))
                option(.file, tr("The archive itself"),
                       "\(archive.info.name) · \(sizeText(archive.info.packed))")
            } header: {
                Text(tr("What should it read?"))
            } footer: {
                if !archive.url.isFileURL {
                    Text(scope == .file
                         ? tr("All of the file passes through this computer, and your provider may charge for what is read from it.")
                         : tr("What is read of it passes through this computer, and your provider may charge for it."))
                }
            }
            .listRowBackground(Tonal.surface)
        }
        .scrollContentBackground(.hidden)
        .background(Tonal.page)
        .safeAreaInset(edge: .bottom) {
            Button { reading = scope } label: {
                Text(tr("Read it"))
                    .font(.nunito(17, .bold, relativeTo: .headline))
                    .frame(maxWidth: .infinity, minHeight: 52)
            }
            .buttonStyle(.glassProminent)
            .padding(.horizontal, 20)
            .padding(.bottom, 8)
            .accessibilityIdentifier("checksum-read")
        }
    }

    private func option(_ which: Scope, _ title: String, _ detail: String, enabled: Bool = true) -> some View {
        Button {
            scope = which
        } label: {
            HStack(spacing: 12) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(title)
                        .font(.nunito(16, .semibold, relativeTo: .body))
                        .lineLimit(1)
                        .truncationMode(.middle)
                    Text(detail)
                        .font(.nunito(13, relativeTo: .footnote))
                        .foregroundStyle(Tonal.muted)
                        .lineLimit(2)
                }
                Spacer(minLength: 8)
                Image(systemName: scope == which ? "checkmark.circle.fill" : "circle")
                    .font(.system(size: 22))
                    .foregroundStyle(scope == which ? Tonal.accent : Tonal.faint)
            }
            .foregroundStyle(Tonal.text)
            .contentShape(.rect)
        }
        .buttonStyle(.plain)
        .disabled(!enabled)
        .opacity(enabled ? 1 : 0.5)
        .accessibilityAddTraits(scope == which ? .isSelected : [])
    }
}

/// The checksums of an archive's files, all or those picked, in one read of it.
struct FilesChecksums: View {
    let archive: OpenedArchive
    let picked: [[String]]

    @AppStorage("checksums") private var shownKeys = "sha256 sha1"
    @State private var made: EntryChecksums?
    @State private var read: UInt64 = 0
    @State private var total: UInt64 = 0
    @State private var job: Job?
    @State private var problem: String?
    @State private var copiedAll = false
    @State private var saved: [String] = []
    @State private var choosingFolder = false
    /// The checksums given with the files, pasted: a list, or one alone.
    @State private var pasted = ""
    @State private var onlyMismatched = false
    @FocusState private var pasting: Bool

    private var shown: [ChecksumKindRow] { shownChecksums(shownKeys) }

    private var compared: PastedCompared? {
        guard let made else { return nil }
        return comparePasted(pasted: pasted, files: made.files, made: shown.map(\.kind))
    }

    /// What what was pasted says of the file at `index` of those made.
    private func verdict(_ index: Int) -> FileVerdict? {
        switch compared {
        case .listed(let verdicts, _): index < verdicts.count ? verdicts[index] : nil
        case .one(_, let found, _): found.first { Int($0.index) == index }.map { .matches(kind: $0.kind) }
        default: nil
        }
    }

    var body: some View {
        List {
            if let made, !made.files.isEmpty {
                compareSection(made)
            }
            if let made {
                if made.files.count == 1, let file = made.files.first {
                    // One file alone: its checksums at once.
                    Section {
                        ForEach(shown, id: \.key) { row in
                            SumRow(row: row, sum: file.sums.first { $0.kind == row.kind }?.sum)
                        }
                    } header: {
                        Text("\(file.name) · \(sizeText(file.size))")
                    }
                    .listRowBackground(Tonal.surface)
                } else if !made.files.isEmpty {
                    Section {
                        ForEach(Array(made.files.enumerated()), id: \.element.name) { i, file in
                            if !onlyMismatched || !matches(verdict(i)) {
                                NavigationLink {
                                    EntrySumsView(file: file, shown: shown)
                                } label: {
                                    fileRow(file, verdict(i))
                                }
                                .accessibilityIdentifier("checksum-file")
                            }
                        }
                    } footer: {
                        Text(tr("Tap a file to see its checksums."))
                    }
                    .listRowBackground(Tonal.surface)
                } else {
                    Text(tr("There are no files in it to read."))
                        .listRowBackground(Tonal.surface)
                }
                notes(made)
            } else if let problem {
                Label(problem, systemImage: "exclamationmark.triangle")
                    .foregroundStyle(Tonal.warnText)
                    .listRowBackground(Tonal.warnBg)
            } else {
                VStack(alignment: .leading, spacing: 8) {
                    ProgressView(value: Double(min(read, max(total, 1))), total: Double(max(total, 1)))
                    Text(total > 0
                         ? tr("Reading… {size} of {size2}", ["size": sizeText(read), "size2": sizeText(total)])
                         : tr("Reading… {size}", ["size": sizeText(read)]))
                        .font(.nunito(13, relativeTo: .footnote))
                        .foregroundStyle(Tonal.muted)
                }
                .padding(.vertical, 6)
                .listRowBackground(Tonal.surface)
            }
        }
        .listStyle(.insetGrouped)
        .scrollContentBackground(.hidden)
        .background(Tonal.page)
        .scrollDismissesKeyboard(.interactively)
        .toolbar {
            ToolbarItemGroup(placement: .keyboard) {
                Spacer()
                Button(tr("Done")) { pasting = false }
                    .accessibilityIdentifier("checksums-pasted-done")
            }
        }
        .safeAreaInset(edge: .bottom) {
            // Not over the files as a list is pasted.
            if let made, !made.files.isEmpty, !pasting {
                HStack(spacing: 10) {
                    Button(action: copyAll) {
                        Label(copiedAll ? tr("Copied") : tr("Copy all"), systemImage: copiedAll ? "checkmark" : "doc.on.doc")
                            .frame(maxWidth: .infinity, minHeight: 44)
                    }
                    .buttonStyle(.glass)
                    Button {
                        if besideWritable { save(in: archive.url.deletingLastPathComponent()) } else { choosingFolder = true }
                    } label: {
                        Label(besideWritable ? tr("Save beside it") : tr("Save in…"), systemImage: "square.and.arrow.down")
                            .frame(maxWidth: .infinity, minHeight: 44)
                    }
                    .buttonStyle(.glassProminent)
                    .accessibilityIdentifier("checksum-save")
                }
                .font(.nunito(16, .semibold, relativeTo: .body))
                .padding(.horizontal, 20)
                .padding(.bottom, 8)
            }
        }
        .fileImporter(isPresented: $choosingFolder, allowedContentTypes: [.folder]) { result in
            if case .success(let folder) = result { save(in: folder) }
        }
        .task(id: shownKeys) { await readThem() }
        .onDisappear { job?.cancel() }
    }

    private func fileRow(_ file: EntryChecksum, _ verdict: FileVerdict?) -> some View {
        let first = shown.first.flatMap { row in file.sums.first { $0.kind == row.kind }?.sum } ?? ""
        return HStack(spacing: 10) {
            if let verdict { mark(verdict) }
            VStack(alignment: .leading, spacing: 2) {
                Text(file.name)
                    .font(.nunito(15, .semibold, relativeTo: .body))
                    .lineLimit(1)
                    .truncationMode(.middle)
                Text("\(sizeText(file.size)) · \(first.prefix(16))…")
                    .font(.system(.caption, design: .monospaced))
                    .foregroundStyle(Tonal.muted)
            }
        }
    }

    private func matches(_ verdict: FileVerdict?) -> Bool {
        if case .matches = verdict { return true }
        return false
    }

    /// What the list says of a file, as a mark beside it.
    @ViewBuilder
    private func mark(_ verdict: FileVerdict) -> some View {
        switch verdict {
        case .matches:
            Image(systemName: "checkmark.circle.fill").foregroundStyle(Tonal.accent)
                .accessibilityLabel(tr("Matches"))
        case .differs(let made, _) where !made.isEmpty:
            Image(systemName: "xmark.octagon.fill").foregroundStyle(Tonal.badText)
                .accessibilityLabel(tr("Differs"))
        case .differs, .unknown:
            Image(systemName: "questionmark.circle").foregroundStyle(Tonal.muted)
        case .notListed:
            Image(systemName: "minus.circle").foregroundStyle(Tonal.faint)
                .accessibilityLabel(tr("The list does not name it."))
        }
    }

    // MARK: Comparing

    /// The checksums given with the files, pasted, and what they say: above
    /// the files, in sight of the keyboard.
    private func compareSection(_ made: EntryChecksums) -> some View {
        Section {
            HStack(alignment: .top, spacing: 8) {
                TextField(tr("Paste the checksums given with the files, or one of them"), text: $pasted, axis: .vertical)
                    .lineLimit(2...5)
                    .font(.system(.footnote, design: .monospaced))
                    .autocorrectionDisabled()
                    .textInputAutocapitalization(.never)
                    .focused($pasting)
                    .accessibilityIdentifier("checksums-pasted")
                Button {
                    if let text = UIPasteboard.general.string { pasted = text }
                } label: {
                    Image(systemName: "doc.on.clipboard")
                }
                .buttonStyle(.borderless)
                .accessibilityLabel(tr("Paste"))
            }
            if let compared {
                said(compared, made)
            }
        } header: {
            Text(tr("Compare with"))
        }
        .listRowBackground(Tonal.surface)
    }

    @ViewBuilder
    private func said(_ compared: PastedCompared, _ made: EntryChecksums) -> some View {
        switch compared {
        case .notOne:
            note(tr("There is no checksum in this."), "exclamationmark.triangle", Tonal.warnText, Tonal.warnBg)
        case .one(let digits, let found, let notMade):
            if checksumKinds().allSatisfy({ !kindOf(digits: digits, $0) }) {
                note(tr("This is not a checksum packwing makes: it has {digits} digits.", ["digits": "\(digits)"]),
                     "exclamationmark.triangle", Tonal.warnText, Tonal.warnBg)
            } else if found.count == 1, let one = found.first {
                note(tr("It matches the {name} checksum of {file}.",
                        ["name": checksumNames([one.kind]), "file": made.files[Int(one.index)].name]),
                     "checkmark.seal.fill", Tonal.onTonal, Tonal.tonal)
                    .accessibilityIdentifier("checksum-one-matches")
            } else if !found.isEmpty {
                note(trN(found.count, "It matches {count} file: {names}", "It matches {count} files: {names}",
                         ["count": "\(found.count)",
                          "names": found.prefix(3).map { made.files[Int($0.index)].name }.joined(separator: ", ")]),
                     "checkmark.seal.fill", Tonal.onTonal, Tonal.tonal)
            } else if shown.contains(where: { kindOf(digits: digits, $0) }) {
                note(tr("No file in it has this checksum."), "xmark.octagon.fill", Tonal.badText, Tonal.badBg)
                    .accessibilityIdentifier("checksum-none-match")
            }
            if found.isEmpty && !notMade.isEmpty {
                HStack(spacing: 12) {
                    Text(tr("It may be a {name} checksum, which is not made yet.", ["name": checksumNames(notMade)]))
                        .font(.nunito(14, relativeTo: .subheadline))
                        .foregroundStyle(Tonal.muted)
                    Spacer(minLength: 8)
                    Button(tr("Make it")) { show(notMade) }
                        .buttonStyle(.bordered)
                }
            }
        case .listed(let verdicts, let notFound):
            let count = { (want: (FileVerdict) -> Bool) in verdicts.filter(want).count }
            let matching = count { if case .matches = $0 { true } else { false } }
            let differing = count { if case .differs(let made, _) = $0 { !made.isEmpty } else { false } }
            let unlisted = count { if case .notListed = $0 { true } else { false } }
            if notFound.isEmpty && matching == verdicts.count {
                note(tr("Every file matches the list."), "checkmark.seal.fill", Tonal.onTonal, Tonal.tonal)
                    .accessibilityIdentifier("checksum-all-match")
            } else {
                let parts = [trN(matching, "{count} file matches", "{count} files match", ["count": "\(matching)"])]
                    + (differing > 0 ? [trN(differing, "{count} file differs", "{count} files differ", ["count": "\(differing)"])] : [])
                    + (unlisted > 0 ? [trN(unlisted, "{count} file is not in the list", "{count} files are not in the list",
                                           ["count": "\(unlisted)"])] : [])
                let line = parts.joined(separator: " · ")
                if differing > 0 {
                    note(line, "xmark.octagon.fill", Tonal.badText, Tonal.badBg)
                        .accessibilityIdentifier("checksum-differs")
                } else {
                    Text(line)
                        .font(.nunito(14, relativeTo: .subheadline))
                        .foregroundStyle(Tonal.muted)
                }
                if !notFound.isEmpty {
                    Text(trN(notFound.count, "{count} line names a file it does not hold: {names}",
                             "{count} lines name files it does not hold: {names}",
                             ["count": "\(notFound.count)", "names": notFound.prefix(3).joined(separator: ", ")]))
                        .font(.nunito(14, relativeTo: .subheadline))
                        .foregroundStyle(Tonal.muted)
                }
                Toggle(tr("Show only those that do not match"), isOn: $onlyMismatched)
                    .font(.nunito(15, relativeTo: .body))
            }
        }
    }

    private func note(_ text: String, _ icon: String, _ ink: Color, _ ground: Color) -> some View {
        Label(text, systemImage: icon)
            .font(.nunito(14, .semibold, relativeTo: .subheadline))
            .foregroundStyle(ink)
            .listRowBackground(ground)
    }

    /// Whether `row` makes checksums `digits` long.
    private func kindOf(digits: UInt32, _ row: ChecksumKindRow) -> Bool {
        let long: [ChecksumKind: UInt32] = [.md5: 32, .sha1: 40, .sha256: 64, .blake3: 64, .sha512: 128]
        return long[row.kind] == digits
    }

    /// Shows `kinds` too, and so reads them.
    private func show(_ kinds: [ChecksumKind]) {
        let keys = Set(shown.map(\.key) + checksumKinds().filter { kinds.contains($0.kind) }.map(\.key))
        shownKeys = checksumKinds().map(\.key).filter(keys.contains).joined(separator: " ")
    }

    @ViewBuilder
    private func notes(_ made: EntryChecksums) -> some View {
        let weak = shown.filter(\.weak)
        if !made.damaged.isEmpty || made.links > 0 || !weak.isEmpty || !saved.isEmpty {
            Section {
                VStack(alignment: .leading, spacing: 8) {
                    if !made.damaged.isEmpty {
                        Text(trN(made.damaged.count, "{count} file is damaged, and has none: {names}",
                                 "{count} files are damaged, and have none: {names}",
                                 ["count": "\(made.damaged.count)", "names": made.damaged.joined(separator: ", ")]))
                            .foregroundStyle(Tonal.warnText)
                    }
                    if made.links > 0 {
                        Text(tr("Links have none of their own: a checking tool reads what they lead to."))
                    }
                    if !weak.isEmpty {
                        Text(tr("A weak checksum ({names}) still tells a damaged file, but not one changed on purpose, which can be made to match.",
                                ["names": checksumNames(weak.map(\.kind))]))
                    }
                    if !saved.isEmpty {
                        Label(tr("Saved as {files}. Where they are extracted, sha256sum -c and the like check them with it, without packwing.",
                                 ["files": saved.joined(separator: ", ")]),
                              systemImage: "checkmark.circle")
                            .foregroundStyle(Tonal.onTonal)
                            .accessibilityIdentifier("checksum-saved")
                    }
                }
                .font(.nunito(13, relativeTo: .footnote))
                .foregroundStyle(Tonal.muted)
            }
            .listRowBackground(Color.clear)
        }
    }

    /// Whether the lists go beside the archive: a file packwing may write beside.
    private var besideWritable: Bool {
        archive.url.isFileURL
            && FileManager.default.isWritableFile(atPath: archive.url.deletingLastPathComponent().path(percentEncoded: false))
    }

    private func readThem() async {
        job?.cancel()
        made = nil
        problem = nil
        saved = []
        read = 0
        let job = Job(listener: ReadListener { bytes, all in
            read = bytes
            total = all
        })
        self.job = job
        let kinds = shown.map(\.kind)
        let archive = archive.archive
        let picked = picked
        let result = await Task.detached(priority: .userInitiated) {
            Result { try archive.entryChecksums(picked: picked, kinds: kinds, job: job) }
        }.value
        guard self.job === job else { return }
        self.job = nil
        switch result {
        case .success(let now): made = now
        case .failure(let error):
            if case .Cancelled = error as? MobileError { return }
            problem = tr("The file could not be read: {why}",
                         ["why": (error as? MobileError).map(reason) ?? error.localizedDescription])
        }
    }

    /// The list of the first checksum shown, as `sha256sum` writes it.
    private func copyAll() {
        guard let made, let kind = shown.first?.kind else { return }
        UIPasteboard.general.string = checksumList(files: made.files, kind: kind)
        copiedAll = true
        Task {
            try? await Task.sleep(for: .seconds(2))
            copiedAll = false
        }
    }

    /// A list for each checksum shown, in `folder`: `a.zip.contents.sha256`.
    private func save(in folder: URL) {
        guard let made else { return }
        let scoped = folder.startAccessingSecurityScopedResource()
        defer { if scoped { folder.stopAccessingSecurityScopedResource() } }
        var kept: [String] = []
        do {
            for row in shown {
                let name = checksumListName(name: archive.info.name, kind: row.kind)
                try checksumList(files: made.files, kind: row.kind)
                    .write(to: folder.appending(path: name), atomically: true, encoding: .utf8)
                kept.append(name)
            }
            saved = kept
        } catch {
            problem = tr("They could not be saved: {why}", ["why": error.localizedDescription])
        }
    }
}

/// All of a file's checksums, each to copy.
private struct EntrySumsView: View {
    let file: EntryChecksum
    let shown: [ChecksumKindRow]

    var body: some View {
        List {
            Section {
                ForEach(shown, id: \.key) { row in
                    SumRow(row: row, sum: file.sums.first { $0.kind == row.kind }?.sum)
                }
            } header: {
                Text("\(file.name) · \(sizeText(file.size))")
            }
            .listRowBackground(Tonal.surface)
        }
        .listStyle(.insetGrouped)
        .scrollContentBackground(.hidden)
        .background(Tonal.page)
        .navigationTitle(file.path.last ?? file.name)
        .navigationBarTitleDisplayMode(.inline)
    }
}
