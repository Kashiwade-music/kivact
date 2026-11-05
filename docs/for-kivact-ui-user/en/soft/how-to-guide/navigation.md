# Navigation

[日本語](../../../jp/soft/how-to-guide/navigation.md)

Keep the current view in application state. For a short set of related views, bind a segmented control to an optional key and use that same key to select the content.

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::ListItem;
use kivact::view::dynamic;
use kivact_ui_soft as soft;

fn project_view() -> impl IntoView {
    let view = signal(Some("summary"));
    column((
        radio_group(
            "Project view",
            vec![ListItem::new("summary", "Summary"), ListItem::new("activity", "Activity")],
            view,
        ).variant(kivact::ui::RadioGroupVariant::Segmented),
        dynamic(view, |view| match view {
            Some("activity") => soft::heading(2, "Activity"),
            _ => soft::heading(2, "Summary"),
        }),
    ))
    .style(|| style().gap(kivact::style::use_tokens::<soft::Tokens>().spacing.s4))
}
```

Use a stable key such as `"summary"` so selection survives a change in the visible label. Standard tabs are a better fit when the selection switches among named panels within a larger screen.

Resizable panels use `kivact::ui::splitter` with an application size binding and a `ScalarRange`.
Use `.collapsible(fraction)` and `.snaps(fractions)` for collapse and detents, `.orientation(..)`
for stacked panes, and `.pointer_scale(..)` for reversed or proportional sizing. A reactive range
can read a container's `NodeRef::rect()` to resolve percentages and pixel bounds. Custom grip
content belongs in `.handle(..)`; its style controls the divider, visibility, position, and overlay.
Use `LayoutRegion::width(..)` when a layout panel follows the same size binding. A supplied end
resize slot controls its own reversed direction. Observe scroll boundaries with `NodeRef::scroll_state()`;
scrollable areas use the core keyboard scroll behavior.

Use standard `disclosure(title, open).content(factory)` and `.animated(true)` for expandable sections. The Soft theme supplies the original spring transition. `disclosure_group(signal(None::<K>))` gives single selection; a `Vec<K>` binding gives multiple selection. `.panel(key, title, factory)` replaces keyed kit sections. Set `.variant(soft::DisclosureVariant { dividers: true, ..Default::default() })` for separators; `chevron` accepts `ChevronPosition::Start` or `End`, and `density` accepts `Some(DisclosureDensity::Compact)`, `Balanced`, or `Spacious`. These visual variants apply to both standalone sections and groups and follow text direction. `.always_open()` replaces disabled collapsing; `.content(factory).attach(trigger)` supplies a custom trigger. Create `signal(default_open)` at the call site for an initial opening.

Use standard `toolbar(label).start(...).center(...).end(...)`; `.variant(soft::ToolbarVariant(soft::SectionVariant::Muted))` selects surface chrome and `.dividers(&[ToolbarEdge::Bottom])` draws logical-edge separators. Wrap the content factory in `soft::with_key_hints(axis, || view)` and attach `soft::keyboard_hint(button)` to actual keyboard targets for one first-focus hint. The hint axis accepts a reactive orientation; arrows, focus departure, or three seconds end the visit. Segmented controls use `radio_group(...).variant(RadioGroupVariant::Segmented)` with optional `soft::SegmentedLayout::{Hug,Fill}`. For icon-only segments, combine `SegmentedIconOnly` with `.item(...)` returning an icon and no visible label; the standard radio retains its accessible name. Attach `tooltip(reason).enabled(disabled)` to explain a disabled control. Use the standard `.disabled(...)` modifier.
