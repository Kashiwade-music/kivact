# Format, map and fine-tune knobs and faders

[日本語](../../../jp/creative/how-to-guide/knobs-and-faders.md)

The Creative kit's numeric controls (`number`, `value_slider`, `knob`, `fader`)
share one implementation: the same binding, range, steps, keys, wheel and edit
phases. This guide shows how to make them read the way your users think about
the value, follow the right curve, and behave like a plugin control under the
pointer.

## Show the value the way your users read it

`format` replaces the built-in text on the control, in the text field, and in
what assistive technology reads. When the text is not a plain number in the
value's units (a linear gain shown in dB), give `text_parser` as well so typed
text and an assistive `SetValue` read back the same way.

```rust,test_harness
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn gain() -> impl IntoView {
    let gain = signal(0.5);
    ui::knob("Gain", gain)
        .range(0.0..=2.0)
        .reset_to(1.0)
        .format(|gain| {
            if gain > 0.0 {
                format!("{:.1} dB", 20.0 * gain.log10())
            } else {
                "-inf dB".to_owned()
            }
        })
        .text_parser(|text, _| {
            let db: f64 = text
                .trim()
                .trim_end_matches("dB")
                .trim()
                .parse()
                .map_err(|_| "Enter decibels, such as -6.0".to_owned())?;
            Ok(10f64.powf(db / 20.0))
        })
}
```

The order is fixed:

| Text | Source |
|---|---|
| the control, the field's value and the accessible value text | the mixed message while `mixed(true)`, else `format`, else `timecode`, else `precision` and `unit` |
| the text the field opens with | `format(value)` when set, else the built-in text without the unit |
| typed text and an assistive `SetValue` that is not a plain number | `text_parser` when set, else the built-in numbers, arithmetic and timecode |

`format` receives the value clamped to the range and may read signals, so a
unit switch or a locale follows without rebuilding the control. `precision`
and `unit` still set the step and the built-in display.

## Map the value to the travel

By default equal travel changes the value by equal amounts. `mapping` changes
that without changing the units of the binding:

| Mapping | Position |
|---|---|
| `ScalarMapping::linear()` | the default; equal value steps for equal travel |
| `ScalarMapping::logarithmic()` | equal ratios (frequency, time); linear when the range's minimum is not positive |
| `ScalarMapping::skewed(0.5)` | the square-root curve a fader uses: below 1 the low end gets more travel, above 1 the high end |
| `ScalarMapping::custom(to_position, to_value)` | your parameter's own normalization |

A fader already uses `ScalarMapping::skewed(0.5)`. Call `mapping` to replace it.
`step` takes a `ScalarStep`: `Value(..)` in value units (the default) or
`Position(..)`, a fraction of the travel. A position step is how one arrow key
moves a logarithmic knob by one percent wherever it is.

```rust,test_harness
use kivact::interaction::{ScalarMapping, ScalarStep};
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn frequency() -> impl IntoView {
    ui::knob("Frequency", signal(440.0))
        .range(20.0..=20_000.0)
        .mapping(ScalarMapping::logarithmic())
        .step(ScalarStep::Position(0.01))
        .unit(" Hz")
        .precision(0)
}
```

The text, the bounds, Home/End, value steps and `reset_to` stay in the value's
units; only the drag, the touch, a position step and the drawing use the
position. `ScalarState::fraction` (from `use_scalar`) is that position, which a
canvas knob uses for its angle.

## Set the drag distance

A knob moves the range over its own height (at least 60 px). Plugin knobs
usually want a fixed sensitivity: `drag_distance` is the pointer travel, in
logical pixels, that crosses the whole range through the mapping.

```rust,test_harness
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn threshold() -> impl IntoView {
    ui::knob("Threshold", signal(-18.0))
        .range(-60.0..=0.0)
        .unit(" dB")
        .drag_distance(200.0)
}
```

A value that is not positive and finite acts as the default, so a reactive
`drag_distance` can come from a setting. A number field scrubs one step per
pixel unless `drag_distance` is set.

## Fine and coarse keys, and the reset click

The modifier keys of scrubbing, the arrow keys and the wheel are one theme
policy, `ScrubPolicy`. The defaults are Ctrl for fine (×0.1), Shift for coarse
(×10) and no reset click. Set them once in the editor's theme:

```rust,no_run
use kivact::interaction::{InteractionPolicy, ModifierKey};
use kivact::style::ThemeMode;
use kivact_ui_creative as ui;

fn editor_theme() -> kivact::style::Theme {
    let theme = ui::theme(ThemeMode::Dark);
    let mut policy = (*theme.ext::<InteractionPolicy>()).clone();
    policy.scrub.fine = Some(ModifierKey::Shift);
    policy.scrub.coarse = None;
    policy.scrub.reset_click = Some(ModifierKey::Alt);
    theme.with(policy)
}
```

Changing the factor in the middle of a scrub does not jump: movement so far
keeps the factor that applied to it, the new factor applies to later movement,
and a value that saturated or was quantized restarts from where it is. Pressing
or releasing the modifier alone dispatches an update at the same pointer
position, so the next movement starts from the new anchor.

The wheel uses the fine key only; the coarse key is ignored, because Shift
turns a wheel sideways on some platforms. A wheel notch is 60 logical pixels of
scroll (`wheel_notch`), so a touchpad's small deltas add up: two 20 px events
write nothing, the third makes one step. A reversed scroll starts a new
accumulation. Each event that writes at least one notch is one edit; a
sub-notch event reports nothing.

## Type a value on a knob

A number or value_slider opens its text field on Enter and on a click that does
not move. A knob or fader opens it on Enter when `text_entry(true)`; the field
is shown over the control. Typing, confirming with Enter or leaving the field,
and Escape behave as on a number field: confirming is one edit, Escape discards
the draft and reports nothing.

```rust,test_harness
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn mix() -> impl IntoView {
    ui::knob("Mix", signal(100.0))
        .range(0.0..=100.0)
        .reset_to(100.0)
        .unit(" %")
        .text_entry(true)
}
```

## Keys, PageUp/PageDown and reset

| Input | Effect |
|---|---|
| ArrowUp / ArrowRight | one key step |
| ArrowDown / ArrowLeft | one key step down |
| PageUp / PageDown | ten key steps |
| Home / End | the range's minimum / maximum |
| Enter | opens the text field (when `text_entry`) |
| Backspace, double click, the reset-click modifier with a primary press | `reset_to` |

Each of these is one edit. A scrub owns the value from press to release: the
keys, the wheel and assistive technology write nothing while it lasts.

## Edit phases and mixed values

Every write goes through `use_scalar`, so `on_edit` reports `Begin`, `Commit`
and `Cancel` around a whole scrub, around each key press or wheel notch, and
around confirmed text. `Cancel` arrives after the starting value has been
written back.

`mixed(true)` shows the mixed message, marks the control indeterminate and
turns off scrubbing, steps, the wheel and the reset click. Typed text or an
assistive `SetValue` are the only writes; a number or value_slider opens its
text field on a press, a knob or fader with `text_entry` with Enter.
