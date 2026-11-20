package com.sfsymbols.data

import com.composables.sfsymbols.SfSymbolMetadata
import com.composables.sfsymbols.SfSymbolsCatalog
import com.sun.jna.Library
import com.sun.jna.Native
import kotlinx.serialization.builtins.SetSerializer
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonPrimitive
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.util.concurrent.atomic.AtomicBoolean

internal interface NativeSearch : Library {
    fun sfstudio_search_json(
        query: ByteArray?,
        queryLength: Long,
        limit: Long,
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
}

internal object RustCatalogSearch {
    private val warned = AtomicBoolean()

    private val symbolsByAppleName by lazy { SfSymbolsCatalog.all.associateBy { it.appleName } }
    private val native: NativeSearch? by lazy { loadNative() }

    fun available(): Boolean = native != null

    fun reportFailure(error: Throwable) = warn(error)

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
