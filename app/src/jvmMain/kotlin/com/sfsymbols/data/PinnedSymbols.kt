package com.sfsymbols.data

import kotlinx.serialization.builtins.SetSerializer
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.Json
import java.io.File

/**
 * Persists the set of pinned symbol apple-names to a JSON file
 * in the user home directory so pins survive app restarts.
 */
public object PinnedSymbols {

    private val json = Json { prettyPrint = true }
    private val serializer = SetSerializer(String.serializer())

    private fun stateDir(): File =
        File(System.getProperty("user.home"), ".sf-symbols-catalog").apply { mkdirs() }

    private fun pinsFile(): File = File(stateDir(), "pins.json")

    public fun load(): Set<String> {
        return runCatching {
            val file = pinsFile()
            val rustText = try {
                RustCatalogSearch.loadFavorites(file.absolutePath)
            } catch (error: Exception) {
                RustCatalogSearch.reportFailure(error)
                null
            } catch (error: LinkageError) {
                RustCatalogSearch.reportFailure(error)
                null
            }
            if (rustText == null && !file.exists()) return emptySet()
            val text = rustText ?: file.readText()
            json.decodeFromString(SetSerializer(String.serializer()), text)
        }.getOrDefault(emptySet())
    }

    public fun save(pinned: Set<String>) {
        val file = pinsFile()
        val text = json.encodeToString(serializer, pinned)
        try {
            if (RustCatalogSearch.saveFavorites(file.absolutePath, text)) return
        } catch (error: Exception) {
            RustCatalogSearch.reportFailure(error)
        } catch (error: LinkageError) {
            RustCatalogSearch.reportFailure(error)
        }
        runCatching { file.writeText(text) }
    }

    public fun toggle(pinned: Set<String>, appleName: String): Set<String> {
        val currentJson = json.encodeToString(serializer, pinned)
        try {
            RustCatalogSearch.toggleFavorites(currentJson, appleName)?.let {
                return json.decodeFromString(serializer, it)
            }
        } catch (error: Exception) {
            RustCatalogSearch.reportFailure(error)
        } catch (error: LinkageError) {
            RustCatalogSearch.reportFailure(error)
        }
        return if (appleName in pinned) pinned - appleName else pinned + appleName
    }
}
