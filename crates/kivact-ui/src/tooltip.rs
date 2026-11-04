//! Role: Style shared tooltip surface and trigger parts without owning timers.
use super::layer::with_recipe_style;
use super::values::TooltipArrowState;
use super::variants::{Variant, Variants};
use kivact_core::geometry::Edge;
use kivact_core::geometry::Rect;
use kivact_core::state::Value;
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::view::{IntoView, NodeRef, OverlayAlign, Placement, View, node_ref, overlay};
use kivact_core::view::{NodeBuilder, node, text};

// xtask-allow(state-fields): Setup-only builder options; content and alignment are independent.
/// A themed hint for a trigger, created by [`tooltip`]; attach it with
/// [`attach`](Tooltip::attach).
pub struct Tooltip {
    label: Value<String>,
    enabled: Value<bool>,
    placement: Placement,
    align: Option<OverlayAlign>,
    offset: f64,
    content: Option<kivact_core::view::ViewFn>,
    variants: Variants,
}
/// A tooltip that says `label`, shown above its trigger by default.
///
/// The timing, the one-at-a-time rule and the dismissal are
/// [`use_tooltip`](kivact_interaction::use_tooltip)'s; this builder draws the
/// surface with the theme's [`TooltipRecipes`](super::TooltipRecipes) and
/// owns no timer.
pub fn tooltip(label: impl IntoText) -> Tooltip {
    Tooltip {
        label: label.into_text_value(),
        enabled: true.into_value(),
        placement: Placement::Above,
        align: None,
        offset: 4.0,
        content: None,
        variants: Variants::default(),
    }
}
impl Tooltip {
    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which every recipe of the component reads with
    /// [`Look::variant`](crate::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipes read the values given before
    /// the component is mounted, and a value never changes afterwards. A
    /// look that changes while the component lives is a state the recipe
    /// reads, or a reactive `.style(..)` of the caller's.
    pub fn variant(self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }
}
impl Tooltip {
    /// Render rich caller content in a fresh tooltip owner on each opening.
    pub fn content<V: IntoView>(mut self, factory: impl kivact_core::view::IntoViewFn<V>) -> Self {
        self.content = Some(factory.into_view_fn());
        self
    }

    /// Only reveal the hint while enabled.
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
    /// Show the hint for `trigger` and return the trigger with it. A trigger
    /// without a description of its own is described by the label; one that
    /// has a description keeps it. The surface is mounted in a fresh owner
    /// while the tooltip is shown. Moving the pointer onto the hint keeps it
    /// shown until the pointer leaves both the trigger and the hint.
    pub fn attach<T: NodeBuilder + IntoView>(self, mut trigger: T) -> View {
        let behavior = kivact_interaction::use_tooltip().disabled(move || !self.enabled.get());
        let part = behavior.trigger();
        let anchor = node_ref();
        let described = trigger.node_mut().has_description();
        let trigger = trigger.node_ref(anchor).with_node(|node| node.part(part));
        let trigger = if described {
            trigger
        } else {
            trigger.description(self.label)
        };
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
                    |r| r.tooltip_recipes().surface_recipe(),
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
/// [`TooltipRecipes::arrow`](super::TooltipRecipes) (by default an empty
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
        |r| r.tooltip_recipes().arrow_recipe(),
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
