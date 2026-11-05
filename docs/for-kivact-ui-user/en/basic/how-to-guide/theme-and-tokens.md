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
                text_input("Name", signal(String::from("Ada"))),
            ))
            .style(
                style()
                    .size(pct(100), pct(100))
                    .padding(24)
                    .gap(12)
                    .background(basic::ColorName::Background)
                    .color(basic::ColorName::Foreground),
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

The kit does not ship fonts. It is sized for Inter and asks for `Geist Mono` in code blocks and inline code. Register the files with `App::font` and name the text family in the theme:

```rust,no_run
use kivact::prelude::*;
use kivact::style::theme::Typography;
use kivact::style::{Theme, ThemeMode};
use kivact_ui_basic as basic;

fn theme(mode: ThemeMode) -> Theme {
    let theme = basic::theme(mode);
    let text = Typography {
        // Add a CJK family after Inter when the app shows Japanese, Chinese or Korean.
        family: Some("Inter, 'Noto Sans JP', sans-serif".into()),
        ..theme.text().clone()
    };
    theme.with_text(text)
}

fn app(fonts: &[&'static [u8]]) -> App {
    let mut app = App::new("Basic").theme(theme(ThemeMode::Light));
    for &font in fonts {
        app = app.font(font);
    }
    app
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

`basic::ColorName` names a color of the palette (`Background`, `Foreground`, `Card`, `Primary`, `Secondary`, `Muted`, `MutedForeground`, `Accent`, `Destructive`, `Border`, `Input`, `Ring`, `Chart1` to `Chart5`, the `Sidebar` colors and the `Code` syntax colors). Every style method that takes a color accepts it, and it is resolved from the theme in effect when the node is drawn, so it follows a theme switch:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn banner() -> impl IntoView {
    text("Heads up").style(
        style()
            .padding(12)
            .background(basic::ColorName::Muted)
            .color(basic::ColorName::MutedForeground)
            .border(1, basic::ColorName::Border),
    )
}
```

To compute a new color from a token (a translucent primary, say), read `basic::Tokens` inside a style closure. The closure runs again when the theme changes:

```rust,no_run
use kivact::prelude::*;
use kivact::style::use_theme;
use kivact_ui_basic as basic;

fn tinted() -> impl IntoView {
    text("Tinted").style(|| {
        let tokens = use_theme().get().ext::<basic::Tokens>();
        style().padding(12).background(tokens.primary.with_alpha(0.1))
    })
}
```

## Change the colors

`basic::Tokens::new(mode)` returns the palette as plain fields. Change the ones you want and install the value with `Theme::with`:

```rust,no_run
use kivact::prelude::*;
use kivact::style::{Theme, ThemeMode, rgb};
use kivact_ui_basic as basic;

fn blue_theme(mode: ThemeMode) -> Theme {
    let mut tokens = basic::Tokens::new(mode);
    tokens.primary = rgb(0x2563eb);
    tokens.primary_foreground = rgb(0xffffff);
    tokens.ring = rgb(0x93c5fd);
    basic::theme(mode).with(tokens)
}

fn main() -> kivact::Result<()> {
    App::new("Blue")
        .theme(blue_theme(ThemeMode::Light))
        .run(|| button("Save"))
}
```

Tokens exist for each mode, so build a customized palette for both when the app can switch. A few things to know:

- The fields mirror shadcn/ui's CSS variables: `background`, `foreground`, `card`, `popover`, `primary`, `secondary`, `muted`, `accent`, `destructive`, `border`, `input`, `ring`, `chart` (five colors), the `sidebar_*` colors and `code` (the syntax colors of code blocks).
- A color you change reaches the standard components too, because their recipes read the tokens of the theme in effect.
- `basic::theme` also copies a few tokens into the core palette (page, text, accent, border, focus, danger). A component from outside the kit, such as one from another library, reads that palette. If you need them to agree with a changed token, also pass the palette to `Theme::with_colors`.

## Adjust how one standard component is drawn

A recipe is the function that turns a component's state into a style. To change how all buttons look, take the kit's `UiRecipes` out of the theme, replace the recipe with a function that calls the current one and adds to it, and install the table again:

```rust,no_run
use kivact::prelude::*;
use kivact::style::{Theme, ThemeMode};
use kivact::ui::{ButtonState, Look, UiRecipes};
use kivact_ui_basic as basic;

fn rounded_buttons(mode: ThemeMode) -> Theme {
    let theme = basic::theme(mode);
    let recipes = (*theme.ext::<UiRecipes>()).clone().button(|current| {
        move |theme: &Theme, look: &Look<ButtonState>| current(theme, look).radius(8)
    });
    theme.with(recipes)
}
```

The replacement receives the current recipe, so you keep everything else the kit does (variants, sizes, hover and focus looks). For a single instance, skip recipes and pass `.style(..)`: a caller's style always wins over the kit's defaults, on standard components and on the kit's own builders alike.

## Use the icons

Every Lucide icon of `basic::ICON_NAMES` (about 125 names such as `check`, `search`, `settings`, `trash-2`) is in the theme's icon set. Draw one with the core `icon(name)`:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn toolbar() -> impl IntoView {
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
- Where a component has one slot for one view (`alert(..).with_icon(..)`, `badge(..).with_icon(..)`, `button(..).with_icon(..)`), pass `icon(name)`. Where an item is a record (`ListItem`, `MenuEntry`), pass the name as a string: `MenuEntry::item("save", "Save").with_icon("save")`.
- An icon is decoration. An icon-only button gets its accessible name from its label, so write `button("Settings").with_icon(..).hide_label(true)` and keep the label.
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
