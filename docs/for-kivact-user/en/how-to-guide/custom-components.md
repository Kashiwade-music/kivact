# Write reusable components

[日本語](../../jp/how-to-guide/custom-components.md)

Use this guide to turn a piece of UI you repeat into a component that other code
can reuse: a chip, a card, a notice with an action, a search box. It covers how to
take props, children and callbacks, how to ship a default look the caller can still
change, how to carry your own colors on the theme and how to keep accessibility
intact when a component wraps another control. It assumes you have read the
getting-started tutorial.

A component is a plain function that returns a view. When it takes props, mark it
with `#[component]`: Kivact writes the props builder from the signature, so you
never write a builder by hand. Three shapes cover almost everything:

| Shape | Use it when |
|---|---|
| `#[component] fn card(title: Value<String>, ..) -> impl IntoView` | The component takes props, children or callbacks, optional ones included. |
| `fn header() -> impl Element` | There are no props, and callers should add `.style(..)`, `.test_id(..)` or another node modifier. |
| `fn section() -> impl IntoView` | The result has no single modifiable root: a branch, a fragment or `scope(..)`. |

## Start with a component

The shortest reusable component takes its props as arguments and returns
`impl IntoView`. `#[component]` turns it into a struct, its constructor and a
setter for each optional prop.

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ChipTone {
    Normal,
    Accent,
}
constant_value!(ChipTone);

/// A reusable label with a reactive tone and an action.
#[component]
pub fn chip(
    label: Value<String>,
    #[prop(default = ChipTone::Normal)] tone: Value<ChipTone>,
    on_press: Callback<()>,
) -> impl IntoView {
    button(label).on_press(on_press).style(move || {
        style()
            .radius(12)
            .padding_x(10)
            .padding_y(4)
            .background(if tone.get() == ChipTone::Accent {
                ColorToken::Accent
            } else {
                ColorToken::SurfaceAlt
            })
    })
}

fn app() -> impl IntoView {
    let highlighted = signal(false);
    chip("Kivact", move || highlighted.update(|value| *value = !*value)).tone(move || {
        if highlighted.get() {
            ChipTone::Accent
        } else {
            ChipTone::Normal
        }
    })
}

fn main() -> kivact::Result<()> {
    App::new("Chips").run(app)
}
```

How the signature becomes the API:

- **A required prop is a positional argument.** `chip("Kivact", ..)` takes the
  label and the press handler by position.
- **`#[prop(default = expr)]` and `#[prop(optional)]` become setters.** The chip
  above can also be written `chip("Kivact", ..)` with the tone left out, or
  `chip("Kivact", ..).tone(ChipTone::Accent)`. An optional prop is declared
  `Option<X>` and its setter takes the input `X` accepts. An optional callback is
  `#[prop(optional)] on_close: Option<Callback<()>>`: the body receives
  `Option<Callback<()>>`, already converted in the component's setup.
- **The body runs once, in its own setup owner.** The generated
  `impl IntoView` puts the body in `scope(..)`: signals, effects, callbacks and
  cleanup that it creates belong to this instance and are disposed when the view
  is removed. It adds no layout box; the body's root is the component's root.
- **Only the return value is public.** The generated struct is named after the
  function (`chip` → `Chip`) and implements `IntoView`, not `Element`, so a
  caller cannot write `.style(..)` or `.test_id(..)` on it. Anything the caller
  may change is a prop; anything else is the component's own decision.
- **Pass a payload handler on with a closure.** A `Callback<()>` prop also takes a `Callback<()>` handle, but a `Callback<E>` prop takes a closure: forward a parent's `on_pick: Callback<i32>` as `picker(move |n| on_pick.call(n))`.

`Value<T>` takes a constant, a signal, a `Value` or a closure, so a prop can be
static or reactive without a second API. A `Value<String>` prop also accepts
`&str` and `String`. A value prop's type must be `Clone + PartialEq`, which is
what lets Kivact skip updates when the value did not change.

## Use your own type as a prop

If a prop has a type of your own, register it once so a constant can be passed
directly. Put this next to the type, in the crate that defines it:

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Tone {
    Info,
    Warning,
}

constant_value!(Tone);

/// A label with a tone that follows a signal.
#[component]
pub fn badge(label: Value<String>, tone: Value<Tone>) -> impl IntoView {
    node()
        .name(label)
        .style(move || match tone.get() {
            Tone::Info => style().background(ColorToken::Accent).color(ColorToken::AccentText),
            Tone::Warning => style().background(ColorToken::Warning).color(ColorToken::Text),
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

Without `constant_value!(Tone)`, `badge("Saved", Tone::Info)` fails to compile.
Signals, `Value`s and closures of `Tone` work either way.

## Give a plain function its own setup lifetime

A `#[component]` already has the setup boundary described above. A plain Rust
function that creates signals, effects or cleanup callbacks and should own them
per instance returns `scope(move || ...)` instead. Kivact calls this initializer
once when that instance mounts, under its own owner and the mounting parent's
context, and disposes those resources when the instance is removed. It adds no
layout box.

```rust,no_run
use kivact::prelude::*;

fn local_counter(label: impl IntoText) -> impl IntoView {
    let label = label.into_text_value();
    scope(move || {
        let count = signal(0);
        on_cleanup(|| println!("Counter removed"));
        column((
            text(label),
            text(move || count.get().to_string()),
            button("Increment").on_press(move || count.update(|n| *n += 1)),
        ))
    })
}
```

Changing `count` updates the text; it does not call the initializer again.
Return `impl IntoView` for this boundary: a transparent `View` does not expose a
root node for modifiers. Put a visible root inside it when you need one.
`scope(..)` itself provides no context value; use `provide(value, factory)` to
pass a value to its children.

## Take children and slots

A component that wraps content takes it as `View`. Several children are a tuple,
because a tuple of views is a view, and `()` means "nothing here", so it
doubles as an empty slot. A component with named slots takes one parameter per slot.

```rust,no_run
use kivact::prelude::*;

/// A titled box. `body` is whatever the caller wants inside, `footer` may be `()`.
#[component]
pub fn card(title: Value<String>, body: View, footer: View) -> impl IntoView {
    column((
        text(title).style(style().padding_bottom(4)),
        body,
        footer,
    ))
    .style(style().padding(12).gap(8).border(1, ColorToken::Border).radius(8))
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
pub fn collapsible(
    title: Value<String>,
    open: Binding<bool>,
    content: ViewFn,
) -> impl IntoView {
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

A signal converts into a `Binding<bool>`, so the caller can hand over the signal
itself. A `Binding` is the type for state a component both reads and writes.

When each child needs its own wrapper, for example every pane of a split view,
take `Vec<View>`. The constructor accepts `impl IntoChildren`: a tuple of up to 16
views or a `Vec` of one kind of view, and the body receives each child as a `View`
to wrap.

## Take callbacks

A callback without a payload takes `Callback<()>`:

```rust,no_run
use kivact::prelude::*;

/// A chip with a remove button. Delete removes it too.
#[component]
pub fn removable_chip(label: Value<String>, on_remove: Callback<()>) -> impl IntoView {
    // `on_remove` is used by two handlers. A plain closure can be moved into only
    // one of them; a `Callback` is a Copy handle.
    let remove = on_remove;
    row((
        text(label),
        button("Remove").on_press(move || remove.call(())),
    ))
    .name(label)
    .focusable(true)
    .on_key(move |event: KeyEvent| {
        if event.key() == Key::Delete {
            remove.call(());
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

The constructor takes `impl Into<Callback<()>>`, so a caller passes a closure or
an existing callback. The closure is stored once, in the app's arena, and the
`Callback` it becomes is `Copy`: any number of `move` closures can hold it. Call
it with `.call(argument)`; for a callback without input, use `()` as the argument
as above.

A press handler gets no event. To see whether a modifier key is held, as
`event.getModifierState("Control")` does in React, read `modifiers()`:

```rust,no_run
use kivact::prelude::*;
use kivact::view::modifiers;

/// Opens its panel; Ctrl+click (Cmd+click) opens only this one.
#[component]
pub fn open_button(open: Callback<bool>) -> impl IntoView {
    button("Open").on_press(move || {
        let held = modifiers().get();
        open.call(held.ctrl || held.meta)
    })
}

fn main() -> kivact::Result<()> {
    App::new("Open").run(|| open_button(|only| println!("only this panel: {only}")))
}
```

A callback with a payload is declared `Callback<E>`; the setter accepts
`impl Fn(E) -> R` and infers the closure's argument type, so `|only| ..` needs no
annotation. If the caller's closure may fail, `R` may be `Result<(), UiError>`
instead of `()`; a failure is reported to the nearest `error_boundary` (see
[Find out why something failed or updated](diagnostics.md)).

## Ship a default look

A component owns its root and its default style. The caller changes what the
props expose, not the root directly, because a `#[component]` type has no node
modifiers.

- **Style the component's root in the body.** A style closure that reads a
  signal, a `Value` or `use_theme()` follows changes without a remount.
- **Expose the look as props.** A tone, a size or a variant prop is better than
  a caller's `.style(..)`: it says what may change, and the component keeps the
  rest consistent.
- **A themed default is a value, not a constant.** Read the tokens with
  `use_theme()` inside the style closure, as `chip` does with `ColorToken`. Hard
  coded colors cannot follow a theme.

An element builder (a type that implements `Element`) is different: the caller
holds the same root, so its default look belongs in `base_style`, which sits
**below** every `.style(..)` the caller adds whatever the order of the calls.
Use `.style(..)` inside such a component only for styling that must beat the
caller, which is rarely what you want. A `#[component]` takes a prop instead.

### When you need an `Element`

Write a type that implements `Element` only when callers must apply node
modifiers to it, or when the standard components require one (for example
`field(label, control)` takes an `Element`). Implement `IntoView` to finish the
view, and `Element` by returning the root node from `node_mut`:

- **`Element` is one method.** `fn node_mut(&mut self) -> &mut Node` gives
  `style`, `name`, `disabled`, `test_id`, `on_key`, `on_press`, `draw` and the
  rest to your type. `node_mut` is `doc(hidden)` because it is the implementation
  seam of the trait, not an everyday call: the trait is not sealed, so an
  external type can implement it, but application code never calls the method.
- **Configure the root with `with_node`.** `role`, `checked`, `expanded`,
  `selected`, `value_text` and `base_style` are `Node`'s own methods, which
  `with_node` reaches from a builder: `.with_node(|root| root.role(Role::Alert))`.
- **State belongs to the setup owner.** Create instance resources inside
  `scope(..)` in `into_view`, not in the constructor: a constructor runs in the
  caller's setup, so the resources would live as long as the caller.

## Carry your own design tokens on the theme

Hard-coded colors in a component cannot follow a theme. Put your component's colors,
sizes and fonts on the theme instead, as a type that implements `ThemeExtension`.
Whoever builds the theme can set them; a theme that knows nothing about your type
gets a fallback you derive from the standard palette, so the component still draws
under any theme.

```rust,test_harness
use kivact::prelude::*;
use kivact::style::ThemeExtension;
use kivact::test::{RenderOptions, render_with};

/// The colors of `status_dot`. One type per component library.
#[derive(Debug, PartialEq)]
pub struct StatusColors {
    pub online: Color,
    pub offline: Color,
}

impl ThemeExtension for StatusColors {
    /// What a theme without `StatusColors` gets: derived from its palette.
    fn fallback(theme: &Theme) -> Self {
        StatusColors {
            online: theme.colors().success,
            offline: theme.colors().text_muted,
        }
    }
}

pub fn status_dot(online: impl IntoValue<bool>) -> impl Element {
    let online = online.into_value();
    node()
        .role(Role::Status)
        // `Value` reads are tracked, so the name follows the signal.
        .name(move || if online.get() { "online" } else { "offline" })
        .style(move || {
            // Reading the theme inside the style closure follows theme changes.
            let colors = use_theme().get().ext::<StatusColors>();
            style()
                .size(12, 12)
                .radius(6)
                .background(if online.get() { colors.online } else { colors.offline })
        })
}

fn main() -> kivact::Result<()> {
    let theme = Theme::light().with(StatusColors {
        online: rgb(0x00aa00),
        offline: rgb(0x888888),
    });
    App::new("Status").theme(theme).run(|| status_dot(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot_color(theme: Theme) -> u32 {
        let options = RenderOptions::default().theme(theme);
        let screen = render_with(options, || row(status_dot(true)).style(style().padding(8)));
        let dot = screen.get_by_role(Role::Status, "online").rect().expect("laid out");
        let shot = screen.screenshot();
        shot.pixel((dot.x + 6.0) as u32, (dot.y + 6.0) as u32).expect("inside the window")
    }

    #[test]
    fn the_theme_sets_the_color_and_a_plain_theme_falls_back() {
        let themed = Theme::light().with(StatusColors {
            online: rgb(0x00aa00),
            offline: rgb(0x888888),
        });
        assert_eq!(dot_color(themed), 0x00aa00);
        assert_ne!(dot_color(Theme::light()), 0x00aa00);
    }
}
```

`Theme::with(value)` installs your extension, `theme.ext::<T>()` reads it (a shared
`ThemeRef<T>`, nothing is copied). Two rules keep this fast and correct:

- The type must be `PartialEq + Debug + Send + Sync`. Two themes are equal when
  their extensions are equal, so an unchanged theme never restyles.
- If the extension holds style functions rather than plain values, build it once
  and share it (a `static OnceLock` that your theme constructor clones). A table
  built anew on every call is a different table each time and restyles everything
  under it.

To switch themes while the app runs, wrap the root in `provide_theme` with a
`Signal<Theme>` ([Switch the theme while the app runs](theming.md#switch-the-theme-while-the-app-runs)).
Every style closure that called
`use_theme()` re-runs with the new theme.

## Draw icons through the theme

Do not embed an image in a component. Name the icon, and let the theme decide what
the name looks like. A kit can then give your component its own glyphs, and an app
can replace any of them.

```rust,test_harness
use kivact::ErrorKind;
use kivact::style::Icon;
use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with};

/// A square button that shows the icon named `name`.
pub fn icon_button(name: impl IntoValue<Icon>, label: impl IntoText) -> impl Element {
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
    // The app (or a kit's theme constructor) supplies the image for the name.
    let theme = Theme::light().with_icon("search", SEARCH);
    App::new("Icons").theme(theme).run(|| icon_button("search", "Search"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_theme_resolves_the_name() {
        let theme = Theme::light().with_icon("search", SEARCH);
        let options = RenderOptions::default().theme(theme);
        let screen = render_with(options, || icon_button("search", "Search"));
        screen.get_by_role(Role::Button, "Search");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }

    #[test]
    fn an_unknown_name_is_reported_in_debug_builds() {
        let screen = render_with(RenderOptions::default(), || icon_button("nope", "Nope"));
        screen.run_until_idle();
        if cfg!(debug_assertions) {
            let errors = screen.errors();
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].kind(), &ErrorKind::Asset);
        }
    }
}
```

`Icon::named("search")` and a bare `"search"` are the same thing, so props of type
`impl IntoValue<Icon>` accept a name, an `Asset` (an image used as is) or a signal of
either. The standard set already has the marks the standard components draw, such
as `check` and `chevron-right`; add yours with `Theme::with_icon(name, image)`.
An unknown name draws nothing and, in debug builds, is reported once.

## Put the accessible name on the real control

A composite component, such as a search box made of an icon and a text input, has a
root that is not the control the user types into. The name and the disabled state
belong to that control: if they stay on the wrapper, a screen reader announces an
unnamed text box. Take them as props and apply them to the control:

```rust,test_harness
use kivact::prelude::*;

/// A search box: an icon and a text field that carry the caller's name and state.
#[component]
pub fn site_search(
    value: Binding<String>,
    #[prop(default = "Site search")] label: Value<String>,
    #[prop(default = false)] disabled: Value<bool>,
) -> impl IntoView {
    let input = text_input(label, value)
        .style(style().grow(1))
        .disabled(move || disabled.get());
    field(
        label,
        row((icon("search"), input)).style(style().gap(6).align_items(Align::Center)),
    )
}

fn app() -> impl IntoView {
    let query = signal(String::new());
    site_search(query).label("Site search")
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

`field(label, control)` also accepts only an `Element`, not a `#[component]`
type. When the caller should wrap your component in a field, either take the
label and the hint as props and build the `field` inside the component, or return
an element builder from a plain function.

Focus needs no forwarding: pressing a field's label activates the first focusable
element inside it, here the text box, as the second test shows.

## Report misuse of your component

When a component gets an argument it cannot honor, fail where the caller can see the
cause. `kivact::custom::raise` panics at the caller with a located message,
exactly like Kivact's own misuse; it ends the App or fails the test:

```rust,no_run

use kivact::ErrorKind;
use kivact::prelude::*;
use kivact::custom::raise;

#[track_caller]
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

- generic parameters or a `where` clause: take `Value<T>` props instead;
- a return type other than `impl IntoView`;
- a method: a `#[component]` is a free function;
- a destructuring pattern: each prop is a named argument;
- `#[prop(optional)]` on a type that is not `Option<..>`;
- `#[prop(optional)]` together with `#[prop(default = ..)]`;
- an unknown key in `#[prop(..)]`;
- an `async`, `const`, `unsafe` or `extern` function: a `#[component]` function is a plain `fn`;
- an attribute other than doc comments, `allow`, `warn`, `deny` and `deprecated` (write `#[allow(..)]` instead of `#[expect(..)]`).

## See also

- [Find out why something failed or updated](diagnostics.md) for what happens to a
  raised error and how to see it.
- [Build controls with custom visuals](custom-controls.md) for components that need
  standard behavior (selection, menus, dragging, text editing) with a look of your own.
- [Change the theme and the look of components](theming.md) for tokens, recipes and icons
  from the app's side.
- [Lay out a screen](layout.md) for the style properties a base style can set.
- `cargo doc -p kivact --open`: `kivact::view::Element`,
  `kivact::view::Node`, `kivact::component`, `kivact::style::{Theme, ThemeExtension, IconSet}`.
