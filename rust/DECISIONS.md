# Rust migration decisions (initial)

## 001 — Convert progressively, never in place

**Decision:** Keep Kotlin/Compose as the reference implementation. Add Rust
crates alongside it, develop behind a separate branch and require test gates
before switching default behavior. Do not overwrite or discard 14,000+
generated Compose vector classes.

**Reason:** A faithful UI/UX rewrite is substantially more than translation
of business logic; it requires rendering equivalence on Windows, macOS, and
Linux and a separate plan for Android/iOS/JS/Wasm library consumers.

## 002 — Port deterministic search and headless protocol first

**Decision:** First port `FuzzySearch.kt`, catalog lookup and headless MCP.
Keep Compose UI unchanged. Ensure exact ranked-result ordering and JSON-RPC
format by differential testing against the JVM implementation.

Packaged `--mcp` now delegates to Rust after the differential and cross-OS
launcher gates pass. Development classpath launches retain Kotlin.

## 003 — Single source of truth for symbol metadata

**Decision:** Embed the existing catalog source and tag file during the
transitional port; extract validated, redistributable portable data as an
explicit later change. Preserve the current 7,007 rows in their original order.

**Reason:** Two independently edited copies would silently drift.

## 004 — rust-anything support boundary

**Decision:** Adopt its record/port/diff/gate workflow, but do not assert that
its Kotlin conversion works. Upstream documents Kotlin/Swift as unsupported.
Create explicit Kotlin-Rust parity tests now, and reconsider the tool when
upstream support is available.

## 005 — Keep distribution and saved data intact

**Decision:** Preserve `com.bxane.glyphstudio` on macOS, the Windows installer
upgrade GUID, the Linux `glyphstudio` package identifier, and
`~/.sf-symbols-catalog/pins.json`. Keep the published Compose library and
its JitPack coordinates backwards compatible during migration.

**Outstanding design decisions:** native UI technology, icon-vector data
conversion/licensing, multiplatform distribution/ABI, KMP bridge vs independent
bindings, exact pixel parity and accessibility, release signing.

## 006 — Keep Kotlin Multiplatform APIs live during Rust migration

**Decision:** Preserve the published Kotlin package and Android, JVM, iOS, JS,
and Wasm targets while Rust code is introduced. Build a separate bridge for
each target behind the existing Kotlin API only after target-specific behavior
and rendering checks pass.

**Reason:** The Rust core now exposes a language-neutral C ABI, but its
adapters are target-specific. Android's weighted outline-point bridge is
verified; iOS, JS, Wasm, and the remaining Android renderer operations still
need adapters and parity gates. Removing a target or changing a public Compose
signature would break downstream consumers.

**Current boundary:** The published library and UI still use Kotlin. Packaged
desktop search, filtering, code generation, favorites toggling, MCP config, and
`--mcp` use Rust through JNA or the bundled Rust server. Kotlin fallbacks remain
available when the desktop native library or MCP process cannot load.

## 007 — Decode existing vector data in Rust before rendering cutover

**Decision:** Extract the repository's existing generated geometry blobs at
Rust build time, decode their raw-DEFLATE blocks in `sfstudio-core`, and verify
all glyph masters and optical variants against Kotlin fingerprints. Port tight
visible bounds with cubic extrema and erase-layer rules, and verify them across
all weights and optical scales for representative symbols. Expose decoded
geometry through the existing C ABI, including weight-interpolated points and
bounds; keep Kotlin as the renderer and public KMP implementation.

**Reason:** This ports reusable vector logic without hand-editing generated
files or introducing a second asset catalog. Geometry parity is a necessary
step, but it does not prove visual renderer or animation parity.

## 008 — Use Apple's symbol references without changing SFStudio's identity

**Decision:** Use [SFSafeSymbols](https://github.com/SFSafeSymbols/SFSafeSymbols)
as a reference for Apple's typed symbol names and availability. Preserve
SFStudio's catalog, `com.composables.sfsymbols` package, public Compose API,
and release identities. Model Rust animation frames against [Apple's
SymbolEffect API](https://developer.apple.com/documentation/symbols/symboleffect)
while retaining every effect already exposed by SFStudio.

**Reason:** SFSafeSymbols is a Swift access wrapper, while SFStudio has its own
Kotlin Multiplatform API and rendering behavior. It informs compatibility; it
does not replace SFStudio's public identity or supply its renderer.

## 009 — Use typed Rust outline buffers in the JVM renderer

**Decision:** Expose weight- and optical-scale-interpolated screen coordinates
as a typed `f32` C ABI buffer and use it from the existing JVM `SfSymbol.points`
path when the Rust native library is available. Keep the Kotlin decoder and
interpolator as a fallback and on Android, iOS, JS, and Wasm until each target
has its own verified bridge.

**Reason:** The Rust vector decoder and interpolation already pass exhaustive
geometry fingerprints and representative parity checks. Returning only the
coordinate buffer avoids parsing a large JSON glyph payload in the renderer,
while retaining the current Compose path construction and all public Kotlin
APIs.

## 010 — Resolve selected desktop metadata through Rust

**Decision:** On JVM desktop symbol selection, call the existing Rust metadata
lookup endpoint and retain the supplied Kotlin metadata if the native library
is unavailable or fails. Keep bulk search-result mapping unchanged to avoid an
extra FFI lookup per result.

**Reason:** This makes Rust metadata authoritative in the detail view without
changing the public KMP metadata type, the search C ABI, or fallback behavior.

## 011 — Gate desktop UI selection on measured parity

**Decision:** Keep Compose as the production desktop UI while evaluating Rust
frontends. Slint is the lead candidate for an isolated fidelity probe; do not
add it to the production app or cut over until its visual, interaction,
accessibility, and license requirements are checked against the live app.

**Evidence:** Slint's Winit backend targets Windows, macOS, and Linux/X11/
Wayland, and its [path element](https://docs.slint.dev/latest/docs/slint/reference/elements/path/)
accepts SVG path commands and even-odd fills. Its desktop AccessKit
integration is documented as always enabled. Its royalty-free app license
requires either the AboutSlint widget or a public attribution badge. egui/eframe
offers custom 2D painting and documents native AccessKit APIs for Windows/macOS;
Iced Canvas supports cubic paths, while the 0.15 accessibility work remains a
draft. Sources: [Slint Winit docs](https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backend_winit/),
[Slint accessibility and license notes](https://slint.dev/blog/slint-1.1-released),
[Slint license terms](https://slint.dev/terms-and-conditions),
[egui README](https://docs.rs/crate/egui/latest/source/README.md),
[Iced Canvas docs](https://docs.rs/iced/latest/iced/widget/canvas/index.html),
and [Iced 0.15 milestone](https://github.com/iced-rs/iced/milestone/26).

**Next gate:** Capture a saved reference at 1280×820 and minimum size, then
compare a Rust symbol tile, inspector, header, and effect preview in light and
dark themes. Check keyboard and screen-reader behavior and preserve the
Rust-computed Apple effect frames. Treat the framework choice and Slint
attribution as unresolved until that probe is measured.

## 012 — Bind Android glyph interpolation through JNI

**Decision:** Keep the existing common `SfSymbol.points()` API and Kotlin
fallback. On Android, package the Rust `sfstudio-ffi` library for all four
standard Android ABIs and call its typed glyph-points C ABI through one private
JNI method. Publish an R8 keep rule for the JNI class.

**Reason:** Android cannot use the JVM JNA adapter. JNI lets the existing
renderer use verified Rust point interpolation without exposing Rust details
through the public KMP API or porting unrelated animation behavior. The device
test compares exact point arrays against Kotlin for ten symbols, all nine
weights, and all three optical scales. Device runtime verification currently
covers x86_64; the other ABIs are built and packaged but not device-tested.

## 013 — Reuse the Rust rendering C ABI from Android JNI

**Decision:** Keep the common KMP renderer and Kotlin fallback. Have the
Android actual pass the existing animation and draw-field JSON requests through
JNI to the Rust C ABI, including sRGB layer appearance and gradients. Keep
wide-gamut colors on Kotlin and validate frame shape, numeric fields, and
reveal-level ranges before returning native results.

**Gate:** Android instrumentation cases compare Apple effect frames, all four
rendering modes and gradients, and draw-field levels with the Kotlin reference.
The Android source compile and four-ABI package gate passed in [Actions run
#44849774443](https://github.com/lylicht/SFStudio/actions/runs/44849774443).
Device runtime parity is still pending; hosted emulator attempts have no usable
virtualization, and the current Windows host has no Android SDK.

## 014 — Bind iOS outline interpolation through Kotlin/Native C interop

**Decision:** Keep the published KMP API and Kotlin geometry fallback. Build
the Rust FFI as target-specific static archives for iOS device and simulator,
embed each archive in its target C-interop KLIB, and call the existing typed
glyph-points C ABI from the iOS actual. Leave effects and Draw On/Off on Kotlin
until their separate parity gates pass.

**Gate:** Rust `cargo check` passes for `aarch64-apple-ios` and
`aarch64-apple-ios-sim`. An iOS simulator test compares the native points with
Kotlin across ten symbols, all nine weights, and all three scales. The Windows
host cannot run Kotlin/Native Apple cinterop; macOS CI compilation, linking, and
simulator parity remain pending.
