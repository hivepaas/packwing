plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

// The app's version is the workspace's, in Cargo.toml: one for every platform
// (docs/releasing.md).
val workspaceVersion: String = rootProject.file("../Cargo.toml").readLines()
    .dropWhile { it.trim() != "[workspace.package]" }
    .firstNotNullOf { Regex("""^version = "(.+)"""").find(it)?.groupValues?.get(1) }

/** 0.3.0-beta1 is 30001 and 0.3.0 is 30099: a beta comes before its release. */
fun versionCodeOf(version: String): Int {
    val parts = Regex("""^(\d+)\.(\d+)\.(\d+)(?:-beta(\d+))?$""").matchEntire(version)
        ?: error("Cargo.toml's version $version is not X.Y.Z or X.Y.Z-betaN")
    val (major, minor, patch, beta) = parts.destructured
    return major.toInt() * 1_000_000 + minor.toInt() * 10_000 + patch.toInt() * 100 + (beta.toIntOrNull() ?: 99)
}

// Release builds are signed when the environment holds the key; without it
// they come out unsigned.
val keystore: String? = System.getenv("PACKWING_KEYSTORE")

android {
    namespace = "com.hivepaas.packwing"
    compileSdk = 37

    defaultConfig {
        applicationId = "com.hivepaas.packwing"
        // scripts/build_android.sh builds the Rust library for the same.
        // Android 10 is the oldest: the app writes through the document
        // picker and MediaStore alone, which is how 10 and later work.
        minSdk = 29
        targetSdk = 36
        versionCode = versionCodeOf(workspaceVersion)
        versionName = workspaceVersion
        // What the Rust library is built for: phones and the emulator.
        ndk { abiFilters += listOf("arm64-v8a", "x86_64") }
    }

    signingConfigs {
        if (keystore != null) {
            create("release") {
                storeFile = file(keystore)
                storePassword = System.getenv("PACKWING_KEYSTORE_PASSWORD")
                keyAlias = System.getenv("PACKWING_KEY_ALIAS")
                keyPassword = System.getenv("PACKWING_KEY_PASSWORD")
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            if (keystore != null) signingConfig = signingConfigs.getByName("release")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    // The Kotlin UniFFI writes from the Rust library (scripts/build_android.sh).
    sourceSets["main"].kotlin.directories += "build/generated/uniffi"
    // Nunito, the desktop's typeface, from where the desktop app keeps it:
    // one copy for every platform.
    sourceSets["main"].assets.srcDir("../../crates/app/assets/fonts")
}

dependencies {
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.material3)
    implementation(libs.compose.material.icons)
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.tooling.preview)
    debugImplementation(libs.compose.ui.tooling)
    implementation(libs.activity.compose)
    implementation(libs.core.ktx)
    implementation(libs.lifecycle.runtime.compose)
    implementation(libs.lifecycle.service)
    implementation(libs.coroutines.android)
    implementation(libs.documentfile)
    // What the generated Kotlin calls the Rust library through.
    implementation(libs.jna) { artifact { type = "aar" } }
}
