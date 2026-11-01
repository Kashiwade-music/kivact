# Lay out a screen

[日本語](../../jp/how-to-guide/layout.md)

Use `row` and `column` to place children, `style()` to size and space them, and
`scroll` for content that can be longer than the window. This guide covers
sizes, flexible space, alignment, wrapping, scrolling, layers and grids, and
ends with a two-pane screen.

Every example uses `use kivact::prelude::*;`. Lengths and layout helpers that
are not in the prelude are imported where they appear.

## The shortest working layout

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    column((
        text("Settings"),
        row((button("Cancel"), button("Save"))).style(style().gap(8)),
    ))
    .style(style().padding(16).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("Layout").run(app)
}
```

`column` stacks its children from top to bottom and `row` places them from left
to right. Both take one view, a tuple of views (up to 16 children, each of its
own type) or a `Vec` of views of one type. `gap` is the space between children
and `padding` is the space inside the box. Numbers are logical pixels, so the
same layout looks the same on a high-density display.

## Pick the container

| You want | Use |
| --- | --- |
| Children in a vertical line | `column((a, b))` |
| Children in a horizontal line | `row((a, b))` |
| Rows and columns of cells | `grid((a, b, c))` with `grid_template_columns` |
| Children on top of each other | `stack((a, b))` |
| Content that scrolls | `scroll(child)` |
| Flexible empty space | `spacer()` |
| A plain box you style yourself | `node().child(a)` |

`node()` is the box under all of them. It is a vertical flex box too, so
`column(x)` is `node().child(x)` plus a column direction.

## Sizes and units

A number is a length in logical pixels. For everything else, use the length
helpers.

```rust,no_run
use kivact::prelude::*;
use kivact::style::{Length, auto};

fn app() -> impl IntoView {
    column((
        // A fixed size.
        node().style(style().width(120).height(32)),
        // Half of the parent's width.
        node().style(style().width(pct(50)).height(32)),
        // The parent's width minus 40 px (`calc(100% - 40px)`).
        node().style(style().width(pct(100) - 40).height(32)),
        // Back to "as big as the content needs".
        node().style(style().width(auto()).height(32)),
        // The content's width, but never wider than the available space.
        node().style(style().width(Length::FitContent).height(32)),
        // Bounds: at least 100 px, at most 400 px.
        node().style(style().min_width(100).max_width(400).height(32)),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Sizes").run(app)
}
```

- `pct(50)` is a percentage of the parent. `pct(100) - 40` and `pct(50) + 8`
  are calc lengths that mix a percentage and pixels.
- `width`, `height`, `min_*`, `max_*`, `padding`, `margin`, `gap` and `basis`
  all take a length. `padding_x`, `padding_y` and the one-side variants such as
  `padding_top` exist for `padding` and `margin`.
- `Length::MinContent`, `MaxContent` and `FitContent` size a box by its
  content. They only work for `width`, `height`, `basis` and grid tracks.
- `aspect_ratio(16.0 / 9.0)` derives one side from the other.
- Every `style()` call returns a new value, so chain the setters in one
  expression. Setting the same property twice keeps the last value (a debug
  build reports the duplicate).

## Fixed and flexible sizes

By default a child takes the space its content needs and shrinks if the parent
is too small. Three properties change that:

- `grow(n)` hands out the free space along the main axis. A child with
  `grow(1)` takes all of it; two children with `grow(1)` and `grow(3)` share it
  one to three.
- `shrink(0)` stops a child from getting smaller than its size.
- `spacer()` is an empty box with `grow(1)`: it pushes what follows it to the
  end.

```rust,no_run
use kivact::prelude::*;

fn toolbar() -> impl IntoView {
    row((
        text("Untitled"),
        spacer(),
        button("Undo"),
        button("Redo"),
    ))
    .style(style().gap(8).padding(8))
}

fn main() -> kivact::Result<()> {
    App::new("Toolbar").run(toolbar)
}
```

`text("Untitled")` stays at the left edge, the two buttons end up at the right
edge, and the spacer takes everything in between.

## Alignment

Two properties place children inside a box:

- `justify_content(..)` distributes children along the **main axis**: top to
  bottom in a column, left to right in a row.
- `align_items(..)` aligns them on the **cross axis**: left to right in a
  column, top to bottom in a row.

`align_items` defaults to `Align::Stretch`, so a column's children are as wide
as the column and a row's children are as tall as the row. This is why a
button in a column spans the full width until you ask otherwise.

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    column((
        // Centered on both axes in a 240 px high box.
        column((text("Hello"),))
            .style(style().height(240).align_items(Align::Center).justify_content(Justify::Center)),
        // The first button keeps its natural width; the second still stretches.
        column((
            button("Natural width").style(style().align_self(Align::Start)),
            button("Full width"),
        ))
        .style(style().gap(8)),
        // Spread out.
        row((text("A"), text("B"), text("C"))).style(style().justify_content(Justify::SpaceBetween)),
    ))
    .style(style().gap(16).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Alignment").run(app)
}
```

`Align` has `Start`, `Center`, `End`, `Stretch` and `Baseline` (a row aligns its
texts on their first baseline). `Justify` has `Start`, `Center`, `End`,
`SpaceBetween`, `SpaceAround`, `SpaceEvenly` and `Stretch`. `align_self` on a
child overrides the parent's `align_items` for that child.

## Wrap

A row that is too narrow normally overflows. Turn on `wrap(true)` and children
move to the next line instead. `gap` applies in both directions (`row_gap` and
`column_gap` set them apart).

```rust,no_run
use kivact::prelude::*;

fn tags() -> impl IntoView {
    row(vec![
        button("Rust"), button("GUI"), button("Signals"),
        button("Native"), button("Windows"), button("Linux"),
    ])
    .style(style().wrap(true).gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("Wrap").min_window_size(200, 200).run(tags)
}
```

Resize the window narrower and the buttons flow onto more lines.

Text wraps by itself inside the width it is given. For a label that must stay on
one line and end in "…" when it is too narrow, use `style().single_line()`. In a
row, the row's own child also needs `min_width(0)` for the text to shrink; a flex
child does not get smaller than its content otherwise.

## Scroll

`scroll(child)` makes the child scroll vertically when it is taller than the
scroll area. The scroll area needs a **bounded size**: give it a height, or let
a parent with a definite height grow it.

```rust,no_run
use kivact::prelude::*;

fn log() -> impl IntoView {
    let lines: Vec<_> = (1..=100).map(|n| text(format!("line {n}"))).collect();
    scroll(column(lines)).style(style().height(200))
}

fn strip() -> impl IntoView {
    scroll(row(vec![button("one"), button("two"), button("three")]))
        .orientation(Orientation::Horizontal)
        .style(style().width(160))
}

fn table() -> impl IntoView {
    scroll(column(vec![text("a very wide and tall table")]))
        .both_axes()
        .style(style().size(300, 200))
}

fn main() -> kivact::Result<()> {
    App::new("Scroll").run(|| column((log(), strip(), table())).style(style().gap(12)))
}
```

- The default scrolls vertically. `.orientation(Orientation::Horizontal)` lays
  the content out in a row and scrolls sideways. `.both_axes()` scrolls both
  ways.
- `scroll` returns a `Scroll` builder: `.style(..)`, `.name(..)` and the other
  node modifiers apply to the scroll area itself.
- The wheel and touch drag scroll the area; at the edge, the wheel passes to the
  parent.
- For hundreds of rows use `virtual_list`, which mounts only the visible rows
  (see [Lists and tables](lists-and-tables.md)).
- In a flex column, a scroll area that should fill the rest of the space needs
  `grow(1)` and `min_height(0)`; without the minimum, the content's height
  keeps the area from shrinking.

## Layers

`stack` lays its children on top of each other. The first child decides the
size; every later child fills the same box.

```rust,no_run
use kivact::prelude::*;

fn badge() -> impl IntoView {
    stack((
        node().style(style().size(64, 64).background(ColorToken::SurfaceAlt)),
        text("3").style(style().color(ColorToken::Danger).padding(4)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Stack").run(badge)
}
```

For a menu or a popup that must float above the screen use the overlay
components instead (see [Overlays](overlays.md)); a stack is for layers inside
one box.

## Grids

`grid` places children in the cells of the tracks you define:

```rust,no_run
use kivact::prelude::*;
use kivact::style::Track;

fn form_grid() -> impl IntoView {
    grid((
        text("Name"), text_input("Name", signal(String::new())),
        text("Email"), text_input("Email", signal(String::new())),
    ))
    .style(
        style()
            .grid_template_columns((96, Track::fr(1)))
            .gap(8)
            .padding(16),
    )
}

fn main() -> kivact::Result<()> {
    App::new("Grid").run(form_grid)
}
```

`(96, Track::fr(1))` is a 96 px column followed by a column that takes the rest.
Children fill the cells in order. `Track::repeat_auto_fill(Track::minmax(160,
Track::fr(1)))` makes as many 160 px-or-wider columns as fit. For a child that
should span cells, use `style().grid_column(GridPlacement::span(2))` with
`kivact::style::GridPlacement`.

## A two-pane screen

A fixed-width sidebar and a main area that takes the remaining space and
scrolls:

```rust,test_harness
use kivact::prelude::*;

fn sidebar() -> impl IntoView {
    column((text("Inbox"), text("Sent"), text("Archive")))
        .style(style().width(200).padding(12).gap(8).background(ColorToken::SurfaceAlt))
        .test_id("sidebar")
}

fn content() -> impl IntoView {
    let items: Vec<_> = (1..=60).map(|n| text(format!("Message {n}"))).collect();
    scroll(column(items).style(style().padding(12).gap(8)))
        .style(style().grow(1).min_width(0))
        .test_id("content")
}

fn app() -> impl IntoView {
    row((sidebar(), content())).style(style().height(pct(100)))
}

fn main() -> kivact::Result<()> {
    App::new("Mail").window_size(900, 600).run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sidebar_is_fixed_and_the_content_takes_the_rest() {
        let screen = kivact::test::render(app);
        let sidebar = screen.get_by_test_id("sidebar").rect().expect("laid out");
        let content = screen.get_by_test_id("content").rect().expect("laid out");
        assert_eq!(sidebar.width, 200.0);
        assert_eq!(content.x, 200.0);
        // The default test window is 800 px wide.
        assert_eq!(content.width, 600.0);
        assert_eq!(content.height, sidebar.height);
    }
}
```

The sidebar's `width(200)` is fixed, and the sidebar stretches to the row's
height because of the default `Align::Stretch`. The content has `grow(1)`, so
it takes the width that is left, and `min_width(0)` lets it shrink in a narrow
window. The whole row is `pct(100)` high so the scroll area has a bounded
height to scroll in.

If you want to check a layout without opening it by hand, `Screen::assert_layout`
in a test reports text that overflows its box, wraps in a button, sits off center
or is clipped without an ellipsis; see [Test your app](testing.md).

## See also

- [Lists and tables](lists-and-tables.md): long or selectable collections.
- [Overlays](overlays.md): dialogs, menus and tooltips that float above the
  layout.
- [Theming](theming.md): colors, spacing and hover or focus looks.
- Reference: `cargo doc -p kivact-style --open` lists every `Style` setter.
