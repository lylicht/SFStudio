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

**Rollback:** Use the existing JVM `--mcp` command unchanged.

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
