package com.sfsymbols.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.animation.Crossfade
import androidx.compose.animation.core.tween
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import com.sfsymbols.ui.theme.AppTheme
import com.sfsymbols.viewmodel.CatalogViewModel
import dev.nucleusframework.macoscompose.theme.MacosTheme

/**
 * Root composable: theme + app chrome. Layout is assembled inside [AppChrome]
 * with macos-ui components.
 */
@Composable
public fun SfSymbolsCatalogApp() {
    val viewModel = remember { CatalogViewModel() }
    val systemIsDark = isSystemInDarkTheme()
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
