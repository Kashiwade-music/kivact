//! Role: Draw a full-bleed Neutral section and propagate content spacing.
use crate::color as c;
use crate::layout::LayoutInset;
use crate::metrics;
use crate::space::{SPACE_SECTION_PADDING_VAR, Space};
use crate::stack::{ContainerInset, Padding, rtl};
use crate::variants::SectionVariant;

use kivact::Prop;
use kivact::component;
use kivact::custom::Caller;
use kivact::state::Value;
use kivact::state::provide;
use kivact::state::try_use_context;
use kivact::style::Color;
use kivact::style::FlexDirection;
use kivact::style::style;
use kivact::style::use_theme;
use kivact::view::Element;
use kivact::view::View;
use kivact::view::column;
use kivact::view::node;

/// Logical edges of a section, for its dividers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub struct SectionEdges(u8);

impl SectionEdges {
    /// Top edge.
    pub const TOP: Self = Self(1);
    /// Bottom edge.
    pub const BOTTOM: Self = Self(2);
    /// Logical leading edge, following reading direction.
    pub const START: Self = Self(4);
    /// Logical trailing edge, following reading direction.
    pub const END: Self = Self(8);
    /// All four edges.
    pub const ALL: Self = Self(15);

    /// Return whether every edge in the supplied mask is present.
    pub fn contains(self, edge: Self) -> bool {
        self.0 & edge.0 == edge.0
    }
}

impl std::ops::BitOr for SectionEdges {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Clone, Copy)]
struct SectionPadding(f64);

/// Full-width surface that cancels inherited inline insets and pads its content.
#[component]
pub fn section(
    content: View,
    padding: Option<Space>,
    padding_x: Option<Space>,
    padding_y: Option<Space>,
    padding_start: Option<Space>,
    padding_end: Option<Space>,
    padding_top: Option<Space>,
    padding_bottom: Option<Space>,
    #[prop(default = SectionEdges::default())] dividers: Value<SectionEdges>,
    #[prop(default = false)] bleed_top: Value<bool>,
    #[prop(default = false)] bleed_bottom: Value<bool>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let parent = try_use_context::<ContainerInset>().unwrap_or_default();
    let (left, right) = parent.sides();
    let spacing = Padding {
        all: padding,
        x: padding_x,
        y: padding_y,
        start: padding_start,
        end: padding_end,
        top: padding_top,
        bottom: padding_bottom,
    };
    let inherited = try_use_context::<SectionPadding>().map_or(
        use_theme().get().get(&SPACE_SECTION_PADDING_VAR),
        |padding| padding.0,
    );
    let inset = spacing.resolve(inherited);
    let layout_inset = LayoutInset::from_padding(inset);
    let content = provide(inset, move || provide(layout_inset, move || content));
    let content = match padding {
        Some(space) => provide(SectionPadding(space.px()), move || content),
        None => content,
    };
    node()
        .with_node(move |node| {
            node.base_style(move || {
                let color = match variants.get::<SectionVariant>().unwrap_or_default() {
                    SectionVariant::Section => c::BG_SURFACE,
                    SectionVariant::Transparent => Color::TRANSPARENT,
                    SectionVariant::Muted => c::BG_MUTED,
                };
                let border = c::BORDER;
                let width = metrics::BORDER_WIDTH;
                let mut base = style()
                    .flex_direction(FlexDirection::Column)
                    .background(color)
                    .margin_left(-left)
                    .margin_right(-right);
                if bleed_top.get() {
                    base = base.margin_top(-parent.top);
                }
                if bleed_bottom.get() {
                    base = base.margin_bottom(-parent.bottom);
                }
                if dividers.get().contains(SectionEdges::TOP) {
                    base = base.border_top(width, border);
                }
                if dividers.get().contains(SectionEdges::BOTTOM) {
                    base = base.border_bottom(width, border);
                }
                let (left, right) = if rtl() {
                    (SectionEdges::END, SectionEdges::START)
                } else {
                    (SectionEdges::START, SectionEdges::END)
                };
                if dividers.get().contains(left) {
                    base = base.border_left(width, border);
                }
                if dividers.get().contains(right) {
                    base = base.border_right(width, border);
                }
                base
            })
        })
        .child(column(content).with_node(move |node| node.base_style(inset.apply(style()))))
}
