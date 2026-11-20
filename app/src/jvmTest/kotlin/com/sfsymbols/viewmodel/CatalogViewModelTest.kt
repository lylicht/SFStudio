package com.sfsymbols.viewmodel

import androidx.compose.runtime.snapshots.Snapshot
import androidx.compose.runtime.snapshots.SnapshotStateObserver
import androidx.compose.ui.graphics.Color
import com.composables.sfsymbols.SfSymbolsCatalog
import com.sfsymbols.data.SymbolMode
import kotlin.test.*

class CatalogViewModelTest {
    private val heart = SfSymbolsCatalog.all.first { it.appleName == "heart.fill" }

    @Test fun `category changes invalidate the observed grid results`() {
        val vm = CatalogViewModel { }
        val category = vm.categories.first().first
        var invalidations = 0
        val observer = SnapshotStateObserver { it() }
        observer.start()
        try {
            observer.observeReads(Any(), { _: Any -> invalidations++ }) { vm.filteredSymbols }
            vm.filterCategory(category)
            Snapshot.sendApplyNotifications()
            assertTrue(invalidations > 0, "The grid must be notified when its result list changes")
            assertTrue(vm.filteredSymbols.isNotEmpty())
            assertTrue(vm.filteredSymbols.all { category in it.categories })
            vm.showAll()
            assertEquals(SfSymbolsCatalog.all, vm.filteredSymbols)
        } finally { observer.stop() }
    }

    @Test fun `copy actions write names and full styled Compose code`() {
        val copied = mutableListOf<String>()
        val vm = CatalogViewModel { copied.add(it) }
        vm.copyAppleName(heart)
        vm.copyPascalName(heart)
        assertEquals(listOf("heart.fill", "SFHeartFill"), copied)
        vm.updateMode(SymbolMode.Monochrome)
        vm.updateIconColor(Color(0x80E0245E))
        vm.updateBackground(Color(0xFFDCE9D3))
        vm.copyCodeSnippet(heart)
        assertEquals(
            javaClass.getResource("/compose-snippet-monochrome-background.kt.txt")!!.readText().replace("\r\n", "\n"),
            copied.last(),
        )
        vm.updateTintIcon(false)
        vm.updateMode(SymbolMode.Dualtone)
        vm.updateBackground(null)
        vm.copyCodeSnippet(heart)
        assertEquals(
            javaClass.getResource("/compose-snippet-dualtone-plain.kt.txt")!!.readText().replace("\r\n", "\n"),
            copied.last(),
        )
    }

    @Test fun `clipboard failures do not report success`() {
        val vm = CatalogViewModel { throw IllegalStateException("clipboard busy") }
        vm.copyAppleName(heart)
        assertEquals("Clipboard unavailable. Try copying again.", vm.badgeText)
    }

    @Test fun `copying MCP configuration uses shared clipboard feedback`() {
        var copied: String? = null
        val vm = CatalogViewModel { copied = it }
        val config = "{\"mcpServers\":{\"sfstudio\":{}}}"

        vm.copyMcpConfig(config)

        assertEquals(config, copied)
        assertEquals("Copied MCP configuration", vm.badgeText)
    }
}
