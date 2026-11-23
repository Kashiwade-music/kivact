# Step 2: Build the panel

[日本語](../../../jp/game/getting-started/02-build-the-panel.md)

In this step you fill the framed panel with a numbered heading, a status
badge, three kinds of meter and buttons that change the meters.

## The program

Replace `src/main.rs` with:

```rust,no_run
use kivact::interaction::Tone;
use kivact::prelude::*;
use kivact::style::{Align, FontStyle, ThemeMode};
use kivact::ui::{ButtonVariant, progress};
use kivact_ui_game::{self as game, Density, ThemeOptions};

/// A monospaced caption over a meter.
#[component]
fn labeled(label: Value<String>, meter: View) -> impl Element {
    column((game::mono(label), meter)).style(style().gap(6))
}

#[component]
fn loadout() -> impl Element {
    let armor = signal(4_usize);
    let energy = signal(76.0_f64);

    game::hud(
        column((
            row((
                text("01").style(move || {
                    let t = kivact::style::use_theme().get();
                    style()
                        .font_family(t.get(&game::fonts::DISPLAY_FAMILY_VAR).as_str().unwrap_or_default())
                        .font_size(34)
                        .line_height(0.9)
                        .font_weight(800)
                        .font_style(FontStyle::Italic)
                        .color(kivact::style::color::ACCENT)
                }),
                kivact::ui::heading("LOADOUT", 2).variant(game::HeadingVariant::Section),
                text("Squad 04").style(style().font_size(12).color(kivact::style::color::TEXT_MUTED)),
            ))
            .style(
                style()
                    .gap(14)
                    .align_items(Align::End)
                    .padding_bottom(10)
                    .border_bottom(2, kivact::style::color::TEXT),
            )
            .draw_over(|d, s| {
                d.fill_path(
                    &kivact::draw::Path::new()
                        .move_to(kivact::geometry::Point::new(s.width - 44., s.height - 4.))
                        .line_to(kivact::geometry::Point::new(s.width, s.height - 4.))
                        .line_to(kivact::geometry::Point::new(s.width - 2., s.height + 6.))
                        .line_to(kivact::geometry::Point::new(s.width - 46., s.height + 6.))
                        .close(),
                    game::color::YELLOW,
                )
            }),
            row((
                kivact::ui::badge("READY").tone(Tone::Success),
                game::mono("SLOT 2"),
            ))
            .style(style().gap(12).align_items(Align::Center)),
            row((
                labeled(
                    "ARMOR",
                    kivact::ui::progress(move || Some(armor.get() as f64 / 10.0))
                        .name("Armor")
                        .variant(kivact::ui::ProgressVariant::Gauge)
                        .variant(game::GaugeSegments(10))
                        .variant(Tone::Info),
                ),
                labeled(
                    "ENERGY",
                    kivact::ui::progress(move || Some(energy.get() / 100.0))
                        .name("ENERGY")
                        .variant(kivact::ui::ProgressVariant::Ring)
                        .variant(kivact::ui::ProgressMotion {
                            determinate_ms: 900,
                            determinate_easing: kivact::motion::Easing::CubicBezier(
                                0.2, 0.7, 0.2, 1.,
                            ),
                            ..Default::default()
                        })
                        .variant(Tone::Warning),
                ),
            ))
            .style(style().gap(32).align_items(Align::Center)),
            labeled(
                "EXPERIENCE",
                progress(Some(0.62))
                    .name("Experience")
                    .variant(kivact::ui::Striped)
                    .moving(true),
            ),
            row((
                button("Upgrade armor")
                    .variant(kivact::ui::Slant)
                    .disabled(move || armor.get() >= 10)
                    .on_press(move || armor.update(|a| *a += 1)),
                button("Recharge")
                    .variant(ButtonVariant::Secondary)
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
        .theme(game::theme_with(
            ThemeMode::Dark,
            ThemeOptions {
                density: Density::Comfortable,
                ..Default::default()
            },
        ))
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
A row of a section number, `kivact::ui::heading(title, 2).variant(game::HeadingVariant::Section)` and a caption composes the section heading.

**`kivact::ui::badge(label).tone(tone)`** is a small status label with a square dot.
`Tone` is Kivact's shared meaning enum: `Neutral`, `Info`, `Success`,
`Warning`, `Danger` and `Accent`. You say what the status means and the
theme picks the color.

**`labeled(label, meter)`** is a small component of this program: it stacks a
monospaced caption over a meter. Its `meter` is a `View`, so any meter fits.

**`progress(..).variant(ProgressVariant::Gauge)`** draws slanted bars. Normalize the application value by its total: `armor.get() as f64 / 10.0` is the fraction of the ten bars. `game::GaugeSegments(10)` sets the number of segments. Give the meter its accessible name with `.name(..)` and draw a caption beside it.
**`ProgressVariant::Ring`** draws a segmented circular meter from the same kind of fraction. The standard progress owner animates its arc.

**`progress(Some(value))`** is the standard progress bar, with a value from 0
to 1. The Game theme draws it flat. `.variant(kivact::ui::Striped)` and
`.moving(true)` add moving diagonal stripes. A bar has no visible or
accessible label of its own, so give it one with `.name(..)`.

**Buttons.** `.variant(kivact::ui::Slant)` slants a button; `.variant(ButtonVariant::Secondary)`
makes it the console's navy secondary button. The standard `ButtonVariant` values still
work, and you can combine them: a variant of a different type adds to the
earlier ones.

**Rust note.** `armor.update(|a| *a += 1)` changes the value in place; `a` is a
mutable reference to the number inside the signal. `move ||` hands the
closure its own copy of the signal handle, which is cheap: signals are `Copy`.

Next: [Step 3: Tabs and items](03-tabs-and-items.md)
