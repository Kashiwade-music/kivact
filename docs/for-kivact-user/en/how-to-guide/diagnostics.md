# Find out why something failed or updated

[日本語](../../jp/how-to-guide/diagnostics.md)

Use this guide when the window does not open, a button does nothing, an error
seems to vanish, or a value changes and you do not know who changed it. It
assumes you have read the getting-started tutorial.

## The shortest way: turn on tracing output

Kivact reports every problem it catches while the app runs through the
[`tracing`](https://docs.rs/tracing) crate, but it does not print anything
itself. Until you install a subscriber, a failing handler or a misused signal
is silent. Add one to your application:

```toml
[dependencies]
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

```text
fn main() -> kivact::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    App::new("Demo").run(app)
}
```

`tracing-subscriber` is your dependency, not Kivact's, so pick any subscriber
you like. With `EnvFilter` you choose what to see when you start the app:

```text
RUST_LOG=kivact=debug cargo run              # bash
$env:RUST_LOG = "kivact=debug"; cargo run    # PowerShell
```

Everything Kivact says uses the target `kivact`, except the layout lint, which
uses `kivact::layout`:

| Level | Target | What it says |
|---|---|---|
| `error` | `kivact` | An error reached the default sink: nothing handled it (see below). The line shows the error kind and the source location. |
| `warn` | `kivact` | Platform trouble, such as a background thread that outlived the window. |
| `warn` | `kivact::layout` | The layout lint: text that overflows, wraps or is cut off (debug builds only). |
| `info` | `kivact` | What the platform cannot do, once at start (`the platform lacks: ...`). |
| `debug` | `kivact` | Misuse that is ignored instead of failing, such as a write to a disposed signal. |
| `trace` | `kivact` | One line per turn (events, writes, re-runs, writes to the window) and the notes of the turn. |

Start with the default (errors only), then raise the level for `kivact` while
you chase one problem.

## What `App::run` returns

`App::run` returns when the window closes. It returns `Err` in only two cases,
both before your UI exists:

- `ErrorKind::Message`: a window setting is invalid, such as a zero size or a
  `min_window_size` larger than `window_size`.
- `ErrorKind::Platform`: the host could not start, for example because no window
  could be opened.

Every error carries a message and, when known, the place in your code that
caused it:

```rust,no_run
use kivact::ErrorKind;
use kivact::prelude::*;

fn main() {
    let result = App::new("Demo").window_size(800, 600).run(|| text("Hello"));
    if let Err(error) = result {
        match error.kind() {
            ErrorKind::Message => eprintln!("invalid window settings: {error}"),
            ErrorKind::Platform => eprintln!("no window could be opened: {error}"),
            _ => eprintln!("{error:?}"),
        }
        if let Some(at) = error.location() {
            eprintln!("raised at {at}");
        }
        std::process::exit(1);
    }
}
```

`ErrorKind` is `#[non_exhaustive]`, so a `match` needs a `_` arm. If you return
`kivact::Result<()>` from `main` instead, Rust prints the error with its
`Debug` form (`UiError(Message: ... at src/main.rs:12)`) and exits with status 1.

Errors inside the running app never make `run` return. They are handled as
described next.

## Where errors inside callbacks go

Kivact runs your code at a few places: component setup, event handlers
(`on_press`, `on_key`, ...), `effect`s, tasks, cleanups and `on_mount`. Each is
a boundary. What happens at a boundary depends on how the code fails:

- **A handler returns `Err`.** Handlers may return `()` or
  `Result<(), UiError>`, so `?` works on any `std::error::Error`. The error is
  reported with the location of the `?`.
- **The code panics.** A panic is a bug: Kivact does not catch it. It ends the
  App (or fails the test) with Rust's usual message.
- **A Kivact function is misused.** Calling `signal` inside a handler, writing a
  signal inside a `memo`, using a handle after its owner was disposed and so on
  are bugs too. Kivact panics at the call with
  `kivact misuse: UiError(<kind>: ... at file:line)` (see the table below).

A reported error goes to the nearest `error_boundary` above the failing code. It
runs its `on_error` handler, its `fallback`, or both. With no boundary, the error
reaches the **default sink**: a `tracing::error!` line and a ring buffer of the
last 256 errors. The app does not stop.

```rust,test_harness
use kivact::ErrorKind;
use kivact::prelude::*;
use kivact::view::error_boundary;

fn export() -> Result<(), UiError> {
    // `?` converts any `std::error::Error` into `UiError`.
    std::fs::read_to_string("does-not-exist.txt")?;
    Ok(())
}

fn app() -> impl IntoView {
    column((
        // No boundary: the error goes to the default sink.
        button("Export").on_press(export),
        // A boundary with a fallback replaces its child and offers a retry.
        error_boundary(|| {
            button("Risky").on_press(|| -> Result<(), UiError> {
                Err(UiError::msg("the disk is full"))
            })
        })
        .fallback(|error, reset| {
            column((
                text(format!("Failed: {error}")),
                button("Retry").on_press(move || reset.run()),
            ))
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Errors").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_reach_the_sink_or_the_boundary() {
        let screen = kivact::test::render(app);

        screen.get_by_role(Role::Button, "Export").click();
        let errors = screen.errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind(), &ErrorKind::External);
        assert!(errors[0].location().is_some());

        screen.get_by_role(Role::Button, "Risky").click();
        screen.get_by_text("Failed: the disk is full");
        // The boundary handled it, so the sink did not get a second error.
        assert_eq!(screen.errors().len(), 1);

        screen.get_by_role(Role::Button, "Retry").click();
        screen.get_by_role(Role::Button, "Risky");
    }
}
```

Three details are worth knowing:

- A fallback's `reset` function mounts a fresh copy of the child, so state
  created inside the boundary starts over.
- A boundary with only `on_error` keeps the child on screen and calls your
  function. A boundary with neither handler passes the error on to the parent
  boundary.
- A failed image or font is a diagnostic, not a failure: it goes to the default
  sink (`ErrorKind::Asset`) even inside a boundary, and the node draws nothing
  while the rest of the app goes on.

## Show recent errors in the running app

`kivact::diagnostics::errors()` returns the errors in the default sink's ring,
oldest first. Tests read the same ring with `Screen::errors()`. The result is a
snapshot that subscribes to nothing, so read it when you need it, for example
when a "Report a problem" dialog opens:

```rust,no_run
use kivact::prelude::*;

fn report_button() -> impl IntoView {
    let summary = signal(String::new());
    column((
        button("Report a problem").on_press(move || {
            let lines: Vec<String> = kivact::diagnostics::errors()
                .iter()
                .map(|error| format!("{:?}: {error}", error.kind()))
                .collect();
            summary.set(lines.join("; "));
        }),
        text(move || summary.get()),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Report").run(report_button)
}
```

Outside the App's own code (no App is running on the thread) the list is empty.

## Common errors and what to do

Match on `error.kind()` or read the tracing line. These are the kinds you will
meet most often.

| Kind | Cause | Fix |
|---|---|---|
| `SetupOnly { api }` | `signal`, `memo`, `effect`, `callback` and the other setup functions were called from a handler, a task or a memo. | Create the handle during component setup and use the handle in the handler. |
| `WriteInPureContext` | A signal was written inside a `memo` or inside a closure that computes a property (`text(move \|\| ...)`). | Derive the value instead of writing it. If you need a side effect, use `effect`. |
| `BorrowedWrite` | A signal was written (or read) inside its own `with` or `update` closure. | Return the decision from `with`, then write after it returns. |
| `Disposed { .. }` | A handle was used after the component that created it was removed. The error names where the handle and its owner were created. | Do not keep handles in places that outlive the component, or read with `try_get`. |
| `WrongApp` | A handle from another App, or from an App that no longer exists, was used. | Keep handles inside the App that created them. |
| `Cycle { path }` | Effects or memos depend on each other in a loop. `path` lists their creation sites. | Break the loop: let one side read with `get_untracked`, or merge the two. |
| `DuplicateKey` | Two rows of `for_each` returned the same key; the later row is dropped. | Return a unique key per item. |
| `Unsupported` | A setting does not apply to this kind of node or the platform. The message names it and the node is mounted without it. | Remove the setting or use the node it applies to. |
| `Platform` | A clipboard, dialog or focus request failed. | Handle the failure in the caller; it is a normal outcome on some systems. |
| `Asset` | An image or font could not be used. The message names the file and the reason. | Fix the path or the format. |

The misuse kinds (`SetupOnly`, `WriteInPureContext`, `BorrowedWrite`,
`Disposed`, `WrongApp` and a memo `Cycle`) appear in that panic message. The
`try_*` functions return `Disposed` and `WrongApp` as `Err` instead. Effects that
do not settle report `Cycle` to the sink.

One case is ignored instead of failing, and appears at `debug` level only:
`signal.set(..)` on a signal that no longer exists does nothing, and the write is
recorded as a note on the turn (`ErrorKind::WriteAfterDispose`). Keeping a signal
handle in a thread or a global and writing it after its component was removed
causes it. (`signal.update(..)` on such a signal has no value to hand to your
closure, so it fails with the same kind as an error.)

## Extra checks in debug builds

Debug builds (`cargo run` without `--release`) check more. These checks cost
nothing in release builds:

- `on_press` on a node that has no role and cannot take focus is reported once as
  `Unsupported`: a keyboard or screen-reader user could not reach it. Use
  `button(..)`, or add `.role(..)` and `.focusable(true)`.
- `icon("name")` with a name the theme's icon set does not contain is reported
  as `Asset` and draws nothing.
- The layout lint warns on target `kivact::layout` when a text overflows its box,
  wraps inside a single-line control such as a button, sits off center in a box
  that centers it, or is clipped without an ellipsis. The line names the node's
  source location.
- A style that sets the same property twice is noted on the turn (the last value
  wins).
- A write to a list item binding (`Signal::item`) after the item disappeared is
  recorded as a `WriteAfterDispose` diagnostic.

## Find out why a value updated

When a text shows a surprising value, or something changes that should not,
ask the node. `kivact::diagnostics::why` takes a `NodeRef` and explains, for each
dynamic property, which effect sets it and which signals and memos that effect
read, with the place where each was created and last written. Values themselves
are never recorded, only types and source locations.

```rust,no_run
use kivact::diagnostics::why;
use kivact::prelude::*;

fn app() -> impl IntoView {
    let count = signal(0);
    let doubled = memo(move || count.get() * 2);
    let label = node_ref();

    column((
        text(move || format!("doubled: {}", doubled.get())).node_ref(label),
        button("+1").on_press(move || count.update(|n| *n += 1)),
        button("Why?").on_press(move || {
            // `None` while the node is not mounted.
            if let Some(explanation) = why(&label) {
                eprintln!("{explanation}");
            }
        }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Why").run(app)
}
```

Click `+1` twice and then `Why?`. The report reads like this:

```text
node 5 Text "doubled: 4"
  Content set by the effect created at src/main.rs:10:9
    reads memo i32 created at src/main.rs:6:19
      reads signal i32 created at src/main.rs:5:17, last written at src/main.rs:12:48 (turn 2)
```

Read it from the top: the `Content` of the text is set by one effect; that effect
reads the memo, and the memo reads the signal, which was last written at the
line you probably wanted to see. A property that never changes does not appear
(`all props are static`). A node bound to no effect at all can only be updated by
something that creates the node again, such as a `show` branch or a list row.

To see the same causes in a running app, record with DevTools and read the
Updates panel; see
[See why the screen updated](devtools.md#see-why-the-screen-updated).

In a test you do not need a `NodeRef`: `Element::why()` does the same for any
element you found.

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    let count = signal(0);
    let doubled = memo(move || count.get() * 2);
    column((
        text(move || format!("doubled: {}", doubled.get())).test_id("doubled"),
        button("+1").on_press(move || count.update(|n| *n += 1)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Why").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_depends_on_the_memo_which_depends_on_the_signal() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "+1").click();
        screen.get_by_role(Role::Button, "+1").click();

        let why = screen.get_by_test_id("doubled").why();
        let content = &why.props[0];
        assert_eq!(content.prop, "Content");
        assert_eq!(content.reads[0].kind, "memo");
        let signal = &content.reads[0].reads[0];
        assert_eq!((signal.kind, signal.ty), ("signal", "i32"));
        assert!(signal.last_write.is_some());
    }
}
```

## Read what happened in a turn

Kivact processes input in turns: events arrive, handlers write signals, effects
and memos re-run, and the changes go to the window. In a test, `Screen::trace()`
returns the recent turns as `TurnRecord`s: what started the turn (`causes`),
every write that changed a value (`writes`, with the type and the line of the
write), which memos and effects re-ran (`reruns`), how many changes went to the
window (`ops`) and the duration. If a click causes more updates than you expect,
the writes and re-runs show which signal started them.

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    let count = signal(0);
    column((
        text(move || format!("count: {}", count.get())),
        button("+1").on_press(move || count.update(|n| *n += 1)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Trace").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_click_writes_the_counter_once() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "+1").click();

        let turns = screen.trace();
        let click = turns
            .iter()
            .find(|turn| turn.causes.iter().any(|cause| cause.starts_with("Activate")))
            .expect("the click started a turn");
        let writes: Vec<_> = click.writes.iter().filter(|w| w.ty == "i32").collect();
        assert_eq!(writes.len(), 1);
        assert!(click.reruns.iter().any(|run| run.kind == "render"));
        assert!(screen.errors().is_empty());
    }
}
```

The same turn records are written to tracing at `trace` level, so a running app
shows them too when you set `RUST_LOG=kivact=trace`.

`Screen::stats()` returns counters of nodes, owners and timers. They exist for
changes to Kivact itself. Use `errors()`, `trace()` and the queries to test your
app, not the counters.

## Common mistakes behind "nothing happens"

- **The text does not update.** `text(count.get())` reads the signal once during
  setup and shows that snapshot. Pass a closure: `text(move || count.get())`, or
  a signal or memo itself.
- **A component runs twice.** Copying one signal into another with an effect
  (`effect(move || other.set(count.get()))`) makes two sources of truth. Derive
  the second value with `memo`.
- **A write fails while reading.** `signal.with(|v| signal.set(..))` writes inside
  the borrow. Compute the new value inside `with`, return it, and write after.
- **An error never shows.** No subscriber is installed. See the first section.

## See also

- [Test your app](testing.md) for `Screen`, queries and the layout check that
  fails a test on a layout problem.
- [Write reusable components](custom-components.md) for components that report
  their own misuse.
- `cargo doc -p kivact --open`: `kivact::diagnostics` (`ErrorKind`, `why`,
  `TurnRecord`) and `kivact::test`.
