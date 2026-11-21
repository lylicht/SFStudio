package com.sfsymbols.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.animation.Crossfade
import androidx.compose.animation.core.tween
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import com.sfsymbols.ui.theme.AppTheme
import com.sfsymbols.data.windowsAppsAreDark
import com.sfsymbols.viewmodel.CatalogViewModel
import dev.nucleusframework.macoscompose.theme.MacosTheme
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive

/**
 * Root composable: theme + app chrome. Layout is assembled inside [AppChrome]
 * with macos-ui components.
 */
@Composable
public fun SfSymbolsCatalogApp() {
    val viewModel = remember { CatalogViewModel() }
    val composeSystemIsDark = isSystemInDarkTheme()
    val windows = System.getProperty("os.name").startsWith("Windows", ignoreCase = true)
    val systemIsDark by produceState(
        initialValue = windowsAppsAreDark() ?: composeSystemIsDark,
        windows,
        composeSystemIsDark,
    ) {
        if (windows) {
            while (isActive) {
                value = windowsAppsAreDark() ?: composeSystemIsDark
                delay(1_000)
            }
        } else {
            value = composeSystemIsDark
        }
    }
    val darkTheme = remember(viewModel.themeMode, systemIsDark) {
        viewModel.resolveDarkTheme(systemIsDark)
    }
    AppTheme(
        darkTheme = darkTheme,
    ) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(MacosTheme.colorScheme.background)
        ) {
            Crossfade(
                targetState = viewModel.screen,
                animationSpec = tween(180),
                label = "screen-transition",
            ) { screen ->
                when (screen) {
                    CatalogViewModel.Screen.Catalog -> AppChrome(viewModel, Modifier.fillMaxSize())
                    CatalogViewModel.Screen.Settings -> SettingsPane(viewModel, Modifier.fillMaxSize())
                }
            }
        }
    }
}
