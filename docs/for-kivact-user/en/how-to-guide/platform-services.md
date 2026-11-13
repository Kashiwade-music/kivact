# Use platform services

[日本語](../../jp/how-to-guide/platform-services.md)

Use this guide to talk to the operating system: copy and paste text, ask the
user for files and folders, open or reveal a path in the file manager, list the
installed fonts, react to the app being suspended, and control or read the
window's size. Everything is in `kivact::platform`.

Services that ask the system for something are functions that return a
**future**. The request is sent the moment you *call* the function, and the
future gives you the answer. There are two ways to use it:

- `spawn(platform::copy_text("…"))` when you do not care about the answer. A
  failure is sent to the nearest `on_error`, like an error from an event handler.
- `.await` it inside `spawn`, an `action` or a `resource` when you want the
  result or want to handle the failure yourself.

Call them from setup, an event handler or a task, not from inside a `memo` or
a render effect.

## Copy and paste text

```rust,test_harness
use kivact::platform;
use kivact::prelude::*;

fn clipboard() -> impl IntoView {
    let pasted = signal(String::new());
    column((
        button("Copy").on_press(|| {
            spawn(platform::copy_text("Hello from Kivact"));
        }),
        button("Paste").on_press(move || {
            spawn(async move {
                let text = platform::read_text().await?;
                pasted.set(text);
                Ok::<(), UiError>(())
            });
        }),
        text(move || format!("Pasted: {}", pasted.get())),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Clipboard").run(clipboard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_and_paste_use_the_scripted_clipboard() {
        let screen = kivact::test::render(clipboard);
        screen.get_by_role(Role::Button, "Copy").click();
        assert_eq!(screen.clipboard().as_deref(), Some("Hello from Kivact"));

        screen.set_clipboard("from another app");
        screen.get_by_role(Role::Button, "Paste").click();
        screen.get_by_text("Pasted: from another app");
    }
}
```

`read_text` returns an empty string when the clipboard holds no text. Text
inputs already copy, cut and paste by themselves; use these functions for
your own "Copy link" or "Copy result" buttons.

The test shows the second half of the story: in tests the clipboard is an
in-memory stand-in, so you can check what the app copied and prepare what it
pastes without touching the real clipboard.

## Ask the user for a file, files or a folder

`pick_file`, `pick_files`, `pick_folder` and `save_file` open the system's
native dialog. They take a `FileDialog` (title, starting directory, suggested
file name and filters) and resolve to `Ok(None)` when the user cancels.
Cancelling is not an error.

```rust,no_run
use kivact::platform::{self, FileDialog};
use kivact::prelude::*;

fn open_document() -> impl IntoView {
    let status = signal(String::from("Nothing opened"));
    column((
        button("Open…").on_press(move || {
            spawn(async move {
                let options = FileDialog {
                    title: "Open a document".into(),
                    // (label, extensions without the dot)
                    filters: vec![("Text".into(), vec!["txt".into(), "md".into()])],
                    ..FileDialog::default()
                };
                match platform::pick_file(options).await? {
                    Some(path) => {
                        // Read on the blocking pool; `?` turns io::Error into UiError.
                        let text = blocking(move || std::fs::read_to_string(&path)).await?;
                        status.set(format!("Read {} bytes", text.len()));
                    }
                    None => status.set("Cancelled".into()),
                }
                Ok::<(), UiError>(())
            });
        }),
        text(move || status.get()),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Open").run(open_document)
}
```

- `pick_files` shows one dialog for several files and gives
  `Ok(Some(Vec<PathBuf>))` in the order the system returns them.
- `pick_folder` uses only `title` and `directory`. `file_name` and `filters` are
  ignored.
- `save_file` is the same as `pick_file` for saving. Set `file_name` to suggest a
  name. It returns the path the user chose, but it does not create the file:
  you write it.
- Reading or writing the chosen file is ordinary blocking I/O, so do it in
  `blocking`, as above. See [Run async work from a component](async-and-tasks.md).

In tests the dialogs answer from a script, so no window appears and the test
chooses what the user "picked". An unscripted dialog is cancelled:

```rust,test_harness
use std::path::PathBuf;
use kivact::platform::{self, FileDialog};
use kivact::prelude::*;

fn export() -> impl IntoView {
    let status = signal(String::new());
    column((
        button("Export").on_press(move || {
            spawn(async move {
                let options = FileDialog {
                    title: "Export".into(),
                    file_name: Some("report.csv".into()),
                    ..FileDialog::default()
                };
                let message = match platform::save_file(options).await? {
                    Some(path) => format!("Saving to {}", path.display()),
                    None => "Cancelled".to_string(),
                };
                status.set(message);
                Ok::<(), UiError>(())
            });
        }),
        text(move || status.get()),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Export").run(export)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_test_decides_what_the_user_picks() {
        let screen = kivact::test::render(export);

        screen.respond_file_dialog(Some(PathBuf::from("out/report.csv")));
        screen.get_by_role(Role::Button, "Export").click();
        let expected = format!("Saving to {}", PathBuf::from("out/report.csv").display());
        screen.get_by_text(&expected);

        // No answer scripted: the dialog is cancelled.
        screen.get_by_role(Role::Button, "Export").click();
        screen.get_by_text("Cancelled");
    }
}
```

Use `Screen::respond_file_dialog` for `pick_file`, `save_file` and `pick_folder`,
and `respond_files_dialog` for `pick_files`.

## Open or reveal a file or folder

`open_path` opens a file with the application the system associates with it,
or a folder in the file manager. `reveal_path` shows the item selected in the
file manager. Both take a path, and a relative path is resolved against the
process's current directory.

```rust,no_run
use kivact::platform;
use kivact::prelude::*;

fn exported() -> impl IntoView {
    row((
        button("Open image").on_press(|| {
            spawn(platform::open_path("exports/frame.png"));
        }),
        button("Show in folder").on_press(|| {
            spawn(platform::reveal_path("exports/frame.png"));
        }),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Exports").run(exported)
}
```

- They work on Windows (Explorer) and Linux (`xdg-open`). Elsewhere they fail
  with `ErrorKind::Unsupported`. On Linux, `reveal_path` cannot select the
  item, so it opens the containing folder.
- `Ok(())` means the handler was started. What it does afterwards is not
  reported. A path that does not exist, or a handler that cannot start, is an
  `ErrorKind::Platform` error that names the path.
- In tests nothing is started. `screen.opened_paths()` and
  `screen.revealed_paths()` return what the app requested, in order, and
  `screen.fail_next_platform_request("reason")` makes the next request fail so
  you can test your error message.

### Handle a failure yourself

To tell the user instead of sending the error to `on_error`, await the future
and match on the error kind:

```rust,no_run
use kivact::ErrorKind;
use kivact::platform;
use kivact::prelude::*;

fn open_with_message(path: &'static str, message: Signal<String>) {
    spawn(async move {
        match platform::open_path(path).await {
            Ok(()) => {}
            Err(error) if *error.kind() == ErrorKind::Unsupported => {
                message.set("This system cannot open files from here".into());
            }
            Err(error) => message.set(format!("Could not open it: {error}")),
        }
    });
}
```

## List the installed fonts

`platform::font_families()` returns the family names the window can use, sorted,
without duplicates, with their original capitalization. It includes the system
fonts and the fonts you registered with `App::font(..)`. It is a snapshot taken
at the call, not a reactive value.

```rust,no_run
use kivact::prelude::*;

fn font_list() -> impl IntoView {
    let families = kivact::platform::font_families();
    scroll(text(families.get().join("\n"))).style(style().size(320, 400))
}

fn main() -> kivact::Result<()> {
    App::new("Fonts").run(font_list)
}
```

A family from the list is what `style().font_family("…")` takes. To ship a font
with your app, register it before the first screen:
`App::new("…").font(include_bytes!("../assets/Inter-Regular.ttf"))`. The
family is the name written inside the font, not the file name.

## React when the app is suspended

`platform::on_lifecycle` registers, in setup, a callback for application
lifecycle events: `Suspended`, `Resumed` and `MemoryWarning`. It lives as long
as the component and runs in that component's context. Which events arrive
depends on the platform: `platform::capabilities().memory_warnings` tells
whether memory warnings are reported at all.

```rust,no_run
use kivact::platform::{self, LifecycleEvent};
use kivact::prelude::*;

fn memory_status() -> impl IntoView {
    let warnings = signal(0usize);
    platform::on_lifecycle(move |event| {
        if event == LifecycleEvent::MemoryWarning {
            // Drop caches you can rebuild.
            warnings.update(|count| *count += 1);
        }
    });
    text(move || format!("Memory warnings: {}", warnings.get()))
}

fn main() -> kivact::Result<()> {
    App::new("Lifecycle").run(memory_status)
}
```

Whether the window can currently be drawn is a separate, reactive question:
`platform::surface_active()`. A lost or hidden surface does not produce a
lifecycle event. On a memory warning the window already releases unused
text and image caches before your callback runs, so shrink only the caches your
own code keeps.

## Set and read the window's size

Set the first size, and the smallest size the user can resize to, on the `App`.
Values are logical pixels. The default is 960 × 720.

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    text("Resizable")
}

fn main() -> kivact::Result<()> {
    App::new("Editor")
        .window_size(1280, 800)
        .min_window_size(640, 480)
        .run(app)
}
```

`run` returns an error, and opens no window, if a size is zero, negative or not
finite, or if the minimum is larger than the starting size. Kivact has no API to
choose the window's position.

**Read the current size** with `platform::window_size()`. It is a tracked read:
a style or view that reads it updates on every resize. This is how you build
responsive layouts, where CSS would use a media query:

```rust,test_harness
use kivact::platform;
use kivact::prelude::*;

fn shell() -> impl IntoView {
    let narrow = memo(move || platform::window_size().get().width < 600.0);
    column(text(move || {
        if narrow.get() { "Narrow layout" } else { "Wide layout" }
    }))
}

fn main() -> kivact::Result<()> {
    App::new("Responsive").run(shell)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_follows_the_window() {
        let screen = kivact::test::render(shell); // 800 x 600 by default
        screen.get_by_text("Wide layout");
        screen.resize(400.0, 600.0);
        screen.get_by_text("Narrow layout");
    }
}
```

Wrap the comparison in a `memo` so the view updates only when the answer
changes, not on every pixel of a drag.

**Ask the system to resize** with `spawn(platform::request_window_size(640, 480))`.
The size is clamped to the minimum you set. It is a request: the system may
refuse, and the future resolves with a `Platform` error then. For borderless
fullscreen, call `spawn(platform::set_fullscreen(true))` and read
`platform::fullscreen()`.

In an embedded editor the host may confirm a size later. The future then waits
for the host's `set_size` (up to two seconds), resolves with the requested size
when the host agrees, and fails with a `Platform` error naming the host's size
when it chooses another. A newer request cancels the older one
(`ErrorKind::Cancelled`), and no answer within two seconds is
`ErrorKind::Timeout`; the window keeps its size in both cases.

The other reads of the environment follow the same rules (tracked, usable from
setup, handlers, tasks and effects):

| Function | What it returns |
|---|---|
| `scale_factor()` | physical pixels per logical pixel; multiply sizes you give to a pixel renderer |
| `reduced_motion()` | the system asks for less animation |
| `safe_area()` | the insets that system UI may cover (zero on desktops) |
| `surface_active()` | the window can currently be drawn |
| `capabilities()` | what the platform supports (touch, pen, soft keyboard, GPU, …), fixed for the app's life |

In tests, `RenderOptions` sets the starting size, scale, dark mode and reduced
motion, and `Screen::resize`, `set_scale`, `set_dark`, `set_reduced_motion` and
`set_safe_area` change them while the screen runs. See
[Test your app](testing.md#configure-the-environment).

## Close the app

`platform::app_handle()` returns the handle that ends the app's run from any
thread, and `App::handle()` returns the same handle before the run starts.
`close()` asks the app to end as the main window's close button does, without
a dismissal:

```rust,no_run
use kivact::platform;
use kivact::prelude::*;

fn app() -> impl IntoView {
    button("Quit").on_press(|| platform::app_handle().close())
}
```

`close` returns at once; queued actions drain for up to three seconds, the
app's owners are disposed, and `App::run` returns `Ok(())`. A handle belongs to
one run, and a close after that run ended does nothing. In an embedded app
`close` reports `ErrorKind::Unsupported`: drop the `Embedded` handle instead.
See [Close the app from code](windows.md#close-the-app-from-code).

## Ask about the window that calls

The environment reads and the services act on the window of the owner that calls
them, not on the main window: `window_size`, `scale_factor`, `is_fullscreen`,
`safe_area`, `keyboard_inset`, `theme_hint`, `is_surface_active`,
`is_motion_reduced` and `pointer_settings`, and the services
`request_window_size`, `set_fullscreen`, `focus_window` and the file dialogs
(the dialog's parent). A handler or callback belongs to the window where it was
created, not to the window where it runs, so create it inside that window's
content.

`platform::focus_window()` brings the calling owner's window to the front and
gives it the keyboard focus:

```rust,no_run
use kivact::platform;
use kivact::prelude::*;

fn settings() -> impl IntoView {
    button("Bring this window to the front").on_press(|| {
        spawn(platform::focus_window());
    })
}
```

It is best effort: Windows may only flash the taskbar entry, and Wayland has no
way to raise a window. It needs no capability. In an embedded editor it fails
with `ErrorKind::Unsupported`, and writing `true` to an already open window's
binding does not raise it. See
[Bring a window to the front](windows.md#bring-a-window-to-the-front).

## See also

- [Run async work from a component](async-and-tasks.md): futures, `spawn` and `blocking`.
- [Test your app](testing.md): scripted dialogs, clipboard and paths in tests.
- [Animate values and elements](animation.md): `reduced_motion` in practice.
- API reference: `cargo doc -p kivact --open`, then `kivact::platform`.
