# SFStudio Rust UI probe

This isolated Slint shell is a visual-fidelity probe for the Rust catalog,
vector paths, Apple Bounce frames, System/Light/Dark palette behavior, full
category navigation, and desktop favorites. Favorites use the existing
`~/.sf-symbols-catalog/pins.json` format and path. The Windows header leaves
space for native caption controls. It is not wired into the production app;
Compose remains the desktop UI.

Run it with `cargo run --manifest-path rust/Cargo.toml -p sfstudio-ui`. On Linux,
install `libfontconfig1-dev` first. The crate requires Rust 1.95 or newer.

The probe still omits full toolbar behavior, persistent settings, and most
inspector behavior. Apple animation frames exist in the Rust core, while this
probe only plays Bounce. Visual and accessibility parity, saved reference
captures, minimum-size and high-DPI checks, and Slint attribution must be
resolved before it can replace or ship with the production UI.
