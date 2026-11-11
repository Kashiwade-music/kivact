# Test your app

[日本語](../../jp/how-to-guide/testing.md)

Use `kivact::test` to test a Kivact app the way a person uses it: render it,
find elements by what a screen reader would announce, click and type, and check
what is on screen. The tests run in-process with `cargo test`. They need no
browser and no driver, and they never wait in real time.

This guide goes deeper than the getting-started tutorial: every kind of query
and action, state and accessibility assertions, time, screenshots, failure
artifacts, testing one component or the whole app, and CI.

## Set up

Enable the `test` feature for tests only, so release builds do not carry it:

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }

[dev-dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", features = ["test"] }
```

Write tests in a `#[cfg(test)] mod tests` next to the code, or in `tests/`.
`kivact::test::render(root)` mounts a component and returns a `Screen`.

```rust,test_harness
use kivact::prelude::*;

fn counter() -> impl IntoView {
    let count = signal(0);
    column((
        text(move || format!("Count: {}", count.get())),
        button("Add one").on_press(move || count.update(|n| *n += 1)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Counter").run(counter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressing_the_button_adds_one() {
        let screen = kivact::test::render(counter);
        screen.get_by_text("Count: 0");
        screen.get_by_role(Role::Button, "Add one").click();
        screen.get_by_text("Count: 1");
        assert!(screen.errors().is_empty());
    }
}
```

What `render` really does: it mounts your component in the virtual application
host with the same layout, text shaping, drawing and accessibility used by an
App. Its default surface is 800 × 600 logical pixels; a desktop can mirror that
surface into a native window. Input is delivered as the window would deliver it, the
clock is virtual, and a settle step runs after each action so the tree is up to
date when the next line runs. There are no mocks to configure.

## Find elements

There are three ways to find an element, and each comes in four forms:

| By | Example | Matches |
|---|---|---|
| role and name | `get_by_role(Role::Button, "Save")` | what assistive technology exposes: the accessible name, exactly |
| text | `get_by_text("Count: 1")` | a `text(..)` node with exactly this content |
| test id | `get_by_test_id("total")` | a node marked with `.test_id("total")` |

| Form | If nothing matches | If several match | Waits for the UI |
|---|---|---|---|
| `get_by_*` | fails the test | fails the test (ambiguous) | no |
| `query_by_*` | returns `None` | returns `None` | no |
| `find_by_*` | waits, then fails after 5 s of virtual time | waits, then fails after 5 s | yes |
| `all_by_*` | returns an empty `Vec` | returns all of them | no |

Prefer **role and name**. It is the same query a user of a screen reader makes,
so a test that passes also proves the element is accessible: an unlabeled icon
button cannot be found this way, which is a bug in the app, not in the test.
Use `get_by_text` for plain text, and `test_id` only for things that have no role
or text, such as a drawn chart.

Use `within(element)` to search inside one part of the screen when the same
label appears more than once:

```rust,no_run
# use kivact::prelude::*;
# fn demo(screen: &kivact::test::Screen) {
let row = screen.get_by_test_id("row-2");
screen.within(row).get_by_role(Role::Button, "Delete").click();
# }
```

`query_by_*` is the way to assert absence: `assert!(screen.query_by_text("Saved").is_none())`.

## Act like a user

Actions on an `Locator` do what a person would do. Before a pointer action, the
test checks the element is something a person could use, and fails with the reason
if not: it must be mounted, visible, have a size, be enabled, be inside the
window and be what the pointer actually hits at its center (not covered by another
element or a modal's backdrop).

| Goal | Call |
|---|---|
| click | `click()`, `double_click()`, `click_with(MouseButton::Secondary)` |
| touch or pen | `tap()`, `tap_with(PointerKind::Pen)` |
| hover | `hover()` |
| type into an input | `type_text("hello")` (focuses first; `\n` is Enter) |
| press a key | `press_key(Key::Enter)`, `press_key_with(Key::Char('a'), Modifiers { ctrl: true, ..Default::default() })` |
| move focus | `focus()`, or `screen.press_tab(false)` (`true` for Shift+Tab) |
| flip a check box or switch | `toggle()` |
| choose an option in a select | `select_option("Large")` |
| drag | `drag(from, to)` in the element's own coordinates |
| scroll | `scroll(dx, dy)` |
| assistive technology | `a11y(A11yAction::…)`, `a11y_set_value("text")` |
| IME composition | `compose("preedit")`, `commit("text")` |

For a click that is *meant* to do nothing, or a press outside a popup, skip the
checks with `force_click()` or the screen-level `click_at(x, y)`, `mouse_move`,
`mouse_down` and `mouse_up`. Place the pointer first (`mouse_move`, or a click
or hover): `mouse_down`, `mouse_up` and `mouse_leave` before that panic, as a
real window has no pointer to press. `contact_down/move/up` run several touch
contacts at once. `Key`, `Modifiers`, `MouseButton` and `PointerKind` are in
`kivact::view` (`Key` is also in the prelude).

```rust,test_harness
use kivact::prelude::*;

fn todo() -> impl IntoView {
    let draft = signal(String::new());
    let tasks = signal(Vec::<String>::new());
    column((
        text_input("New task", draft).on_submit(move || {
            let task = draft.get();
            if !task.trim().is_empty() {
                tasks.update(|tasks| tasks.push(task));
                draft.set(String::new());
            }
        }),
        dynamic(move || tasks.get(), |tasks| {
            column(tasks.into_iter().map(text).collect::<Vec<_>>())
        }),
        text(move || format!("{} tasks", tasks.get().len())),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Todo").run(todo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_adds_a_task_and_clears_the_field() {
        let screen = kivact::test::render(todo);
        let input = screen.get_by_role(Role::TextBox, "New task");
        input.type_text("Buy milk");
        input.press_key(Key::Enter);

        screen.get_by_text("Buy milk");
        screen.get_by_text("1 tasks");
        assert_eq!(input.value().as_deref(), Some(""));
    }
}
```

## Check what the user would see

An `Locator` answers questions about the same published state a screen reader
gets, not about internal variables.

| Question | Call |
|---|---|
| text, name, description | `text()`, `name()`, `description()` |
| role | `role()` |
| input value, slider range | `value()`, `value_range()` |
| state | `is_disabled()`, `checked()` (`Option<CheckedState>`: `None` if not checkable), `is_selected()`, `is_expanded()`, `is_focused()`, `invalid()`, `read_only()` |
| position in a set | `level()`, `set_size()`, `pos_in_set()` |
| geometry | `rect()`, `accessibility_bounds()` |
| shown | `exists()`, `is_visible()` |
| relations | `labelled_by()`, `described_by()`, `controls()` |

On the screen: `screen.focused()`, `screen.hovered()`, `screen.cursor()` and
`screen.title()`. `screen.errors()` returns the errors that reached the app's
default error sink, that is, ones no `on_error` or `error_boundary` caught.
Asserting it is empty at the end of a test catches a lot.

```rust,test_harness
use kivact::prelude::*;

fn terms() -> impl IntoView {
    let accepted = signal(false);
    column((
        checkbox("I accept the terms", accepted),
        button("Continue").disabled(move || !accepted.get()),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Terms").run(terms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continue_needs_the_terms() {
        let screen = kivact::test::render(terms);
        let proceed = screen.get_by_role(Role::Button, "Continue");
        assert!(proceed.is_disabled());

        let terms = screen.get_by_role(Role::CheckBox, "I accept the terms");
        assert_eq!(terms.checked(), Some(CheckedState::Unchecked));
        terms.toggle();

        assert_eq!(terms.checked(), Some(CheckedState::Checked));
        assert!(!proceed.is_disabled());
        screen.assert_layout();
    }
}
```

`screen.assert_layout()` also checks the layout: text that overflows its box,
for example. It fails with every violation it finds and keeps the frame.
`check_layout()` returns the violations as values for your own assertions, and
`assert_layout_with(&LayoutCheck::default().ignore_test_id("clipped"))` skips
things that are meant to be clipped.

A panic in your app's code (a handler, an effect, a memo, a setup function, a
task, a `blocking` job, or a misuse Kivact detects) is a bug, and the test
library does not catch it. It comes out of the `Screen` call that drove the turn
with its original message, the failure artifacts are written, and the test
fails. To test that Kivact detects a misuse, expect the panic:

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    button("Misuse").on_press(|| {
        // A setup function in a handler is a bug.
        let _late = signal(0);
    })
}

fn main() -> kivact::Result<()> {
    App::new("Misuse").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "SetupOnly")]
    fn a_setup_function_in_a_handler_panics() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "Misuse").click();
    }
}
```

## Test one component or the whole app

A component is a function, so a test can render one on its own with the
arguments it needs. When the test must read or change a signal the component
uses, create it inside the setup and hand it back with `render_with_handle`:

```rust,test_harness
use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with_handle};

/// A `−` and `+` pair around a number you own.
fn stepper(value: Signal<i32>) -> impl IntoView {
    row((
        button("Decrease").on_press(move || value.update(|n| *n -= 1)),
        text(move || value.get().to_string()),
        button("Increase").on_press(move || value.update(|n| *n += 1)),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Stepper").run(|| stepper(signal(0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_edits_the_signal_it_is_given() {
        let (screen, value) = render_with_handle(RenderOptions::default(), || {
            let value = signal(5);
            (value, stepper(value))
        });
        screen.get_by_role(Role::Button, "Increase").click();
        assert_eq!(value.get_untracked(), 6);

        // The other direction: the test writes, the screen follows once it settles.
        value.set(1);
        screen.run_until_idle();
        screen.get_by_text("1");
    }
}
```

To test the whole application, render its root function the way `main` does:
`render(app)`. Test the whole app for flows that cross components (log in, create,
find, delete). Test components in isolation for their states, keyboard behavior
and edge cases, because that is faster to set up and to read.

If a component needs context that the app normally provides (a theme, i18n
messages, a store), provide it in the closure you pass to `render`, the same way
`main` does, or give `RenderOptions` a `theme`, `locale` and `messages`.

Do not call `App::run` in a test binary that renders screens: all test windows
share one event loop per process. Keep `main` out of your tests; `render` is
the test entry point.

## Test windows

A window is a `Screen` of its own. `screen.window("Settings")` waits for that
window to open (the same time limit as `find_by_*`) and returns its handle;
`screen.windows()` returns every open window in id order. `Screen::has_window()` is now `is_mirrored()` (a public rename; the meaning, whether this window is shown in a real window, is unchanged). `RenderOptions::default().single_window()` models an environment that cannot open more windows, such as an embedded editor. Keep the handle in a `let` before you take an `Locator` from it:

```rust,no_run
# use kivact::prelude::*;
# fn demo(screen: &kivact::test::Screen) {
let settings = screen.window("Settings");
settings.get_by_role(Role::Button, "Apply").click();
assert!(settings.is_open());

settings.request_close();
assert!(!settings.is_open());
# }
```

`screen.window(..)` returns a temporary, so `let apply =
screen.window("Settings").get_by_role(..);` does not compile (E0716); the
handle keeps the window alive. Two open windows with the same title make
`window` fail, like an ambiguous `get_by_*`. See
[Open multiple windows](windows.md#test-windows).

## Time and async work

Time in tests is virtual. A timer waits for `advance`, not for the clock on the
wall, so a 30-second timeout takes no time and the result never depends on how
fast the machine is.

| Call | What it does |
|---|---|
| `screen.run_until_idle()` | runs everything that is ready now, and waits (in real time, up to 5 s) for blocking jobs started with `blocking` |
| `screen.advance(duration)` | moves the virtual clock forward, stopping at every timer and frame on the way |
| `screen.wait_for(timeout, \|s\| condition)` | advances frame by frame until the condition holds, or fails when the virtual `timeout` passes |
| `screen.find_by_text("…")` | like `get_by_text`, but waits up to 5 s of virtual time for the element to appear |
| `screen.now()` | the virtual time since the screen started |

Actions such as `click` already settle, so you rarely call `run_until_idle`.
After starting a timer, call `advance`.

```rust,test_harness
use std::time::Duration;
use kivact::prelude::*;
use kivact::task::sleep;

fn saver() -> impl IntoView {
    let shown = signal(false);
    column((
        button("Save").on_press(move || {
            spawn(async move {
                shown.set(true);
                sleep(Duration::from_secs(2)).await;
                shown.set(false);
            });
        }),
        show(shown, || text("Saved")),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Saver").run(saver)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_notice_disappears_after_two_seconds() {
        let screen = kivact::test::render(saver);
        screen.get_by_role(Role::Button, "Save").click();
        screen.get_by_text("Saved");

        screen.advance(Duration::from_millis(1900));
        assert!(screen.query_by_text("Saved").is_some());
        screen.advance(Duration::from_millis(200));
        assert!(screen.query_by_text("Saved").is_none());
    }

    #[test]
    fn waiting_for_a_condition() {
        let screen = kivact::test::render(saver);
        screen.get_by_role(Role::Button, "Save").click();
        let waited = screen.wait_for(Duration::from_secs(5), |s| s.query_by_text("Saved").is_none());
        assert!(waited >= Duration::from_secs(2));
    }
}
```

Work started with `blocking` runs on real threads, so it takes real time, but
the test waits for it automatically (up to five seconds) before it continues.
Frame loops, animations and transitions run on the same virtual clock; see
[Animate values and elements](animation.md#test-animations).

## Configure the environment

`render_with(options, root)` starts a screen with a `RenderOptions`. Every field
has a default, so set only what the test is about:

```rust,test_harness
use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with};

fn size_label() -> impl IntoView {
    text(|| {
        let size = kivact::platform::window_size().get();
        format!("{} x {}", size.width, size.height)
    })
}

fn main() -> kivact::Result<()> {
    App::new("Size").run(size_label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_phone_sized_window_at_double_density() {
        let options = RenderOptions::default().window_size(390.0, 844.0).scale(2.0);
        let screen = render_with(options, size_label);
        screen.get_by_text("390 x 844");
        assert_eq!(screen.screenshot().width, 780); // physical pixels

        screen.resize(800.0, 600.0);
        screen.get_by_text("800 x 600");
    }
}
```

Fields of `RenderOptions`: `width`, `height`, `scale`, `theme`, `locale`,
`host_locale` (the system's locale), `messages`, `dark` (the system prefers a
dark theme), `reduced_motion`, `fonts`, `window`, `gpu`, `frame_period_ns`
(the display's refresh interval; default 60 Hz) and `title`.

While the screen runs, the test can play the part of the system:
`resize`, `set_scale`, `set_dark`, `set_reduced_motion`, `set_host_locale`,
`set_safe_area`, `set_window_focused`, `suspend` and `resume`, `memory_warning`,
`lose_surface` and `restore_surface`, and `request_close`. Platform services are
scripted too: `clipboard` and `set_clipboard`, `respond_file_dialog`,
`respond_files_dialog`, `opened_paths`, `revealed_paths` and
`fail_next_platform_request`. See [Use platform services](platform-services.md).

Text is shaped with a **bundled font** (Tuffy) by default, so layout and pixels
are the same on every machine. That font has no Japanese or other CJK glyphs:
such text takes the same space on every machine but is drawn as empty boxes. To
test with real glyphs, set `.fonts(TestFonts::Only(vec![Asset::from(bytes)]))` with a font file
you ship, or `TestFonts::System` to use the machine's fonts (layout then depends
on the machine).

## Screenshots

`screen.screenshot()` returns the last frame as a `Screenshot`: `width` and
`height` in physical pixels and `pixels` as `0x00RRGGBB` values.
`pixel(x, y)` reads one, `crop(x, y, w, h)` cuts out a part, `hash()` summarizes
a frame, and `save_png(path)` writes a file. `element.screenshot()` is the part
under one element.

```rust,test_harness
use kivact::prelude::*;
use kivact::test::{RenderOptions, render_with};

fn badge() -> impl IntoView {
    column(text("New")).style(style().padding(8).background(rgb(0x2050d0)))
}

fn main() -> kivact::Result<()> {
    App::new("Badge").run(badge)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_badge_is_drawn_in_blue() {
        let options = RenderOptions::default().window_size(200.0, 100.0);
        let screen = render_with(options, badge);
        let shot = screen.screenshot();
        assert_eq!((shot.width, shot.height), (200, 100));
        // A corner inside the padding shows the background.
        assert_eq!(shot.pixel(2, 2), Some(0x2050d0));
    }
}
```

By default the CPU draws the frames, which makes them identical bit for bit
across machines, so a pixel or `hash()` comparison is stable. Use screenshots
for drawn content (canvas, colors, gradients), and queries for everything that
has a role or text. Do not compare whole-window screenshots to stored
images as the main check; they break on every intentional change.

Call `RenderOptions::default().gpu()` (or the environment variable
`KIVACT_TEST_GPU=1` with `RenderOptions::from_env()`) to render through the GPU
and read back the pixels. `kivact::test::gpu_unavailable()` says why a GPU test
cannot start, or `None` when it can.

## When a test fails

A failing query or action says what it looked for, every candidate it found, the
focus, the recent actions, errors, and the accessibility tree and semantic
tree. A failed test also leaves two files:

- `<test name>.png`: the last frame, as the user would have seen it;
- `<test name>.txt`: the accessibility tree the window published (role, name,
  value, states, test id and bounds, one node per line), the app's semantic
  tree, recent actions and the virtual time.

They are written to a `kivact-test-failures` folder in the Cargo target
directory (`CARGO_TARGET_DIR` if set, otherwise the target folder that holds the
test binary, usually `target/kivact-test-failures/`). Special characters in the
test name become `_`, and a second screen in the same test adds `-2`. The panic
message and standard error name the files. `kivact::test::failure_dir()`
returns the folder.

To look at the same data on a passing test, print `screen.accessibility_tree()`
(what a screen reader sees) or `screen.debug()` (the retained tree), or save
`screen.screenshot().save_png("frame.png")`. Every settle draws requested frames.
`KIVACT_TEST_HOLD_MS=3000` keeps the mirror open for three seconds when a test ends.

## Test shaders on the GPU and the CPU

A shader is drawn in two lanes, and a test can cover both. On the **CPU lane**
(the default) the shader is not run: the CPU draws the paint that
`ShaderPaint::fallback` gives, so the picture is the fallback and the test still
renders the real screen and finds its elements. On the **GPU lane** the window
runs the WGSL on the graphics card, and the picture is the shader's output. The
GPU lane needs a graphics adapter and a native window, so it is skipped where
either is missing (see [Run tests in CI](#run-tests-in-ci)).

A GPU test asks for the lane with `RenderOptions::from_env().gpu()`, requires
the native window with `.mirror(MirrorMode::Required)`, and returns early when
`gpu_unavailable()` gives a reason. `from_env()` also reads `KIVACT_TEST_GPU=1`, so
a test that leaves out `.gpu()` runs on the GPU lane only when a CI job sets that
variable on a machine with a card. A GPU test
that is skipped does not compile the shader, so keep a CPU test that renders the
screen, as below: it compiles the WGSL on every machine.

```rust,test_harness
use kivact::draw::PaintShader;
use kivact::geometry::Rect;
use kivact::prelude::*;

const WAVE: &str = r#"
struct Params { speed: f32, tint: vec4<f32> }

fn paint(position: vec2<f32>, context: Context, params: Params) -> vec4<f32> {
    let uv = position / context.size;
    let wave = 0.5 + 0.5 * sin(uv.x * 12.0 + context.time * params.speed);
    return vec4<f32>(params.tint.rgb * wave, 1.0);
}
"#;

fn wave() -> impl IntoView {
    let shader = PaintShader::wgsl(WAVE).expect("the shader is valid WGSL");
    canvas(move |draw, size| {
        // The CPU lane draws the solid fallback instead of the shader.
        let paint = shader
            .paint()
            .param("speed", 2.0)
            .param("tint", rgb(0x3b82f6))
            .fallback(rgb(0x1e3a8a));
        draw.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), paint);
    })
    .style(style().size(120, 60))
    .test_id("wave")
}

fn main() -> kivact::Result<()> {
    App::new("Wave").run(wave)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::{MirrorMode, RenderOptions, gpu_unavailable, render, render_with};

    #[test]
    fn the_wave_renders_on_the_cpu_lane() {
        let screen = render(wave);
        screen.get_by_test_id("wave");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }

    #[test]
    fn the_wave_renders_on_the_gpu_lane() {
        if let Some(reason) = gpu_unavailable() {
            eprintln!("skipped: {reason}");
            return;
        }
        let options = RenderOptions::from_env().gpu().mirror(MirrorMode::Required);
        let screen = render_with(options, wave);
        screen.get_by_test_id("wave");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
```

The screen is the same in both lanes, so a test that checks structure or text
needs no lane. Compare pixels only in a test that says which lane it is on,
because the two lanes draw different pictures.

## Run tests in CI

Tests run with the normal `cargo test`. What differs from unit tests is the
window:

- **A desktop session gives real mirrors.** `RenderOptions::default()` and
  `RenderOptions::from_env()` read `KIVACT_TEST_MIRROR`: `off` keeps frames without
  a native window, `auto` opens a mirror when possible (the default), and `required`
  fails if one cannot open. Call `.mirror(MirrorMode::Off)`, `.mirror(MirrorMode::Auto)`, or
  `.mirror(MirrorMode::Required)` explicitly to override that preference.
  `RenderOptions::from_env()` also reads `KIVACT_TEST_GPU=1`; GPU presentation
  requires a native mirror. CPU rendering produces the same pixels in every mode.
- **On a CI machine without a display** the default options fall back to
  headless, and everything above works. If you want real windows there (to
  exercise the whole windowing path), run the job inside a virtual display
  (for example `xvfb-run` on Linux) or on a runner with a desktop session.
- **Do not run window tests from several processes at the same time** on one
  desktop. Windows from parallel test processes compete for focus and can hang.
  Run test binaries one after another.
- **One event loop per process**: a test binary that renders screens must not call
  `App::run`.
- A test waits for `blocking` jobs for at most five real seconds; everything
  else runs on the virtual clock, so no test should sleep.

## See also

- [Run async work from a component](async-and-tasks.md): what the virtual clock drives.
- [Animate values and elements](animation.md): testing transitions and reduced motion.
- [Use platform services](platform-services.md): scripted clipboard, dialogs and paths.
- [Draw with a canvas](canvas.md): finding drawn regions by role and name.
- API reference: `cargo doc -p kivact --features test --open`, then `kivact::test`.
