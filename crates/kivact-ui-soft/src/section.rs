//! Role: Draw a full-bleed Neutral section and propagate content spacing.
use crate::layout::LayoutInset;
use crate::stack::{ContainerInset, Padding, Space, padding_options, rtl};

use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::column;
use kivact::view::node;

use kivact::state::provide;
use kivact::state::try_use_context;
use kivact::style::Color;
use kivact::style::Direction;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::View;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Background treatment of a full-bleed section.
pub enum SectionVariant {
    #[default]
    /// Use the surface background; the default.
    Section,
    /// Leave the section background transparent.
    Transparent,
    /// Use the muted background.
    Muted,
}

/// Logical edges of a container.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Edges(u8);

impl Edges {
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

impl std::ops::BitOr for Edges {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Clone, Copy)]
struct SectionPadding(f64);

/// Full-width surface that cancels inherited inline insets and pads its content.
pub struct Section {
    root: Node,
    content: View,
    variant: Value<SectionVariant>,
    padding: Padding,
    dividers: Value<Edges>,
    bleed_top: Value<bool>,
    bleed_bottom: Value<bool>,
}

/// Create a surface-colored section with inherited section padding and no dividers.
pub fn section(content: impl IntoView) -> Section {
    Section {
        root: node(),
        content: content.into_view(),
        variant: (SectionVariant::Section).into_value(),
        padding: Padding::default(),
        dividers: (Edges::default()).into_value(),
        bleed_top: (false).into_value(),
        bleed_bottom: (false).into_value(),
    }
}

impl Section {
    padding_options!();
    /// Set a constant or reactive section background treatment.
    pub fn variant(mut self, variant: impl IntoValue<SectionVariant>) -> Self {
        self.variant = variant.into_value();
        self
    }
    /// Set constant or reactive border edges; combine edges with `|`.
    pub fn dividers(mut self, dividers: impl IntoValue<Edges>) -> Self {
        self.dividers = dividers.into_value();
        self
    }
    /// Reactively extend into the parent’s top inset; disabled by default.
    pub fn bleed_top(mut self, bleed: impl IntoValue<bool>) -> Self {
        self.bleed_top = bleed.into_value();
        self
    }
    /// Reactively extend into the parent’s bottom inset; disabled by default.
    pub fn bleed_bottom(mut self, bleed: impl IntoValue<bool>) -> Self {
        self.bleed_bottom = bleed.into_value();
        self
    }
}

impl NodeBuilder for Section {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Section {
    fn into_view(self) -> View {
        let Self {
            root,
            content,
            variant,
            padding,
            dividers,
            bleed_top,
            bleed_bottom,
        } = self;
        kivact::view::component(move || {
            let parent = try_use_context::<ContainerInset>()
                .map(|value| value.get())
                .unwrap_or_default();
            let (left, right) = parent.sides();
            let inherited = try_use_context::<SectionPadding>()
                .map(|value| value.get())
                .map_or(
                    kivact::style::use_tokens::<crate::Tokens>()
                        .spacing
                        .section_padding,
                    |padding| padding.0,
                );
            let inset = padding.resolve(inherited);
            let layout_inset = LayoutInset::from_padding(inset);
            let content = provide(inset, move || provide(layout_inset, move || content));
            let content = match padding.all {
                Some(space) => provide(SectionPadding(space.px()), move || content),
                None => content,
            };
            root.with_node(move |node| {
                node.base_style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let color = match variant.get() {
                        SectionVariant::Section => t.colors.bg_surface,
                        SectionVariant::Transparent => Color::TRANSPARENT,
                        SectionVariant::Muted => t.colors.bg_muted,
                    };
                    let border = t.colors.border;
                    let width = t.sizes.border_width;
                    let mut base = style()
                        .direction(Direction::Column)
                        .background(color)
                        .margin_left(-left)
                        .margin_right(-right);
                    if bleed_top.get() {
                        base = base.margin_top(-parent.top);
                    }
                    if bleed_bottom.get() {
                        base = base.margin_bottom(-parent.bottom);
                    }
                    if dividers.get().contains(Edges::TOP) {
                        base = base.border_top(width, border);
                    }
                    if dividers.get().contains(Edges::BOTTOM) {
                        base = base.border_bottom(width, border);
                    }
                    let (left, right) = if rtl() {
                        (Edges::END, Edges::START)
                    } else {
                        (Edges::START, Edges::END)
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
        })
    }
}

kivact::state::constant_value!(SectionVariant);

kivact::state::constant_value!(Edges);
