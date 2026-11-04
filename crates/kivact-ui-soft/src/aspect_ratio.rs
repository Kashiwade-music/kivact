//! Role: Keep content clipped to a specified aspect ratio.
use kivact::Value;
use kivact::control::{Node, NodeBuilder, node};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Justify, Position, pct, style};
use kivact::view::{IntoValue, IntoView, View};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RatioShape {
    #[default]
    Rectangle,
    Ellipse,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RatioFit {
    #[default]
    Natural,
    Cover,
    Contain,
    Center,
}

pub struct AspectRatio {
    root: Node,
    children: View,
    ratio: Value<f64>,
    shape: RatioShape,
    fit: RatioFit,
}

/// A full-width box whose height follows the supplied width-to-height ratio.
pub fn aspect_ratio(ratio: impl IntoValue<f64>, children: impl IntoView) -> AspectRatio {
    AspectRatio {
        root: node(),
        children: children.into_view(),
        ratio: ratio.into_value(),
        shape: RatioShape::Rectangle,
        fit: RatioFit::Natural,
    }
}

impl AspectRatio {
    pub fn shape(mut self, shape: RatioShape) -> Self {
        self.shape = shape;
        self
    }
    pub fn fit(mut self, fit: RatioFit) -> Self {
        self.fit = fit;
        self
    }
}

impl NodeBuilder for AspectRatio {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for AspectRatio {
    fn into_view(self) -> View {
        let Self {
            root,
            children,
            ratio,
            shape,
            fit,
        } = self;
        let mut slot = style()
            .position(Position::Absolute)
            .top(0)
            .right(0)
            .bottom(0)
            .left(0);
        if fit == RatioFit::Center {
            slot = slot
                .align_items(Align::Center)
                .justify_content(Justify::Center);
        } else if matches!(fit, RatioFit::Cover | RatioFit::Contain) {
            slot = slot
                .display_grid()
                .grid_template_rows(pct(100))
                .grid_template_columns(pct(100))
                .align_items(Align::Stretch)
                .justify_items(Align::Stretch);
        }
        root.base_style(move || {
            let ratio = ratio.get();
            let base = style()
                .position(Position::Relative)
                .width(pct(100))
                .aspect_ratio(if ratio.is_finite() && ratio > 0.0 {
                    ratio
                } else {
                    1.0
                })
                .clip(true)
                .shrink(0)
                .min_height(0);
            if shape == RatioShape::Ellipse {
                base.radius(pct(50))
            } else {
                base
            }
        })
        .child(node().base_style(slot).child(children))
        .into_view()
    }
}
