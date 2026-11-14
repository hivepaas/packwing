import SwiftUI
import UIKit

// The Soft tonal design, as the desktop draws it (crates/app/src/colors.rs):
// each color by what it is for, light and dark; and the desktop's typeface,
// Nunito, which has every Vietnamese letter.

extension UIColor {
    convenience init(hex: UInt32) {
        self.init(
            red: CGFloat((hex >> 16) & 0xff) / 255,
            green: CGFloat((hex >> 8) & 0xff) / 255,
            blue: CGFloat(hex & 0xff) / 255,
            alpha: 1
        )
    }
}

enum Tonal {
    /// The screen's ground.
    static let page = dynamic(0xf3f7f4, 0x0e1318)
    /// Cards and rows on it.
    static let surface = dynamic(0xffffff, 0x161d24)
    /// Text fields, and quiet buttons.
    static let field = dynamic(0xe2ece6, 0x273038)
    static let outline = dynamic(0xb9c7bf, 0x36414c)
    static let text = dynamic(0x17211c, 0xe2e8ee)
    /// Hints, sizes, dates.
    static let muted = dynamic(0x46524b, 0xa8b3be)
    static let faint = dynamic(0x5c6861, 0x919ca7)
    static let accent = dynamic(0x1f6b4f, 0x7ad6c0)
    static let onAccent = dynamic(0xffffff, 0x00352b)
    /// The accent's soft fill, and the text on it.
    static let tonal = dynamic(0xcdebdd, 0x1b4747)
    static let onTonal = dynamic(0x0b3b2a, 0xb3eee4)
    /// Something to know before going on.
    static let warnBg = dynamic(0xffe1cc, 0x55391f)
    static let warnText = dynamic(0x5a2e0c, 0xffdcc2)
    /// Something went wrong.
    static let badBg = dynamic(0xffdad6, 0x55211d)
    static let badText = dynamic(0x7a1e16, 0xffdad6)

    static func dynamic(_ light: UInt32, _ dark: UInt32) -> Color {
        Color(UIColor { $0.userInterfaceStyle == .dark ? UIColor(hex: dark) : UIColor(hex: light) })
    }
}

/// A file's tag: its ground and the letters on it.
struct Tint {
    let ground: Color
    let ink: Color

    static let folder = Tint(ground: Tonal.dynamic(0xffe9b8, 0x433712), ink: Tonal.dynamic(0x5c4200, 0xffe08a))
    static let red = Tint(ground: Tonal.dynamic(0xffdad6, 0x55211d), ink: Tonal.dynamic(0x7a1e16, 0xffdad6))
    static let blue = Tint(ground: Tonal.dynamic(0xd6e6ff, 0x1c3557), ink: Tonal.dynamic(0x123c73, 0xcfe2ff))
    static let green = Tint(ground: Tonal.dynamic(0xcdebdd, 0x1b4747), ink: Tonal.dynamic(0x0b3b2a, 0xb3eee4))
    static let plain = Tint(ground: Tonal.dynamic(0xd5e1da, 0x2b3540), ink: Tonal.dynamic(0x2c3a33, 0xcbd3db))
    static let orange = Tint(ground: Tonal.dynamic(0xffe1cc, 0x55391f), ink: Tonal.dynamic(0x5a2e0c, 0xffdcc2))
    static let video = Tint(ground: Tonal.dynamic(0xfbdaea, 0x4f2340), ink: Tonal.dynamic(0x6b1a48, 0xffd8ee))
    static let link = Tint(ground: Tonal.dynamic(0xe8defc, 0x352a57), ink: Tonal.dynamic(0x3d2470, 0xe4dbff))
}

/// What a file is, as far as its tag and color go: the desktop's
/// (crates/app/src/model.rs).
enum FileKind {
    case folder, pdf, image, document, sheet, text, video, program, archive, link, other

    static func of(name: String, folder: Bool = false, link: Bool = false) -> FileKind {
        if link { return .link }
        if folder { return .folder }
        switch extensionOf(withoutPartNumber(name)) {
        case "pdf": return .pdf
        case "jpg", "jpeg", "png", "gif", "webp", "heic", "bmp", "tif", "tiff", "svg": return .image
        case "doc", "docx", "odt", "rtf", "pages", "ppt", "pptx", "odp", "key", "epub": return .document
        case "xls", "xlsx", "ods", "csv", "tsv", "numbers": return .sheet
        case "txt", "md", "markdown", "rst", "log", "json", "xml", "yaml", "yml", "toml", "ini", "conf",
             "cfg", "properties", "env", "html", "htm", "css", "py", "rb", "pl", "php", "go", "rs", "c",
             "h", "cc", "cpp", "hpp", "java", "kt", "swift", "ts", "tsx", "jsx", "sql", "srt", "vtt",
             "tex", "diff", "patch", "gitignore":
            return .text
        case "mp4", "m4v", "mkv", "mov", "avi", "webm", "wmv", "flv", "mpg", "mpeg": return .video
        case "exe", "msi", "dll", "app", "dmg", "pkg", "deb", "udeb", "rpm", "snap", "appimage", "sh",
             "bat", "cmd", "ps1", "js", "lnk", "jar", "apk":
            return .program
        case "zip", "tar", "gz", "tgz", "zst", "tzst", "xz", "txz", "bz2", "tbz2", "tbz", "lz", "lz4",
             "tlz4", "lzma", "tlz", "7z", "rar", "age", "iso", "cpio", "xar", "cbz", "cbr", "cb7", "squashfs":
            return .archive
        default: return .other
        }
    }

    var tint: Tint {
        switch self {
        case .folder: .folder
        case .pdf: .red
        case .image, .document: .blue
        case .sheet, .archive: .green
        case .text, .other: .plain
        case .video: .video
        case .program: .orange
        case .link: .link
        }
    }

    /// The letters on its tag: its extension, as the desktop shows it.
    func tag(of name: String) -> String {
        switch self {
        case .folder: return "DIR"
        case .link: return "LINK"
        default:
            let ext = extensionOf(self == .archive ? withoutPartNumber(name) : name)
            return !ext.isEmpty && ext.count <= 4 ? ext.uppercased() : "FILE"
        }
    }
}

private func extensionOf(_ name: String) -> String {
    guard let dot = name.lastIndex(of: "."), dot != name.startIndex else { return "" }
    return String(name[name.index(after: dot)...]).lowercased()
}

/// `a.7z.001` is a 7z: a split archive's part number is not its type.
private func withoutPartNumber(_ name: String) -> String {
    guard let dot = name.lastIndex(of: ".") else { return name }
    let number = name[name.index(after: dot)...]
    return number.count >= 3 && number.allSatisfy(\.isASCIIDigit) ? String(name[..<dot]) : name
}

private extension Character {
    var isASCIIDigit: Bool { isASCII && isNumber }
}

extension Font {
    /// Nunito at `size`, growing with Dynamic Type as `style` does.
    static func nunito(_ size: CGFloat, _ weight: Font.Weight = .regular, relativeTo style: Font.TextStyle = .body) -> Font {
        .custom(nunitoName(weight), size: size, relativeTo: style)
    }
}

func nunitoName(_ weight: Font.Weight) -> String {
    switch weight {
    case .bold, .heavy, .black: "Nunito-Bold"
    case .semibold: "Nunito-SemiBold"
    case .medium: "Nunito-Medium"
    default: "Nunito-Regular"
    }
}

/// The navigation bars' titles in Nunito, as the rest.
func useNunitoInBars() {
    let large = UIFont(name: "Nunito-Bold", size: 34) ?? .systemFont(ofSize: 34, weight: .bold)
    let inline = UIFont(name: "Nunito-Bold", size: 17) ?? .systemFont(ofSize: 17, weight: .bold)
    let bar = UINavigationBar.appearance()
    bar.largeTitleTextAttributes = [.font: UIFontMetrics(forTextStyle: .largeTitle).scaledFont(for: large)]
    bar.titleTextAttributes = [.font: UIFontMetrics(forTextStyle: .headline).scaledFont(for: inline)]
}

/// A size as Files says it: `12 MB`.
func sizeText(_ bytes: UInt64) -> String {
    Int64(clamping: bytes).formatted(.byteCount(style: .file).locale(appLocale))
}

/// A file's tag: its extension on its tint.
struct KindTile: View {
    let name: String
    let kind: FileKind
    var size: CGFloat = 42
    /// Its own colors, where its kind's would not show (on a ground as its own).
    var tint: Tint?

    var body: some View {
        Group {
            if kind == .folder {
                Image(systemName: "folder")
                    .font(.system(size: size * 0.45, weight: .semibold))
            } else {
                Text(kind.tag(of: name))
                    .font(.nunito(size * 0.26, .bold, relativeTo: .caption2))
                    .lineLimit(1)
                    .minimumScaleFactor(0.6)
                    .padding(.horizontal, 3)
            }
        }
        .foregroundStyle((tint ?? kind.tint).ink)
        .frame(width: size, height: size)
        .background((tint ?? kind.tint).ground, in: .rect(cornerRadius: size * 0.31))
        .accessibilityHidden(true)
    }
}

/// `code` (a checksum, a fingerprint), to break between its groups of eight
/// only: broken anywhere, iOS shows a hyphen that is not in it.
func breakable(_ code: String) -> String {
    stride(from: 0, to: code.count, by: 8).map { at in
        let start = code.index(code.startIndex, offsetBy: at)
        return String(code[start..<(code.index(start, offsetBy: 8, limitedBy: code.endIndex) ?? code.endIndex)])
    }.joined(separator: "\u{200B}")
}
