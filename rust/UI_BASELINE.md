# Desktop UI baseline

Observed on Windows x64 on 2026-11-21. The running Compose app displayed at
approximately 1282×821 pixels; `Main.kt` configures a 1280×820 dp initial size
and a 1000×680 dp minimum size.

## Catalog screen

- Native title bar with native minimize, maximize, and close controls.
- Brand and symbol search row below the title bar.
- Left sidebar with All Symbols (7,007), Favorites, rendering-style filters,
  categories, and Settings.
- Center symbol grid with mode, size, and grid/list controls.
- Right inspector with Color, BG, and Basic info tabs, a selected-symbol
  preview, an Apple Bounce effect selector and Play action, appearance/tint
  controls, and Copy Name / Copy Code actions.
- The search header reserves the three Windows caption-button widths plus a
  128 dp margin. A fresh launch at 1280×820 visually confirms that the search
  and gallery controls clear the native window buttons. The gallery toolbar is
  below the search row and uses the ordinary 18 dp pane inset.

The fresh screenshot was inspected in-session but not saved as a repository
artifact. The minimum-size and high-DPI layouts remain visually unverified. The
lower-toolbar and caption-control inset changes passed the local JVM app suite
and the Windows/macOS/Linux JVM CI jobs in
[run #44873269421](https://github.com/lylicht/SFStudio/actions/runs/44873269421).

## Rust-native fidelity probe

The isolated `rust/sfstudio-ui` Slint 1.19 probe compiled and ran on Windows
x64. Its initial System appearance followed the current dark OS scheme, and the
Settings buttons visibly switched between Light, Dark, and System. The 1280×820
probe window kept its native title controls clear of the app controls. This is
not a saved parity capture: the probe has only 16 categories and 96 search
results, disabled Favorites and rendering-mode controls, no persistent app
state, and a simplified inspector. It remains separate from the production
Compose UI.

## Appearance and remaining gate

The production Compose catalog screenshot was in light appearance. Its Settings
screen exposes System, Light, and Dark preference modes in code, but its
appearance states were not visually captured. The minimum window is configured
in `app/src/jvmMain/kotlin/com/sfsymbols/Main.kt`.

This is a textual and in-session baseline, not the saved screenshot required by
decision 011. Rust-native UI selection remains gated on a saved reference at the
default and minimum sizes, light and dark visual comparisons, and keyboard and
screen-reader checks. The current Compose UI remains the compatibility UI.
