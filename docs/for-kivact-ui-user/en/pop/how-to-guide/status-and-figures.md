# Show status and figures

[日本語](../../../jp/pop/how-to-guide/status-and-figures.md)

Use standard `kivact::ui` components with the Pop theme. The application supplies formatted text and read-only chart data; the kit owns their appearance.

## Statistics, headings and details

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;
fn summary() -> impl IntoView {
    let count = signal(1284usize);
    column((
        row((kivact::ui::badge("01").variant(pop::SectionNumber),
             kivact::ui::heading("Customers", 2))).style(style().gap(16)),
        kivact::ui::stat("Active customers", move || count.get().to_string())
            .delta("↗ 8.2%")
            .trend(kivact::ui::Trend::Up)
            .variant(pop::StatHue::new(pop::Hue::Mint))
            .icon(icon("users"))
            .chart(kivact::ui::sparkline(vec![2.0, 4.0, 3.0, 7.0, 9.0])),
        kivact::ui::description_list((
            kivact::ui::description("Name", text("Haruka Sato")),
            kivact::ui::description("Status", kivact::ui::badge("Active").variant(pop::Hue::Mint)),
            kivact::ui::description("Used", kivact::ui::progress(Some(0.72))
                .name("Used capacity").variant(kivact::ui::ProgressVariant::Ring)
                .variant(pop::Hue::Blue)),
        )),
    )).style(style().gap(16).width(420))
}
```

`stat(label, value)` accepts reactive text. `.delta(..)` supplies the formatted change, `.trend(Trend::Up | Down | Neutral)` supplies its meaning, and `pop::StatHue` optionally selects any reactive Pop hue. `.icon(..)`, `.chart(..)`, `.label_content(..)` and `.value_content(..)` are mounted view slots. `kivact::ui::sparkline(points)` draws the standard chart in the Pop look and can be placed in any chart slot.

`heading(text, level)` exposes the semantic level. Compose a section number with a standard badge using `pop::SectionNumber`, and put optional supporting text beside or below the heading. `card(children)` supplies the bordered surface. No extra numeric state belongs to a statistic.

`description_list(..)` groups `description(term, detail)` rows. Details accept any view, including a badge, button or progress ring; `.term_content(..)` changes the visible term while retaining its accessible name.

`progress(Some(fraction))` takes a value from 0 to 1; `None` is indeterminate. `ProgressVariant::Ring` shows a circle and visible percentage, `Bar` the normal bar and `Gauge` a segmented bar. `.name(..)` supplies its accessible name. A static `pop::Hue` or reactive `pop::ProgressHue` selects its color; `pop::Striped` and `pop::Segments(n)` retain the bar options. Draw a visible label beside the component when needed.

## Record rows and avatars

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;
fn customer() -> impl IntoView {
    let look=pop::AvatarLook::default().size(48).hue(330.0).halo(true).online(true);
    kivact::ui::item_list((
        kivact::ui::item_row("Haruka Sato").description("Team plan")
            .media(kivact::ui::avatar("HS").fallback_content(||text("HS"))
                .variant(look).status_content(look.decorations()))
            .actions(kivact::ui::button("Open")),
    ))
}
```

`item_row` exposes a list item, with `.media(..)`, `.description(..)` and `.actions(..)` slots. An icon tile can use `.media(icon("user")).variant(pop::ItemHue::new(pop::Hue::Blue))`.

`AvatarLook` preserves a pixel diameter, continuous hue angle, halo and online dot. Its options accept reactive values. Supply initials through `.fallback_content(|| text("HS"))` and mount `look.decorations()` in `.status_content(..)`. Standard avatar image, shape, size and semantic status options remain available.

## Navigation and steps

Use keyed `kivact::ui::breadcrumb` items and `.separator(|| icon("chevron-right"))` for the Pop trail. Step notes belong to `StepperItem::description`. `pop::DiamondSteps` selects the diamond appearance and `.min_step_width(0)` keeps all steps visible; the standard threshold enables the responsive collapsed layout.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;
fn progress() -> impl IntoView {
    let current=signal(1usize);
    let notes=signal(vec!["10/01".to_owned(),"In progress".to_owned()]);
    kivact::ui::stepper(move || ["Received","Working","Reported"].into_iter().enumerate()
        .map(|(index,label)|kivact::ui::StepperItem::new(label)
            .description(notes.with(|notes|notes.get(index).cloned().unwrap_or_default())))
        .collect::<Vec<_>>(),current)
        .variant(pop::DiamondSteps).min_step_width(0)
}
```
