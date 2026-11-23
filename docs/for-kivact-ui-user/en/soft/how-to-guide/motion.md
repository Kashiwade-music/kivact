# Motion

[日本語](../../../jp/soft/how-to-guide/motion.md)

For custom paint feedback, use the kit's values: the fast duration of 125 ms and the standard easing of the kit. This example transitions a background color on hover. The default decorative transition respects reduced motion.

```rust,no_run
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact_ui_soft as soft;

#[component]
fn hover_surface() -> impl Element {
    soft::styled_text("Build information").style(
        style()
            .padding(soft::SPACE_S2)
            .border_radius(soft::RADIUS_ELEMENT)
            .background(soft::BG_SURFACE)
            .hover(style().background(soft::BG_MUTED))
            .transition(
                TransitionProp::All,
                Transition::tween(125.0, Easing::CubicBezier(0.24, 1.0, 0.4, 1.0)),
            ),
    )
}
```

Paint transitions can interpolate properties such as a solid background or opacity. Keep layout changes and text decoration out of the hover paint layer.

Use `kivact::ui::spinner()` with `ControlSize::ExtraSmall`, `Small`, `Medium`, or `Large` for the former Sm, Md, Lg, and Xl glyph sizes. `.variant(soft::SpinnerVariant { shade: soft::SpinnerShade::Subtle, ..Default::default() })` selects ink treatment. `OnMedia` and `Inherit` remain available; supply `ink: Some(color)` for explicit inherited ink. Use `.name(label).label_content(text(label))` for a named visible label, or pass arbitrary content to `label_content`. Soft sets the normal cycle and a slower 3000 ms reduced-motion cycle.

Use standard `skeleton().variant(soft::SkeletonRadius::Rounded)` for the previous rounded placeholder, and retain `.index(n)` for stagger. All radius choices remain kit variants. `.shape(...)` and `.lines(...)` provide standard circle and text groups. Soft supplies the original 400 ms, ten-step alternating pulse after 1000 ms plus 100 ms per index; reduced motion keeps the static low opacity.
