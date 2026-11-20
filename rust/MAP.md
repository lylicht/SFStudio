# Migration status

This table tracks the Rust implementation separately from compatibility code
that must remain available to existing Kotlin Multiplatform consumers.

| Area | Current implementation | Rust target | State / required gate |
| --- | --- | --- | --- |
| Symbol names, categories, restricted flags, aliases | KMP catalog and JVM tag file; Rust reads those same sources at compile time | `sfstudio-core` | `PORTED`; catalog count and basic lookup tests pass, target matrix pending |
| Fuzzy query ranking | `FuzzySearch.kt` remains the desktop reference | `sfstudio-core` | `PORTED`; 92 MCP response cases match locally, broader UI-query parity pending |
| MCP server | Kotlin `McpServer.kt` remains installed default | `sfstudio-mcp` | `PORTED`; 92 Kotlin/Rust response cases match locally; app `--mcp` cutover pending |
| Favorites model and JSON codec | `PinnedSymbols.kt` remains the active disk reader/writer at `~/.sf-symbols-catalog/pins.json` | `sfstudio-core::favorites` | `IN_PROGRESS`; ordered set semantics and Kotlin/Rust JSON fixtures pass; Rust disk adapter and integration pending |
| Compose snippet generation | `ComposeSnippet.kt` remains the active desktop path | `sfstudio-core::codegen` | `PORTED`; Kotlin and Rust match shared byte-for-byte fixtures for both background and plain variants; runtime bridge pending |
| UI state and filtering | `CatalogViewModel.kt` | Rust state API and per-target adapters | `NOT_STARTED`; verify state transitions and all consumers |
| Desktop UI | Kotlin/Compose | Rust-native UI if parity gates pass | `RETAINED_FOR_COMPATIBILITY`; visual and interaction baselines required |
| Vector rendering and symbol effects | KMP generated sources and renderer | Rust renderer/data | `NOT_STARTED`; keep authoritative assets and licensing limits |
| Published KMP API | `com.composables.sfsymbols` Kotlin API | Rust-backed adapters by target where feasible | `RETAINED_FOR_COMPATIBILITY`; Android/JVM/iOS/JS/Wasm remain unchanged |
| Installers and release identities | Compose Desktop packaging | Rust desktop packaging after upgrade proof | `RETAINED_FOR_COMPATIBILITY`; Windows UUID, macOS bundle ID and Debian package ID unchanged |
| Cross-platform verification | Existing Gradle targets and Rust workspace | Rust desktop matrix plus existing KMP jobs | `IN_PROGRESS`; Windows JS/Wasm compilation passes locally, Android/iOS and GitHub matrix pending |
| Application cutover | Kotlin desktop app and MCP launcher | Rust desktop app and MCP launcher | `NOT_STARTED`; no runtime cutover before compatibility gates pass |

The Rust MCP binary remains an opt-in executable. It is not yet bundled in the
desktop installers or used by the existing `--mcp` launch path.
