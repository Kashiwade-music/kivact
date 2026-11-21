//! Role: Theme policy and paint for sticky data table cells.
use kivact_core::style::{Color, Style, Theme, TransitionProp, style, use_theme};

pub(super) fn surface(header: bool, selected: bool, tint: Option<Color>) -> Style {
    let theme = use_theme().get();
    let mut color = theme.get(&crate::data_table::look::STICKY_SURFACE_VAR);
    if let Some(tint) = tint {
        color = over(&theme, tint, color);
    }
    if selected {
        color = over(
            &theme,
            theme.get(&crate::data_table::look::STICKY_SELECTED_VAR),
            color,
        );
    }
    let mut base = style().background(color);
    if !header {
        base = base.group_hover(style().background(over(
            &theme,
            theme.get(&crate::data_table::look::STICKY_HOVER_VAR),
            color,
        )));
        if let Some(transition) = theme
            .ext::<crate::data_table::look::DataTableTheme>()
            .sticky_overlay_transition_value()
            .clone()
        {
            base = base.transition(TransitionProp::All, transition);
        }
    }
    base
}

fn over(theme: &Theme, overlay: Color, surface: Color) -> Color {
    let [tr, tg, tb, ta] = overlay.components(theme);
    let [br, bg, bb, _] = surface.components(theme);
    let mix = |a: f64, b: f64| a * ta + b * (1.0 - ta);
    Color::from_rgba_f64(mix(tr, br), mix(tg, bg), mix(tb, bb), 1.0)
}
