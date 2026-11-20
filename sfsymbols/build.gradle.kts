import org.gradle.api.tasks.testing.Test

plugins {
    alias(libs.plugins.kotlin.multiplatform)
    alias(libs.plugins.android.multiplatform.library)
    alias(libs.plugins.compose.compiler)
    alias(libs.plugins.compose.multiplatform)
    id("maven-publish")
}

// Keep Gradle's settings-level repository policy while using CI-provided JS tools.
if (providers.gradleProperty("sfstudioUseSystemWebTools").orNull == "true") {
    project.plugins.withType<org.jetbrains.kotlin.gradle.targets.js.nodejs.NodeJsPlugin> {
        project.the<org.jetbrains.kotlin.gradle.targets.js.nodejs.NodeJsEnvSpec>().download = false
    }
    project.plugins.withType<org.jetbrains.kotlin.gradle.targets.wasm.nodejs.WasmNodeJsPlugin> {
        project.the<org.jetbrains.kotlin.gradle.targets.wasm.nodejs.WasmNodeJsEnvSpec>().download = false
    }
    rootProject.plugins.withType<org.jetbrains.kotlin.gradle.targets.js.nodejs.NodeJsPlugin> {
        rootProject.the<org.jetbrains.kotlin.gradle.targets.js.nodejs.NodeJsEnvSpec>().download = false
    }
    rootProject.plugins.withType<org.jetbrains.kotlin.gradle.targets.wasm.nodejs.WasmNodeJsPlugin> {
        rootProject.the<org.jetbrains.kotlin.gradle.targets.wasm.nodejs.WasmNodeJsEnvSpec>().download = false
    }
    rootProject.plugins.withType<org.jetbrains.kotlin.gradle.targets.js.yarn.YarnPlugin> {
        rootProject.the<org.jetbrains.kotlin.gradle.targets.js.yarn.YarnRootEnvSpec>().download = false
    }
    rootProject.plugins.withType<org.jetbrains.kotlin.gradle.targets.wasm.yarn.WasmYarnPlugin> {
        rootProject.the<org.jetbrains.kotlin.gradle.targets.wasm.yarn.WasmYarnRootEnvSpec>().download = false
    }
}

kotlin {
    android {
        namespace = "com.composables.sfsymbols"
        compileSdk = 37
        minSdk = 21
        compilerOptions {
            jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
        }
    }

    jvm()

    iosArm64()
    iosSimulatorArm64()

    js {
        browser()
        nodejs()
    }

    @OptIn(org.jetbrains.kotlin.gradle.ExperimentalWasmDsl::class)
    wasmJs {
        browser()
        nodejs()
    }

    sourceSets {
        commonMain.dependencies {
            api(libs.compose.ui)
            api(libs.compose.ui.graphics)
        }

        jvmMain.dependencies {
            implementation(libs.jna)
            implementation(libs.kotlinx.serialization.json)
        }

        jvmTest.dependencies {
            implementation(kotlin("test-junit"))
            // Skia natives for the headless render tests (picked for the machine running them)
            val os = System.getProperty("os.name").lowercase()
            val arm = System.getProperty("os.arch").lowercase().let { it.contains("aarch64") || it.contains("arm64") }
            val skikoOs = when {
                os.contains("win") -> "windows-x64"
                os.contains("mac") -> if (arm) "macos-arm64" else "macos-x64"
                else -> if (arm) "linux-arm64" else "linux-x64"
            }
            implementation("org.jetbrains.skiko:skiko-awt-runtime-$skikoOs:0.150.1")
        }
    }
}

val rustFfiName = when {
    System.getProperty("os.name").startsWith("Windows", ignoreCase = true) -> "sfstudio_ffi.dll"
    System.getProperty("os.name").contains("mac", ignoreCase = true) -> "libsfstudio_ffi.dylib"
    else -> "libsfstudio_ffi.so"
}

tasks.named<Test>("jvmTest") {
    dependsOn(rootProject.tasks.named("buildRustFfi"))
    systemProperty(
        "sfstudio.rust.animation.library",
        rootProject.file("rust/target/release/$rustFfiName").absolutePath,
    )
    if (Runtime.version().feature() >= 24) jvmArgs("--enable-native-access=ALL-UNNAMED")
}
