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
`~/.sf-symbols-catalog/pins.json`.
The Rust favorites module currently handles ordered state and JSON only; it
does not read or write this file.

## Compatibility rules

- Keep the existing Kotlin sources, published coordinates, names, signatures,
  and target variants operational during migration.
- Do not change installer IDs, user-data paths, or JSON formats as part of a
  Rust-only change.
- Treat Kotlin behavior and generated symbol sources as the reference until a
  Rust replacement passes per-target behavioral and visual checks.
- Do not claim that current Rust code backs KMP callers: the first Rust slice
  is a separate core and opt-in MCP executable, with no FFI integration yet.
- A Rust bridge must sit behind existing public Kotlin APIs; Rust types and
  ABI details must not leak into downstream Compose source.

## Per-target bridge status

| Target | Existing implementation | Rust bridge status |
| --- | --- | --- |
| Android | Kotlin Multiplatform + Compose | `NOT_STARTED`; keep the Kotlin path |
| JVM / desktop | Kotlin Multiplatform + Compose | `NOT_STARTED`; Rust MCP remains opt-in |
| iOS device and simulator | Kotlin Multiplatform + Compose | `NOT_STARTED`; evaluate a narrow C ABI / generated Swift bridge |
| JS | Kotlin/JS + Compose | `NOT_STARTED`; evaluate wasm-bindgen or a JS-compatible interface |
| Wasm JS | Kotlin/Wasm + Compose | `NOT_STARTED`; evaluate a Wasm-compatible interface |

One FFI mechanism is not assumed to serve all targets. Each bridge needs an
integration test through the existing public API on that target before any
Kotlin implementation can be retired.

## Data and licensing

The Rust catalog currently reads the existing Kotlin catalog and JVM alias
file at compile time. This avoids a second hand-maintained metadata source but
does not port the generated vector assets or their renderer. The 7,007 symbol
definitions remain in the original KMP library. Do not extract or redistribute
Apple-owned symbol assets beyond the repository's existing permitted scope.
