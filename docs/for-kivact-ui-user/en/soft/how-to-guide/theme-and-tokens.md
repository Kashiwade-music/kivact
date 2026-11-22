# Theme and variables

[日本語](../../../jp/soft/how-to-guide/theme-and-tokens.md)

Use the kit's colors, spacing and radii as constants inside a style so a custom surface follows the nearest theme. Use semantic colors for the surface, border, and text. Use the spacing and radius constants to align the custom view with the surrounding controls.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

#[component]
fn project_summary() -> impl Element {
    column((
        kivact::ui::heading("Project summary", 2),
        soft::styled_text("Three milestones remain.").color(soft::TextColor::Secondary),
    ))
    .style(
        style()
            .padding(soft::SPACE_S4)
            .gap(soft::SPACE_S2)
            .background(soft::BG_SURFACE)
            .border(1.0, kivact::style::color::BORDER)
            .border_radius(soft::RADIUS_CONTAINER),
    )
}
```

Each color, space and radius of the kit is a theme variable, and a `const` names it: `soft::BG_SURFACE`, `soft::TEXT_PRIMARY`, `soft::SPACE_S4`, `soft::RADIUS_CONTAINER`. A style that uses one reads it from the theme that is active when the view is drawn, so it follows theme switches the same way the kit's own components do. The kit's values come with `soft::theme(mode)`; your application chooses the mode, not the individual values.

The Soft catalog registers Figtree. An application that uses Figtree supplies and registers its own font bytes. Without registration, the theme's font stack uses its system fallback.
