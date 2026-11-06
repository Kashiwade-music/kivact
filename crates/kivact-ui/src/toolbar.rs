//! Role: Compose named toolbar regions with native roving focus.
use crate::layer::with_base_recipe_style;
use crate::variants::{Variant, Variants};
use crate::{ControlSize, provide_control_size};
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::style::{Align, Direction, Justify, style};
use kivact_core::view::{
    IntoChildren, IntoView, Node, NodeBuilder, Orientation, ReadingDirection, Role, View, node,
    row, use_direction,
};

use crate::collection_values::{ToolbarEdge, ToolbarState};
/// Three optional regions that share a single keyboard Tab stop.
pub struct Toolbar {
    root: Node,
    // xtask-allow(state-fields): Independent presentation or construction input, not interaction state.
    start: Option<Vec<View>>,
    // xtask-allow(state-fields): Independent presentation or construction input, not interaction state.
    center: Option<Vec<View>>,
    // xtask-allow(state-fields): Independent presentation or construction input, not interaction state.
    end: Option<Vec<View>>,
    orientation: Orientation,
    size: Value<ControlSize>,
    // xtask-allow(state-fields): Independent presentation or construction input, not interaction state.
    gap: Option<Value<f64>>,
    dividers: Vec<ToolbarEdge>,
    variants: Variants,
}

/// Group controls under an accessible name. Arrow keys along the configured
/// axis move focus without activating controls; Home and End reach its ends.
/// Disabled controls are skipped unless explicitly focusable while disabled.
/// Use at least three controls for the APG toolbar pattern.
#[track_caller]
pub fn toolbar(label: impl IntoText) -> Toolbar {
    Toolbar {
        root: node().role(Role::Toolbar).name(label),
        start: None,
        center: None,
        end: None,
        orientation: Orientation::Horizontal,
        size: crate::values::default_control_size().into_value(),
        gap: None,
        dividers: Vec::new(),
        variants: Variants::default(),
    }
}
impl Toolbar {
    /// Controls at the logical start of the toolbar.
    pub fn start(mut self, children: impl IntoChildren) -> Self {
        self.start = Some(children.into_views());
        self
    }
    /// Controls centered independently of the widths of the outer regions.
    pub fn center(mut self, children: impl IntoChildren) -> Self {
        self.center = Some(children.into_views());
        self
    }
    /// Controls at the logical end of the toolbar.
    pub fn end(mut self, children: impl IntoChildren) -> Self {
        self.end = Some(children.into_views());
        self
    }
    /// The layout and keyboard navigation axis, fixed for this toolbar.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    /// Inherited size for controls that do not set their own size.
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
    /// Space in pixels between controls; otherwise the theme supplies it.
    pub fn gap(mut self, gap: impl IntoValue<f64>) -> Self {
        self.gap = Some(gap.into_value());
        self
    }
    /// Add dividers on logical edges.
    pub fn dividers(mut self, edges: &[ToolbarEdge]) -> Self {
        self.dividers = edges.to_vec();
        self
    }
    /// Add a theme presentation variant.
    pub fn variant(self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for Toolbar {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Toolbar {
    fn into_view(self) -> View {
        let orientation = self.orientation;
        let vertical = orientation == Orientation::Vertical;
        let rtl = use_direction().get() == ReadingDirection::Rtl;
        let size = self.size;
        let gap = self.gap;
        let centered = self.center.is_some();
        let dividers = self.dividers;
        let make = move |children, justify, column| {
            row(children).base_style(move || {
                let tokens = kivact_core::style::use_tokens::<crate::UiTokens>();
                style()
                    .min_width(0)
                    .align_items(Align::Center)
                    .justify_content(justify)
                    .direction(if vertical {
                        Direction::Column
                    } else if rtl {
                        Direction::RowReverse
                    } else {
                        Direction::Row
                    })
                    .gap(gap.map_or(tokens.toolbar.gap, |gap| gap.get()))
                    .grid_column(column)
            })
        };
        let regions = provide_control_size(size.get(), move || {
            (
                make(self.start, Justify::Start, if rtl { 3 } else { 1 }),
                self.center
                    .map(|children| make(Some(children), Justify::Center, 2)),
                make(self.end, Justify::End, if rtl { 1 } else { 3 }),
            )
        });
        with_base_recipe_style(
            self.root.orientation(orientation).roving_focus(orientation),
            |r| r.toolbar_recipes().root_recipe(),
            &self.variants,
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
        .into_view()
    }
}
