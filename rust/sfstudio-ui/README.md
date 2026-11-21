# SFStudio Rust UI probe

This isolated Slint shell is a visual-fidelity probe for the Rust catalog,
vector paths, Apple Bounce frames, and System/Light/Dark palette behavior. It is
not wired into the production app; Compose remains the desktop UI.

Run it with `cargo run --manifest-path rust/Cargo.toml -p sfstudio-ui`. On Linux,
install `libfontconfig1-dev` first. The crate requires Rust 1.95 or newer.

The current probe deliberately omits complete category coverage, Favorites,
toolbar actions, persistent settings, and most inspector behavior. Visual and
accessibility parity, saved reference captures, and Slint attribution must be
resolved before it can replace or ship with the production UI.
