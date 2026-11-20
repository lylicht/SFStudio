package com.sfsymbols.data

import kotlin.test.Test
import kotlin.test.assertEquals

class ThemeModeTest {
    @Test
    fun `system follows OS while light and dark override it`() {
        assertEquals(true, ThemeMode.System.isDark(systemIsDark = true))
        assertEquals(false, ThemeMode.System.isDark(systemIsDark = false))
        assertEquals(false, ThemeMode.Light.isDark(systemIsDark = true))
        assertEquals(true, ThemeMode.Dark.isDark(systemIsDark = false))
    }
}
