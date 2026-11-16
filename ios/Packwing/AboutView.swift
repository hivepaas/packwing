import SwiftUI

/// Settings › About, as the desktop has it: what packwing is and which build
/// this is, that the App Store keeps it up to date, where its code is, and the
/// other apps from HivePaaS.
struct AboutView: View {
    @State private var copied = false

    private static let source = URL(string: "https://github.com/hivepaas/packwing")!
    private static let license = URL(string: "https://github.com/hivepaas/packwing/blob/main/LICENSE")!
    private static let issues = URL(string: "https://github.com/hivepaas/packwing/issues/new")!
    private static let releases = URL(string: "https://github.com/hivepaas/packwing/releases")!
    /// The year in the copyright line.
    private static let year = "2026"

    private var version: String {
        Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? ""
    }

    private var system: String { "iOS \(UIDevice.current.systemVersion)" }

    /// What a problem report needs to say about this build, as the desktop says it.
    private var versionInfo: String { "packwing \(version) (\(system) arm64)" }

    var body: some View {
        Form {
            Section {
                identity
            }
            .listRowBackground(Tonal.surface)

            Section {
                Text(tr("The App Store keeps packwing up to date."))
                    .font(.nunito(15, relativeTo: .subheadline))
                    .foregroundStyle(Tonal.muted)
            } header: {
                Text(tr("Updates"))
            }
            .listRowBackground(Tonal.surface)

            Section {
                linkRow(tr("Free software"), tr("Under the GNU General Public License, version 3 or later"),
                        tr("View license"), Self.license)
                linkRow(tr("Made by HivePaaS"), tr("On hivepack, our library for reading and writing archives"),
                        tr("Source code"), Self.source)
                linkRow(tr("Something wrong?"), tr("Tell us, with the version above"),
                        tr("Report a problem"), Self.issues)
                linkRow(tr("What changed"), tr("Every release, and what is new in it"),
                        tr("Release notes"), Self.releases)
            }
            .listRowBackground(Tonal.surface)

            Section {
                ForEach(OtherApp.all) { app in
                    OtherAppCard(app: app)
                }
            } header: {
                Text(tr("More from HivePaaS"))
            } footer: {
                Text(tr("Free and open source, like packwing."))
            }
            .listRowBackground(Tonal.surface)

            Section {
            } footer: {
                Text(tr("© {year} HivePaaS. The packwing name and logo are trademarks of HivePaaS.",
                        ["year": Self.year]))
            }
        }
        .scrollContentBackground(.hidden)
        .background(Tonal.page)
        .navigationTitle(tr("About"))
        .navigationBarTitleDisplayMode(.inline)
    }

    /// The logo, the name, what it is and which build this is.
    private var identity: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack(spacing: 16) {
                PackwingLogo(size: 72)
                VStack(alignment: .leading, spacing: 2) {
                    Text("packwing")
                        .font(.nunito(28, .bold, relativeTo: .title))
                    Text(tr("Version {version} · {os} {arch}",
                            ["version": version, "os": system, "arch": "arm64"]))
                        .font(.nunito(14, relativeTo: .subheadline))
                        .foregroundStyle(Tonal.muted)
                        .accessibilityIdentifier("about-version")
                }
            }
            Text(tr("A file archiver that puts every core to work and opens archives safely."))
                .font(.nunito(15, relativeTo: .body))
                .fixedSize(horizontal: false, vertical: true)
            Button(copied ? tr("Copied") : tr("Copy version info"), systemImage: "doc.on.doc") {
                UIPasteboard.general.string = versionInfo
                copied = true
                Task {
                    try? await Task.sleep(for: .seconds(2))
                    copied = false
                }
            }
            .buttonStyle(.bordered)
            .accessibilityIdentifier("about-copy-version")
        }
        .padding(.vertical, 8)
    }

    /// A row that says what something is, and opens its page.
    private func linkRow(_ title: String, _ detail: String, _ action: String, _ url: URL) -> some View {
        Link(destination: url) {
            HStack(spacing: 12) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(title)
                        .font(.nunito(16, .semibold, relativeTo: .body))
                        .foregroundStyle(Tonal.text)
                    Text(detail)
                        .font(.nunito(13, relativeTo: .footnote))
                        .foregroundStyle(Tonal.muted)
                }
                Spacer(minLength: 8)
                Text(action)
                    .font(.nunito(14, .semibold, relativeTo: .subheadline))
                    .foregroundStyle(Tonal.accent)
                    .multilineTextAlignment(.trailing)
            }
        }
    }
}

/// packwing's logo on its accent tile, as the desktop draws it: the zipper in
/// the tile's ink, and the feather in its own colour.
struct PackwingLogo: View {
    let size: CGFloat

    var body: some View {
        ZStack {
            Image("LogoZipper").resizable().foregroundStyle(Tonal.onAccent)
            Image("LogoFeather").resizable().foregroundStyle(Tonal.dynamic(0xffe1cc, 0x55391f))
        }
        .frame(width: size / 2, height: size / 2)
        .frame(width: size, height: size)
        .background(Tonal.accent, in: .rect(cornerRadius: size / 3))
        .accessibilityHidden(true)
    }
}

/// An app from HivePaaS: its mark, name, what it is, its strong points, and
/// where to see it.
private struct OtherApp: Identifiable {
    let id: String
    let what: String
    let points: [String]
    let site: (label: String, url: URL)
    let source: URL

    // Said again each time: in the language in use.
    static var all: [OtherApp] {
        [
            OtherApp(
                id: "shellwing",
                what: tr("A fast, native SSH client for Linux, macOS and Windows."),
                points: [tr("SSH and SFTP"), tr("Follows your ssh_config"), tr("A real terminal"),
                         tr("GPL-3.0-or-later")],
                site: (tr("Visit shellwing.app"), URL(string: "https://shellwing.app")!),
                source: URL(string: "https://github.com/hivepaas/shellwing")!),
            OtherApp(
                id: "tracewing",
                what: tr("A native HTTP(S) debugging proxy that keeps up with big sessions."),
                points: [tr("HTTP/2, WebSocket and SSE"), tr("A million requests, smooth"),
                         tr("Breakpoints and rewrites"), tr("Phones and emulators"), tr("GPL-3.0-or-later")],
                site: (tr("Visit tracewing.app"), URL(string: "https://tracewing.app")!),
                source: URL(string: "https://github.com/hivepaas/tracewing")!),
            OtherApp(
                id: "HivePaaS",
                what: tr("A self-hosted platform for your apps, built on Docker Swarm: an open-source alternative to Heroku, Render and Coolify."),
                points: [tr("300+ one-click apps"), tr("Deploy on push"), tr("Pull request previews"),
                         tr("Automatic TLS"), tr("Live logs and metrics"), tr("Apache-2.0")],
                site: (tr("Visit hivepaas.com"), URL(string: "https://hivepaas.com")!),
                source: URL(string: "https://github.com/hivepaas/hivepaas")!),
        ]
    }
}

private struct OtherAppCard: View {
    let app: OtherApp

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 14) {
                AppMark(id: app.id)
                VStack(alignment: .leading, spacing: 2) {
                    Text(app.id).font(.nunito(18, .bold, relativeTo: .headline))
                    Text(app.what)
                        .font(.nunito(13, relativeTo: .footnote))
                        .foregroundStyle(Tonal.muted)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
            FlowLayout(spacing: 6) {
                ForEach(app.points, id: \.self) { point in
                    Text(point)
                        .font(.nunito(13, .semibold, relativeTo: .footnote))
                        .padding(.horizontal, 12)
                        .padding(.vertical, 6)
                        .background(Tonal.field, in: .capsule)
                }
            }
            HStack(spacing: 8) {
                Link(app.site.label, destination: app.site.url)
                    .buttonStyle(.borderedProminent)
                    .accessibilityIdentifier("about-\(app.id.lowercased())")
                Link("GitHub", destination: app.source)
                    .buttonStyle(.bordered)
            }
            .font(.nunito(14, .semibold, relativeTo: .subheadline))
        }
        .padding(.vertical, 8)
        .accessibilityElement(children: .contain)
    }
}

/// Each app's mark on its own tile, as the desktop draws them, in either look.
private struct AppMark: View {
    let id: String

    var body: some View {
        Group {
            switch id {
            case "shellwing":
                Image("ShellwingMark").resizable()
                    .foregroundStyle(Color(red: 0x02 / 255, green: 0x84 / 255, blue: 0xc7 / 255))
                    .frame(width: 36, height: 36)
                    .tile(Color(red: 0xee / 255, green: 0xf1 / 255, blue: 0xf5 / 255))
            case "tracewing":
                Image("TracewingMark").resizable()
                    .foregroundStyle(.white)
                    .frame(width: 30, height: 30)
                    .tile(Color(red: 0x2f / 255, green: 0x55 / 255, blue: 0xb4 / 255))
            default:
                HiveMark().frame(width: 36, height: 36)
                    .tile(Color(red: 0xfb / 255, green: 0xf7 / 255, blue: 0xf0 / 255))
            }
        }
        .accessibilityHidden(true)
    }
}

/// HivePaaS's mark: seven cells of a hive in its brown and amber.
private struct HiveMark: View {
    private static let brown = Color(red: 0x54 / 255, green: 0x3d / 255, blue: 0x2b / 255)
    private static let amber = Color(red: 0xf5 / 255, green: 0x9e / 255, blue: 0x0b / 255)
    /// Centers in the mark's 36-unit box.
    private static let cells: [(CGFloat, CGFloat, Bool)] = [
        (12.98, 9.30, true), (23.02, 9.30, false),
        (7.95, 18.0, true), (18.0, 18.0, false), (28.05, 18.0, false),
        (12.98, 26.70, true), (23.02, 26.70, false),
    ]

    var body: some View {
        Canvas { context, size in
            let unit = size.width / 36
            for (x, y, brown) in Self.cells {
                let r = 4.3 * unit
                let rect = CGRect(x: x * unit - r, y: y * unit - r, width: 2 * r, height: 2 * r)
                context.fill(Path(ellipseIn: rect), with: .color(brown ? Self.brown : Self.amber))
            }
        }
    }
}

private extension View {
    /// On a logo's tile, of its own ground.
    func tile(_ ground: Color) -> some View {
        frame(width: 56, height: 56).background(ground, in: .rect(cornerRadius: 16))
    }
}

/// Views side by side, wrapped onto the next line where they do not fit.
struct FlowLayout: Layout {
    var spacing: CGFloat = 8

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let width = proposal.width ?? .infinity
        let rows = arrange(subviews, width: width)
        let height = rows.last.map { $0.y + $0.height } ?? 0
        let used = rows.map(\.width).max() ?? 0
        return CGSize(width: proposal.width ?? used, height: height)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        for row in arrange(subviews, width: bounds.width) {
            var x = bounds.minX
            for index in row.items {
                let size = subviews[index].sizeThatFits(.unspecified)
                subviews[index].place(at: CGPoint(x: x, y: bounds.minY + row.y), proposal: ProposedViewSize(size))
                x += size.width + spacing
            }
        }
    }

    private struct Row {
        var items: [Int] = []
        var y: CGFloat = 0
        var width: CGFloat = 0
        var height: CGFloat = 0
    }

    private func arrange(_ subviews: Subviews, width: CGFloat) -> [Row] {
        var rows: [Row] = []
        var row = Row()
        for index in subviews.indices {
            let size = subviews[index].sizeThatFits(.unspecified)
            let needed = row.items.isEmpty ? size.width : row.width + spacing + size.width
            if needed > width, !row.items.isEmpty {
                let next = row.y + row.height + spacing
                rows.append(row)
                row = Row(y: next)
            }
            row.width = row.items.isEmpty ? size.width : row.width + spacing + size.width
            row.height = max(row.height, size.height)
            row.items.append(index)
        }
        if !row.items.isEmpty { rows.append(row) }
        return rows
    }
}
