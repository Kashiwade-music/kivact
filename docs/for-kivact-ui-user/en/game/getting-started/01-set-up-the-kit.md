# Step 1: Set up the kit

[日本語](../../../jp/game/getting-started/01-set-up-the-kit.md)

In this step you add the Game kit to a project and open a window with a
framed panel, a caption, a heading and a slanted button in the Game look.

## Create the project

```text
cargo new loadout
cd loadout
```

Kivact and its kits are not on crates.io yet, so add them as Git
dependencies in `Cargo.toml`:

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }
kivact-ui-game = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

The `dev-dependencies` line turns on the testing tools you use in step 3.

## What a kit does

A kit is a **theme**. Kivact's standard components (`button`, `list`, `tabs`,
`text_input` and so on) come from the `kivact` crate, and they look plain until
a theme tells them how to draw themselves. `kivact-ui-game` provides that
theme, plus components of its own: panel frames, meters, badges and monospaced
captions. You keep writing the same `button(..)` as in the core tutorial; the
kit changes how it looks.

## Install the theme

Replace `src/main.rs`:

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_game::{self as game, Density, ThemeOptions as _};

fn loadout() -> impl IntoView {
    game::hud(
        column((
            game::mono("SQUAD 04 / SLOT 2"),
            text("LOADOUT").style(style().font_size(24).weight(700)),
            button("Deploy").variant(game::Slant),
        ))
        .style(style().gap(10).padding(20)),
    )
    .style(style().width(360).margin(24))
}

fn main() -> kivact::Result<()> {
    App::new("Loadout")
        .theme(game::theme(ThemeMode::Dark).density(Density::Comfortable))
        .run(loadout)
}
```

Run it:

```text
cargo run
```

You should see a dark window with a bordered panel. Two of its corners have
short bright brackets (top left and bottom right), the caption is in a
monospaced font, and the Deploy button is slanted like a parallelogram with a
small yellow square at its top right. Hover over the button and it lightens;
press it and it sinks two pixels.

Four things are new:

- `game::theme(ThemeMode::Dark)` builds the Game theme, and `App::theme(..)`
  installs it for the whole window. The result is an ordinary Kivact `Theme`.
  `ThemeMode::Light` gives the light console palette.
- `.density(Density::Comfortable)` is the kit's one option. It sets the
  metrics: `Comfortable` (the default) has 34 px controls and 20 px panel
  padding, `Compact` has 28 px controls and 14 px padding. Try `Density::Compact`
  and run again; the button and panel shrink. The method comes from the
  trait `ThemeOptions`, which must be in scope: that is the `ThemeOptions as _`
  in the `use` line.
- `game::hud(..)` is a Game component: a bordered surface with accent brackets
  on two opposite corners. `game::mono(..)` is a small monospaced caption for
  ids, shortcuts and machine-like values.
- `.variant(game::Slant)` is how a kit adds a look to a standard component.
  The button is still the `button` you know; only its drawing changes.

The app keeps its state in signals, as before. The kit never owns your data.

## Fonts

The Game kit works with the font that Kivact bundles, so you need nothing
more for this tutorial. The kit's catalog also registers Barlow Condensed and
JetBrains Mono with `App::font` and names them in the theme's tokens for
the display and monospaced text. Fonts belong to the application. See
[Set the theme and density](../how-to-guide/theme-and-density.md) if you want to
do the same.

Next: [Step 2: Build the panel](02-build-the-panel.md)
