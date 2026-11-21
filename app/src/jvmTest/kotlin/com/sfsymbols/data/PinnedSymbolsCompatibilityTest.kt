package com.sfsymbols.data

import kotlinx.serialization.builtins.SetSerializer
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.Json
import java.io.File
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class PinnedSymbolsCompatibilityTest {
    private val json = Json { prettyPrint = true }
    private val serializer = SetSerializer(String.serializer())

    private fun fixture(name: String): String =
        javaClass.getResource("/$name")!!.readText().replace("\r\n", "\n")

    @Test
    fun `Kotlin can read legacy and Rust-written favorite arrays`() {
        assertEquals(
            linkedSetOf("heart.fill", "star"),
            json.decodeFromString(serializer, fixture("pinned-symbols-kotlin.json")),
        )
        assertEquals(
            linkedSetOf("star", "gear"),
            json.decodeFromString(serializer, fixture("pinned-symbols-rust.json")),
        )
    }

    @Test
    fun `desktop favorites use Rust file storage at the existing user path`() {
        assertTrue(RustCatalogSearch.available(), "jvmTest must load the Rust native library")
        val originalHome = System.getProperty("user.home")
        val temporaryHome = Files.createTempDirectory("sfstudio-pins-").toFile()
        try {
            System.setProperty("user.home", temporaryHome.absolutePath)
            val file = File(temporaryHome, ".sf-symbols-catalog/pins.json")
            file.parentFile.mkdirs()
            file.writeText(fixture("pinned-symbols-kotlin.json"))

            assertEquals(linkedSetOf("heart.fill", "star"), PinnedSymbols.load())
            PinnedSymbols.save(linkedSetOf("star", "gear"))
            assertEquals(
                fixture("pinned-symbols-rust.json").trimEnd('\r', '\n'),
                file.readText().replace("\r\n", "\n"),
            )
            assertEquals(
                fixture("pinned-symbols-rust.json").trimEnd('\r', '\n'),
                RustCatalogSearch.loadFavorites(file.absolutePath),
            )
            assertEquals(linkedSetOf("star", "gear"), PinnedSymbols.load())
        } finally {
            if (originalHome == null) System.clearProperty("user.home")
            else System.setProperty("user.home", originalHome)
            temporaryHome.deleteRecursively()
        }
    }
}
