# Step 1: Set up the kit

[日本語](../../../jp/pop/getting-started/01-set-up-the-kit.md)

In this step you add the Pop kit to a project and open a window with a card,
a heading, a text field and a button in the Pop look.

## Create the project

```text
cargo new new-account
cd new-account
```

Kivact and its kits are not on crates.io yet, so add them as Git
dependencies in `Cargo.toml`:

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }
kivact-ui-pop = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

The `dev-dependencies` line turns on the testing tools you use in step 3.

## What a kit does

A kit is a **theme**. Kivact's standard components (`text_input`, `button`,
`radio_group`, `tabs` and so on) come from the `kivact` crate, and they look
plain until a theme tells them how to draw themselves. `kivact-ui-pop`
provides that theme for standard cards, date inputs, badges and alerts, plus
its scheduling and business display components. You keep writing the same `button(..)` and
`text_input(..)` as in the core tutorial; the kit changes how they look.

## Install the theme

Replace `src/main.rs`:

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;

fn sign_up() -> impl IntoView {
    let name = signal(String::new());
    kivact::ui::card((
        text("New account").style(style().font_size(20).weight(600)),
        text_input("Name", name),
        button("Create account"),
    ))
    .style(style().width(360).margin(24).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("New account")
        .theme(pop::theme(ThemeMode::Light))
        .run(sign_up)
}
```

Run it:

```text
cargo run
```

You should see a white card with a thin border and rounded corners on a
light background, a heading, a rounded text field and a blue button. Click the
field and type; the field shows a blue focus ring.

Three things are new:

- `pop::theme(ThemeMode::Light)` builds the Pop theme, and
  `App::theme(..)` installs it for the whole window. The result is an
  ordinary Kivact `Theme`. Use `ThemeMode::Dark` for the dark palette.
- `kivact::ui::card(..)` is a standard component with the Pop appearance: a bordered, rounded card that lays its
  children out in a column.
- `use kivact_ui_pop as pop;` is only a short name. Components of the kit are
  written `pop::name`; standard components stay unqualified.

The app keeps its state in a `signal`, exactly as before. The kit never owns
your data: `name` is yours, and `text_input` reads and writes it.

## Fonts

Pop works with the font that Kivact bundles, so you need nothing more for this
tutorial. The kit's own catalog also registers Inter, Barlow and Noto Sans JP
with `App::font` to match the original design. Fonts belong to the
application. See
[Set up the Pop theme and tokens](../how-to-guide/theme-and-tokens.md) if you
want to do the same.

Next: [Step 2: Build the form](02-build-the-form.md)
