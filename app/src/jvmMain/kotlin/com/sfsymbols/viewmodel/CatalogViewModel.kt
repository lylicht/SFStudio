package com.sfsymbols.viewmodel

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.Color
import com.composables.sfsymbols.SfSymbolMetadata
import com.composables.sfsymbols.SfSymbolsCatalog
import com.sfsymbols.data.FuzzySearch
import com.sfsymbols.data.PinnedSymbols
import com.sfsymbols.data.RustCatalogSearch
import com.sfsymbols.data.ThemeMode
import com.sfsymbols.data.SymbolMode
import com.sfsymbols.util.copyToClipboard
import com.sfsymbols.util.composeSnippet
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.util.prefs.Preferences

/**
 * Central state holder for the SF Symbols catalog.
 * Plain Compose state + coroutines (no Android ViewModel dependency).
 */
public class CatalogViewModel(
    private val clipboardWriter: (String) -> Unit = ::copyToClipboard,
) {

    private val scope = CoroutineScope(Dispatchers.Default)

    public enum class Screen { Catalog, Settings }

    private val preferences = runCatching { Preferences.userNodeForPackage(CatalogViewModel::class.java) }.getOrNull()

    public var themeMode: ThemeMode by mutableStateOf(
        runCatching { ThemeMode.valueOf(preferences?.get("themeMode", ThemeMode.System.name) ?: ThemeMode.System.name) }
            .getOrDefault(ThemeMode.System),
    )
        private set

    public var screen: Screen by mutableStateOf(Screen.Catalog)
        private set

    public var query: String by mutableStateOf("")
        private set

    public var selectedCategory: String? by mutableStateOf(null)
        private set

    public var onlyPinned: Boolean by mutableStateOf(false)
        private set

    public var mode: SymbolMode by mutableStateOf(SymbolMode.Dualtone)
        private set

    public var pinnedIds: Set<String> by mutableStateOf(PinnedSymbols.load())
        private set

    public var selectedSymbol: SfSymbolMetadata? by mutableStateOf(
        SfSymbolsCatalog.all.firstOrNull()
    )
        private set

    public var iconColor: Color by mutableStateOf(Color.White)
        private set

    public var backgroundColor: Color? by mutableStateOf(null)
        private set

    public var badgeText: String? by mutableStateOf(null)
        private set

    public var tintIcon: Boolean by mutableStateOf(true)
        private set

    public var gridScale: Float by mutableStateOf(1f)
        private set

    public var listView: Boolean by mutableStateOf(false)
        private set

    public fun toggleListView() {
        listView = !listView
    }

    private var badgeJob: Job? = null

    private val allSymbols by lazy { SfSymbolsCatalog.all }

    public val categories: List<Pair<String, Int>> by lazy {
        allSymbols
            .flatMap { it.categories }
            .groupingBy { it }
            .eachCount()
            .entries
            .sortedByDescending { it.value }
            .map { it.key to it.value }
    }

    /**
     * Result of the active query/category/pinned filter. Recomputed only when
     * one of those inputs actually changes, so recomposition stays free.
     */
    public var filteredSymbols: List<SfSymbolMetadata> by mutableStateOf(emptyList())
        private set

    public val pinnedSymbols: List<SfSymbolMetadata>
        get() = allSymbols.filter { it.appleName in pinnedIds }

    init {
        recomputeFiltered()
    }

    private fun recomputeFiltered() {
        val q = query.trim()
        val cat = selectedCategory
        if (cat != null || onlyPinned) {
            val rustResults = if (cat == "") null else try {
                RustCatalogSearch.searchFiltered(
                    q,
                    cat,
                    if (onlyPinned) pinnedIds else null,
                    if (q.isEmpty()) allSymbols.size else 300,
                )
            } catch (error: Exception) {
                RustCatalogSearch.reportFailure(error)
                null
            } catch (error: LinkageError) {
                RustCatalogSearch.reportFailure(error)
                null
            }
            if (rustResults != null) {
                filteredSymbols = rustResults
                return
            }
        }
        val base = when {
            onlyPinned -> allSymbols.filter { it.appleName in pinnedIds }
            else -> allSymbols
        }
        val scoped = if (cat == null) base else base.filter { it.categories.contains(cat) }
        filteredSymbols = when {
            q.isEmpty() -> scoped
            cat == null && !onlyPinned -> FuzzySearch.search(q, limit = 300)
            else -> {
                val allowed = scoped.mapTo(HashSet(scoped.size)) { it.appleName }
                FuzzySearch.search(q, limit = allSymbols.size).filter { it.appleName in allowed }.take(300)
            }
        }
    }

    public fun search(text: String) {
        query = text
        recomputeFiltered()
        if (selectedSymbol == null && filteredSymbols.isNotEmpty()) {
            selectedSymbol = filteredSymbols.first()
        }
    }

    public fun filterCategory(category: String?) {
        updateFilter(RustCatalogSearch.FilterAction.Category, category, category to false)
    }

    public fun showOnlyPinned() {
        updateFilter(RustCatalogSearch.FilterAction.Favorites, null, null to true)
    }

    public fun showAll() {
        updateFilter(RustCatalogSearch.FilterAction.All, null, null to false)
    }

    private fun updateFilter(
        action: RustCatalogSearch.FilterAction,
        category: String?,
        kotlinFallback: Pair<String?, Boolean>,
    ) {
        val selection = try {
            RustCatalogSearch.filterSelection(action, category)
        } catch (error: Exception) {
            RustCatalogSearch.reportFailure(error)
            null
        } catch (error: LinkageError) {
            RustCatalogSearch.reportFailure(error)
            null
        } ?: kotlinFallback
        selectedCategory = selection.first
        onlyPinned = selection.second
        recomputeFiltered()
    }

    public fun selectSymbol(meta: SfSymbolMetadata) {
        val rustMetadata = try {
            RustCatalogSearch.lookup(meta.appleName)
        } catch (error: Exception) {
            RustCatalogSearch.reportFailure(error)
            null
        } catch (error: LinkageError) {
            RustCatalogSearch.reportFailure(error)
            null
        }
        selectedSymbol = rustMetadata ?: meta
    }

    public fun updateMode(newMode: SymbolMode) {
        mode = newMode
    }

    public fun updateIconColor(color: Color) {
        iconColor = color
    }

    public fun updateBackground(color: Color?) {
        backgroundColor = color
    }

    public fun updateTintIcon(enabled: Boolean) {
        tintIcon = enabled
    }

    public fun navigate(screen: Screen) {
        this.screen = screen
    }

    public fun updateThemeMode(mode: ThemeMode) {
        themeMode = mode
        runCatching {
            preferences?.put("themeMode", mode.name)
            preferences?.flush()
        }
    }

    public fun resolveDarkTheme(systemIsDark: Boolean): Boolean {
        val rustResult = try {
            RustCatalogSearch.themeModeIsDark(themeMode, systemIsDark)
        } catch (error: Exception) {
            RustCatalogSearch.reportFailure(error)
            null
        } catch (error: LinkageError) {
            RustCatalogSearch.reportFailure(error)
            null
        }
        return rustResult ?: themeMode.isDark(systemIsDark)
    }

    public fun updateGridScale(scale: Float) {
        val rustScale = try {
            RustCatalogSearch.clampGridScale(scale)
        } catch (error: Exception) {
            RustCatalogSearch.reportFailure(error)
            null
        } catch (error: LinkageError) {
            RustCatalogSearch.reportFailure(error)
            null
        }
        gridScale = rustScale ?: scale.coerceIn(0.5f, 1.6f)
    }

    public fun toggleFavorite(meta: SfSymbolMetadata) {
        val next = PinnedSymbols.toggle(pinnedIds, meta.appleName)
        pinnedIds = next
        PinnedSymbols.save(next)
        recomputeFiltered()
    }

    public fun isFavorite(meta: SfSymbolMetadata): Boolean = meta.appleName in pinnedIds

    public fun copyAppleName(meta: SfSymbolMetadata) {
        copyText(meta.appleName, "Copied \"${meta.appleName}\"")
    }

    public fun copyPascalName(meta: SfSymbolMetadata) {
        copyText(meta.pascalName, "Copied \"${meta.pascalName}\"")
    }

    public fun copyCodeSnippet(meta: SfSymbolMetadata) {
        copyText(composeSnippet(meta, mode, iconColor, tintIcon, backgroundColor), "Copied Compose code")
    }

    public fun copyMcpConfig(config: String) {
        copyText(config, "Copied MCP configuration")
    }

    private fun copyText(text: String, success: String) {
        try {
            clipboardWriter(text)
            showBadge(success)
        } catch (_: Exception) {
            showBadge("Clipboard unavailable. Try copying again.")
        }
    }

    private fun showBadge(text: String) {
        badgeText = text
        badgeJob?.cancel()
        badgeJob = scope.launch {
            delay(1800)
            badgeText = null
        }
    }
}
