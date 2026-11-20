package com.sfsymbols.util

import androidx.compose.ui.graphics.Color
import com.composables.sfsymbols.SfSymbolsCatalog
import com.sfsymbols.data.RustCatalogSearch
import com.sfsymbols.data.SymbolMode
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ComposeSnippetParityTest {
    @Test
    fun `Rust C ABI snippet generation matches the Kotlin reference`() {
        assertTrue(RustCatalogSearch.available(), "jvmTest must load the Rust search library")
        val symbol = SfSymbolsCatalog.all.first { it.appleName == "heart.fill" }
        val cases = listOf(
            Triple(SymbolMode.Monochrome, Color(0x80E0245E), Color(0xFFDCE9D3)),
            Triple(SymbolMode.Dualtone, Color(0xFF123456), null),
        )
        for ((mode, tint, background) in cases) {
            assertEquals(
                kotlinComposeSnippet(symbol, mode, tint, true, background),
                composeSnippet(symbol, mode, tint, true, background),
                "mode=$mode background=$background",
            )
        }
    }
}
