# Write reusable components

[日本語](../../jp/how-to-guide/custom-components.md)

Use this guide to turn a piece of UI you repeat into a component that other code
can reuse: a chip, a card, a notice with an action, a search box. It covers how to
take props, children and callbacks, how to ship a default look the caller can still
change, how to put your own colors on the theme and how to keep accessibility
intact when a component wraps another control. It assumes you have read the
getting-started tutorial.

A component is a function marked `#[component]`. It is the only way to write
one: your app, the standard components of `kivact::ui` and the official kits all
use it. Kivact writes the props builder from the signature, so you never write a
builder by hand.

| Return type | Use it when |
|---|---|
| `#[component] fn card(..) -> impl Element` | The usual case: the result has one root, and callers may add `.style(..)`, `.name(..)`, `.test_id(..)` and handlers to it. |
| `#[component] fn section(..) -> impl IntoView` | The result has no single root, such as a `show(..)` branch or a tuple. Callers cannot add modifiers. |

## Start with a component

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, Copy, PartialEq, Prop)]
pub enum ChipTone {
    Normal,
    Accent,
}

/// A reusable label with a reactive tone and an action.
#[component]
pub fn chip(
    label: Value<String>,
    on_press: Callback<()>,
    #[prop(default = ChipTone::Normal)] tone: Value<ChipTone>,
) -> impl Element {
    button(label).on_press(on_press).style(move || {
        style()
            .border_radius(12)
            .padding_x(10)
            .padding_y(4)
            .background(if tone.get() == ChipTone::Accent {
                color::ACCENT
            } else {
                color::SURFACE_ALT
            })
    })
}

fn app() -> impl IntoView {
    let highlighted = signal(false);
    chip("Kivact", move || highlighted.update(|value| *value = !*value))
        .tone(move || if highlighted.get() { ChipTone::Accent } else { ChipTone::Normal })
        .style(style().width(120))
}

fn main() -> kivact::Result<()> {
    App::new("Chips").run(app)
}
```

How the signature becomes the API:

- **A required prop is a positional argument**, in the order of the signature.
  `chip("Kivact", ..)` takes the label and the press handler by position.
- **An `Option<..>` prop and a `#[prop(default = expr)]` prop become setters.**
  The chip above can be written `chip("Kivact", ..)` with the tone left out, or
  `chip("Kivact", ..).tone(ChipTone::Accent)`. An optional callback is
  `on_close: Option<Callback<()>>`: the caller writes `.on_close(..)`, and the body
  receives an `Option<Callback<()>>`.
- **The body runs once, in its own setup owner.** Signals, effects, callbacks and
  cleanup that it creates belong to this instance and are disposed when the view
  is removed. The component adds no layout box; the body's root is the
  component's root.
- **The caller can change the root.** Because `chip` returns `impl Element`, the
  caller's `.style(..)` applies to the button inside. The caller's style comes
  after the body's own style and wins where both set the same property. The
  caller's handlers run before the body's, and the caller's `name`,
  `description`, `disabled`, `test_id`, `node_ref` and `variant` replace the
  body's.
- **Callbacks take closures.** A `Callback<()>` prop accepts a closure
  (`move || ..`) or a `Callback<()>` the caller already has. A `Callback<E>` prop
  with an input accepts a closure (`move |event| ..`), whose argument type Rust
  infers from `E`; forward a parent's `on_pick: Callback<i32>` as
  `picker(move |n| on_pick.call(n))`.

`Value<T>` takes a constant, a signal, a `Value` or a closure, so a prop can be
static or reactive without a second API. A `Value<String>` prop also accepts
`&str` and `String`. A value prop's type must be `Clone + PartialEq`, which is
what lets Kivact skip updates when the value did not change.

The macro recognizes the prop kinds by their written names: `Value<T>`,
`Signal<T>`, `Callback<E>`, `View`, `ViewFn` and `Vec<View>`. A type alias of one
of them is a plain prop; write the name itself.

## Use your own type as a prop

If a prop has a type of your own, derive `Prop` on it so a constant can be passed
directly:

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, Copy, PartialEq, Prop)]
pub enum Tone {
    Info,
    Warning,
}

/// A label with a tone that follows a signal.
#[component]
pub fn badge(label: Value<String>, tone: Value<Tone>) -> impl Element {
    node()
        .name(label)
        .style(move || match tone.get() {
            Tone::Info => style().background(color::ACCENT).color(color::ACCENT_TEXT),
            Tone::Warning => style().background(color::WARNING).color(color::TEXT),
        })
        .child(text(label))
}

fn app() -> impl IntoView {
    let online = signal(true);
    row((
        badge("Saved", Tone::Info),
        badge("Offline", move || if online.get() { Tone::Info } else { Tone::Warning }),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Badges").run(app)
}
```

Without `Prop`, `badge("Saved", Tone::Info)` fails to compile. Signals, `Value`s
and closures of `Tone` work either way.

## Take children and slots

A component that wraps content takes it as `View`. Several children are a tuple,
because a tuple of views is a view, and `()` means "nothing here", so it doubles
as an empty slot. A component with named slots takes one parameter per slot; an
`Option<View>` slot is optional.

```rust,no_run
use kivact::prelude::*;

/// A titled box. `body` is whatever the caller wants inside, `footer` may be `()`.
#[component]
pub fn card(title: Value<String>, body: View, footer: View) -> impl Element {
    column((
        text(title).style(style().padding_bottom(4)),
        body,
        footer,
    ))
    .style(
        style()
            .padding(space::MD)
            .gap(8)
            .border(1, color::BORDER)
            .border_radius(radius::MD),
    )
}

fn app() -> impl IntoView {
    column((
        card("Profile", (text("Ann"), text("Admin")), ()),
        card("Danger zone", text("Deleting cannot be undone"), button("Delete")),
    ))
    .style(style().gap(12).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Cards").run(app)
}
```

A `View` slot is consumed once, when the component is used. If your component
mounts its content more than once, for example each time a section opens, ask for
`ViewFn`: the caller passes a function, and the content is built again on every
opening.

```rust,no_run
use kivact::prelude::*;

/// A section that builds its content when it opens and drops it when it closes.
#[component]
pub fn collapsible(title: Value<String>, open: Signal<bool>, content: ViewFn) -> impl Element {
    column((
        button(title).on_press(move || open.update(|open| *open = !*open)),
        show(open, content),
    ))
}

fn app() -> impl IntoView {
    let open = signal(false);
    collapsible("Details", open, || text("Created by Ann"))
}

fn main() -> kivact::Result<()> {
    App::new("Collapsible").run(app)
}
```

A `Signal<bool>` prop is state that the component both reads and writes; the
caller hands over its own signal, or `bind!(form.details_open)` for one field of
a struct.

When each child needs its own wrapper, for example every pane of a split view,
take `Vec<View>`. The caller passes a tuple of up to 16 views or a `Vec` of one
kind of view, and the body receives each child as a `View` to wrap.

## Take callbacks

```rust,no_run
use kivact::prelude::*;

/// A chip with a remove button. Delete removes it too.
#[component]
pub fn removable_chip(label: Value<String>, on_remove: Callback<()>) -> impl Element {
    // `on_remove` is a Copy handle: the button and the key handler both use it.
    row((text(label), button("Remove").on_press(on_remove)))
        .name(label)
        .focusable(true)
        .on_key(move |event: KeyEvent| {
            if event.key() == Key::Delete {
                on_remove.call(());
                Handled::Yes
            } else {
                Handled::No
            }
        })
}

fn app() -> impl IntoView {
    let removed = signal(0);
    column((
        removable_chip("Rust", move || removed.update(|n| *n += 1)),
        text(move || format!("removed: {}", removed.get())),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Remove").run(app)
}
```

The body receives a `Copy` `Callback`: give it to any `on_*`, or call it with
`.call(argument)`; for a callback without input, the argument is `()`.

A press handler gets no event. To see whether a modifier key is held, as
`event.getModifierState("Control")` does in React, read `modifiers()`:

```rust,no_run
use kivact::prelude::*;
use kivact::view::modifiers;

/// Opens its panel; Ctrl+click (Cmd+click) opens only this one.
#[component]
pub fn open_button(open: Callback<bool>) -> impl Element {
    button("Open").on_press(move || {
        let held = modifiers().get();
        open.call(held.ctrl || held.meta)
    })
}

fn main() -> kivact::Result<()> {
    App::new("Open").run(|| open_button(|only| println!("only this panel: {only}")))
}
```

The caller's closure for a `Callback<E>` prop gets its argument type from `E`, so
`|only| ..` needs no annotation. If the caller's closure may fail, it may return
`Result<(), UiError>` instead of `()`; a failure is reported to the nearest
`error_boundary` (see [Find out why something failed or updated](diagnostics.md)).

## Ship a default look

A component owns its default style, and the caller can still change it:

- **Style the root in the body.** A style closure that reads a signal or a
  `Value` follows changes without a remount.
- **Use theme variables, not fixed colors.** `color::ACCENT`, `space::MD` and
  `radius::MD` follow the active theme, with no closure. A hard-coded color cannot
  follow a theme.
- **Expose meaning as props.** A tone, a size or a variant prop says what may
  change, and the component keeps the rest consistent. The caller's `.style(..)`
  is for the rest: it comes after the body's style.

## Put your own values on the theme

Hard-coded colors in a component cannot follow a theme. Declare a theme variable
for each value your component library needs. Whoever builds the theme can set it;
a theme that does not know it uses the fallback, which can be a core variable, so
your component still matches any theme.

```rust,test_harness
use kivact::prelude::*;
use kivact::style::ThemeVar;
use kivact::test::{RenderOptions, render_with};

/// The colors of `status_dot`. Without a value in the theme, they follow the
/// core success and muted colors.
pub static ONLINE_VAR: ThemeVar<Color> = ThemeVar::alias("status.online", &color::SUCCESS_VAR);
pub static OFFLINE_VAR: ThemeVar<Color> = ThemeVar::alias("status.offline", &color::TEXT_MUTED_VAR);
pub const ONLINE: Color = Color::var(&ONLINE_VAR);
pub const OFFLINE: Color = Color::var(&OFFLINE_VAR);

#[component]
pub fn status_dot(online: Value<bool>) -> impl Element {
    node()
        .role(Role::Status)
        // `Value` reads are tracked, so the name follows the signal.
        .name(move || if online.get() { "online" } else { "offline" })
        .style(move || {
            style()
                .size(12, 12)
                .border_radius(6)
                .background(if online.get() { ONLINE } else { OFFLINE })
        })
}

fn main() -> kivact::Result<()> {
    let theme = Theme::light().set(&ONLINE_VAR, rgb(0x00aa00)).set(&OFFLINE_VAR, rgb(0x888888));
    App::new("Status").theme(theme).run(|| status_dot(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot_color(theme: Theme) -> u32 {
        let screen = render_with(RenderOptions::with_theme(theme), || {
            row(status_dot(true)).style(style().padding(8))
        });
        let dot = screen.get_by_role(Role::Status, "online").rect().expect("laid out");
        let shot = screen.screenshot();
        shot.pixel((dot.x + 6.0) as u32, (dot.y + 6.0) as u32).expect("inside the window")
    }

    #[test]
    fn the_theme_sets_the_color_and_a_plain_theme_falls_back() {
        let themed = Theme::light().set(&ONLINE_VAR, rgb(0x00aa00));
        assert_eq!(dot_color(themed), 0x00aa00);
        assert_ne!(dot_color(Theme::light()), 0x00aa00);
    }
}
```

- A theme variable is a `static` `ThemeVar` (a name and a fallback) and a
  `const` `Color` (or `Length` with `Length::var`) that points to it. Use the
  `const` anywhere a value goes; when the theme changes, every style that uses it
  follows, like `var(--x)` in CSS.
- `theme.set(&VAR, value)` gives the variable a value in that theme. The value
  may be another variable: `theme.set(&ONLINE_VAR, &color::ACCENT_VAR)`.
- A variable that follows another one until a theme sets it is declared with
  `ThemeVar::alias(name, &OTHER_VAR)`, as both variables above are. This is the
  one way to write an alias, for every kind of variable.
- The name identifies the variable in diagnostics and DevTools; prefix it with
  the name of your library.
- `ThemeExtension` is for tables that are not values, such as the recipe tables
  of the standard components ([Theming](theming.md)).

To switch themes while the app runs, wrap the root in `provide_theme` with a
closure or a signal ([Switch the theme while the app runs](theming.md#switch-the-theme-while-the-app-runs)).

## Draw icons through the theme

Do not embed an image in a component. Name the icon, and let the theme decide what
the name looks like. A kit can then give your component its own glyphs, and an app
can replace any of them. For an ordinary icon button, use the standard
`icon_button(icon, label)`; write your own only for a different shape.

```rust,test_harness
use kivact::ErrorKind;
use kivact::prelude::*;
use kivact::style::Icon;
use kivact::test::{RenderOptions, render_with};

/// A square tool button that shows the icon named `name`.
#[component]
pub fn tool_button(name: Value<Icon>, label: Value<String>) -> impl Element {
    node()
        .role(Role::Button)
        .focusable(true)
        .name(label)
        .style(style().size(32, 32))
        .child(icon(name))
}

const SEARCH: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'>\
<circle cx='10' cy='10' r='6' fill='none' stroke='black' stroke-width='2'/>\
<path d='M15 15l6 6' stroke='black' stroke-width='2'/></svg>";

fn main() -> kivact::Result<()> {
    // The app (or a kit's theme) supplies the image for the name.
    let theme = Theme::light().with_icon("search", SEARCH);
    App::new("Icons").theme(theme).run(|| tool_button("search", "Search"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_theme_resolves_the_name() {
        let theme = Theme::light().with_icon("search", SEARCH);
        let screen = render_with(RenderOptions::with_theme(theme), || tool_button("search", "Search"));
        screen.get_by_role(Role::Button, "Search");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }

    #[test]
    fn an_unknown_name_is_reported_in_debug_builds() {
        let screen = render_with(RenderOptions::default(), || tool_button("nope", "Nope"));
        screen.run_until_idle();
        if cfg!(debug_assertions) {
            let errors = screen.errors();
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].kind(), &ErrorKind::Asset);
        }
    }
}
```

`Icon::named("search")` and a bare `"search"` are the same thing, so a
`Value<Icon>` prop accepts a name, an `Asset` (an image used as is) or a signal of
either. The standard set already has the marks the standard components draw, such
as `check` and `chevron-right`; add yours with `Theme::with_icon(name, image)`.
An unknown name draws nothing and, in debug builds, is reported once.

## Put the accessible name on the real control

A composite component, such as a search box made of an icon and a text input, has a
root that is not the control the user types into. The name and the disabled state
belong to that control: if they stay on the wrapper, a screen reader announces an
unnamed text box. `field(label, control)` shows a label and gives it to the
control as its name:

```rust,test_harness
use kivact::prelude::*;

/// A search box: an icon and a labelled text field that carries the caller's state.
#[component]
pub fn site_search(
    value: Signal<String>,
    #[prop(default = "Site search")] label: Value<String>,
    #[prop(default = false)] disabled: Value<bool>,
) -> impl Element {
    row((
        icon("search"),
        field(label, text_input(value).disabled(disabled)).style(style().flex_grow(1)),
    ))
    .style(style().gap(6).align_items(Align::Center))
}

fn app() -> impl IntoView {
    let query = signal(String::new());
    site_search(query)
}

fn main() -> kivact::Result<()> {
    App::new("Search").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_box_carries_the_name() {
        let screen = kivact::test::render(app);
        let input = screen.get_by_role(Role::TextBox, "Site search");
        input.type_text("kivact");
        assert_eq!(input.value().as_deref(), Some("kivact"));
    }

    #[test]
    fn pressing_the_label_focuses_the_text_box() {
        let screen = kivact::test::render(app);
        screen.get_by_text("Site search").click();
        assert!(screen.get_by_role(Role::TextBox, "Site search").is_focused());
    }
}
```

`field` accepts any element, including your own components that return
`impl Element`. A control that can take focus and has no name is reported when it
mounts; in debug builds and in `kivact::test` the report is an error.

Focus needs no forwarding: pressing a field's label activates the control
inside it, as the second test shows.

## Read what the caller set

A component of a library sometimes depends on what its caller wrote after the
call: a tag looks pressable only when the caller gave it `.on_press(..)`. Add an
argument marked `#[caller]`: Kivact passes a snapshot of the modifiers the
caller set on this component (its variant, disabled state, description and
whether it has a press handler). The caller does not pass it.

```rust,no_run
use kivact::custom::Caller;
use kivact::prelude::*;

/// A tag that looks pressable only when the caller gave it `on_press`.
#[component]
pub fn tag(label: Value<String>, #[caller] caller: Caller) -> impl Element {
    let pressable = caller.has_press();
    text(label).style(
        style()
            .padding_x(8)
            .border_radius(radius::SM)
            .background(color::SURFACE_ALT)
            .cursor(if pressable { Cursor::Pointer } else { Cursor::Default }),
    )
}

fn app() -> impl IntoView {
    row((tag("Draft"), tag("Open").on_press(|| println!("open")))).style(style().gap(8))
}

fn main() -> kivact::Result<()> {
    App::new("Tags").run(app)
}
```

The snapshot is taken when the component mounts. Read it in the body, as above;
apps rarely need it.

## Report misuse of your component

When a component gets an argument it cannot honor, fail where the developer can
see the cause. `kivact::custom::raise` panics with a located message, exactly
like Kivact's own misuse; it ends the App or fails the test:

```rust,no_run
use kivact::ErrorKind;
use kivact::custom::raise;
use kivact::prelude::*;

#[component]
pub fn progress_ring(done: u32, total: u32) -> impl Element {
    if total == 0 {
        raise(
            ErrorKind::Message,
            "progress_ring: `total` must be greater than zero",
        );
    }
    text(format!("{done} / {total}"))
}

fn main() -> kivact::Result<()> {
    App::new("Ring").run(|| progress_ring(3, 10))
}
```

Use it for programmer errors only. A problem the user can cause, such as a bad file,
is a `Result` that your component shows, not a misuse.

## What `#[component]` rejects

The macro reports a signature it cannot turn into a builder, with a message that
says what to change:

- a return type other than `impl Element` or `impl IntoView`;
- lifetime or const parameters (type parameters are accepted);
- a method: a `#[component]` is a free function;
- a destructuring pattern: each prop is a named argument;
- an unknown key in `#[prop(..)]`;
- an `async`, `const`, `unsafe` or `extern` function: a `#[component]` function is a plain `fn`;
- an attribute other than doc comments, `allow`, `warn`, `deny` and `deprecated` (write `#[allow(..)]` instead of `#[expect(..)]`).

## See also

- [Find out why something failed or updated](diagnostics.md) for what happens to a
  raised error and how to see it.
- [Build controls with your own look](custom-controls.md) for components that need
  standard behavior (selection, menus, dragging, text editing) with a look of your own.
- [Change the theme and the look of components](theming.md) for theme variables,
  component themes and icons from the app's side.
- [Lay out a screen](layout.md) for the style properties.
- `cargo doc -p kivact --open`: `kivact::view::Element`,
  `kivact::view::Node`, `kivact::component`, `kivact::style::{Theme, ThemeVar, IconSet}`.
