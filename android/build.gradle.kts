// The app's plugins are applied in app/build.gradle.kts; this only declares
// them, so the versions live in gradle/libs.versions.toml alone.
// Kotlin needs no plugin of its own: AGP 9 has it built in.
plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.kotlin.compose) apply false
}
