# Meters and status

[日本語](../../../jp/game/how-to-guide/meters-and-status.md)

Use the standard `kivact::ui` components under the Game theme. Their recipes draw the console's slanted segments, segmented rings, KPI tiles, status badges and feedback panels.

## Show progress

Progress values are fractions from zero to one. A gauge converts the application's count to `value / total`, guarding a zero total. `game::GaugeSegments` takes the same reactive total to choose the number of bars; it owns no progress state. A ring uses `ProgressVariant::Ring` and can include a small caption. `game::Striped` adds moving diagonal stripes to a bar.

```rust,no_run
use kivact::interaction::Tone;
use kivact::prelude::*;
use kivact::ui::{self, ProgressVariant};
use kivact_ui_game as game;

fn meters() -> impl IntoView {
    let bars = signal(7usize);
    let total = signal(10usize);
    let percent = signal(76.0);
    row((
        ui::progress(move || {
            Some(if total.get() == 0 {
                0.0
            } else {
                bars.get() as f64 / total.get() as f64
            })
        })
        .name("Hull")
        .variant(ProgressVariant::Gauge)
        .variant(game::GaugeSegments(total.into_value()))
        .variant(Tone::Danger),
        ui::progress(move || Some(percent.get() / 100.0))
            .name("Shield")
            .variant(ProgressVariant::Ring)
            .variant(ui::ProgressMotion {
                determinate_ms: 900,
                determinate_easing: kivact::motion::Easing::CubicBezier(0.2, 0.7, 0.2, 1.),
                ..Default::default()
            })
            .variant(Tone::Info)
            .with_node(|node| node.child(game::mono("SHIELD").style(style().font_size(9)))),
        ui::progress(move || Some(percent.get() / 100.0))
            .name("Upload")
            .moving(true)
            .variant(game::Striped),
    ))
    .style(style().gap(16))
}
# fn main() {}
```

## Format a statistic

Use `stat(label, formatted_value)` with `game::StatVariant::Kpi`. The application owns numeric formatting, decimal precision, separators, units and animation. Pass reactive text to the value, `label_content` for an icon and heading, `value_content` for separately styled numbers and units, and `chart` for `kivact::ui::sparkline(samples)`, the standard sparkline in the console's look. The sparkline only draws application-provided samples. `delta` is the change caption; `Tone` chooses its semantic color. `trend` independently reports the direction of change.

```rust,no_run
use kivact::interaction::Tone;
use kivact::motion::{Animated, Easing, Transition};
use kivact::prelude::*;
use kivact::style::Align;
use kivact_ui_game as game;

fn accuracy() -> impl IntoView {
    let target = signal(87.5);
    let shown = Animated::from(
        0.0,
        target,
        Transition::tween(900, Easing::CubicBezier(0.2, 0.7, 0.2, 1.0)),
    );
    let formatted = memo(move || format!("{:.1}", shown.get()));
    kivact::ui::stat("ACCURACY", move || format!("{}%", formatted.get()))
        .variant(game::StatVariant::Kpi)
        .variant(Tone::Success)
        .label_content(row((icon("target"), text("ACCURACY"))))
        .value_content(
            row((
                text(formatted),
                text("%").style(style().font_size(16).color(ColorToken::TextMuted)),
            ))
            .style(style().gap(2).align_items(Align::Baseline)),
        )
        .delta("+12.4%")
        .chart(kivact::ui::sparkline(vec![3.0, 5.0, 4.0, 8.0, 9.0, 12.0]))
}
# fn main() {}
```

## Show badges and avatars

A badge uses `kivact::ui::badge(label).tone(tone)`. `game::BadgeVariant` selects `Dot` (the default square marker), `Plain`, `Solid` or `Outline`. Ordinary modifiers apply to every builder.

```rust,no_run
use kivact::interaction::Tone;
use kivact::prelude::*;
use kivact_ui_game as game;

fn statuses() -> impl IntoView {
    row((
        kivact::ui::badge("RUNNING").tone(Tone::Info),
        kivact::ui::badge("v2.4.1")
            .tone(Tone::Neutral)
            .variant(game::BadgeVariant::Outline),
        kivact::ui::badge("OFFLINE")
            .tone(Tone::Danger)
            .variant(game::BadgeVariant::Solid),
    ))
}
# fn main() {}
```

Use `kivact::ui::avatar(name).fallback_content(..)` for exact initials. `.variant(game::AvatarVariant { side: 32., color })` chooses logical-pixel dimensions and fill; `.status(Some(Tone::Success))` adds the square presence marker. Group avatars with `avatar_group(children).max(3usize)`. For partially loaded groups, `.total(8usize)` states the total participant count, including supplied avatars: three visible avatars show the cut-corner `+5` tile. The theme sets six logical pixels of overlap.

## Show feedback

`alert(title)` supports a description, icon, actions and recreated detail content. `.open(binding)` makes the application the sole visibility authority: its setter can refuse a dismissal. `.on_dismiss(..)` reports a completed accepted dismissal after focus returns. `.dismiss_content(..)` changes visible control contents while `.dismiss_label(..)` retains its accessible name. The default alert is assertive; callers can explicitly choose polite status semantics for informational messages and banners.

A banner is `AlertVariant::Banner`; the theme draws its striped title tape and full-width navy surface. Empty states accept an action and optional illustration (`.icon(())` uses the Game recipe's illustration). `SkeletonVariant::Rows` shows the cut-corner avatar and two caption placeholders, animated by the standard skeleton owner and paused for reduced motion.

```rust,no_run
use kivact::interaction::Tone;
use kivact::prelude::*;
use kivact_ui_game as game;

fn warning() -> impl IntoView {
    let visible = signal(true);
    column((
        kivact::ui::alert("Low fuel")
            .tone(Tone::Warning)
            .icon(icon("alert-triangle"))
            .description("Return to base within 2 minutes.")
            .open(visible)
            .dismissible(true)
            .dismiss_label("Close alert")
            .dismiss_content(text("×")),
        kivact::ui::alert("NOTICE")
            .variant(kivact::ui::AlertVariant::Banner)
            .tone(Tone::Warning)
            .with_node(|node| {
                node.role(Role::Status)
                    .live_region(kivact::view::LiveRegion::Polite)
            })
            .content(|| text("Maintenance at 02:00 UTC"))
            .actions(button("Details")),
        kivact::ui::empty_state("NO RECORDS")
            .icon(())
            .description("Try another search.")
            .actions(button("Clear filters")),
        kivact::ui::skeleton()
            .variant(game::SkeletonVariant::Rows)
            .lines(2usize),
    ))
}
# fn main() {}
```

Short-lived messages use `kivact::ui::toast_region(toaster())` and `Toast::new(label).tone(tone)`. The application supplies all messages.

## Pair terms with details

Use `description_list` and `description` for a shaded 120-pixel term column beside arbitrary detail views. Details can be different types. Read signals inside each view to update its contents. `game::activity` keeps the Game-specific diamond timeline; an `ActivityEntry` consumes its time, content and tone once.

```rust,no_run
use kivact::interaction::Tone;
use kivact::prelude::*;
use kivact_ui_game as game;

fn details() -> impl IntoView {
    column((
        kivact::ui::description_list((
            kivact::ui::description("PILOT", game::mono("Mika")),
            kivact::ui::description("STATUS", kivact::ui::badge("ACTIVE").tone(Tone::Success)),
        )),
        game::activity(vec![game::ActivityEntry::new(
            "09:40",
            text("Review passed"),
            Tone::Success,
        )]),
    ))
}
# fn main() {}
```
