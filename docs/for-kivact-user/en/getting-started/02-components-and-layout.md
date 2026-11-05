# Step 2: Components and layout

[日本語](../../jp/getting-started/02-components-and-layout.md)

In this step you lay out the screen with `column` and `row`, and split it into
small components: a header and a placeholder list of tasks.

## The program so far

```rust,no_run
use kivact::prelude::*;

fn header() -> impl Component {
    column((
        text("Tasks"),
        text("Things to do today"),
    ))
    .style(style().gap(4))
}

fn task_row(title: impl IntoText) -> impl Component {
    row((text("-"), text(title))).style(style().gap(8))
}

fn task_list() -> impl Component {
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

**View helpers are plain functions.** `header`, `task_row` and `task_list` run
when their caller calls them. `task_row` accepts `impl IntoText`, so callers can
supply a string or a reactive text value. These helpers return `impl Component`:
a view with one modifiable root, so callers can add `.style(..)`, `.name(..)` or
other standard node modifiers. Use `impl IntoView` when the result can be a
branch, a fragment or another view without one modifiable root.

**A setup boundary is explicit.** Calling a Rust helper does not create a new
lifetime for its state. `App::run(app)` sets up the root once. For a reusable
view that should own its own signals, effects and cleanup, return
`component(move || { .. })`:

```rust,no_run
use kivact::prelude::*;

fn click_counter() -> impl IntoView {
    component(move || {
        let count = signal(0);
        column((
            text(move || format!("Pressed {} times", count.get())),
            button("Press me").on_press(move || count.update(|n| *n += 1)),
        ))
        .style(style().gap(8))
    })
}
```

The closure runs once when this view mounts. Its state is disposed when the
view unmounts; changing `count` updates the text without running setup again.
The boundary adds no visible layout box. Style the `column` inside it, since
the returned `View` does not expose a single root to modify. You will use
signals and reactive closures in the next step.

**`column` and `row`** place their children vertically and horizontally. They
take their children as a tuple: `column((a, b, c))`. The extra pair of
parentheses is the tuple, so `column(a, b)` does not compile. The children can
have different types, and a tuple can hold up to 16 of them.

**`style()`** describes how a view looks and is laid out. You attach it with
`.style(..)`:

- `gap(8)` puts 8 pixels between the children of a `column` or `row`.
- `padding(24)` adds 24 pixels of space inside the view, around its children.

Lengths are logical pixels, so they scale with the display. Style methods chain
because each one returns the style again.

`text` accepts a string literal, a `&str` or a `String`.

## Run it

```text
cargo run
```

You see the title "Tasks" with "Things to do today" below it, then three lines
starting with a dash, with spacing between them. Change `gap(8)` to `gap(24)`
and run again to see the lines move apart.

Next: [Step 3: State](03-state.md)
