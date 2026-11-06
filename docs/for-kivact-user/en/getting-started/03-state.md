# Step 3: State

[日本語](../../jp/getting-started/03-state.md)

In this step you add a counter of remaining tasks and a button that changes
it. The screen updates by itself when the state changes.

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

**Setup runs once per mount.** `App::run` calls the root setup once. Ordinary
helpers execute in their caller’s setup scope; `component(move || ..)` creates
the explicit setup and cleanup boundary introduced in step 2. This resembles a
Solid component more than a React render function. State changes rerun the
small reactive expressions that read that state, rather than the whole setup.
Create signals, memos and bindings during setup; handlers update the handles
already created there.

## React and Solid correspondences

If you know React or Solid, choose by the role a value plays rather than by
renaming one to one. The prelude already provides the common value, binding,
callback and effect names:

| React / Solid concept | Kivact |
| --- | --- |
| `useState` / `createSignal` | `signal(initial)`; read with `get` or `with`, write with `set` or `update` |
| `useMemo` / `createMemo` | `memo(move || ..)` for a shared derived value |
| A value prop or accessor | `impl IntoValue<T>` accepts a constant, signal, memo or closure; `Value<T>` is a readonly handle |
| A controlled input | A `Binding<T>` connects reads and writes; a signal can be passed directly |
| An event prop | A closure, or a retained `Callback<T>` made with `callback(..)` |
| `useEffect` / `createEffect` | `effect(move || ..)` synchronizes with something outside the UI when the state it reads changes; compute displayed values with `memo` instead |
| `useReducer` with effects and timers | `Machine` keeps related states and events together; a transition returns `Step` with optional effects and deadlines |

This counter needs only `signal` and `memo`. Use a signal for each independent
editable value, and a memo for a value derived from it. When several flags
describe stages of one operation, such as press-and-hold, waiting, running and
cancelling, use one Machine instead of synchronizing several signals. Machine definitions use
`kivact::state::{Machine, Step, machine}`. The
[React and Solid guide](../how-to-guide/coming-from-react-or-solid.md) shows
these patterns in more detail.

## Run it

```text
cargo run
```

You see "3 tasks left" and a "Complete one" button. Each press lowers the
number by one. At "0 tasks left" the button becomes disabled.

Next: [Step 4: Input](04-input.md)
