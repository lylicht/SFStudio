package com.composables.sfsymbols

import org.junit.Assert.assertEquals
import org.junit.Test

class SfAnimationRustParityTest {
    private data class Case(val name: String, val id: String, val effect: SfSymbolEffect, val time: Float, val rtl: Boolean = false)

    @Test
    fun rustEffectFramesMatchKotlinReferenceFixture() {
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
        val actual = buildString {
            for (case in cases) {
                val symbol = SfSymbol.named(case.name) ?: error("unknown SF Symbol ${case.name}")
                val levels = sfVariableLevels(symbol, 0.63f)
                val rustFrames = checkNotNull(
                    sfRustAnimationFrames(symbol, case.effect, case.time, case.rtl, 0.63f),
                ) { "Rust animation frames unavailable for ${case.id}" }
                for (index in symbol.layers.indices) {
                    val fx = SfFx()
                    applyEffect(case.effect, case.time, index, symbol, fx, case.rtl)
                    val expectedFrame = floatArrayOf(
                        fx.scale, fx.dx, fx.dy, fx.rot, fx.alpha, fx.draw, fx.pivotX, fx.pivotY,
                        levels.alpha[index], levels.draw[index],
                    )
                    expectedFrame.forEachIndexed { field, expectedValue ->
                        val actualValue = rustFrames[index * 10 + field]
                        if (expectedValue.isNaN()) {
                            assertEquals("${case.id} layer $index field $field", true, actualValue.isNaN())
                        } else {
                            assertEquals("${case.id} layer $index field $field", expectedValue, actualValue, 0.0002f)
                        }
                    }
                    append(listOf(
                        case.name, case.id, case.time.toString(), case.rtl.toString(), index.toString(),
                        effectDuration(case.effect, symbol).toString(), case.effect.isHeld().toString(),
                        (case.effect.defaultRepeat() == SfEffectRepeat.Continuous).toString(),
                        fx.scale.toString(), fx.dx.toString(), fx.dy.toString(), fx.rot.toString(),
                        fx.alpha.toString(), fx.draw.toString(), fx.pivotX.toString(), fx.pivotY.toString(),
                        levels.alpha[index].toString(), levels.draw[index].toString(),
                    ).joinToString("|")).append('\n')
                }
            }
        }
        val expected = javaClass.getResourceAsStream("/sf-animation-fingerprints.txt")
            ?.bufferedReader()?.use { it.readText() }
            ?: error("missing Kotlin/Rust animation parity fixture")
        assertEquals(expected.replace("\r\n", "\n"), actual)
    }
}
