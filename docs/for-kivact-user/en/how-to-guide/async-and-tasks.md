# Run async work from a component

[日本語](../../jp/how-to-guide/async-and-tasks.md)

Use this guide to load data, save in the background, wait on a timer, run
heavy work off the UI thread and receive values from other threads, without
freezing the window. It assumes you have read the getting-started tutorial.

The tools live in `kivact::task`. Four of them are enough for almost
everything:

| You want to | Use |
|---|---|
| load data that depends on a signal, and show loading, error and ready | `resource` |
| run something when the user acts (save, submit, retry) | `action` |
| start a future that has no result to show (a timer, a channel reader) | `spawn` |
| run blocking or CPU-heavy code | `blocking` |

`resource`, `action` and `spawn` are also in `kivact::prelude`; `blocking` is in `kivact::task`.

## Load data and show loading, error and ready

`resource(source, fetch)` calls `fetch` once at start and again whenever
`source` changes. Whatever `fetch` returns is turned into an `AsyncState`:
`Idle`, `Pending`, `Ready(value)` or `Failed(error)`. Start with `async_view`: it
shows loading and retryable errors for you, and its callback builds the ready view.

```rust,no_run
use std::time::Duration;
use kivact::prelude::*;

fn app() -> impl Element {
    let id = signal(1u32);
    let user = resource(id, |id| async move {
        kivact::task::sleep(Duration::from_millis(300)).await;
        if id == 0 {
            Err(UiError::msg("There is no user 0"))
        } else {
            Ok(format!("User #{id}"))
        }
    });

    column((
        row((
            button("Previous")
                .disabled(move || id.get() == 0)
                .on_press(move || id.update(|n| *n -= 1)),
            button("Next").on_press(move || id.update(|n| *n += 1)),
        ))
        .style(style().gap(8)),
        async_view(user, |name: String| text(name)),
    ))
    .style(style().padding(24).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("Resource").run(app)
}
```

A few rules make this behave well:

- **The source is anything a property accepts.** Pass a `Signal`, a closure
  such as `move || (query.get(), page.get())`, or a constant. The source is
  the only thing `resource` tracks. Signals you read inside `fetch` or inside
  the future are *not* dependencies.
- **The fetch function returns `Result<T, UiError>`.** `Err(UiError::msg("…"))`
  makes a failure from text, and `?` converts any `std::error::Error`
  (an `io::Error`, a parse error) into a `UiError`. `T` must be `Clone`,
  and so must the source value, because each request gets its own copy.
- **Only the latest request counts.** If the source changes while a request is
  in flight, the old future is dropped and its result is never shown. You
  never have to guard against a slow, stale response overwriting a newer one.
- **Failures are data, not panics.** A failed fetch lands in
  `AsyncState::Failed`. It does not reach an `error_boundary`. Show it
  through `async_view`, or match the state in `dynamic` for a custom presentation.

Rust note: `async move` moves the captured values into the future, which
outlives the closure call. Signals are `Copy` handles, so moving them is free.
A `String` you still need elsewhere has to be cloned first.

### Keep the old result visible while refreshing

`latest()` returns the last successful value. It stays available while a new
request is `Pending` and after a failure, so you can show stale data with a
spinner on top instead of blanking the screen. Combine `show` and `show_some`,
one per concern:

```rust,no_run
# use std::time::Duration;
# use kivact::prelude::*;
#[component]
fn search() -> impl Element {
    let query = signal(String::new());
    let hits = resource(query, |word| async move {
        kivact::task::sleep(Duration::from_millis(200)).await;
        if word.contains('!') {
            return Err(UiError::msg("Search words cannot contain !"));
        }
        Ok(format!("Results for “{word}”"))
    });

    column((
        text_input(query).label("Search"),
        show(move || hits.state().is_pending(), || text("Searching…")),
        show_some(move || hits.error(), |error| {
            text(move || error.get().to_string()).role(Role::Alert)
        }),
        show_some(move || hits.latest(), |latest| text(latest)),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Search").run(search) }
```

Other `Resource` methods you will use:

- `refetch()` fetches again with the current source (a "Reload" button).
- `cancel()` drops the request in flight. The state becomes `Failed` with an
  error whose `is_cancelled()` is `true`, and `latest()` keeps the old value.
- `state().value()` and `state().error()` read the value or error of the
  current state without a `match`.

## Run something when the user acts

A `resource` runs because its source changed. An `action` runs because you
called `run`. Use it for saving, submitting a form, deleting, or anything with
a side effect. It has the same `AsyncState` and the same `latest()`.

```rust,test_harness
use std::time::Duration;
use kivact::prelude::*;

#[component]
fn save_form() -> impl Element {
    let name = signal(String::new());
    let save = action(|name: String| async move {
        kivact::task::sleep(Duration::from_millis(300)).await;
        if name.is_empty() {
            Err(UiError::msg("Name is empty"))
        } else {
            Ok(name)
        }
    });

    column((
        text_input(name).label("Name"),
        button("Save")
            .disabled(move || save.state().is_pending())
            .on_press(move || save.run(name.get())),
        dynamic(move || save.state(), |state| match state {
            AsyncState::Idle => text(""),
            AsyncState::Pending => text("Saving…"),
            AsyncState::Ready(name) => text(format!("Saved {name}")),
            AsyncState::Failed(error) => text(format!("Failed: {error}")).role(Role::Alert),
        }),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Action").run(save_form)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saving_shows_pending_then_the_result() {
        let screen = kivact::test::render(save_form);
        screen.get_by_role(Role::TextBox, "Name").type_text("Ada");
        screen.get_by_role(Role::Button, "Save").click();
        screen.get_by_text("Saving…");
        assert!(screen.get_by_role(Role::Button, "Save").is_disabled());
        // The test clock is virtual: no real waiting.
        screen.advance(Duration::from_millis(300));
        screen.get_by_text("Saved Ada");
    }
}
```

A new `run` while one is still going cancels the old one (see below). The closure
takes the input (`|name: String|`) or no input at all (`|| async { … }`, run
with `save.run(())`). Rust cannot infer an argument's type before it sees how
you use it, so write the type when you call a method on the input.

Create `resource` and `action` while the component is being set up, not inside
an event handler. Inside handlers, call `run`, `refetch` and `cancel` on the
handles you already have.

### What happens when the user clicks twice

Choose the constructor by what a new `run` does to one that is still going:

| Constructor | Behaviour |
|---|---|
| `action(f)` | The new run starts at once and cancels the old one. |
| `queued_action(f)` | Runs go one at a time, in order; every run is kept. |

```rust,no_run
# use kivact::prelude::*;
# use kivact::task::queued_action;
#[component]
fn drafts() -> impl Element {
    let draft = signal(String::new());
    let save = queued_action(|text: String| async move {
        kivact::task::sleep(std::time::Duration::from_millis(500)).await;
        Ok(text.len())
    });
    let save = save.on_complete(|result: Result<usize, UiError>| match result {
        Ok(count) => println!("saved: {count}"),
        Err(error) => eprintln!("save failed: {error}"),
    });
    column((
        text_input(draft).label("Draft"),
        // Pressing this ten times quickly saves ten times, one after another.
        button("Save draft").on_press(move || save.run(draft.get())),
    ))
}
# fn main() -> kivact::Result<()> { App::new("Drafts").run(drafts) }
```

An action belongs to the component that created it. When that component is
removed (its screen closes, its row disappears), its runs are cancelled, queued
ones included. For a save that must finish even when the user leaves the screen,
create the action in the root function and hand it to the screens through
context. Closing the app waits up to three seconds for the runs still queued.

```rust,no_run
use std::time::Duration;

use kivact::prelude::*;
use kivact::task::queued_action;

/// The app's saves. The root creates them, so closing a screen does not cancel them.
#[derive(Clone, Copy)]
struct Saves {
    document: Action<usize, String>,
}

#[component]
fn editor_screen() -> impl Element {
    let save = use_context::<Saves>().document;
    let draft = signal(String::new());
    column((
        text_input(draft).label("Draft"),
        button("Save").on_press(move || save.run(draft.get())),
    ))
}

fn app() -> impl IntoView {
    let saves = Saves {
        document: queued_action(|text: String| async move {
            kivact::task::sleep(Duration::from_millis(500)).await;
            Ok(text.len())
        }),
    };
    let editing = signal(true);
    provide(saves, move || {
        column((
            button("Close editor").on_press(move || editing.set(false)),
            show(editing, editor_screen),
        ))
    })
}

fn main() -> kivact::Result<()> {
    App::new("Saves").run(app)
}
```

Other `Action` methods: `retry()` re-runs the input of the latest run,
`cancel()` stops the run in flight, `input()` returns the input of the run
the state describes, and `on_complete(|result| …)`, which gets a
`Result<T, UiError>`, registers a handler that runs when a run finishes (not
when it is cancelled). You can register several.

## Start a future that has no result to show

`spawn` runs any future on the UI thread, owned by the component it was called
from. Call it from setup, an event handler or another task.

```rust,no_run
use std::time::Duration;
use kivact::prelude::*;
use kivact::task::sleep;

#[component]
fn copy_with_confirmation() -> impl Element {
    let copied = signal(false);
    column((
        button("Copy").on_press(move || {
            spawn(async move {
                kivact::platform::copy_text("hello").await?;
                copied.set(true);
                sleep(Duration::from_secs(2)).await;
                copied.set(false);
                Ok::<(), UiError>(())
            });
        }),
        show(copied, || text("Copied")),
    ))
}
# fn main() -> kivact::Result<()> { App::new("Spawn").run(copy_with_confirmation) }
```

- **Signals can be written directly after an `await`.** The future runs on the
  UI thread, so there is no hand-off step.
- **A future may return `()` or `Result<(), UiError>`.** An `Err` is reported
  like an error returned from an event handler: it goes to the nearest
  `on_error` of the component that owns the task. Wrap part of the UI in
  `kivact::view::error_boundary(..).on_error(..)` to catch it.
- `spawn` returns a `Task`. Keep it only if you want to call `cancel()` or ask
  `is_finished()`.
- `kivact::task::sleep(duration)` waits on the app's clock (virtual in tests,
  see below). `kivact::task::timeout(duration, future)` gives
  `Err(UiError)` of kind `ErrorKind::Timeout` if the future is not done in time.

A repeating job is a loop with `sleep`. Even a loop that never ends on its own
stops when its owner goes away:

```rust,no_run
# use std::time::Duration;
# use kivact::prelude::*;
#[component]
fn countdown() -> impl Element {
    let remaining = signal(10u32);
    spawn(async move {
        while remaining.get() > 0 {
            kivact::task::sleep(Duration::from_secs(1)).await;
            remaining.update(|s| *s -= 1);
        }
    });
    text(move || format!("{} s left", remaining.get()))
}
# fn main() -> kivact::Result<()> { App::new("Countdown").run(countdown) }
```

To refresh a resource on a timer, call `refetch()` in such a loop.

## Keep blocking and CPU-heavy work off the UI thread

Async code runs on the UI thread. An `await` lets other work run in between, but
a long calculation, a blocking file read, a synchronous network call or
`std::thread::sleep` does not yield: the window stops painting and answering
input until it returns. Put that work in `blocking`.

```rust,no_run
use kivact::prelude::*;
use kivact::task::CancelToken;

/// Counts primes below `limit`; stops early when `cancel` turns true.
fn count_primes(limit: u32, cancel: &CancelToken) -> Option<u32> {
    let mut count = 0;
    for n in 2..limit {
        if n % 4096 == 0 && cancel.is_cancelled() {
            return None;
        }
        if (2..).take_while(|d| d * d <= n).all(|d| n % d != 0) {
            count += 1;
        }
    }
    Some(count)
}

#[component]
fn primes() -> impl Element {
    let limit = signal(200_000u32);
    let result = resource(limit, |limit| async move {
        // `limit` is a plain number here, so it can move to another thread.
        let count = kivact::task::blocking(move |cancel: CancelToken| count_primes(limit, &cancel)).await;
        count.ok_or_else(|| UiError::msg("Stopped"))
    });
    column((
        button("Double the limit").on_press(move || limit.update(|n| *n *= 2)),
        show(move || result.state().is_pending(), || text("Counting…")),
        show_some(move || result.latest(), |n| text(move || format!("{} primes", n.get()))),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("Primes").run(primes) }
```

`blocking(f)` runs `f` on a small thread pool and returns a future for the
result. You `await` it inside a resource, an action or a spawned task.

- **The closure must be `Send`.** It runs on another thread, so it cannot
  capture a `Signal`, `Value` or any other UI handle. The compiler tells you
  so. Read the value *outside* the closure and move the plain value in
  (`limit` above), then write the result back after the `await`.
- **A panic in the closure** is resumed on the UI thread by the task that awaits
  it; it ends the App.
- **Cancellation is cooperative.** When the future is dropped (a newer request
  replaced it, or its component was disposed) the closure is not killed. Its
  `CancelToken` turns true, and long work should check it now and then and
  return early. Take the token as a parameter: `|cancel: CancelToken| …`
  (the type annotation is required).
- The pool defaults to `min(4, CPUs)` threads. `App::blocking_threads(n)`
  changes it.
- At shutdown the app waits up to two seconds per worker, then gives up on
  jobs that have not stopped.

## Receive values from another thread

If your own code produces values in the background (a game or audio engine, a
file watcher, a network client on its own runtime), give it the sender of a
`remote` signal. `remote(initial)` returns a `Signal<T>` for the UI and a
`Remote<T>` that is `Send`, so it can go to any thread.

```rust,no_run
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use kivact::prelude::*;
use kivact::task::remote;

#[component]
fn sensor() -> impl Element {
    let (reading, sender) = remote(0u32);
    let stop = Arc::new(AtomicBool::new(false));

    // Your own thread, started in setup.
    let stopped = stop.clone();
    std::thread::spawn(move || {
        for n in 0.. {
            if stopped.load(Ordering::Relaxed) {
                break;
            }
            sender.set(n);
            std::thread::sleep(Duration::from_millis(500));
        }
    });
    // Tell the thread to end when this component is removed.
    on_cleanup(move || stop.store(true, Ordering::Relaxed));

    text(move || format!("reading: {}", reading.get()))
}
# fn main() -> kivact::Result<()> { App::new("Sensor").run(sensor) }
```

`set` wakes the UI, so values show up even when the user is not touching the
window. Writes are merged: the UI sees the latest value once, at the start of
its next turn, however often the thread writes. `set` is the only operation:
compute the new value in your thread and send it whole. When the component is
removed the sender closes, and a later `set` drops the value; stop the thread
with your own flag, as `on_cleanup` does here.

Do not block the UI thread waiting for a worker, and do not hold a `Mutex` that
a worker also takes for a long time. For continuous pictures from your own
renderer, see [Draw with a canvas](canvas.md#show-frames-from-your-own-renderer).

Futures that bring their own reactor (for example `tokio` timers and sockets)
expect to run inside that runtime. Run such a runtime on its own thread and
send the results through a `remote` sender as above.

### End the app from a worker

A worker can stop the UI with an `AppHandle`, without `std::process::exit`.
Take the handle before the run with `App::handle()`, move a clone into the
thread, and call `close()` when the session ends:

```rust,no_run
use kivact::prelude::*;

fn main() -> kivact::Result<()> {
    let app = App::new("Session").window_size(800, 500);
    let handle = app.handle();
    std::thread::spawn(move || {
        // Run the session here; on a failure or Ctrl+C:
        handle.close();
    });
    app.run(|| text("Running"))
}
```

The next turn begins the exit, and `run` returns `Ok(())` after the normal
cleanup. The close cannot be cancelled and runs no `on_dismiss`; see
[Close the app from code](windows.md#close-the-app-from-code).

## Wake the UI without queueing values

When the worker only has to say "something changed, read the latest", it does not
need to send a value at all. `remote_trigger()` returns a `Trigger` for the UI
and a `RemoteTrigger` for the worker: `notify()` never blocks, and several
notifications before the next turn count as one.

```rust,no_run
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kivact::prelude::*;
use kivact::task::remote_trigger;

#[component]
fn level_meter() -> impl Element {
    let shared = Arc::new(Mutex::new(0u32));
    let (changed, wake) = remote_trigger();

    // Your own thread, started in setup.
    let latest = shared.clone();
    std::thread::spawn(move || {
        for n in 0.. {
            *latest.lock().expect("invariant: the level lock is not poisoned") = n;
            wake.notify();
            std::thread::sleep(Duration::from_millis(100));
        }
    });

    text(move || {
        changed.track();
        let level = *shared.lock().expect("invariant: the level lock is not poisoned");
        format!("level: {level}")
    })
}
# fn main() -> kivact::Result<()> { App::new("Level").run(level_meter) }
```

`notify` never waits, so the producer never blocks on a busy UI thread. It wakes
the UI's event loop, which is a system call on most platforms: from a real-time
audio callback, set your own lock-free flag and notify from a non-real-time
thread.

## What cancels a task

A task, a resource request and a running action belong to the component that
created them (the nearest *owner*: a `show` branch, a list row, the root). When
that part of the UI is removed:

- the future is dropped, so it never resumes after its next `await`;
- its `sleep` timers are released;
- a `blocking` job's `CancelToken` turns true;
- an action's runs are cancelled, queued ones included (an action created by the
  root lives as long as the app; closing the app waits up to three seconds for
  its queued runs).

So you do not need cleanup code for ordinary async work. If you start something
that is not a Kivact task (a thread, a file handle), release it with
`on_cleanup(move || …)`.

## Test async UI without waiting

Tests use a virtual clock. `screen.advance(Duration)` moves it forward and runs
every timer on the way, and `screen.find_by_text("…")` waits (in virtual time)
for text to appear. The action example above shows both. See
[Test your app](testing.md#time-and-async-work).

## See also

- [Draw with a canvas](canvas.md): showing frames from a worker.
- [Animate values and elements](animation.md): per-frame work and transitions.
- [Use platform services](platform-services.md): clipboard and dialogs return futures too.
- [Test your app](testing.md): the virtual clock in detail.
- API reference: `cargo doc -p kivact --open`, then `kivact::task`.
