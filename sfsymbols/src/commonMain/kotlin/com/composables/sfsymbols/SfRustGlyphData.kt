package com.composables.sfsymbols

import com.composables.sfsymbols.core.SfGlyph
import com.composables.sfsymbols.core.SfScaleId

/** Return the Rust-decoded outline and layer tree, or null when this target cannot load Rust. */
internal expect fun sfRustGlyphData(name: String, scale: SfScaleId): SfGlyph?
