package com.composables.sfsymbols

import androidx.compose.ui.graphics.Color

internal expect fun sfRustGlyphPoints(
    symbol: SfSymbol,
    weight: Float,
    scale: SfScale,
): FloatArray?

internal expect fun sfRustGlyphBounds(
    symbol: SfSymbol,
    weight: Float,
    scale: SfScale,
): FloatArray?

internal expect fun sfRustAnimationFrames(
    symbol: SfSymbol,
    effect: SfSymbolEffect,
    time: Float,
    rtl: Boolean,
    variableValue: Float?,
    mode: SfRenderingMode = SfRenderingMode.Monochrome,
    tint: Color = Color.Black,
    palette: List<Color> = emptyList(),
    gradient: Boolean = false,
): FloatArray?
