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
Kotlin. Packaged desktop search, category-count aggregation, snippet generation,
favorite toggling and file storage, filter-selection transitions, MCP client
configuration, theme resolution, and grid-scale clamping call the Rust C ABI
through JNA, with Kotlin fallbacks. Kotlin resolves the existing favorites
path and keeps the fallback; Rust reads and writes the same ordered JSON format
when available. Kotlin still owns Java Preferences persistence.

The JVM implementation of the public `SfSymbolIcon` obtains weighted outline
points, Apple effect frames, and Draw On/Off reveal levels through the Rust C
ABI when the native library is available. It also requests Rust-resolved layer
colors and automatic gradient stops for sRGB colors. Android uses private JNI
bridges for points, effect frames, layer appearance, and draw-field levels,
with Kotlin fallback. iOS now has a Kotlin/Native C-interop path for weighted
outline points and Apple effect frames with sRGB layer appearance. Point
parity passed in run #44854205293, and simulator animation/appearance parity
passed in [run #44857761656](https://github.com/lylicht/SFStudio/actions/runs/44857761656).
iOS draw-field levels and horizontal spans now use the Rust C ABI; Compose
still rasterizes masks and materializes clip paths from those spans. JS and Wasm
now load the shared Rust Wasm C ABI for
weighted points, Apple effect frames, and draw-field levels/spans, with Kotlin
fallbacks while the browser fetches the module or when loading fails. JS Node
parity and local Wasm browser parity in Edge 157 pass; the browser loader uses
asynchronous instantiation for the large module. Published-consumer resource
loading remains unverified. Glyph data cached through Kotlin before the Rust
module finishes loading remains compatible but is not replaced in that session.
Wide-gamut colors use Kotlin.

The shared catalog search and filters, filter-selection transitions, theme
resolution, grid-scale clamping, symbol metadata lookup, Compose code
generator, ordered favorite toggle, glyph
geometry decoder and point interpolator, and animation evaluator export C ABI entry points without
exposing Rust types. The header is
`include/sfstudio.h`; `sfstudio_glyph_json` returns raw weight masters, points
interpolated at a requested `wght` value, tight visible bounds at an optical
scale, the hierarchical layer tree, and Rust-flattened animation-layer records
including contours, group assignments, tie points, and stroke lengths;
`sfstudio_glyph_points` returns just the interpolated screen-coordinate
points through a typed float buffer for renderer use. `sfstudio_animation_json` evaluates Apple-style effect frames and can
optionally resolve per-layer monochrome, hierarchical, palette, or multicolor
appearance plus automatic gradient stops, including Draw On/Off By Layer and
Individually scheduling. `sfstudio_draw_field_json` accepts a
platform-rasterized 112x112 layer mask and returns the 48-step rail-guided
reveal level for each pixel plus horizontal spans encoded as
`[level, y, x_start, x_end]` with inclusive x bounds.
`sfstudio_mcp_client_config_json` builds the JSON
shown in desktop Settings. Bounds match Kotlin over ten symbols, all nine
weights, and all three optical scales. C, C++,
Python, JNI/JNA, Kotlin/Native, Swift, and other runtimes with a C FFI can call
these entry points.
This ABI is a language-neutral integration surface, not a claim that each
language or platform already has a packaged adapter or complete app binding.
The JVM `SfSymbolIcon` calls the animation and draw-field endpoints when its
native library is available; Android calls the point, animation, and draw-field
endpoints through JNI; iOS calls the typed glyph-points and animation endpoints
through Kotlin/Native C interop. The desktop JVM app calls search, code generation, favorite toggling,
category/favorites filtering, filter-selection transitions, MCP client
configuration, theme resolution, grid-scale clamping, and effect evaluation
through JNA. Desktop favorite-file I/O also uses the C ABI on Windows, macOS,
and Linux, while Kotlin continues to resolve the existing path and provides
the fallback. The JNA metadata lookup
binding passed parity checks on Windows, macOS, and Linux in run #20. Rust
animation frames match the Kotlin evaluator in JVM parity tests; sRGB appearance
and gradients pass differential checks in all four rendering modes, and Compose
render tests pass with Rust enabled. The draw-field bridge matches the Kotlin
fixture for five real symbol layers, and Rust spans reconstruct those exact
pixel levels before Compose uses them to build reveal paths. JS and Wasm bundle the shared Rust Wasm
library and route rendering requests through the C ABI; JS Node parity covers
points, animation frames, and draw-field levels/spans. The Wasm browser parity
test passes locally in Edge 157, including geometry, optical scaling, points,
bounds, animation frames, and draw-field output. The Windows Wasm Node runner
still fails while loading Compose's separate `skiko.wasm` file before the Rust
parity test can run; published-consumer resource loading is also unverified.
Kotlin fallbacks stay in place. The vector geometry,
animation, and draw-field C ABI smoke checks passed on Windows, macOS, and Linux.
Run #34 passed the filter-selection C ABI on those operating systems and
retained the KMP builds.

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
| Android | Kotlin Multiplatform + Compose | `IN_PROGRESS`; weighted outline points, animation frames, layer appearance, and draw-field levels/spans use Rust through JNI with Kotlin fallback; the four-ABI AAR and device-test sources compile in [run #44885280359](https://github.com/lylicht/SFStudio/actions/runs/44885280359); point parity passes on an API 36.1 x86_64 emulator; new rendering runtime parity still needs a device run |
| JVM / desktop | Kotlin Multiplatform + Compose; packaged search, category counts, filtering, snippets, favorites, MCP config, glyph points, animation frames, draw-field levels/spans, and `--mcp` use Rust when available | `IN_PROGRESS`; JNA search/snippet/favorite/filter, metadata lookup, and category-count ordering/value parity passed Windows/macOS/Linux in run #44888335626; Rust glyph points pass exact Kotlin parity across representative symbols, all nine weights, and three optical scales; Apple animation frames and draw-field levels/spans pass native/Kotlin parity and the Compose suite; Compose rasterizes symbol masks and materializes `Path` clips |
| iOS device and simulator | Kotlin Multiplatform + Compose | `IN_PROGRESS`; weighted points, Apple effect frames with sRGB layer appearance, and draw-field levels/spans use Rust C-interop; point parity across ten symbols, nine weights, and three scales passed in [run #44854205293](https://github.com/lylicht/SFStudio/actions/runs/44854205293), animation/appearance parity passed in [run #44857761656](https://github.com/lylicht/SFStudio/actions/runs/44857761656), and the new response parser and draw-span parity passed simulator tests in [run #44885280359](https://github.com/lylicht/SFStudio/actions/runs/44885280359); physical-device runtime is unverified |
| JS | Kotlin/JS + Compose | `IN_PROGRESS`; packages the Rust Wasm C ABI as a Compose web resource and uses it for points, Apple animation frames, and draw-field levels/spans; JS Node parity passes locally; JS browser runtime and published-package loading remain unverified |
| Wasm JS | Kotlin/Wasm + Compose | `IN_PROGRESS`; shares the Rust Wasm web adapter; local Edge 157 browser parity passes for geometry, scale, points, bounds, animation frames, and draw-field levels/spans; Node execution is blocked before parity by Compose Skiko's `skiko.wasm` fetch; published-package loading remains unverified |

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
JSON C ABI, along with tagged-layer Magic Replace morph paths. The Rust
`sfstudio_glyph_points` C ABI supplies weighted, optical-scale screen
coordinates, which the JVM and Android `SfSymbol.points()` implementations
now use with exact Kotlin parity across ten representative symbols, all nine
weights, and all three optical scales. Android packages four ABI variants and
retains its Kotlin fallback when JNI is unavailable.
`sfstudio_glyph_bounds` supplies tight visible bounds in `[left, top, right,
bottom]` order; the public KMP `SfSymbol.bounds()` keeps its Kotlin fallback
and cache behavior while using the Rust result when the platform bridge is
available. `sfstudio_glyph_json` also supplies the flattened animation-layer
records used by the unchanged public `SfSymbol.layers` property. Its Kotlin
tree-flattening path remains the fallback when an older or unavailable bridge
does not provide those records.
Selected Kotlin differential fixtures cover effect timing and layer
transforms, and the JVM renderer consumes Rust points, frames, sRGB layer
colors, and gradient stops while Compose continues to build/rasterize paths
and clip geometry. Rendering color
calculations have Rust unit tests, Windows/macOS/Linux C ABI smoke coverage, and
JVM/Kotlin differential checks; wide-gamut color parity still uses the Kotlin
fallback.
Draw-field distance, level, and horizontal clip-span calculations are ported;
the JVM renderer uses these Rust results to materialize Compose clip geometry.
Platform path rasterization remains in Kotlin to preserve native antialiasing.
Android runtime parity for the new
animation, appearance, and draw-field adapters; iOS physical-device runtime;
pixel-by-pixel Rust/Kotlin rendering comparison;
broader morph fixtures; and JS/Wasm browser runtime validation are outstanding.
The published symbol definitions and KMP API remain available. Do not extract
or redistribute Apple-owned symbol assets beyond the repository's existing
permitted scope.
