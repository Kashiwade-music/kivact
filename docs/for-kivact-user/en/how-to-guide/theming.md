# Change the theme and the look of components

[日本語](../../jp/how-to-guide/theming.md)

A `Theme` holds the colors, spacing, radii, type sizes, icons and component
recipes that every standard component draws from. Pick a theme for the app with
`App::theme`, switch it at run time with `provide_theme`, read its tokens in
your own styles, and change single parts with recipes. The official kits ship
complete themes, so you rarely start from nothing.

## Choose a theme

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;

fn main() -> kivact::Result<()> {
    // `kivact-ui-basic` is in Cargo.toml; `Theme::dark()` needs no kit.
    let theme = kivact_ui_basic::theme(ThemeMode::Dark);

    App::new("Dark app")
        .theme(theme)
        .run(|| button("Hello").style(style().margin(16)))
}
```

- `Theme::light()` is the default and `Theme::dark()` its dark counterpart.
  `theme.mode()` returns a `ThemeMode` (`Light` or `Dark`).
- Every kit exposes `theme(mode: ThemeMode)`: `kivact_ui_basic::theme`,
  `kivact_ui_pop::theme`, `kivact_ui_game::theme`, `kivact_ui_creative::theme`
  and `kivact_ui_soft::theme`. Pass the result to `App::theme`. The kit guides
  (see the [kits index](../../../for-kivact-ui-user/en/README.md)) describe each
  look and the extra components that come with it.
- Which mode to use, system or user setting, is your app's decision. Kivact does
  not choose for you.

## Switch the theme while the app runs

`App::theme` fixes the theme at start. To change it later, hold the theme in a
signal and wrap your screen in `provide_theme`:

```rust,test_harness
use kivact::prelude::*;
use kivact::style::{ThemeMode, current_theme};
use kivact::view::provide_theme;

fn app() -> impl IntoView {
    let theme = signal(Theme::light());

    provide_theme(theme, move || {
        column((
            button("Toggle dark mode").on_press(move || {
                theme.update(|t| {
                    *t = match t.mode() {
                        ThemeMode::Light => Theme::dark(),
                        ThemeMode::Dark => Theme::light(),
                    }
                })
            }),
            text(move || format!("mode: {:?}", current_theme().mode())),
        ))
        // `provide_theme` draws nothing, so give its top node the theme's colors.
        .style(
            style()
                .height(pct(100))
                .padding(16)
                .gap(8)
                .background(ColorToken::Surface)
                .color(ColorToken::Text),
        )
    })
}

fn main() -> kivact::Result<()> {
    App::new("Theme switch").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_toggle_switches_the_mode_without_remounting() {
        let screen = kivact::test::render(app);
        screen.get_by_text("mode: Light");
        screen.get_by_role(Role::Button, "Toggle dark mode").click();
        screen.get_by_text("mode: Dark");
    }
}
```

All styles under `provide_theme` re-resolve their colors when the signal changes;
nothing is rebuilt and no state is lost. The theme can also be a closure or a
`memo`. `provide_theme` works for a subtree too: a settings panel can be dark
inside a light app, and siblings keep the theme from above.

## Read tokens in your own styles

Colors given as `ColorToken`s resolve against the theme they are drawn in, so
they follow a switch for free:

```rust,no_run
use kivact::prelude::*;
use kivact::style::current_theme;

fn card(body: impl IntoView) -> impl IntoView {
    column(body).style(
        style()
            .background(ColorToken::SurfaceAlt)
            .border(1, ColorToken::Border)
            .radius(8)
            .padding(12),
    )
}

fn spaced(body: impl IntoView) -> impl IntoView {
    // Read the theme inside the closure: the style is computed again when it changes.
    column(body).style(move || {
        let theme = current_theme();
        style().padding(theme.space().lg).gap(theme.space().sm)
    })
}

fn main() -> kivact::Result<()> {
    App::new("Tokens").run(|| card(spaced(text("Hello"))))
}
```

- The color tokens: `Surface`, `SurfaceAlt`, `Text`, `TextMuted`, `Accent`,
  `AccentText`, `Border`, `Focus`, `Danger`, `Success`, `Warning`, `Overlay`,
  `Shadow`, `Selection`, `Hover`, `Pressed` and `Raised`. `rgb(0x2563eb)` and
  `rgba(..)` give fixed colors.
- Other tokens are read with accessors: `theme.space().md`, `theme.radius().lg`,
  `theme.text().size`, `theme.focus_ring()`, `theme.focus_ring_offset()`,
  `theme.disabled_opacity()` and `theme.control_height()`.
- Read `current_theme()` **inside** a style closure, not once in setup. A copy
  taken in setup is a fixed snapshot and misses theme switches.

## Change tokens

Every `with_*` builder returns a new theme and keeps the rest:

```rust,no_run
use kivact::prelude::*;
use kivact::style::rgb;
use kivact::style::theme::Palette;

fn brand() -> Theme {
    let base = Theme::light().with_focus_ring(3.0).with_focus_ring_offset(2.0);
    let colors = Palette {
        accent: rgb(0x7c3aed),
        focus: rgb(0x7c3aed),
        ..base.colors().clone()
    };
    base.with_colors(colors).with_disabled_opacity(0.4)
}

fn main() -> kivact::Result<()> {
    App::new("Brand").theme(brand()).run(|| button("Purple"))
}
```

Every standard component picks the new accent up. The same works for
`with_space`, `with_radius`, `with_text` (type sizes and the default font
family), `with_control_height` and `with_icons`. Starting from a kit's theme
instead of `Theme::light()` keeps the kit's recipes and tokens.

Install `kivact::view::OverlayViewport::new(16.0)` with `Theme::with`
to fit overlays inside a 16 px viewport margin. Anchored overlays try
flips, spans and slides before clamping. Use `Overlay::viewport_margin`
to set the margin on an individual layer.

## Change how one standard component looks

Tokens change everything at once. To change one part of one component, wrap its
**recipe**: a function from the theme and the part's state to a `Style`. The
recipe table is `UiRecipes`, installed with `Theme::with`.

```rust,no_run
use kivact::prelude::*;
use kivact::ui::UiRecipes;

fn square_buttons() -> Theme {
    let theme = Theme::light();
    // Read the recipes the theme has now (a kit's, if it is a kit's theme) ...
    let recipes = UiRecipes::clone(&theme.get::<UiRecipes>());
    // ... and wrap the button recipe: call the old one, then override one thing.
    let recipes = recipes.button(|previous| move |theme, state| previous(theme, state).radius(0));
    theme.with(recipes)
}

fn main() -> kivact::Result<()> {
    App::new("Square").theme(square_buttons()).run(|| button("Save"))
}
```

- The method named after a part (`button`, `text_input`, `checkbox`, ...)
  replaces that recipe; it receives the current one so you can build on it. Tables of
  several parts, such as the dialog, are read with `dialog_recipes()` and
  changed with `dialog(|table| table.surface(..))`.
- Build the theme **once** (at start, or in a `static`). A recipe table is
  compared by identity, so a theme rebuilt on every run restyles everything under it
  each time.
- Behavior (keys, focus, accessibility) is not in a recipe, so restyling a
  component never changes how it works.
- To change a single instance instead of a theme, add `.style(..)` to it. A
  style you set on a component comes after the recipe and wins.

Themes can draw checkbox marks in the `checkbox_check` and `checkbox_mixed`
recipes inside `checkbox_indicator`, and draw a switch thumb with
`switch_thumb` inside `switch_track`. Replace the root recipe's drawing
when drawing these parts separately. The radio group's `mark` recipe
styles the dot icon. Custom parts stay mounted as the checked state changes,
so a paint transition can animate opacity, translation or scale.

### Variants: the same component in another look

`.variant(value)` passes a value to the recipes without changing the theme.
`ButtonVariant` is the built-in one:

```rust,no_run
use kivact::prelude::*;

fn buttons() -> impl IntoView {
    row((
        button("Primary"),
        button("Secondary").variant(ButtonVariant::Secondary),
        button("Outline").variant(ButtonVariant::Outline),
        button("Ghost").variant(ButtonVariant::Ghost),
        button("Danger").variant(ButtonVariant::Danger),
        button("Link").variant(ButtonVariant::Link),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Variants").run(buttons)
}
```

A button without a variant is `Primary`. A theme decides how each variant looks;
kits often add their own variant types. Define one yourself with a type that
implements `kivact::ui::Variant` and read it in a recipe with
`look.variant::<MyVariant>()`. `button(..).size(kivact::ui::ControlSize::Small)` and
the other `size` options change the metrics the same way.

## Hover, pressed, focus and disabled looks

State looks are part of the style you return, not events you listen to:

```rust,no_run
use kivact::prelude::*;

fn tile() -> impl IntoView {
    node()
        .child(text("Open"))
        .role(Role::Button)
        .focusable(true)
        .style(
            style()
                .padding(12)
                .background(ColorToken::SurfaceAlt)
                .hover(style().background(ColorToken::Hover))
                .pressed(style().background(ColorToken::Pressed))
                .focus_visible(style().ring(2, ColorToken::Focus))
                .disabled(style().opacity(0.5)),
        )
}

fn row_with_actions() -> impl IntoView {
    // `group()` on the parent lets a child react to the parent's hover.
    row((
        text("Report.pdf"),
        spacer(),
        button("Delete").style(style().opacity(0.0).group_hover(style().opacity(1.0))),
    ))
    .style(style().group().padding(8))
}

fn main() -> kivact::Result<()> {
    App::new("States").run(|| column((tile(), row_with_actions())))
}
```

- `hover` applies while the pointer is over the node or a descendant, `pressed`
  while it is pressed, `focus` while it has focus, `focus_visible` while the focus
  should be visible (keyboard use), and `disabled` while the node is disabled.
- A focusable element without its own `focus` or `focus_visible` gets the theme's
  focus ring automatically, and a disabled one is drawn with the theme's
  `disabled_opacity`. Remove them with `style().focus_visible(style().ring_none())`
  or `style().disabled(style().opacity(1.0))`.
- `group()` and `group_hover(..)` (also `group_focus(..)`) react to an ancestor's
  state with no pointer handler in your code.
- A state style changes paint only (background, border, ring, radius, opacity,
  shadow, transform, cursor, filters, color); anything else set in it is
  reported and ignored. `disabled` and `selected` can change any property.
- You do not read hover state in a recipe and branch on it. Describe each state
  as a style and the engine applies it.

## Icons

An icon is a name that the current theme resolves to an image, so a kit can draw
its own glyph for the same name. The standard marks (`check`, `dot`,
`chevron-down`, `chevron-right`, `x`) are in every theme. Add your own with
`Theme::with_icon`, then draw it with `icon(..)`:

```rust,test_harness
use kivact::prelude::*;

static STAR: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2l3 7h7l-6 4 2 7-6-4-6 4 2-7-6-4h7z"/></svg>"#;

fn app() -> impl IntoView {
    row((
        icon("star").name("Favorite").style(style().size(24, 24).color(ColorToken::Warning)),
        text("Starred"),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    let theme = Theme::light().with_icon("star", STAR);
    App::new("Icons").theme(theme).run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_theme_icon_draws_without_errors() {
        let options = kivact::test::kivact::test::RenderOptions::from_env().theme(Theme::light().with_icon("star", STAR));
        let screen = kivact::test::render_with(options, app);
        screen.get_by_role(Role::Image, "Favorite");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
```

- `icon(name)` draws a 16 px image in the text color. A style sets its size and
  color, and `.name(..)` makes it visible to assistive technology (without a
  name, it is decorative).
- A name the theme does not know draws nothing; a debug build also reports it
  once.
- Components that take an icon (`MenuEntry::icon`, `ListItem::icon`) accept the
  same names or an image `Asset`. See [Images and fonts](images-and-fonts.md) for
  icons that are plain files.

## Add your own tokens

Share your own colors and sizes through the theme by defining a type that
implements `ThemeExtension`:

```rust,test_harness
use kivact::prelude::*;
use kivact::style::{Color, ThemeExtension, current_theme, rgb};

#[derive(Debug, PartialEq)]
struct Brand {
    highlight: Color,
    gutter: f64,
}

impl ThemeExtension for Brand {
    // What a theme without `Brand` reads, for example when your component runs
    // under another theme.
    fn fallback(_theme: &Theme) -> Self {
        Brand { highlight: rgb(0xff8800), gutter: 12.0 }
    }
}

fn banner() -> impl IntoView {
    text("New").style(move || {
        let brand = current_theme().get::<Brand>();
        style().color(brand.highlight).padding(brand.gutter)
    })
}

fn main() -> kivact::Result<()> {
    let theme = Theme::light().with(Brand { highlight: rgb(0x0088ff), gutter: 16.0 });
    App::new("Brand tokens").theme(theme).run(banner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_installed_value_wins_and_a_missing_one_falls_back() {
        let themed = Theme::light().with(Brand { highlight: rgb(0x0088ff), gutter: 16.0 });
        assert_eq!(themed.get::<Brand>().gutter, 16.0);
        assert_eq!(Theme::dark().get::<Brand>().gutter, 12.0);
    }
}
```

- The type needs `Debug + PartialEq + Send + Sync + 'static`. Two themes are equal
  when their installed values are equal, which keeps unchanged themes from
  restyling.
- `Theme::get::<Brand>()` returns an `Arc<Brand>` that shares the installed value;
  nothing is copied.
- Read tokens inside a style closure (as above) so a theme switch restyles the
  component without remounting.
- A library that ships components uses the same mechanism for its own
  recipe tables. See [Write reusable components](custom-components.md).

## See also

- [Kits](../../../for-kivact-ui-user/en/README.md): ready-made themes and
  components.
- [Layout](layout.md): sizes, spacing and alignment.
- [Images and fonts](images-and-fonts.md): custom fonts and icons from SVG.
- [Internationalization](i18n.md): locale, texts and text direction.
- Reference: `cargo doc -p kivact-style --open` for `Theme`, `Style` and
  `ColorToken`; `cargo doc -p kivact-ui --open` for `UiRecipes`.
