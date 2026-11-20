package com.sfsymbols.data

import kotlinx.serialization.builtins.SetSerializer
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals

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
}
