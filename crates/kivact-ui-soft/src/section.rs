//! Role: Draw a full-bleed Neutral section and propagate content spacing.
use crate::layout::LayoutInset;
use crate::stack::{ContainerInset, Padding, Space, padding_options, rtl};
use crate::tokens;
use kivact::control::{Node, NodeBuilder, column, node};
use kivact::primitive::NodeExt;
use kivact::style::{Color, Direction, style};
use kivact::view::{IntoView, View, provide, try_context};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SectionVariant {
    #[default]
    Section,
    Transparent,
    Muted,
}

/// Logical edges of a container.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Edges(u8);

impl Edges {
    pub const TOP: Self = Self(1);
    pub const BOTTOM: Self = Self(2);
    pub const START: Self = Self(4);
    pub const END: Self = Self(8);
    pub const ALL: Self = Self(15);

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
struct SectionPadding(f32);

pub struct Section {
    root: Node,
    content: View,
    variant: SectionVariant,
    padding: Padding,
    dividers: Edges,
    bleed_top: bool,
    bleed_bottom: bool,
}

pub fn section(content: impl IntoView) -> Section {
    Section {
        root: node(),
        content: content.into_view(),
        variant: SectionVariant::Section,
        padding: Padding::default(),
        dividers: Edges::default(),
        bleed_top: false,
        bleed_bottom: false,
    }
}

impl Section {
    padding_options!();
    pub fn variant(mut self, variant: SectionVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn dividers(mut self, dividers: Edges) -> Self {
        self.dividers = dividers;
        self
    }
    pub fn bleed_top(mut self, bleed: bool) -> Self {
        self.bleed_top = bleed;
        self
    }
    pub fn bleed_bottom(mut self, bleed: bool) -> Self {
        self.bleed_bottom = bleed;
        self
    }
}

impl NodeBuilder for Section {
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
        provide((), move || {
            let parent = try_context::<ContainerInset>().unwrap_or_default();
            let (left, right) = parent.sides();
            let inherited = try_context::<SectionPadding>()
                .map_or(tokens().spacing.section_padding, |padding| padding.0);
            let inset = padding.resolve(inherited);
            let layout_inset = LayoutInset::from_padding(inset);
            let content = provide(inset, move || provide(layout_inset, move || content));
            let content = match padding.all {
                Some(space) => provide(SectionPadding(space.px()), move || content),
                None => content,
            };
            root.base_style(move || {
                let t = tokens();
                let color = match variant {
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
                if bleed_top {
                    base = base.margin_top(-parent.top);
                }
                if bleed_bottom {
                    base = base.margin_bottom(-parent.bottom);
                }
                if dividers.contains(Edges::TOP) {
                    base = base.border_top(width, border);
                }
                if dividers.contains(Edges::BOTTOM) {
                    base = base.border_bottom(width, border);
                }
                let (left, right) = if rtl() {
                    (Edges::END, Edges::START)
                } else {
                    (Edges::START, Edges::END)
                };
                if dividers.contains(left) {
                    base = base.border_left(width, border);
                }
                if dividers.contains(right) {
                    base = base.border_right(width, border);
                }
                base
            })
            .child(column(content).base_style(inset.apply(style())))
        })
    }
}
