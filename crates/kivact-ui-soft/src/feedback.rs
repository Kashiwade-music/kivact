//! Role: Render badges, loading placeholders, and status indicators.
use crate::tokens;
use kivact::Value;
use kivact::control::{Node, NodeBuilder, node, row, text};
use kivact::motion::{Animated, Easing, Repeat, Transition};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Justify, pct, style};
use kivact::view::{IntoText, IntoView, Role, View};

/// Semantic or decorative colors of a read-only badge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Neutral,
    Info,
    Success,
    Warning,
    Error,
    Blue,
    Cyan,
    Green,
    Orange,
    Pink,
    Purple,
    Red,
    Teal,
    Yellow,
    Gray,
}

/// Read-only status label.
pub struct Badge {
    root: Node,
    label: Value<String>,
    variant: BadgeVariant,
    icon: Option<View>,
}

#[track_caller]
pub fn badge(label: impl IntoText) -> Badge {
    let label = label.into_text_value();
    Badge {
        root: row(()).name(label),
        label,
        variant: BadgeVariant::Neutral,
        icon: None,
    }
}

impl Badge {
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
}

impl NodeBuilder for Badge {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Badge {
    fn into_view(self) -> View {
        let variant = self.variant;
        self.root
            .base_style(move || {
                let t = tokens();
                let c = &t.colors;
                let (fill, ink) = match variant {
                    BadgeVariant::Info => (c.info, c.on_info),
                    BadgeVariant::Success => (c.success, c.on_success),
                    BadgeVariant::Warning => (c.warning, c.on_warning),
                    BadgeVariant::Error => (c.error, c.on_error),
                    _ => {
                        let hue = match variant {
                            BadgeVariant::Blue => c.hue.blue,
                            BadgeVariant::Cyan => c.hue.cyan,
                            BadgeVariant::Green => c.hue.green,
                            BadgeVariant::Orange => c.hue.orange,
                            BadgeVariant::Pink => c.hue.pink,
                            BadgeVariant::Purple => c.hue.purple,
                            BadgeVariant::Red => c.hue.red,
                            BadgeVariant::Teal => c.hue.teal,
                            BadgeVariant::Yellow => c.hue.yellow,
                            _ => c.hue.gray,
                        };
                        (hue.bg, hue.text)
                    }
                };
                style()
                    .min_height(t.sizes.badge_height)
                    .min_width(0)
                    .max_width(pct(100))
                    .radius(t.radius.full)
                    .padding_x(t.spacing.s2)
                    .gap(t.spacing.s1)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .background(fill)
                    .color(ink)
                    .font_size(12)
                    .line_height(20.0 / 12.0)
                    .weight(500)
            })
            .child(self.icon.map(|icon| icon_box(icon, 12.0)))
            .child(text(self.label).style(style().min_width(0).text_wrap(false).line_clamp(1)))
            .into_view()
    }
}

pub(crate) fn icon_box(icon: View, size: f32) -> Node {
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
    radius: SkeletonRadius,
    index: usize,
}

#[track_caller]
pub fn skeleton() -> Skeleton {
    Skeleton {
        root: node(),
        radius: SkeletonRadius::R3,
        index: 0,
    }
}

impl Skeleton {
    pub fn radius(mut self, radius: SkeletonRadius) -> Self {
        self.radius = radius;
        self
    }

    pub fn index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }
}

impl NodeBuilder for Skeleton {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Skeleton {
    fn into_view(self) -> View {
        let radius = self.radius;
        let pulse = Animated::from(
            0.25f32,
            1.0f32,
            Transition::tween(400, Easing::steps_end(10))
                .delay(1000.0 + self.index as f64 * 100.0)
                .repeat(Repeat::Forever)
                .alternate(),
        );
        self.root
            .base_style(move || {
                let t = tokens();
                let radius = match radius {
                    SkeletonRadius::None | SkeletonRadius::R0 => 0.0,
                    SkeletonRadius::R1 => t.radius.inner,
                    SkeletonRadius::R2 => t.radius.element,
                    SkeletonRadius::R3 | SkeletonRadius::R4 => t.radius.container,
                    SkeletonRadius::Rounded => t.radius.full,
                };
                let opacity = if kivact::platform::reduced_motion() {
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
    pulsing: bool,
}

#[track_caller]
pub fn status_dot(variant: StatusDotVariant, label: impl IntoText) -> StatusDot {
    StatusDot {
        root: row(()).role(Role::Image).name(label),
        variant,
        icon: None,
        pulsing: false,
    }
}

impl StatusDot {
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    pub fn pulsing(mut self, pulsing: bool) -> Self {
        self.pulsing = pulsing;
        self
    }
}

impl NodeBuilder for StatusDot {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for StatusDot {
    fn into_view(self) -> View {
        let variant = self.variant;
        let pulse = self.pulsing.then(|| {
            Animated::from(
                1.0f32,
                0.5f32,
                Transition::tween(1000, Easing::CubicBezier(0.42, 0.0, 0.58, 1.0))
                    .repeat(Repeat::Forever)
                    .alternate(),
            )
        });
        self.root
            .base_style(move || {
                let t = tokens();
                let c = &t.colors;
                let (fill, ink) = match variant {
                    StatusDotVariant::Success => (c.success, c.on_success),
                    StatusDotVariant::Warning => (c.warning, c.on_warning),
                    StatusDotVariant::Error => (c.error, c.on_error),
                    StatusDotVariant::Accent => (c.accent, c.on_accent),
                    StatusDotVariant::Neutral => (c.icon_secondary, c.bg_surface),
                };
                let opacity = if kivact::platform::reduced_motion() {
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
            .child(self.icon.map(|icon| icon_box(icon, 8.0)))
            .into_view()
    }
}
