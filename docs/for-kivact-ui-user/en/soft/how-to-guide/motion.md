# Motion

[日本語](../../../jp/soft/how-to-guide/motion.md)

For custom paint feedback, use the motion tokens from the active theme. This example transitions a background color on hover. The default decorative transition respects reduced motion.

```rust,no_run
use kivact::motion::Transition;
use kivact::prelude::*;
use kivact::style::TransitionProp;
use kivact_ui_soft as soft;

fn hover_surface() -> impl IntoView {
    soft::text("Build information").style(|| {
        let p = soft::tokens();
        style()
            .padding(p.spacing.s2)
            .radius(p.radius.element)
            .background(p.colors.bg_surface)
            .hover(style().background(p.colors.bg_muted))
            .transition(
                TransitionProp::Paint,
                Transition::tween(p.motion.fast as f64, p.motion.ease_standard),
            )
    })
}
```

Paint transitions can interpolate properties such as a solid background or opacity. Keep layout changes and text decoration out of the hover paint layer.
