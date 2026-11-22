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
use kivact_ui_game::{self as game, Density, ThemeOptions};

fn app() -> impl Element {
    column((text("Console"), button("Run").variant(kivact::ui::Slant)))
        .style(style().gap(12).padding(24))
}

fn main() -> kivact::Result<()> {
    App::new("Console")
        .theme(game::theme_with(ThemeMode::Dark, ThemeOptions { density: Density::Compact, ..Default::default() }))
        .run(app)
}
```

The theme styles all standard components of `kivact::ui` (buttons, inputs,
checkboxes, switches, lists, tabs, radio groups, sliders, tables, menus,
dialogs, toasts) and the kit's own components. It also installs the kit's
icons, so `icon("shield-check")` draws one. You choose `ThemeMode::Light` or
`ThemeMode::Dark` yourself; the theme does not read the operating system.

## Choose the density

`ThemeOptions { density, ..Default::default() }` is the kit's option, passed to `game::theme_with(mode, options)`. It changes only the console metrics:

| | `Density::Comfortable` (default) | `Density::Compact` |
| --- | --- | --- |
| Control height | 34 px | 28 px |
| Table row height | 42 px | 32 px |
| Panel padding | 20 px | 14 px |

The colors and the font families stay the same. Use `Compact` for dense
tools and `Comfortable` for menus a player reads at a distance.

## Switch mode and density at run time

Keep the choices in signals and give `provide_theme` a closure that builds the
theme. Every style re-resolves when a signal changes; nothing is remounted, so
typed text, scroll position and focus stay.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::provide_theme;
use kivact_ui_game::{self as game, Density, ThemeOptions};

fn app() -> impl IntoView {
    let dark = signal(true);
    let compact = signal(false);
    let theme = move || {
        let mode = if dark.get() { ThemeMode::Dark } else { ThemeMode::Light };
        let density = if compact.get() { Density::Compact } else { Density::Comfortable };
        game::theme_with(mode, ThemeOptions { density, ..Default::default() })
    };
    provide_theme(theme, move || {
        column((
            switch("Dark", dark),
            switch("Compact", compact),
            text_input(signal(String::new())).label("Callsign"),
        ))
        .style(
            style()
                .gap(12)
                .padding(24)
                .height(pct(100))
                .background(game::color::CANVAS)
                .color(kivact::style::color::TEXT),
        )
    })
}

fn main() -> kivact::Result<()> {
    App::new("Settings").run(app)
}
```

`provide_theme` creates no node and paints nothing, so give the top node of the
subtree the page colors. `game::color::CANVAS` is the kit's page background,
and it follows the theme as it switches.

## Change colors and fonts

The kit's own values are theme variables. A color is a `ThemeVar<Color>` static in
`kivact_ui_game::color`, such as `CYAN_VAR`, and a font family is a `ThemeVar<FontFamily>`
static in `kivact_ui_game::fonts`: `DISPLAY_FAMILY_VAR` for headings, buttons and numbers,
and `MONO_FAMILY_VAR` for codes and identifiers. The `const` of the same name (`CYAN`)
reads the variable in a style. Change a variable with `Theme::set`, which returns the
theme, so you can chain several calls:

```rust,no_run
use kivact::prelude::*;
use kivact::style::{FontFamily, ThemeMode, rgb};
use kivact_ui_game::{self as game, Density, ThemeOptions};

fn console_theme() -> Theme {
    game::theme_with(ThemeMode::Dark, ThemeOptions { density: Density::Compact, ..Default::default() })
        // The kit's own variables: an accent color and the display and monospaced families.
        .set(&game::color::CYAN_VAR, rgb(0xffb300))
        .set(&game::fonts::DISPLAY_FAMILY_VAR, FontFamily::from("Barlow Condensed"))
        .set(&game::fonts::MONO_FAMILY_VAR, FontFamily::from("JetBrains Mono"))
}

/// The font files are yours to embed, for example
/// `include_bytes!("../assets/BarlowCondensed-Bold.ttf")`.
fn application(display: &'static [u8], mono: &'static [u8]) -> App {
    App::new("Console")
        .theme(console_theme().with_fonts([display, mono]))
}
# fn main() {}
```

- `DISPLAY_FAMILY_VAR` is used by buttons, tabs, headings and big numbers;
  `MONO_FAMILY_VAR` by `game::mono`, KPI captions and the table captions.
  Both default to `sans-serif` and `monospace`.
- Fonts belong to the application: pass the font files to `Theme::with_fonts` and use
  the family name written inside the font, not the file name.
- `ThemeOptions` also has `display_font` and `mono_font` fields, which set the two
  families when you build the theme with `theme_with`.
- Choose the density with `theme_with` before you change the variables.

The kit's brand colors for the standard parts (accent, success, warning,
danger, border) are the theme variables of `kivact::style::color`, because the
standard components read them. To recolor the accent everywhere, set
`color::ACCENT_VAR` as well as the kit variable you care about:

```rust,no_run
use kivact::prelude::*;
use kivact::style::{ThemeMode, color, rgb};
use kivact_ui_game as game;

fn orange_theme() -> Theme {
    game::theme(ThemeMode::Dark).set(&color::ACCENT_VAR, rgb(0xff6a00))
}
# fn main() {}
```

## Use the kit's colors in your own components

Pass `game::color::NAVY` or any other `game::color::*` constant anywhere a style takes a color: the value is read
from the theme the node is painted in, so it follows theme switches without a
closure.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

#[component]
fn warning_strip(message: Value<String>) -> impl Element {
    row((game::mono("WARN"), text(message))).style(
        style()
            .gap(10)
            .padding(8)
            .background(game::color::NAVY)
            .color(game::color::YELLOW)
            .border_left(3, game::color::CYAN),
    )
}
# fn main() {}
```

The standard `kivact::style::color::SURFACE`, `TEXT`, `TEXT_MUTED`, `BORDER`, `ACCENT`
and `DANGER` work with every theme and are the right choice for a component
you want to use in other kits too.

## Replace the kit's texts

The words that components draw themselves (a dialog's Close, the command
palette's hint, a table's "Progress") are English. Replace them with
`App::messages`; the keys are listed in the rustdoc of each component.

## See also

- [Show items in lists and trees](item-lists-and-selection.md)
- [Show meters and status](meters-and-status.md)
- Every token and component: `cargo doc -p kivact-ui-game --open`
