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
- The search header reserves space for Windows caption buttons. At the time of
  this capture, source used an 80 dp margin after the three-button strip. After
  a further overlap report, the source margin was raised to 128 dp; that update
  compiled, but the app has not yet been relaunched for a new capture. The
  gallery toolbar is below the search row and uses the ordinary 18 dp pane inset.

The refreshed screenshot shows about 78 px between the search field and the
caption-button strip at the default window size. It was inspected in-session but
not saved as a repository artifact. The lower-toolbar and previous caption-control
inset changes passed the local JVM app suite and the Windows/macOS/Linux JVM CI
jobs in
[run #44873269421](https://github.com/lylicht/SFStudio/actions/runs/44873269421).
The 1000 dp minimum-size layout remains visually unverified.

## Appearance and remaining gate

The catalog screenshot was in light appearance. Settings exposes System, Light,
and Dark preference modes in code, but navigation into Settings did not activate
during the UI inspection, so those appearance states were not visually captured.
The app window minimum is configured in `app/src/jvmMain/kotlin/com/sfsymbols/Main.kt`.

This is a textual and in-session baseline, not the saved screenshot required by
decision 011. Rust-native UI selection remains gated on a saved reference at the
default and minimum sizes, light and dark visual comparisons, and keyboard and
screen-reader checks. The current Compose UI remains the compatibility UI.
