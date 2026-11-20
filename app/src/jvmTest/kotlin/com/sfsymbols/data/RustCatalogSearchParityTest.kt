package com.sfsymbols.data

import com.composables.sfsymbols.SfSymbolsCatalog
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class RustCatalogSearchParityTest {
    @Test
    fun `Rust C ABI search matches the Kotlin reference ranking`() {
        assertTrue(RustCatalogSearch.available(), "jvmTest must load the Rust search library")
        val queries = listOf(
            "home", "house", "phone", "garbage", "logout", "ai", "cog", "money",
            "chekmark", "weather", "hsf", "heart", "tradh", "trsh", "hosue",
            "gearshap", "garbgae", "settigns", "msg", "pw", "pics", "bunny", "zap",
            "doggy paw", "four boxes", "4 cubes stacked on top of each other", "a yucky can",
            "4 circle", "arrow.right", "arrow.right.circle",
            "rectangle.portrait.and.arrow.right", "zzzzzz", "🐈", "naïve", "   ", "a",
            "house fill",
        )
        val limit = SfSymbolsCatalog.all.size
        for (query in queries) {
            val expected = KotlinFuzzySearch.search(query, limit).map { it.appleName }
            val actual = checkNotNull(RustCatalogSearch.search(query, limit)).map { it.appleName }
            assertEquals(expected, actual, "query=$query")
        }
    }
}
