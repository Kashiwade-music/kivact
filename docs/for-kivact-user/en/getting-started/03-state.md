# Step 3: State

[日本語](../../jp/getting-started/03-state.md)

In this step you add a counter of remaining tasks and a button that changes
it. The screen updates by itself when the state changes.

## The program so far

```rust,no_run
use kivact::prelude::*;

fn header() -> impl IntoView {
    column((
        text("Tasks"),
        text("Things to do today"),
    ))
    .style(style().gap(4))
}

fn task_row(title: &str) -> impl IntoView {
    row((text("-"), text(title))).style(style().gap(8))
}

fn task_list() -> impl IntoView {
    column((
        task_row("Buy milk"),
        task_row("Write the tutorial"),
        task_row("Go for a walk"),
    ))
    .style(style().gap(8))
}

fn app() -> impl IntoView {
    let completed = signal(0);
    let left = memo(move || 3 - completed.get());

    column((
        header(),
        task_list(),
        text(move || format!("{} tasks left", left.get())),
        button("Complete one")
            .disabled(move || left.get() == 0)
            .on_press(move || completed.update(|n| *n += 1)),
    ))
    .style(style().padding(24).gap(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

The list is still a placeholder. The counter and the button are a stand-in
that lets you practice state before the real list arrives in step 5.

## What is new

**`signal(0)`** creates a piece of state that starts at `0`. It returns a
`Signal<i32>`, which is a small handle to the state, not the number itself.
The handle is `Copy`, so you can pass it to as many closures as you like
without cloning anything.

You use a signal with a few methods:

- `completed.get()` reads the current value.
- `completed.set(5)` replaces the value.
- `completed.update(|n| *n += 1)` changes the value in place. Use it when the
  new value depends on the old one.
- `completed.with(|n| ..)` borrows the value without copying it. That is
  useful for large values such as a list, and you will use it in step 5.

**Reactive closures.** `text("Tasks")` shows a fixed string. `text(move ||
format!(..))` takes a closure instead. Kivact runs the closure, notes which
signals it read with `get()`, and runs it again whenever one of them changes.
Only that one piece of text is updated. The same works for properties such as
`.disabled(move || ..)`.

**`memo`** computes a value from other state: `left` is `3 - completed`. It
recalculates when `completed` changes, and the closures that read `left`
update only if the result is different. Use `memo` for a value that several
places read; for a single place, a closure is enough.

**`button(..).on_press(..)`** creates a button and runs a closure when the
user presses it, by click, or by the Enter or Space key when it has focus. The button
label doubles as its accessible name.

**Why `move`?** These closures are used long after `app` has returned, so they
cannot borrow local variables. `move` copies the handles into the closure.
Because a `Signal` is `Copy`, the same `completed` can go into several `move`
closures. Without `move` the compiler reports that the closure may outlive
the borrowed value.

**Setup runs once.** If you know React, this is the main difference: `app` is
not a render function that runs again after each state change. It runs once to
build the screen, and the signals created inside it live as long as that part
of the screen does. After that, only the closures that read a signal run again.
This is also why you create signals in the component body, never inside an
event handler.

## Run it

```text
cargo run
```

You see "3 tasks left" and a "Complete one" button. Each press lowers the
number by one. At "0 tasks left" the button becomes disabled.

Next: [Step 4: Input](04-input.md)
