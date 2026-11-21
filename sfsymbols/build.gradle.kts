import org.gradle.api.tasks.testing.Test
import com.sfstudio.build.BuildRustAndroidFfi
import com.sfstudio.tasks.BuildRustIosFfi

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

val iosRustInputs = rootProject.files(
    rootProject.fileTree("rust") {
        include("**/*.rs", "**/*.toml")
        exclude("target/**")
    },
    rootProject.file("rust/Cargo.lock"),
    rootProject.file("include/sfstudio.h"),
    rootProject.file("sfsymbols/src/commonMain/kotlin/com/composables/sfsymbols/SfSymbolsCatalog.kt"),
    rootProject.file("app/src/jvmMain/resources/symbol_tags.txt"),
)
val iosRustBuilds = mapOf(
    "aarch64-apple-ios" to "buildRustIosArm64Ffi",
    "aarch64-apple-ios-sim" to "buildRustIosSimulatorArm64Ffi",
).mapValues { (target, taskName) ->
    tasks.register<BuildRustIosFfi>(taskName) {
        targetTriple.set(target)
        rustInputs.from(iosRustInputs)
        repositoryDirectory.set(rootProject.layout.projectDirectory)
        staticLibrary.set(rootProject.layout.projectDirectory.file("rust/target/$target/release/libsfstudio_ffi.a"))
        definitionFile.set(layout.buildDirectory.file("generated/rust-cinterop/$target/sfstudio.def"))
    }
}

kotlin {
    android {
        namespace = "com.composables.sfsymbols"
        compileSdk = 37
        minSdk = 21
        withDeviceTest {
            instrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
            execution = "HOST"
        }
        androidResources.enable = true
        optimization.consumerKeepRules.apply {
            publish = true
            file("consumer-proguard-rules.pro")
        }
        compilerOptions {
            jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
        }
    }

    jvm()

    iosArm64().compilations.getByName("main").cinterops.create("sfstudioRust") {
        definitionFile.set(iosRustBuilds.getValue("aarch64-apple-ios").flatMap { it.definitionFile })
    }
    iosSimulatorArm64().compilations.getByName("main").cinterops.create("sfstudioRust") {
        definitionFile.set(iosRustBuilds.getValue("aarch64-apple-ios-sim").flatMap { it.definitionFile })
    }

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

        androidMain.dependencies {
            implementation(libs.kotlinx.serialization.json)
        }

        iosTest.dependencies {
            implementation(kotlin("test"))
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

        getByName("androidDeviceTest") {
            dependencies {
                implementation("androidx.test:runner:1.7.0")
                implementation("androidx.test.ext:junit:1.3.0")
            }
        }
    }
}

val buildRustAndroidFfi = tasks.register<BuildRustAndroidFfi>("buildRustAndroidFfi") {
    outputDirectory.set(layout.buildDirectory.dir("generated/android/jniLibs"))
    repositoryDirectory.set(rootProject.layout.projectDirectory)
    rustInputs.from(
        rootProject.fileTree("rust") {
            include("**/*.rs", "**/*.toml")
            exclude("target/**")
        },
        rootProject.file("rust/Cargo.lock"),
        rootProject.file("sfsymbols/src/commonMain/kotlin/com/composables/sfsymbols/SfSymbolsCatalog.kt"),
        rootProject.file("app/src/jvmMain/resources/symbol_tags.txt"),
    )
}

androidComponents {
    onVariants { variant ->
        variant.sources.jniLibs?.addGeneratedSourceDirectory(buildRustAndroidFfi) { it.outputDirectory }
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
