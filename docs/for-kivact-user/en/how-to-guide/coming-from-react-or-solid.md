# Map what you know from React or SolidJS to Kivact

[日本語](../../jp/how-to-guide/coming-from-react-or-solid.md)

If you know React or SolidJS, most Kivact concepts have a direct counterpart.
This guide maps them one to one, explains the single big difference (components
run once, like Solid's, not on every state change like React's), and covers the
seven Rust details that trip up developers who have not written Rust before.

It assumes you have done the [tutorial](../getting-started/README.md).

## The one idea: setup runs once

A Kivact component is a plain function. It runs **once**, when its parent builds
the screen. It creates state, wires things together and returns a view. After
that, the function never runs again. When a signal changes, only the pieces of
the screen that read it are updated.

```tsx
// React: the whole function runs again on every click.
function Counter() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(count + 1)}>count: {count}</button>;
}
```

```rust,test_harness
use kivact::prelude::*;

// Kivact: the function runs once; only the `text` closure re-runs.
fn counter() -> impl IntoView {
    let count = signal(0);
    column((
        text(move || format!("count: {}", count.get())),
        button("+1").on_press(move || count.update(|c| *c += 1)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Counter").run(counter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_reading_closure_updates() {
        let screen = kivact::test::render(counter);
        screen.get_by_text("count: 0");
        screen.get_by_role(Role::Button, "+1").click();
        screen.get_by_text("count: 1");
    }
}
```

This is the model of SolidJS. Two consequences:

- **Pass a closure where a value changes.** `text(format!("count: {}", n))` reads
  `n` once while building and shows that text forever. `text(move || ...)` reads
  it inside a closure Kivact re-runs when `n` changes. The same goes for
  `.disabled(..)`, `.style(..)`, `.name(..)` and every other property that can
  change: give them a closure, a signal or a memo.
- **There is no dependency array.** Anything you read with `.get()` or `.with()`
  inside a closure, memo or effect is tracked automatically.

## Concept map

| Idea | React | SolidJS | Kivact |
| --- | --- | --- | --- |
| Component | function returning JSX | function returning JSX | function returning `impl IntoView` |
| Props | props object | props object | function arguments; `impl IntoText` for text, `impl IntoValue<T>` for values that may change |
| Children | `children` | `props.children` | an argument of type `V: IntoView` |
| Callback prop | `onSave` prop | `onSave` prop | `impl Into<Callback<()>>` for an action without input; convert once to share it |
| State machine | `useReducer` plus effects | reducer plus effects | `Machine`: explicit states, events, effects and timers |
| State | `useState` | `createSignal` | `signal(value)`; `get`, `with`, `set`, `update` on one `Signal<T>` |
| Derived value | compute in render, `useMemo` | `createMemo` | a closure for one use, `memo(..)` to share or cache |
| Effect | `useEffect` | `createEffect` | `effect(..)`; `effect_on(source, f)` to skip the first run |
| Cleanup | `useEffect` return value | `onCleanup` | `on_cleanup(..)` |
| Run once after mount | `useEffect(.., [])` | `onMount` | `on_mount(..)` |
| Mutable value, no re-render | `useRef` | a plain variable | `var(value)` |
| Element ref | `useRef` on a DOM node | `ref` | `node_ref()` and `.node_ref(r)`; `r.focus()` |
| Context | `createContext`, `useContext` | `createContext`, `useContext` | `provide(value, \|\| child)` and `use_context::<T>()` |
| Conditional | `cond && <A/>` | `<Show when>` | `show(cond, \|\| a)`, `show_some(option, \|v\| a)` |
| Several branches | ternary or `switch` | `<Switch>` | `dynamic(source, \|v\| match v { .. })` |
| List | `items.map(..)` with `key` | `<For>` | `for_each(items, \|i\| key, \|item\| row)` |
| List by position | `key={index}` | `<Index>` | `for_index(items, \|i, item\| row)` |
| Key | `key` prop | the `For` identity | the key function you give `for_each` |
| Controlled input | `value` + `onChange` | `value` + `onInput` | a `Signal<String>` (or a `Binding<T>`) passed to the input |
| Error boundary | `ErrorBoundary` | `ErrorBoundary` | `error_boundary(..)` |
| Data loading | libraries or `useEffect` | `createResource` | `resource(source, fetch)`; `action(..)` for user actions |

All the ordinary UI names in the table are available through
`use kivact::prelude::*;`, including `callback`, `error_boundary`
and `effect_on`. For a state machine, also import `Machine`, `Step` and
`machine` from `kivact::state`.


`Machine` is `useReducer` with effects and timers. Define explicit states and
events, return a `Step` from `transition`, and execute effects in the runner.
See [custom controls](custom-controls.md#write-behavior-with-a-state-machine).


## Props, children and callbacks

```rust,test_harness
use kivact::prelude::*;

// `impl IntoText`: a literal, a String, or a closure that returns text.
// `impl IntoValue<i32>`: a number, a signal, a memo or a closure.
fn stat(label: impl IntoText, value: impl IntoValue<i32>) -> impl IntoView {
    let value = value.into_value();
    row((text(label), text(move || value.get().to_string()))).style(style().gap(8))
}

// `V: IntoView`: any view, including another component.
fn panel<V: IntoView>(title: &str, child: V) -> impl IntoView {
    column((text(title), child)).style(style().gap(8).padding(12))
}

// A callback prop: a closure the caller gives you.
fn buttons(on_add: impl Into<Callback<()>>) -> impl IntoView {
    let on_add = on_add.into();
    row((button("Add").on_press(on_add), button("Add again").on_press(on_add)))
}

fn app() -> impl IntoView {
    let hits = signal(0);
    panel(
        "Stats",
        column((
            stat("Fixed", 42),
            stat("Signal", hits),
            stat("Derived", move || hits.get() * 2),
            buttons(move || hits.update(|h| *h += 1)),
        )),
    )
}

fn main() -> kivact::Result<()> {
    App::new("Props").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_prop_type_takes_constants_signals_and_closures() {
        let screen = kivact::test::render(app);
        screen.get_by_text("42");
        assert_eq!(screen.all_by_text("0").len(), 2);
        screen.get_by_role(Role::Button, "Add again").click();
        screen.get_by_text("1");
        screen.get_by_text("2");
    }
}
```

- A prop that may change takes `impl IntoValue<T>`: the caller passes a
  constant, a signal, a memo or a closure, and you call `.into_value()` once to
  get a `Value<T>` handle to read.
- A callback prop without an input takes `impl Into<Callback<()>>`. Convert it
  once with `.into()`; the resulting handle is `Copy`, so both buttons can use
  the same action. The caller can pass a `move || ...` closure directly.

## Effects, cleanup and mount

```rust,no_run
use kivact::prelude::*;

fn tracker(query: Signal<String>) -> impl IntoView {
    component(move || {
        // Runs once the screen is built, then again whenever `query` changes.
        effect(move || {
            println!("searching for {}", query.get());
        });
        // Runs once, after the nodes of this component exist.
        on_mount(|| println!("mounted"));
        // Runs when this component is removed (a branch closes, a row disappears).
        on_cleanup(|| println!("gone"));

        text(move || format!("query: {}", query.get()))
    })
}

fn main() -> kivact::Result<()> {
    App::new("Effects").run(|| tracker(signal("rust".to_owned())))
}
```

- Use `effect` for side effects such as logging, saving or talking to something
  outside the UI. Do not use it to copy one signal into another to show a derived
  value: read the source inside a closure or a `memo` instead.
- `on_cleanup` called inside an effect runs before the effect re-runs and when
  it is removed, like the function a React effect returns.
- State belongs to the nearest *structural boundary*: the root, a `show` branch, a
  `for_each` row, a `provide` or an `error_boundary`. Signals created inside are
  freed when that boundary goes away. A plain function call does not create a
  boundary.

## Conditions, branches and lists

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    let logged_in = signal(false);
    let user = signal(None::<String>);
    let names = signal(vec!["Ann".to_owned(), "Bob".to_owned()]);

    column((
        // `<Show when=.. fallback=..>`
        show(logged_in, || text("Welcome back")).fallback(|| text("Please sign in")),
        // Option: the factory gets a `Value<String>` for the current value.
        show_some(user, |name| text(move || format!("Hello, {}", name.get()))),
        // `<For>`: rows keep their state by key.
        column(for_each(names, |name| name.clone(), |name| text(move || name.get()))),
        // `<Index>`: rows keep their state by position.
        column(for_index(names, |i, name| text(move || format!("{i}: {}", name.get())))),
        button("Toggle").on_press(move || logged_in.update(|v| *v = !*v)),
        button("Sign in as Ann").on_press(move || user.set(Some("Ann".to_owned()))),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Branches").run(app)
}
```

`show(condition, factory)` builds the branch when the condition is true and
throws it away (with its state) when it turns false. `condition` is a signal, a
memo or a closure returning `bool`. The factory returns a view and runs each time
the branch opens. [Lists and tables](lists-and-tables.md) goes into keys and row
state in detail.

## `dynamic` and match arms

Rust requires every arm of a `match` to have the same type. The element
functions (`text`, `column`, `node`) all return `Node`, so arms made only of those
agree, but a component such as `button(..)` returns its own builder type, and
mixing the two is a type error: `match arms have incompatible types`. Inside
`dynamic`, end each arm with `.into_view()` to erase the type:

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
enum Mode {
    Read,
    Edit,
}

fn editor() -> impl IntoView {
    let mode = signal(Mode::Read);
    column((
        button("Switch").on_press(move || {
            mode.update(|m| {
                *m = match m {
                    Mode::Read => Mode::Edit,
                    Mode::Edit => Mode::Read,
                }
            })
        }),
        // The source must be `Clone + PartialEq`. The view is rebuilt
        // when it changes to a different value.
        dynamic(mode, |mode| match mode {
            Mode::Read => text("Read only").into_view(),
            Mode::Edit => button("Save").into_view(),
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Modes").run(editor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_swaps_the_arms() {
        let screen = kivact::test::render(editor);
        screen.get_by_text("Read only");
        screen.get_by_role(Role::Button, "Switch").click();
        assert!(screen.query_by_role(Role::Button, "Save").is_some());
        assert!(screen.query_by_text("Read only").is_none());
    }
}
```

`.into_view()` is needed only for this shape: one `match` or `if` that returns
different kinds of view. Tuples (up to 16 children, each of its own type), `Vec`,
and the separate factories of `show` and `show_some` need no conversion. For two
branches prefer `show(..).fallback(..)`; reach for `dynamic` when you have three
or more or the kinds differ. `dynamic` rebuilds the view when the source changes,
so do not use it just to update a piece of text: use a closure.

## Context

`provide(value, || child)` makes `value` available to the child and everything
below it; `use_context::<T>()` reads the nearest one. `T` must be `Clone`, so provide
a struct of `Copy` handles, or an `Rc` for anything heavy.

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, Copy)]
struct Session {
    user: Signal<String>,
}

fn greeting() -> impl IntoView {
    let session = use_context::<Session>(); // panics, naming the type, if nothing provided it
    text(move || format!("Signed in as {}", session.get().user.get()))
}

fn app() -> impl IntoView {
    let session = Session { user: signal("ann".to_owned()) };
    provide(session, || column((greeting(), settings())))
}

fn settings() -> impl IntoView {
    let session = use_context::<Session>();
    button("Rename").on_press(move || session.get().user.set("ann2".to_owned()))
}

fn main() -> kivact::Result<()> {
    App::new("Context").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn children_share_the_provided_state() {
        let screen = kivact::test::render(app);
        screen.get_by_text("Signed in as ann");
        screen.get_by_role(Role::Button, "Rename").click();
        screen.get_by_text("Signed in as ann2");
    }
}
```

- There are no default values. `use_context::<T>()` panics with the type name when
  nothing was provided. When the value is optional, use
  `kivact::state::try_use_context::<T>()`, which returns an `Option<Value<T>>`.
- `provide` only affects its child. Siblings built after it do not see the value.
- Themes, locale, messages and reading direction have their own `provide_*`
  functions in `kivact::view` ([Theming](theming.md), [Internationalization](i18n.md)).

## Refs

- `var(value)` is a mutable box that does **not** notify anything when it changes,
  like `useRef(value).current`. It is also the way to share a non-`Copy` value
  (a path, a counter for ids) between several closures.
- `node_ref()` makes a handle you attach with `.node_ref(r)`; after mount,
  `r.focus()` focuses the element and `r.rect()` reads its box. It is not a
  DOM object you can modify: Kivact owns the elements.

```rust,no_run
use kivact::prelude::*;

fn search() -> impl IntoView {
    let query = signal(String::new());
    let input = node_ref();
    let searches = var(0_u32); // changing it does not redraw anything

    column((
        text_input("Search", query).node_ref(input),
        button("Search").on_press(move || {
            searches.update(|n| *n += 1);
            input.focus();
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Refs").run(search)
}
```

## The seven Rust points

### 1. Closures need `move`

A closure that Kivact stores (an event handler, a text closure) lives longer than
the function that created it. A plain `|| count.get()` would only *borrow*
`count` from a function that is about to return, and the compiler refuses:

```text
error[E0373]: closure may outlive the current function, but it borrows `count`,
which is owned by the current function
help: to force the closure to take ownership of `count`, use the `move` keyword
```

Write `move || count.get()`. `move` takes the variables into the closure; it does
not copy big data by itself. Stored closures must be `'static`, meaning they hold
no references to local variables.

### 2. Handles are `Copy`; data is cloned on request

`Signal<T>`, `Value<T>`, `Binding<T>`, `Var<T>` and `Callback<E>` are small `Copy`
handles into the app's state. Copy as many as you like into `move` closures;
they all point at the same value. You never write `.clone()` on a handle.

The data inside is a different story. `signal.get()` returns a **clone** of the
value, so it needs `T: Clone`. To look at a `String` or `Vec` without cloning,
borrow it: `names.with(|n| n.len())`, then return the result you need. Do not
write to the same signal from inside `with`.

### 3. `Clone + PartialEq` bounds

`set` only notifies readers when the new value differs, so it needs
`T: PartialEq`. `get` needs `T: Clone`. `memo`, `for_each` items and a
`dynamic` source also compare values. For your own types, derive both:

```rust,no_run
#[derive(Clone, PartialEq)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}
# fn main() {}
```

`signal(value)` itself, and `with` and `update`, work for any type.

### 4. `Option<T>` for "no value"

There is no `null`. A value that may be absent is an `Option<T>`: `Some(x)` or
`None`. Handle it with `match`, `if let`, `.map(..)` or `.unwrap_or(..)`, never a
blind `.unwrap()`. For views use `show_some(option_signal, |v| ..)`; its factory
receives a `Value<T>` for the current contents. `select` and `radio_group` edit an
`Option<K>` (nothing chosen is `None`); `.optional()` adapts a binding that holds
a plain value.

### 5. `String` and `&str`

`String` owns its text; `&str` borrows it. Props that take `impl IntoText` accept
both, so `text("Hello")` and `text(some_string)` work. A closure that returns text
may return a `&'static str` literal or a `String`:

```rust,no_run
use kivact::prelude::*;

fn status(saved: Signal<bool>, name: Signal<String>) -> impl IntoView {
    column((
        text(move || if saved.get() { "Saved" } else { "Not saved" }),
        text(move || format!("Name: {}", name.get())),
        // Return an owned value out of `with`; a borrow cannot leave it.
        text(move || name.with(|n| n.to_uppercase())),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Strings").run(|| status(signal(false), signal("ann".to_owned())))
}
```

A closure can not return a `&str` that points into a signal, because the borrow
ends with `with`. Return an owned `String` (`n.clone()`, `format!(..)`).

### Closure arguments sometimes need a type

Rust infers closure argument types from the function that receives them. For
`on_key(move |event| ..)` that works. For `callback(..)` and `action(..)`, where
the argument type is chosen by the closure itself, annotate it when you call a
method on it: `callback(move |word: String| len.set(word.len()))`. The compiler
says `type annotations needed` when you must.

### 6. A numeric closure must return the expected type

A constant integer can be accepted by a floating-point property, but a stored
closure must return that property's exact type. If `count` is a `Signal<i32>`,
`move || count.get()` cannot provide an `IntoValue<f64>`. Cast inside the closure:

```rust,no_run
use kivact::prelude::*;

fn count_slider(count: Signal<i32>) -> impl IntoView {
    let value = binding(
        move || count.get() as f64,
        move |next: f64| count.set(next.round() as i32),
    );
    slider("Count", value).range(0.0..=100.0)
}
```

For a plain numeric property, write `move || count.get() as f64`. Casting the
handle itself does not convert the values read later.

### 7. A local name can hide a prelude function

Rust uses the nearest binding for a name. After `let text = signal(String::new())`,
`text("Hello")` tries to call that signal and reports E0618, “expected function”.
Use names that describe the data: `message`, `rows`, `choices`, `name_field` or
`save_action`, leaving `text`, `row`, `list` and `field` available as UI functions.
The same rule applies to an imported type such as `Action`; avoid reusing it for
another item. If a name must stay, qualify the function, such as
`kivact::view::text("Hello")` or `kivact::ui::field("Name", input)`.

## See also

- [Tutorial](../getting-started/README.md): build an app step by step.
- [Lists and tables](lists-and-tables.md): keys, row state and large collections.
- [Forms](forms.md): the draft-and-validate pattern.
- [Run async work from a component](async-and-tasks.md): `resource` and `action`.
- [Write reusable components](custom-components.md): writing components other people use.
- Reference: `cargo doc -p kivact --open`.
