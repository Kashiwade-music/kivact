# 2. Edit values

[日本語](../../../jp/creative/getting-started/02-edit-values.md)

In this step the inspector grows into a small property editor: position and opacity fields that share one value, an undo button that reacts to every scrub, key press or typed entry, a fill color you pick in a popup, and an easing curve you drag into shape.

Start from the project of [step 1](01-set-up-the-kit.md). Replace `src/main.rs`:

```rust,no_run
use kivact::prelude::*;
use kivact::style::{ThemeMode, rgb};
use kivact::view::EditPhase;
use kivact_ui_creative as ui;

fn inspector() -> impl IntoView {
    let x = signal(120.0);
    let opacity = signal(0.8);
    let fill = signal(rgb(0x8b7cff));
    let easing = signal(ui::CubicBezier::default());

    // One undo step per edit of X: (value before, value after).
    let started = signal(0.0);
    let history = signal(Vec::<(f64, f64)>::new());
    let record = move |phase: EditPhase| match phase {
        EditPhase::Begin => started.set(x.get_untracked()),
        EditPhase::Commit => {
            history.update(|steps| steps.push((started.get_untracked(), x.get_untracked())));
        }
        EditPhase::Cancel => {}
    };
    let undo = move || {
        if let Some((before, _)) = history.update(|steps| steps.pop()) {
            x.set(before);
        }
    };

    ui::panel(
        "Inspector",
        column((
            ui::property_row(
                "X",
                ui::number("X", x)
                    .range(kivact::interaction::ScalarRange::new(-1000.0, 1000.0).expect("valid range"))
                    .step(1.0)
                    .precision(1)
                    .unit(" px")
                    .label_visible(false)
                    .reset_to(0.0)
                    .on_edit(record),
            ),
            ui::property_row(
                "Opacity",
                ui::number("Opacity", opacity)
                    .range(kivact::interaction::ScalarRange::new(0.0, 1.0).expect("valid range"))
                    .step(0.01)
                    .precision(2)
                    .label_visible(false),
            ),
            ui::property_row(
                "Fade",
                ui::value_slider("Fade", opacity)
                    .range(kivact::interaction::ScalarRange::new(0.0, 1.0).expect("valid range"))
                    .step(0.01)
                    .precision(2)
                    .label_visible(false),
            ),
            ui::property_row("Fill", ui::color_popup("Fill", fill)),
            ui::bezier_curve(easing),
            text(move || {
                let e = easing.get();
                format!("ease {:.2}, {:.2}, {:.2}, {:.2}", e.x1, e.y1, e.x2, e.y2)
            }),
            row((
                button("Undo X").disabled(move || history.get().is_empty()).on_press(undo),
                text(move || format!("{} undo step(s)", history.get().len())),
            ))
            .style(style().gap(8)),
        )).role(kivact::view::Role::List)
        .style(style().gap(6).padding(10)),
    )
    .style(style().width(300))
}

fn main() -> kivact::Result<()> {
    App::new("Layer inspector")
        .theme(ui::theme(ThemeMode::Dark))
        .run(inspector)
}
```

Run it. You should see the panel from step 1 with more rows, an easing curve with two handles, and an **Undo X** button (disabled for now) with the text `0 undo step(s)` beside it.

## Try it

1. **Scrub.** Press on the **X** field, drag right and release. The value follows the pointer and the text changes to `1 undo step(s)` once, not once per pixel. Press **Undo X** and the field returns to where the drag began.
2. **Cancel.** Drag again but press `Esc` before releasing. The value jumps back to where the drag started and the count does not change: a cancelled edit is not an undo step.
3. **Keys and typing.** Focus the field with `Tab`, press `Right`: that single key press is one edit. Press `Enter` to open the text field, type `10 + 5` and press `Enter`: that is another. `Backspace` (or a double click) resets to `0`, the value you gave to `reset_to`.
4. **One value, two editors.** Drag the **Fade** slider. The **Opacity** field follows, and the other way round. Both were given the same `opacity` signal, and neither keeps a copy, so there is nothing to synchronize.
5. **Color.** Click the **Fill** swatch. A popup opens with a color wheel. Drag in it, then press `Esc` to put the original color back, or click outside the popup to keep the new one.
6. **Curve.** Drag one of the two round handles of the easing curve. The numbers under it follow. Press `Esc` during the drag and the curve snaps back. With a handle focused, the arrow keys nudge it by 0.01 (`Shift` for 0.1).

## What you wrote

- **The app owns the values.** Every component takes the value it edits as an argument. The kit never stores your data; it only reads and writes the signal you give it. This is the same idea as a controlled component in React.
- **`on_edit` reports an edit as three phases.** `EditPhase::Begin` arrives *before* the first write of an edit, so it is the place to remember the starting value. `Commit` arrives after the last write, which is where you close the undo step. `Cancel` arrives after the component has already written the starting value back, so there is nothing to undo; you only drop what you remembered. A drag, however long, is one edit; each key press, wheel notch and typed entry is its own. Assistive technology uses the same phases, so an undo history built on them covers every way to change a value.
- **`move` closures.** `record` and `undo` use `move` so they own copies of the signal handles. Handles are `Copy`, so the same `x` is still usable after it has been moved into a closure.
- **`color_popup(label, color)`** is a swatch button that opens a `color_wheel` in a popup. It is a transaction: `Esc` restores the color the popup opened with, and an outside click commits.
- **`bezier_curve(binding)`** edits a `CubicBezier`, the control points of an easing curve whose ends stay at (0, 0) and (1, 1). It, too, takes its value as an argument and reports `on_edit` phases.
- **`value_slider`** is a numeric field drawn as a filled track. `ui::knob` and `ui::fader` are other shapes of the same component and take the same arguments. The plain track-and-thumb slider is `kivact::ui::slider`. All three also take `format` and `mapping` when the value needs its own text or curve; [Format, map and fine-tune knobs and faders](../how-to-guide/knobs-and-faders.md) covers them.

## Next

[3. Put it in a dock](03-dock-and-outliner.md)
