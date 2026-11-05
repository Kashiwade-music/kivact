//! Role: Draw presence indicators and report their labels to the containing avatar.

use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::node;

use kivact::state::IntoText;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::state::effect;
use kivact::state::on_cleanup;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::Color;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::Role;
use kivact::view::View;

#[derive(Clone, Copy)]
pub(crate) struct AvatarStatusContext {
    pub(crate) side: f64,
    pub(crate) label: Signal<Option<String>>,
}

/// The meaning and appearance of a presence indicator.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AvatarStatus {
    #[default]
    Success,
    Neutral,
    Error,
    Custom {
        fill: Color,
        ink: Color,
    },
}

pub(crate) struct StatusMetrics {
    pub(crate) side: f64,
    border: f64,
    stroke: f64,
    icon: f64,
}

pub(crate) fn status_metrics(side: f64) -> StatusMetrics {
    if side <= 36.0 {
        StatusMetrics {
            side: 10.0,
            border: 1.0,
            stroke: 1.0,
            icon: 0.0,
        }
    } else if side < 96.0 {
        StatusMetrics {
            side: 20.0,
            border: 2.0,
            stroke: 1.5,
            icon: 12.0,
        }
    } else {
        StatusMetrics {
            side: 32.0,
            border: 4.0,
            stroke: 2.0,
            icon: 18.0,
        }
    }
}

/// A small presence indicator, usable with or without an avatar.
pub struct AvatarStatusDot {
    root: Node,
    status: AvatarStatus,
    label: Option<Value<String>>,
    icon: Option<View>,
}

/// Create a presence indicator.
pub fn avatar_status_dot(status: AvatarStatus) -> AvatarStatusDot {
    AvatarStatusDot {
        root: node(),
        status,
        label: None,
        icon: None,
    }
}

impl AvatarStatusDot {
    /// Set the text exposed to assistive technology.
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = Some(label.into_text_value());
        self
    }

    /// Replace the default glyph at medium and large sizes.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
}

impl NodeBuilder for AvatarStatusDot {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for AvatarStatusDot {
    fn into_view(self) -> View {
        let Self {
            mut root,
            status,
            label,
            icon,
        } = self;
        let cx = try_use_context::<AvatarStatusContext>().map(|value| value.get());
        let StatusMetrics {
            side,
            border,
            stroke,
            icon: icon_side,
        } = status_metrics(cx.map_or(36.0, |cx| cx.side));
        if let Some(cx) = cx {
            effect(move || {
                cx.label.set(
                    label
                        .map(|value| value.get())
                        .filter(|text| !text.trim().is_empty()),
                )
            });
            on_cleanup(move || cx.label.set(None));
        } else if let Some(label) = label {
            root = root
                .with_node(move |node| node.role(Role::Image))
                .name(label);
        }
        let field = side - 2.0 * border;
        let glyph = if let Some(icon) = icon.filter(|_| icon_side > 0.0) {
            Some(
                node()
                    .style(
                        style()
                            .size(icon_side, icon_side)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center),
                    )
                    .child(icon),
            )
        } else {
            match status {
                AvatarStatus::Neutral => Some(node().style(move || {
                    style()
                        .size(field, field)
                        .radius(kivact::style::use_tokens::<crate::Tokens>().radius.full)
                        .border(
                            stroke,
                            kivact::style::use_tokens::<crate::Tokens>()
                                .colors
                                .text_secondary,
                        )
                })),
                AvatarStatus::Error => Some(node().style(move || {
                    style()
                        .size(field * 0.75, stroke)
                        .radius(kivact::style::use_tokens::<crate::Tokens>().radius.full)
                        .background(kivact::style::use_tokens::<crate::Tokens>().colors.on_error)
                })),
                _ => None,
            }
        };
        root.with_node(move |node| {
            node.base_style(move || {
                let p = kivact::style::use_tokens::<crate::Tokens>();
                let (fill, ink) = match status {
                    AvatarStatus::Success => (p.colors.success, p.colors.on_success),
                    AvatarStatus::Neutral => (p.colors.bg_surface, p.colors.text_secondary),
                    AvatarStatus::Error => (p.colors.error, p.colors.on_error),
                    AvatarStatus::Custom { fill, ink } => (fill, ink),
                };
                style()
                    .size(side, side)
                    .shrink(0)
                    .radius(p.radius.full)
                    .border(border, p.colors.bg_surface)
                    .background(fill)
                    .color(ink)
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
            })
        })
        .child(glyph)
        .into_view()
    }
}
