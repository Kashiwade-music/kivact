![](assets/Kivact_Banner.svg)

# Kivact

[日本語](README_JP.md)

Kivact is a native GUI framework for Rust. Components are plain functions
that run once; fine-grained signals update only the parts of the screen that
read them. The model will be familiar to React and SolidJS developers, and
the same codebase holds the application's Rust domain logic.

> **Status: early development.** APIs and crate layout change without
> notice and there is no release yet. Bug reports are welcome as issues;
> pull requests are not accepted for now.

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

## Features

- **Plain functions and signals.** `signal`, `memo`, `effect` and keyed
  lists, with no virtual DOM and no re-rendering of whole components.
- **Standard components.** Buttons, inputs, selects, dialogs, menus, tabs,
  tables, toasts and more in `kivact::ui`, styled by the active theme.
- **Accessibility by default.** Every component exposes role, name and state
  to the platform's assistive technologies and works with the keyboard.
- **Real-window testing.** `kivact::test` opens real windows from
  `cargo test` and drives them by role and name, like Testing Library.
- **Text and input.** CJK, combining characters, bidirectional text and IME
  composition.
- **Drawing.** A Canvas with an HTML Canvas–like vocabulary, frames from the
  application's own renderer, and off-screen image export.
- **UI kits.** Five official kits restyle the standard components and add
  domain components, built only on the public API.

## Getting started

Kivact is not published on crates.io yet. Depend on the Git repository:

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

The toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml).
Start with the [tutorial](docs/for-kivact-user/en/getting-started/README.md).

To run the examples, clone the repository and run from its root:

```text
cargo run -p kivact-example-counter
cargo run -p kivact-example-form
cargo run -p kivact-example-list
cargo run -p kivact-example-async
cargo run -p kivact-example-component
cargo run -p kivact-ui-basic-catalog
```

## Documentation

| Audience | English | 日本語 |
|---|---|---|
| Application developers | [Kivact guide](docs/for-kivact-user/en/README.md) | [Kivact ガイド](docs/for-kivact-user/jp/README.md) |
| UI kit users | [UI kit guide](docs/for-kivact-ui-user/en/README.md) | [UI キット ガイド](docs/for-kivact-ui-user/jp/README.md) |
| Kivact developers | — | [製品戦略](docs/for-kivact-developer/strategy.md) |

The API reference is the rustdoc: `cargo doc -p kivact --open`.

## Crates

| Crate | Purpose |
|---|---|
| `kivact` | The framework. Applications and component libraries depend on this crate only. |
| `kivact-ui-basic` | Neutral general-purpose kit (after the shadcn/ui "Lyra" preset). |
| `kivact-ui-pop` | Calm business-application kit. |
| `kivact-ui-game` | Game-style control panels and HUDs. |
| `kivact-ui-creative` | Creative editing tools: timelines, node graphs, docks, inspectors. |
| `kivact-ui-soft` | Astryx Neutral in light and dark appearances. |
| `kivact-gpu-share` | Engine-side half of GPU texture sharing (experimental). |

The other crates under [`crates/`](crates) are implementation details of
`kivact` (reactive core, engine, layout, text and the native host Arvane)
and are not meant to be used directly. [`architecture/`](architecture)
contains the generated dependency graphs.

## Cargo features of `kivact`

| Feature | Default | Description |
|---|---|---|
| `gpu` | yes | GPU rendering with automatic CPU fallback. Without it the build is CPU-only. `ARVANE_RENDERER=cpu` forces the CPU renderer at run time. |
| `test` | no | The real-window testing library `kivact::test`. |
| `embed` | no | Experimental. Open an app inside a host window, e.g. an audio plugin editor. |
| `gpu-interop` | no | Experimental. Show GPU textures from the application's own renderer without read-back. |

## Platform support

| Platform | Status |
|---|---|
| Windows | Primary target. |
| Linux (Wayland, X11) | Primary target. Desktop, IME and screen reader coverage is partial. |
| macOS | Not supported yet. |
| Android, iOS | Considered in the design; not supported. |

## Repository layout

| Path | Contents |
|---|---|
| `crates/` | The framework, the native host and the UI kits |
| `examples/` | Small applications and the kit catalogs |
| `templates/basic/` | The smallest application with a test |
| `docs/` | User guides and the developer strategy |
| `dev/` | Workspace checks and the public-API test suite |
| `architecture/` | Generated crate and module dependency graphs |

## Contributing

Development is done by the author for now; see [CONTRIBUTING](CONTRIBUTING.md).
Report vulnerabilities privately as described in [SECURITY](SECURITY.md).

## License

Copyright (c) 2026 Kashiwade. Licensed under either of [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE) at your option. Bundled fonts and icons keep
their own licenses; see [THIRD-PARTY-NOTICES](THIRD-PARTY-NOTICES.md).
