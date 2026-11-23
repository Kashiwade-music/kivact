# Apply, switch and customize the theme

[日本語](../../../jp/basic/how-to-guide/theme-and-tokens.md)

This guide shows how to give an app the Basic look, switch between light and dark while it runs, read and change the kit's colors, adjust how a standard component is drawn, and use the Lucide icons. It assumes you have done the [tutorial](../getting-started/README.md).

## Apply the theme

Pass `basic::theme(mode)` to `App::theme`:

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_basic as basic;

fn main() -> kivact::Result<()> {
    App::new("Basic")
        .theme(basic::theme(ThemeMode::Light))
        .run(|| button("Save"))
}
```

The theme carries the Neutral palette, 12 px text, square corners, 32 px controls and the drawing rules (recipes) for every standard component in `kivact::ui`: `button`, `toggle`, `checkbox`, `switch`, `text_input`, `text_area`, `select`, `menu`, `dialog`, `tooltip`, `tabs`, `radio_group`, `slider`, `range_slider`, `toast_region`, `splitter`, `split`, `progress`, `disclosure`, `data_table` and more. The kit has no copies of these components. That is why a `kivact::ui::button` and a `kivact::ui::card` sit together without any glue, and why changing the theme restyles both.

To limit the look to one part of the window, wrap that part in `kivact::style::provide_theme(theme, || ..)`. Siblings outside it keep the theme from above.

## Switch light and dark while the app runs

Pass a closure that reads a signal instead of a fixed theme:

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_basic as basic;

#[component]
fn root() -> impl IntoView {
    let dark = signal(false);
    kivact::style::provide_theme(
        move || {
            basic::theme(if dark.get() { ThemeMode::Dark } else { ThemeMode::Light })
        },
        move || {
            column((
                switch("Dark mode", dark),
                button("Save"),
                text_input(signal(String::from("Ada"))).label("Name"),
            ))
            .style(
                style()
                    .size(pct(100), pct(100))
                    .padding(24)
                    .gap(12)
                    .background(basic::BACKGROUND)
                    .color(basic::FOREGROUND),
            )
        },
    )
}

fn main() -> kivact::Result<()> {
    App::new("Theme").run(root)
}
```

- Mounted components are restyled, not rebuilt, so typed text, focus and open state survive the switch.
- `provide_theme` paints nothing. Give the top node of the subtree the page colors yourself, as above. Text below a node without a color takes the color of the `App` theme.
- The mode is yours to store. If the app should remember it between runs, save the `dark` value yourself and read it back when you create the signal.
- Building the theme inside the closure costs little: the kit shares one recipe table and one icon set between themes, so an unchanged table never restyles.

## Choose the fonts

The kit does not ship fonts. It is sized for Inter and asks for `Geist Mono` in code blocks and inline code. Name the text family in the theme with `font::FAMILY_VAR`, and register the files with `Theme::with_fonts`:

```rust,no_run
use kivact::prelude::*;
use kivact::style::{FontFamily, Theme, ThemeMode, font};
use kivact_ui_basic as basic;

fn theme(mode: ThemeMode) -> Theme {
    // Add a CJK family after Inter when the app shows Japanese, Chinese or Korean.
    basic::theme(mode).set(&font::FAMILY_VAR, FontFamily::from("Inter, 'Noto Sans JP', sans-serif"))
}

fn app(fonts: &[&'static [u8]]) -> App {
    App::new("Basic").theme(theme(ThemeMode::Light).with_fonts(fonts.iter().copied()))
}
```

Pass the bytes from `include_bytes!` files in your `main` (`app(&[include_bytes!("../assets/Inter[opsz,wght].ttf") as &[u8], include_bytes!("../assets/GeistMono[wght].ttf")])`). The family name is the name written inside the font file, not the file name. A family that is not registered falls back to the system fonts.

The texts the components show themselves (Loading, Close, Copy, Next, ...) are English. Replace them with `App::messages`; the keys have the prefix `basic.` and are listed in the kit's README:

```rust,no_run
use kivact::i18n::Messages;
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn main() -> kivact::Result<()> {
    let messages = Messages::new()
        .with("basic.pagination.next", "Weiter")
        .with("basic.pagination.previous", "Zurück");
    App::new("Basic")
        .theme(basic::theme(kivact::style::ThemeMode::Light))
        .messages(messages)
        .run(|| kivact::ui::pagination(5usize, signal(0usize)))
}
```

## Use the kit's colors in your own styles

The kit's colors are constants of `kivact_ui_basic`: `basic::BACKGROUND`, `FOREGROUND`, `CARD`, `CARD_FOREGROUND`, `POPOVER`, `PRIMARY`, `PRIMARY_FOREGROUND`, `SECONDARY`, `MUTED`, `MUTED_FOREGROUND`, `ACCENT`, `DESTRUCTIVE`, `BORDER`, `INPUT`, `RING`, `CHART_1` to `CHART_5`, and the `SIDEBAR_*` colors. Each one is a theme variable. Every style method that takes a color accepts it, and it is resolved from the theme in effect when the node is drawn, so it follows a theme switch:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn banner() -> impl Element {
    text("Heads up").style(
        style()
            .padding(12)
            .background(basic::MUTED)
            .color(basic::MUTED_FOREGROUND)
            .border(1, basic::BORDER),
    )
}
```

To compute a new color from a theme color (a translucent primary, say), read the variable from the theme inside a style closure. The closure runs again when the theme changes:

```rust,no_run
use kivact::prelude::*;
use kivact::style::use_theme;
use kivact_ui_basic as basic;

#[component]
fn tinted() -> impl Element {
    text("Tinted").style(|| {
        let primary = use_theme().get().get(&basic::PRIMARY_VAR);
        style().padding(12).background(primary.with_alpha(0.1))
    })
}
```

## Change the colors

Each color is a variable, so you change one with `Theme::set` on the theme the kit returns. Set the variable, and every component and style that reads it follows:

```rust,no_run
use kivact::prelude::*;
use kivact::style::{Theme, ThemeMode, rgb};
use kivact_ui_basic as basic;

fn blue_theme(mode: ThemeMode) -> Theme {
    basic::theme(mode)
        .set(&basic::PRIMARY_VAR, rgb(0x2563eb))
        .set(&basic::PRIMARY_FOREGROUND_VAR, rgb(0xffffff))
        .set(&basic::RING_VAR, rgb(0x93c5fd))
}

fn main() -> kivact::Result<()> {
    App::new("Blue")
        .theme(blue_theme(ThemeMode::Light))
        .run(|| button("Save"))
}
```

A few things to know:

- The variables mirror shadcn/ui's CSS variables: `BACKGROUND`, `FOREGROUND`, `CARD`, `POPOVER`, `PRIMARY`, `SECONDARY`, `MUTED`, `ACCENT`, `DESTRUCTIVE`, `BORDER`, `INPUT`, `RING`, `CHART_1` to `CHART_5`, the `SIDEBAR_*` colors and the other colors of the kit. Each has a `_VAR` static for `set` and `get`.
- Set light and dark values separately when the app can switch: build the theme for each mode, as `blue_theme` does for the mode it is given.
- A color you change reaches the standard components too, because their recipes read the variables of the theme in effect.
- The kit sets the core color variables of `kivact::style::color` (`ACCENT`, `TEXT`, `BORDER`, `FOCUS`, `DANGER`, and so on) from its own colors, so a component from outside the kit follows the kit. To change one core color on its own, `set` its core variable after `basic::theme`, such as `.set(&color::FOCUS_VAR, rgb(0x2563eb))`.

## Adjust how one standard component is drawn

A recipe is the rule that turns a component's state into a style. To change how every button looks, install a `ButtonTheme` part in the theme. Its `root` closure receives the style the kit draws and returns the changed style, so you keep everything else the kit does (variants, sizes, hover and focus looks):

```rust,no_run
use kivact::prelude::*;
use kivact::style::{Theme, ThemeMode};
use kivact::ui::theme::ButtonTheme;
use kivact_ui_basic as basic;

fn rounded_buttons(mode: ThemeMode) -> Theme {
    basic::theme(mode).with(ButtonTheme::default().root(|_theme, _look, base| base.border_radius(8)))
}
```

For a single instance, skip the theme and pass `.style(..)`: a caller's style always wins over the kit's defaults, on standard components and on the kit's own builders alike.

## Use the icons

Every Lucide icon of `basic::ICON_NAMES` (about 125 names such as `check`, `search`, `settings`, `trash-2`) is in the theme's icon set. Draw one with the core `icon(name)`:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn toolbar() -> impl Element {
    assert!(basic::ICON_NAMES.contains(&"settings"));
    row((
        button("Download").icon(icon("download")),
        button("Settings").icon(icon("settings")).hide_label(true),
        kivact::ui::badge("New").icon(icon("sparkles")),
    ))
    .style(style().gap(8))
}
```

- An icon is 16 px and drawn in the current text color. In a smaller box, such as the icon slot of a small button, it shrinks to fit. To resize one by hand, set a size on it: `icon("info").style(style().size(24, 24))`.
- Where a component has one slot for one view (`alert(..).icon(..)`, `badge(..).icon(..)`, `button(..).icon(..)`), pass `icon(name)`. Where an item is a record (`ListItem`, `MenuEntry`), pass the name as a string: `MenuEntry::item("save", "Save").with_icon("save")`.
- An icon is decoration. An icon-only button gets its accessible name from its label, so write `button("Settings").icon(..).hide_label(true)` and keep the label.
- A name that is not in the set draws nothing. In a debug build the missing name is reported at the call site, so a typo does not go unnoticed.
- Add your own icon to the theme with `with_icon`, from SVG bytes (a stroke or fill color in the file is replaced by the text color):

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_basic as basic;

static STAR: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#000' stroke-width='2'><path d='M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1L3.2 9.5l6.1-.9z'/></svg>";

fn main() -> kivact::Result<()> {
    let theme = basic::theme(ThemeMode::Light).with_icon("star", STAR);
    App::new("Icons")
        .theme(theme)
        .run(|| button("Favorite").icon(icon("star")))
}
```

## See also

- [Overlays](overlays.md): dialogs, sheets, popovers, menus and toasts in this theme.
- [Navigation](navigation.md), [Data display](data-display.md) and [Forms](forms.md): the kit's component families.
- Every component with its source: `cargo run -p kivact-ui-basic-catalog` in a clone of the repository.
- The full API: `cargo doc -p kivact-ui-basic --open`.
