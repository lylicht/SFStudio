# Migration handoff

## Current branch and baseline

The working branch is `migration/rust-max-coverage-2026-11-20`, created from
PR #7's head. It has a corresponding remote branch on GitHub. The original
Kotlin/Compose implementation and all public KMP targets remain active.

On Windows x64, Rust fmt, all 29 workspace tests, Clippy, Gradle JVM tests,
JS/Wasm compilation, MCP assembly, the Windows MSI build, and MCP stdio smoke
tests against both Kotlin and Rust passed. The Rust vector decoder matches all
8,526 Kotlin glyph fingerprints and available optical variants; its C ABI
returns weight-interpolated points and tight visible bounds. Bounds match
Kotlin across ten symbols, all nine weights, and all three optical scales.
All 99 Kotlin/Rust MCP differential responses match, including malformed JSON,
invalid IDs and batches, and notifications. See
[VERIFICATION.md](VERIFICATION.md).

The Rust C ABI is loaded by packaged JVM search, category/favorites filtering,
Compose snippet generation, and favorite toggling through JNA. Search,
filtering, snippets, and favorites passed Windows/macOS/Linux checks in run #17.
The metadata lookup binding passed Windows/macOS/Linux parity and the Windows
MSI passed in run #20; lookup is not yet called by a production UI path.
Android/iOS/JS/Wasm Kotlin targets still compile and retain their Kotlin paths;
Rust-backed adapters for those targets are not implemented.

The branch ports Compose snippet generation and favorites JSON handling,
bundles the Rust MCP executable in desktop packages, adds MCP protocol edge
coverage and a Rust vector decoder/bounds implementation, and checks
Rust/KMP/package behavior in CI. GitHub Actions run
[#24](https://github.com/lylicht/SFStudio/actions/runs/44782746583) passed the
vector/bounds baseline on Windows/macOS/Linux, JVM, Android API 37, iOS,
JS/Wasm, packaged launchers, and MCP replay. The animation port passed the full
cross-platform matrix in [run #25](https://github.com/lylicht/SFStudio/actions/runs/44787007184).

## Existing work inherited from PR #7

- Located and inspected the reference SFStudio Kotlin/Compose architecture.
- Compared it with rust-anything's documented language coverage; Kotlin is
  not supported by rust-anything's automatic porting workflow.
- Added a Cargo workspace with a Rust search engine and standalone Rust
  JSON-RPC MCP server.
- Added unit regression tests plus a differential Kotlin/Rust MCP replay
  harness and CI workflow.
- Kept the existing UI, icon sources and installers untouched.

## Next unblocked migration work

1. Evaluate Android/JNI, Kotlin/Native/iOS, and Wasm/JS adapters separately;
   one FFI mechanism is not assumed to fit every target.
2. Expand Rust animation parity coverage, port Magic Replace matching/morphing
   and draw-rail clipping, then keep the existing renderer as the visual
   reference until Rust rendering is verified.
3. Preserve existing user data and package identities; test upgrade behavior
   before changing persistence or installers.

## References

- `AGENTS.md`: canonical contributor and compatibility rules.
- `rust/DECISIONS.md`: decisions.
- `rust/MAP.md`: unit by unit migration status.
- `scripts/check_rust_mcp_parity.py`: differential replay.
