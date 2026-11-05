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
        let p = kivact::style::use_tokens::<soft::Tokens>();
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

Use `kivact::ui::spinner()` with `ControlSize::ExtraSmall`, `Small`, `Medium`, or `Large` for the former Sm, Md, Lg, and Xl glyph sizes. `.variant(soft::SpinnerVariant { shade: soft::SpinnerShade::Subtle, ..Default::default() })` selects ink treatment. `OnMedia` and `Inherit` remain available; supply `ink: Some(color)` for explicit inherited ink. Use `.name(label).label_content(text(label))` for a named visible label, or pass arbitrary content to `label_content`. Soft sets the normal cycle and a slower 3000 ms reduced-motion cycle.
