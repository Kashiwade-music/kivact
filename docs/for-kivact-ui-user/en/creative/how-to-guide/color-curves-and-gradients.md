# Edit colors, easing curves and gradients

[日本語](../../../jp/creative/how-to-guide/color-curves-and-gradients.md)

Goal: let the user edit a color, an easing curve or a gradient, with your own signal holding the value and an undo step for each edit.

The three editors of this guide edit *one value* that you pass in as a binding: a `Color`, a `CubicBezier` and a `Vec<GradientStop>`. They keep no copy, so what you read from your signal is always current. This guide assumes you finished the [tutorial](../getting-started/README.md).

## Pick a color

The smallest solution is `color_popup`: a swatch button that opens a color wheel in a popup.

```rust,no_run
use kivact::prelude::*;
use kivact::style::rgb;
use kivact_ui_creative as ui;

fn fill_picker() -> impl IntoView {
    let fill = signal(rgb(0x8b7cff));
    ui::property_row("Fill", ui::color_popup("Fill", fill))
}
```

`Esc` closes the popup and puts back the color it opened with; clicking outside closes it and keeps the change. The swatch always shows the current color, and its accessible name is the label you give.

For a permanently visible editor, use `color_wheel`:

```rust,no_run
use kivact::prelude::*;
use kivact::style::rgb;
use kivact::view::EditPhase;
use kivact_ui_creative as ui;

fn grading_wheel() -> impl IntoView {
    let color = signal(rgb(0x8b7cff));
    let model = signal(ui::ColorModel::Rgb);
    let palette = signal(vec![rgb(0xffffff), rgb(0x000000)]);
    let edits = signal(0);

    column((
        ui::color_wheel(color, model)
            .wheel_size(200.0)
            .alpha(false)
            .swatches(palette)
            .on_edit(move |phase| {
                if phase == EditPhase::Commit {
                    edits.update(|n| *n += 1);
                }
            }),
        text(move || format!("{} color edit(s)", edits.get())),
    ))
    .style(style().gap(8).padding(12).width(260))
}
```

`color_wheel(color, model)` takes two bindings. `color` is the value. `model` is which channels the numeric fields show (`ColorModel::Rgb`, `Hsv` or `Hex`); switching it changes the fields, not the color, so keep it in a signal of its own, or share one between several wheels so they all show the same model.

| Option | Effect |
|--------|--------|
| `.wheel_size(px)` | Height of the wheel area in logical pixels (at least 96; default 220). |
| `.alpha(false)` | Hide the alpha strip and field; the color's alpha is kept as it is. |
| `.fields(false)` | Hide the model selector and the numeric fields. |
| `.swatches(binding)` | A palette the user can add colors to and remove from. Selecting a swatch is not an edit. |
| `.on_edit(f)` | The [`EditPhase`](#make-each-edit-one-undo-step) of each drag, key press or typed entry. |

`alpha`, `fields` and `wheel_size` also accept a signal or a closure, so they can change while the app runs. When saturation reaches zero, the wheel remembers the last hue, so dragging back out of gray does not jump.

## Make each edit one undo step

All the editors of this guide, like the numeric fields, report an edit as three phases through `on_edit`:

- `EditPhase::Begin` arrives before the first write. Remember the value here.
- `EditPhase::Commit` arrives after the last write. Close the undo step here.
- `EditPhase::Cancel` arrives after the editor wrote the starting value back (the user pressed `Esc`, or the editor was hidden or removed in the middle of an edit). Drop what you remembered; the binding already holds the old value.

A drag is a single edit from press to release. Each key press and each typed entry is its own edit, and assistive technology uses the same phases. A small history that works for any `Clone` value:

```rust,no_run
use kivact::prelude::*;
use kivact::view::EditPhase;

/// Records `(before, after)` pairs for the value in `value`.
fn history<T: Clone + PartialEq + 'static>(
    value: Signal<T>,
) -> (impl Fn(EditPhase) + 'static, Signal<Vec<(T, T)>>) {
    let start = signal(None::<T>);
    let steps = signal(Vec::<(T, T)>::new());
    let on_edit = move |phase: EditPhase| match phase {
        EditPhase::Begin => start.set(Some(value.get_untracked())),
        EditPhase::Commit => {
            if let Some(before) = start.update(|saved| saved.take()) {
                steps.update(|all| all.push((before, value.get_untracked())));
            }
        }
        EditPhase::Cancel => start.set(None),
    };
    (on_edit, steps)
}
```

Pass `on_edit` to any editor of the same value: `color_wheel(color, model).on_edit(on_edit)`. To undo, pop the last pair and write `before` to the signal.

## Edit an easing curve

`bezier_curve` edits a `CubicBezier`: the two inner control points of a cubic easing curve between the fixed ends (0, 0) and (1, 1). `curve_preview` draws read-only samples, and `progress` evaluates the curve for your animation code.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn easing_editor() -> impl IntoView {
    let easing = signal(ui::CubicBezier::new(0.25, 0.1, 0.25, 1.0));
    column((
        ui::bezier_curve(easing),
        ui::curve_preview(move || easing.get().samples(64)).style(style().size(240, 120)),
        text(move || format!("halfway: {:.3}", easing.get().progress(0.5))),
    ))
    .style(style().gap(8).padding(12))
}
```

Drag the two handles, or focus one and use the arrow keys (`0.01` per press, `Shift` for `0.1`, `Ctrl` for `0.001`). Both the editor and the preview read the same `easing`, so the preview follows every change.

- `CubicBezier::new(x1, y1, x2, y2)` limits `x1` and `x2` to `0..=1`, so time always moves forward. `y1` and `y2` may go outside `0..=1`; the curve then overshoots, which is how bounce and anticipation easings are made.
- `progress(t)` returns the eased value for a time `t` in `0..=1`; a time outside is limited to that range. The result is not limited, so it keeps the overshoot. In an animation, evaluate it with your normalized time: `start + (end - start) * easing.progress(t)`.
- `samples(n)` returns `n` evenly spaced points (from 2 to 513). `curve_preview` takes any list of `Point`s, so it can draw other curves, too.
- A binding value that is invalid (for example `NaN` from a corrupt file) is cleaned for drawing, but your signal is not rewritten until the user edits.

## Edit a gradient

`gradient_editor(stops, selected)` shows a color ramp with draggable stops and, for the selected stop, a position field and a color wheel.

```rust,no_run
use kivact::draw::linear_gradient;
use kivact::prelude::*;
use kivact::style::rgb;
use kivact_ui_creative as ui;

fn gradient_panel() -> impl IntoView {
    let stops = signal(vec![
        ui::GradientStop::new(0.0, rgb(0x1b1c22)),
        ui::GradientStop::new(1.0, rgb(0x8b7cff)),
    ]);
    // The index of the selected stop in `stops`, if any.
    let selected = signal(Some(0));

    column((
        ui::gradient_editor(stops, selected).name("Background gradient"),
        // Your own use of the stops: a preview strip.
        node().style(move || {
            let mut ordered = stops.get();
            ordered.sort_by(|a, b| a.position.total_cmp(&b.position));
            let mut ramp = linear_gradient(90);
            for stop in ordered {
                ramp = ramp.stop(stop.color, pct(stop.position * 100.0));
            }
            style().height(24).radius(6).background(ramp)
        }),
    ))
    .style(style().gap(10).padding(12).width(300))
}
```

- **Stops have no ID.** A stop is identified by its index in your vector, and `selected` is that index (or `None`). Adding or deleting a stop adjusts the selected index for you.
- **Your vector is never reordered.** The editor draws the stops by position but leaves their order in your data alone, so sort a copy when you need an ordered list, as the preview above does.
- **Keys:** with a stop focused, `Left`/`Down` and `Right`/`Up` move it by `0.01` (`Shift`: `0.1`), `Home` and `End` send it to `0` and `1`, and `Delete` or `Backspace` removes it. The editor's own buttons add and remove stops.
- **Changing the selection is not an edit.** `on_edit` fires for moves, additions, removals and color changes only.
- **Color spaces.** If you store colors in another space than the screen's, `.color_transform(to_display, from_display)` converts when a stop is shown and when it is written back.

## See also

- [Show frames your application renders](show-app-rendered-images.md): a histogram or scope you draw can sit next to a color wheel.
- [Theme the kit and add icons](theme-and-icons.md)
- [Tutorial step 2: Edit values](../getting-started/02-edit-values.md): the same `EditPhase` with numeric fields.
- Full API: `cargo doc -p kivact-ui-creative --open`
- The catalog's grading and animation chapters: `cargo run -p kivact-ui-creative-catalog`
