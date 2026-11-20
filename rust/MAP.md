# Migration status

This table tracks the Rust implementation separately from compatibility code
that must remain available to existing Kotlin Multiplatform consumers.

| Area | Current implementation | Rust target | State / required gate |
| --- | --- | --- | --- |
| Symbol names, categories, restricted flags, aliases | KMP catalog and JVM tag file; Rust reads those same sources at compile time | `sfstudio-core` | `PORTED`; catalog count and basic lookup tests pass; Rust builds on Windows/macOS/Linux |
| Fuzzy query ranking | Kotlin search remains the fallback and reference | `sfstudio-core` via C ABI | `VERIFIED`; 37-query JNA parity passed on Windows/macOS/Linux in Actions run #14 |
| Cross-language core entry point | Rust core had no stable foreign-language ABI | `sfstudio-ffi` C ABI + `include/sfstudio.h` | `IN_PROGRESS`; Windows/macOS/Linux builds and Python `ctypes` search smoke passed; codegen C ABI smoke passed locally; platform adapters remain pending |
| Desktop search caller | `FuzzySearch.kt` | Rust search via JNA C ABI | `VERIFIED`; Kotlin fallback preserved; Windows/macOS/Linux parity passed in Actions run #14 |
| MCP server | Kotlin `McpServer.kt` remains the development fallback | `sfstudio-mcp`; packaged `--mcp` launcher delegates to it | `VERIFIED`; 99 Kotlin/Rust response cases match locally; packaged smoke passes on Windows, macOS, and Linux |
| Favorites model and JSON codec | `PinnedSymbols.kt` remains the active disk reader/writer at `~/.sf-symbols-catalog/pins.json` | `sfstudio-core::favorites` | `IN_PROGRESS`; ordered set semantics and Kotlin/Rust JSON fixtures pass; Rust disk adapter and integration pending |
| Compose snippet generation | Kotlin implementation remains the fallback and reference | `sfstudio-core::codegen` via C ABI | `IN_PROGRESS`; Rust C ABI matches both golden fixtures and Kotlin parity locally; cross-OS JNA run pending |
| UI state and filtering | `CatalogViewModel.kt` | Rust state API and per-target adapters | `NOT_STARTED`; verify state transitions and all consumers |
| Desktop UI | Kotlin/Compose | Rust-native UI if parity gates pass | `RETAINED_FOR_COMPATIBILITY`; visual and interaction baselines required |
| Vector rendering and symbol effects | KMP generated sources and renderer | Rust renderer/data | `NOT_STARTED`; keep authoritative assets and licensing limits |
| Published KMP API | `com.composables.sfsymbols` Kotlin API | Rust-backed adapters by target where feasible | `RETAINED_FOR_COMPATIBILITY`; Android/JVM/iOS/JS/Wasm remain unchanged |
| Installers and release identities | Compose Desktop packaging | Rust desktop packaging after upgrade proof | `RETAINED_FOR_COMPATIBILITY`; Windows UUID, macOS bundle ID and Debian package ID unchanged |
| Cross-platform verification | Existing Gradle targets and Rust workspace | Rust desktop matrix plus existing KMP jobs | `IN_PROGRESS`; C ABI search and JNA desktop search passed on Windows/macOS/Linux in runs #13/#14; codegen JNA matrix and platform-specific Rust adapters remain pending |
| Desktop GUI cutover | Kotlin/Compose desktop app | Rust-native desktop app | `NOT_STARTED`; UI, vector, and interaction parity gates required |

The Rust MCP binary is bundled into standalone MCP assemblies and desktop
installers. Packaged `--mcp` invocations delegate to it; classpath-based Gradle
launches keep the Kotlin implementation as a development fallback.
