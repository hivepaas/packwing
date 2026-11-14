# JNA, which the generated UniFFI Kotlin calls the Rust library through, finds
# its structures and callbacks by name at runtime.
-keep class com.sun.jna.** { *; }
-keep class * implements com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.Structure { *; }
# The generated bindings, for the same reason: their callback interfaces are
# handed to Rust, which calls them back by their JNA names.
-keep class com.hivepaas.packwing.uniffi.** { *; }
-keep class uniffi.packwing_mobile.** { *; }
