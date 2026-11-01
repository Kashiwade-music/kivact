# Set the theme and density

[日本語](../../../jp/game/how-to-guide/theme-and-density.md)

Use this guide to install the Game look, choose light or dark, switch the
compact density, change colors and fonts, and use the kit's colors in your own
components.

## Install the theme

`game::theme(mode)` returns an ordinary Kivact `Theme`. Give it to
`App::theme`:

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_game::{self as game, Density, ThemeOptions as _};

fn app() -> impl IntoView {
    column((text("Console"), button("Run").variant(game::Slant)))
        .style(style().gap(12).padding(24))
}

fn main() -> kivact::Result<()> {
    App::new("Console")
        .theme(game::theme(ThemeMode::Dark).density(Density::Compact))
        .run(app)
}
```

The theme styles all standard components of `kivact::ui` (buttons, inputs,
checkboxes, switches, lists, tabs, radio groups, sliders, tables, menus,
dialogs, toasts) and the kit's own components. It also installs the kit's
icons, so `icon("shield-check")` draws one. You choose `ThemeMode::Light` or
`ThemeMode::Dark` yourself; the theme does not read the operating system.

## Choose the density

`.density(..)` is the one option of the kit. It belongs to the trait
`ThemeOptions`, which has to be in scope (`ThemeOptions as _` in the `use`
line). It changes the metrics of the console and nothing else:

| | `Density::Comfortable` (default) | `Density::Compact` |
| --- | --- | --- |
| Control height | 34 px | 28 px |
| Table row height | 42 px | 32 px |
| Panel padding | 20 px | 14 px |

The colors, the font families and the rest of the tokens stay as they are, so
you can apply `.density(..)` after changing tokens. Use `Compact` for dense
tools and `Comfortable` for menus a player reads at a distance.

## Switch mode and density at run time

Keep the choices in signals and give `provide_theme` a closure that builds the
theme. Every style re-resolves when a signal changes; nothing is remounted, so
typed text, scroll position and focus stay.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::view::provide_theme;
use kivact_ui_game::{self as game, Density, ThemeOptions as _};

fn app() -> impl IntoView {
    let dark = signal(true);
    let compact = signal(false);
    let theme = move || {
        let mode = if dark.get() { ThemeMode::Dark } else { ThemeMode::Light };
        let density = if compact.get() { Density::Compact } else { Density::Comfortable };
        game::theme(mode).density(density)
    };
    provide_theme(theme, move || {
        column((
            switch("Dark", dark),
            switch("Compact", compact),
            text_input("Callsign", signal(String::new())),
        ))
        .style(
            style()
                .gap(12)
                .padding(24)
                .height(pct(100))
                .background(game::Color::Canvas)
                .color(ColorToken::Text),
        )
    })
}

fn main() -> kivact::Result<()> {
    App::new("Settings").run(app)
}
```

`provide_theme` creates no node and paints nothing, so give the top node of the
subtree the page colors. `game::Color::Canvas` is the kit's page background,
and it follows the theme as it switches.

## Change colors and fonts

The kit's own values are in `game::Tokens`, a plain struct with public fields:
the font families `display_family` and `mono_family`, the colors `yellow`,
`cyan`, `navy`, `canvas` and others, and the metrics `control_height`,
`row_height` and `padding`. Copy the tokens, change fields and install them with
`Theme::with`:

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::theme::Typography;
use kivact_ui_game::{self as game, Density, ThemeOptions as _};

fn console_theme() -> Theme {
    let theme = game::theme(ThemeMode::Dark).density(Density::Compact);

    // The kit's own tokens: display and monospaced families, an accent color.
    let mut tokens = (*theme.get::<game::Tokens>()).clone();
    tokens.display_family = "Barlow Condensed".into();
    tokens.mono_family = "JetBrains Mono".into();
    tokens.yellow = rgb(0xffb300);

    // Kivact's shared type settings: the font of ordinary text.
    let text = Typography {
        family: Some("Noto Sans JP".into()),
        ..theme.text().clone()
    };
    theme.with(tokens).with_text(text)
}

/// The font files are yours to embed, for example
/// `include_bytes!("../assets/BarlowCondensed-Bold.ttf")`.
fn application(display: &'static [u8], mono: &'static [u8], text: &'static [u8]) -> App {
    App::new("Console")
        .font(display)
        .font(mono)
        .font(text)
        .theme(console_theme())
}
# fn main() {}
```

- `display_family` is used by buttons, tabs, headings and big numbers;
  `mono_family` by `game::mono`, KPI captions and the table captions.
  Both default to `sans-serif` and `monospace`.
- Fonts belong to the application: register each file with `App::font` and use
  the family name written inside the font, not the file name.
- Call `.density(..)` after editing tokens if you want the density to win, as
  it replaces the metrics fields (`control_height`, `row_height`, `padding`).

The kit's brand colors for the standard parts (accent, success, warning,
danger, border) live in the theme's palette (`theme.colors()` and
`.with_colors(..)`), because the standard components read the palette. The
tokens hold what the kit adds on top. To recolor the accent everywhere,
change `colors.accent` as well as the token you care about:

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_game as game;

fn orange_theme() -> Theme {
    let theme = game::theme(ThemeMode::Dark);
    let mut colors = theme.colors().clone();
    colors.accent = rgb(0xff6a00);
    theme.with_colors(colors)
}
# fn main() {}
```

## Use the kit's colors in your own components

Pass `game::Color::..` anywhere a style takes a color: the value is looked up
in the theme the node is painted in, so it follows theme switches without a
closure.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn warning_strip(message: &'static str) -> Node {
    row((game::mono("WARN"), text(message))).style(
        style()
            .gap(10)
            .padding(8)
            .background(game::Color::Navy)
            .color(game::Color::Yellow)
            .border_left(3, game::Color::Cyan),
    )
}
# fn main() {}
```

The standard `ColorToken::Surface`, `Text`, `TextMuted`, `Border`, `Accent`
and `Danger` work with every theme and are the right choice for a component
you want to use in other kits too. Read a `Tokens` field by hand only when you
need to compute a new color from it.

## Replace the kit's texts

The words that components draw themselves (a dialog's Close, the command
palette's hint, a table's "Progress") are English. Replace them with
`App::messages`; the keys are listed in the rustdoc of each component.

## See also

- [Show items in lists and trees](item-lists-and-selection.md)
- [Show meters and status](meters-and-status.md)
- Every token and component: `cargo doc -p kivact-ui-game --open`
