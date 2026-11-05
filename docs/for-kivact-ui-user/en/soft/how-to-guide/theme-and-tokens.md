# Theme and tokens

[日本語](../../../jp/soft/how-to-guide/theme-and-tokens.md)

Read `kivact::style::use_tokens::<soft::Tokens>()` inside a style closure so a custom surface follows the nearest theme. Use semantic colors for the surface, border, and text. Use the spacing and radius tokens to align the custom view with the surrounding controls.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn project_summary() -> impl IntoView {
    column((
        kivact::ui::heading("Project summary", 2),
        soft::styled_text("Three milestones remain.").color(soft::TextColor::Secondary),
    ))
    .style(|| {
        let p = kivact::style::use_tokens::<soft::Tokens>();
        style()
            .padding(p.spacing.s4)
            .gap(p.spacing.s2)
            .background(p.colors.bg_surface)
            .border(p.sizes.border_width, p.colors.border)
            .radius(p.radius.container)
    })
}
```

The Soft catalog registers Figtree. An application that uses Figtree supplies and registers its own font bytes. Without registration, the theme's font stack uses its system fallback.
