package com.composables.sfsymbols

import com.composables.sfsymbols.core.SfData
import com.composables.sfsymbols.core.SfGlyph
import com.composables.sfsymbols.core.SfNode
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

internal fun parseRustGlyphData(response: String): SfGlyph? = try {
    val root = Json.parseToJsonElement(response).jsonObject
    val geometry = root["geometry"]?.jsonObject ?: return null
    val ops = geometry["ops"].ints()?.also { values ->
        if (values.any { it !in 0..3 }) return null
    } ?: return null
    val regular = geometry["regular"].floats() ?: return null
    val light = geometry["light"].floats() ?: return null
    val heavy = geometry["heavy"].floats() ?: return null
    var expectedCoordinates = 0
    var contourCount = 0
    for (op in ops) {
        expectedCoordinates += when (op) {
            0, 1 -> 2
            2 -> 6
            else -> { contourCount++; 0 }
        }
    }
    if (regular.size != expectedCoordinates || light.size != expectedCoordinates || heavy.size != expectedCoordinates) return null
    if (regular.any { !it.isFinite() } || light.any { !it.isFinite() } || heavy.any { !it.isFinite() }) return null

    val anchors = root["anchors"].floats() ?: return null
    if (anchors.size % 2 != 0 || anchors.any { !it.isFinite() }) return null
    val treeJson = root["tree"]?.jsonArray ?: return null
    val tree = ArrayList<SfNode>(treeJson.size)
    for (item in treeJson) tree += parseRustNode(item, contourCount) ?: return null

    SfGlyph(
        ops = ByteArray(ops.size) { ops[it].toByte() },
        reg = regular,
        lo = light,
        hi = heavy,
        layersExact = root.bool("layersExact") ?: return null,
        backToFront = root.bool("backToFront") ?: return null,
        gapless = root.bool("gapless") ?: return null,
        defaultVariableRendering = root.int("defaultVariableRendering") ?: return null,
        wiggle = root.int("wiggle") ?: return null,
        wiggleAngle = root.long("wiggleAngle")?.toInt() ?: return null,
        rotation = root.int("rotation") ?: return null,
        anchors = anchors,
        tree = tree,
    )
} catch (_: Exception) {
    null
}

private fun parseRustNode(element: JsonElement, contourCount: Int): SfNode? {
    val node = element.jsonObject
    val ties = node["ties"].floats() ?: return null
    if (ties.size % 2 != 0 || ties.any { !it.isFinite() }) return null
    val contours = node["contours"].ints() ?: return null
    if (contours.any { it !in 0 until contourCount }) return null
    val tags = node["tagIds"].ints() ?: return null
    if (tags.any { it !in 0 until SfData.tagCount() }) return null
    val colorId = when (val value = node["colorId"]) {
        null, JsonNull -> -1
        else -> value.jsonPrimitive.content.toIntOrNull()
            ?.takeIf { it in 0 until SfData.colorCount() } ?: return null
    }
    val childrenJson = node["children"]?.jsonArray ?: return null
    val children = ArrayList<SfNode>(childrenJson.size)
    for (child in childrenJson) children += parseRustNode(child, contourCount) ?: return null

    return SfNode(
        isGroup = node.bool("isGroup") ?: return null,
        canPulse = node.bool("canPulse") ?: return null,
        canRotate = node.bool("canRotate") ?: return null,
        clearBehind = node.bool("clearBehind") ?: return null,
        animateTogether = node.bool("animateTogether") ?: return null,
        variableDraw = node.bool("variableDraw") ?: return null,
        autoVariable = node.bool("autoVariable") ?: return null,
        monoErase = node.bool("monoErase") ?: return null,
        shapeErase = node.bool("shapeErase") ?: return null,
        hierarchical = node.int("hierarchical")?.takeIf { it in -1..2 } ?: return null,
        colorId = colorId,
        opacity = node.float("opacity") ?: return null,
        rootU = node.nullableFloat("rootU") ?: return null,
        rootV = node.nullableFloat("rootV") ?: return null,
        ties = ties,
        contours = contours,
        tags = tags,
        children = children,
    )
}

private fun JsonElement?.ints(): IntArray? {
    val values = this as? JsonArray ?: return null
    val result = IntArray(values.size)
    for (i in values.indices) result[i] = values[i].jsonPrimitive.content.toIntOrNull() ?: return null
    return result
}

private fun JsonElement?.floats(): FloatArray? {
    val values = this as? JsonArray ?: return null
    val result = FloatArray(values.size)
    for (i in values.indices) {
        result[i] = values[i].jsonPrimitive.content.toFloatOrNull()?.takeIf { it.isFinite() } ?: return null
    }
    return result
}

private fun JsonObject.int(key: String): Int? = this[key]?.jsonPrimitive?.content?.toIntOrNull()
private fun JsonObject.long(key: String): Long? = this[key]?.jsonPrimitive?.content?.toLongOrNull()
private fun JsonObject.bool(key: String): Boolean? = this[key]?.jsonPrimitive?.content?.toBooleanStrictOrNull()
private fun JsonObject.float(key: String): Float? = this[key]?.jsonPrimitive?.content
    ?.toFloatOrNull()?.takeIf { it.isFinite() }

private fun JsonObject.nullableFloat(key: String): Float? = when (val value = this[key]) {
    JsonNull -> Float.NaN
    null -> null
    else -> value.jsonPrimitive.content.toFloatOrNull()?.takeIf { it.isFinite() }
}
