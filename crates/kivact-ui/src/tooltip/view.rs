//! Role: The tooltip builder, its surface and its arrow placement.

use crate::layer::with_recipe_style;
use crate::tooltip::look::TooltipArrowState;
use crate::tooltip::look::TooltipTheme;
use crate::variants::{Variant, Variants};
use kivact_core::geometry::Edge;
use kivact_core::geometry::Rect;
use kivact_core::state::Value;
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::view::{Element, Node, node, text};
use kivact_core::view::{IntoView, NodeRef, OverlayAlign, Placement, View, node_ref, overlay};

/// A themed hint for a trigger, created by [`tooltip`]. The builder is the
/// trigger's element: its modifiers, `.style(..)` and `.test_id(..)` among
/// them, go to the trigger's root.
pub struct Tooltip<T> {
    trigger: T,
    label: Value<String>,
    enabled: Value<bool>,
    placement: Placement,
    align: Option<OverlayAlign>,
    offset: f64,
    content: Option<kivact_core::view::ViewFn>,
    variants: Variants,
}
/// A tooltip that says `label` for `trigger`, shown above it by default.
/// The trigger stays the element the caller styles: `tooltip("Save",
/// button("Save"))`.
///
/// The timing, the one-at-a-time rule and the dismissal are
/// [`use_tooltip`](kivact_interaction::use_tooltip)'s; this builder draws the
/// surface with the theme's [`TooltipTheme`](crate::theme::TooltipTheme) and
/// owns no timer.
#[track_caller]
pub fn tooltip<T: Element + IntoView>(label: impl IntoText, trigger: T) -> Tooltip<T> {
    Tooltip {
        trigger,
        label: label.into_text_value(),
        enabled: true.into_value(),
        placement: Placement::Above,
        align: None,
        offset: 4.0,
        content: None,
        variants: Variants::default(),
    }
}
impl<T> Tooltip<T> {
    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which every recipe of the component reads with
    /// [`Look::variant`](crate::theme::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipes read the values given before
    /// the component is mounted, and a value never changes afterwards. A
    /// look that changes while the component lives is a state the recipe
    /// reads, or a reactive `.style(..)` of the caller's.
    pub fn variant(mut self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }
}
impl<T> Tooltip<T> {
    /// Render rich caller content in a fresh tooltip owner on each opening.
    pub fn content<V: IntoView>(mut self, factory: impl kivact_core::view::IntoViewFn<V>) -> Self {
        self.content = Some(factory.into_view_fn());
        self
    }

    /// Only reveal the hint while enabled.
    #[track_caller]
    pub fn enabled(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.enabled = enabled.into_value();
        self
    }

    /// Align the surface along its anchor, preserving the overlay default if unset.
    pub fn align(mut self, align: OverlayAlign) -> Self {
        self.align = Some(align);
        self
    }
    /// Set the gap from the anchor in logical pixels; the default is 4.
    pub fn offset(mut self, offset: impl Into<f64>) -> Self {
        let offset = offset.into();
        self.offset = offset;
        self
    }
    /// The side of the trigger the surface shows on (default
    /// [`Placement::Above`]).
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }
}

impl<T: Element + IntoView> Element for Tooltip<T> {
    fn node_mut(&mut self) -> &mut Node {
        self.trigger.node_mut()
    }
}

impl<T: Element + IntoView> IntoView for Tooltip<T> {
    /// Show the hint for the trigger and return the trigger with it. A trigger
    /// without a description of its own is described by the label; one that
    /// has a description keeps it. The surface is mounted in a fresh owner
    /// while the tooltip is shown. Moving the pointer onto the hint keeps it
    /// shown until the pointer leaves both the trigger and the hint.
    fn into_view(self) -> View {
        let behavior = kivact_interaction::use_tooltip().disabled(move || !self.enabled.get());
        let part = behavior.trigger();
        let anchor = node_ref();
        let fallback = kivact_core::custom::Part::new(part.state(), move |mut node| {
            // Earlier parts publish their semantics when the node is resolved.
            if node.has_description() {
                node
            } else {
                node.description(self.label)
            }
        });
        let trigger = self
            .trigger
            .node_ref(anchor)
            .with_node(|node| node.part(part).part(fallback));
        let surface = kivact_core::view::show(behavior.is_open(), move || {
            let part = behavior.content();
            let state = part.state();
            let placed = node_ref();
            let layer = overlay(
                anchor,
                self.placement,
                with_recipe_style(
                    node()
                        .node_ref(placed)
                        .child(self.content.as_ref().map_or_else(
                            || text(self.label).into_view(),
                            |factory| factory.call(()),
                        ))
                        .child(arrow(anchor, placed, self.placement, &self.variants)),
                    |r: &TooltipTheme| r.surface_recipe(),
                    &self.variants,
                    move || state.get(),
                ),
            )
            .with_node(|node| node.part(part))
            .name(self.label)
            .keep_anchor_clear()
            .offset(self.offset);
            match self.align {
                Some(align) => layer.align(align),
                None => layer,
            }
        });
        (trigger, surface).into_view()
    }
}

/// The arrow box of the surface bound to `placed`, styled by the theme's
/// [`TooltipTheme::arrow`](crate::theme::TooltipTheme) (by default an empty
/// box out of the layout).
fn arrow(
    anchor: NodeRef,
    placed: NodeRef,
    requested: Placement,
    variants: &Variants,
) -> kivact_core::view::Node {
    let (anchor, placed) = (anchor.rect(), placed.rect());
    let state =
        kivact_core::state::memo(move || arrow_state(anchor.get(), placed.get(), requested));
    with_recipe_style(
        node(),
        |r: &TooltipTheme| r.arrow_recipe(),
        variants,
        move || state.get(),
    )
}

/// The side the surface shows on and the anchor's centre along its edge,
/// from where the host placed both; the requested side until then.
fn arrow_state(
    anchor: Option<Rect>,
    placed: Option<Rect>,
    requested: Placement,
) -> TooltipArrowState {
    let asked = match requested {
        Placement::Below => Edge::Bottom,
        Placement::Start => Edge::Left,
        Placement::End => Edge::Right,
        Placement::Above | Placement::Center => Edge::Top,
    };
    let (Some(a), Some(s)) = (anchor, placed) else {
        return TooltipArrowState {
            side: asked,
            offset: 0.0,
        };
    };
    // Half a pixel of slack for rounded positions.
    let side = if s.y + s.height <= a.y + 0.5 {
        Edge::Top
    } else if s.y >= a.y + a.height - 0.5 {
        Edge::Bottom
    } else if s.x + s.width <= a.x + 0.5 {
        Edge::Left
    } else if s.x >= a.x + a.width - 0.5 {
        Edge::Right
    } else {
        asked
    };
    let offset = match side {
        Edge::Top | Edge::Bottom => (a.x + a.width / 2.0 - s.x).clamp(0.0, s.width),
        Edge::Left | Edge::Right => (a.y + a.height / 2.0 - s.y).clamp(0.0, s.height),
    };
    TooltipArrowState { side, offset }
}
