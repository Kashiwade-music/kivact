# Show meters and status

[日本語](../../../jp/game/how-to-guide/meters-and-status.md)

Use this guide to show how things stand: health and energy bars, a status
label, key figures, a property sheet and an event log. These are display
components. They draw the values you give them and never change them, so a live
meter is a signal you update.

## Show a live value as a meter

Three meters accept a signal and redraw when it changes.

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::{Tone};
use kivact::ui::{progress};
use kivact_ui_game as game;

fn vitals() -> impl IntoView {
    let hp = signal(7_usize);
    let shield = signal(64.0_f64);
    let loading = signal(0.35_f64);
    column((
        game::mono("HULL"),
        game::gauge("Hull", hp).total(10).tone(Tone::Danger),
        game::ring("SHIELD", shield, Tone::Info),
        progress(move || Some(loading.get()))
            .name("Loading")
            .variant(Tone::Success),
        button("Take a hit").on_press(move || {
            hp.update(|hp| *hp = hp.saturating_sub(1));
            shield.update(|s| *s = (*s - 8.0).max(0.0));
        }),
    ))
    .style(style().gap(12).margin(24))
}
# fn main() {}
```

| Meter | Value | Looks like |
| --- | --- | --- |
| `game::gauge(label, value)` | a count of bars, out of `.total(n)` (100 by default) | a row of slanted bars |
| `game::ring(label, percent, tone)` | 0 to 100 | a segmented circle with the percentage |
| `kivact::ui::progress(value)` | `Some(0.0..=1.0)`, or `None` while unknown | a flat bar |

- The first argument of `gauge` is its accessible name (the meter has
  `Role::Meter`); it is not drawn. Draw a caption with `game::mono(..)`.
- `.tone(Tone::..)` colors a gauge and `ring` takes the tone as its last
  argument. A progress bar takes the tone as its variant:
  `.variant(Tone::Success)`.
- `.size(ControlSize::Small)` makes a progress bar thinner. `.variant(game::Striped)`
  with `.moving(true)` adds moving stripes for a busy bar.
- A progress bar has no label of its own: name it with `.name(..)`.
- `Tone` is `Neutral`, `Info`, `Success`, `Warning`, `Danger` or `Accent`. You
  say what the value means; the theme chooses the color.

## Show a key figure

`game::kpi(label, target)` is a tile with a big number that counts up from zero
when it first appears.

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn scoreboard() -> impl IntoView {
    row((
        game::kpi("KILLS", 1284.0)
            .decimals(0)
            .change("+12.4%")
            .tone(Tone::Info)
            .trend(vec![3.0, 5.0, 4.0, 8.0, 9.0, 12.0])
            .style(style().grow(1)),
        game::kpi("ACCURACY", 87.5)
            .decimals(1)
            .unit("%")
            .tone(Tone::Success)
            .style(style().grow(1)),
    ))
    .style(style().gap(16).margin(24))
}
# fn main() {}
```

- `.decimals(n)` sets the digits after the decimal point (0 by default);
  `.format(|value| String)` replaces the number formatting; `.unit(..)` adds a
  unit.
- `.change(text)` is a small line of text for the change (you write the text
  and the arrow); `.tone(..)` colors it.
- `.trend(samples)` draws a sparkline of the `f64` samples. Without it no line
  is drawn.
- `.heading(|| view)` replaces the caption with your own view, such as an icon
  and text.
- The target is a constant. The count-up plays once; for a number that changes
  while the screen is open, use a `text`, `game::mono` or a gauge over a signal.

## Show a status

`kivact::ui::badge(label).tone(tone)` is a short status label with a square dot.
`kivact::ui::badge(label).tone(tone) .variant(BadgeVariant::..)` picks the look: `Dot` (the
default), `Plain`, `Solid` or `Outline`. Both return a node that takes the usual
`.style(..)`, `.test_id(..)` and other modifiers.

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn statuses() -> impl IntoView {
    row((
        kivact::ui::badge("RUNNING").tone(Tone::Info),
        kivact::ui::badge("v2.4.1").tone(Tone::Neutral) .variant(game::BadgeVariant::Outline),
        kivact::ui::badge("OFFLINE").tone(Tone::Danger) .variant(game::BadgeVariant::Solid),
    ))
    .style(style().gap(8).margin(24))
}
# fn main() {}
```

## Show a message to the player

- `game::alert(title)` is an inline box with `.description(..)`, `.tone(..)`
  and `.actions(|| view)`. It shows a close button only when you give it
  `.on_dismiss(..)`; the handler reports the request, and you hide the alert by
  clearing your own signal (see the example below).
- `game::banner(title, tone, action)` is a full-width tape for system
  messages; `.content(|| view)` is its message and `action` is whatever you
  want at its end.
- `kivact::ui::empty_state(title)` is a dashed panel with `.description(..)` and
  `.actions(view)` for an action; `kivact::ui::skeleton().variant(game::SkeletonVariant::Rows).lines(n)` is a loading
  placeholder.
- Short-lived messages are toasts: mount `kivact::ui::toast_region(toaster())`
  once and push `Toast::new("Saved").tone(Tone::Success)` into the toaster.

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn warning() -> impl IntoView {
    let shown = signal(true);
    show(shown, move || {
        game::alert("Low fuel")
            .tone(Tone::Warning)
            .description("Return to base within 2 minutes.")
            .on_dismiss(move || shown.set(false))
    })
}
# fn main() {}
```

## Show properties and a log

`game::property_list(entries)` is a sheet of key and value rows. The keys are
strings; the values are texts or any views, as long as all of them have the
same type. Turn a view into the shared type with `.into_view()` to mix:

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn sheet() -> impl IntoView {
    game::property_list(vec![
        ("CALLSIGN".to_owned(), game::mono("VIPER-02").into_view()),
        ("STATUS".to_owned(), kivact::ui::badge("ACTIVE").tone(Tone::Success).into_view()),
        ("SQUAD".to_owned(), text("Alpha").into_view()),
    ])
}
# fn main() {}
```

`game::activity(entries)` is a timeline of `ActivityEntry::new(time, message,
tone)`. The message is a string or a view. The entries own their views and are
used once when the timeline is built, so a log that grows is rebuilt from your
signal with `dynamic`. Property lists are built once in the same way; put a
changing value inside its view (`game::mono(move || ..)`).

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

fn log(events: Signal<Vec<(String, String)>>) -> impl IntoView {
    dynamic(events, |events| {
        game::activity(
            events
                .into_iter()
                .map(|(time, message)| game::ActivityEntry::new(time, message, Tone::Info))
                .collect(),
        )
    })
}
# fn main() {}
```

## See also

- [Set the theme and density](theme-and-density.md) for colors and fonts
- [Show items in lists and trees](item-lists-and-selection.md)
- Every component with its options: `cargo doc -p kivact-ui-game --open`

Use `kivact::ui::avatar(name)` with `.fallback_content(..)` when supplying exact initials rather than initials derived from a full name. `.variant(game::AvatarVariant { side: 32., color })` chooses logical-pixel dimensions and the console fill; `.status(Some(Tone::Success))` adds the square presence marker. Group avatars with `kivact::ui::avatar_group(children).max(3usize)`. For partially loaded groups, `.total(8usize)` states the total participant count, including the supplied avatars; three visible avatars then show the styled `+5` tile. The theme sets the overlap to six logical pixels.
