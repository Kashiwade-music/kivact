# Set up the Pop theme and tokens

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
use kivact::style::provide_theme;
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
            text_input("Search", signal(String::new())),
        ))
        .style(
            style()
                .gap(12)
                .padding(24)
                .height(pct(100))
                .background(pop::ColorName::Background)
                .color(pop::ColorName::Text),
        )
    })
}

fn main() -> kivact::Result<()> {
    App::new("Theme switch").run(app)
}
```

`provide_theme` creates no node and paints nothing, so the top node of the
subtree gives itself the page colors. `pop::ColorName` names the kit's colors
(`Background`, `Surface`, `Line`, `Text`, `TextMuted` and more). A style takes
one directly, and it is looked up in the theme the node is painted in, so it
follows the switch without a closure.

## Change colors

The kit's colors live in `pop::Tokens`, a plain struct with public fields:
`blue`, `mint`, `yellow`, `pink`, `red`, `navy`, `surface`, `line`, `text` and
more. Components read it from the theme, so changing it restyles them all.
Copy the tokens, change a field and install them with `Theme::with`:

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;

fn brand_theme() -> Theme {
    let theme = pop::theme(ThemeMode::Light);
    let mut tokens = (*theme.ext::<pop::Tokens>()).clone();
    tokens.blue = rgb(0x0f766e);
    let mut colors = theme.colors().clone();
    colors.accent = tokens.blue;
    theme.with(tokens).with_colors(colors)
}

fn main() -> kivact::Result<()> {
    App::new("Teal")
        .theme(brand_theme())
        .run(|| button("Teal").variant(kivact::ui::ButtonVariant::Primary))
}
```

Two tables are involved, which is why the example changes both. `Tokens` holds
the kit's own colors (the blue of a primary button, the hue of a badge).
`theme.colors()` is Kivact's shared palette that standard parts read, such as
the accent of a slider track. `pop::theme` fills the palette from the tokens
once, so a token you change later does not move the palette: set both, as
above.

`Tokens::hue(hue)` returns the background and foreground colors of one of the
six `pop::Hue` values (`Blue`, `Mint`, `Yellow`, `Pink`, `Red`, `Gray`), the
colors badges, avatars and stat cards use.

## Read the tokens in your own components

Prefer `pop::ColorName::..` in a style, as above. Read the `Tokens` struct itself
when you need to compute something from it, such as the two colors of a hue.
Do it in a style closure, so the result follows theme switches:

```rust,no_run
use kivact::prelude::*;
use kivact::style::use_theme;
use kivact_ui_pop as pop;

fn highlight(content: impl IntoView) -> Node {
    column(content).style(move || {
        let theme = use_theme().get();
        let tokens = theme.ext::<pop::Tokens>();
        let (background, foreground) = tokens.hue(pop::Hue::Yellow);
        style()
            .padding(12)
            .radius(tokens.control_radius)
            .background(background)
            .color(foreground)
    })
}
# fn main() {}
```

For plain colors you do not need the kit at all: `ColorToken::Surface`,
`Text`, `TextMuted`, `Border`, `Accent` and `Danger` come from the shared
palette and work with any theme, so a component that uses only them also works
in other kits.

## Fonts and type

Fonts belong to the application. Register the files you ship with `App::font`
and name the family in the theme's typography. The kit's catalog uses Inter
for text and Noto Sans JP for Japanese; three Pop components (the
`BadgeVariant::Slant` badge, avatars and the numbered `section_heading`) ask for the family
`Barlow` and fall back to `sans-serif` when it is not registered.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::theme::Typography;
use kivact_ui_pop as pop;

fn themed() -> Theme {
    let theme = pop::theme(ThemeMode::Light);
    let text = Typography {
        family: Some("Inter, 'Noto Sans JP', sans-serif".into()),
        ..theme.text().clone()
    };
    theme.with_text(text)
}

/// `inter` and `noto` are font files you embed, for example
/// `include_bytes!("../assets/Inter.ttf")`.
fn application(inter: &'static [u8], noto: &'static [u8]) -> App {
    App::new("Pop").font(inter).font(noto).theme(themed())
}
# fn main() {}
```

The family is the name written inside the font file, not the file name.

## Replace the kit's texts

Texts that components draw themselves (the calendar's month buttons, the date
format error, a dropzone's picker button) are English. Replace them with
`App::messages`; the keys start with `pop.`, for example `pop.date.open` and
`pop.date.error.format`. [Pick dates and ranges](dates-and-calendars.md) shows
an example.

## See also

- [Pick dates and ranges](dates-and-calendars.md)
- [Show rows in a table](tables.md)
- Full list of tokens and components: `cargo doc -p kivact-ui-pop --open`

Use the standard drop target with the Pop theme. Enable picking and multiple files explicitly; the decorative icon is a mounted slot.

```rust,no_run
use kivact::prelude::*;

fn upload() -> impl IntoView {
    kivact::ui::dropzone("Upload")
        .multiple()
        .pick_on_press()
        .icon(icon("cloud-upload").style(style().size(24, 24)))
        .hint("PNG, PDF")
        .on_files(|files| {
            for file in files {
                println!("{}", file.path.display());
            }
        })
}
```
