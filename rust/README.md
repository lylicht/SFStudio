# SFStudio Rust migration

This is the **first executable slice** of a gradual Rust port. The desktop UI
and published Compose Multiplatform API remain compatible while Rust replaces
portable logic behind the installed desktop entry points.

## What has been ported

- `sfstudio-core`: the SFStudio catalog index, typo-tolerant search algorithm,
  category/favorites filtering, and catalog filter-selection transitions
  originally split across `FuzzySearch.kt` and `CatalogViewModel.kt`.
- `sfstudio-core::codegen`: Compose snippet generation ported against shared
  Kotlin/Rust golden fixtures and exposed through the C ABI; the packaged JVM
  caller uses Rust through JNA with a Kotlin fallback.
- `sfstudio-core::favorites`: ordered favorite state and JSON codec compatible
  with the existing Kotlin array format; packaged JVM toggles use the C ABI,
  while Kotlin retains ownership of the favorite file.
- `sfstudio-core::rendering` and `sfstudio-core::draw`: Apple rendering-mode
  colors, layer opacity, gradient stops, and rail-guided Draw On/Off reveal
  levels are calculated in Rust. The platform still rasterizes paths and
  builds Compose clip geometry.
- `sfstudio-core::mcp_config`: the installed-launcher and development-classpath
  MCP configuration shown by Settings, serialized byte-for-byte like the
  existing Kotlin implementation. The desktop Settings helper uses Rust
  through JNA when the native library is available and keeps its Kotlin fallback.
- `sfstudio-core::vector`: compile-time extraction and Rust decoding of the
  generated glyph geometry, layer trees, and optical variants. Exact Kotlin
  geometry fingerprints match for all 8,526 glyphs locally. This is the data
  source for vector path operations; visible bounds match Kotlin over ten
  symbols, all nine weights, and all three optical scales.
- `sfstudio-core::animation`: Apple-style effect timing, per-layer transforms,
  variable-layer staging, replace recipes, and Magic Replace tag matching with
  morphed outlines. Rust covers all effect types in SFStudio's current public
  API: Appear, Disappear, Bounce, Pulse, Variable Color, Scale, Wiggle, Rotate,
  Breathe, Draw On/Off, and Magic Replace. The JSON C ABI makes frame evaluation
  available to foreign-language callers. Actual rendering and path rasterization
  remain in Kotlin.
- `sfstudio-mcp`: a native, newline-delimited JSON-RPC/MCP stdio executable
  providing the existing `search_symbols` and `get_symbol` tools.
- `sfstudio-ffi`: a language-neutral C ABI for catalog search/filtering,
  filter-selection transitions, symbol metadata lookup, glyph geometry at a
  requested weight and optical
  scale, animation frames, Draw On/Off levels, MCP configuration, Compose code
  generation, and favorite toggling. Its header and UTF-8 buffer contract are
  in `include/sfstudio.h`. Python `ctypes`
  smoke checks for search, filtering,
  codegen, and favorites passed on Windows, macOS, and Linux in run #17. The
  packaged JVM UI calls it through JNA; published KMP callers remain unchanged.
  The JVM metadata lookup binding passed parity checks on Windows, macOS, and
  Linux in run #20; it is not yet called by a production UI path.
- The Rust catalog and vector data are **read directly at compile time from
  the existing Kotlin catalog, alias tags, and generated geometry sources**.
  This is deliberately temporary: no parallel icon catalog is maintained.
  The embedded sources are not parsed at runtime.

Packaged `--mcp` launches delegate to the Rust executable after parity checks;
classpath-based development launches retain the Kotlin fallback. Packaged UI
search, category/favorites filtering and filter-selection transitions, snippet
generation, favorite toggling, and Settings MCP configuration call the Rust C
ABI through JNA, keeping Kotlin behavior as the development and load-failure
fallback. The filter-selection addition passes local Rust, Python, and JVM
checks; cross-OS CI is pending. [Run #33](https://github.com/lylicht/SFStudio/actions/runs/44803230690)
passed the Windows/macOS/Linux Rust and C ABI jobs, JVM tests on all three
desktop systems, Android, iOS, JS/Wasm, packaged MCP launchers, and differential
MCP replay.

## Verify

From the repository root, with a Rust toolchain installed:

```bash
cargo test --manifest-path rust/Cargo.toml --workspace
cargo clippy --manifest-path rust/Cargo.toml --workspace --all-targets -- -D warnings
cargo build --manifest-path rust/Cargo.toml -p sfstudio-mcp
cargo build --manifest-path rust/Cargo.toml -p sfstudio-ffi
python3 scripts/test_rust_ffi.py
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
- The C ABI is callable from languages with C FFI support. Android, iOS,
  JavaScript, and Wasm still need target-specific adapters before their public
  APIs use Rust.
