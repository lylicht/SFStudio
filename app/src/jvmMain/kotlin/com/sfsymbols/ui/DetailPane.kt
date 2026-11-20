package com.sfsymbols.ui

import androidx.compose.animation.Crossfade
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.composables.sfsymbols.SfEffectRepeat
import com.composables.sfsymbols.SfRenderingMode
import com.composables.sfsymbols.SfSymbol
import com.composables.sfsymbols.SfSymbolMetadata
import com.composables.sfsymbols.SfSymbolEffect
import com.composables.sfsymbols.SfSymbolIcon
import com.composables.sfsymbols.SfWeight
import com.sfsymbols.data.SfSymbolIconResolver
import com.sfsymbols.data.SymbolMode
import com.sfsymbols.data.SymbolTags
import com.sfsymbols.viewmodel.CatalogViewModel
import dev.nucleusframework.macoscompose.components.ColorGrid
import dev.nucleusframework.macoscompose.components.GroupBox
import dev.nucleusframework.macoscompose.components.PushButton
import dev.nucleusframework.macoscompose.components.SegmentedControl
import dev.nucleusframework.macoscompose.components.Surface
import dev.nucleusframework.macoscompose.components.Switch
import dev.nucleusframework.macoscompose.components.Text
import dev.nucleusframework.macoscompose.theme.GlassMaterialSize
import dev.nucleusframework.macoscompose.theme.MacosTheme

private val BACKGROUND_PALETTE = listOf(
    null to "None",
    Color.White to "White",
    Color(0xFF1E1F22) to "Dark",
    Color(0xFF2E3A52) to "Slate",
    Color(0xFFFFE9B8) to "Cream",
    Color(0xFFDCE9D3) to "Mint",
    Color(0xFFF2D3CD) to "Rose",
)

private const val TAB_COLOR = 0
private const val TAB_BG = 1
private const val TAB_INFO = 2

private data class AppleEffectOption(
    val label: String,
    val effect: SfSymbolEffect,
    val held: Boolean = false,
    val continuous: Boolean = false,
)

private val APPLE_EFFECTS = listOf(
    AppleEffectOption("Appear", SfSymbolEffect.Appear(), held = true),
    AppleEffectOption("Disappear", SfSymbolEffect.Disappear(), held = true),
    AppleEffectOption("Bounce", SfSymbolEffect.Bounce()),
    AppleEffectOption("Pulse", SfSymbolEffect.Pulse()),
    AppleEffectOption("Variable Color", SfSymbolEffect.VariableColor()),
    AppleEffectOption("Scale", SfSymbolEffect.Scale(), held = true),
    AppleEffectOption("Wiggle", SfSymbolEffect.Wiggle()),
    AppleEffectOption("Rotate", SfSymbolEffect.Rotate()),
    AppleEffectOption("Breathe", SfSymbolEffect.Breathe(), continuous = true),
    AppleEffectOption("Draw On", SfSymbolEffect.DrawOn(), held = true),
    AppleEffectOption("Draw Off", SfSymbolEffect.DrawOff(), held = true),
)

/**
 * Trailing inspector pane: a 3-tab preview segmented control
 * (Color / BG / Basic info) above a centered preview image + box.
 */
@Composable
public fun DetailPane(
    viewModel: CatalogViewModel,
    modifier: Modifier = Modifier,
) {
    val cs = MacosTheme.colorScheme
    val meta = viewModel.selectedSymbol ?: return
    var tab by remember { mutableIntStateOf(TAB_COLOR) }

    Surface(
        modifier = modifier,
        shape = RoundedCornerShape(14.dp),
        materialSize = GlassMaterialSize.Medium,
        tintColor = cs.background.copy(alpha = 0.6f),
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            // 3-tab preview segmented control
            SegmentedControl(
                options = listOf("Color", "BG", "Basic info"),
                selectedIndex = tab,
                onSelectedIndexChange = { tab = it },
            )

            Spacer(Modifier.height(16.dp))

            AppleSymbolPreview(meta, viewModel)

            Spacer(Modifier.height(16.dp))

            when (tab) {
                TAB_COLOR -> ColorTab(viewModel)
                TAB_BG -> BgTab(viewModel)
                TAB_INFO -> InfoTab(viewModel, meta)
            }

            Spacer(Modifier.height(12.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                PushButton(onClick = { viewModel.copyAppleName(meta) }) {
                    Text("Copy Name")
                }
                PushButton(onClick = { viewModel.copyCodeSnippet(meta) }) {
                    Text("Copy Code")
                }
            }

            viewModel.badgeText?.let { badge ->
                Spacer(Modifier.height(16.dp))
                Text(text = badge, color = cs.success)
            }
        }
    }
}

@Composable
private fun AppleSymbolPreview(meta: SfSymbolMetadata, viewModel: CatalogViewModel) {
    val cs = MacosTheme.colorScheme
    var effectIndex by remember { mutableIntStateOf(2) }
    var menuOpen by remember { mutableStateOf(false) }
    var animationRequested by remember(meta) { mutableStateOf(false) }
    var animationActive by remember(meta) { mutableStateOf(false) }
    var trigger by remember(meta) { mutableIntStateOf(0) }
    val option = APPLE_EFFECTS[effectIndex]

    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(240.dp),
        ) {
            Box(
                modifier = Modifier
                    .align(Alignment.Center)
                    .size(240.dp)
                    .clip(RoundedCornerShape(10.dp))
                    .background(viewModel.backgroundColor ?: cs.surfaceContainerLow)
                    .border(1.dp, cs.borderSubtle, RoundedCornerShape(10.dp)),
                contentAlignment = Alignment.Center,
            ) {
                Crossfade(targetState = meta, label = "preview-symbol") { previewMeta ->
                    val symbol = remember(previewMeta) { SfSymbol.named(previewMeta.appleName) }
                    if (animationRequested && symbol != null) {
                        SfSymbolIcon(
                            symbol = symbol,
                            size = 110.dp,
                            weight = SfWeight.Regular,
                            mode = when {
                                viewModel.mode == SymbolMode.Monochrome || viewModel.tintIcon -> SfRenderingMode.Monochrome
                                else -> SfRenderingMode.Multicolor
                            },
                            tint = if (viewModel.tintIcon) viewModel.iconColor else Color.Black,
                            effect = option.effect,
                            repeat = if (option.continuous) SfEffectRepeat.Continuous else null,
                            trigger = trigger,
                            active = animationActive,
                            contentDescription = previewMeta.appleName,
                        )
                    } else {
                        val vector = remember(previewMeta, viewModel.mode) {
                            SfSymbolIconResolver.resolve(previewMeta, viewModel.mode)
                        }
                        if (vector != null) {
                            Icon(
                                imageVector = vector,
                                contentDescription = previewMeta.appleName,
                                tint = if (viewModel.tintIcon) viewModel.iconColor else Color.Unspecified,
                                modifier = Modifier.size(110.dp),
                            )
                        }
                    }
                }
            }
        }

        Spacer(Modifier.height(10.dp))
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            Box {
                PushButton(onClick = { menuOpen = true }) {
                    Text("Effect: ${option.label} ▾")
                }
                DropdownMenu(expanded = menuOpen, onDismissRequest = { menuOpen = false }) {
                    APPLE_EFFECTS.forEachIndexed { index, item ->
                        DropdownMenuItem(
                            text = { Text(item.label) },
                            onClick = {
                                effectIndex = index
                                menuOpen = false
                                animationRequested = false
                                animationActive = false
                                trigger = 0
                            },
                        )
                    }
                }
            }
            PushButton(onClick = {
                animationRequested = true
                if (option.held || option.continuous) {
                    if (animationActive) animationActive = false
                    else {
                        trigger++
                        animationActive = true
                    }
                } else {
                    trigger++
                    animationActive = true
                }
            }) {
                Text(
                    when {
                        option.continuous && animationActive -> "Stop"
                        option.held && animationActive -> "Reset"
                        else -> "Play"
                    },
                )
            }
        }
    }
}

@Composable
private fun ColorTab(viewModel: CatalogViewModel) {
    val cs = MacosTheme.colorScheme
    GroupBox(label = "Appearance", modifier = Modifier.fillMaxWidth()) {
        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = "Tint icon",
                    color = cs.textPrimary,
                    modifier = Modifier.weight(1f),
                )
                Switch(
                    checked = viewModel.tintIcon,
                    onCheckedChange = viewModel::updateTintIcon,
                )
            }
            if (viewModel.tintIcon) {
                ColorGrid(
                    selectedColor = viewModel.iconColor,
                    onColorSelected = viewModel::updateIconColor,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
    }
}

@Composable
private fun BgTab(viewModel: CatalogViewModel) {
    GroupBox(label = "Background", modifier = Modifier.fillMaxWidth()) {
        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                BACKGROUND_PALETTE.forEach { (color, label) ->
                    BackgroundSwatch(
                        color = color,
                        label = label,
                        selected = viewModel.backgroundColor == color,
                        onClick = { viewModel.updateBackground(color) },
                    )
                }
            }
        }
    }
}

@Composable
private fun InfoTab(viewModel: CatalogViewModel, meta: SfSymbolMetadata) {
    val cs = MacosTheme.colorScheme
    GroupBox(label = "Basic info", modifier = Modifier.fillMaxWidth()) {
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(text = meta.appleName, color = cs.textPrimary)
            Text(text = "SfSymbols.${meta.pascalName}", color = cs.textSecondary)
            Text(text = meta.categories.joinToString(" · "), color = cs.textTertiary)
            val tags = remember(meta) { SymbolTags.tagsFor(meta.appleName) }
            if (tags.isNotEmpty()) {
                Text(text = "Tags: " + tags.joinToString(", "), color = cs.textTertiary)
            }
            Spacer(Modifier.height(4.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                PushButton(onClick = { viewModel.toggleFavorite(meta) }) {
                    FavoriteIcon(viewModel.isFavorite(meta), cs.textSecondary, Modifier.size(14.dp))
                }
            }
        }
    }
}

@Composable
private fun BackgroundSwatch(
    color: Color?,
    label: String,
    selected: Boolean,
    onClick: () -> Unit,
) {
    val cs = MacosTheme.colorScheme
    val border = if (selected) cs.accent else cs.borderSubtle
    Column(
        modifier = Modifier.clickable(onClick = onClick).padding(2.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Box(
            modifier = Modifier
                .size(26.dp)
                .clip(CircleShape)
                .background(color ?: cs.surfaceContainerLowest)
                .border(1.dp, border, CircleShape),
        )
        Text(
            text = label,
            color = cs.textTertiary,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
