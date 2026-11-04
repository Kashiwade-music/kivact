//! Role: Keep content clipped to a specified aspect ratio.
use kivact::state::Value;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::node;

use kivact::state::IntoValue;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::View;

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
    shape: Value<RatioShape>,
    fit: Value<RatioFit>,
}

/// A full-width box whose height follows the supplied width-to-height ratio.
pub fn aspect_ratio(ratio: impl IntoValue<f64>, children: impl IntoView) -> AspectRatio {
    AspectRatio {
        root: node(),
        children: children.into_view(),
        ratio: ratio.into_value(),
        shape: (RatioShape::Rectangle).into_value(),
        fit: (RatioFit::Natural).into_value(),
    }
}

impl AspectRatio {
    pub fn shape(mut self, shape: impl IntoValue<RatioShape>) -> Self {
        self.shape = shape.into_value();
        self
    }
    pub fn fit(mut self, fit: impl IntoValue<RatioFit>) -> Self {
        self.fit = fit.into_value();
        self
    }
}

impl NodeBuilder for AspectRatio {
    #[doc(hidden)]
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
        if fit.get() == RatioFit::Center {
            slot = slot
                .align_items(Align::Center)
                .justify_content(Justify::Center);
        } else if matches!(fit.get(), RatioFit::Cover | RatioFit::Contain) {
            slot = slot
                .display(kivact::style::Display::Grid)
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
            if shape.get() == RatioShape::Ellipse {
                base.radius(pct(50))
            } else {
                base
            }
        })
        .child(node().base_style(slot).child(children))
        .into_view()
    }
}

kivact::state::constant_value!(RatioFit, RatioShape);
