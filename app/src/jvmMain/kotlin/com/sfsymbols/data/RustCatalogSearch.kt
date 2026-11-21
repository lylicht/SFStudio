package com.sfsymbols.data

import com.composables.sfsymbols.SfSymbolMetadata
import com.composables.sfsymbols.SfSymbolsCatalog
import com.sun.jna.Library
import com.sun.jna.Native
import kotlinx.serialization.builtins.SetSerializer
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.util.concurrent.atomic.AtomicBoolean

internal interface NativeSearch : Library {
    fun sfstudio_filter_selection_json(
        action: Int,
        category: ByteArray?,
        categoryLength: Long,
        categoryPresent: Int,
        output: ByteArray?,
        outputCapacity: Long,
    ): Long

    fun sfstudio_search_json(
        query: ByteArray?,
        queryLength: Long,
        limit: Long,
        output: ByteArray?,
        outputCapacity: Long,
    ): Long

    fun sfstudio_symbol_json(
        symbolName: ByteArray?,
        symbolNameLength: Long,
        output: ByteArray?,
        outputCapacity: Long,
    ): Long

    fun sfstudio_search_filtered_json(
        query: ByteArray?,
        queryLength: Long,
        category: ByteArray?,
        categoryLength: Long,
        favoritesJson: ByteArray?,
        favoritesJsonLength: Long,
        favoritesOnly: Int,
        limit: Long,
        output: ByteArray?,
        outputCapacity: Long,
    ): Long

    fun sfstudio_compose_snippet(
        symbolName: ByteArray?,
        symbolLength: Long,
        mode: Int,
        tintArgb: Int,
        tintEnabled: Int,
        backgroundArgb: Int,
        backgroundEnabled: Int,
        output: ByteArray?,
        outputCapacity: Long,
    ): Long

    fun sfstudio_favorites_toggle_json(
        favoritesJson: ByteArray?,
        favoritesJsonLength: Long,
        symbolName: ByteArray?,
        symbolNameLength: Long,
        output: ByteArray?,
        outputCapacity: Long,
    ): Long

    fun sfstudio_mcp_client_config_json(
        appPath: ByteArray?,
        appPathLength: Long,
        appPathPresent: Int,
        developmentLibPath: ByteArray?,
        developmentLibPathLength: Long,
        output: ByteArray?,
        outputCapacity: Long,
    ): Long

    fun sfstudio_theme_mode_is_dark(themeMode: Int, systemIsDark: Int): Int

    fun sfstudio_grid_scale_clamp(scale: Float): Float
}

internal object RustCatalogSearch {
    private val warned = AtomicBoolean()

    private val symbolsByAppleName by lazy { SfSymbolsCatalog.all.associateBy { it.appleName } }
    private val native: NativeSearch? by lazy { loadNative() }

    enum class FilterAction(val nativeId: Int) { Category(0), Favorites(1), All(2) }

    fun available(): Boolean = native != null

    fun reportFailure(error: Throwable) = warn(error)

    fun themeModeIsDark(mode: ThemeMode, systemIsDark: Boolean): Boolean? {
        val library = native ?: return null
        val modeId = when (mode) {
            ThemeMode.System -> 0
            ThemeMode.Light -> 1
            ThemeMode.Dark -> 2
        }
        return when (val result = library.sfstudio_theme_mode_is_dark(modeId, if (systemIsDark) 1 else 0)) {
            0 -> false
            1 -> true
            else -> error("Rust theme resolver failed with code $result")
        }
    }

    fun clampGridScale(scale: Float): Float? = native?.sfstudio_grid_scale_clamp(scale)

    fun filterSelection(action: FilterAction, category: String? = null): Pair<String?, Boolean>? {
        val library = native ?: return null
        val categoryBytes = category?.toByteArray(Charsets.UTF_8)
        val required = library.sfstudio_filter_selection_json(
            action.nativeId,
            categoryBytes,
            categoryBytes?.size?.toLong() ?: 0,
            if (category == null) 0 else 1,
            null,
            0,
        )
        check(required >= 0) { "Rust filter selection failed with code $required" }
        check(required <= Int.MAX_VALUE) { "Rust filter selection is too large for the JVM" }

        val output = ByteArray(required.toInt())
        val written = library.sfstudio_filter_selection_json(
            action.nativeId,
            categoryBytes,
            categoryBytes?.size?.toLong() ?: 0,
            if (category == null) 0 else 1,
            output,
            output.size.toLong(),
        )
        check(written == required) { "Rust filter selection wrote $written bytes; expected $required" }
        val selection = Json.parseToJsonElement(output.decodeToString()).jsonArray
        check(selection.size == 2) { "Rust filter selection returned an invalid pair" }
        val selectedCategory = selection[0].let { value ->
            if (value is kotlinx.serialization.json.JsonNull) null else value.jsonPrimitive.content
        }
        val onlyPinned = selection[1].jsonPrimitive.content.toBooleanStrict()
        return selectedCategory to onlyPinned
    }

    fun lookup(symbolName: String): SfSymbolMetadata? {
        val library = native ?: return null
        val name = symbolName.toByteArray(Charsets.UTF_8)
        val required = library.sfstudio_symbol_json(name, name.size.toLong(), null, 0)
        if (required == -6L) return null
        check(required >= 0) { "Rust symbol lookup sizing failed with code $required" }
        check(required <= Int.MAX_VALUE) { "Rust symbol metadata is too large for the JVM" }

        val output = ByteArray(required.toInt())
        val written = library.sfstudio_symbol_json(
            name,
            name.size.toLong(),
            output,
            output.size.toLong(),
        )
        check(written == required) { "Rust symbol lookup wrote $written bytes; expected $required" }
        val metadata = Json.parseToJsonElement(output.decodeToString()).jsonObject
        return SfSymbolMetadata(
            appleName = metadata.getValue("appleName").jsonPrimitive.content,
            pascalName = metadata.getValue("pascalName").jsonPrimitive.content,
            categories = metadata.getValue("categories").jsonArray.map { it.jsonPrimitive.content },
            isRestricted = metadata.getValue("isRestricted").jsonPrimitive.content.toBooleanStrict(),
        )
    }

    fun search(query: String, limit: Int): List<SfSymbolMetadata>? {
        val library = native ?: return null
        val queryBytes = query.toByteArray(Charsets.UTF_8)
        val required = library.sfstudio_search_json(queryBytes, queryBytes.size.toLong(), limit.toLong(), null, 0)
        check(required >= 0) { "Rust search sizing failed with code $required" }
        check(required <= Int.MAX_VALUE) { "Rust search result is too large for the JVM" }

        val output = ByteArray(required.toInt())
        val written = library.sfstudio_search_json(
            queryBytes,
            queryBytes.size.toLong(),
            limit.toLong(),
            output,
            output.size.toLong(),
        )
        check(written == required) { "Rust search wrote $written bytes; expected $required" }
        return decodeSymbols(output)
    }

    fun searchFiltered(
        query: String,
        category: String?,
        favorites: Set<String>?,
        limit: Int,
    ): List<SfSymbolMetadata>? {
        val library = native ?: return null
        val queryBytes = query.toByteArray(Charsets.UTF_8)
        val categoryBytes = category?.toByteArray(Charsets.UTF_8) ?: ByteArray(0)
        val favoritesBytes = favorites?.let {
            Json.encodeToString(SetSerializer(String.serializer()), it).toByteArray(Charsets.UTF_8)
        }
        val required = library.sfstudio_search_filtered_json(
            queryBytes,
            queryBytes.size.toLong(),
            categoryBytes,
            categoryBytes.size.toLong(),
            favoritesBytes,
            favoritesBytes?.size?.toLong() ?: 0,
            if (favorites == null) 0 else 1,
            limit.toLong(),
            null,
            0,
        )
        check(required >= 0) { "Rust filtered search sizing failed with code $required" }
        check(required <= Int.MAX_VALUE) { "Rust filtered search result is too large for the JVM" }

        val output = ByteArray(required.toInt())
        val written = library.sfstudio_search_filtered_json(
            queryBytes,
            queryBytes.size.toLong(),
            categoryBytes,
            categoryBytes.size.toLong(),
            favoritesBytes,
            favoritesBytes?.size?.toLong() ?: 0,
            if (favorites == null) 0 else 1,
            limit.toLong(),
            output,
            output.size.toLong(),
        )
        check(written == required) { "Rust filtered search wrote $written bytes; expected $required" }
        return decodeSymbols(output)
    }

    private fun decodeSymbols(output: ByteArray): List<SfSymbolMetadata> =
        Json.parseToJsonElement(output.decodeToString()).jsonArray.map { item ->
            val name = item.jsonPrimitive.content
            checkNotNull(symbolsByAppleName[name]) { "Rust search returned unknown symbol '$name'" }
        }

    fun composeSnippet(
        symbol: SfSymbolMetadata,
        mode: SymbolMode,
        tintArgb: Int,
        tintEnabled: Boolean,
        backgroundArgb: Int?,
    ): String? {
        val library = native ?: return null
        val name = symbol.appleName.toByteArray(Charsets.UTF_8)
        val modeId = if (mode == SymbolMode.Dualtone) 0 else 1
        val tintFlag = if (tintEnabled) 1 else 0
        val background = backgroundArgb ?: 0
        val backgroundFlag = if (backgroundArgb == null) 0 else 1
        val required = library.sfstudio_compose_snippet(
            name,
            name.size.toLong(),
            modeId,
            tintArgb,
            tintFlag,
            background,
            backgroundFlag,
            null,
            0,
        )
        check(required >= 0) { "Rust snippet generation failed with code $required" }
        check(required <= Int.MAX_VALUE) { "Rust snippet is too large for the JVM" }

        val output = ByteArray(required.toInt())
        val written = library.sfstudio_compose_snippet(
            name,
            name.size.toLong(),
            modeId,
            tintArgb,
            tintFlag,
            background,
            backgroundFlag,
            output,
            output.size.toLong(),
        )
        check(written == required) { "Rust snippet wrote $written bytes; expected $required" }
        return output.decodeToString()
    }

    fun toggleFavorites(favoritesJson: String, appleName: String): String? {
        val library = native ?: return null
        val json = favoritesJson.toByteArray(Charsets.UTF_8)
        val name = appleName.toByteArray(Charsets.UTF_8)
        val required = library.sfstudio_favorites_toggle_json(
            json,
            json.size.toLong(),
            name,
            name.size.toLong(),
            null,
            0,
        )
        check(required >= 0) { "Rust favorites toggle failed with code $required" }
        check(required <= Int.MAX_VALUE) { "Rust favorites result is too large for the JVM" }

        val output = ByteArray(required.toInt())
        val written = library.sfstudio_favorites_toggle_json(
            json,
            json.size.toLong(),
            name,
            name.size.toLong(),
            output,
            output.size.toLong(),
        )
        check(written == required) { "Rust favorites wrote $written bytes; expected $required" }
        return output.decodeToString()
    }

    fun mcpClientConfig(appPath: String?, developmentLibPath: String): String? {
        val library = native ?: return null
        val appPathBytes = appPath?.toByteArray(Charsets.UTF_8)
        val developmentLibPathBytes = developmentLibPath.toByteArray(Charsets.UTF_8)
        val required = library.sfstudio_mcp_client_config_json(
            appPathBytes,
            appPathBytes?.size?.toLong() ?: 0,
            if (appPath == null) 0 else 1,
            developmentLibPathBytes,
            developmentLibPathBytes.size.toLong(),
            null,
            0,
        )
        check(required >= 0) { "Rust MCP client config failed with code $required" }
        check(required <= Int.MAX_VALUE) { "Rust MCP client config is too large for the JVM" }

        val output = ByteArray(required.toInt())
        val written = library.sfstudio_mcp_client_config_json(
            appPathBytes,
            appPathBytes?.size?.toLong() ?: 0,
            if (appPath == null) 0 else 1,
            developmentLibPathBytes,
            developmentLibPathBytes.size.toLong(),
            output,
            output.size.toLong(),
        )
        check(written == required) { "Rust MCP client config wrote $written bytes; expected $required" }
        return output.decodeToString()
    }

    private fun loadNative(): NativeSearch? {
        return try {
            val explicitPath = System.getProperty("sfstudio.rust.search.library")
            if (!explicitPath.isNullOrBlank()) {
                Native.load(explicitPath, NativeSearch::class.java)
            } else {
                val osName = System.getProperty("os.name")
                val libraryName = when {
                    osName.startsWith("Windows", ignoreCase = true) -> "sfstudio_ffi.dll"
                    osName.contains("mac", ignoreCase = true) -> "libsfstudio_ffi.dylib"
                    else -> "libsfstudio_ffi.so"
                }
                val resource = javaClass.getResourceAsStream("/native/$libraryName") ?: return null
                val suffix = libraryName.substringAfterLast('.')
                val extracted = Files.createTempFile("sfstudio-rust-search-", ".$suffix")
                extracted.toFile().deleteOnExit()
                resource.use { Files.copy(it, extracted, StandardCopyOption.REPLACE_EXISTING) }
                Native.load(extracted.toAbsolutePath().toString(), NativeSearch::class.java)
            }
        } catch (error: Exception) {
            warn(error)
            null
        } catch (error: LinkageError) {
            warn(error)
            null
        }
    }

    private fun warn(error: Throwable) {
        if (warned.compareAndSet(false, true)) {
            System.err.println("Rust native module could not load; Kotlin fallback will be used: ${error.message}")
        }
    }
}
