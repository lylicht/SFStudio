# Migration status

This table tracks the Rust implementation separately from compatibility code
that must remain available to existing Kotlin Multiplatform consumers.

| Area | Current implementation | Rust target | State / required gate |
| --- | --- | --- | --- |
| Symbol names, categories, restricted flags, aliases | KMP catalog and JVM tag file; Rust reads those same sources at compile time | `sfstudio-core` | `PORTED`; catalog count and basic lookup tests pass; Rust builds on Windows/macOS/Linux |
| Fuzzy query ranking | `FuzzySearch.kt` remains the desktop reference | `sfstudio-core` | `PORTED`; 99 MCP response cases match locally, broader UI-query parity pending |
| Cross-language search entry point | Existing Rust search had no stable foreign-language ABI | `sfstudio-ffi` C ABI + `include/sfstudio.h` | `IN_PROGRESS`; Windows/macOS/Linux build and Python `ctypes` smoke; Kotlin/Native, JNI, Swift, JS and Wasm adapters remain pending |
| MCP server | Kotlin `McpServer.kt` remains the development fallback | `sfstudio-mcp`; packaged `--mcp` launcher delegates to it | `VERIFIED`; 99 Kotlin/Rust response cases match locally; packaged smoke passes on Windows, macOS, and Linux |
| Favorites model and JSON codec | `PinnedSymbols.kt` remains the active disk reader/writer at `~/.sf-symbols-catalog/pins.json` | `sfstudio-core::favorites` | `IN_PROGRESS`; ordered set semantics and Kotlin/Rust JSON fixtures pass; Rust disk adapter and integration pending |
| Compose snippet generation | `ComposeSnippet.kt` remains the active desktop path | `sfstudio-core::codegen` | `PORTED`; Kotlin and Rust match shared byte-for-byte fixtures for both background and plain variants; runtime bridge pending |
| UI state and filtering | `CatalogViewModel.kt` | Rust state API and per-target adapters | `NOT_STARTED`; verify state transitions and all consumers |
| Desktop UI | Kotlin/Compose | Rust-native UI if parity gates pass | `RETAINED_FOR_COMPATIBILITY`; visual and interaction baselines required |
| Vector rendering and symbol effects | KMP generated sources and renderer | Rust renderer/data | `NOT_STARTED`; keep authoritative assets and licensing limits |
| Published KMP API | `com.composables.sfsymbols` Kotlin API | Rust-backed adapters by target where feasible | `RETAINED_FOR_COMPATIBILITY`; Android/JVM/iOS/JS/Wasm remain unchanged |
| Installers and release identities | Compose Desktop packaging | Rust desktop packaging after upgrade proof | `RETAINED_FOR_COMPATIBILITY`; Windows UUID, macOS bundle ID and Debian package ID unchanged |
| Cross-platform verification | Existing Gradle targets and Rust workspace | Rust desktop matrix plus existing KMP jobs | `IN_PROGRESS`; GitHub Actions run #11 passed Rust and JVM on Windows/macOS/Linux, plus Android API 37, iOS, JS/Wasm, and packaged MCP on all desktop OSes; Rust-backed KMP API tests remain pending |
| Desktop GUI cutover | Kotlin/Compose desktop app | Rust-native desktop app | `NOT_STARTED`; UI, vector, and interaction parity gates required |

The Rust MCP binary is bundled into standalone MCP assemblies and desktop
installers. Packaged `--mcp` invocations delegate to it; classpath-based Gradle
launches keep the Kotlin implementation as a development fallback.
