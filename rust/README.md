# SFStudio Rust migration

This is the **first executable slice** of a gradual Rust port. The desktop UI
and published Compose Multiplatform API remain compatible while Rust replaces
portable logic behind the installed desktop entry points.

## What has been ported

- `sfstudio-core`: the SFStudio catalog index and typo-tolerant search algorithm
  originally in `app/src/jvmMain/kotlin/com/sfsymbols/data/FuzzySearch.kt`.
- `sfstudio-core::codegen`: Compose snippet generation ported against shared
  Kotlin/Rust golden fixtures; the current desktop caller stays Kotlin.
- `sfstudio-core::favorites`: ordered favorite state and JSON codec compatible
  with the existing Kotlin array format; the Kotlin file store stays active.
- `sfstudio-mcp`: a native, newline-delimited JSON-RPC/MCP stdio executable
  providing the existing `search_symbols` and `get_symbol` tools.
- The Rust catalog is **read directly at compile time from the existing Kotlin
  catalog data and alias tags**. This is deliberately temporary: no parallel
  icon catalog is maintained. The embedded source is not parsed at runtime.

Packaged `--mcp` launches delegate to the Rust executable after parity checks;
classpath-based development launches retain the Kotlin fallback.

## Verify

From the repository root, with a Rust toolchain installed:

```bash
cargo test --manifest-path rust/Cargo.toml --workspace
cargo clippy --manifest-path rust/Cargo.toml --workspace --all-targets -- -D warnings
cargo build --manifest-path rust/Cargo.toml -p sfstudio-mcp
./gradlew :app:assembleMcp
python3 scripts/check_rust_mcp_parity.py
python3 scripts/test_mcp.py --rust-binary rust/target/debug/sfstudio-mcp
```

On Windows run `gradlew.bat :app:assembleMcp` in place of the Gradle shell script.
The Python harness accepts `--java` and `--rust-binary` overrides.
It compares parsed JSON-RPC responses—including exact ranked search-result
orders—from the existing Kotlin MCP server with those from the Rust binary.

The direct executable can be launched via:

```bash
cargo run --manifest-path rust/Cargo.toml -p sfstudio-mcp
```

It reads one JSON-RPC message per stdin line and writes one response per
request to stdout, preserving notifications with no response.

## How rust-anything fits

The migration methodology follows
[erkamyaman/rust-anything](https://github.com/erkamyaman/rust-anything):
record original behavior, port a small unit, replay and gate before cutover.

**Do not run `rust-anything bridge`, `app lift`, or `analyze --whole-app`
against the Kotlin source and claim success.** As of the inspected upstream
documentation (November 2026), Kotlin and Swift are not supported. This
repository therefore uses an independent Kotlin-to-Rust differential replay
harness for the first slice. Add native rust-anything compatibility only if
it gains working Kotlin parsing, bridge and replay support, or if an actually
supported original-language unit is identified.

The `sfstudio-mcp` binary is a first step, not a full Rust desktop app.

## Non-negotiable compatibility

See [DECISIONS.md](DECISIONS.md) and [MAP.md](MAP.md).
See [COMPATIBILITY.md](COMPATIBILITY.md) for the per-target Kotlin contract
and [VERIFICATION.md](VERIFICATION.md) for current checks and unverified work.

- Keep existing light/dark styling, Liquid Glass elements, layout and controls.
- Keep the 7,007 symbol names, order, categories, vector rendering and
  monochrome/dualtone variants.
- Keep the published `com.composables.sfsymbols` Kotlin API.
- Preserve user favorites at `~/.sf-symbols-catalog/pins.json` and installer
  upgrade IDs; never rewrite, delete or move user settings as a test.
- Preserve MCP tool names, parameters, error codes and JSON-RPC behavior.
- Keep Kotlin UI and symbol rendering operational until a native replacement
  passes visually reviewed per-OS tests; no UI rewrite is attempted here.
