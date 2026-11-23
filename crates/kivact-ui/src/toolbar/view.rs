//! Role: The toolbar component: a row of controls with keyboard movement.

use super::look::ToolbarEdge;
use crate::control_size::{ControlSize, default_control_size};
use crate::layer::with_base_recipe_style;
use crate::provide_control_size;
use crate::toolbar::look::{ToolbarState, ToolbarTheme};
use kivact_core::custom::Caller;
use kivact_core::state::{IntoValue, Value};
use kivact_core::style::{Align, FlexDirection, Justify, style};
use kivact_core::view::{
    Children, Orientation, ReadingDirection, Role, View, node, row, use_direction,
};
use kivact_macros::component;

/// Group controls under an accessible name. Arrow keys along the configured
/// axis move focus without activating controls; Home and End reach its ends.
/// Disabled controls are skipped unless explicitly focusable while disabled.
/// Use at least three controls for the APG toolbar pattern.
///
/// `start`, `center` and `end` are its three optional regions;
/// `dividers` adds lines on logical edges.
#[component]
pub fn toolbar(
    label: Value<String>,
    start: Option<Children>,
    center: Option<Children>,
    end: Option<Children>,
    #[prop(default = Orientation::Horizontal)] orientation: Orientation,
    size: Option<Value<ControlSize>>,
    gap: Option<Value<f64>>,
    #[prop(default = Vec::new())] dividers: Vec<ToolbarEdge>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let vertical = orientation == Orientation::Vertical;
    let rtl = use_direction() == ReadingDirection::Rtl;
    let size = size.unwrap_or_else(|| default_control_size().into_value());
    let centered = center.is_some();
    let make = move |children: Option<Vec<View>>, justify, column| {
        row(children).base_style(move || {
            let tokens = kivact_core::style::use_theme().get();
            style()
                .min_width(0)
                .align_items(Align::Center)
                .justify_content(justify)
                .flex_direction(if vertical {
                    FlexDirection::Column
                } else if rtl {
                    FlexDirection::RowReverse
                } else {
                    FlexDirection::Row
                })
                .gap(gap.map_or(tokens.get(&crate::toolbar::look::GAP_VAR), |gap| gap.get()))
                .grid_column(column)
        })
    };
    let regions = provide_control_size(size.get(), move || {
        (
            make(
                start.map(Children::into_views),
                Justify::Start,
                if rtl { 3 } else { 1 },
            ),
            center.map(|children| make(Some(children.into_views()), Justify::Center, 2)),
            make(
                end.map(Children::into_views),
                Justify::End,
                if rtl { 1 } else { 3 },
            ),
        )
    });
    with_base_recipe_style(
        node()
            .role(Role::Toolbar)
            .name(label)
            .orientation(orientation)
            .roving_focus(orientation),
        |r: &ToolbarTheme| r.root_recipe(),
        &variants,
        move || ToolbarState {
            orientation,
            size: size.get(),
            gap: gap.map(|gap| gap.get()),
            centered,
            rtl,
            dividers: dividers.clone(),
        },
    )
    .child(regions)
}
