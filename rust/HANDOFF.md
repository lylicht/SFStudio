# Migration handoff

## Current branch and baseline

The working branch is `migration/rust-max-coverage-2026-11-20`, created from
PR #7's head. It has a corresponding remote branch on GitHub. The original
Kotlin/Compose implementation and all public KMP targets remain active.

On Windows x64, Rust fmt, all eleven workspace tests, Clippy, Gradle JVM tests,
JS/Wasm compilation, MCP assembly, the Windows MSI build, and MCP stdio smoke
tests against both Kotlin and Rust passed. All 92 Kotlin/Rust MCP differential
responses match. See [VERIFICATION.md](VERIFICATION.md).

The branch ports Compose snippet generation and favorites JSON handling,
bundles the Rust MCP executable in desktop packages, fixes Rust formatting,
adds an opt-in system Node/Yarn setup for strict Gradle settings, and adds
Rust/KMP/package CI. The last GitHub run passed Rust, JVM, iOS, and JS/Wasm
jobs; Android API 37 could not be installed from the stable SDK channel. The
workflow now requests the preview channel and is awaiting another run.

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

1. Rerun Android compile with preview SDK channel and verify Linux/macOS
   packaged `--mcp` stdio entrypoints.
2. Expand MCP replay for invalid JSON-RPC IDs and batch messages.
3. Add the Rust core behind the existing JVM/KMP compatibility API, then
   evaluate JNI/Android, C ABI/iOS, and Wasm/JS bridges separately.
4. Port preferences, app state, vector decoding, and render semantics with
   parity fixtures before any UI or library cutover.
5. Capture visual/interaction baselines and test installer upgrades while
   preserving package identities and user data.

## References

- `AGENTS.md`: canonical contributor and compatibility rules.
- `rust/DECISIONS.md`: decisions.
- `rust/MAP.md`: unit by unit migration status.
- `scripts/check_rust_mcp_parity.py`: differential replay.
