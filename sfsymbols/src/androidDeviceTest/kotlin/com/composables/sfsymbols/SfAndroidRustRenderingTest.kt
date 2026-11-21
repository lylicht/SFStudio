package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class SfAndroidRustRenderingTest {
    private data class Case(val name: String, val effect: SfSymbolEffect, val time: Float, val rtl: Boolean = false)

    @Test
    fun rustEffectFramesMatchKotlinForAppleEffects() {
        val cases = listOf(
            Case("wifi", SfSymbolEffect.Appear(SfEffectScope.ByLayer), 0.08f),
            Case("heart.fill", SfSymbolEffect.Disappear(), 0.1f),
            Case("figure.run", SfSymbolEffect.Bounce(SfBounceDirection.Down, SfEffectScope.ByLayer), 0.31f),
            Case("wifi", SfSymbolEffect.Pulse(SfEffectScope.ByLayer), 0.9f),
            Case("wifi", SfSymbolEffect.VariableColor(reversing = true), 1.5f),
            Case("wifi", SfSymbolEffect.VariableColor(SfVariableColorStyle.Cumulative, hideInactiveLayers = true), 0.55f),
            Case("heart.fill", SfSymbolEffect.Scale(returns = true), 0.5f),
            Case("heart.fill", SfSymbolEffect.Scale(up = false), 0.1f),
            Case("bell.fill", SfSymbolEffect.Wiggle(SfWiggleDirection.Left), 0.09f),
            Case("bell.fill", SfSymbolEffect.Wiggle(SfWiggleDirection.Forward, continuous = true), 0.31f, rtl = true),
            Case("gearshape.2", SfSymbolEffect.Rotate(SfRotateDirection.Clockwise, SfEffectScope.ByLayer), 0.5f),
            Case("gearshape.2", SfSymbolEffect.Rotate(), 0.5f),
            Case("heart.fill", SfSymbolEffect.Breathe(), 0.7f),
            Case("heart.fill", SfSymbolEffect.Breathe(pulsing = true), 2.2f),
            Case("checkmark", SfSymbolEffect.DrawOn(SfEffectScope.ByLayer), 0.2f),
            Case("wifi", SfSymbolEffect.DrawOn(individually = true), 0.06f),
            Case("checkmark", SfSymbolEffect.DrawOff(), 0.2f),
        )
        for (case in cases) {
            val symbol = SfSymbol.named(case.name) ?: error("unknown SF Symbol ${case.name}")
            val variable = sfVariableLevels(symbol, 0.63f)
            val rust = sfRustAnimationFrames(symbol, case.effect, case.time, case.rtl, 0.63f)
            assertNotNull("Rust frames missing for ${case.name} at ${case.time}", rust)
            for (layer in symbol.layers.indices) {
                val fx = SfFx()
                applyEffect(case.effect, case.time, layer, symbol, fx, case.rtl)
                val expected = floatArrayOf(
                    fx.scale, fx.dx, fx.dy, fx.rot, fx.alpha, fx.draw, fx.pivotX, fx.pivotY,
                    variable.alpha[layer], variable.draw[layer],
                )
                for (field in expected.indices) {
                    val actual = checkNotNull(rust)[layer * 24 + field]
                    if (expected[field].isNaN()) {
                        assertTrue("${case.name} layer $layer field $field", actual.isNaN())
                    } else {
                        assertEquals("${case.name} layer $layer field $field", expected[field], actual, 0.0002f)
                    }
                }
            }
        }
    }

    @Test
    fun rustLayerAppearancesMatchKotlinInEveryMode() {
        val tint = Color(0xFF1C1C1E)
        val palette = listOf(Color(0xFFFF3B30), Color(0xFF007AFF), Color(0xFF34C759))
        for (name in listOf("checkmark.circle.fill", "cloud.sun.rain.fill", "wifi")) {
            val symbol = SfSymbol.named(name) ?: error("unknown SF Symbol $name")
            for (mode in SfRenderingMode.values()) for (gradient in listOf(false, true)) {
                val frames = sfRustAnimationFrames(
                    symbol, SfSymbolEffect.Breathe(), 0.7f, false, 0.63f, mode, tint, palette, gradient,
                )
                assertNotNull("Rust appearance missing for $name $mode gradient=$gradient", frames)
                for ((index, layer) in symbol.layers.withIndex()) {
                    val result = checkNotNull(frames)
                    val offset = index * 24
                    assertEquals(1f, result[offset + 14], 0f)
                    val color = layerColor(layer, mode, tint, palette)
                    if (color == null) {
                        assertTrue("$name $mode layer $index should have no color", result[offset + 13].isNaN())
                    } else {
                        listOf(color.red, color.green, color.blue, color.alpha).forEachIndexed { channel, value ->
                            assertEquals("$name $mode layer $index color $channel", value, result[offset + 10 + channel], 0.002f)
                        }
                    }
                    val stops = color?.takeIf { gradient && it.alpha > 0f }?.let(::sfAutomaticGradient)
                    if (stops == null) {
                        assertEquals(0f, result[offset + 15], 0f)
                    } else {
                        assertEquals(1f, result[offset + 15], 0f)
                        listOf(stops.first, stops.second).forEachIndexed { stop, stopColor ->
                            listOf(stopColor.red, stopColor.green, stopColor.blue, stopColor.alpha).forEachIndexed { channel, value ->
                                assertEquals(
                                    "$name $mode layer $index stop $stop channel $channel",
                                    value, result[offset + 16 + stop * 4 + channel], 0.002f,
                                )
                            }
                        }
                    }
                }
            }
        }
    }

    @Test
    fun rustDrawFieldMatchesKotlinRevealLevels() {
        val mask = ByteArray(112 * 112)
        mask[0] = 1
        mask[1] = 1
        val field = sfRustDrawField(mask, 0f, 0f, 1f, Rect(0f, 0f, 112f, 112f), FloatArray(0))
        assertNotNull("Rust draw-field result unavailable", field)
        val actual = checkNotNull(field)
        assertEquals(0, actual.levels[0].toInt() and 0xff)
        assertEquals(47, actual.levels[1].toInt() and 0xff)
        assertEquals(255, actual.levels[2].toInt() and 0xff)
        assertEquals(listOf(SfRustDrawRun(0, 0, 0, 0), SfRustDrawRun(47, 0, 1, 1)), actual.runs)
    }
}
