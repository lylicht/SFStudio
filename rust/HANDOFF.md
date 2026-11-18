# Migration handoff

## Completed in the first branch

- Located and inspected the reference SFStudio Kotlin/Compose architecture.
- Compared it with rust-anything's documented language coverage; Kotlin is
  not supported by rust-anything's automatic porting workflow.
- Added a Cargo workspace with a Rust search engine and standalone Rust
  JSON-RPC MCP server.
- Added unit regression tests plus a differential Kotlin/Rust MCP replay
  harness and CI workflow.
- Kept the existing UI, icon sources and installers untouched.

## Verification status

Code was authored and committed through the GitHub connector. The local
tool environment did not have an installed Rust compiler, but GitHub Actions
**successfully compiled the Rust workspace on Linux**, ran its six unit tests,
passed Clippy with warnings denied, built the original Kotlin MCP implementation
and confirmed **92 out of 92 differential JSON-RPC responses matched**.
See GitHub Actions run 44417720106.

Do not mark this slice production-ready without additional Windows/macOS
smoke tests and review of behavior not covered by the 92 replay cases.

## Next

1. Fix any Rust build, clippy, or parity defects found by CI.
2. Expand parity corpus and add exact lookup/case/category and unicode tests.
3. Add Windows/macOS Rust CI and benchmark search on representative queries.
4. Migrate favorites and snippet generation with byte-for-byte tests while
   keeping the Compose UI unchanged.
5. Design native Rust GUI and public KMP compatibility strategy before a
   desktop or library cutover.

## References

- `AGENTS.md`: canonical contributor and compatibility rules.
- `rust/DECISIONS.md`: decisions.
- `rust/MAP.md`: unit by unit migration status.
- `scripts/check_rust_mcp_parity.py`: differential replay.
