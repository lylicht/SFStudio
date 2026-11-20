import org.gradle.api.tasks.Exec

plugins {
    alias(libs.plugins.kotlin.multiplatform) apply false
    alias(libs.plugins.kotlin.compose) apply false
    alias(libs.plugins.jetbrains.compose) apply false
    alias(libs.plugins.android.multiplatform.library) apply false
}

val windowsHost = System.getProperty("os.name").startsWith("Windows", ignoreCase = true)
val rustFfiName = when {
    windowsHost -> "sfstudio_ffi.dll"
    System.getProperty("os.name").contains("mac", ignoreCase = true) -> "libsfstudio_ffi.dylib"
    else -> "libsfstudio_ffi.so"
}
val rustFfiBinary = layout.projectDirectory.file("rust/target/release/$rustFfiName")

tasks.register<Exec>("buildRustFfi") {
    group = "build"
    description = "Build the shared Rust catalog and animation library for this platform"
    workingDir(rootProject.projectDir)
    commandLine("cargo", "build", "--manifest-path", "rust/Cargo.toml", "-p", "sfstudio-ffi", "--release")
    inputs.files(
        fileTree(rootProject.file("rust")) { include("**/*.rs", "**/*.toml") },
        rootProject.file("rust/Cargo.lock"),
        rootProject.file("sfsymbols/src/commonMain/kotlin/com/composables/sfsymbols/SfSymbolsCatalog.kt"),
        rootProject.file("app/src/jvmMain/resources/symbol_tags.txt"),
    )
    outputs.file(rustFfiBinary)
}
