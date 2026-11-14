package com.hivepaas.packwing

// The app's words, in the user's language: the desktop's catalogs, through the
// core (crates/mobile/src/i18n.rs). The English is said here, with `{name}`
// for each value, so every platform shares one translation of a message.

/** `english` in the language in use, its `{name}`s filled from `args`. */
fun tr(english: String, vararg args: Pair<String, String>): String =
    uniffi.packwing_mobile.tr(english, args.toMap())

/** A message that says a count, `{n}`: English's two forms. */
fun trN(n: Long, one: String, other: String, vararg args: Pair<String, String>): String =
    uniffi.packwing_mobile.trN(n, one, other, args.toMap())

fun trN(n: Int, one: String, other: String, vararg args: Pair<String, String>): String =
    trN(n.toLong(), one, other, *args)

fun trN(n: ULong, one: String, other: String, vararg args: Pair<String, String>): String =
    trN(n.toLong(), one, other, *args)
