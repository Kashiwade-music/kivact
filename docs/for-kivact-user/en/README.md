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

By default Kivact renders on the GPU and falls back to the CPU automatically
when no GPU is usable. To build without GPU support, disable default features
(`default-features = false`). To force CPU rendering at run time, set the
environment variable `ARVANE_RENDERER=cpu`.

## Where to start

| Section | What it is for |
| --- | --- |
| [getting-started/](getting-started/README.md) | The tutorial. Build a small to-do app, one page at a time. Start here. |
| [how-to-guide/](how-to-guide/README.md) | Guides for specific goals, such as validating a form or showing a dialog. |
| [UI kits](../../for-kivact-ui-user/en/README.md) | The four official component kits (`kivact-ui-basic`, `kivact-ui-pop`, `kivact-ui-game`, `kivact-ui-creative`) and their themes. |
| API reference | Run `cargo doc -p kivact --open` in a project that depends on Kivact, or in a clone of the repository. |
