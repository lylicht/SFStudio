package com.composables.sfsymbols

import androidx.compose.ui.graphics.Color
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class SfIosRustAnimationTest {
    private data class Case(
        val name: String,
        val id: String,
        val effect: SfSymbolEffect,
        val time: Float,
        val rtl: Boolean = false,
    )

    @Test
    fun malformedRustAnimationFramesAreRejected() {
        assertNull(parseRustAnimationFrames("{}", 1))
        assertNull(parseRustAnimationFrames("""{"layers":[]}""", 1))
        assertNull(parseRustAnimationFrames("""{"layers":[{}]}""", 1))
        assertNull(parseRustAnimationFrames("""{"layers":[{"scale":1}]}""", 1))
    }

    @Test
    fun rustEffectFramesMatchKotlinAcrossAppleEffects() {
        val cases = listOf(
            Case("wifi", "appear_layer", SfSymbolEffect.Appear(SfEffectScope.ByLayer), 0.08f),
            Case("heart.fill", "disappear", SfSymbolEffect.Disappear(), 0.1f),
            Case("figure.run", "bounce_down_layer", SfSymbolEffect.Bounce(SfBounceDirection.Down, SfEffectScope.ByLayer), 0.31f),
            Case("wifi", "pulse_layer", SfSymbolEffect.Pulse(SfEffectScope.ByLayer), 0.9f),
            Case("wifi", "variable_iterative_reverse", SfSymbolEffect.VariableColor(reversing = true), 1.5f),
            Case("wifi", "variable_cumulative_hide", SfSymbolEffect.VariableColor(SfVariableColorStyle.Cumulative, hideInactiveLayers = true), 0.55f),
            Case("heart.fill", "scale_up_return", SfSymbolEffect.Scale(returns = true), 0.5f),
            Case("heart.fill", "scale_down", SfSymbolEffect.Scale(up = false), 0.1f),
            Case("bell.fill", "wiggle_left", SfSymbolEffect.Wiggle(SfWiggleDirection.Left), 0.09f),
            Case("bell.fill", "wiggle_continuous_rtl", SfSymbolEffect.Wiggle(SfWiggleDirection.Forward, continuous = true), 0.31f, rtl = true),
            Case("gearshape.2", "rotate_clockwise_layer", SfSymbolEffect.Rotate(SfRotateDirection.Clockwise, SfEffectScope.ByLayer), 0.5f),
            Case("gearshape.2", "rotate_preferred", SfSymbolEffect.Rotate(), 0.5f),
            Case("heart.fill", "breathe", SfSymbolEffect.Breathe(), 0.7f),
            Case("heart.fill", "breathe_pulsing", SfSymbolEffect.Breathe(pulsing = true), 2.2f),
            Case("checkmark", "draw_on_layer", SfSymbolEffect.DrawOn(SfEffectScope.ByLayer), 0.2f),
            Case("wifi", "draw_on_individually", SfSymbolEffect.DrawOn(individually = true), 0.06f),
            Case("checkmark", "draw_off", SfSymbolEffect.DrawOff(), 0.2f),
        )
        for (case in cases) {
            val symbol = checkNotNull(SfSymbol.named(case.name))
            val variable = sfVariableLevels(symbol, 0.63f)
            val rust = checkNotNull(
                sfRustAnimationFrames(symbol, case.effect, case.time, case.rtl, 0.63f),
            ) { "Rust animation frames missing for ${case.id}" }
            for (index in symbol.layers.indices) {
                val fx = SfFx()
                applyEffect(case.effect, case.time, index, symbol, fx, case.rtl)
                val expected = floatArrayOf(
                    fx.scale, fx.dx, fx.dy, fx.rot, fx.alpha, fx.draw, fx.pivotX, fx.pivotY,
                    variable.alpha[index], variable.draw[index],
                )
                expected.forEachIndexed { field, value ->
                    val actual = rust[index * RUST_FRAME_STRIDE + field]
                    if (value.isNaN()) {
                        assertTrue("${case.id} layer $index field $field", actual.isNaN())
                    } else {
                        assertEquals("${case.id} layer $index field $field", value, actual, 0.0002f)
                    }
                }
            }
        }
    }

    @Test
    fun rustLayerAppearancesMatchKotlinForEveryMode() {
        val tint = Color(0xFF1C1C1E)
        val palette = listOf(Color(0xFFFF3B30), Color(0xFF007AFF), Color(0xFF34C759))
        for (name in listOf("checkmark.circle.fill", "cloud.sun.rain.fill", "wifi")) {
            val symbol = checkNotNull(SfSymbol.named(name))
            for (mode in SfRenderingMode.values()) for (gradient in listOf(false, true)) {
                val frames = checkNotNull(
                    sfRustAnimationFrames(
                        symbol, SfSymbolEffect.Breathe(), 0.7f, false, 0.63f,
                        mode, tint, palette, gradient,
                    ),
                )
                for ((index, layer) in symbol.layers.withIndex()) {
                    val offset = index * RUST_FRAME_STRIDE
                    assertEquals(1f, frames[offset + 14], 0f)
                    val color = layerColor(layer, mode, tint, palette)
                    if (color == null) {
                        assertTrue("$name $mode layer $index should have no color", frames[offset + 13].isNaN())
                    } else {
                        listOf(color.red, color.green, color.blue, color.alpha).forEachIndexed { channel, value ->
                            assertEquals("$name $mode layer $index color $channel", value, frames[offset + 10 + channel], 0.002f)
                        }
                    }

                    val expectedStops = color?.takeIf { gradient && it.alpha > 0f }?.let(::sfAutomaticGradient)
                    if (expectedStops == null) {
                        assertEquals("$name $mode layer $index gradient", 0f, frames[offset + 15], 0f)
                    } else {
                        assertEquals("$name $mode layer $index gradient", 1f, frames[offset + 15], 0f)
                        listOf(expectedStops.first, expectedStops.second).forEachIndexed { stop, stopColor ->
                            listOf(stopColor.red, stopColor.green, stopColor.blue, stopColor.alpha)
                                .forEachIndexed { channel, value ->
                                    assertEquals(
                                        "$name $mode layer $index stop $stop channel $channel",
                                        value,
                                        frames[offset + 16 + stop * 4 + channel],
                                        0.002f,
                                    )
                                }
                        }
                    }
                }
            }
        }
    }
}
