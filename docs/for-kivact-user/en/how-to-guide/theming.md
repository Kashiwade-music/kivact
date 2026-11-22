# Change the theme and the look of components

[日本語](../../jp/how-to-guide/theming.md)

A `Theme` holds the colors, spacing, radii, type sizes, icons and component
recipes that every standard component draws from. Pick a theme for the app with
`App::theme`, switch it at run time with `provide_theme`, read its theme
variables in your own styles, and change single parts with recipes. The official
kits ship complete themes, so you rarely start from nothing.

## Choose a theme

```rust,no_run
use kivact::prelude::*;

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
            text(move || format!("mode: {:?}", use_theme().get().mode())),
        ))
        // `provide_theme` draws nothing, so give its top node the theme's colors.
        .style(
            style()
                .height(pct(100))
                .padding(16)
                .gap(8)
                .background(color::SURFACE)
                .color(color::TEXT),
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

## Use theme variables in your styles

Theme variables resolve against the theme they are drawn in, so they follow a
switch for free:

```rust,no_run
use kivact::prelude::*;

#[component]
fn card(body: View) -> impl Element {
    column(body).style(
        style()
            .background(color::SURFACE_ALT)
            .border(1, color::BORDER)
            .border_radius(radius::MD)
            .padding(space::MD),
    )
}

#[component]
fn spaced(body: View) -> impl Element {
    // A theme variable needs no closure: the style follows a theme switch.
    column(body).style(style().padding(space::LG).gap(space::SM).width(pct(100) - space::XL))
}

fn main() -> kivact::Result<()> {
    App::new("Theme variables").run(|| card(spaced(text("Hello"))))
}
```

- The color variables in `color`: `SURFACE`, `SURFACE_ALT`, `TEXT`, `TEXT_MUTED`,
  `ACCENT`, `ACCENT_TEXT`, `BORDER`, `FOCUS`, `DANGER`, `SUCCESS`, `WARNING`,
  `OVERLAY`, `SHADOW`, `SELECTION`, `HOVER`, `PRESSED` and `RAISED`. `space::XS`
  to `space::XL`, `radius::SM` to `radius::LG` and `font::SIZE`, `font::SIZE_SMALL`
  and `font::SIZE_LARGE` are lengths. `rgb(0x2563eb)` and `rgba(..)` give fixed
  colors. Lengths combine with `+` and `-`, like CSS `calc()`:
  `pct(100) - space::XL` mixes a percentage and a theme variable and is computed
  again when the theme changes.
- Other theme values: `theme.focus_ring()` and `theme.focus_ring_offset()` are
  read with accessors; the control height, the disabled opacity and the overlay
  margin are the variables `control::HEIGHT_VAR`, `control::DISABLED_OPACITY_VAR`
  and `control::OVERLAY_MARGIN_VAR` (in `kivact::style::control`), read with
  `theme.get(..)`. `theme.get(&color::ACCENT_VAR)` reads the value of a variable,
  for example to draw on a canvas.
- `use_theme()` returns the theme in effect; read it inside a style closure when
  a style depends on more than variables.

## Set theme variables

`set(&VAR, value)` returns a new theme with one variable changed and keeps the
rest; the value may be another variable, as `FOCUS` follows `ACCENT` here:

```rust,no_run
use kivact::prelude::*;
use kivact::style::control;

fn brand() -> Theme {
    Theme::light()
        .with_focus_ring(3.0)
        .with_focus_ring_offset(2.0)
        .set(&color::ACCENT_VAR, rgb(0x7c3aed))
        .set(&color::FOCUS_VAR, &color::ACCENT_VAR)
        .set(&control::DISABLED_OPACITY_VAR, 0.4)
}

fn main() -> kivact::Result<()> {
    App::new("Brand").theme(brand()).run(|| button("Purple"))
}
```

Every kind of theme variable can take another variable as its value, sizes
included: a component's own variables, such as the button height, default to the
core variables, so changing a core value reaches every component that follows it.
A loop of variables is reported once and falls back to the fallback value.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeVar;

/// The gap of a toolbar: the theme's medium spacing until a theme sets it.
pub static TOOLBAR_GAP_VAR: ThemeVar<f64> = ThemeVar::alias("toolbar.gap", &space::MD_VAR);

fn roomy() -> Theme {
    Theme::light().set(&TOOLBAR_GAP_VAR, &space::LG_VAR)
}
# fn main() { let _ = roomy(); }
```

An alias is written one way: `ThemeVar::alias(name, &OTHER_VAR)` for the default,
and `theme.set(&VAR, &OTHER_VAR)` in a theme.

Every standard component picks the new accent up. The same works for the `space`,
`radius` and `font` variables (`space::MD_VAR` and so on). `with_icons` sets the
icons; the control height, the disabled opacity and the overlay margin are
`control::HEIGHT_VAR`, `control::DISABLED_OPACITY_VAR` and
`control::OVERLAY_MARGIN_VAR`, set the same way. Starting from a kit's theme
instead of `Theme::light()` keeps the kit's recipes and tokens.

Use `theme.set(&control::OVERLAY_MARGIN_VAR, 16.0)` to fit overlays inside a
16 px viewport margin. The default is zero inset with viewport fitting enabled.
Anchored overlays try flips, spans and slides before clamping. Use
`Overlay::viewport_margin` to set the margin on an individual layer.

## Change how one standard component looks

Theme variables change everything at once. To change one part of one component,
wrap its **recipe**: a function from the theme, the part's look and the style
made so far to a `Style`. Each standard component has a table in
`kivact::ui::theme`, such as `ButtonTheme`, installed with `Theme::with`.

```rust,no_run
use kivact::prelude::*;
use kivact::ui::theme::ButtonTheme;

fn square_buttons() -> Theme {
    // `base` is the style the theme's recipe made; return it changed.
    Theme::light().with(ButtonTheme::default().root(|_theme, _look, base| base.border_radius(0)))
}

fn main() -> kivact::Result<()> {
    App::new("Square").theme(square_buttons()).run(|| button("Save"))
}
```

- A method named after a part (`root` and the other parts of the table) adds your
  function after the theme's recipe for that part: `base` is the style the theme
  made, and `look` holds the part's state and variant.
- Build the theme **once**, at start, and keep it: pass it to `App::theme` or
  keep it in a signal for `provide_theme`. A recipe table is compared by identity,
  so a theme rebuilt on every run restyles everything under it each time.
- To change a single instance instead of a theme, add `.style(..)` to it. A
  style you set on a component comes after the recipe and wins.

Themes can draw checkbox marks in the `check` and `mixed` parts of
`CheckboxTheme`, which sit inside `indicator`, and draw a switch thumb with the
`thumb` part of `SwitchTheme`, inside `track`. Replace the root recipe's drawing
when drawing these parts separately. The radio group's `mark` part (in
`RadioGroupTheme`) styles the dot icon. Custom parts stay mounted as the checked
state changes, so a paint transition can animate opacity, translation or scale.

### Variants: the same component in another look

`.variant(value)` passes a value to the recipes without changing the theme.
`ButtonVariant` is the built-in one:

```rust,no_run
use kivact::prelude::*;

#[component]
fn buttons() -> impl Element {
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
implements `kivact::ui::theme::Variant` and read it in a recipe with
`look.variant::<MyVariant>()`. `button(..).size(kivact::ui::ControlSize::Small)` and
the other `size` options change the metrics the same way.

## Hover, active, focus and disabled looks

State looks are part of the style you return, not events you listen to:

```rust,no_run
use kivact::prelude::*;

#[component]
fn tile() -> impl Element {
    node()
        .child(text("Open"))
        .role(Role::Button)
        .focusable(true)
        .style(
            style()
                .padding(12)
                .background(color::SURFACE_ALT)
                .hover(style().background(color::HOVER))
                .active(style().background(color::PRESSED))
                .focus_visible(style().outline(2, color::FOCUS))
                .disabled(style().opacity(0.5)),
        )
}

#[component]
fn row_with_actions() -> impl Element {
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

- `hover` applies while the pointer is over the node or a descendant, `active`
  while it is pressed, `focus` while it has focus, `focus_visible` while the focus
  should be visible (keyboard use), and `disabled` while the node is disabled.
- A focusable element without its own `focus` or `focus_visible` gets the theme's
  focus ring automatically, and a disabled one is drawn with the theme's
  `disabled_opacity`. Remove them with `style().focus_visible(style().outline_none())`
  or `style().disabled(style().opacity(1.0))`.
- `group()` and `group_hover(..)` (also `group_focus(..)`) react to an ancestor's
  state with no pointer handler in your code.
- A state style can set any property, including layout ones such as padding or
  width; the node is laid out again when the state changes.
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
        icon("star").name("Favorite").style(style().size(24, 24).color(color::WARNING)),
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
        let options = kivact::test::RenderOptions::from_env().theme(Theme::light().with_icon("star", STAR));
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

## Choose colors in other color spaces

A color can be written in the CSS color spaces, not only as hex or `rgb`. The
functions `oklch`, `oklab`, `lab`, `lch`, `hwb`, `hsl` and `rgba` are in
`kivact::style`, and each takes the numbers its space uses. `oklch(l, c, h)` is
the one to reach for when a palette needs steps that look equally bright: `l`
runs from 0 to 1, `c` (chroma) starts at 0, and `h` is the hue in degrees. In
`lab` and `lch`, `l` runs from 0 to 100. `hwb` takes the hue, then whiteness and
blackness from 0 to 1.

`Color::css(text)` parses a color written as CSS, such as `oklch(70% 0.15 250)`,
`hsl(210 60% 40%)` or `#2563eb`. It returns an error for text it does not
understand, so check it once in setup, when the text comes from a file or a
setting.

```rust,no_run
use kivact::prelude::*;
use kivact::style::{hwb, oklch};

#[component]
fn swatches(brand_text: String) -> impl Element {
    // A color from a setting: a value that does not parse falls back to the default.
    let brand = Color::css(&brand_text).unwrap_or(oklch(0.62, 0.19, 255.0));
    row((
        text("Brand").style(
            style()
                .padding(8)
                .background(brand)
                .color(oklch(0.98, 0.0, 0.0)),
        ),
        text("Ready").style(
            style()
                .padding(8)
                .background(hwb(150.0, 0.12, 0.25))
                .color(oklch(0.98, 0.0, 0.0)),
        ),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Swatches").run(|| swatches("oklch(70% 0.15 250)".to_string())) }
```

Colors from these spaces work everywhere a color is accepted: a style, a border,
a shadow, a gradient stop or a canvas drawing. A color that the screen cannot show
is mapped into the screen's range when it is drawn.

## Blend gradients and place their hints

A gradient mixes its stops in Oklab by default. `.interpolate(space)` mixes them
in another space, the way CSS `linear-gradient(in oklch, ..)` does, and for the
polar spaces (`Oklch`, `Lch`, `Hsl`, `Hwb`) `.hue(..)` chooses the way round the
hue wheel: `HueInterpolation::Shorter` (the default), `Longer`, `Increasing`
or `Decreasing`.

`.hint(at)` moves the midpoint between a stop and the next one. Call it right
after the stop: where the color is halfway between the two is then at `at`, not
at the middle of the two stops, as CSS color hints do.

```rust,no_run
use kivact::prelude::*;
use kivact::style::{ColorSpace, HueInterpolation, linear_gradient, oklch};

#[component]
fn dusk() -> impl Element {
    // Sweep through the hues in oklch, the long way round, with the
    // midpoint of the first pair moved towards the warm end.
    let sky = linear_gradient(180)
        .interpolate(ColorSpace::Oklch)
        .hue(HueInterpolation::Longer)
        .stop(oklch(0.45, 0.16, 280.0), pct(0))
        .hint(pct(70))
        .stop(oklch(0.78, 0.17, 45.0), pct(100));

    text("Dusk")
        .style(style().padding(24).height(120).background(sky).color(oklch(0.98, 0.0, 0.0)))
}
# fn main() -> kivact::Result<()> { App::new("Dusk").run(dusk) }
```

## Add your own theme variables

Share your own colors and sizes through the theme with your own theme variables:

```rust,test_harness
use kivact::prelude::*;
use kivact::style::{Length, ThemeVar};

static HIGHLIGHT_VAR: ThemeVar<Color> = ThemeVar::new("brand.highlight", rgb(0xff8800));
static GUTTER_VAR: ThemeVar<f64> = ThemeVar::new("brand.gutter", 12.0);
const HIGHLIGHT: Color = Color::var(&HIGHLIGHT_VAR);
const GUTTER: Length = Length::var(&GUTTER_VAR);

#[component]
fn banner() -> impl Element {
    text("New").style(style().color(HIGHLIGHT).padding(GUTTER))
}

fn main() -> kivact::Result<()> {
    let theme = Theme::light().set(&HIGHLIGHT_VAR, rgb(0x0088ff)).set(&GUTTER_VAR, 16.0);
    App::new("Brand variables").theme(theme).run(banner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_set_value_wins_and_an_unset_one_falls_back() {
        let themed = Theme::light().set(&GUTTER_VAR, 16.0);
        assert_eq!(themed.get(&GUTTER_VAR), 16.0);
        assert_eq!(Theme::dark().get(&GUTTER_VAR), 12.0);
    }
}
```

- A variable is a `static` `ThemeVar` (a name and a fallback) and a `const` `Color`
  or `Length` that points to it. Use the `const` anywhere a value goes.
- `theme.set(&VAR, value)` gives it a value in one theme; a theme that never sets
  it uses the fallback, which may be another variable.
- Tables of functions, such as recipes, stay `ThemeExtension`s installed with
  `Theme::with`. See [Write reusable components](custom-components.md).

## See also

- [Kits](../../../for-kivact-ui-user/en/README.md): ready-made themes and
  components.
- [Layout](layout.md): sizes, spacing and alignment.
- [Images and fonts](images-and-fonts.md): custom fonts and icons from SVG.
- [Internationalization](i18n.md): locale, texts and text direction.
- Reference: `cargo doc -p kivact-style --open` for `Theme`, `Style` and
  `ThemeVar`; `cargo doc -p kivact-ui --open` for `kivact::ui::theme`.
