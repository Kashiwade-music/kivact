# Step 2: Build the panel

[日本語](../../../jp/game/getting-started/02-build-the-panel.md)

In this step you fill the framed panel with a numbered heading, a status
badge, three kinds of meter and buttons that change the meters.

## The program

Replace `src/main.rs` with:

```rust,no_run
use kivact::prelude::*;
use kivact::style::Align;
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
            row((
    text("01").style(move || { let t=kivact::style::use_theme().get(); style().font_family(t.ext::<game::Tokens>().display_family.as_str()).font_size(34).line_height(0.9).weight(800).italic(true).color(ColorToken::Accent) }),
    kivact::ui::heading("LOADOUT", 2).variant(game::HeadingVariant::Section),
    text("Squad 04").style(style().font_size(12).color(ColorToken::TextMuted)),
)).style(style().gap(14).align_items(Align::End).padding_bottom(10).border_bottom(2, ColorToken::Text))
.draw_over(|d,s| d.fill_path(&kivact::draw::Path::new().move_to(kivact::geometry::Point::new(s.width-44.,s.height-4.))
    .line_to(kivact::geometry::Point::new(s.width,s.height-4.)).line_to(kivact::geometry::Point::new(s.width-2.,s.height+6.))
    .line_to(kivact::geometry::Point::new(s.width-46.,s.height+6.)).close(), d.theme().ext::<game::Tokens>().yellow)),
            row((
                kivact::ui::badge("READY").tone(Tone::Success),
                game::mono("SLOT 2"),
            ))
            .style(style().gap(12).align_items(Align::Center)),
            row((
                labeled(
                    "ARMOR",
                    { let total: kivact::state::Value<usize> = kivact::state::IntoValue::into_value(10);
    let value: kivact::state::Value<usize> = kivact::state::IntoValue::into_value(armor);
    kivact::ui::progress(move || Some(if total.get()==0 {0.0} else {value.get() as f64/total.get() as f64}))
        .name("Armor").variant(kivact::ui::ProgressVariant::Gauge)
        .variant(game::GaugeSegments(total)).variant(Tone::Info) },
                ),
                { let value: kivact::state::Value<f64> = kivact::state::IntoValue::into_value(energy);
    kivact::ui::progress(move || Some(value.get()/100.0)).name("ENERGY")
        .variant(kivact::ui::ProgressVariant::Ring).variant(Tone::Warning)
        .with_node(|node| node.child(game::mono("ENERGY").style(style().font_size(9).color(ColorToken::TextMuted)))) },
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
A row of a section number, `kivact::ui::heading(title, 2).variant(game::HeadingVariant::Section)` and a caption composes the section heading.

**`kivact::ui::badge(label).tone(tone)`** is a small status label with a square dot.
`Tone` is Kivact's shared meaning enum: `Neutral`, `Info`, `Success`,
`Warning`, `Danger` and `Accent`. You say what the status means and the
theme picks the color.

**`progress(..).variant(ProgressVariant::Gauge)`** draws slanted bars. Normalize the application value by its total, returning zero when the total is zero. `game::GaugeSegments(total)` supplies the same reactive segment count. Give the meter its accessible name with `.name(..)` and draw a caption beside it.
**`ProgressVariant::Ring`** draws a segmented circular meter from the same normalized fraction. Add the small center caption through the ordinary child slot. The standard progress owner animates its arc.

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
