# Verification record

Last local check: 2026-11-20, Windows x64, branch
`migration/rust-max-coverage-2026-11-20`.

## Passed locally

- `cargo fmt --manifest-path rust/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path rust/Cargo.toml --workspace` — 8 Rust tests
  passed (5 core, including two shared snippet fixtures, and 3 MCP).
- `cargo clippy --manifest-path rust/Cargo.toml --workspace --all-targets -- -D warnings` — passed.
- `gradlew.bat :app:jvmTest :sfsymbols:jvmTest --no-daemon` — passed on JDK 25.
- The `CatalogViewModelTest` snippet cases match the Rust fixtures exactly,
  including import order, colors, mode, background wrapper, and final newline.
- `gradlew.bat :app:assembleMcp --no-daemon` — passed.
- `cargo build --manifest-path rust/Cargo.toml -p sfstudio-mcp` — passed.
- `python scripts/check_rust_mcp_parity.py` — 92/92 Kotlin and Rust MCP
  responses matched.
- `gradlew.bat :sfsymbols:jsNodeTest :sfsymbols:wasmJsNodeTest --no-daemon -PsfstudioUseSystemWebTools=true`
  — JS and Wasm source compilation passed; both test tasks reported `NO-SOURCE`.

## Not yet verified

- Linux and macOS Rust builds on this branch.
- GitHub Actions for the new Windows/macOS/Linux Rust matrix.
- Android and iOS target jobs in the expanded workflow.
- UI and vector-rendering parity, installer upgrades, user-data migration,
  and native bindings.
- Rust desktop application behavior; a Rust desktop UI does not exist yet.

The branch workflow now schedules these platform checks. A configured job is
not evidence of a passing job; update this record only from completed runs.
