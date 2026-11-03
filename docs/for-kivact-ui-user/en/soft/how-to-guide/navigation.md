# Navigation

[日本語](../../../jp/soft/how-to-guide/navigation.md)

Keep the current view in application state. For a short set of related views, bind a segmented control to an optional key and use that same key to select the content.

```rust,no_run
use kivact::prelude::*;
use kivact::ui::ListItem;
use kivact::view::dynamic;
use kivact_ui_soft as soft;

fn project_view() -> impl IntoView {
    let view = signal(Some("summary"));
    column((
        soft::segmented_control(
            "Project view",
            vec![ListItem::new("summary", "Summary"), ListItem::new("activity", "Activity")],
            view,
        ),
        dynamic(view, |view| match view {
            Some("activity") => soft::heading(2, "Activity"),
            _ => soft::heading(2, "Summary"),
        }),
    ))
    .style(|| style().gap(soft::tokens().spacing.s4))
}
```

Use a stable key such as `"summary"` so selection survives a change in the visible label. Standard tabs are a better fit when the selection switches among named panels within a larger screen.
