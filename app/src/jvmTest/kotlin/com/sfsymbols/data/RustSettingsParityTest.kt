package com.sfsymbols.data

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class RustSettingsParityTest {
    @Test
    fun `Rust theme resolver matches the existing preference behavior`() {
        assertTrue(RustCatalogSearch.available(), "jvmTest must load the Rust native library")

        ThemeMode.values().forEach { mode ->
            listOf(false, true).forEach { systemIsDark ->
                assertEquals(
                    mode.isDark(systemIsDark),
                    checkNotNull(RustCatalogSearch.themeModeIsDark(mode, systemIsDark)),
                    "mode=$mode systemIsDark=$systemIsDark",
                )
            }
        }
    }

    @Test
    fun `Rust grid scale normalization matches Kotlin at boundaries`() {
        assertTrue(RustCatalogSearch.available(), "jvmTest must load the Rust native library")

        listOf(
            Float.NEGATIVE_INFINITY,
            -1f,
            0.5f,
            1f,
            1.6f,
            2f,
            Float.POSITIVE_INFINITY,
        ).forEach { scale ->
            assertEquals(
                scale.coerceIn(0.5f, 1.6f),
                checkNotNull(RustCatalogSearch.clampGridScale(scale)),
                "scale=$scale",
            )
        }
        assertTrue(checkNotNull(RustCatalogSearch.clampGridScale(Float.NaN)).isNaN())
    }
}
