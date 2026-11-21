package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.add
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

internal fun buildRustDrawFieldRequest(
    mask: ByteArray,
    originX: Float,
    originY: Float,
    cell: Float,
    symbolBounds: Rect,
    ties: FloatArray,
): ByteArray = buildJsonObject {
    put("mask", buildJsonArray { mask.forEach { add(it.toInt() and 0xff) } })
    put("origin", buildJsonArray { add(originX); add(originY) })
    put("cell", cell)
    put("symbolBounds", buildJsonArray {
        add(symbolBounds.left); add(symbolBounds.top); add(symbolBounds.right); add(symbolBounds.bottom)
    })
    put("ties", buildJsonArray { ties.forEach { add(it) } })
}.toString().encodeToByteArray()

internal fun parseRustDrawFieldLevels(response: String, maskSize: Int): ByteArray? {
    val values = Json.parseToJsonElement(response).jsonObject["levels"]
        ?.takeUnless { it == JsonNull }?.jsonArray ?: return null
    if (values.size != maskSize) return null
    return ByteArray(values.size) { index ->
        val value = values[index].jsonPrimitive.content.toIntOrNull()?.takeIf { it in 0..255 } ?: return null
        value.toByte()
    }
}
