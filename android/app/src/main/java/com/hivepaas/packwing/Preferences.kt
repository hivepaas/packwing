package com.hivepaas.packwing

import com.hivepaas.packwing.ui.sizeText

// The desktop's settings, as the phone keeps them (Settings), each by the key
// the desktop and the iOS app name it by: how packwing looks, where extracted
// files go, and what a new archive is made with.

/** Light, dark, or as the system is. */
enum class Appearance(val key: String) {
    System("system"),
    Light("light"),
    Dark("dark"),
    ;

    val label: String
        get() = when (this) {
            System -> tr("System")
            Light -> tr("Light")
            Dark -> tr("Dark")
        }

    companion object {
        /** The one kept as `key`; as the system is if there is none. */
        fun of(key: String?): Appearance = entries.firstOrNull { it.key == key } ?: System
    }
}

/** Where the files of an archive go, in the folder extracting writes to. */
enum class ExtractPlace(val key: String) {
    /** Right in the folder if what goes is one thing, else a new folder. */
    Smart("smart"),

    /** A new folder, named after the archive. */
    NewFolder("new-folder"),

    /** Straight into the folder. */
    Here("here"),
    ;

    val label: String
        get() = when (this) {
            Smart -> tr("Smart")
            NewFolder -> tr("In a new folder")
            Here -> tr("Straight into the folder")
        }

    val detail: String
        get() = when (this) {
            Smart -> tr("One item right here, more in a new folder")
            NewFolder -> tr("Named after the archive")
            Here -> tr("No folder of its own")
        }

    /** Whether `going` items go in a new folder. */
    fun newFolder(going: Int): Boolean = when (this) {
        Smart -> going != 1
        NewFolder -> true
        Here -> false
    }

    companion object {
        /** The one kept as `key`; Smart, as on the desktop, if there is none. */
        fun of(key: String?): ExtractPlace = entries.firstOrNull { it.key == key } ?: Smart
    }
}

/**
 * Above how many bytes a new archive bound for a storage is written straight
 * there, as the desktop offers it; kept by its key.
 */
class StreamAbove private constructor(val key: String, val bytes: ULong) {
    val label: String get() = if (bytes == 0UL) tr("Always") else sizeText(bytes)

    companion object {
        val all = listOf(
            StreamAbove("always", 0UL),
            StreamAbove("100mb", 100_000_000UL),
            StreamAbove("500mb", 500_000_000UL),
            StreamAbove("1gb", 1_000_000_000UL),
            StreamAbove("10gb", 10_000_000_000UL),
        )

        /** 500 MB, as on the desktop. */
        val standard = all[2]

        /** The one kept as `key`; 500 MB if there is none. */
        fun of(key: String?): StreamAbove = all.firstOrNull { it.key == key } ?: standard
    }
}

/** The most an archive may unpack to, as the desktop offers it; 0 for no limit. */
class ExtractLimit private constructor(val key: String, val bytes: ULong, private val named: String?) {
    // Said again each time: in the language in use.
    val label: String get() = named ?: tr("No limit")

    companion object {
        val all = listOf(
            ExtractLimit("10gb", 10_000_000_000UL, "10 GB"),
            ExtractLimit("100gb", 100_000_000_000UL, "100 GB"),
            ExtractLimit("1tb", 1_000_000_000_000UL, "1 TB"),
            ExtractLimit("none", 0UL, null),
        )

        /** 100 GB, as on the desktop. */
        val standard = all[1]

        /** The one kept as `key`; 100 GB if there is none. */
        fun of(key: String?): ExtractLimit = all.firstOrNull { it.key == key } ?: standard
    }
}
