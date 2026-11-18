# Migration status

| Area | Current implementation | Rust target | State / required gate |
| --- | --- | --- | --- |
| Symbol metadata + aliases | Kotlin catalog + text tags | `sfstudio-core` | Initial port; unit tests and JVM parity required |
| Fuzzy query ranking | `FuzzySearch.kt` | `sfstudio-core` | Initial port; differential replay required |
| Headless MCP server | `McpServer.kt` | `sfstudio-mcp` | Initial port; differential replay required |
| Favorites persistence | `PinnedSymbols.kt` | shared Rust core | Not started; filesystem format and backups |
| Snippet generation | `ComposeSnippet.kt` | shared Rust core | Not started; byte-for-byte snippet tests |
| UI state | `CatalogViewModel.kt` | Rust shared state / bindings | Not started; test UI state invariants |
| Compose desktop UI | `app/.../ui/` | Native Rust UI (TBD) | Not started; pixel and interaction parity |
| Symbol vector rendering | `sfsymbols/` generated classes | Rust renderer/data | Not started; 7,007 symbol asset parity |
| Compose KMP public API | `sfsymbols/` | Stable-compatible bindings | Not started; Android/JVM/iOS/JS/Wasm compatibility |
| Installers and releases | Gradle Compose packaging | Multi-OS Rust packaging | Not started; preserve installer IDs |
| Overall cutover | JVM app | Rust app | BLOCKED until all gates pass |

The Rust MCP binary is **opt-in** and is not wired into release packaging.
