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

**Reason:** The Rust core now exposes a language-neutral C ABI, but target
adapters are still missing for Android, iOS, JS, and Wasm. No single bridge is
assumed to cover every KMP target. Removing a target or changing a public
Compose signature would break downstream consumers.

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
