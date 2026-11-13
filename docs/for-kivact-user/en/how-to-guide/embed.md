# Open a Kivact UI inside a plugin window

[日本語](../../jp/how-to-guide/embed.md)

> **Experimental.** The `embed` feature works on Windows and on Linux with X11. It
> has not been tested in the major plugin hosts yet, macOS and Wayland are not
> supported, and the API may change without notice.

Use this guide to show a Kivact UI inside a window that another program gives you,
which is the case for the editor of an audio plugin (CLAP or VST3). Your UI stays
ordinary Kivact code: components, signals, bindings and `kivact::test`. Kivact owns the
window contents and the UI; the plugin owns audio, parameters, saving and the glue to
the plugin format.

Two people usually share the work:

| Role | Writes | Needs from this guide |
|---|---|---|
| **UI author** | The editor screen, as a component. | [Write the UI](#write-the-ui-ui-author) |
| **Wrapper author** | The plugin glue that opens the editor in the host's window and closes it. | [Open the editor](#open-the-editor-wrapper-author) |

Kivact ships no adapter for a plugin framework (nih-plug, clack, ...). The wrapper
author converts the framework's window handle into a `Parent` and calls
`App::embed`.

## Enable the feature

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["embed"] }
```

The default `gpu` feature stays on: embedded windows use the GPU with the same
automatic CPU fallback as a desktop window.

## Write the UI (UI author)

An editor is a function that returns a view, exactly like the root of a desktop app.
Nothing in it mentions the host window, the plugin format or the parent handle. You
write it, test it with `kivact::test::render` and run it in a normal window with
`App::run` while you develop. The wrapper author's `App::embed` call is what places
it in the host.

Two rules differ from a desktop app:

- **Keep plugin state outside the UI.** The editor is created every time the host
  opens it, and dropped when the host closes it. Parameters, presets and anything that
  must survive belong to the plugin. Pass the editor an `Arc` of that state. Selection,
  hover and scroll position may stay in signals; they start over each time.
- **Do not wait for the host's GUI thread.** The editor's code runs on the UI thread
  of the App, which may be another thread than the host's. A callback that blocks on
  the host can deadlock opening or closing.

### Read and write parameters

Define what the editor needs as a trait. The wrapper implements it with the plugin
format's calls, and a test implements it with a fake:

```rust,test_harness
use std::sync::Arc;

use kivact::motion::FrameRate;
use kivact::prelude::*;

/// What the editor needs from the plugin. The wrapper implements it.
pub trait Params: Send + Sync + 'static {
    fn gain(&self) -> f64;
    fn begin_gesture(&self);
    fn set_gain(&self, value: f64);
    fn end_gesture(&self);
}

pub fn editor(params: Arc<dyn Params>) -> impl IntoView {
    let value = signal(params.gain());

    // Follow what the host changes (automation, presets) once per display frame.
    let source = params.clone();
    frame_loop(move |_| value.set(source.gain())).start(FrameRate::Display);

    // Writes go to the plugin first, then to the signal that the screen reads.
    let sink = params.clone();
    let gain = binding(
        move || value.get(),
        move |next: f64| {
            sink.set_gain(next);
            value.set(next);
        },
    );

    // A drag or a key press is one gesture for the host's undo and automation.
    let gestures = params;
    column((
        slider("Gain", gain)
            .range(0.0..=1.0)
            .step(0.1)
            .on_edit(move |phase| match phase {
                EditPhase::Begin => gestures.begin_gesture(),
                EditPhase::Commit | EditPhase::Cancel => gestures.end_gesture(),
            }),
        text(move || format!("Gain: {:.2}", value.get())),
    ))
    .style(style().gap(8).padding(16))
}

fn main() {}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::Duration;

    use super::*;

    /// A stand-in for the plugin: it records what the editor does to it.
    #[derive(Default)]
    struct Fake {
        gain: Mutex<f64>,
        log: Mutex<Vec<String>>,
    }

    impl Params for Fake {
        fn gain(&self) -> f64 {
            *self.gain.lock().unwrap()
        }
        fn begin_gesture(&self) {
            self.log.lock().unwrap().push("begin".into());
        }
        fn set_gain(&self, value: f64) {
            *self.gain.lock().unwrap() = value;
            self.log.lock().unwrap().push(format!("set {value:.1}"));
        }
        fn end_gesture(&self) {
            self.log.lock().unwrap().push("end".into());
        }
    }

    #[test]
    fn the_editor_writes_gestures_and_follows_the_host() {
        let fake = Arc::new(Fake::default());
        let params = fake.clone();
        let screen = kivact::test::render(move || editor(params));

        screen.get_by_role(Role::Slider, "Gain").press_key(Key::ArrowRight);
        screen.get_by_text("Gain: 0.10");
        assert_eq!(*fake.log.lock().unwrap(), ["begin", "set 0.1", "end"]);

        // The host automates the parameter; the editor catches up on a frame.
        *fake.gain.lock().unwrap() = 0.5;
        screen.advance(Duration::from_millis(100));
        screen.get_by_text("Gain: 0.50");
    }
}
```

The order of the calls is `Begin`, one or more writes, then `Commit`. When the user
presses Escape or the pointer is canceled, the value is written back to where it
started first, and `Cancel` arrives after that. So on `Cancel`, only end the gesture,
and do not write the value again. Do not start a gesture for a change that came from
the host.

### Receive notifications instead of polling

If the plugin can notify the editor, such as through a channel, receive them in a task.
The task belongs to the component and is canceled when the editor closes. Polling in a
`frame_loop` costs one read per frame; a channel costs nothing while nothing changes.

```rust,no_run
use kivact::prelude::*;

fn gain_label(incoming: async_channel::Receiver<f64>) -> impl IntoView {
    let value = signal(0.0);
    spawn(async move {
        while let Ok(next) = incoming.recv().await {
            value.set(next);
        }
    });
    text(move || format!("Gain: {:.2}", value.get()))
}
```

Any channel with an async receiver works, `async_channel` here is only an example.
Never block the audio thread on the UI: sending into a bounded or lock-free queue is the
wrapper's job.

### What differs from a desktop window

- **Title and fullscreen do nothing.** The host owns the window. `App::title` is
  ignored and `platform::set_fullscreen` fails as unsupported.
- **Size.** The host decides. The App's `window_size` is only the first size.
  `min_window_size` limits what your own `platform::request_window_size` asks for, not
  what the host gives you. To ask the host for another size, see
  [Resize](#resize-in-both-directions); without the wrapper's help the request fails
  as `Unsupported`.
- **Close.** There is no close button to react to. The wrapper drops the editor when the
  host closes the window. `AppHandle::close` reports `ErrorKind::Unsupported` in an
  embedded App; drop the `Embedded` handle instead.
- **One window.** The host owns the one window the editor occupies.
  `platform::capabilities().get().multi_window` is false, and a `window` declaration
  reports `ErrorKind::Unsupported` and stays closed (see
  [Where windows are not available](windows.md#where-windows-are-not-available)).
- **No input injection.** DevTools can inspect the editor, but `inject_input` is
  refused with `UNSUPPORTED` (-32015); the embed ports keep their own input
  state.
- **Keys.** The editor receives the keys the host's window gives it. By default a
  key the editor does not handle stays there; `Parent::forward_unhandled_keys`
  (see [Keys and the host](#keys-and-the-host)) gives it back to the host. Kivact
  has no workaround for a host that steals keys such as Space before the editor
  sees them.
- **Panics stop the editor.** A panic ends the editor and is reported by
  `Embedded::status` as `ErrorKind::Panicked`; it never unwinds into your
  wrapper, so a C-ABI host does not abort. The handle's methods keep
  returning normally, and dropping it closes what is left. Handle failures
  that you expect with an `error_boundary` (see
  [Find out why something failed or updated](diagnostics.md)).

## Open the editor (wrapper author)

The wrapper turns the plugin format's calls into calls on Kivact. The entry point is
`App::embed(parent, root)`, which returns an `Embedded` handle. Dropping the handle
closes the editor.

### Make the parent

A `Parent` stores a raw handle value and a scale. Making one never reads the handle;
`App::embed` validates it.

| What the host gives you | Construct |
|---|---|
| CLAP `clap_window.win32`, VST3 `"HWND"`, nih-plug `Win32Hwnd` | `Parent::win32(hwnd)` |
| CLAP `clap_window.x11`, VST3 `"X11EmbedWindowID"`, nih-plug `X11Window` | `Parent::x11(id)` |

```rust,no_run
use std::ffi::c_void;

use kivact::embed::Parent;

fn windows_parent(hwnd: *mut c_void, scale: f64) -> Parent {
    Parent::win32(hwnd).scale(scale)
}

fn x11_parent(window: u32, scale: f64) -> Parent {
    Parent::x11(window).scale(scale)
}
```

If the host tells you its scale (CLAP `set_scale`, VST3 `setContentScaleFactor`), pass it
with `.scale(..)`. Without it Kivact uses the OS scale. When you computed a size from
your own scale, give Kivact the same scale, or the sizes disagree.

### Open

```rust,no_run
use std::sync::Arc;

use kivact::embed::{Embedded, Parent};
use kivact::prelude::*;

# pub trait Params: Send + Sync + 'static {}
# fn editor(_: Arc<dyn Params>) -> impl IntoView { text("Editor") }
fn open(parent: Parent, params: Arc<dyn Params>) -> kivact::Result<Embedded> {
    App::new("Editor")
        .window_size(640, 480)
        .min_window_size(320, 240)
        .embed(parent, move || editor(params))
}
```

`window_size` is in **logical pixels**; multiplied by the scale it gives the first
physical size. Without it the default is 960 by 720.

The contract of `embed`:

- **Thread.** Call it on the host's GUI thread. The root closure must be `Send`, and
  it runs on the App's UI thread. It runs once, and `embed` returns after it finished.
- **Errors.** Invalid sizes or scale return `ErrorKind::Message`. An invalid or
  null parent, a missing OS resource or an unsupported OS return `ErrorKind::Platform`.
  A panic in the root or while opening returns `ErrorKind::Panicked`; a later
  panic stops the editor and is reported by `Embedded::status`. Kivact never
  unwinds into the caller. A failed open leaves no window.
- **The handle.** `Embedded` is `Send + Sync`. Keep it for as long as the editor is
  open; keep it behind an `Arc` if several threads call its setters.

### Respond to the host

Setters queue a change for the UI thread and return at once; getters return the last
applied value. All sizes at this boundary are **physical pixels**; the scale is
physical pixels per logical pixel.

| The host calls (CLAP / VST3) | The wrapper does |
|---|---|
| `set_scale` / `setContentScaleFactor` before opening | Store the scale. |
| `get_size` / `getSize` before opening | Return `round(logical size * scale)`. |
| `set_parent` / `attached` | Pass the same scale in `Parent`, call `App::embed`, keep the handle. |
| `show` / `hide` (CLAP `gui.show` / `gui.hide`) | `Embedded::show()` / `Embedded::hide()`; `Embedded::is_visible()` reads the applied state. |
| `set_size` / `onSize` | `Embedded::set_size(width, height)`; this also answers a pending resize request. |
| `get_size` / `getSize` after opening | `Embedded::size()` |
| the scale changes | `Embedded::set_scale(scale)`, then ask the host for the new physical size if you want to keep the logical size. |
| `adjust_size` / `checkSizeConstraint` | Round to the wrapper's own limits; the host's `set_size` is not limited by `min_window_size`. |
| `destroy` / `removed` | Drop the handle. |
| the OS changes the display scale (Windows) | `Parent::on_notice` gives `EmbedNotice::ScaleChanged(scale)`; read `scale()` and keep the logical size by asking the host for `round(logical size * scale)`. |
| the editor stopped (parent destroyed, failure, panic) | `Parent::on_notice` gives `EmbedNotice::Stopped`; read `status()`. |

```rust,no_run
use kivact::embed::Embedded;

/// The part of a wrapper that holds an open editor.
pub struct OpenEditor {
    embedded: Option<Embedded>,
}

impl OpenEditor {
    pub fn set_size(&self, width: u32, height: u32) {
        if let Some(editor) = &self.embedded {
            editor.set_size(width, height);
        }
    }

    pub fn size(&self) -> Option<(u32, u32)> {
        self.embedded.as_ref().map(Embedded::size)
    }

    pub fn set_scale(&self, scale: f64) {
        if let Some(editor) = &self.embedded {
            editor.set_scale(scale);
        }
    }

    /// Why the editor stopped, if it is open: `Running`, `Closed` (the host
    /// destroyed the parent), `Failed(error)` or `Panicked(error)`.
    pub fn status(&self) -> Option<kivact::embed::EmbedStatus> {
        self.embedded.as_ref().map(Embedded::status)
    }

    pub fn close(&mut self) {
        // Dropping closes the editor and waits for its cleanup.
        self.embedded = None;
    }
}
```

A size of zero, a size beyond the platform limit (`i32::MAX` on Windows, `u16::MAX` on
X11) and a scale that is not finite and positive are ignored with a warning in
`tracing`; the editor keeps running.

### Receive notices instead of polling

`Parent::on_notice` wakes the wrapper when the operating system changes the
applied scale or when the editor stops. A notice is a wake-up, not a state: read
`status()` or `scale()` to learn what happened. Until you have a callback, poll
`status()` from the host's idle timer (CLAP `timer-support`, the VST3 run loop).

```rust,no_run
use std::sync::Arc;

use kivact::embed::{EmbedNotice, Parent};
use kivact::embed::EmbedStatus;

/// The part of a wrapper that reacts to the editor.
#[derive(Clone)]
pub struct Wake {
    /// Post a message to the host's main thread (any channel works).
    post: Arc<dyn Fn() + Send + Sync>,
}

fn notices(parent: Parent, wake: Wake) -> Parent {
    parent.on_notice(move |notice| {
        // Do not block here: only ask the host to run later.
        keep_logical_size(notice);
        let post: &(dyn Fn() + Send + Sync) = &*wake.post;
        post();
    })
}

/// Whether the host must keep the logical size by asking for a new physical size.
fn keep_logical_size(notice: EmbedNotice) -> bool {
    matches!(notice, EmbedNotice::ScaleChanged(_))
}

/// On the host's main thread, after the wake arrives: whether the editor runs.
fn alive(status: EmbedStatus) -> bool {
    status == EmbedStatus::Running
}
```

The callback must not wait for the host's GUI thread and must not panic. On X11
it can run from the editor thread while it exits; a close started by dropping
the handle can run on the thread that drops it. `ScaleChanged` is sent only on
Windows: X11 has a single scale (Xft), so a monitor change does not change it.

### Show and hide

CLAP calls `gui.show` and `gui.hide` after `set_parent`; VST3 and standalone
windows open shown. Open with `Parent::hidden()` when the host starts the editor
hidden, then call `Embedded::show`/`Embedded::hide`. Both are idempotent and
queued like the other setters. A hidden editor runs its App (tasks, timers,
signals), but draws nothing: frame loops do not tick and
`platform::is_surface_active()` is false until `show`. Hiding returns the
keyboard focus to the parent and cancels a pressed pointer. A queued call is not
reflected by `is_visible` until the UI thread applies it.

```rust,no_run
use kivact::embed::{Embedded, Parent};

fn hidden_until_shown(parent: Parent) -> Parent {
    parent.hidden()
}

fn show(editor: &Embedded) {
    editor.show();
}
```

### Keys and the host

A key press is **unhandled** when no focused text input, focus movement, IME or
copy/paste step, `on_key` handler (a failing handler counts as handled) or
default action (Enter/Space activation, Escape dismissal) used it. A release
follows its press.

By default the editor keeps unhandled keys. `Parent::forward_unhandled_keys`
gives them and their releases back to the parent window: on Windows Kivact posts
the original `WM_KEYDOWN`/`WM_KEYUP` to the parent (as JUCE editors do); on X11
it sends a synthetic `KeyPress`/`KeyRelease` (`XSendEvent`; some toolkits ignore
synthetic events). A host that wants its Space shortcut back opts in; the editor
keeps handling text input and its own shortcuts first.

```rust,no_run
use kivact::embed::Parent;

fn forwarding(parent: Parent) -> Parent {
    parent.forward_unhandled_keys()
}
```

The verdict is synchronous: Kivact probes each native press (delivers it and
runs the turn before the native message returns), so the host receives the key
in the same order the user pressed it. Kivact does not probe while App code is
already on the stack (a modal dialog opened by an App handler); that press is
not forwarded. A wrapper must not wait for the host's GUI thread inside its
callbacks.

### Resize in both directions

When the user or the host resizes the window, call `set_size`. When the UI wants a
different size, it calls `platform::request_window_size` with a **logical** size. The
request reaches the wrapper through `Parent::on_resize_request`, in **physical**
pixels, on the UI thread. Return what the host did:

| Return | Meaning | Then |
|---|---|---|
| `ResizeAnswer::Accepted` | The host accepted now (CLAP `request_resize` returned true). | Kivact resizes the window itself; a later `set_size` still wins. The UI's request succeeds at once. |
| `ResizeAnswer::Pending` | The host decides later and confirms with `set_size` (VST3 `resizeView` → `onSize`). | The window keeps its size. The request succeeds when the host's size equals the request, fails as a platform error naming the host's size when it differs, is cancelled by a newer request, or times out after two seconds. |
| `ResizeAnswer::Refused` | The host refused. | The request fails as a platform error. |

Without the callback, the request fails as `Unsupported`.

```rust,no_run
use kivact::embed::{Parent, ResizeAnswer};

fn resizable(parent: Parent) -> Parent {
    parent.on_resize_request(|width, height| {
        // Ask the host for width x height physical pixels. This runs on the UI
        // thread: post the request to the host's own run loop and answer without
        // waiting for its GUI thread.
        let _ = (width, height);
        ResizeAnswer::Pending
    })
}

// In the UI, in a task or an effect:
async fn fit_content() -> kivact::Result<()> {
    kivact::platform::request_window_size(800, 600).await
}
```

Answering the wrapper's own size question needs one order: **store the new logical size,
ask the host, restore the old size if the host refuses.** Some frameworks (nih-plug)
read the editor's size again while the request is running.

With `Pending`, the child does not resize before the host confirms: call
`Embedded::set_size` from the host's size callback (`onSize`, or on X11 after the
posted CLAP `request_resize` returns true). The request resolves from that call. A
host that answers `Pending` and never confirms makes the UI request time out after
two seconds; the child stays at its old size, which is the safe side.

To keep the logical size when the scale changes, call `set_scale` first, then ask the
host for `round(logical size * new scale)`.

### Close

Drop the `Embedded` on the host's GUI thread. Outside an App callback, the drop waits
for the App and its native window to clean up, for at most two seconds per thread the
App owns. Work that outlives that limit continues after the drop returns, and Kivact
keeps its own code mapped so the host can unload the plugin safely. You do not need to
retain the module yourself. Threads you start yourself are yours to stop and join.

If the editor drops its own handle from inside an App callback (a "Close" button, say),
the current turn finishes, cleanup follows, and no later input is delivered. Queued
actions are not drained.

If the host destroys the parent window first, the editor stops safely and
`status()` returns `EmbedStatus::Closed`. A platform failure stops the App,
destroys its window and makes `status()` return `EmbedStatus::Failed` with the
reason. A panic stops it the same way and makes `status()` return
`EmbedStatus::Panicked` with the panic's message; no call and no drop resumes
it. You still drop the handle later. Long-running `blocking` work should check
its `CancelToken` regularly so that closing does not wait for it.

### Test the wrapper

With the `test` feature, `kivact::test::TestParent` opens a real parent window so you
can embed, pump its message loop and capture what the editor presented. It exists on
Windows and Linux.

```rust,no_run
# #[cfg(any(windows, target_os = "linux"))]
# fn example() -> kivact::Result<()> {
use std::time::Duration;

use kivact::embed::Embedded;
use kivact::prelude::*;
use kivact::test::TestParent;

let parent = TestParent::open(320, 240)?;
let editor: Embedded = App::new("Editor")
    .window_size(320, 240)
    .embed(parent.parent().scale(1.0), || text("Editor"))?;
parent.pump(Duration::from_millis(50));
assert_eq!(parent.capture()?.width, 320);
assert_eq!(editor.status(), kivact::embed::EmbedStatus::Running);
drop(editor);
# Ok(())
# }
```

## What works

| Capability | Windows | Linux X11 |
|---|---|---|
| Rendering (GPU, CPU fallback), mouse, wheel, keyboard | yes | yes (needs `DISPLAY`; XWayland works) |
| IME | yes, including reconversion | XIM; plain keyboard input where no input method runs; no reconversion |
| Screen readers | yes | yes (AT-SPI) |
| Clipboard | yes | yes |
| File and folder dialogs | yes | yes (the child window is the dialog's parent) |
| Dropping files on the editor | yes, drop only (no hover or cancel) | yes (XDND), drop only |
| Touch and pen | no | no |
| Additional windows | no | no |
| DevTools input injection | no (`UNSUPPORTED`) | no (`UNSUPPORTED`) |
| Fullscreen, window title | no effect | no effect |
| Largest size (physical) | `i32::MAX` | `u16::MAX` |
| macOS (NSView) | not supported | |
| Wayland (native embedding) | | not supported |
| Giving unhandled keys back to the host | opt-in (`Parent::forward_unhandled_keys`) | opt-in, host-dependent (`XSendEvent`) |
| Showing and hiding the editor | yes (`Embedded::show`/`hide`, `Parent::hidden`) | yes |
| Resize requests confirmed by the host | yes (`ResizeAnswer::Pending`) | yes |
| Operating-system scale changes | `EmbedNotice::ScaleChanged` | never (Xft has one scale) |

Real plugin hosts (REAPER, Bitwig Studio, Ableton Live, Cubase and others), an IME in a
real DAW and screen readers inside a plugin window have not been verified.

## Themes in an embedded editor

`App::theme` accepts a fixed theme for embedding. A caller-thread reactive theme
returns `UiError::Unsupported`. For a changing theme, create its signal and
`provide_theme` inside the `Send` root factory passed to `embed`. In desktop
`run` and tests, `App::theme` evaluates reactive themes under the root owner.

## See also

- [Build controls with your own look](custom-controls.md) for `EditPhase` and the
  controls that report gestures.
- [Run async work from a component](async-and-tasks.md) for `spawn` and `blocking`.
- [Find out why something failed or updated](diagnostics.md) for how panics end the
  App.
- `cargo doc -p kivact --features embed --open`: `kivact::embed`.
