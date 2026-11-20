# Verification record

Last local check: 2026-11-20, Windows x64, branch
`migration/rust-max-coverage-2026-11-20`.

## Passed locally

- `cargo fmt --manifest-path rust/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path rust/Cargo.toml --workspace` — 11 Rust tests
  passed (8 core, including snippet and favorites fixtures, and 3 MCP).
- `cargo clippy --manifest-path rust/Cargo.toml --workspace --all-targets -- -D warnings` — passed.
- `gradlew.bat :app:jvmTest :sfsymbols:jvmTest --no-daemon` — passed on JDK 25.
- The `CatalogViewModelTest` snippet cases match the Rust fixtures exactly,
  including import order, colors, mode, background wrapper, and final newline.
- `PinnedSymbolsCompatibilityTest` reads both the legacy Kotlin JSON fixture
  and the Rust-written JSON fixture with the existing Kotlin set serializer.
- `gradlew.bat :app:assembleMcp --no-daemon` — passed.
- `gradlew.bat :app:packageMsi --no-daemon` — Windows MSI build passed; the
  packaged app image launcher passed the Rust MCP stdio smoke test.
- `cargo build --manifest-path rust/Cargo.toml -p sfstudio-mcp` — passed.
- `python scripts/check_rust_mcp_parity.py` — 99/99 Kotlin and Rust MCP
  responses matched, including malformed JSON, invalid IDs/batches, and
  notification behavior.
- `python scripts/test_mcp.py` — Kotlin `--mcp` stdio smoke test passed.
- `python scripts/test_mcp.py --rust-binary rust/target/debug/sfstudio-mcp.exe`
  — Rust server through the existing `--mcp` launcher passed.
- `python scripts/test_mcp.py --launcher app/build/compose/binaries/main/app/SFStudio/SFStudio.exe`
  — packaged Windows Rust MCP entrypoint passed, including Unicode and
  malformed-request checks; this smoke run requires Rust and fails if the
  launcher falls back to Kotlin.
- `gradlew.bat :sfsymbols:jsNodeTest :sfsymbols:wasmJsNodeTest --no-daemon -PsfstudioUseSystemWebTools=true`
  — JS and Wasm source compilation passed; both test tasks reported `NO-SOURCE`.
- [GitHub Actions run #11](https://github.com/lylicht/SFStudio/actions/runs/44750825015)
  — all jobs passed: Rust and JVM across Windows/macOS/Linux; Android API 37;
  iOS; JS/Wasm; and packaged MCP stdio smoke on Windows/macOS/Linux.

## Not yet verified

- Rust FFI bindings and their per-platform public API integration tests.
- UI and vector-rendering parity, installer upgrades, user-data migration,
  performance baselines, and native bindings.
- Rust desktop application behavior; a Rust desktop UI does not exist yet.

The passing CI run verifies the existing Kotlin Multiplatform targets remain
buildable. It does not verify that Rust backs the public KMP APIs.
