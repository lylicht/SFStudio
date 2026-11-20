# Migration handoff

## Current branch and baseline

The working branch is `migration/rust-max-coverage-2026-11-20`, created from
PR #7's head. It has a corresponding remote branch on GitHub. The original
Kotlin/Compose implementation and all public KMP targets remain active.

On Windows x64, Rust fmt, all nineteen workspace tests, Clippy, Gradle JVM tests,
JS/Wasm compilation, MCP assembly, the Windows MSI build, and MCP stdio smoke
tests against both Kotlin and Rust passed. All 99 Kotlin/Rust MCP differential
responses match, including malformed JSON, invalid IDs and batches, and
notifications. See [VERIFICATION.md](VERIFICATION.md).

The Rust C ABI is loaded by packaged JVM search, category/favorites filtering,
Compose snippet generation, and favorite toggling through JNA. All four passed
Windows/macOS/Linux checks in run #17; the Windows MSI includes the shared
library. Android/iOS/JS/Wasm Kotlin targets still compile and retain their
Kotlin paths; Rust-backed adapters for those targets are not implemented.

The branch ports Compose snippet generation and favorites JSON handling,
bundles the Rust MCP executable in desktop packages, adds MCP protocol edge
coverage, and checks Rust/KMP/package behavior in CI. GitHub Actions run
[#11](https://github.com/lylicht/SFStudio/actions/runs/44750825015) passed
Rust on Windows/macOS/Linux; JVM on Windows/macOS/Linux; Android API 37; iOS;
JS/Wasm; and packaged Rust MCP launchers on Windows/macOS/Linux.

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
2. Port compressed vector decoding and rendering semantics with reference
   fixtures before any UI or library cutover.
3. Preserve existing user data and package identities; test upgrade behavior
   before changing persistence or installers.

## References

- `AGENTS.md`: canonical contributor and compatibility rules.
- `rust/DECISIONS.md`: decisions.
- `rust/MAP.md`: unit by unit migration status.
- `scripts/check_rust_mcp_parity.py`: differential replay.
