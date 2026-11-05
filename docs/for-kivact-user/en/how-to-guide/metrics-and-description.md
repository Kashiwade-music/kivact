# Metrics and descriptions

Use standard builders from `kivact::ui` with any installed theme. Values and labels accept reactive text; format numbers and units in the application.

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{self, ProgressVariant, Trend};

fn overview() -> impl IntoView {
    let completed = signal(Some(0.7));
    let revenue = signal("$12,400".to_owned());
    column((
        ui::heading("Overview", 2),
        ui::progress(completed)
            .variant(ProgressVariant::Ring)
            .name("Upload"),
        ui::stat("Revenue", revenue).delta("+8% this month").trend(Trend::Up),
        ui::description_list((
            ui::description("Owner", text("Ada")),
            ui::description("Profile", button("Open profile")),
        )).name("Project details"),
    )).style(style().gap(16).width(320))
}
```

Progress takes a fraction from 0 to 1 and clamps values outside that range. `None` means indeterminate. `ProgressVariant::{Bar, Ring, Gauge}` use the same `.variant(impl Variant)` contract as custom looks: a later value of the same type replaces the earlier one. All appearances keep the progress role, range and accessible percentage. The default ring also displays the percentage; the gauge is segmented. `.size(..)` and `.moving(..)` retain their usual meaning. Motion follows the installed theme and reduced-motion preference.

`stat(label, value)` exposes a group named by the label, with the application-formatted value as its accessible value. `.delta(impl IntoText)` adds a change caption, and `.trend(impl IntoValue<Trend>)` supplies `Up`, `Down` or `Neutral` to every recipe. Direction alone does not mean success or danger: choose any semantic tone with `.variant(..)` and let the theme style it. `.icon(impl IntoView)` takes a decorative view. `.label_content(..)` and `.value_content(..)` replace visible content while retaining the accessible label and value. `.chart(..)` accepts an application-owned chart, including a kit's sparkline. Numeric animation, custom formatting and units can be supplied as reactive text; these builders do not parse or animate text themselves.

`heading(text, level)` reports `Role::Heading` and clamps levels to 1 through 6. Its typography comes from the theme's heading recipe. Heading text remains reactive.

`description_list(children)` exposes a list. Each `description(term, detail)` is a list item named by its term; its detail is a group labelled by the visible term. This published association updates when the term changes. Details can contain arbitrary views and controls. `.term_content(impl IntoView)` replaces the visible term (for example, with an icon and text) while keeping its canonical accessible name. Use regular layout styles or description-list recipes for column widths, grids and spacing. Application-owned disclosure and child composition can limit or expand long lists. Editable inspector rows remain in the Creative kit.

For constrained media, apply the ordinary style property rather than an aspect-ratio component:

```rust,no_run
use kivact::prelude::*;

fn preview() -> impl IntoView {
    node().style(style().width(320).aspect_ratio(16.0 / 9.0))
}
```

Themes customize `UiRecipes::heading`, `UiRecipes::stat` and `UiRecipes::description_list`. Statistic parts are `root`, `label`, `value`, `delta`, `icon` and `chart`; description parts are `root`, `item`, `term` and `detail`. Progress keeps its `track` and `fill` recipes and adds `label` for ring text. Every part receives the component's variant values.
