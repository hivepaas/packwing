package com.hivepaas.packwing.ui

import android.content.res.AssetManager
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight

// The Soft tonal design, as the desktop draws it (crates/app/src/colors.rs)
// and the iOS app has it (ios/Packwing/Theme.swift): each color by what it is
// for, light and dark. Material's own roles are filled from these below, so a
// Material component and a hand-drawn one are the same color.

/** Every color the app draws with, by what it is for. */
data class Tonal(
    /** The screen's ground. */
    val page: Color,
    /** Cards and rows on it. */
    val surface: Color,
    /** Text fields, and quiet buttons. */
    val field: Color,
    val outline: Color,
    val text: Color,
    /** Hints, sizes, dates. */
    val muted: Color,
    val faint: Color,
    val accent: Color,
    val onAccent: Color,
    /** The accent's soft fill, and the text on it. */
    val tonal: Color,
    val onTonal: Color,
    /** The navigation bar, and footers. */
    val chrome: Color,
    /** Something to know before going on. */
    val warnBg: Color,
    val warnText: Color,
    /** Something went wrong. */
    val badBg: Color,
    val badText: Color,
)

private val light = Tonal(
    page = rgb(0xf3f7f4),
    surface = rgb(0xffffff),
    field = rgb(0xe2ece6),
    outline = rgb(0xb9c7bf),
    text = rgb(0x17211c),
    muted = rgb(0x46524b),
    faint = rgb(0x5c6861),
    accent = rgb(0x1f6b4f),
    onAccent = rgb(0xffffff),
    tonal = rgb(0xcdebdd),
    onTonal = rgb(0x0b3b2a),
    chrome = rgb(0xe8f0eb),
    warnBg = rgb(0xffe1cc),
    warnText = rgb(0x5a2e0c),
    badBg = rgb(0xffdad6),
    badText = rgb(0x7a1e16),
)

private val dark = Tonal(
    page = rgb(0x0e1318),
    surface = rgb(0x161d24),
    field = rgb(0x273038),
    outline = rgb(0x36414c),
    text = rgb(0xe2e8ee),
    muted = rgb(0xa8b3be),
    faint = rgb(0x919ca7),
    accent = rgb(0x7ad6c0),
    onAccent = rgb(0x00352b),
    tonal = rgb(0x1b4747),
    onTonal = rgb(0xb3eee4),
    chrome = rgb(0x121a21),
    warnBg = rgb(0x55391f),
    warnText = rgb(0xffdcc2),
    badBg = rgb(0x55211d),
    badText = rgb(0xffdad6),
)

/** `0xrrggbb`, opaque. */
fun rgb(hex: Long) = Color(0xff000000 or hex)

/** The palette in use: `tonal().accent`, never a value. */
val LocalTonal = staticCompositionLocalOf { light }

@Composable
fun tonal(): Tonal = LocalTonal.current

/** Nunito, the desktop's typeface, which has every Vietnamese letter. */
private fun nunito(assets: AssetManager) = FontFamily(
    Font("Nunito-Regular.ttf", assets, FontWeight.Normal),
    Font("Nunito-Medium.ttf", assets, FontWeight.Medium),
    Font("Nunito-SemiBold.ttf", assets, FontWeight.SemiBold),
    Font("Nunito-Bold.ttf", assets, FontWeight.Bold),
)

@Composable
fun PackwingTheme(darkTheme: Boolean = isSystemInDarkTheme(), content: @Composable () -> Unit) {
    val c = if (darkTheme) dark else light
    val assets = LocalContext.current.assets
    val typography = remember(assets) {
        val family = nunito(assets)
        Typography().let { t ->
            Typography(
                displayLarge = t.displayLarge.copy(fontFamily = family),
                displayMedium = t.displayMedium.copy(fontFamily = family),
                displaySmall = t.displaySmall.copy(fontFamily = family),
                headlineLarge = t.headlineLarge.copy(fontFamily = family),
                headlineMedium = t.headlineMedium.copy(fontFamily = family),
                headlineSmall = t.headlineSmall.copy(fontFamily = family),
                titleLarge = t.titleLarge.copy(fontFamily = family, fontWeight = FontWeight.Bold),
                titleMedium = t.titleMedium.copy(fontFamily = family, fontWeight = FontWeight.SemiBold),
                titleSmall = t.titleSmall.copy(fontFamily = family, fontWeight = FontWeight.SemiBold),
                bodyLarge = t.bodyLarge.copy(fontFamily = family),
                bodyMedium = t.bodyMedium.copy(fontFamily = family),
                bodySmall = t.bodySmall.copy(fontFamily = family),
                labelLarge = t.labelLarge.copy(fontFamily = family, fontWeight = FontWeight.SemiBold),
                labelMedium = t.labelMedium.copy(fontFamily = family, fontWeight = FontWeight.Medium),
                labelSmall = t.labelSmall.copy(fontFamily = family, fontWeight = FontWeight.Medium),
            )
        }
    }
    val scheme = if (darkTheme) {
        darkColorScheme(
            primary = c.accent, onPrimary = c.onAccent,
            primaryContainer = c.tonal, onPrimaryContainer = c.onTonal,
            secondary = c.accent, onSecondary = c.onAccent,
            secondaryContainer = c.tonal, onSecondaryContainer = c.onTonal,
            tertiaryContainer = c.warnBg, onTertiaryContainer = c.warnText,
            background = c.page, onBackground = c.text,
            surface = c.page, onSurface = c.text,
            surfaceVariant = c.field, onSurfaceVariant = c.muted,
            surfaceContainerLowest = c.page, surfaceContainerLow = c.surface,
            surfaceContainer = c.chrome, surfaceContainerHigh = c.surface,
            surfaceContainerHighest = c.field,
            outline = c.faint, outlineVariant = c.outline,
            errorContainer = c.badBg, onErrorContainer = c.badText,
        )
    } else {
        lightColorScheme(
            primary = c.accent, onPrimary = c.onAccent,
            primaryContainer = c.tonal, onPrimaryContainer = c.onTonal,
            secondary = c.accent, onSecondary = c.onAccent,
            secondaryContainer = c.tonal, onSecondaryContainer = c.onTonal,
            tertiaryContainer = c.warnBg, onTertiaryContainer = c.warnText,
            background = c.page, onBackground = c.text,
            surface = c.page, onSurface = c.text,
            surfaceVariant = c.field, onSurfaceVariant = c.muted,
            surfaceContainerLowest = c.surface, surfaceContainerLow = c.surface,
            surfaceContainer = c.chrome, surfaceContainerHigh = c.field,
            surfaceContainerHighest = c.field,
            outline = c.faint, outlineVariant = c.outline,
            errorContainer = c.badBg, onErrorContainer = c.badText,
        )
    }
    androidx.compose.runtime.CompositionLocalProvider(LocalTonal provides c) {
        MaterialTheme(colorScheme = scheme, typography = typography, content = content)
    }
}
