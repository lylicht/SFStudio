package com.sfsymbols.data

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class PinnedSymbolsParityTest {
    @Test
    fun `Rust favorite toggle preserves Kotlin set order and behavior`() {
        assertTrue(RustCatalogSearch.available(), "jvmTest must load the Rust native library")
        assertEquals(
            linkedSetOf("heart.fill", "star", "gear"),
            PinnedSymbols.toggle(linkedSetOf("heart.fill", "star"), "gear"),
        )
        assertEquals(
            linkedSetOf("star"),
            PinnedSymbols.toggle(linkedSetOf("heart.fill", "star"), "heart.fill"),
        )
    }
}
