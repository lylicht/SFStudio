package com.sfstudio.tasks

import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.Internal
import org.gradle.api.tasks.OutputFile
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction
import org.gradle.process.ExecOperations
import javax.inject.Inject

abstract class BuildRustIosFfi @Inject constructor(
    private val execOperations: ExecOperations,
) : DefaultTask() {
    @get:Input
    abstract val targetTriple: Property<String>

    @get:InputFiles
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val rustInputs: ConfigurableFileCollection

    @get:Internal
    abstract val repositoryDirectory: DirectoryProperty

    @get:OutputFile
    abstract val staticLibrary: RegularFileProperty

    @get:OutputFile
    abstract val definitionFile: RegularFileProperty

    @TaskAction
    fun build() {
        val target = targetTriple.get()
        execOperations.exec {
            workingDir(repositoryDirectory.get().asFile)
            commandLine(
                "cargo", "build", "--manifest-path", "rust/Cargo.toml",
                "-p", "sfstudio-ffi", "--release", "--target", target,
            )
        }

        val archive = staticLibrary.get().asFile
        if (!archive.isFile) throw GradleException("Rust iOS archive was not produced: $archive")
        val header = repositoryDirectory.file("include/sfstudio.h").get().asFile
        val definition = definitionFile.get().asFile
        definition.parentFile.mkdirs()
        definition.writeText(
            """
            headers = sfstudio.h
            package = com.composables.sfsymbols.ffi
            compilerOpts = -I${header.parentFile.absolutePath}
            staticLibraries = ${archive.name}
            libraryPaths = ${archive.parentFile.absolutePath}
            """.trimIndent() + "\n",
        )
    }
}
