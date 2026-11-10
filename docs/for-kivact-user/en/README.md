# Kivact for application developers

[日本語](../jp/README.md)

Kivact is a native GUI framework for Rust. You write components as plain
functions that run once, and fine-grained signals update only the parts of the
window that read them. If you know React or SolidJS, the model will feel
familiar, and your Rust domain logic lives in the same codebase.

> **Early development.** Kivact has no release yet, and APIs change without
> notice. Pin a commit if you need a stable build.

Windows and Linux (Wayland and X11) are the target platforms. macOS is not
tested.

```rust,no_run
use kivact::prelude::*;

fn counter() -> impl IntoView {
    let count = signal(0);
    column((
        text(move || format!("count: {}", count.get())),
        button("Increment").on_press(move || count.update(|n| *n += 1)),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Counter").run(counter)
}
```

## Install

Kivact is not on crates.io yet. Applications depend on the Git repository:

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

The repository pins its toolchain to Rust 1.98.1 (edition 2024) in
`rust-toolchain.toml`; use that version or newer.

By default Kivact draws frames on the CPU and starts the GPU device when the
window needs it (a user shader drawn, or the App reading `gpu_interop::adapter()`
to share GPU textures; CPU frames that keep missing half the display period start
it too). Without a usable GPU it
stays on the CPU. To force CPU rendering at run time, set the environment variable
`ARVANE_RENDERER=cpu`; `ARVANE_RENDERER=gpu` starts the device with the window.

### Ship a smaller binary

The facade enables two features by default: `gpu`, the GPU renderer, and
`complex-scripts`, ICU's word and line dictionaries for scripts without spaces
(Thai, Lao, Khmer, Myanmar and Chinese/Japanese word breaking, about 3 MB). An
app that does not need that segmentation can leave `complex-scripts` out and keep
everything else:

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", default-features = false, features = ["gpu"] }
```

Leave `gpu` out of the list too for a build with no GPU renderer. `kivact-bench`
builds with neither feature for its `--no-default-features` binary, which measures
the smallest build.
The `[profile.release]` of `templates/basic/Cargo.toml` (fat LTO, one codegen
unit, `opt-level = "s"` with `3` for the drawing, shaping and layout crates,
stripped symbols) is the other half: it makes the binary smaller and its hot
paths faster, and it makes the release build longer.

## Where to start

| Section | What it is for |
| --- | --- |
| [getting-started/](getting-started/README.md) | The tutorial. Build a small to-do app, one page at a time. Start here. |
| [how-to-guide/](how-to-guide/README.md) | Guides for specific goals, such as validating a form or showing a dialog. |
| [UI kits](../../for-kivact-ui-user/en/README.md) | The five official component kits (`kivact-ui-basic`, `kivact-ui-pop`, `kivact-ui-game`, `kivact-ui-creative`, `kivact-ui-soft`) and their themes. |
| API reference | Run `cargo doc -p kivact --open` in a project that depends on Kivact, or in a clone of the repository. |
