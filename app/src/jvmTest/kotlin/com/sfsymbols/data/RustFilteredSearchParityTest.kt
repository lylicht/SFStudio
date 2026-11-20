package com.sfsymbols.data

import com.composables.sfsymbols.SfSymbolsCatalog
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class RustFilteredSearchParityTest {
    @Test
    fun `Rust category and favorites filters preserve Kotlin ranking and catalog order`() {
        assertTrue(RustCatalogSearch.available(), "jvmTest must load the Rust native library")
        val all = SfSymbolsCatalog.all
        val heartCategory = all.first { it.appleName == "heart.fill" }.categories.first()
        val houseCategory = all.first { it.appleName == "house" }.categories.first()
        val largeCategory = all.flatMap { it.categories }.groupingBy { it }.eachCount()
            .maxBy { it.value }.key
        assertTrue(all.count { largeCategory in it.categories } > 300)
        val favorites = setOf("heart.fill", "heart", "heart.circle.fill", "star.fill", "house")
        val cases: List<Triple<String, String?, Set<String>?>> = listOf(
            Triple("   ", largeCategory, null),
            Triple("heart", heartCategory, null),
            Triple("hosue", houseCategory, null),
            Triple("", null, favorites),
            Triple("heart", null, favorites),
            Triple("   ", null, emptySet()),
            Triple("heart", heartCategory, favorites),
            Triple("🐈", heartCategory, favorites),
        )

        for ((query, category, pinned) in cases) {
            val scoped = all.filter { symbol ->
                (category == null || category in symbol.categories) &&
                    (pinned == null || symbol.appleName in pinned)
            }
            val expected = if (query.isBlank()) {
                scoped
            } else {
                val allowed = scoped.mapTo(HashSet(scoped.size)) { it.appleName }
                KotlinFuzzySearch.search(query, all.size).filter { it.appleName in allowed }.take(300)
            }.map { it.appleName }
            val actual = checkNotNull(
                RustCatalogSearch.searchFiltered(
                    query,
                    category,
                    pinned,
                    if (query.isBlank()) all.size else 300,
                ),
            ).map { it.appleName }
            assertEquals(expected, actual, "query=$query category=$category favorites=$pinned")
        }
    }
}
