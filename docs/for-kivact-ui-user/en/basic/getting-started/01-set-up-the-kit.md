# 1. Set up the kit

[日本語](../../../jp/basic/getting-started/01-set-up-the-kit.md)

In this step you add `kivact-ui-basic` to a new project, apply the kit's theme to the app, and open a window with a heading.

## Create the project

Create a binary project and add the dependencies. Kivact is not on crates.io yet, so both crates come from the Git repository.

```text
cargo new settings
cd settings
```

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }
kivact-ui-basic = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

Kivact is built with Rust 1.98.1 and edition 2024 (the repository pins them in `rust-toolchain.toml`). The `dev-dependencies` entry turns on the `test` feature, which you use for the test in step 3.

## Write the first screen

Replace `src/main.rs` with this program:

```rust,no_run
use kivact::prelude::*;
use kivact::style::theme::Typography;
use kivact::style::{Theme, ThemeMode};
use kivact_ui_basic as basic;

/// The kit theme with Inter as the text family.
fn theme(mode: ThemeMode) -> Theme {
    let theme = basic::theme(mode);
    let text = Typography {
        family: Some("Inter, sans-serif".into()),
        ..theme.text().clone()
    };
    theme.with_text(text)
}

/// Font files to register with the app. Empty for now: the window uses the system font.
fn fonts() -> Vec<&'static [u8]> {
    Vec::new()
}

fn settings() -> impl IntoView {
    column((
        basic::h3("Settings"),
        basic::muted("Manage your profile and notifications."),
    ))
    .style(
        style()
            .size(pct(100), pct(100))
            .padding(24)
            .gap(8)
            .background(basic::Color::Background)
            .color(basic::Color::Foreground),
    )
}

fn main() -> kivact::Result<()> {
    let mut app = App::new("Settings").theme(theme(ThemeMode::Light));
    for font in fonts() {
        app = app.font(font);
    }
    app.run(settings)
}
```

Run it with `cargo run`. You should see a white window with the heading "Settings" and a gray line of text below it.

## What the code does

- `basic::theme(mode)` returns a Kivact `Theme`. It carries the kit's colors, spacing, 12 px text and the drawing rules (recipes) for every standard control: buttons, switches, inputs, selects, dialogs and more. You pass it to `App::theme`, and every standard component under the app looks like the Lyra preset without any further code.
- `basic::h3` and `basic::muted` are the kit's typography helpers. They return a `Node`, the plain building block of a view, so you can place them anywhere.
- `basic::Color` names a color of the kit's palette, such as `Background` or `Foreground`. A style that uses it reads the color from the theme that is active when it is drawn. That is why it follows the theme when you switch it in step 3. The root view is painted with the page colors so that the whole window follows the theme, the way a page sets its colors on `body` in CSS.
- `fonts()` is where the app's font files go. The kit does not ship fonts; the `Typography` change in `theme` asks for the family `Inter`, with `sans-serif` as the fallback. Until you register Inter, the system font is used, and everything in this tutorial still works.

## Try the dark palette

In `main`, change `ThemeMode::Light` to `ThemeMode::Dark` and run again. The page turns near-black and the text turns white. Change it back to `Light`; step 3 switches the mode while the app runs.

## Optional: register Inter

The kit is sized for Inter (text) and Geist Mono (code blocks). Both are open fonts under the SIL OFL. To use Inter, download `Inter[opsz,wght].ttf`, put it in `assets/fonts/` in your project, and return its bytes from `fonts`:

```text
fn fonts() -> Vec<&'static [u8]> {
    vec![include_bytes!("../assets/fonts/Inter[opsz,wght].ttf") as &[u8]]
}
```

The family name is the one written inside the font file, not the file name. Later steps do not depend on this.

## Next

[2. Build the settings form](02-build-the-settings-form.md) puts cards, inputs and switches on this screen.
