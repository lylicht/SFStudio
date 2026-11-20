# Verification record

Last local check: 2026-11-20, Windows x64, branch
`migration/rust-max-coverage-2026-11-20`.

## Passed locally

- `cargo fmt --manifest-path rust/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path rust/Cargo.toml --workspace` — 20 Rust tests
  passed (9 core, 8 C ABI, and 3 MCP).
- `cargo clippy --manifest-path rust/Cargo.toml --workspace --all-targets -- -D warnings` — passed.
- `cargo build --manifest-path rust/Cargo.toml -p sfstudio-ffi` — passed on
  Windows x64; Python `ctypes` smoke passed, including sizing calls, typo search,
  Unicode, invalid UTF-8, null pointers, short-buffer handling, exact Compose
  snippet fixtures, favorite ordering/toggle behavior, filtered search, and
  symbol metadata lookup.
- `gradlew.bat :sfsymbols:jvmTest --no-daemon` — passed on JDK 25.
- `gradlew.bat :app:jvmTest --no-daemon` — passed with the Rust search library
  loaded and extracted from JVM resources; 37 Rust-versus-Kotlin search queries,
  both Compose snippet variants, favorite ordering/removal, filtered
  category/favorites results, and symbol metadata lookup matched Kotlin references.
- `gradlew.bat :app:packageMsi --no-daemon` — Windows MSI build passed with the
  Rust FFI DLL and MCP executable in `app-jvm.jar`.
- The `CatalogViewModelTest` snippet cases match the Rust fixtures exactly,
  including import order, colors, mode, background wrapper, and final newline.
- `PinnedSymbolsCompatibilityTest` reads both the legacy Kotlin JSON fixture
  and the Rust-written JSON fixture with the existing Kotlin set serializer.
- `gradlew.bat :app:assembleMcp --no-daemon` — passed.
- The packaged app image launcher passed the Rust MCP stdio smoke test.
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
- [GitHub Actions run #13](https://github.com/lylicht/SFStudio/actions/runs/44754758733)
  — C ABI builds and Python `ctypes` smoke passed on Windows/macOS/Linux; all
  existing Kotlin targets and MCP differential replay passed.
- [GitHub Actions run #14](https://github.com/lylicht/SFStudio/actions/runs/44758827641)
  — JNA-backed Rust search passed 37-query Kotlin parity on Windows/macOS/Linux;
  all existing Kotlin targets and packaged MCP checks passed.
- [GitHub Actions run #16](https://github.com/lylicht/SFStudio/actions/runs/44762564449)
  — C ABI search, codegen, and favorites passed Windows/macOS/Linux smoke tests;
  JVM search/snippet/favorites tests passed on Windows/macOS/Linux; Android,
  iOS, JS/Wasm, and packaged MCP jobs passed.
- [GitHub Actions run #17](https://github.com/lylicht/SFStudio/actions/runs/44764947372)
  — filtered C ABI and JNA parity passed on Windows/macOS/Linux; all existing
  Kotlin targets and packaged MCP jobs passed as well.
- The metadata lookup C ABI, Python smoke, JVM parity test, and Windows MSI
  packaging passed locally. Cross-platform CI for this endpoint is pending;
  run #17 predates the metadata lookup change.

## Not yet verified

- Kotlin/Native, Swift, JS, and Wasm bindings for the shared Rust API. Existing
  KMP targets remain buildable but still execute their Kotlin implementation.
- UI and vector-rendering parity, installer upgrades, user-data migration,
  performance baselines, and native bindings.
- Rust desktop application behavior; a Rust desktop UI does not exist yet.

The passing CI run verifies the existing Kotlin Multiplatform targets remain
buildable. It does not verify that Rust backs the public KMP APIs.
