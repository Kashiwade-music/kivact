# Show a plugin editor in its own window

[日本語](../../jp/how-to-guide/foreign-windows.md)

A DAW usually shows a plugin editor in its own small top-level window. Use
`foreign_window(title, open)` for that: Kivact opens an ordinary window of the
App whose whole content area is one native child window. Another library draws
the plugin editor into that child. Kivact draws only the theme background, so
the two never paint over each other.

This guide assumes a host app that owns the plugin instance and calls its GUI
functions. The plugin wrapper decides what `open_editor`, `set_editor_size`
and `close_editor` do (`vivace-host` does this for CLAP and VST3).

## Declare the window

`foreign_window` is a view, like `window`. Put it where the editor may open,
next to the operation that sets `open`:

```rust
use std::sync::Arc;

use kivact::platform::NativeWindow;
use kivact::prelude::*;
use kivact::view::ForeignChild;
use kivact::view::SurfaceSize;
use kivact::view::foreign_window;

struct Editor;

impl Editor {
    fn open(&self, _parent: NativeWindow, _size: SurfaceSize) {}
    fn resize(&self, _size: SurfaceSize) {}
    fn close(&self) {}
}

fn plugin_window(editor: Arc<Editor>, open: Signal<bool>) -> impl IntoView {
    let resize_editor = editor.clone();
    let close_editor = editor.clone();
    foreign_window("Plugin editor", open)
        .size(480.0, 320.0)
        .min_size(320.0, 200.0)
        .mode(WindowMode::Tool)
        .on_ready(move |child: ForeignChild| {
            // Give the plugin the child as its parent and its physical size.
            editor.open(child.window(), child.size());
        })
        .on_resize(move |size: SurfaceSize| {
            // The user resized the window, or it moved to another monitor.
            resize_editor.resize(size);
        })
        .on_destroy(move || {
            // Detach the plugin's GUI now: the child still exists here.
            close_editor.close();
        })
}
```

While `open` is true the window exists. Writing false closes it; the title
bar's close calls `on_dismiss` and, unless that returns
`Handled::Yes`/`Handled::PreventDefault`, writes `open` false. The window also
closes with the owner that declared it (for example when a surrounding `show`
branch disappears) and when the App ends.

## The three callbacks

| Callback | When it runs | What to do |
| --- | --- | --- |
| `on_ready(child)` | Once per opening, after the child exists | Open the plugin GUI with `child.window()` as its parent. `child.size()` is the physical size at that moment. |
| `on_resize(size)` | The child's physical size or scale changed | Send the new physical size and scale to the plugin. `size.logical` is the logical content size, `size.physical_width`/`height` the physical one. |
| `on_destroy()` | Once per opening, before the child and its window are destroyed | Close the plugin GUI and drop what it owns. |

`on_ready` and `on_resize` take an ordinary Kivact handler, so they may return
errors; a returned `UiError` reaches the App's error handler. `on_destroy` is
a plain `Fn()`.

The guarantee of `on_destroy` covers every normal close path: writing `open`
false, the title bar's close, disposing the declaring branch, and the App
ending (including `AppHandle::close`). A platform failure while the window
opens, or a panic, destroys the window without running it; treat that as the
one exception.

## Plugin-initiated resize

When the plugin asks for a new size, call `ForeignChild::request_size`. It
takes physical pixels, because that is what plugin APIs report, and resizes
the window so the child gets exactly that size. The request is clamped to
`min_size`; a refusal is a platform error. After the window closed, the future
resolves to `ErrorKind::Closed`.

```rust
use kivact::prelude::*;
use kivact::view::ForeignChild;
use kivact::view::foreign_window;

fn resizer(child: Signal<Option<ForeignChild>>, wanted: Signal<Option<(u32, u32)>>) {
    effect(move || {
        let Some((width, height)) = wanted.get() else {
            return;
        };
        let Some(child) = child.get_untracked() else {
            return;
        };
        spawn(async move {
            let _ = child.request_size(width, height).await;
        });
    });
}
```

Keep the handle in a signal from `on_ready`:

```rust,ignore
use kivact::view::ForeignChild;
use kivact::view::foreign_window;

let child = signal(None::<ForeignChild>);
foreign_window("Plugin editor", open).on_ready(move |handle| {
    child.set(Some(handle));
});
```

A fixed-size editor cannot be resized by the user. Use
`.resizable(false)`; `request_size` still works for the sizes the plugin
itself asks for. Bind `resizable` to a signal to follow the plugin:

```rust,ignore
use kivact::view::foreign_window;

let can_resize = signal(true);
foreign_window("Plugin editor", open).resizable(can_resize);
```

## Scale changes

`on_resize` is called for a monitor move as well: the new `SurfaceSize` carries
the new `scale_factor`. Pass it and the physical size to the plugin (many
wrappers have one `set_editor_size`/`set_editor_scale` pair). On X11 the
system's monitor scale is not reported, so only user resizes and
`request_size` arrive there.

## Keyboard focus

The child is a real OS child window. Clicking the plugin's controls gives the
plugin's own windows the keyboard; Kivact does not route keys into the plugin.
When the foreign window is activated, Kivact gives the focus to the child's
first child window, which is what most plugin editors expect. A plugin with
several top-level child windows may need one click before it takes keys.

## Platforms and capability

Foreign windows exist on Windows (Win32) and Linux X11. On Wayland, in an
embedded App, and in a test host without the capability, `foreign_window`
reports `ErrorKind::Unsupported` once and stays closed; `open` is not changed.
`platform::capabilities().foreign_window` says whether the running host can
open one.

## Test it

The test host opens virtual foreign windows when the test asks for the
capability:

```rust
use kivact::prelude::*;
use kivact::test::RenderOptions;

let options = RenderOptions::default().foreign_windows();
```

`Screen::foreign_child()` returns the native handle, `Screen::is_resizable()`
reports the current size lock, and the usual `resize`, `set_scale` and
`request_close` work on the window. `kivact::test::is_native_window_alive`
answers whether a handle is still alive; use it in `on_destroy` to prove the
child outlives the plugin's GUI.

```rust,ignore
use std::cell::Cell;
use std::rc::Rc;

use kivact::prelude::*;
use kivact::test::RenderOptions;

let alive = Rc::new(Cell::new(None::<bool>));
let seen = alive.clone();
let (screen, ()) = kivact::test::render_with_handle(
    RenderOptions::default().foreign_windows(),
    move || {
        let open = signal(true);
        (
            (),
            column((
                text("host"),
                foreign_window("Plugin editor", open)
                    .on_ready(move |child| {
                        seen.set(Some(kivact::test::is_native_window_alive(child.window())));
                    })
                    .on_destroy(move || {
                        // The window is still alive while this runs.
                    }),
            )),
        )
    },
);
let editor = screen.window("Plugin editor");
assert!(editor.foreign_child().is_some());
```

## See also

- [Open multiple windows](windows.md): ordinary Kivact windows, their modes,
  contexts and tests.
- [Open a Kivact UI inside a plugin window](embed.md): the opposite direction,
  for the plugin author.
