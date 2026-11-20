# Migration status

This table tracks the Rust implementation separately from compatibility code
that must remain available to existing Kotlin Multiplatform consumers.

| Area | Current implementation | Rust target | State / required gate |
| --- | --- | --- | --- |
| Symbol names, categories, restricted flags, aliases | KMP catalog and JVM tag file; Rust reads those same sources at compile time | `sfstudio-core` | `PORTED`; catalog count and basic lookup tests pass; Rust builds on Windows/macOS/Linux |
| Fuzzy query ranking | Kotlin search remains the fallback and reference | `sfstudio-core` via C ABI | `VERIFIED`; 37-query JNA parity passed on Windows/macOS/Linux in Actions run #14 |
| Category and favorites filtering | `CatalogViewModel.kt` retains a Kotlin fallback | `sfstudio-core::Catalog::search_filtered` via C ABI | `VERIFIED`; exact Kotlin/Rust parity and cross-OS JNA tests pass for category, favorites, blank, typo, and Unicode queries in run #17 |
| Cross-language core entry point | Rust core had no stable foreign-language ABI | `sfstudio-ffi` C ABI + `include/sfstudio.h` | `IN_PROGRESS`; search, filtering, codegen, favorites, metadata, glyph geometry, and visible bounds are exposed; Python glyph smoke is local, while target-specific adapters and cross-OS glyph checks remain pending |
| Symbol metadata lookup | Kotlin catalog lookup is the reference | `sfstudio-core::Catalog::find` via `sfstudio_symbol_json` C ABI | `VERIFIED`; Rust and Python C ABI smoke and JVM parity passed Windows/macOS/Linux in run #20 |
| Vector data, geometry, and visible bounds | Generated Kotlin blobs are authoritative | `sfstudio-core::vector` | `IN_PROGRESS`; exact fingerprints match all 8,526 glyphs and available optical variants; visible bounds match Kotlin across ten symbols, all nine weights, and all three scales |
| Desktop search caller | `FuzzySearch.kt` and `CatalogViewModel.kt` retain fallbacks | Rust search and filtered search via JNA C ABI | `VERIFIED`; search and filtering passed Windows/macOS/Linux in runs #14/#17 |
| JVM metadata lookup binding | Kotlin catalog lookup is the reference | Rust metadata lookup via JNA C ABI | `VERIFIED`; parity passed Windows/macOS/Linux in run #20; not yet used by a production UI caller |
| MCP server | Kotlin `McpServer.kt` remains the development fallback | `sfstudio-mcp`; packaged `--mcp` launcher delegates to it | `VERIFIED`; 99 Kotlin/Rust response cases match locally; packaged smoke passes on Windows, macOS, and Linux |
| Favorites model and JSON codec | `PinnedSymbols.kt` remains the active disk reader/writer at `~/.sf-symbols-catalog/pins.json` | `sfstudio-core::favorites` via C ABI | `VERIFIED`; desktop toggle parity passed Windows/macOS/Linux in run #16; Kotlin still owns persistence |
| Compose snippet generation | Kotlin implementation remains the fallback and reference | `sfstudio-core::codegen` via C ABI | `VERIFIED`; Rust matches both golden fixtures and JNA parity passed Windows/macOS/Linux in run #16 |
| UI state and filtering | `CatalogViewModel.kt` | Rust state API and per-target adapters | `NOT_STARTED`; verify state transitions and all consumers |
| Desktop UI | Kotlin/Compose | Rust-native UI if parity gates pass | `RETAINED_FOR_COMPATIBILITY`; visual and interaction baselines required |
| Vector rendering and symbol effects | KMP generated sources and renderer | Rust geometry decoder, then renderer/effects | `IN_PROGRESS`; geometry, layers, optical variants, and visible bounds decode in Rust; rendering modes, visual effects, and animation parity remain pending |
| Published KMP API | `com.composables.sfsymbols` Kotlin API | Rust-backed adapters by target where feasible | `RETAINED_FOR_COMPATIBILITY`; Android/JVM/iOS/JS/Wasm remain unchanged |
| Installers and release identities | Compose Desktop packaging | Rust desktop packaging after upgrade proof | `RETAINED_FOR_COMPATIBILITY`; Windows UUID, macOS bundle ID and Debian package ID unchanged |
| Cross-platform verification | Existing Gradle targets and Rust workspace | Rust desktop matrix plus existing KMP jobs | `IN_PROGRESS`; search, codegen, favorites, and filtering passed Windows/macOS/Linux in runs #13/#14/#16/#17; KMP target builds pass, but Rust-backed platform adapters remain pending |
| Desktop GUI cutover | Kotlin/Compose desktop app | Rust-native desktop app | `NOT_STARTED`; UI, vector, and interaction parity gates required |

The Rust MCP binary is bundled into standalone MCP assemblies and desktop
installers. Packaged `--mcp` invocations delegate to it; classpath-based Gradle
launches keep the Kotlin implementation as a development fallback.
