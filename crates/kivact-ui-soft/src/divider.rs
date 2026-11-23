//! Role: Draw the Neutral separator and its optional label.
use crate::color as c;
use crate::metrics;
use crate::space as s;
use crate::stack::ContainerInset;
use crate::type_scale as t;

use kivact::Prop;
use kivact::component;
use kivact::custom::Caller;
use kivact::state::Value;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::FlexDirection;
use kivact::style::style;
use kivact::ui::theme::Variant;
use kivact::view::Element;
use kivact::view::Orientation;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::text;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
/// Theme border color used by a separator.
pub enum DividerVariant {
    #[default]
    /// Use the normal border color; the default.
    Subtle,
    /// Use the emphasized border color.
    Strong,
}

impl Variant for DividerVariant {}

/// Semantic separator with optional content between two rule segments.
#[component]
pub fn divider(
    #[prop(default = Orientation::Horizontal)] orientation: Value<Orientation>,
    #[prop(default = false)] full_bleed: Value<bool>,
    label: Option<View>,
    label_text: Option<Value<String>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let mut root = node().with_node(move |node| node.role(Role::Separator));
    let mut label = label;
    if let Some(value) = label_text {
        root = root.name(value);
        label = Some(text(value).into_view());
    }
    let vertical = orientation.get() == Orientation::Vertical;
    let mut root = root.orientation(orientation.get()).with_node(move |node| {
        node.base_style(move || {
            let inset = try_use_context::<ContainerInset>().unwrap_or_default();
            let mut base = style()
                .flex_shrink(0)
                .align_self(Align::Stretch)
                .align_items(Align::Center);
            if vertical {
                base = base.flex_direction(FlexDirection::Column);
                if full_bleed.get() {
                    base = base.margin_top(-inset.top).margin_bottom(-inset.bottom);
                }
            } else {
                base = base.flex_direction(FlexDirection::Row);
                if full_bleed.get() {
                    let (left, right) = inset.sides();
                    base = base.margin_left(-left).margin_right(-right);
                }
            }
            base
        })
    });
    let rule = || {
        let variants = variants.clone();
        node().style(move || {
            let color = match variants.get::<DividerVariant>().unwrap_or_default() {
                DividerVariant::Subtle => c::BORDER,
                DividerVariant::Strong => c::BORDER_EMPHASIZED,
            };
            let rule = style().flex_grow(1).flex_shrink(1).background(color);
            if vertical {
                rule.width(metrics::BORDER_WIDTH)
            } else {
                rule.height(metrics::BORDER_WIDTH)
            }
        })
    };
    root = root.child(rule());
    if let Some(label) = label {
        let label_ref = node_ref();
        root = root
            .with_node(move |node| node.labelled_by(label_ref))
            .child(
                node()
                    .node_ref(label_ref)
                    .style(move || {
                        let label = style()
                            .flex_shrink(0)
                            .font_size(t::SUPPORTING.size_px)
                            .line_height(t::SUPPORTING.line_height_px / t::SUPPORTING.size_px)
                            .color(c::TEXT_SECONDARY);
                        if vertical {
                            label.padding_y(s::SPACE_S3)
                        } else {
                            label.padding_x(s::SPACE_S3)
                        }
                    })
                    .child(label),
            )
            .child(rule());
    }
    root
}
