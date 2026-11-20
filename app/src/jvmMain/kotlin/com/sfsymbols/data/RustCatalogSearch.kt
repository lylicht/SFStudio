package com.sfsymbols.data

import com.composables.sfsymbols.SfSymbolMetadata
import com.composables.sfsymbols.SfSymbolsCatalog
import com.sun.jna.Library
import com.sun.jna.Native
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
        return Json.parseToJsonElement(output.decodeToString()).jsonArray.map { item ->
            val name = item.jsonPrimitive.content
            checkNotNull(symbolsByAppleName[name]) { "Rust search returned unknown symbol '$name'" }
        }
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
            System.err.println("Rust catalog search could not load; Kotlin fallback will be used: ${error.message}")
        }
    }
}
