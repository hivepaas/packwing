import ImageIO
import SwiftUI
import UIKit

/// Whether `entry` is a picture the grid shows as itself.
func isPicture(_ entry: EntryRow) -> Bool {
    !entry.folder && !entry.link && entry.size <= Thumbnails.most
        && FileKind.of(name: entry.name) == .image
        // Drawn, not decoded: ImageIO does not read SVG.
        && !entry.name.lowercased().hasSuffix(".svg")
}

/// Whether a folder's grid is the way to show it: pictures, mostly.
func mostlyPictures(_ entries: [EntryRow]) -> Bool {
    !entries.isEmpty && entries.count(where: isPicture) * 2 > entries.count
}

/// The pictures of an archive's files, small, for its grid. An archive holds
/// no thumbnails, so each is read whole from it, once, and made small as it
/// is decoded (ImageIO decodes at the size asked, not the size it is). A few
/// at a time, and only for tiles still on screen: a tile gone stops its own.
actor Thumbnails {
    static let shared = Thumbnails()

    /// The largest picture read for a tile.
    static let most: UInt64 = 64 * 1024 * 1024
    /// How many are read and decoded at once.
    private static let atOnce = 3

    private let made = NSCache<NSString, UIImage>()
    /// Those that are not pictures packwing can show, not to try again.
    private var failed = Set<String>()
    private var running = 0
    private var waiting: [CheckedContinuation<Void, Never>] = []

    init() {
        made.totalCostLimit = 96 * 1024 * 1024
    }

    /// The file at `path` of `opened`, at most `edge` pixels across.
    func picture(of opened: OpenedArchive, at path: [String], edge: Int) async -> UIImage? {
        let key = "\(ObjectIdentifier(opened).hashValue)/\(path.joined(separator: "/"))" as NSString
        if let image = made.object(forKey: key) { return image }
        if failed.contains(key as String) { return nil }

        await enter()
        defer { leave() }
        if Task.isCancelled { return nil }

        let job = Job.quiet()
        let image = await withTaskCancellationHandler {
            await Task.detached(priority: .utility) {
                guard let bytes = try? opened.archive.preview(entry: path, most: Self.most, job: job) else {
                    return nil as UIImage?
                }
                return Self.small(bytes, edge: edge)
            }.value
        } onCancel: {
            job.cancel()
        }
        if let image {
            made.setObject(image, forKey: key, cost: Int(image.size.width * image.size.height * image.scale * image.scale * 4))
        } else if !Task.isCancelled {
            failed.insert(key as String)
        }
        return image
    }

    private func enter() async {
        if running < Self.atOnce {
            running += 1
            return
        }
        await withCheckedContinuation { waiting.append($0) }
    }

    private func leave() {
        if waiting.isEmpty {
            running -= 1
        } else {
            // Its place goes straight to the next.
            waiting.removeFirst().resume()
        }
    }

    /// `bytes` decoded no wider or taller than `edge`, turned as it was taken;
    /// nothing if it is not a picture, or one too big to decode safely.
    private static func small(_ bytes: Data, edge: Int) -> UIImage? {
        guard pictureFits(bytes: bytes),
              let source = CGImageSourceCreateWithData(bytes as CFData, nil)
        else { return nil }
        let options: [CFString: Any] = [
            kCGImageSourceCreateThumbnailFromImageAlways: true,
            kCGImageSourceThumbnailMaxPixelSize: edge,
            kCGImageSourceCreateThumbnailWithTransform: true,
            kCGImageSourceShouldCacheImmediately: true,
        ]
        guard let image = CGImageSourceCreateThumbnailAtIndex(source, 0, options as CFDictionary) else {
            return nil
        }
        return UIImage(cgImage: image)
    }
}

/// A folder of an archive as a grid: its pictures as themselves, its folders
/// and other files as tiles of their kind.
struct GalleryGrid: View {
    @Environment(AppModel.self) private var model
    let opened: OpenedArchive
    let folder: [String]
    let entries: [EntryRow]
    let selecting: Bool
    @Binding var picked: Set<[String]>
    let extract: ([String]) -> Void

    /// How many tiles a row has: three on an iPhone held upright, more as
    /// there is room for them (an iPad, an iPhone turned).
    @State private var across = 3

    /// The narrowest a tile is, but on the narrowest screens.
    private static let tile: CGFloat = 140

    var body: some View {
        let columns = Array(repeating: GridItem(.flexible(), spacing: 4), count: across)
        ScrollView {
            LazyVGrid(columns: columns, spacing: 4) {
                ForEach(Array(entries.enumerated()), id: \.element.name) { i, entry in
                    let path = folder + [entry.name]
                    tile(entry, at: path, corner: corner(i))
                }
            }
            .padding(.horizontal, 12)
            .padding(.top, 8)
            .padding(.bottom, 24)
        }
        .onGeometryChange(for: Int.self) { geometry in
            max(3, Int((geometry.size.width - 24 + 4) / (Self.tile + 4)))
        } action: { across = $0 }
        .background(Tonal.page)
    }

    /// The grid's own corners round with the screen's: its first row's ends.
    /// The rest are small.
    private func corner(_ i: Int) -> UnevenRoundedRectangle {
        let big: CGFloat = 22, small: CGFloat = 6
        return UnevenRoundedRectangle(
            topLeadingRadius: i == 0 ? big : small,
            bottomLeadingRadius: small,
            bottomTrailingRadius: small,
            topTrailingRadius: i == across - 1 ? big : small)
    }

    @ViewBuilder
    private func tile(_ entry: EntryRow, at path: [String], corner: UnevenRoundedRectangle) -> some View {
        let on = picked.contains(path)
        let face = Tile(opened: opened, entry: entry, path: path)
            // Chosen: washed lighter and framed, to tell at a glance on any picture.
            .overlay {
                if selecting && on {
                    corner.fill(Tonal.surface.opacity(0.28))
                    corner.strokeBorder(Tonal.accent, lineWidth: 3)
                }
            }
            .clipShape(corner)
            .overlay(alignment: .bottomTrailing) {
                if selecting {
                    Image(systemName: on ? "checkmark.circle.fill" : "circle")
                        .font(.system(size: 22, weight: .semibold))
                        .symbolRenderingMode(.palette)
                        .foregroundStyle(on ? Tonal.onAccent : Tonal.surface, on ? Tonal.accent : Color.black.opacity(0.25))
                        .background(Circle().fill(on ? Tonal.onAccent : .clear).padding(1))
                        .shadow(color: .black.opacity(0.3), radius: 2)
                        .padding(7)
                        .accessibilityHidden(true)
                }
            }
            .accessibilityAddTraits(on ? .isSelected : [])
        Group {
            if selecting {
                Button {
                    if on { picked.remove(path) } else { picked.insert(path) }
                } label: { face }
            } else if entry.folder {
                NavigationLink(value: Place(archive: opened, folder: path)) { face }
            } else if entry.link {
                face
            } else {
                Button { model.preview(opened, path) } label: { face }
            }
        }
        .buttonStyle(.plain)
        .contextMenu {
            if !entry.folder && !entry.link {
                Button(tr("Preview"), systemImage: "eye") { model.preview(opened, path) }
            }
            Button(tr("Extract"), systemImage: "square.and.arrow.down") { extract(path) }
        }
    }
}

/// One square of the grid.
private struct Tile: View {
    let opened: OpenedArchive
    let entry: EntryRow
    let path: [String]
    @State private var image: UIImage?
    @State private var width: CGFloat = 0
    @Environment(\.displayScale) private var scale

    var body: some View {
        let kind = FileKind.of(name: entry.name, folder: entry.folder, link: entry.link)
        Color.clear
            .aspectRatio(1, contentMode: .fit)
            .background(kind.tint.ground)
            .overlay {
                if let image {
                    Image(uiImage: image)
                        .resizable()
                        .scaledToFill()
                        .accessibilityLabel(entry.name)
                        .accessibilityAddTraits(.isImage)
                } else {
                    VStack(spacing: 6) {
                        KindTile(name: entry.name, kind: kind, size: 34,
                                 tint: Tint(ground: Tonal.surface.opacity(0.6), ink: kind.tint.ink))
                        Text(entry.name)
                            .font(.nunito(11, .semibold, relativeTo: .caption2))
                            .foregroundStyle(kind.tint.ink)
                            .lineLimit(2)
                            .multilineTextAlignment(.center)
                            .truncationMode(.middle)
                            .padding(.horizontal, 6)
                    }
                    .accessibilityElement(children: .combine)
                    .accessibilityLabel(entry.name)
                }
            }
            .clipped()
            .contentShape(.rect)
            // Shown as itself once read: an image, to VoiceOver and to the tests.
            .accessibilityAddTraits(image == nil ? [] : .isImage)
            .accessibilityIdentifier(image == nil ? "tile" : "picture")
            .onGeometryChange(for: CGFloat.self) { $0.size.width } action: { width = $0 }
            .task(id: width) {
                guard width > 0, isPicture(entry) else { return }
                // As many pixels across as the tile has.
                let edge = Int((width * scale).rounded(.up))
                image = await Thumbnails.shared.picture(of: opened, at: path, edge: edge)
            }
    }
}
