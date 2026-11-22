# Step 2: Components and layout

[日本語](../../jp/getting-started/02-components-and-layout.md)

In this step you lay out the screen with `column` and `row`, and split it into
small components: a header and a placeholder list of tasks.

## The program so far

```rust,no_run
use kivact::prelude::*;

#[component]
fn header() -> impl Element {
    column((
        text("Tasks"),
        text("Things to do today"),
    ))
    .style(style().gap(4))
}

#[component]
fn task_row(title: Value<String>) -> impl Element {
    row((text("-"), text(title))).style(style().gap(8))
}

#[component]
fn task_list() -> impl Element {
    column((
        task_row("Buy milk"),
        task_row("Write the tutorial"),
        task_row("Go for a walk"),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    column((header(), task_list())).style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

## What is new

**A component is a function marked `#[component]`.** `header`, `task_row` and
`task_list` are components. This is the only way to write one: the standard
components such as `button` and the components of the UI kits are written the
same way. A component returns `impl Element`, a view with one root that its
caller can change: `task_list().style(..)`, `.name(..)` or `.test_id(..)` apply
to the `column` inside it.

**Props are arguments.** `task_row` takes a `title: Value<String>`. A `Value`
prop accepts a fixed value (`"Buy milk"`, a `String`) or one that changes (a
signal, or a closure, from the next step on), so `task_row("Buy milk")` works.
Arguments are positional, in the order of the signature. An argument of type
`Option<..>` is optional and becomes a setter instead, such as
`.on_close(..)`; see [Write reusable components](../how-to-guide/custom-components.md).

**The body runs once.** Kivact runs a component's body once, when the
component appears on screen, and gives it its own lifetime: state created in the
body belongs to that one instance and is cleaned up when it disappears. When
state changes, only the small closures that read it run again (next step); the
body itself never runs again. This is how SolidJS works, not how a React render
function works.

```rust,no_run
use kivact::prelude::*;

#[component]
fn click_counter() -> impl Element {
    let count = signal(0);
    column((
        text(move || format!("Pressed {} times", count.get())),
        button("Press me").on_press(move || count.update(|n| *n += 1)),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    row((click_counter(), click_counter())).style(style().gap(24).padding(24))
}

fn main() -> kivact::Result<()> {
    App::new("Counters").run(app)
}
```

Each `click_counter` has its own `count`. Pressing one button updates only the
text next to it. You will use signals and these closures in the next step.

Create signals and memos in setup code: the body of a component, the root
function, and the code that `show`, `dynamic` and `for_each` run. Closures such
as `text(move || ..)` run again on changes, so they read values that setup
already created.

**The root is a plain function.** `App::new("Tasks").run(app)` calls `app`
once and shows what it returns. `run` takes any function that returns a view.
The root of this tutorial stays a plain function named `app`; everything below it
is a component.

**`column` and `row`** place their children vertically and horizontally. They
take their children as a tuple: `column((a, b, c))`. The extra pair of
parentheses is the tuple, so `column(a, b)` does not compile. The children can
have different types, and a tuple can hold up to 16 of them.

**`style()`** describes how a view looks and is laid out. You attach it with
`.style(..)`:

- `gap(8)` puts 8 pixels between the children of a `column` or `row`.
- `padding(24)` adds 24 pixels of space inside the view, around its children.

The names follow CSS: `gap`, `padding`, `flex_grow`, `border_radius`,
`font_weight` and so on. Lengths are logical pixels, so they scale with the
display. Style methods chain because each one returns the style again.

`text` accepts a string literal, a `&str`, a `String` or a `Value<String>`.

## Run it

```text
cargo run
```

You see the title "Tasks" with "Things to do today" below it, then three lines
starting with a dash, with spacing between them. Change `gap(8)` to `gap(24)`
and run again to see the lines move apart.

Next: [Step 3: State](03-state.md)
