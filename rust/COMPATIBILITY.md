# Compatibility contract

## Current supported consumers

The published library is Kotlin Multiplatform. Its Gradle targets are Android,
JVM, iOS ARM64, iOS Simulator ARM64, JS (browser and Node.js), and Wasm JS
(browser and Node.js). Its public package and APIs remain under
`com.composables.sfsymbols`, including `SfSymbol`, `SfSymbolIcon`, weights,
scales, rendering modes, effects, metadata, and the generated monochrome and
dualtone vectors.

The desktop application is Kotlin/Compose on JVM. The installed app remains
the default MCP endpoint and continues to use the existing favorite file at
`~/.sf-symbols-catalog/pins.json`. Packaged `--mcp` invocations delegate to
the bundled Rust MCP process; development classpath launches fall back to
Kotlin. Packaged desktop search, snippet generation, favorite toggling,
filter-selection transitions, and MCP client configuration call the Rust C ABI
through JNA, with Kotlin fallbacks.
Kotlin still owns favorite file reads and writes.

The JVM implementation of the public `SfSymbolIcon` now evaluates Apple effect
frames, layer colors, and automatic gradient stops through the same Rust C ABI
when the native library is available and colors use sRGB. Compose still
rasterizes paths and applies the results; wide-gamut colors and unavailable
native libraries use Kotlin. Android, iOS, JS, and Wasm keep using the Kotlin
evaluator until their Rust adapters are implemented.

The shared catalog search and filters, filter-selection transitions, symbol
metadata lookup, Compose code generator, ordered favorite toggle, glyph
geometry decoder, and animation evaluator export C ABI entry points without
exposing Rust types. The header is
`include/sfstudio.h`; `sfstudio_glyph_json` returns raw weight masters, points
interpolated at a requested `wght` value, and tight visible bounds at an optical
scale. `sfstudio_animation_json` evaluates Apple-style effect frames and can
optionally resolve per-layer monochrome, hierarchical, palette, or multicolor
appearance plus automatic gradient stops, including Draw On/Off By Layer and
Individually scheduling. `sfstudio_draw_field_json` accepts a
platform-rasterized 112x112 layer mask and returns the 48-step rail-guided
reveal level for each pixel. `sfstudio_mcp_client_config_json` builds the JSON
shown in desktop Settings. Bounds match Kotlin over ten symbols, all nine
weights, and all three optical scales. C, C++,
Python, JNI/JNA, Kotlin/Native, Swift, and other runtimes with a C FFI can call
these entry points.
This ABI is a language-neutral integration surface, not a claim that each
language or platform already has a packaged adapter or complete app binding.
The JVM `SfSymbolIcon` calls the animation endpoint when its native library is
available; Android, iOS, JS, and Wasm retain the Kotlin evaluator. The desktop
JVM app calls search, code generation, favorite toggling, category/favorites
filtering, filter-selection transitions, MCP client configuration, and effect
evaluation through JNA. The JNA metadata lookup binding passed parity checks on
Windows, macOS, and Linux in run #20. Rust animation frames match the Kotlin
evaluator in the JVM parity test; sRGB appearance and gradients pass differential
checks in all four rendering modes, and Compose render tests pass with Rust
enabled. JS and Wasm consumers still use Kotlin until target-specific Rust/Wasm
adapters pass their tests. The vector geometry
and animation C ABI smoke checks passed on Windows, macOS, and Linux. Run #34
passed the filter-selection C ABI on those operating systems and retained the
KMP builds.

## Compatibility rules

- Keep the existing Kotlin sources, published coordinates, names, signatures,
  and target variants operational during migration.
- Do not change installer IDs, user-data paths, or JSON formats as part of a
  Rust-only change.
- Treat Kotlin behavior and generated symbol sources as the reference; retain
  the Kotlin fallback on targets without a tested Rust adapter.
- JVM effect frames use Rust when the optional native library loads. The
  Compose renderer still draws the paths. Packaged desktop MCP launches use
  Rust; development classpath launches retain Kotlin.
- A Rust bridge must sit behind existing public Kotlin APIs; Rust types and
  ABI details must not leak into downstream Compose source.

## Per-target bridge status

| Target | Existing implementation | Rust bridge status |
| --- | --- | --- |
| Android | Kotlin Multiplatform + Compose | `NOT_STARTED`; keep the Kotlin path |
| JVM / desktop | Kotlin Multiplatform + Compose; packaged search, filtering, snippets, favorites, MCP config, animation frames, and `--mcp` use Rust when available | `IN_PROGRESS`; JNA search/snippet/favorite/filter and metadata lookup parity passed Windows/macOS/Linux; MCP config has local Kotlin parity; Apple animation frames pass native/Kotlin parity and the Compose suite; Compose still rasterizes symbol paths |
| iOS device and simulator | Kotlin Multiplatform + Compose | `NOT_STARTED`; evaluate a narrow C ABI / generated Swift bridge |
| JS | Kotlin/JS + Compose | `NOT_STARTED`; evaluate wasm-bindgen or a JS-compatible interface |
| Wasm JS | Kotlin/Wasm + Compose | `NOT_STARTED`; evaluate a Wasm-compatible interface |

One FFI mechanism is not assumed to serve all targets. Each bridge needs an
integration test through the existing public API on that target before any
Kotlin implementation can be retired.

## Data and licensing

The Rust catalog and vector decoder read existing Kotlin catalog, alias, and
generated geometry sources at compile time. Geometry decoding now matches all
8,526 reference glyph fingerprints locally, including available optical
variants. Visible bounds match Kotlin on a ten-symbol matrix of all nine
weights and three optical scales. Apple-style effect timing, per-layer
transforms, variable-layer staging, replace recipes, layer color-mode
calculations, and automatic gradient stops have Rust core implementations and
JSON C ABI, along with tagged-layer Magic Replace morph paths. Selected Kotlin
differential fixtures cover effect timing and layer transforms, and the JVM
renderer now consumes Rust frames, sRGB layer colors, and gradient stops while
Compose continues to rasterize paths and clip geometry. Rendering color
calculations have Rust unit tests, Windows/macOS/Linux C ABI smoke coverage, and
JVM/Kotlin differential checks; wide-gamut color parity still uses the Kotlin
fallback.
Draw-field distance and level calculations are ported; platform path
rasterization and clip geometry remain in Kotlin. Pixel-by-pixel Rust/Kotlin
rendering comparison, broader morph fixtures, and mobile/JS/Wasm adapters are
outstanding. The published symbol definitions and KMP API remain available. Do
not extract or redistribute Apple-owned symbol assets beyond the repository's
existing permitted scope.
