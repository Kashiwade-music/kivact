//! Role: Render badges, loading placeholders, and status indicators.

use kivact::motion::Animated;
use kivact::motion::Easing;
use kivact::motion::Repeat;
use kivact::motion::Transition;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::node;
use kivact::view::row;

use kivact::state::IntoText;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::Role;
use kivact::view::View;

pub(crate) fn icon_box(icon: View, size: f64) -> Node {
    node().style(style().size(size, size).shrink(0)).child(icon)
}

/// Corner profile of a loading placeholder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonRadius {
    None,
    R0,
    R1,
    R2,
    #[default]
    R3,
    R4,
    Rounded,
}

/// Decorative loading placeholder.
pub struct Skeleton {
    root: Node,
    radius: Value<SkeletonRadius>,
    index: usize,
}

#[track_caller]
pub fn skeleton() -> Skeleton {
    Skeleton {
        root: node(),
        radius: (SkeletonRadius::R3).into_value(),
        index: 0,
    }
}

impl Skeleton {
    pub fn radius(mut self, radius: impl IntoValue<SkeletonRadius>) -> Self {
        self.radius = radius.into_value();
        self
    }

    pub fn index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }
}

impl NodeBuilder for Skeleton {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Skeleton {
    fn into_view(self) -> View {
        let radius = self.radius.get();
        let pulse = Animated::from(
            0.25f64,
            1.0f64,
            Transition::tween(400, Easing::steps_end(10))
                .delay(1000.0 + self.index as f64 * 100.0)
                .repeat(Repeat::Forever)
                .alternate(),
        );
        self.root
            .with_node(move |node| {
                node.base_style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let radius = match radius {
                        SkeletonRadius::None | SkeletonRadius::R0 => 0.0,
                        SkeletonRadius::R1 => t.radius.inner,
                        SkeletonRadius::R2 => t.radius.element,
                        SkeletonRadius::R3 | SkeletonRadius::R4 => t.radius.container,
                        SkeletonRadius::Rounded => t.radius.full,
                    };
                    let opacity = if kivact::platform::is_motion_reduced().get() {
                        0.25
                    } else {
                        pulse.get().min(0.925)
                    };
                    style()
                        .size(pct(100), pct(100))
                        .radius(radius)
                        .background(t.colors.skeleton)
                        .opacity(opacity)
                })
            })
            .into_view()
    }
}

/// Status represented by a dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusDotVariant {
    Success,
    Warning,
    Error,
    Accent,
    Neutral,
}

/// Named noninteractive status indicator.
pub struct StatusDot {
    root: Node,
    variant: StatusDotVariant,
    icon: Option<View>,
    pulsing: Value<bool>,
}

#[track_caller]
pub fn status_dot(variant: StatusDotVariant, label: impl IntoText) -> StatusDot {
    StatusDot {
        root: row(())
            .with_node(move |node| node.role(Role::Image))
            .name(label),
        variant,
        icon: None,
        pulsing: (false).into_value(),
    }
}

impl StatusDot {
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    pub fn pulsing(mut self, pulsing: impl IntoValue<bool>) -> Self {
        self.pulsing = pulsing.into_value();
        self
    }
}

impl NodeBuilder for StatusDot {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for StatusDot {
    fn into_view(self) -> View {
        let variant = self.variant;
        let pulse = self.pulsing.get().then(|| {
            Animated::from(
                1.0f64,
                0.5f64,
                Transition::tween(1000, Easing::CubicBezier(0.42, 0.0, 0.58, 1.0))
                    .repeat(Repeat::Forever)
                    .alternate(),
            )
        });
        self.root
            .with_node(move |node| {
                node.base_style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let c = &t.colors;
                    let (fill, ink) = match variant {
                        StatusDotVariant::Success => (c.success, c.on_success),
                        StatusDotVariant::Warning => (c.warning, c.on_warning),
                        StatusDotVariant::Error => (c.error, c.on_error),
                        StatusDotVariant::Accent => (c.accent, c.on_accent),
                        StatusDotVariant::Neutral => (c.icon_secondary, c.bg_surface),
                    };
                    let opacity = if kivact::platform::is_motion_reduced().get() {
                        1.0
                    } else {
                        pulse.as_ref().map_or(1.0, Animated::get)
                    };
                    style()
                        .size(8, 8)
                        .shrink(0)
                        .radius(t.radius.full)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                        .background(fill)
                        .color(ink)
                        .opacity(opacity)
                })
            })
            .child(self.icon.map(|icon| icon_box(icon, 8.0)))
            .into_view()
    }
}

kivact::state::constant_value!(SkeletonRadius);
