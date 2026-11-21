package com.sfsymbols.data

import com.sun.jna.platform.win32.Advapi32Util
import com.sun.jna.platform.win32.WinReg

internal fun windowsAppsAreDark(): Boolean? {
    if (!System.getProperty("os.name").startsWith("Windows", ignoreCase = true)) return null
    val lightTheme = runCatching {
        Advapi32Util.registryGetIntValue(
            WinReg.HKEY_CURRENT_USER,
            "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
            "AppsUseLightTheme",
        )
    }.getOrNull() ?: return null
    return appsUseLightThemeToDark(lightTheme)
}

internal fun appsUseLightThemeToDark(value: Int): Boolean? = when (value) {
    0 -> true
    1 -> false
    else -> null
}
