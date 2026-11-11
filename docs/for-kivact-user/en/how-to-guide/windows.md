# Open multiple windows

[日本語](../../jp/how-to-guide/windows.md)

An app starts with one OS window, the main window. `window(title, open)` adds
another one: while `open` is true its content is mounted and shown in its own
OS window, and when `open` becomes false the content is unmounted and the
window closes. The content inherits the contexts of the place where you declare
it, like a React portal.

## Open a window

`window` is a view. Put it in the tree where its content should inherit from,
next to the control that opens it:

```rust
use kivact::prelude::*;

fn editor() -> impl IntoView {
    let settings_open = signal(false);
    column((
        button("Settings").on_press(move || settings_open.set(true)),
        window("Settings", settings_open)
            .size(480, 360)
            .mode(WindowMode::Tool)
            .placement(WindowPlacement::CenterOnParent)
            .content(|| text("Settings go here")),
    ))
}
```

`title` is reactive: pass a closure or a signal and the OS title follows it.
`open` is an ordinary `bool` binding, like the ones inputs take. Writing `true`
mounts the content in a new window; writing `false` unmounts it and closes the
window. If the owner that declared the window is destroyed, the window closes
too. Writing `true` while the window is already open does nothing: it does not
raise the window (see [Bring a window to the front](#bring-a-window-to-the-front)).

`content` is a factory: it runs each time the window opens, in a fresh owner. A
signal created inside it is dropped when the window closes, so create anything
that must survive a close outside the factory and capture the handle. This is
the same rule as a dialog's content.

## Close it from the window and from the title bar

The title bar's close button and Alt+F4 ask the window to close. If the window
has an `on_dismiss`, it is called with `DismissReason::WindowClose`. Return `()`
or `Handled::No` to let it close: `open` is set to false and the content is
unmounted. `Handled::Yes` and `Handled::PreventDefault` keep it open, and so
does a handler that returns an error (the error is reported). A window closes
itself by default, like `dialog`; an overlay does not.

```rust,test_harness
use kivact::prelude::*;
use kivact::view::DismissReason;

fn app() -> impl IntoView {
    let open = signal(false);
    let dirty = signal(true);
    column((
        button("Settings").on_press(move || open.set(true)),
        window("Settings", open)
            .content(|| text("Settings go here"))
            .on_dismiss(move |reason| match reason {
                DismissReason::WindowClose if dirty.get() => Handled::PreventDefault,
                _ => Handled::No,
            }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Preferences").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_prevented_dismiss_keeps_the_window_open() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "Settings").click();

        let settings = screen.window("Settings");
        settings.request_close();
        assert!(settings.is_open());
    }
}
```

Setting `open` to false yourself does not call `on_dismiss`; the user's close
request does. Closing the main window still ends the app, and this change adds
no way to cancel that.

## Window modes and placement

`.mode(..)` and `.placement(..)` are structural options: they are constants
chosen when the view is built, not bindings that change later.

| `WindowMode` | What it is |
| --- | --- |
| `Normal` (default) | an ordinary window with its own taskbar entry |
| `Tool` | a utility window that stays above its parent and has no taskbar entry |
| `Modal` | a window that blocks pointer, keyboard and assistive technology input to its parent while it is open |

| `WindowPlacement` | Where it opens |
| --- | --- |
| `Auto` (default) | where the platform puts it; the main window is centered on the primary monitor's work area |
| `CenterOnParent` | centered over the window it was declared in |
| `NextToParent` | to the right of the parent, or to its left when the right side has no room, or `Auto` when neither has room |

`.size(width, height)` is the logical size the window opens with (default
640 × 480), and `.min_size(..)` is the smallest size the user can resize it to.
Both are values for the moment it opens; after that the user resizes the
window. Kivact checks them once when the view is mounted, with the same rule as
`App::window_size` (at least 1 logical pixel, and finite). A bad size is
reported to the owner that declared the window as `ErrorKind::Message`, and the
window does not open; `open` is not changed.

On Wayland, `platform::capabilities().get().window_placement` is false: the
platform ignores placements other than `Auto`. Windows still open there; only
the requested place is not honoured.

## Contexts follow the declaration, like a React portal

The content factory runs under the owner that declared the window, so every
context available at the declaration is available inside the window: values
from `provide`, the theme, the locale, the messages, the reading direction and
the error handlers. This is what React's `createPortal` does: the child is
rendered in another place, but it still sees the context where it was written.

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, Copy)]
struct Draft {
    dirty: Signal<bool>,
}

fn form() -> impl IntoView {
    let draft = use_context::<Draft>();
    text(move || if draft.get().dirty.get() { "Unsaved changes" } else { "Saved" })
}

fn app() -> impl IntoView {
    let settings_open = signal(false);
    let dirty = signal(true);
    provide(Draft { dirty }, move || {
        column((
            button("Settings").on_press(move || settings_open.set(true)),
            window("Settings", settings_open).content(|| form()),
        ))
    })
}

fn main() -> kivact::Result<()> {
    App::new("Editor").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_reads_the_context_of_its_declaration() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "Settings").click();
        screen.window("Settings").get_by_text("Unsaved changes");
    }
}
```

`form` is built inside the window's factory, so it can read `Draft` even though
`window(..)` is in a different part of the tree. A signal created in the factory
itself starts over at every open; one created outside it and captured survives.

## Bring a window to the front

`platform::focus_window()` brings the window of the calling owner to the front
and gives it the keyboard focus. Call it from inside the window you want to
raise:

```rust,no_run
use kivact::platform;
use kivact::prelude::*;

fn settings() -> impl IntoView {
    button("Bring this window to the front")
        .on_press(|| {
            spawn(platform::focus_window());
        })
}
```

- It is best effort. Windows may only flash the taskbar entry instead of
  raising the window, and Wayland has no way to raise a window. It needs no
  capability. In an embedded editor it fails with `ErrorKind::Unsupported`.
- Writing `true` to the `open` binding of a window that is already open does
  nothing; it does not raise the window. To raise it from an action in another
  window, send a signal to that window and call `focus_window` from a handler
  or callback created in its content (see the next section).

## Ask about the current window

The `platform` queries and services act on the window of the owner that calls
them: `window_size`, `scale_factor`, `is_fullscreen`, `safe_area`,
`keyboard_inset`, `theme_hint`, `is_surface_active`, `is_motion_reduced` and
`pointer_settings`, and the services `request_window_size`, `set_fullscreen`,
`focus_window` and the file dialogs (the parent of the dialog). A handler or
callback belongs to the window where it was created, not to the window where it
runs, so create the callback inside the window's content when that window uses
it.

Wrong: the callback is created in the main window, so the dialog it opens is
parented to the main window.

```rust,no_run
use kivact::platform::{self, FileDialog};
use kivact::prelude::*;

fn app() -> impl IntoView {
    let open = signal(false);
    let choose: Callback<()> = (|| {
        spawn(async { let _ = platform::pick_file(FileDialog::default()).await; });
    })
    .into();
    column((
        button("Settings").on_press(move || open.set(true)),
        window("Settings", open)
            .content(move || button("Choose a file…").on_press(choose)),
    ))
}
```

Right: the callback is created in the Settings content, so the Settings window
is the dialog's parent.

```rust,no_run
use kivact::platform::{self, FileDialog};
use kivact::prelude::*;

fn app() -> impl IntoView {
    let open = signal(false);
    column((
        button("Settings").on_press(move || open.set(true)),
        window("Settings", open).content(|| {
            button("Choose a file…").on_press(|| {
                spawn(async { let _ = platform::pick_file(FileDialog::default()).await; });
            })
        }),
    ))
}
```

The same applies to `request_window_size` and `set_fullscreen`: call them from
the window whose size or fullscreen state you mean. Focus is per window too:
`NodeRef::focus()` moves the focus inside that node's window.

## Test windows

In a test, each window is a `Screen` of its own. `screen.window("Settings")`
waits until that window is open (the same time limit as `find_by_*`) and
returns its handle. `screen.windows()` returns every open window in id order,
the main window first.

```rust,test_harness
use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with};
use kivact::ErrorKind;

fn app() -> impl IntoView {
    let settings_open = signal(false);
    column((
        button("Settings").on_press(move || settings_open.set(true)),
        window("Settings", settings_open).content(|| text("Settings go here")),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Editor").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_is_a_screen_of_its_own() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "Settings").click();

        let settings = screen.window("Settings");
        settings.get_by_text("Settings go here");
        assert!(settings.is_open());

        settings.request_close();
        assert!(!settings.is_open());
    }

    #[test]
    fn windows_lists_the_open_windows() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "Settings").click();

        let windows = screen.windows();
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].title(), screen.title());
        assert_eq!(windows[1].title(), "Settings");
    }

    #[test]
    fn an_environment_with_one_window_reports_unsupported() {
        let screen = render_with(RenderOptions::default().single_window(), app);
        screen.get_by_role(Role::Button, "Settings").click();

        assert!(screen.query_by_text("Settings go here").is_none());
        assert!(screen.errors().iter().any(|error| *error.kind() == ErrorKind::Unsupported));
    }
}
```

A few rules:

- `screen.window(..)` returns a temporary. Store the handle in a `let` before
  you take an `Locator` from it: `let save =
  screen.window("Settings").get_by_role(..);` does not compile (E0716). The
  `let` only keeps the handle value alive; it does not keep the window open
  (writing `false` to `open` still closes it, and `is_open()` turns false).
- `is_open()` says whether the window is still open. `request_close()` makes
  the same close request as the title bar, so `on_dismiss` runs. On a closed
  handle, the window methods fail with "window <id> is closed".
- Two open windows with the same title make `window(title)` fail, like an
  ambiguous `get_by_*`.
- `RenderOptions::default().single_window()` models an environment that cannot
  open more windows: every `window` reports `ErrorKind::Unsupported` and stays
  closed. Use it to test the path where no window can open.

## Where windows are not available

`platform::capabilities().get().multi_window` says whether the environment can
open more windows than the first. It is false in an embedded editor (`embed`)
and on mobile. When it is false and `open` becomes true, Kivact reports an
`ErrorKind::Unsupported` error to the owner that declared the window, once per
declaration, and does not open the window; the binding is not changed.

In an embedded editor, `platform::focus_window()` also fails with
`ErrorKind::Unsupported`.

## See also

- [Show a dialog, menu, tooltip or toast](overlays.md): overlays inside one
  window, and the close rules a window shares with `dialog`.
- [Test your app](testing.md): queries, actions, time and failure artifacts.
- [Use platform services](platform-services.md): the queries and services
  listed above.
- API reference: `cargo doc -p kivact --open`, then `kivact::view` and
  `kivact::platform`.
