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
```
