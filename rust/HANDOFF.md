# Migration handoff

## Current branch and baseline

The working branch is `migration/rust-max-coverage-2026-11-20`, created from
PR #7's head. It has a corresponding remote branch on GitHub. The original
Kotlin/Compose implementation and all public KMP targets remain active.
The Windows titlebar header now reserves the three caption-button widths plus a
128 dp margin; the lower gallery toolbar uses the content-pane inset so its
controls fit at the 1000 dp minimum. A fresh Windows launch at 1280×820 visually
confirms clearance between the native titlebar controls and SFStudio controls.
The minimum-size and high-DPI layouts remain visually unverified.
Windows System appearance now falls back to the AppsUseLightTheme registry value
when Compose reports Light; the running app resolved System to Dark on this
machine. The value is polled once per second while the Windows UI is open.
The caption-spacing change passed the full cross-platform workflow in [run
#44886962212](https://github.com/lylicht/SFStudio/actions/runs/44886962212).
The existing KMP `SfSymbol.layers` API now uses Rust-flattened animation layer
records from the glyph JSON bridge when available and keeps the Kotlin tree
flattening fallback. Full field parity passes for all 8,526 catalog symbols.
The observed catalog layout and unverified UI gates are recorded in
[UI_BASELINE.md](UI_BASELINE.md); its screenshot was not saved to the repo.

On Windows x64, Rust fmt, all 51 workspace tests, Clippy, Gradle JVM tests,
JS/Wasm compilation, MCP assembly, the Windows MSI build, and MCP stdio smoke
tests against both Kotlin and Rust passed. The Rust vector decoder matches all
8,526 Kotlin glyph fingerprints and available optical variants; its C ABI
returns weight-interpolated points and tight visible bounds. Bounds match
Kotlin across ten symbols, all nine weights, and all three optical scales.
All 99 Kotlin/Rust MCP differential responses match, including malformed JSON,
invalid IDs and batches, and notifications. See
[VERIFICATION.md](VERIFICATION.md).

The Rust C ABI is loaded by packaged JVM search, category count aggregation,
category/favorites filtering, filter-selection transitions, Compose snippet
generation, favorite toggling, MCP client config generation, complete
glyph/layer decoding, outline
interpolation, and Apple effect rendering through JNA. Android JNI and iOS
Kotlin/Native actuals now request the same glyph JSON with Kotlin fallback; JS
and Wasm are wired too. Browser Wasm loads asynchronously, so a glyph first
cached from Kotlin stays on that fallback for the app session. JVM parity now
matches all 8,526 glyphs and available optical variants, including full layer
trees. The desktop ViewModel also resolves System/Light/Dark and clamps
grid scale through Rust, retaining Kotlin fallbacks; Kotlin continues to own
Java Preferences persistence and Compose state. The latest cross-platform run
[#44888335626](https://github.com/lylicht/SFStudio/actions/runs/44888335626)
passed all 13 jobs, including category-count parity. The caption-spacing change
also passed all 13 jobs in [run #44886962212](https://github.com/lylicht/SFStudio/actions/runs/44886962212).
Run #44885280359 passed Rust/C ABI checks, JVM tests on Windows/macOS/Linux,
Android compilation and four-ABI packaging, iOS device/simulator compilation
and simulator tests,
packaged MCP launchers on all three desktop OSes, JS Node parity, Wasm
compilation, and differential MCP replay. Wasm runtime and browser validation
remain pending because of the Compose Skiko loader and Karma resource-serving
issues described below.
The web bridge’s JS parameter collision is fixed and the JS Node parity test now
passes. Kotlin/Wasm compilation passes in CI; runtime execution is still open
because Skiko fails before Wasm Node tests and the Karma browser runner returns
404 for the packaged Compose resource.
The Android instrumentation source for effect, appearance, and draw-field
parity compiled; runtime execution remains a device gate. Local Rust,
Python `ctypes`, JVM, desktop app, and JS Node checks pass. Rust Wasm-backed JS
parity covers points, Apple animation frames, and draw-field levels; Kotlin/Wasm
compiles, while its Windows Node test runner fails before the parity test when
Compose Skiko cannot fetch `skiko.wasm`. The Kotlin metadata fallback remains
active when Rust is unavailable or errors.
The metadata lookup endpoint and its live desktop caller passed
Windows/macOS/Linux parity in runs #20 and #44; the Windows MSI passed in run
#20.
Android/iOS/JS/Wasm Kotlin targets still compile and retain their compatibility
paths. Android weighted glyph outline points now use a Rust JNI adapter;
Android effects, appearance, and draw-field rendering now have Rust JNI
adapters with the existing Kotlin fallback. Their instrumentation parity cases
are added. Run [#44849774443](https://github.com/lylicht/SFStudio/actions/runs/44849774443)
passed Android main/device-test compilation and four-ABI packaging; runtime
parity still needs a device. iOS weighted outline points now pass device and
simulator compilation plus simulator Kotlin parity for ten symbols across all
nine weights and three scales in [run #44854205293](https://github.com/lylicht/SFStudio/actions/runs/44854205293).
iOS Apple effect frames and sRGB layer appearance now pass simulator parity in
[run #44857761656](https://github.com/lylicht/SFStudio/actions/runs/44857761656).
iOS draw-field C-interop checks passed in [run #44859231638](https://github.com/lylicht/SFStudio/actions/runs/44859231638).
Physical-device runtime remains unverified. JS and Wasm now bundle the Rust
Wasm library behind the existing APIs with Kotlin fallback; browser runtime and
published-package loading remain unverified. Compose still rasterizes the mask
and materializes platform `Path` clips from Rust-generated horizontal reveal
spans. The Kotlin reveal-level and path-building fallback remains in place.
The Rust draw-field response now adds horizontal spans without removing its
existing `levels` array. Local Rust and JVM/JS/Wasm checks pass; [run
#44885280359](https://github.com/lylicht/SFStudio/actions/runs/44885280359)
passed the full desktop, Android packaging, iOS simulator, JS/Wasm, and MCP
matrix with this change. Android rendering still needs a device, iOS physical-
device rendering remains unverified, and browser Wasm runtime is still open.

The Android JNI point bridge built `libsfstudio_ffi.so` for `armeabi-v7a`,
`arm64-v8a`, `x86`, and `x86_64`. The local AAR contains all four libraries
and the consumer R8 keep rule. Its Android device parity test passed on the
local Android 16 / API 36.1 x86_64 emulator for ten symbols across all nine
weights and all three optical scales. Hosted Linux software emulation failed or
stalled, and the Apple Silicon runner reported `HV_UNSUPPORTED`; standard hosted
Actions runners do not provide the nested virtualization this device test needs.
CI verifies Android compilation and four-ABI packaging, while the outline JNI
runtime parity test remains locally verified on x86_64. Run #44849774443 also
compiled the new effects/appearance/draw-field device tests. Their runtime
parity is not verified: this Windows host has no Android SDK or `adb`, and the
hosted runners cannot run the required emulator.
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

The desktop ViewModel now resolves System/Light/Dark mode and clamps the grid
scale through Rust with Kotlin fallbacks. Java Preferences persistence and
Compose state remain Kotlin; this settings bridge has only been validated on
the Windows JVM target.

Desktop favorites now load and save through Rust at the same user-home path
when the native library is available. Rust keeps the ordered JSON array,
normalizes duplicates, preserves regular-file permissions, and replaces via a
same-directory temporary file. Kotlin still resolves the path and retains its
fallback, including symlink writes. Rust, Python, and JVM checks pass locally;
Windows/macOS/Linux Rust C ABI and JVM checks passed in run #44869413797. The
user path and JSON contract remain unchanged.

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

1. Execute the Android animation, appearance, and draw-field parity cases on a
   device; verify the JS/Wasm bridge in browsers and published packages.
2. Continue porting the shared vector renderer beyond Rust-generated horizontal
   clip spans; add pixel-level comparison for the resulting clip geometry while
   preserving Compose as the public UI until a Rust-native frontend meets the
   existing screenshot and interaction gates.
3. Compare current Rust UI frameworks against desktop visual/interaction
   baselines before implementing a cutover candidate.
4. Preserve existing user data and package identities; test upgrade behavior
   before changing persistence or installers.

## References

- `AGENTS.md`: canonical contributor and compatibility rules.
- `rust/DECISIONS.md`: decisions.
- `rust/MAP.md`: unit by unit migration status.
- `scripts/check_rust_mcp_parity.py`: differential replay.
