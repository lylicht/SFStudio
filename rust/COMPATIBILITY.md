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
Kotlin. Packaged desktop search, snippet generation, and favorite toggling call
the Rust C ABI through JNA, with Kotlin fallbacks. Kotlin still owns favorite
file reads and writes.

The shared catalog search and filters, symbol metadata lookup, Compose code
generator, ordered favorite toggle, and glyph geometry decoder export C ABI
entry points without exposing Rust types. The header is
`include/sfstudio.h`; `sfstudio_glyph_json` returns raw weight masters, points
interpolated at a requested `wght` value, and tight visible bounds at an optical
scale. `sfstudio_animation_json` evaluates Apple-style effect frames and can
optionally resolve per-layer monochrome, hierarchical, palette, or multicolor
appearance plus automatic gradient stops. Bounds match Kotlin over ten symbols,
all nine weights, and all three optical scales. C, C++,
Python, JNI/JNA, Kotlin/Native, Swift, and other runtimes with a C FFI can call
these entry points.
This ABI is a language-neutral integration surface, not a claim that each
language or platform already has a packaged adapter or complete app binding.
The published KMP API does not call this ABI yet; its Android, iOS, JS, and
Wasm implementations remain active. The desktop JVM app calls search, code
generation, favorite toggling, and category/favorites filtering through JNA on
Windows, macOS, and Linux. The JNA metadata lookup binding passed parity checks
on all three operating systems in run #20. JS and Wasm consumers still use
Kotlin until target-specific Rust/Wasm adapters pass their tests. Glyph
geometry C ABI smoke has passed locally on Windows; the current endpoint has
not yet run in cross-OS CI or through platform adapters.

## Compatibility rules

- Keep the existing Kotlin sources, published coordinates, names, signatures,
  and target variants operational during migration.
- Do not change installer IDs, user-data paths, or JSON formats as part of a
  Rust-only change.
- Treat Kotlin behavior and generated symbol sources as the reference until a
  Rust replacement passes per-target behavioral and visual checks.
- Do not claim that current Rust code backs KMP callers: the Rust core has no
  FFI integration into the published Kotlin APIs yet. Packaged desktop MCP
  launches use Rust; development classpath launches retain Kotlin.
- A Rust bridge must sit behind existing public Kotlin APIs; Rust types and
  ABI details must not leak into downstream Compose source.

## Per-target bridge status

| Target | Existing implementation | Rust bridge status |
| --- | --- | --- |
| Android | Kotlin Multiplatform + Compose | `NOT_STARTED`; keep the Kotlin path |
| JVM / desktop | Kotlin Multiplatform + Compose; packaged search, filtering, snippets, favorites, and `--mcp` use Rust | `IN_PROGRESS`; JNA search/snippet/favorite/filter and metadata lookup parity passed Windows/macOS/Linux; published KMP APIs, UI state, and rendering remain Kotlin |
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
differential fixtures cover effect timing and layer transforms; rendering color
calculations currently have Rust unit and C ABI smoke coverage, but no Kotlin
differential fixture. Kotlin remains the active visual renderer; Rust renderer
integration, draw-rail clipping, visual rendering parity, broader morph
fixtures, and mobile/JS/Wasm adapters are still outstanding. The published
symbol definitions and KMP API remain available. Do not extract or redistribute
Apple-owned symbol assets beyond the repository's existing permitted scope.
