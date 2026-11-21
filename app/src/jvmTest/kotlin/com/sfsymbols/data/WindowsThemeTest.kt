package com.sfsymbols.data

import kotlin.test.Test
import kotlin.test.assertEquals

class WindowsThemeTest {
    @Test
    fun `maps the Windows app theme registry values`() {
        assertEquals(true, appsUseLightThemeToDark(0))
        assertEquals(false, appsUseLightThemeToDark(1))
        assertEquals(null, appsUseLightThemeToDark(2))
    }
}
