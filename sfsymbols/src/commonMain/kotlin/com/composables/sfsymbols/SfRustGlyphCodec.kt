package com.composables.sfsymbols

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonPrimitive

internal fun parseRustGlyphPoints(response: String): FloatArray? {
    val points = Json.parseToJsonElement(response).jsonArray
    return FloatArray(points.size) { index ->
        points[index].jsonPrimitive.content.toFloatOrNull()
            ?.takeIf { it.isFinite() } ?: return null
    }
}
