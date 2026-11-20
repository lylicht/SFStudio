import org.jetbrains.compose.desktop.application.dsl.TargetFormat
import org.gradle.api.tasks.Copy
import org.gradle.api.tasks.Exec
import org.gradle.api.tasks.bundling.Jar
import org.gradle.api.tasks.testing.Test

plugins {
    alias(libs.plugins.kotlin.multiplatform)
    alias(libs.plugins.jetbrains.compose)
    alias(libs.plugins.kotlin.compose)
}

val windowsHost = System.getProperty("os.name").startsWith("Windows", ignoreCase = true)
val rustMcpName = "sfstudio-mcp" + if (windowsHost) ".exe" else ""
val rustMcpBinary = rootProject.file("rust/target/release/$rustMcpName")
val rustFfiName = when {
    windowsHost -> "sfstudio_ffi.dll"
    System.getProperty("os.name").contains("mac", ignoreCase = true) -> "libsfstudio_ffi.dylib"
    else -> "libsfstudio_ffi.so"
}
val rustFfiBinary = rootProject.file("rust/target/release/$rustFfiName")
val rustFfiResources = layout.buildDirectory.dir("rust-ffi-resources")
val buildRustMcp by tasks.registering(Exec::class) {
    group = "build"
    description = "Build the native Rust MCP server for this platform"
    workingDir(rootProject.projectDir)
    commandLine("cargo", "build", "--manifest-path", "rust/Cargo.toml", "-p", "sfstudio-mcp", "--release")
    inputs.files(
        fileTree(rootProject.file("rust")) { include("**/*.rs", "**/*.toml") },
        rootProject.file("rust/Cargo.lock"),
        rootProject.file("sfsymbols/src/commonMain/kotlin/com/composables/sfsymbols/SfSymbolsCatalog.kt"),
        rootProject.file("app/src/jvmMain/resources/symbol_tags.txt"),
    )
    outputs.file(rustMcpBinary)
}

val stageRustMcp by tasks.registering(Copy::class) {
    dependsOn(buildRustMcp)
    from(rustMcpBinary)
    into(layout.buildDirectory.dir("rust-mcp-resources"))
}

val buildRustFfi = rootProject.tasks.named("buildRustFfi")

val stageRustFfi by tasks.registering(Copy::class) {
    dependsOn(buildRustFfi)
    from(rustFfiBinary)
    into(rustFfiResources.map { it.dir("native") })
}

// A standalone stdio launcher needs the complete runtime classpath. The normal
// jvmJar is not executable with `java -jar` and does not bundle dependencies.
tasks.register<Sync>("assembleMcp") {
    group = "distribution"
    description = "Assemble the headless MCP server and its runtime dependencies"
    dependsOn(stageRustMcp)
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
    into(layout.buildDirectory.dir("mcp"))
    from(tasks.named("jvmJar")) { into("lib") }
    from(requireNotNull(kotlin.targets.getByName("jvm").compilations.getByName("main").runtimeDependencyFiles)) {
        into("lib")
        // Different artifacts can share a filename; retain both on the wildcard classpath.
        eachFile { name = "${file.parentFile.name}-$name" }
    }
    from(layout.buildDirectory.dir("rust-mcp-resources")) { into("bin") }
    from(project.file("../scripts/mcp.cmd"))
    from(project.file("../scripts/mcp.sh"))
}

kotlin {
    jvm()

    sourceSets {
        val jvmMain by getting {
            dependencies {
                implementation(compose.desktop.currentOs)
                implementation(compose.material3)
                implementation(project(":sfsymbols"))
                implementation(libs.jna)
                implementation(libs.kotlinx.coroutines.core)
                implementation(libs.kotlinx.serialization.json)
                implementation(libs.compose.macos.ui)
            }
        }

        val jvmTest by getting {
            dependencies {
                implementation(kotlin("test"))
            }
        }
    }
}

tasks.named<Test>("jvmTest") {
    dependsOn(stageRustFfi)
    if (Runtime.version().feature() >= 24) {
        jvmArgs("--enable-native-access=ALL-UNNAMED")
    }
}

kotlin.sourceSets.getByName("jvmMain").resources.srcDir(rustFfiResources)
tasks.named("jvmProcessResources") { dependsOn(stageRustFfi) }

tasks.named<Jar>("jvmJar") {
    dependsOn(stageRustMcp, stageRustFfi)
    from(stageRustMcp) { into("native") }
}

compose.desktop {
    application {
        mainClass = "com.sfsymbols.MainKt"
        javaHome = System.getProperty("java.home")
        if (Runtime.version().feature() >= 24) {
            jvmArgs += "--enable-native-access=ALL-UNNAMED"
        }

        nativeDistributions {
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb)
            packageName = "SFStudio"
            packageVersion = providers.gradleProperty("appVersion").orElse(
                providers.environmentVariable("GITHUB_REF_NAME").map { ref ->
                    ref.removePrefix("v").takeIf { it.matches(Regex("\\d+\\.\\d+\\.\\d+")) } ?: "1.0.0"
                }
            ).orElse("1.0.0").get()
            description = "Browse, search, and copy SF Symbols for Compose projects"
            vendor = "licht"
            modules("java.prefs", "java.management", "jdk.unsupported", "jdk.crypto.ec")

            windows {
                iconFile.set(project.file("src/jvmMain/resources/app_icon.ico"))
                shortcut = true
                menu = true
                menuGroup = "SFStudio"
                perUserInstall = true
                dirChooser = true
                // Keep this ID stable so newer package versions upgrade the app.
                upgradeUuid = "E09297C4-1F95-4BDB-B14C-60F3114F05BD"
            }
            linux {
                // Keep the Debian package identifier so upgrades replace the existing package.
                packageName = "glyphstudio"
                iconFile.set(project.file("src/jvmMain/resources/app_icon.png"))
                shortcut = true
            }

            macOS {
                // Keep the original bundle ID so existing installs and user data continue to upgrade in place.
                bundleID = "com.bxane.glyphstudio"
                iconFile.set(project.file("src/jvmMain/resources/app_icon.icns"))
                // Signing + notarization switch on only when CI provides the
                // Developer ID secrets; local and fork builds stay unsigned.
                val identity = System.getenv("MACOS_SIGNING_IDENTITY")
                if (!identity.isNullOrBlank()) {
                    signing {
                        sign.set(true)
                        this.identity.set(identity)
                    }
                    notarization {
                        appleID.set(System.getenv("NOTARIZATION_APPLE_ID"))
                        password.set(System.getenv("NOTARIZATION_PASSWORD"))
                        teamID.set(System.getenv("NOTARIZATION_TEAM_ID"))
                    }
                }
            }
        }
    }
}
