# Verification record

Last local check: 2026-11-20, Windows x64, branch
`migration/rust-max-coverage-2026-11-20`.

## Passed locally

- `cargo fmt --manifest-path rust/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path rust/Cargo.toml --workspace` — 38 Rust tests
  passed (26 core, 9 C ABI, and 3 MCP), including Rust fingerprints for every
  decoded Kotlin glyph and available optical-size variant.
- `cargo clippy --manifest-path rust/Cargo.toml --workspace --all-targets -- -D warnings` — passed.
- `cargo build --manifest-path rust/Cargo.toml -p sfstudio-ffi` — passed on
  Windows x64; Python `ctypes` smoke passed, including sizing calls, typo search,
  Unicode, invalid UTF-8, null pointers, short-buffer handling, exact Compose
  snippet fixtures, favorite ordering/toggle behavior, filtered search,
  symbol metadata lookup, and glyph decoding/interpolation at medium, small,
  and large scales, including visible bounds; animation rendering modes,
  gradient stops, the draw-field C ABI, installed/development MCP config
  generation with escaped Unicode paths, and invalid color/mask/config input
  also pass.
- `gradlew.bat :sfsymbols:jvmTest --tests com.composables.sfsymbols.SfVectorRustParityTest --rerun-tasks --no-daemon` —
  passed; all 8,526 Kotlin glyphs and available small/large variants match the
  committed geometry fingerprint fixture.
- `gradlew.bat :sfsymbols:jvmTest --tests com.composables.sfsymbols.SfVectorRustParityTest.rustVisibleBoundsMatchKotlinAcrossWeightsAndOpticalScales --no-daemon` —
  passed; Rust and Kotlin bounds match for ten symbols across all nine weights
  and three optical scales.
- `gradlew.bat :sfsymbols:jvmTest --no-daemon` — passed on JDK 25.
- `gradlew.bat :app:jvmTest --no-daemon` — passed with the Rust search library
  loaded and extracted from JVM resources; 37 Rust-versus-Kotlin search queries,
  both Compose snippet variants, favorite ordering/removal, filtered
  category/favorites results, and symbol metadata lookup matched Kotlin references.
- `gradlew.bat :app:jvmTest --tests com.sfsymbols.mcp.McpServerTest --no-daemon` —
  passed; the Settings MCP config path calls the Rust JNA function and matches
  the Kotlin JSON contract for installed and development launches, including
  escaped Unicode paths. The full `:app:jvmTest` suite passed afterward.
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
- [GitHub Actions run #20](https://github.com/lylicht/SFStudio/actions/runs/44768406103)
  — metadata lookup C ABI, Python `ctypes` smoke, and JVM parity passed on
  Windows/macOS/Linux; Android, iOS, JS/Wasm, and packaged MCP jobs passed too.
- The vector decoder and weight-aware glyph C ABI passed local Rust, Kotlin
  parity, Clippy, Python, and the Windows/macOS/Linux CI checks in run #24.
- [GitHub Actions run #22](https://github.com/lylicht/SFStudio/actions/runs/44775446369)
  passed Rust and Kotlin checks on macOS/Linux, Rust and FFI checks on Windows,
  Android, iOS, JS/Wasm, and packaged MCP jobs. Its Windows JVM job exposed
  CRLF line endings in the fingerprint fixture; the test now normalizes them,
  and the failure was reproduced locally with a CRLF fixture. The corrected
  run #24 passed the vector and bounds baseline on Windows/macOS/Linux, JVM,
  Android, iOS, JS/Wasm, packaged launchers, and MCP replay.
- The new visible-bounds matrix is verified locally; cross-OS CI for this
  latest geometry addition passed in run #24.

## Rust-backed desktop logic

- Category, favorites, and show-all state transitions now come from Rust through
  the language-neutral C ABI in the desktop ViewModel. Rust/Python/JVM checks
  cover Unicode and empty categories, action reset behavior, error flags, and
  the Kotlin empty-category fallback. [Run #34](https://github.com/lylicht/SFStudio/actions/runs/44805912613)
  passed the C ABI checks on Windows/macOS/Linux and the existing KMP/package
  matrix.
- MCP client configuration is generated in Rust and called by the existing
  Settings helper through JNA when the native library is available. The
  original Kotlin generator remains the native-library fallback. Local byte-
  exact C ABI/JVM parity and the cross-platform checks in run #33 pass.

## Apple animation evaluator

- Rust unit tests cover Apple draw timing and replace recipe boundaries.
- `python scripts/test_rust_ffi.py` exercises the animation JSON C ABI through
  `ctypes`, including Appear transforms, DownUp replacement, variable layers,
  malformed input, and unknown symbols.
- Rust effect calculations match a Kotlin-generated fixture across 16 effect
  configurations spanning all effect types in SFStudio's public API, selected
  symbols, and their flattened layers. This includes Apple's Appear, Disappear,
  Bounce, Pulse, Scale, Variable Color, Wiggle, Rotate, Breathe, Draw On/Off,
  and Magic Replace behaviors. [GitHub
  Actions run #25](https://github.com/lylicht/SFStudio/actions/runs/44787007184)
  passed the full Windows/macOS/Linux, JVM, Android, iOS, JS/Wasm, packaged
  launcher, and MCP matrix for those effect frames. The local
  `gradlew.bat :sfsymbols:jvmTest --tests com.composables.sfsymbols.SfDrawFieldRustParityTest --no-daemon -PsfstudioUseSystemWebTools=true`
  run also passed exact Rust/Kotlin level parity for five real symbol layers.
- The Magic Replace extension matches tagged layers and emits morphed path
  operations/points. Rust unit and Python ABI checks pass, and [GitHub Actions
  run #27](https://github.com/lylicht/SFStudio/actions/runs/44790870392) passed
  the full cross-platform matrix. Rust now computes draw-rail reveal levels;
  path rasterization, clip geometry, and Compose remain in the active renderer.
- Layer appearance calculations now cover monochrome, hierarchical, palette,
  and multicolor modes, named Apple colors, layer opacity, and Apple's
  automatic two-stop gradient. Rust unit and Python C ABI smoke checks passed
  locally. [GitHub Actions run #29](https://github.com/lylicht/SFStudio/actions/runs/44794238481)
  passed the Windows/macOS/Linux Rust and C ABI checks plus the existing JVM,
  Android, iOS, JS/Wasm, packaged desktop, and MCP jobs. Kotlin differential
  fixtures and integration with a Rust renderer remain pending.
- Draw On/Off reveal levels now use the Kotlin renderer's 112x112 grid,
  48-level distance field, tie search, connected components, and no-corner
  diagonal rule. A Kotlin-generated full-grid fixture for five real symbol
  layers passes exact Rust parity, along with the JVM parity test and Python C
  ABI smoke. [GitHub Actions run #31](https://github.com/lylicht/SFStudio/actions/runs/44799137234)
  passed the Windows/macOS/Linux Rust and C ABI checks plus the existing JVM,
  Android, iOS, JS/Wasm, packaged desktop, and MCP jobs. The platform still
  rasterizes the source path; Rust returns the mask's reveal levels and does
  not create Compose Path clip geometry. Visual clip comparison remains
  pending.
- SF Symbols 7 Draw On/Off playback now distinguishes staggered `ByLayer` from
  `individually = true`, which waits for each motion group to finish. Kotlin
  frames match the Rust reference fixture; Rust unit tests cover both Draw On
  and Draw Off scheduling, and the Python C ABI smoke covers the additive JSON
  option. Local `cargo fmt`, workspace tests, Clippy, full `:sfsymbols:jvmTest`,
  the desktop `:app:jvmTest`, and JS/Wasm compilation pass. Android compilation
  was unavailable because this Windows host has no Android SDK; iOS compilation
  and tests require macOS.
- The JVM `SfSymbolIcon` renderer now requests one Rust animation frame batch
  per draw, including Rust-resolved sRGB layer colors and gradient stops, and
  falls back to Kotlin if the native library or result is unavailable. The JVM
  parity tests compare every layer frame for all listed Apple effects and
  compare layer appearance in monochrome, hierarchical, palette, and multicolor
  modes. Colors and gradient stops allow 0.002 float tolerance for Compose color
  channel packing and floating-point math; wide-gamut colors use Kotlin. The
  full `:sfsymbols:jvmTest` suite passes with the native path enabled, including
  animated renders in all four modes, as does `:app:jvmTest`. JS and Wasm main
  source sets compile. The shared `buildRustFfi` Gradle task now lives at the
  root so both app packaging and library parity tests build the same native
  library.
- [GitHub Actions run #37](https://github.com/lylicht/SFStudio/actions/runs/44815230235)
  passed every job: Rust and FFI on Windows/macOS/Linux; JVM animation parity
  on Windows/macOS/Linux; Android, iOS, JS/Wasm, all packaged launchers, and MCP
  differential replay.
- [GitHub Actions run #39](https://github.com/lylicht/SFStudio/actions/runs/44819189332)
  passed every job: Rust and MCP on Windows/macOS/Linux; JVM and Compose API
  checks on Windows/macOS/Linux; Android, iOS, JS/Wasm, packaged MCP launchers
  on Linux/macOS/Windows, and Kotlin/Rust MCP differential replay.

## Not yet verified

- Android, Kotlin/Native, Swift, JS, and Wasm bindings for the shared Rust API.
  Their existing KMP targets remain buildable and use the Kotlin evaluator;
  only JVM effect-frame evaluation calls Rust today.
- Pixel-by-pixel rendering parity, installer upgrades, user-data migration,
  and performance baselines. Draw-field API integration with the active
  renderer is pending. Geometry and draw-field C ABIs have cross-OS CI coverage,
  but target-specific bindings remain untested.
- Rust desktop application behavior; a Rust desktop UI does not exist yet.

The passing CI run verifies the existing Kotlin Multiplatform targets remain
buildable. Local JVM parity verifies that the public effect renderer uses Rust
frames when the native library is available; the broader public KMP behavior is
still Kotlin-backed on other targets.
