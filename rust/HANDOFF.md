# Migration handoff

## Current branch and baseline

The working branch is `migration/rust-max-coverage-2026-11-20`, created from
PR #7's head. It has a corresponding remote branch on GitHub. The original
Kotlin/Compose implementation and all public KMP targets remain active.

On Windows x64, Rust fmt, all 42 workspace tests, Clippy, Gradle JVM tests,
JS/Wasm compilation, MCP assembly, the Windows MSI build, and MCP stdio smoke
tests against both Kotlin and Rust passed. The Rust vector decoder matches all
8,526 Kotlin glyph fingerprints and available optical variants; its C ABI
returns weight-interpolated points and tight visible bounds. Bounds match
Kotlin across ten symbols, all nine weights, and all three optical scales.
All 99 Kotlin/Rust MCP differential responses match, including malformed JSON,
invalid IDs and batches, and notifications. See
[VERIFICATION.md](VERIFICATION.md).

The Rust C ABI is loaded by packaged JVM search, category/favorites filtering,
filter-selection transitions, Compose snippet generation, favorite toggling,
MCP client config generation, outline interpolation, and Apple effect rendering
through JNA. The latest completed Rust migration [GitHub Actions run #44](https://github.com/lylicht/SFStudio/actions/runs/44832738954)
passed Rust/C ABI checks on Windows/macOS/Linux, JVM tests on Windows/macOS/Linux,
Android, iOS, JS/Wasm, packaged MCP launchers on all three desktop OSes, and
differential MCP replay. This includes the typed-float outline endpoint in the
JVM renderer and the live Rust metadata lookup used by desktop symbol
selection. Local Rust, Python `ctypes`, JVM, desktop app, JS, and Wasm checks
pass. The Kotlin metadata fallback remains active when Rust is unavailable or
errors.
The metadata lookup endpoint and its live desktop caller passed
Windows/macOS/Linux parity in runs #20 and #44; the Windows MSI passed in run
#20.
Android/iOS/JS/Wasm Kotlin targets still compile and retain their compatibility
paths. Android weighted glyph outline points now use a Rust JNI adapter;
Android effects, appearance, and draw-field rendering, plus iOS, JS, and Wasm,
remain Kotlin-backed.

The Android JNI point bridge built `libsfstudio_ffi.so` for `armeabi-v7a`,
`arm64-v8a`, `x86`, and `x86_64`. The local AAR contains all four libraries
and the consumer R8 keep rule. Its Android device parity test passed on the
local Android 16 / API 36.1 x86_64 emulator for ten symbols across all nine
weights and all three optical scales. The first hosted emulator attempt failed
before boot because current Emulator 37 requires `ANDROID_I_WANT_MY_TCG=yes`
without hardware acceleration. The next run booted, but its package manager
failed during APK installation with a broken pipe and SwiftShader color-buffer
errors; CI now allocates four emulator cores and disables GPU rendering.
Building the Android target now requires Android NDK `28.2.13676358` and
`cargo-ndk` `4.1.2`; CI installs both before assembling the KMP AAR.

The branch ports Compose snippet generation and favorites JSON handling,
bundles the Rust MCP executable in desktop packages, adds MCP protocol edge
coverage and a Rust vector decoder/bounds implementation, and checks
Rust/KMP/package behavior in CI. GitHub Actions run
[#24](https://github.com/lylicht/SFStudio/actions/runs/44782746583) passed the
vector/bounds baseline on Windows/macOS/Linux, JVM, Android API 37, iOS,
JS/Wasm, packaged launchers, and MCP replay. The animation port passed the full
cross-platform matrix in [run #25](https://github.com/lylicht/SFStudio/actions/runs/44787007184).
The Magic Replace morph extension passed the full matrix in
[run #27](https://github.com/lylicht/SFStudio/actions/runs/44790870392).
Layer rendering-mode colors and automatic gradient calculations are now
available through the animation C ABI; Rust unit tests and Python `ctypes`
smoke pass locally. [GitHub Actions run #29](https://github.com/lylicht/SFStudio/actions/runs/44794238481)
passed the cross-platform C ABI and existing platform matrix. Kotlin
differential coverage for those color calculations is still pending. The
draw-field distance calculation is now in Rust and exposed through a second
C ABI function; Kotlin fixture parity and Linux/Windows/macOS checks passed in
[GitHub Actions run #31](https://github.com/lylicht/SFStudio/actions/runs/44799137234).
The Settings screen now gets installed-app and development-classpath MCP
configuration from Rust through JNA when the native library is available.
Byte-exact parity and escaped path handling pass locally, and run #33 verifies
the change across the Rust/C ABI, JVM, and packaged MCP matrix.
The category/favorites/all filter-selection transition passes Rust, Python C
ABI, and JVM ViewModel checks locally, plus Windows/macOS/Linux and KMP matrix
coverage in [run #34](https://github.com/lylicht/SFStudio/actions/runs/44805912613).

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

1. Extend the verified Android JNI path to animation frames, appearance, and
   draw-field levels behind the same Kotlin fallback, then test each behavior
   through the public renderer. Evaluate Kotlin/Native and JS/Wasm bindings
   independently.
2. Continue porting the shared vector renderer, including path/clip generation
   and pixel-level comparison, while preserving Compose as the public UI until
   a Rust-native frontend meets the existing screenshot and interaction gates.
3. Compare current Rust UI frameworks against desktop visual/interaction
   baselines before implementing a cutover candidate.
4. Preserve existing user data and package identities; test upgrade behavior
   before changing persistence or installers.

## References

- `AGENTS.md`: canonical contributor and compatibility rules.
- `rust/DECISIONS.md`: decisions.
- `rust/MAP.md`: unit by unit migration status.
- `scripts/check_rust_mcp_parity.py`: differential replay.
