package com.hivepaas.packwing.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Folder
import androidx.compose.material.icons.outlined.Link
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

// What a file is, as far as its tag and color go: the desktop's
// (crates/app/src/model.rs), as the iOS app has it (ios/Packwing/Theme.swift).

/** A file's tag: its ground and the letters on it. */
data class Tint(val ground: Color, val ink: Color)

enum class FileKind {
    Folder, Pdf, Image, Document, Sheet, Text, Video, Program, Archive, Link, Other;

    fun tint(dark: Boolean): Tint = when (this) {
        Folder -> tint(dark, 0xffe9b8, 0x433712, 0x5c4200, 0xffe08a)
        Pdf -> tint(dark, 0xffdad6, 0x55211d, 0x7a1e16, 0xffdad6)
        Image, Document -> tint(dark, 0xd6e6ff, 0x1c3557, 0x123c73, 0xcfe2ff)
        Sheet, Archive -> tint(dark, 0xcdebdd, 0x1b4747, 0x0b3b2a, 0xb3eee4)
        Text, Other -> tint(dark, 0xd5e1da, 0x2b3540, 0x2c3a33, 0xcbd3db)
        Video -> tint(dark, 0xfbdaea, 0x4f2340, 0x6b1a48, 0xffd8ee)
        Program -> tint(dark, 0xffe1cc, 0x55391f, 0x5a2e0c, 0xffdcc2)
        Link -> tint(dark, 0xe8defc, 0x352a57, 0x3d2470, 0xe4dbff)
    }

    /** The letters on its tag: its extension, as the desktop shows it. */
    fun tag(name: String): String = when (this) {
        Folder -> "DIR"
        Link -> "LINK"
        else -> {
            val ext = extensionOf(if (this == Archive) withoutPartNumber(name) else name)
            if (ext.isNotEmpty() && ext.length <= 4) ext.uppercase() else "FILE"
        }
    }

    companion object {
        fun of(name: String, folder: Boolean = false, link: Boolean = false): FileKind {
            if (link) return Link
            if (folder) return Folder
            return when (extensionOf(withoutPartNumber(name))) {
                "pdf" -> Pdf
                "jpg", "jpeg", "png", "gif", "webp", "heic", "bmp", "tif", "tiff", "svg" -> Image
                "doc", "docx", "odt", "rtf", "pages", "ppt", "pptx", "odp", "key", "epub" -> Document
                "xls", "xlsx", "ods", "csv", "tsv", "numbers" -> Sheet
                "txt", "md", "markdown", "rst", "log", "json", "xml", "yaml", "yml", "toml", "ini",
                "conf", "cfg", "properties", "env", "html", "htm", "css", "py", "rb", "pl", "php",
                "go", "rs", "c", "h", "cc", "cpp", "hpp", "java", "kt", "swift", "ts", "tsx", "jsx",
                "sql", "srt", "vtt", "tex", "diff", "patch", "gitignore" -> Text
                "mp4", "m4v", "mkv", "mov", "avi", "webm", "wmv", "flv", "mpg", "mpeg" -> Video
                "exe", "msi", "dll", "app", "dmg", "pkg", "deb", "udeb", "rpm", "snap", "appimage",
                "sh", "bat", "cmd", "ps1", "js", "lnk", "jar", "apk" -> Program
                "zip", "tar", "gz", "tgz", "zst", "tzst", "xz", "txz", "bz2", "tbz2", "tbz", "lz",
                "lz4", "tlz4", "lzma", "tlz", "7z", "rar", "age", "iso", "cpio", "xar", "cbz",
                "cbr", "cb7", "squashfs" -> Archive
                else -> Other
            }
        }

        private fun tint(dark: Boolean, ground: Long, groundDark: Long, ink: Long, inkDark: Long) =
            Tint(rgb(if (dark) groundDark else ground), rgb(if (dark) inkDark else ink))
    }
}

private fun extensionOf(name: String): String {
    val dot = name.lastIndexOf('.')
    if (dot <= 0) return ""
    return name.substring(dot + 1).lowercase()
}

/** `a.7z.001` is a 7z: a split archive's part number is not its type. */
private fun withoutPartNumber(name: String): String {
    val dot = name.lastIndexOf('.')
    if (dot < 0) return name
    val number = name.substring(dot + 1)
    return if (number.length >= 3 && number.all { it in '0'..'9' }) name.substring(0, dot) else name
}

/** A file's tag: its extension on its tint. */
@Composable
fun KindTile(name: String, kind: FileKind, size: Dp = 42.dp, dark: Boolean = isDark()) {
    val tint = kind.tint(dark)
    Box(
        modifier = Modifier
            .size(size)
            .clip(RoundedCornerShape(size * 0.31f))
            .background(tint.ground)
            .clearAndSetSemantics {},
        contentAlignment = Alignment.Center,
    ) {
        when (kind) {
            FileKind.Folder -> Icon(
                Icons.Outlined.Folder, null,
                tint = tint.ink, modifier = Modifier.size(size * 0.5f),
            )
            FileKind.Link -> Icon(
                Icons.Outlined.Link, null,
                tint = tint.ink, modifier = Modifier.size(size * 0.5f),
            )
            else -> Text(
                kind.tag(name),
                color = tint.ink,
                fontSize = (size.value * 0.26f).sp,
                lineHeight = (size.value * 0.3f).sp,
                fontWeight = FontWeight.Bold,
                textAlign = TextAlign.Center,
                maxLines = 1,
            )
        }
    }
}

@Composable
fun isDark(): Boolean = tonal().page.luminance() < 0.5f

private fun Color.luminance(): Float = 0.2126f * red + 0.7152f * green + 0.0722f * blue

/** A size as the Files app says it: `12 MB`. */
fun sizeText(bytes: ULong): String {
    val b = bytes.toDouble()
    val units = listOf("B", "kB", "MB", "GB", "TB")
    var value = b
    var unit = 0
    while (value >= 1000 && unit < units.size - 1) {
        value /= 1000
        unit++
    }
    return when {
        unit == 0 -> "${bytes} B"
        value >= 100 -> "${value.toInt()} ${units[unit]}"
        value >= 10 -> String.format("%.1f %s", value, units[unit])
        else -> String.format("%.2f %s", value, units[unit])
    }
}
