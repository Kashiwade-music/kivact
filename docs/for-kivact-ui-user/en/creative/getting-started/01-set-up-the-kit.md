# 1. Set up the Creative kit

[日本語](../../../jp/creative/getting-started/01-set-up-the-kit.md)

In this step you add `kivact-ui-creative` to a project, install its theme and show the first piece of the inspector: a panel with one numeric field.

## 1. Add the dependency

Create a project with `cargo new layer-inspector` and add Kivact and the Creative kit to `Cargo.toml`. Kivact is not on crates.io yet, so both come from the Git repository:

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }
kivact-ui-creative = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

The `dev-dependencies` entry turns on the test tools that step 3 uses. Rust 1.98.1 (edition 2024) is the supported toolchain; copy `rust-toolchain.toml` from the Kivact repository if you want the same one.

## 2. Show a panel

Replace `src/main.rs`:

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_creative as ui;

fn inspector() -> impl IntoView {
    let x = signal(120.0);
    ui::panel(
        "Inspector",
        column((ui::property_row(
            "X",
            ui::number("X", x)
                .range(kivact::interaction::ScalarRange::new(-1000.0, 1000.0).expect("valid range"))
                .step(1.0)
                .precision(1)
                .unit(" px")
                .label_visible(false),
        ),)).role(kivact::view::Role::List)
        .style(style().gap(6).padding(10)),
    )
    .style(style().width(280))
}

fn main() -> kivact::Result<()> {
    App::new("Layer inspector")
        .theme(ui::theme(ThemeMode::Dark))
        .run(inspector)
}
```

Run it with `cargo run`. You should see a dark window with a rounded **Inspector** panel that holds one row: the label **X** and a field showing `120.0 px`.

## What you wrote

- `ui::theme(ThemeMode::Dark)` is an ordinary Kivact `Theme`. `App::theme` installs it for the whole window, so the kit's components, and the standard components from `kivact::ui` (buttons, text fields, menus), all get the Creative look. It needs no font files: text uses the system `sans-serif` and `monospace` families (to use your own fonts, see [Theme the Creative kit, set fonts and add icons](../how-to-guide/theme-and-icons.md#use-your-own-fonts)).
- `signal(120.0)` is the value the field edits. It is a `Copy` handle, like a SolidJS signal, so you can pass `x` to several places without cloning.
- `ui::number("X", x)` is a numeric field. The first argument is its accessible name (also drawn inside the field unless you call `.label_visible(false)`, which the row's own label makes redundant), and `x` is the value it edits. The field keeps no copy of its own: every write goes straight to your signal.
- `ui::property_row("X", field)` is an inspector row: a right-aligned label, then the value control.
- `ui::panel(title, content)` is a rounded "island" with a title bar. The kit's components are builders, so ordinary modifiers such as `.style(..)` work on all of them.

## Try it

Drag the field left and right, or click it and type `2 * 8` followed by Enter. The field accepts arithmetic and clamps the result to the range you gave. The next page shows what else it can do.

## Next

[2. Edit values](02-edit-values.md)
