# Map what you know from React or SolidJS to Kivact

[日本語](../../jp/how-to-guide/coming-from-react-or-solid.md)

If you know React or SolidJS, most Kivact concepts have a direct counterpart.
This guide maps them one to one, explains the single big difference (components
run once, like Solid's, not on every state change like React's), and covers the
Rust details that trip up developers who have not written Rust before.

It assumes you have done the [tutorial](../getting-started/README.md).

## The one idea: setup runs once

A Kivact component is a function marked `#[component]`. Its body runs **once**,
when the component appears on screen. It creates state, wires things together
and returns a view. After that, the body never runs again. When a signal
changes, only the pieces of the screen that read it are updated.

```tsx
// React: the whole function runs again on every click.
function Counter() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(count + 1)}>count: {count}</button>;
}
```

```rust,test_harness
use kivact::prelude::*;

// Kivact: the body runs once; only the `text` closure runs again.
#[component]
fn counter() -> impl Element {
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
  it inside a closure Kivact runs again when `n` changes. The same goes for
  `.disabled(..)`, `.style(..)`, `.name(..)` and every other property that can
  change: give them a closure, a signal or a memo.
- **There is no dependency array.** Anything you read with `.get()` or `.with()`
  inside a closure, memo or effect is tracked automatically.

## Concept map

| Idea | React | SolidJS | Kivact |
| --- | --- | --- | --- |
| Component | function returning JSX | function returning JSX | `#[component] fn card(..) -> impl Element`, with or without props |
| Props | props object | props object | arguments of the component: `Value<T>` for a value that may change, `Signal<T>` for a value the component writes, `View` for children, `Callback<E>` for handlers; required ones are positional, `Option<..>` ones become setters |
| Children | `children` | `props.children` | a `View` prop (one slot), `Vec<View>` (several), or `ViewFn` (built again on every opening) |
| Callback prop | `onSave` prop | `onSave` prop | a `Callback<()>` prop takes a closure or a `Callback<()>`; a `Callback<E>` prop takes a closure `move \|e\| ..` |
| Shared handler | `useCallback` | a function | `callback(move \|_\| ..)` makes a `Copy` `Callback`; pass it to several handlers |
| State machine | `useReducer` plus effects | reducer plus effects | `Machine`: explicit states, events, effects and timers |
| State | `useState` | `createSignal` | `signal(value)`; `get`, `with`, `set`, `update` on one `Signal<T>` |
| Two-way value | `value` + `onChange` | `value` + `onInput` | pass a `Signal<T>`; `bind!(form.name)` for one field of a struct; `binding(get, set)` for a converted value |
| Derived value | compute in render, `useMemo` | `createMemo` | a closure for one use, `memo(..)` to share or cache |
| Effect | `useEffect` | `createEffect` | `effect(..)`; `effect_on(source, f)` to skip the first run |
| Cleanup | `useEffect` return value | `onCleanup` | `on_cleanup(..)` |
| Run once after mount | `useEffect(.., [])` | `onMount` | `on_mount(..)` |
| Mutable value, no re-render; an app object | `useRef` | a plain variable | `var(value)`: a `Copy` box the screen does not watch |
| A change with no value | a counter in state | `createTrigger` | `trigger()`: `track()` and `notify()` |
| Is this row selected? | `selected === id` in every row | `createSelector` | `kivact::state::selector(source)`, then `is(&key)` in each row |
| Element ref | `useRef` on a DOM node | `ref` | `node_ref()` and `.node_ref(r)`; `r.focus()` |
| Context | `createContext`, `useContext` | `createContext`, `useContext` | `provide(value, \|\| child)` and `use_context::<T>()`, which returns the value |
| Portal | `createPortal(child, el)`; `window.open` + a portal for a new window | `<Portal>` | `overlay(..)` in the same window; `window(title, open)` for an OS window |
| Conditional | `cond && <A/>` | `<Show when>` | `show(cond, \|\| a)`, `show_some(option, \|v\| a)` |
| Several branches | ternary or `switch` | `<Switch>` | `dynamic(source, \|v\| match v { .. })` |
| List | `items.map(..)` with `key` | `<For>` | `for_each(items, \|i\| key, \|item\| row)` |
| List by position | `key={index}` | `<Index>` | `for_index(items, \|i, item\| row)` |
| Key | `key` prop | the `For` identity | the key function you give `for_each` |
| Error boundary | `ErrorBoundary` | `ErrorBoundary` | `error_boundary(..)` |
| Data loading | libraries or `useEffect` | `createResource` | `resource(source, fetch)`; `action(f)` for user actions |
| Loading and error UI | `<Suspense>` and `<ErrorBoundary>` | the same | `async_view(resource, ready).loading(..).fallback(..)` |
| Data from another thread | `postMessage` from a worker | the same | `kivact::task::remote(initial)`: a `Signal` and a `Send` sender |
| Shared element between two places (Framer Motion `layoutId`) | `layoutId` (Framer Motion) | a library | `.layout_id("id")` on both nodes; the new one glides from the old box with a layout transition in its style |
| Old content leaves before new content enters (`AnimatePresence mode="wait"`) | `AnimatePresence mode="wait"` | a library | `show(cond, \|\| a).fallback(\|\| b).presence(PresenceMode::Wait)`; `Sync` and `PopLayout` are the other modes |
| A value that follows scrolling (`useScroll`) | `useScroll` | a scroll listener | `scroll_progress(scroller, Orientation::Vertical)`, or `view_progress(target, scroller, axis, ViewRange::Entry)` for one element |
| A sequence on one clock (GSAP `timeline`) | `gsap.timeline()` | a library | `timeline()`, `track(..)`, `to(track, target, transition).at(At::AfterPrevious(ms))`, `play()` |
| A delay per list item (GSAP `stagger`) | `stagger` | a library | `Stagger::new(ms).from(StaggerFrom::Center)` and `stagger.delay(index, count)` |
| CSS `var(--accent)` | `var(--accent)` | the same | `color::ACCENT`, `space::MD`, `radius::MD`: theme variables you put anywhere a value goes |
| CSS `:hover`, `:active`, `:focus-visible` | the same | the same | `style().hover(..)`, `.active(..)`, `.focus_visible(..)` |
| CSS `transform: perspective(..) rotateX(..)` | `style={{ transform }}` | `style={{ transform }}` | `style().transform(Transform::new().perspective(..).rotate_x(..))` |
| CSS `filter: blur(4px)` | `style={{ filter }}` | `style={{ filter }}` | `style().filter(Filter::new().blur(4))` (`kivact::style`) |
| CSS `mask`, `clip-path`, `mix-blend-mode` | `style={{ mask }}` and so on | the same | `style().mask(..)`, `.clip_path(shape)`, `.mix_blend_mode(BlendMode::Multiply)` |
| CSS `font-variation-settings`, `font-stretch` | `style={{ fontVariationSettings }}` | the same | `style().font_variation_settings([("wght", 650.0)])`, `.font_width(75.0)` |
| CSS `-webkit-text-stroke`, `paint-order` | `style={{ WebkitTextStroke }}` | the same | `style().text_stroke(width, color)`, `.paint_order(..)` |

The ordinary names in the table come with `use kivact::prelude::*;`, including
`callback`, `trigger`, `error_boundary` and `effect_on`. Import `selector` from
`kivact::state`, `remote` and `remote_trigger` from `kivact::task`, and, for a
state machine, `Machine`, `Step` and `machine` from `kivact::state`.

`Machine` is `useReducer` with effects and timers. Define explicit states and
events, return a `Step` from `transition`, and execute effects in the runner.
See [custom controls](custom-controls.md#write-behavior-with-a-state-machine).

## Props, children and callbacks

A component's props are its arguments; Kivact writes the props builder from the
signature.

```rust,test_harness
use kivact::prelude::*;

// `Value<String>`: a literal, a String, a signal or a closure that returns text.
// `Value<i32>`: a number, a signal, a memo or a closure.
#[component]
fn stat(label: Value<String>, value: Value<i32>) -> impl Element {
    row((text(label), text(move || value.get().to_string()))).style(style().gap(8))
}

// `View`: any view, including another component. `Option<View>`: an optional slot.
#[component]
fn panel(title: Value<String>, child: View, footer: Option<View>) -> impl Element {
    column((text(title), child, footer)).style(style().gap(8).padding(12))
}

// A callback prop: the caller passes a closure or a `Callback`.
#[component]
fn buttons(on_add: Callback<()>) -> impl Element {
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
    .footer(text("Counts start again when the app restarts."))
    .style(style().width(320))
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
        screen.get_by_text("Counts start again when the app restarts.");
        screen.get_by_role(Role::Button, "Add again").click();
        screen.get_by_text("1");
        screen.get_by_text("2");
    }
}
```

- A prop that may change is `Value<T>`. The caller passes a constant, a
  signal, a memo or a closure, and the body gets a `Value<T>` to read with
  `.get()` or `.with(..)`.
- A callback prop is `Callback<E>`. For an action without input
  (`Callback<()>`), the caller passes a closure or a `Callback<()>` it already
  has. For a callback with input, the caller passes a closure; a shared
  `Callback<E>` goes in as `move |e| cb.call(e)`. The body gets a `Copy`
  handle, so both buttons can use the same action; hand a `Callback<()>` to any
  `on_press`, or call any callback with `.call(event)`.
- An `Option<..>` prop is optional: the caller sets it with a setter of the
  same name, `.footer(..)`. `#[prop(default = expr)]` gives a prop a default
  value instead.
- Wherever a closure returning `()` is accepted, a closure returning
  `Result<(), UiError>` is accepted too, so you can use `?` inside a handler.
  The error reaches the nearest error handler.
- A component returns `impl Element`, so its caller can add `.style(..)`,
  `.name(..)` or `.test_id(..)`, as `app` does on `panel`. The caller's style
  comes after the component's own and wins.
- A component can have type parameters (`fn picker<K: Clone + PartialEq + 'static>(..)`).
  Where React would take any child, take a `View`, a `Vec<View>` or a `Value<T>` prop.

## Effects, cleanup and mount

```rust,no_run
use kivact::prelude::*;

#[component]
fn tracker(query: Signal<String>) -> impl Element {
    // Runs once the screen is built, then again whenever `query` changes.
    effect(move || {
        println!("searching for {}", query.get());
    });
    // Runs once, after the nodes of this component exist.
    on_mount(|| println!("mounted"));
    // Runs when this component is removed (a branch closes, a row disappears).
    on_cleanup(|| println!("gone"));

    text(move || format!("query: {}", query.get()))
}

fn main() -> kivact::Result<()> {
    App::new("Effects").run(|| tracker(signal("rust".to_owned())))
}
```

- Use `effect` for side effects such as logging, saving or talking to something
  outside the UI. Do not use it to copy one signal into another to show a derived
  value: read the source inside a closure or a `memo` instead.
- `on_cleanup` called inside an effect runs before the effect runs again and
  when it is removed, like the function a React effect returns.
- State belongs to the nearest *boundary*: a component, the root, a `show`
  branch, a `for_each` row, a `provide` or an `error_boundary`. Signals created
  inside are freed when that boundary goes away.

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
agree, but a component such as `button(..)` returns its own type, and mixing the
two is a type error: `match arms have incompatible types`. Inside `dynamic`, end
each arm with `.into_view()` to erase the type:

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
enum Mode {
    Read,
    Edit,
}

#[component]
fn editor() -> impl Element {
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
or more or the kinds differ. For loading, error and ready, use `async_view`
rather than a `dynamic` over the state. `dynamic` rebuilds the view when the
source changes, so do not use it just to update a piece of text: use a closure.

## Context

`provide(value, || child)` makes `value` available to the child and everything
below it; `use_context::<T>()` returns the nearest one. `T` must be `Clone`, so
provide a struct of `Copy` handles, or an `Rc` for anything heavy.

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, Copy)]
struct Session {
    user: Signal<String>,
}

#[component]
fn greeting() -> impl Element {
    let session = use_context::<Session>(); // panics, naming the type, if nothing provided it
    text(move || format!("Signed in as {}", session.user.get()))
}

#[component]
fn settings() -> impl Element {
    let session = use_context::<Session>();
    button("Rename").on_press(move || session.user.set("ann2".to_owned()))
}

fn app() -> impl IntoView {
    let session = Session { user: signal("ann".to_owned()) };
    provide(session, || column((greeting(), settings())))
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

- `use_context` returns the provided value itself, like Solid's `useContext`.
  The value does not change; put signals inside it, as `Session` does, for
  state that changes.
- There are no default values. `use_context::<T>()` panics with the type name when
  nothing was provided. When the value is optional, use
  `kivact::state::try_use_context::<T>()`, which returns an `Option<T>`.
- `provide` only affects its child. Siblings built after it do not see the value.
- Themes, locale, messages and reading direction have their own `provide_*`
  functions: `provide_theme`, `kivact::i18n::provide_locale`,
  `kivact::i18n::provide_messages` and `kivact::view::provide_direction`
  ([Theming](theming.md), [Internationalization](i18n.md)).

## Refs and app objects

- `var(value)` is a mutable box that does **not** notify anything when it
  changes, like `useRef(value).current`. It is also how you keep an object of
  your app that is not `Copy` (a history, a parser, a connection) in a `Copy`
  handle that any number of closures can share.
- `node_ref()` makes a handle you attach with `.node_ref(r)`; after mount,
  `r.focus()` focuses the element and `r.rect()` reads its box. It is not a
  DOM object you can modify: Kivact owns the elements.

```rust,no_run
use kivact::prelude::*;

/// An object of the app that is not `Copy`.
struct History {
    entries: Vec<String>,
}

#[component]
fn search() -> impl Element {
    let query = signal(String::new());
    let input = node_ref();
    let history = var(History { entries: Vec::new() }); // changing it redraws nothing

    column((
        text_input(query).label("Search").node_ref(input),
        button("Search").on_press(move || {
            history.update(|h| h.entries.push(query.get()));
            input.focus();
        }),
        button("Print history").on_press(move || {
            history.with(|h| println!("{} searches", h.entries.len()));
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Refs").run(search)
}
```

## Values from another thread

A worker thread of your own (an audio engine, a file watcher, a network client)
cannot touch a `Signal`: signals live on the UI thread. `remote(initial)` gives
you a signal and a sender that can go to any thread.

```rust,no_run
use std::time::Duration;

use kivact::prelude::*;
use kivact::task::remote;

#[component]
fn ticks_from_a_thread() -> impl Element {
    // `ticks` is an ordinary signal; `sender` is `Send` and goes to the thread.
    let (ticks, sender) = remote(0u64);
    std::thread::spawn(move || {
        for n in 1.. {
            std::thread::sleep(Duration::from_millis(500));
            sender.set(n);
        }
    });
    text(move || format!("ticks: {}", ticks.get()))
}

fn main() -> kivact::Result<()> {
    App::new("Remote").run(ticks_from_a_thread)
}
```

Writes are merged: the UI sees the latest value once, at the start of its next
turn, however often the thread writes. When the component is removed, the
sender closes and later values are dropped. When the thread only has to say "something changed",
`remote_trigger()` gives a `Trigger` for the UI and a `Send` sender whose
`notify()` never blocks. See
[Run async work from a component](async-and-tasks.md#receive-values-from-another-thread).

## Save what the user is still typing

A text field with `.on_commit(..)` writes its value when the user presses Enter
or leaves the field. A keyboard shortcut does not leave the field, so before
saving, ask the focused control to commit with `commit_focused_edit()`:

```rust,no_run
use kivact::prelude::*;
use kivact::view::commit_focused_edit;

#[component]
fn document() -> impl Element {
    let title = signal(String::from("Untitled"));
    let saved = signal(String::new());
    let save = action(move || async move {
        // Commit the field the user is still typing in, then read the state.
        commit_focused_edit().await;
        saved.set(title.get());
        Ok(())
    });
    column((
        text_input(title)
            .label("Title")
            .on_commit(move |value: String| title.set(value)),
        text(move || format!("Saved: {}", saved.get())),
    ))
    .on_key(move |event: KeyEvent| {
        if event.key() == Key::Char('s') && event.modifiers().ctrl {
            save.run(());
            Handled::Yes
        } else {
            Handled::No
        }
    })
}

fn main() -> kivact::Result<()> {
    App::new("Document").run(document)
}
```

## The Rust points

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

`Signal<T>`, `Value<T>`, `Var<T>`, `Callback<E>` and `Trigger` are small `Copy`
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

`signal(value)` itself, and `with` and `update`, work for any type. To pass a
constant of your own type to a `Value<T>` prop, also derive `Prop`
(`#[derive(Clone, PartialEq, Prop)]`); see
[Write reusable components](custom-components.md#use-your-own-type-as-a-prop).

### 4. `Option<T>` for "no value"

There is no `null`. A value that may be absent is an `Option<T>`: `Some(x)` or
`None`. Handle it with `match`, `if let`, `.map(..)` or `.unwrap_or(..)`, never a
blind `.unwrap()`. For views use `show_some(option_signal, |v| ..)`; its factory
receives a `Value<T>` for the current contents. `select` and `radio_group` edit an
`Option<K>` (nothing chosen is `None`); `.optional()` turns a `Signal<T>` into a
`Signal<Option<T>>` for them.

### 5. `String` and `&str`

`String` owns its text; `&str` borrows it. Text arguments accept both, so
`text("Hello")` and `text(some_string)` work. A closure that returns text may
return a `&'static str` literal or a `String`:

```rust,no_run
use kivact::prelude::*;

#[component]
fn status(saved: Signal<bool>, name: Signal<String>) -> impl Element {
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
`on_key(move |event| ..)` and a `Callback<E>` prop that works. For `callback(..)`
and `action(..)`, where the argument type is chosen by the closure itself,
annotate it when you call a method on it:
`callback(move |word: String| len.set(word.len()))`. The compiler says
`type annotations needed` when you must.

### 6. A numeric closure must return the expected type

A constant integer can be accepted by a floating-point property, but a stored
closure must return that property's exact type. If `count` is a `Signal<i32>`,
`move || count.get()` cannot provide an `f64`. Cast inside the closure. When
the error names a trait, check the closure's return type first:

```rust,no_run
use kivact::prelude::*;

#[component]
fn count_slider(count: Signal<i32>) -> impl Element {
    let value = binding(
        move || count.get() as f64,
        move |next: f64| count.set(next.round() as i32),
    );
    slider(value).label("Count").range(0.0..=100.0)
}
```

`binding(get, set)` returns a `Signal<f64>` that reads and writes through the two
closures. For a plain numeric property, write `move || count.get() as f64`.
Casting the handle itself does not convert the values read later.

### 7. A local name can hide a prelude name

Rust uses the nearest binding for a name. After `let text = signal(String::new())`,
`text("Hello")` tries to call that signal and reports E0618, “expected function”.
Use names that describe the data: `label`, `rows`, `choices`, `name_field` or
`save_action`, leaving `text`, `row`, `list`, `field`, `message` and `window`
available as prelude functions. The theme variable modules `color`, `space`,
`radius` and `font` are prelude names too: `let color = ..` hides `color::ACCENT`,
so call it `tint`. `window` is a common local name (`let window = …`): call it
`settings_window` instead. Name a row callback's argument after its data too,
such as `item` or `todo`, rather than `row`. The same rule applies to an imported
type such as `Action`; avoid reusing it for another item. If a name must stay,
qualify it, such as `kivact::view::text("Hello")`,
`kivact::ui::field("Name", input)` or `kivact::task::Action`.

```rust,no_run
use kivact::prelude::*;

#[component]
fn heading_row(label: Value<String>) -> impl Element {
    row((text(label), button("Open")))
}
```

## See also

- [Tutorial](../getting-started/README.md): build an app step by step.
- [Lists and tables](lists-and-tables.md): keys, row state and large collections.
- [Forms](forms.md): the draft-and-validate pattern.
- [Run async work from a component](async-and-tasks.md): `resource`, `action` and `remote`.
- [Write reusable components](custom-components.md): writing components other people use.
- [Migrate an app to the reworked API](migrating-to-rework.md): old names and their replacements.
- Reference: `cargo doc -p kivact --open`.
