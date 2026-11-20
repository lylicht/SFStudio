package com.sfsymbols.data

import com.composables.sfsymbols.SfSymbolsCatalog
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class RustSymbolLookupParityTest {
    @Test
    fun `Rust symbol metadata lookup matches the public Kotlin catalog`() {
        assertTrue(RustCatalogSearch.available(), "jvmTest must load the Rust native library")
        for (name in listOf("heart.fill", "house", "SFHeartFill", "HEART.FILL")) {
            val expected = SfSymbolsCatalog.all.first {
                it.appleName.equals(name, ignoreCase = true) ||
                    it.pascalName.equals(name, ignoreCase = true)
            }
            assertEquals(expected, RustCatalogSearch.lookup(name), "name=$name")
        }
        assertNull(RustCatalogSearch.lookup("not-a-symbol"))
    }
}
