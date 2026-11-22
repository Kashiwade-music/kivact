# Set up the Pop theme and variables

[日本語](../../../jp/pop/how-to-guide/theme-and-tokens.md)

Use this guide to install the Pop look, switch between light and dark at run
time, change its colors and fonts, and read its values from your own
components.

## Install the theme

`pop::theme(mode)` returns a Kivact `Theme`. Give it to `App::theme` to
restyle the whole window:

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;

fn app() -> impl IntoView {
    column((
        text("Settings"),
        button("Save").variant(kivact::ui::ButtonVariant::Primary),
    ))
    .style(style().gap(12).padding(24))
}

fn main() -> kivact::Result<()> {
    App::new("Settings")
        .theme(pop::theme(ThemeMode::Light))
        .run(app)
}
```

The theme styles every standard component of `kivact::ui` (buttons, inputs,
checkboxes, tabs, radio groups, sliders, tables, dialogs, menus, toasts) and
every component of `kivact-ui-pop`. It also installs the kit's icons, so
`icon("check")` draws a Pop icon. It does not read the operating system: you
choose `ThemeMode::Light` or `ThemeMode::Dark`.

## Switch light and dark at run time

Keep the choice in a signal and wrap the root in `provide_theme`. Pass it a
closure that builds the theme: every style re-resolves when the signal
changes, and nothing is remounted, so typed text and focus stay where they
are.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::{color, provide_theme};
use kivact_ui_pop as pop;

fn app() -> impl IntoView {
    let dark = signal(false);
    let theme = move || {
        pop::theme(if dark.get() {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        })
    };
    provide_theme(theme, move || {
        column((
            checkbox("Dark mode", dark),
            text_input(signal(String::new())).label("Search"),
        ))
        .style(
            style()
                .gap(12)
                .padding(24)
                .height(pct(100))
                .background(color::SURFACE)
                .color(color::TEXT),
        )
    })
}

fn main() -> kivact::Result<()> {
    App::new("Theme switch").run(app)
}
```

`provide_theme` creates no node and paints nothing, so the top node of the
subtree gives itself the page colors. `color::SURFACE` and `color::TEXT` are
the core color variables; the Pop look is drawn from them, so a style that
takes one follows the switch without a closure.

## Change colors

Pop's colors are the core color variables of `kivact::style::color`: the
accent of a primary button, the danger color of a destructive action, the
border of a card. The kit sets them when the theme is built, so you change a
color by setting its variable on the theme. Components read the variable in
the theme in effect, so changing it restyles them all:

```rust,no_run
use kivact::prelude::*;
use kivact::style::{Theme, ThemeMode, color, rgb};
use kivact_ui_pop as pop;

fn brand_theme() -> Theme {
    pop::theme(ThemeMode::Light).set(&color::ACCENT_VAR, rgb(0x0f766e))
}

fn main() -> kivact::Result<()> {
    App::new("Teal")
        .theme(brand_theme())
        .run(|| button("Teal").variant(kivact::ui::ButtonVariant::Primary))
}
```

Set the light and dark values separately when the app can switch: build the
theme for the mode it is given, as `brand_theme` does for light.

The variables you will most often change are `color::ACCENT_VAR` (the primary
action and the accent of sliders), `color::DANGER_VAR` (destructive actions),
`color::SUCCESS_VAR`, `color::WARNING_VAR`, `color::BORDER_VAR` and
`color::SURFACE_VAR` (the page and card surfaces). Each one has a matching
constant, such as `color::ACCENT`, for use in a style.

## Read the variables in your own components

Use the constants in a style, as above. Read the variable itself when you need
to compute something from it. Do it in a style closure, so the result follows
theme switches:

```rust,no_run
use kivact::prelude::*;
use kivact::style::{color, radius, use_theme};

#[component]
fn highlight(content: View) -> impl IntoView {
    column(content).style(move || {
        let accent = use_theme().get().get(&color::ACCENT_VAR);
        style()
            .padding(12)
            .border_radius(radius::MD)
            .background(accent.with_alpha(0.15))
            .color(color::TEXT)
    })
}
```

## Fonts and type

Fonts belong to the application. Name the family in the theme with
`font::FAMILY_VAR`, and register the files you ship with `Theme::with_fonts`.
The kit's catalog uses Inter for text and Noto Sans JP for Japanese; three Pop
components (the `kivact::ui::Slant` badge, avatars and the numbered
`SectionNumber`) ask for the family `Barlow` and fall back to `sans-serif` when
it is not registered.

```rust,no_run
use kivact::prelude::*;
use kivact::style::{FontFamily, Theme, ThemeMode, font};
use kivact_ui_pop as pop;

fn themed() -> Theme {
    // `Inter` and `Noto Sans JP` are the families the catalog uses.
    pop::theme(ThemeMode::Light)
        .set(&font::FAMILY_VAR, FontFamily::from("Inter, 'Noto Sans JP', sans-serif"))
}

/// `inter` and `noto` are font files you embed, for example
/// `include_bytes!("../assets/Inter.ttf")`.
fn application(inter: &'static [u8], noto: &'static [u8]) -> App {
    App::new("Pop").theme(themed().with_fonts([inter, noto]))
}
# fn main() {}
```

The family is the name written inside the font file, not the file name.

## Replace the kit's texts

Texts that components draw themselves (the calendar's month buttons, the date
format error, a dropzone's picker button) are English. Replace them with
`App::messages`; standard components use canonical keys, for example `date.open` and
`date.error.format`. [Pick dates and ranges](dates-and-calendars.md) shows
an example.

## See also

- [Pick dates and ranges](dates-and-calendars.md)
- [Show rows in a table](tables.md)
- Full list of components: `cargo doc -p kivact-ui-pop --open`

Use the standard drop target with the Pop theme. Enable picking and multiple files explicitly; the decorative icon is a mounted slot.

```rust,no_run
use kivact::prelude::*;

#[component]
fn upload() -> impl Element {
    kivact::ui::dropzone("Upload")
        .multiple(true)
        .pick_on_press(true)
        .icon(icon("cloud-upload").style(style().size(24, 24)))
        .hint("PNG, PDF")
        .on_files(|files| {
            for file in files {
                println!("{}", file.path.display());
            }
        })
}
```
