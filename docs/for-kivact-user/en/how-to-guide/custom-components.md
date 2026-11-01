# Write reusable components

[日本語](../../jp/how-to-guide/custom-components.md)

Use this guide to turn a piece of UI you repeat into a component that other code
can reuse: a chip, a card, a notice with an action, a search box. It covers how to
take props, children and callbacks, how to ship a default look the caller can still
change, how to carry your own colors on the theme and how to keep accessibility
intact when a component wraps another control. It assumes you have read the
getting-started tutorial.

A component is a plain function. It runs once, in the setup of the place where it
is used, and returns something that can be shown. Two shapes cover almost
everything:

| Shape | Use it when |
|---|---|
| A function returning `Node` | There are a few props, all of them needed. |
| A builder struct that holds a root `Node` | There are optional props, slots or settings that read better as methods. |

## Start with a function

The shortest reusable component takes its props as arguments and returns a `Node`.

```rust,no_run
use kivact::prelude::*;

/// A label in a pill. It looks highlighted while `selected` is true.
pub fn chip(label: impl IntoText, selected: impl IntoValue<bool>) -> Node {
    let label = label.into_text_value();
    let selected = selected.into_value();
    node()
        .name(label)
        .style(move || {
            let (background, color) = if selected.get() {
                (ColorToken::Accent, ColorToken::AccentText)
            } else {
                (ColorToken::SurfaceAlt, ColorToken::Text)
            };
            style()
                .padding_x(10)
                .padding_y(4)
                .radius(12)
                .background(background)
                .color(color)
        })
        .child(text(label))
}

fn app() -> impl IntoView {
    let on = signal(false);
    row((
        chip("Rust", true),
        chip("Kivact", on),
        chip("Tutorial", move || on.get()),
        button("Toggle").on_press(move || on.update(|v| *v = !*v)),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Chips").run(app)
}
```

Three habits make this component pleasant to call:

- **Take `impl IntoText` and `impl IntoValue<T>`, not `String` and `bool`.** The
  caller can then pass a constant (`true`), a signal (`on`) or a closure
  (`move || on.get()`), and the component follows it. A plain `bool` argument
  would freeze the value at the moment of the call.
- **Convert once, at the top.** `into_text_value()` and `into_value()` turn the
  argument into a `Value`, a small `Copy` handle. Because it is `Copy`, several
  `move` closures can use it without cloning anything. This is the same rule as
  for signals in the tutorial.
- **Return `Node` so the caller can keep chaining.** `chip("Rust", true).test_id("x")`,
  `.style(..)`, `.on_press(..)` and `.name(..)` all work because `Node` has the
  everyday modifiers. When you must return several roots at once, return
  `impl IntoView` instead and accept that the caller can no longer modify them.

If a prop has a type of your own, register it once so a constant can be passed
directly. Put this next to the type, in the crate that defines it:

```rust,no_run
use kivact::prelude::*;
use kivact::view::constant_value;

#[derive(Clone, Copy, PartialEq)]
pub enum Tone {
    Info,
    Warning,
}

constant_value!(Tone);

pub fn badge(label: impl IntoText, tone: impl IntoValue<Tone>) -> Node {
    let label = label.into_text_value();
    let tone = tone.into_value();
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
Signals, `Value`s and closures of `Tone` work either way, and a caller can wrap a
constant by hand with `kivact::view::constant(Tone::Info)`. The prop type must be
`Clone + PartialEq`, which is what lets Kivact skip updates when the value did not
change.

## Take children and slots

A component that wraps content takes it as `impl IntoView`. Several children are a
tuple, because a tuple of views is a view, and `()` means "nothing here", so it
doubles as an empty slot. A component with named slots takes one parameter per slot.

```rust,no_run
use kivact::prelude::*;

/// A titled box. `body` is whatever the caller wants inside, `footer` may be `()`.
pub fn card(title: impl IntoText, body: impl IntoView, footer: impl IntoView) -> Node {
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

A value of type `impl IntoView` is used up when it is mounted. If your component
mounts its content more than once, for example each time a section opens, ask for a
function that builds the content, as `show` does:

```rust,no_run
use kivact::Binding;
use kivact::prelude::*;

/// A section that builds its content when it opens and drops it when it closes.
pub fn collapsible<V: IntoView>(
    title: impl IntoText,
    open: impl Into<Binding<bool>>,
    content: impl Fn() -> V + 'static,
) -> Node {
    let title = title.into_text_value();
    let open = open.into();
    column((
        button(title).on_press(move || open.set(!open.get())),
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
take `impl kivact::ui::IntoChildren`. It accepts a tuple of up to 16 views or a
`Vec` of one kind of view, and gives you each child as a `View` to wrap.

## Take callbacks

A callback prop is a closure the caller supplies. Accept it as `impl Fn(..) + 'static`:

```rust,no_run
use kivact::prelude::*;
use kivact::view::{Handled, KeyEvent, callback};

/// A chip with a remove button. Delete removes it too.
pub fn removable_chip(label: impl IntoText, on_remove: impl Fn() + 'static) -> Node {
    let label = label.into_text_value();
    // `on_remove` is used by two handlers. A plain closure can be moved into only
    // one of them; `callback` stores it and gives back a Copy handle.
    let remove = callback(move |_: ()| on_remove());
    row((
        text(label),
        button("Remove").on_press(move || remove.call(())),
    ))
    .name(label)
    .focusable(true)
    .on_key(move |event: KeyEvent| {
        if event.key == Key::Delete {
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

The Rust point behind `callback`: a closure that owns captured data can be moved
into one place only, and an `Fn` bound gives you no way to clone it. `callback(f)` stores
`f` in the app's arena and returns a `Callback`, which is `Copy`, so any number of
`move` closures can hold it. Call it with `.call(argument)`; for a callback without
input, use `()` as the argument as above.

If the caller's closure may fail, widen the bound so `?` works for them:
`impl Fn() -> R + 'static where R: HandlerResult` accepts `()` and
`Result<(), UiError>`, and `callback` takes either. A failure is reported to the
nearest `error_boundary` (see [Find out why something failed or
updated](diagnostics.md)).

## Use a builder when props pile up

Once a component has optional props, slots or settings, a builder reads better than a
long argument list. A builder is a struct that holds its root `Node`, implements
`NodeBuilder` to get all the everyday modifiers for free, and implements `IntoView`
to finish the job.

```rust,test_harness
use kivact::Callback;
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, current_theme};
use kivact::view::{HandlerResult, View, callback, constant_value};

#[derive(Clone, Copy, PartialEq)]
pub enum Tone {
    Info,
    Warning,
}

constant_value!(Tone);

pub struct Notice {
    root: Node,
    message: Value<String>,
    tone: Value<Tone>,
    action: Option<View>,
    on_dismiss: Option<Callback<()>>,
}

/// A message bar. `#[track_caller]` makes every notice report the place it was
/// created, not a line inside this file.
#[track_caller]
pub fn notice(message: impl IntoText) -> Notice {
    let message = message.into_text_value();
    Notice {
        root: node().role(Role::Alert).name(message),
        message,
        tone: Tone::Info.into_value(),
        action: None,
        on_dismiss: None,
    }
}

impl Notice {
    pub fn tone(mut self, tone: impl IntoValue<Tone>) -> Self {
        self.tone = tone.into_value();
        self
    }

    /// A slot for one extra control, such as a retry button.
    pub fn action(mut self, action: impl IntoView) -> Self {
        self.action = Some(action.into_view());
        self
    }

    pub fn on_dismiss<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.on_dismiss = Some(callback(move |_: ()| f()));
        self
    }
}

impl NodeBuilder for Notice {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Notice {
    fn into_view(self) -> View {
        let Notice { root, message, tone, action, on_dismiss } = self;
        // The default look. It goes below every style the caller adds.
        let mut root = root.base_style(move || {
            let (background, color) = match tone.get() {
                Tone::Info => (ColorToken::Accent, ColorToken::AccentText),
                Tone::Warning => (ColorToken::Warning, ColorToken::Text),
            };
            let theme = current_theme();
            style()
                .direction(Direction::Row)
                .align_items(Align::Center)
                .gap(theme.space().sm)
                .padding(theme.space().md)
                .radius(theme.radius().md)
                .width(240)
                .background(background)
                .color(color)
        });
        root = root.child(text(message).style(style().grow(1)));
        if let Some(action) = action {
            root = root.child(action);
        }
        if let Some(dismiss) = on_dismiss {
            root = root.child(button("Dismiss").on_press(move || dismiss.call(())));
        }
        root.into_view()
    }
}

fn app() -> impl IntoView {
    let visible = signal(true);
    column((
        notice("Saved").test_id("plain"),
        notice("Narrow").style(style().width(120)).test_id("narrow"),
        show(visible, move || {
            notice("Offline")
                .tone(Tone::Warning)
                .action(button("Retry"))
                .on_dismiss(move || visible.set(false))
        }),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Notices").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_caller_style_beats_the_default_and_the_slots_work() {
        let screen = kivact::test::render(app);

        // The default width is 240; the caller's style wins although the
        // component's `base_style` ran after the caller's `.style(..)`.
        let plain = screen.get_by_test_id("plain").rect().expect("laid out");
        let narrow = screen.get_by_test_id("narrow").rect().expect("laid out");
        assert_eq!((plain.width, narrow.width), (240.0, 120.0));

        screen.get_by_role(Role::Button, "Retry");
        screen.get_by_role(Role::Button, "Dismiss").click();
        assert!(screen.query_by_text("Offline").is_none());
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
```

Points worth knowing about this shape:

- **`NodeBuilder` is one method.** `fn node_mut(&mut self) -> &mut Node` gives
  `style`, `name`, `disabled`, `test_id`, `on_key`, `on_press`, `draw` and the rest
  to your builder. A modifier the caller writes after your constructor wins over
  what the constructor set.
- **`NodeExt` is for what the component is.** `role`, `checked`, `expanded`,
  `selected`, `value_text` and `base_style` belong to the component, so they are
  in `kivact::primitive::NodeExt`, which you import in the component's crate. An
  app that imports only the prelude cannot overwrite the role you set on a builder.
- **Do the final work in `into_view`.** It runs when the component is mounted, when
  every setting is known. Read your fields there instead of sharing a `Cell`
  between a setter and a style closure.
- **State belongs to the instance.** Signals you create in `into_view` (or in the
  constructor) are owned by the place where the component is mounted, so two
  notices on one screen share nothing. You never need a string key or a global id.
- **`#[track_caller]` names each instance.** Diagnostics and test failure reports
  name a node by the place its builder was called. Put the attribute on your
  constructor, and every instance is reported at its own call site. A component
  that creates nodes later, in `into_view`, can capture `Location::caller()` in its
  constructor and pass it to `.at(..)` of those nodes.

### Default styles the caller can override

`base_style` adds a style layer **below** every `.style(..)` of the caller, whatever
the order of the calls. It is the right place for a component's default look. The
test above shows it: the notice sets `width(240)` as its default, and
`.style(style().width(120))` still wins. Use `.style(..)` inside a component only
for styling that must beat the caller, which is rarely what you want.

A base style may be a closure. Reading a signal, a `Value` or `current_theme()`
inside it makes the style follow the change, with no remount.

## Carry your own design tokens on the theme

Hard-coded colors in a component cannot follow a theme. Put your component's colors,
sizes and fonts on the theme instead, as a type that implements `ThemeExtension`.
Whoever builds the theme can set them; a theme that knows nothing about your type
gets a fallback you derive from the standard palette, so the component still draws
under any theme.

```rust,test_harness
use kivact::prelude::*;
use kivact::style::{Color, ThemeExtension, current_theme};
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

pub fn status_dot(online: impl IntoValue<bool>) -> Node {
    let online = online.into_value();
    node()
        .role(Role::Status)
        .name(move || if online.get() { "online" } else { "offline" }.to_owned())
        .style(move || {
            // Reading the theme inside the style closure follows theme changes.
            let colors = current_theme().get::<StatusColors>();
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
        let options = RenderOptions { theme, ..RenderOptions::default() };
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

`Theme::with(value)` installs your extension, `theme.get::<T>()` reads it (a shared
`Arc<T>`, nothing is copied). Two rules keep this fast and correct:

- The type must be `PartialEq + Debug + Send + Sync`. Two themes are equal when
  their extensions are equal, so an unchanged theme never restyles.
- If the extension holds style functions rather than plain values, build it once
  and share it (a `static OnceLock` that your theme constructor clones). A table
  built anew on every call is a different table each time and restyles everything
  under it.

To switch themes while the app runs, wrap the root in `provide_theme` with a
`Signal<Theme>` ([Switch the theme while the app runs](theming.md#switch-the-theme-while-the-app-runs)).
Every style closure that called
`current_theme()` re-runs with the new theme.

## Draw icons through the theme

Do not embed an image in a component. Name the icon, and let the theme decide what
the name looks like. A kit can then give your component its own glyphs, and an app
can replace any of them.

```rust,test_harness
use kivact::diagnostics::ErrorKind;
use kivact::style::Icon;
use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with};

/// A square button that shows the icon named `name`.
pub fn icon_button(name: impl IntoValue<Icon>, label: impl IntoText) -> Node {
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
        let options = RenderOptions { theme, ..RenderOptions::default() };
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

## Forward accessibility to the real control

A composite component, such as a search box made of an icon and a text input, has a
root that is not the control the user types into. Callers still write
`.name("Site search")`, `.disabled(..)` and `.invalid(..)` on the component, and a
`field` wraps it with `.labelled_by(..)` and `.described_by(..)`. Those settings land
on your root. Read them from the root and apply them to the real control, otherwise
a screen reader announces an unnamed text box.

```rust,test_harness
use kivact::Binding;
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::view::View;

pub struct SearchBox {
    root: Node,
    value: Binding<String>,
}

#[track_caller]
pub fn search_box(value: impl Into<Binding<String>>) -> SearchBox {
    SearchBox {
        root: row(()).style(style().gap(6).align_items(Align::Center)),
        value: value.into(),
    }
}

impl NodeBuilder for SearchBox {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SearchBox {
    fn into_view(self) -> View {
        let SearchBox { mut root, value } = self;

        // What the caller said about the component as a whole.
        let name = root.accessible_name();
        let disabled = root.accessible_disabled();
        let invalid = root.accessible_invalid();
        let labelled_by = root.accessible_labelled_by();
        let described_by = root.accessible_described_by();

        // The name the caller gave wins; otherwise the component has its own.
        let label: Value<String> = match name {
            Some(read) => (move || read()).into_value(),
            None => "Search".into_text_value(),
        };

        // Hand all of it to the control that takes the keystrokes.
        let mut input = text_input(label, value).style(style().grow(1));
        if let Some(read) = disabled {
            input = input.disabled(move || read());
        }
        if let Some(read) = invalid {
            input = input.invalid(move || read());
        }
        for target in labelled_by {
            input = input.labelled_by(target);
        }
        for target in described_by {
            input = input.described_by(target);
        }

        root.child((icon("search"), input)).into_view()
    }
}

fn app() -> impl IntoView {
    let query = signal(String::new());
    field("Site search", search_box(query)).hint("Searches every page")
}

fn main() -> kivact::Result<()> {
    App::new("Search").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_box_carries_the_name_and_the_description() {
        let screen = kivact::test::render(app);
        let input = screen.get_by_role(Role::TextBox, "Site search");
        assert_eq!(input.described_by().len(), 1);
        input.type_text("kivact");
        assert_eq!(input.value().as_deref(), Some("kivact"));
    }
}
```

The `accessible_*` readers copy what the caller set without consuming it, so the
root keeps its own settings as well. The same pattern applies to anything a
caller can set on the whole component and that belongs to one part inside it.

## Report misuse of your component

When a component gets an argument it cannot honor, fail where the caller can see the
cause. `kivact::primitive::raise` raises a located error that the nearest
`error_boundary` or the default sink reports, exactly like Kivact's own misuse errors:

```rust,no_run
use std::panic::Location;

use kivact::diagnostics::ErrorKind;
use kivact::prelude::*;
use kivact::primitive::raise;

#[track_caller]
pub fn progress_ring(done: u32, total: u32) -> Node {
    if total == 0 {
        raise(
            ErrorKind::Message,
            "progress_ring: `total` must be greater than zero",
            Location::caller(),
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

## See also

- [Find out why something failed or updated](diagnostics.md) for what happens to a
  raised error and how to see it.
- [Build controls with custom visuals](custom-controls.md) for components that need
  standard behavior (selection, menus, dragging, text editing) with a look of your own.
- [Change the theme and the look of components](theming.md) for tokens, recipes and icons
  from the app's side.
- [Lay out a screen](layout.md) for the style properties a base style can set.
- `cargo doc -p kivact --open`: `kivact::control::NodeBuilder`,
  `kivact::primitive::NodeExt`, `kivact::style::{Theme, ThemeExtension, IconSet}`.
