import SwiftUI
import UIKit

/// An archive's own checksums, to compare with the one given where it came
/// from: those chosen are made as it shows, all of the file read once for
/// them, and one pasted says whether it matches. A checksum says a file is
/// the one it was made of, not who made it.
struct ArchiveFileChecksums: View {
    let archive: OpenedArchive

    /// The checksums shown, by their keys: the desktop's first two.
    @AppStorage("checksums") private var shownKeys = "sha256 sha1"
    /// Made so far, in the order they are shown.
    @State private var made: [Checksum] = []
    /// Being made, and how much of the file is read for them.
    @State private var making: [ChecksumKind] = []
    @State private var read: UInt64 = 0
    @State private var job: Job?
    @State private var problem: String?
    @State private var pasted = ""

    private static let kinds = checksumKinds()

    private var shown: [ChecksumKindRow] { shownChecksums(shownKeys) }

    /// Those chosen, and any made besides, asked for by a checksum pasted.
    private var rows: [ChecksumKindRow] {
        Self.kinds.filter { row in
            shown.contains { $0.kind == row.kind } || made.contains { $0.kind == row.kind }
        }
    }

    /// Its file's: on a storage, as the listing says it.
    private var size: UInt64 {
        archive.url.isFileURL
            ? UInt64((try? archive.url.resourceValues(forKeys: [.fileSizeKey]).fileSize) ?? 0)
            : archive.info.packed
    }

    var body: some View {
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
                        Text(sizeText(size))
                            .font(.nunito(13, relativeTo: .footnote))
                    }
                }
                .foregroundStyle(Tonal.onTonal)
                .listRowBackground(Tonal.tonal)
            }

            // First: in sight above the keyboard as one is pasted, and
            // what it says too.
            Section {
                HStack(alignment: .top, spacing: 8) {
                    TextField(tr("Paste the checksum given with the file"), text: $pasted, axis: .vertical)
                        .lineLimit(2...5)
                        .font(.system(.footnote, design: .monospaced))
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)
                        .accessibilityIdentifier("checksum-pasted")
                    Button {
                        if let text = UIPasteboard.general.string { pasted = text }
                    } label: {
                        Image(systemName: "doc.on.clipboard")
                    }
                    .buttonStyle(.borderless)
                    .accessibilityLabel(tr("Paste"))
                }
                if let compared {
                    said(compared)
                }
            } header: {
                Text(tr("Compare with"))
            }
            .listRowBackground(Tonal.surface)

            Section {
                ForEach(rows, id: \.key) { row in
                    sumRow(row)
                }
            } footer: {
                footer
            }
            .listRowBackground(Tonal.surface)

            if let problem {
                Section {
                    Label(problem, systemImage: "exclamationmark.triangle")
                        .font(.nunito(14, relativeTo: .subheadline))
                        .foregroundStyle(Tonal.warnText)
                        .listRowBackground(Tonal.warnBg)
                }
            }
        }
        .scrollContentBackground(.hidden)
        .scrollDismissesKeyboard(.interactively)
        .background(Tonal.page)
        .task { make(shown.map(\.kind)) }
        .onChange(of: shownKeys) { make(shown.map(\.kind)) }
        .onDisappear { job?.cancel() }
    }

    // MARK: The checksums

    private func sumRow(_ row: ChecksumKindRow) -> some View {
        SumRow(row: row, sum: made.first { $0.kind == row.kind }?.sum)
    }

    @ViewBuilder
    private var footer: some View {
        VStack(alignment: .leading, spacing: 8) {
            if job != nil {
                ProgressView(value: Double(min(read, size)), total: Double(max(size, 1)))
                Text(size > 0
                     ? tr("Reading… {size} of {size2}", ["size": sizeText(read), "size2": sizeText(size)])
                     : tr("Reading… {size}", ["size": sizeText(read)]))
            }
            let weak = rows.filter(\.weak)
            if !weak.isEmpty {
                Text(tr("A weak checksum ({names}) still tells a damaged file, but not one changed on purpose, which can be made to match.",
                        ["names": names(weak.map(\.kind))]))
            }
        }
        .font(.nunito(13, relativeTo: .footnote))
    }

    /// Makes those of `kinds` not made yet, unless some are being made: the
    /// rest are made after them.
    private func make(_ kinds: [ChecksumKind]) {
        guard job == nil else { return }
        let wanted = kinds.filter { kind in !made.contains { $0.kind == kind } }
        guard !wanted.isEmpty else { return }
        problem = nil
        making = wanted
        read = 0
        let job = Job(listener: ReadListener { read, _ in self.read = read })
        self.job = job
        let archive = archive.archive
        Task {
            let result = await Task.detached(priority: .userInitiated) {
                Result { try archive.checksums(kinds: wanted, job: job) }
            }.value
            self.job = nil
            making = []
            switch result {
            case .success(let sums):
                let all = made + sums
                made = Self.kinds.compactMap { row in all.first { $0.kind == row.kind } }
                // Any chosen meanwhile.
                make(shown.map(\.kind))
            case .failure(let error):
                switch error as? MobileError {
                case .Cancelled:
                    break
                case .NotAnArchive(let why), .WrongPassword(let why), .Failed(let why), .Locked(let why, _, _):
                    problem = tr("The file could not be read: {why}", ["why": why])
                case nil:
                    problem = tr("The file could not be read: {why}", ["why": error.localizedDescription])
                }
            }
        }
    }

    // MARK: Comparing

    private var compared: Compared? {
        compareChecksum(pasted: pasted, name: archive.info.name, made: made, making: making)
    }

    /// What the checksum pasted says, as a note.
    @ViewBuilder
    private func said(_ compared: Compared) -> some View {
        switch compared {
        case .notOne:
            note(tr("There is no checksum in this."), "exclamationmark.triangle", Tonal.warnText, Tonal.warnBg)
        case .unknown(let digits):
            note(tr("This is not a checksum packwing makes: it has {digits} digits.", ["digits": "\(digits)"]),
                 "exclamationmark.triangle", Tonal.warnText, Tonal.warnBg)
        case .matches(let kind):
            note(tr("It matches the file’s {name} checksum.", ["name": names([kind])]),
                 "checkmark.seal.fill", Tonal.onTonal, Tonal.tonal)
                .accessibilityIdentifier("checksum-matches")
        case .waiting:
            Text(tr("It is compared once the file is read."))
                .font(.nunito(14, relativeTo: .subheadline))
                .foregroundStyle(Tonal.muted)
        case .differs(let made, let notMade):
            if !made.isEmpty {
                note(tr("It does not match the file’s {name} checksum: this is not the file it was made of, or the file changed on the way.",
                        ["name": names(made)]),
                     "xmark.octagon.fill", Tonal.badText, Tonal.badBg)
                    .accessibilityIdentifier("checksum-differs")
            }
            if !notMade.isEmpty {
                HStack(spacing: 12) {
                    Text(tr("It may be a {name} checksum, which is not made yet.", ["name": names(notMade)]))
                        .font(.nunito(14, relativeTo: .subheadline))
                        .foregroundStyle(Tonal.muted)
                    Spacer(minLength: 8)
                    Button(tr("Make it")) { make(notMade) }
                        .buttonStyle(.bordered)
                }
            }
        }
    }

    private func note(_ text: String, _ icon: String, _ ink: Color, _ ground: Color) -> some View {
        Label(text, systemImage: icon)
            .font(.nunito(14, .semibold, relativeTo: .subheadline))
            .foregroundStyle(ink)
            .listRowBackground(ground)
    }

    /// Their names, apart by slashes: names, not words to translate.
    private func names(_ kinds: [ChecksumKind]) -> String {
        checksumNames(kinds)
    }
}

/// Told by the core how much is read, and of how much, off the main thread.
final class ReadListener: JobListener, @unchecked Sendable {
    let told: @MainActor (UInt64, UInt64) -> Void

    init(told: @escaping @MainActor (UInt64, UInt64) -> Void) {
        self.told = told
    }

    func progress(progress: JobProgress) {
        let bytes = progress.bytes
        let total = progress.totalBytes
        Task { @MainActor in told(bytes, total) }
    }
}

// MARK: Shared by both

/// The checksums shown, by the keys `keys` keeps (Settings, the menu); never none.
func shownChecksums(_ keys: String) -> [ChecksumKindRow] {
    let all = checksumKinds()
    let kept = Set(keys.split(separator: " ").map(String.init))
    let chosen = all.filter { kept.contains($0.key) }
    return chosen.isEmpty ? [all[0]] : chosen
}

/// Their names, apart by slashes: names, not words to translate.
func checksumNames(_ kinds: [ChecksumKind]) -> String {
    let all = checksumKinds()
    return kinds.compactMap { kind in all.first { $0.kind == kind }?.name }.joined(separator: " / ")
}

/// The menu of the checksums shown: each on or off, the last one staying.
struct ChecksumsShownMenu: View {
    @AppStorage("checksums") private var shownKeys = "sha256 sha1"
    private let kinds = checksumKinds()

    var body: some View {
        Menu {
            ForEach(kinds, id: \.key) { row in
                Toggle(isOn: chosen(row)) {
                    Text(row.weak ? tr("{name} (weak)", ["name": row.name]) : row.name)
                }
            }
        } label: {
            Image(systemName: "slider.horizontal.3")
        }
        .accessibilityLabel(tr("Choose the checksums shown"))
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

/// A checksum's row: its name, its digits, and Copy.
struct SumRow: View {
    let row: ChecksumKindRow
    let sum: String?
    @State private var copied = false

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 8) {
                Text(row.name)
                    .font(.nunito(15, .bold, relativeTo: .subheadline))
                if row.weak {
                    Text(tr("weak"))
                        .font(.nunito(12, .semibold, relativeTo: .caption))
                        .foregroundStyle(Tonal.warnText)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 2)
                        .background(Tonal.warnBg, in: .capsule)
                }
                Spacer(minLength: 8)
                if let sum {
                    Button {
                        UIPasteboard.general.string = sum
                        copied = true
                        Task {
                            try? await Task.sleep(for: .seconds(2))
                            copied = false
                        }
                    } label: {
                        Label(copied ? tr("Copied") : tr("Copy"), systemImage: copied ? "checkmark" : "doc.on.doc")
                            .font(.nunito(14, .semibold, relativeTo: .subheadline))
                    }
                    .buttonStyle(.borderless)
                }
            }
            // Copy copies it whole.
            Text(sum.map(breakable) ?? tr("Reading…"))
                .font(.system(.footnote, design: .monospaced))
                .foregroundStyle(sum == nil ? Tonal.muted : Tonal.text)
                .accessibilityLabel(sum ?? tr("Reading…"))
                .accessibilityIdentifier("checksum-\(row.key)")
        }
        .padding(.vertical, 4)
    }
}
