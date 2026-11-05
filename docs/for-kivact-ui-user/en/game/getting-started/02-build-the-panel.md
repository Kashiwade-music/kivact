# Step 2: Build the panel

[日本語](../../../jp/game/getting-started/02-build-the-panel.md)

In this step you fill the framed panel with a numbered heading, a status
badge, three kinds of meter and buttons that change the meters.

## The program

Replace `src/main.rs` with:

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::interaction::{Tone};
use kivact::ui::{progress};
use kivact_ui_game::{self as game, Density, ThemeOptions};

/// A monospaced caption over a meter.
fn labeled(label: &'static str, meter: impl IntoView) -> Node {
    column((game::mono(label), meter)).style(style().gap(6))
}

fn loadout() -> impl IntoView {
    let armor = signal(4_usize);
    let energy = signal(76.0_f64);

    game::hud(
        column((
            game::section_heading("01", "LOADOUT", "Squad 04"),
            row((
                game::badge("READY", Tone::Success),
                game::mono("SLOT 2"),
            ))
            .style(style().gap(12).align_items(Align::Center)),
            row((
                labeled(
                    "ARMOR",
                    game::gauge("Armor", armor).total(10).tone(Tone::Info),
                ),
                game::ring("ENERGY", energy, Tone::Warning),
            ))
            .style(style().gap(32).align_items(Align::Center)),
            labeled(
                "EXPERIENCE",
                progress(Some(0.62))
                    .name("Experience")
                    .variant(game::Striped)
                    .moving(true),
            ),
            row((
                button("Upgrade armor")
                    .variant(game::Slant)
                    .disabled(move || armor.get() >= 10)
                    .on_press(move || armor.update(|a| *a += 1)),
                button("Recharge")
                    .variant(game::ButtonTone::Dark)
                    .on_press(move || energy.update(|e| *e = (*e + 10.0).min(100.0))),
            ))
            .style(style().gap(10)),
        ))
        .style(style().gap(18).padding(20)),
    )
    .style(style().width(420).margin(24))
}

fn main() -> kivact::Result<()> {
    App::new("Loadout")
        .theme(game::theme_with(ThemeMode::Dark, ThemeOptions { density: Density::Comfortable }))
        .run(loadout)
}
```

Run it. You should see a heading with a large slanted number and a yellow mark
on its underline, a green READY badge, a row of ten slanted bars with four
filled, a segmented circle that reads 76% above the word ENERGY, a striped bar
that moves, and two buttons. Press **Upgrade armor** and one more bar fills; after the tenth
the button is disabled. Press **Recharge** and the ring climbs by ten until
it reaches 100.

## What each part does

**`game::hud(content)`** is the frame from step 1: a surface with a border and
two accent brackets. Put one around each group of related controls.
**`game::section_heading(number, title, description)`** is the numbered
heading of a screen section.

**`game::badge(label, tone)`** is a small status label with a square dot.
`Tone` is Kivact's shared meaning enum: `Neutral`, `Info`, `Success`,
`Warning`, `Danger` and `Accent`. You say what the status means and the
theme picks the color.

**`game::gauge(label, value)`** is a row of slanted bars, one per point of
`value`, out of `.total(n)` (100 if you do not set it). The first argument is
the accessible name, not a visible label, so the program draws its own caption
with `labeled`. A gauge reads a `Signal<usize>`, so the bars follow the signal.
**`game::ring(label, value, tone)`** is a circular meter for a value from 0 to
100. It prints the percentage with the label under it in the middle, and the
arc grows to the value when the meter first appears and whenever it changes.

**`progress(Some(value))`** is the standard progress bar, with a value from 0
to 1. The Game theme draws it flat. `.variant(game::Striped)` and
`.moving(true)` add moving diagonal stripes. A bar has no visible or
accessible label of its own, so give it one with `.name(..)`.

**Buttons.** `.variant(game::Slant)` slants a button; `game::ButtonTone::Dark`
makes it the navy console button. The standard `ButtonVariant` values still
work, and you can combine them: a variant of a different type adds to the
earlier ones.

**Rust note.** `armor.update(|a| *a += 1)` changes the value in place; `a` is a
mutable reference to the number inside the signal. `move ||` hands the
closure its own copy of the signal handle, which is cheap: signals are `Copy`.

Next: [Step 3: Tabs and items](03-tabs-and-items.md)
