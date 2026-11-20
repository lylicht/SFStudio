# Migration handoff

## Current branch and baseline

The working branch is `migration/rust-max-coverage-2026-11-20`, created from
PR #7's head. It has a corresponding remote branch on GitHub. The original
Kotlin/Compose implementation and all public KMP targets remain active.

On Windows x64, Rust fmt, all eleven workspace tests, Clippy, Gradle JVM tests,
JS/Wasm compilation, MCP assembly, the Windows MSI build, and MCP stdio smoke
tests against both Kotlin and Rust passed. All 99 Kotlin/Rust MCP differential
responses match, including malformed JSON, invalid IDs and batches, and
notifications. See [VERIFICATION.md](VERIFICATION.md).

The Rust C ABI is now loaded by the packaged JVM fuzzy-search path through JNA.
Windows `:app:jvmTest` verified 37-query ranking parity against the Kotlin
reference, and the Windows MSI build passed with the shared library packaged.
Cross-OS verification of this JNA integration is pending.

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

1. Verify the packaged JNA search adapter on macOS and Linux; keep the Kotlin
   fallback for development and library-load failures.
2. Evaluate Android/JNI, Kotlin/Native/iOS, and Wasm/JS adapters separately;
   one FFI mechanism is not assumed to fit every target.
3. Port favorites persistence, settings, app state, vector decoding, and
   render semantics with
   parity fixtures before any UI or library cutover.
4. Capture visual/interaction baselines and test installer upgrades while
   preserving package identities and user data.

## References

- `AGENTS.md`: canonical contributor and compatibility rules.
- `rust/DECISIONS.md`: decisions.
- `rust/MAP.md`: unit by unit migration status.
- `scripts/check_rust_mcp_parity.py`: differential replay.
