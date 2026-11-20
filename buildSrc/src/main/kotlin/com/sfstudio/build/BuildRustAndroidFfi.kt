package com.sfstudio.build

import org.gradle.api.DefaultTask
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.Internal
import org.gradle.api.tasks.OutputDirectory
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction
import org.gradle.process.ExecOperations
import javax.inject.Inject

abstract class BuildRustAndroidFfi @Inject constructor(
    private val execOperations: ExecOperations,
) : DefaultTask() {
    @get:InputFiles
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val rustInputs: ConfigurableFileCollection

    @get:OutputDirectory
    abstract val outputDirectory: DirectoryProperty

    @get:Internal
    abstract val repositoryDirectory: DirectoryProperty

    @TaskAction
    fun build() {
        execOperations.exec {
            workingDir(repositoryDirectory.dir("rust").get().asFile)
            commandLine(
                "cargo", "ndk", "-P", "21",
                "-t", "armeabi-v7a", "-t", "arm64-v8a", "-t", "x86", "-t", "x86_64",
                "-o", outputDirectory.get().asFile.absolutePath,
                "--manifest-path", "Cargo.toml", "build", "-p", "sfstudio-ffi", "--release",
            )
        }
    }
}
