# Migration handoff

## Current branch and baseline

The working branch is `migration/rust-max-coverage-2026-11-20`, created from
PR #7's head. It has a corresponding remote branch on GitHub. The original
Kotlin/Compose implementation and all public KMP targets remain active.

On Windows x64, Rust fmt, all eight workspace tests, Clippy, both Gradle JVM
test tasks, JS/Wasm compilation, MCP assembly, Rust MCP build, and all 92
Kotlin/Rust differential MCP responses passed. Compose snippet output is
covered by two shared Kotlin/Rust fixtures. See [VERIFICATION.md](VERIFICATION.md).

The current changes fix Rust formatting, add an opt-in system Node/Yarn setup
for the repository's strict Gradle settings, and add Rust/KMP platform CI.
Cross-OS Rust and Android/iOS target CI still need a GitHub run before results
can be recorded.

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

1. Run the expanded Windows/macOS/Linux Rust and retained KMP CI jobs; fix any
   failures without weakening parity checks.
2. Expand MCP protocol replay for invalid IDs, pipelined requests, EOF,
   Unicode, and stdout purity.
3. Port one portable feature only alongside a target-compatible bridge and
   differential tests; favorites and snippet generation are next candidates.
4. Design and evaluate per-target Rust bridges for Android, JVM, iOS, JS, and
   Wasm before changing any public KMP path.
5. Establish UI, vector rendering, persistence, and installer baselines before
   planning desktop or library cutover.

## References

- `AGENTS.md`: canonical contributor and compatibility rules.
- `rust/DECISIONS.md`: decisions.
- `rust/MAP.md`: unit by unit migration status.
- `scripts/check_rust_mcp_parity.py`: differential replay.
