# Show figures, status and records on a business screen

[日本語](../../../jp/pop/how-to-guide/status-and-figures.md)

Use this guide to build the typical overview screen of a business app: key
figures with a trend, a list of records with status badges, and a details
card. These are display components. They show the values you give them; they
never fetch, count or save anything.

## Put figures in cards

`pop::stat_card(title, value)` is a card with a number. Everything else is an
optional builder method.

```rust,no_run
use kivact::prelude::*;
use kivact::style::Icon;
use kivact_ui_pop as pop;

fn figures() -> impl IntoView {
    let open = signal(128_u32);
    row((
        pop::stat_card("Open orders", move || open.get().to_string())
            .icon(Icon::named("clipboard-list"))
            .trend("+12% this week")
            .tone(pop::Hue::Mint)
            .points(vec![3.0, 5.0, 4.0, 8.0, 7.0, 10.0])
            .style(style().grow(1)),
        pop::stat_card("Refunds", "7")
            .icon(Icon::named("coin"))
            .trend("+2 this week")
            .tone(pop::Hue::Red)
            .style(style().grow(1)),
    ))
    .style(style().gap(16).padding(24))
}
# fn main() {}
```

- The value and the trend are text. Format numbers yourself, because you know
  the locale and the unit. Pass a closure, as above, to make them follow a
  signal.
- `.tone(hue)` colors the trend text: `Mint` for good, `Red` for bad, `Gray`
  (the default) for neutral. The kit does not decide what a good trend is.
- `.points(..)` draws a sparkline of the samples under the number. The
  samples are `f64` and the card never changes them.
- `.icon(..)` takes an icon name from the kit's icon set. Names such as
  `home`, `users`, `calendar`, `chart-bar`, `coin` and `clipboard-list` are
  included.
- Cards, like every Pop builder, accept `.style(..)`, `.test_id(..)` and the
  other modifiers every Kivact node has.

## Group content in a panel with a heading

`kivact::ui::card(children)` is a bordered card that lays its children out in a
column. `pop::section_heading(number, title, subtitle)` is a numbered heading
for a screen with several sections. Put a `panel` around each section.

## List records with a status

`pop::list_item(title)` is a row for a record: an optional round icon, a title
with a subtitle, and a trailing view. `kivact::ui::badge(label)` is the status chip.

```rust,no_run
use kivact::prelude::*;
use kivact::style::Icon;
use kivact_ui_pop as pop;

#[derive(Clone, PartialEq)]
struct Customer {
    name: &'static str,
    plan: &'static str,
    active: bool,
}

fn customers(rows: Signal<Vec<Customer>>) -> impl IntoView {
    kivact::ui::card((
        pop::section_heading("01", "Customers", "Last 7 days"),
        for_each(
            rows,
            |customer: &Customer| customer.name,
            |customer| {
                let customer = customer.get_untracked();
                pop::list_item(customer.name)
                    .subtitle(customer.plan)
                    .icon(Icon::named("user"))
                    .tone(pop::Hue::Blue)
                    .trailing({
                        if customer.active {
                            kivact::ui::badge("Active").variant(pop::BadgeHue::new(pop::Hue::Mint))
                        } else {
                            kivact::ui::badge("Paused").variant(pop::BadgeHue::new(pop::Hue::Gray))
                        }
                    })
            },
        ),
    ))
    .style(style().width(420).margin(24).gap(8))
}
# fn main() {}
```

`for_each(items, key, row)` builds one `list_item` per customer and keeps each
row (and its state) alive while the list changes. The row closure receives one `Binding<Customer>` because the source is a
signal. Read it in reactive text or style closures when customer data changes.
The trailing slot takes a view mounted once.

`badge` has five looks through `.variant(pop::BadgeVariant::..)`: `Soft` (the
default), `Dot` (a colored dot before the label), `Solid`, `Slant` and `Count`
(a small number). Its color is `.hue(pop::Hue::..)`, one of `Blue`, `Mint`,
`Yellow`, `Pink`, `Red` and `Gray`.

## Show details as label and value rows

`pop::key_value(key, value)` is one row of a details card. The value is any
view, so it can be text, a badge or a button.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;

fn details() -> impl IntoView {
    kivact::ui::card((
        pop::key_value("Name", text("Haruka Sato")),
        pop::key_value("Plan", text("Team")),
        pop::key_value("Status", kivact::ui::badge("Active").variant(pop::BadgeHue::new(pop::Hue::Mint))),
        pop::key_value(
            "Storage",
            pop::progress_ring(72.0, "Used", pop::Hue::Blue.into_value()),
        ),
    ))
    .style(style().width(360).margin(24))
}
# fn main() {}
```

`pop::progress_ring(percent, label, hue)` draws a circle for a value from 0 to
100 and prints the rounded percentage inside. The value can be a signal, and
the ring animates when it changes. For a plain bar use the standard
`kivact::ui::progress(Some(0.4))`: its value runs from 0 to 1, `None` makes it
indeterminate, and the Pop theme draws it. Name it for assistive technology
with `.name("Upload progress")`.

## Show who

Use `kivact::ui::avatar(name)` with `pop::AvatarLook` for the Pop pixel diameter, continuous hue, halo and online decoration. Pass literal initials through `.fallback_content(|| text("AS"))` and mount `look.decorations()` through `.status_content(..)`. The same look is supplied with `.variant(look)`.

## Hue and Tone: which one

Two small enums look alike. `pop::Hue` is a color of the Pop palette, used by
badges, stat cards, list items and calendars. `kivact::interaction::Tone` (`Neutral`,
`Info`, `Success`, `Warning`, `Danger`, `Accent`) is a meaning, used by alerts,
toasts and anything that comes from the core. When a Pop component asks for
a `Hue`, decide the color; when a standard or message component asks for a
`Tone`, say what it means and the theme picks the color.

## See also

- [Show rows in a table](tables.md) for many records with sorting and paging
- [Show messages and notifications](messages-and-notifications.md) for alerts,
  empty states and toasts
- [Set up the Pop theme and tokens](theme-and-tokens.md) to change the hues
- All display components: `cargo doc -p kivact-ui-pop --open`

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;
fn person() -> impl IntoView {
    let look = pop::AvatarLook::default().size(48).hue(330.0).halo(true).online(true);
    kivact::ui::avatar("AS")
        .fallback_content(|| text("AS"))
        .variant(look)
        .status_content(look.decorations())
}
```
