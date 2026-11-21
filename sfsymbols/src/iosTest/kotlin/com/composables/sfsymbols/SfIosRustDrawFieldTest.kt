package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class SfIosRustDrawFieldTest {
    @Test
    fun rustDrawFieldMatchesKotlinReferenceForAdjacentPixels() {
        val mask = ByteArray(GRID * GRID)
        mask[0] = 1
        mask[1] = 1

        val levels = checkNotNull(
            sfRustDrawFieldLevels(mask, 0f, 0f, 1f, Rect(0f, 0f, 112f, 112f), FloatArray(0)),
        )

        assertEquals(GRID * GRID, levels.size)
        assertEquals(0, levels[0].toInt() and 0xff)
        assertEquals(47, levels[1].toInt() and 0xff)
        assertEquals(255, levels[2].toInt() and 0xff)
    }

    @Test
    fun rustAdapterReturnsNullForEmptyOrInvalidMasks() {
        val bounds = Rect(0f, 0f, 112f, 112f)
        assertNull(sfRustDrawFieldLevels(ByteArray(GRID * GRID), 0f, 0f, 1f, bounds, FloatArray(0)))
        assertNull(sfRustDrawFieldLevels(ByteArray(1), 0f, 0f, 1f, bounds, FloatArray(0)))
    }

    private companion object {
        const val GRID = 112
    }
}
