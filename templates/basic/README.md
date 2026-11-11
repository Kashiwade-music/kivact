# Kivact basic template

The smallest Kivact app: one component, a window and a real-window test.

```text
cargo run -p kivact-template-basic      # window (Arvane host, Windows verified)
cargo test -p kivact-template-basic     # the same UI in a test window (kivact::test)
```

To start a new app, copy this directory out of the Kivact checkout as a standalone root package, rename the package and point the
`kivact` dependency at your checkout (or a git dependency). The app uses `use kivact::prelude::*;` including the standard components of `kivact::ui`: a button with a Button role, focus, accessible name and an Activate action, in the theme's look. Add `features = ["test"]` in `dev-dependencies` for `kivact::test`.

Guides: [tutorial](../../docs/for-kivact-user/en/getting-started/README.md).

In a standalone app, remove `[lints] workspace = true` or define matching workspace lint rules in the new workspace root.

## Release-build performance and size

Kivact's workspace root carries its recommended release profile, which covers this template while it is a member of Kivact's workspace: fat LTO, one codegen unit, opt-level "s" with opt-level 3 for the drawing, shaping and layout crates, and stripped symbols. It makes the binary smaller and its hot paths faster, and makes the release build much longer, because fat LTO links the whole dependency graph in one step. Cargo reads profiles only from the root manifest, so once your copy is a standalone root, add this to its Cargo.toml.

```toml
# A standalone App reads profiles only from its own root manifest; these are Kivact's recommended release settings (size and speed; fat LTO lengthens release builds).
[profile.release]
lto = "fat"
codegen-units = 1
opt-level = "s"
strip = "symbols"
# panic stays unwind: an embedded UI (a DAW plugin) must not unwind a panic into its host.

# The drawing, shaping and layout hot paths favour speed.
[profile.release.package.vello_cpu]
opt-level = 3
[profile.release.package.vello_common]
opt-level = 3
[profile.release.package.fearless_simd]
opt-level = 3
[profile.release.package.kurbo]
opt-level = 3
[profile.release.package.skrifa]
opt-level = 3
[profile.release.package.read-fonts]
opt-level = 3
[profile.release.package.harfrust]
opt-level = 3
[profile.release.package.parley]
opt-level = 3
[profile.release.package.taffy]
opt-level = 3
[profile.release.package.kivact-layout]
opt-level = 3
[profile.release.package.kivact-engine]
opt-level = 3
[profile.release.package.arvane-render]
opt-level = 3
[profile.release.package.arvane-compose]
opt-level = 3
[profile.release.package.arvane-text]
opt-level = 3

# Third-party crates that run only on the asset worker thread (SVG parsing
# and rasterizing, image decoding) or wait on the OS (file dialogs, the
# clipboard) favour size. Crates on the UI thread keep their level, even
# when they look cold: the accessibility adapter (AccessKit, every update
# while assistive technology is active), shader validation and translation
# (naga), date, number and message formatting (ICU, kivact-i18n) and SVG
# path data (svgtypes, `Path::parse_svg`).
[profile.release.package.usvg]
opt-level = "z"
[profile.release.package.resvg]
opt-level = "z"
[profile.release.package.tiny-skia]
opt-level = "z"
[profile.release.package.tiny-skia-path]
opt-level = "z"
[profile.release.package.rustybuzz]
opt-level = "z"
[profile.release.package.ttf-parser]
opt-level = "z"
[profile.release.package.fontdb]
opt-level = "z"
[profile.release.package.roxmltree]
opt-level = "z"
[profile.release.package.simplecss]
opt-level = "z"
[profile.release.package.image]
opt-level = "z"
[profile.release.package.zune-jpeg]
opt-level = "z"
[profile.release.package.zune-core]
opt-level = "z"
[profile.release.package.image-webp]
opt-level = "z"
[profile.release.package.png]
opt-level = "z"
[profile.release.package.gif]
opt-level = "z"
[profile.release.package.fdeflate]
opt-level = "z"
[profile.release.package.miniz_oxide]
opt-level = "z"
[profile.release.package.flate2]
opt-level = "z"
[profile.release.package.rfd]
opt-level = "z"
[profile.release.package.arboard]
opt-level = "z"
```

## Debug-build performance

Unoptimized, the engine, layout, renderer and font pipeline are too slow for a debug build to keep up with a drag or a hover. Kivact's workspace root optimizes them in debug builds, which covers this template while it is a member of Kivact's workspace. Cargo reads profiles only from the root manifest, so once your copy is a standalone root (its own `Cargo.toml` at the top of the project), add this to that `Cargo.toml`. App, component and runtime code remains debuggable.

```toml
[profile.dev.package.vello_cpu]
opt-level = 3

[profile.dev.package.vello_common]
opt-level = 3

[profile.dev.package.vello_hybrid]
opt-level = 3

[profile.dev.package.fearless_simd]
opt-level = 3

[profile.dev.package.glifo]
opt-level = 3

[profile.dev.package.kurbo]
opt-level = 3

[profile.dev.package.peniko]
opt-level = 3

[profile.dev.package.skrifa]
opt-level = 3

[profile.dev.package.read-fonts]
opt-level = 3

[profile.dev.package.font-types]
opt-level = 3

[profile.dev.package.taffy]
opt-level = 3

[profile.dev.package.kivact-engine]
opt-level = 3

[profile.dev.package.kivact-editing]
opt-level = 3

[profile.dev.package.kivact-types]
opt-level = 3

[profile.dev.package.arvane]
opt-level = 3

[profile.dev.package.arvane-asset]
opt-level = 3

[profile.dev.package.arvane-compose]
opt-level = 3

[profile.dev.package.kivact-gpu-share]
opt-level = 3

[profile.dev.package.arvane-render]
opt-level = 3

[profile.dev.package.arvane-text]
opt-level = 3

[profile.dev.package.arvane-native]
opt-level = 3

[profile.dev.package.arvane-shell]
opt-level = 3

[profile.dev.package.arvane-virtual]
opt-level = 3

[profile.dev.package.parley]
opt-level = 3

[profile.dev.package.wgpu]
opt-level = 3

[profile.dev.package.wgpu-core]
opt-level = 3

[profile.dev.package.wgpu-hal]
opt-level = 3

[profile.dev.package.accesskit]
opt-level = 3

[profile.dev.package.accesskit_consumer]
opt-level = 3

[profile.dev.package.accesskit_windows]
opt-level = 3

[profile.dev.package.kivact-layout]
opt-level = 3

[profile.dev.package.kivact-shader]
opt-level = 3
```
