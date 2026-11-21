//! Role: Define responsive application layout insets.
use crate::color as c;
use crate::metrics;
use crate::space::Space;
use crate::stack::{ContainerInset, rtl};
use kivact::component;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::provide;
use kivact::state::try_use_context;
use kivact::style::FlexDirection;
use kivact::style::Length;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::Element;

use kivact::view::NodeRef;
use kivact::view::View;
use kivact::view::column;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::scroll;

use kivact::Prop;

/// Insets inherited by Layout regions from a parent surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutInset {
    /// Inherited inline outer padding in logical pixels; None uses the region fallback.
    pub outer_x: Option<f64>,
    /// Inherited block outer padding in logical pixels; None uses the region fallback.
    pub outer_y: Option<f64>,
    /// Padding between inline regions in logical pixels; None uses the region fallback.
    pub inner_x: Option<f64>,
    /// Padding between block regions in logical pixels; None uses the region fallback.
    pub inner_y: Option<f64>,
    /// This layout's own outer padding (`Layout::padding`) on both axes in
    /// logical pixels; when set it replaces `outer_x` and `outer_y`.
    pub(crate) own_outer: Option<f64>,
}

impl LayoutInset {
    pub(crate) fn from_padding(padding: ContainerInset) -> Self {
        let x = Some(padding.start);
        let y = Some(padding.top);
        Self {
            outer_x: x,
            outer_y: y,
            inner_x: x,
            inner_y: y,
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
/// Whether a page layout fills its parent or follows intrinsic content height.
pub enum LayoutHeight {
    #[default]
    /// Fill the available parent height; the default.
    Fill,
    /// Follow content height while retaining at least the parent height.
    Auto,
}

/// Page regions arranged around a central content area.
#[component]
pub fn layout(
    header: Option<LayoutRegion>,
    footer: Option<LayoutRegion>,
    start: Option<LayoutRegion>,
    end: Option<LayoutRegion>,
    content: Option<LayoutRegion>,
    start_resize: Option<View>,
    end_resize: Option<View>,
    #[prop(default = LayoutHeight::Fill)] height: Value<LayoutHeight>,
    padding: Option<Space>,
    content_width: Option<Value<Length>>,
    #[prop(default = false)] default_dividers: Value<bool>,
) -> impl Element {
    let parent = try_use_context::<ContainerInset>().unwrap_or_default();
    let mut insets = try_use_context::<LayoutInset>().unwrap_or_default();
    insets.own_outer = padding.map(Space::px);
    let mut header = header;
    let mut footer = footer;
    for region in [&mut header, &mut footer].into_iter().flatten() {
        region.divider.get_or_insert(default_dividers.get());
    }
    let root_ref = node_ref();
    let slots = LayoutSlots {
        header_divider: header.as_ref().and_then(|region| region.divider),
        footer_divider: footer.as_ref().and_then(|region| region.divider),
        has_start: start.is_some(),
        has_end: end.is_some(),
        content_width: content_width.map(|width| ContentWidth {
            root: root_ref,
            width,
        }),
    };
    let start = start.map(|slot| slot.in_region(Region::Start));
    let end = end.map(|slot| slot.in_region(Region::End));
    let content = content.map(|slot| slot.in_region(Region::Content));
    let (left, right) = parent.sides();
    let base = style()
        .flex_direction(FlexDirection::Column)
        .min_width(0)
        .margin_left(-left)
        .margin_right(-right)
        .margin_top(-parent.top)
        .margin_bottom(-parent.bottom);
    let base = match height.get() {
        LayoutHeight::Fill => base
            .flex_grow(1)
            .min_height(0)
            .height(pct(100) + parent.top + parent.bottom),
        LayoutHeight::Auto => base.min_height(pct(100)),
    };
    node()
        .node_ref(root_ref)
        .with_node(move |node| node.base_style(base))
        .child(provide(insets, move || {
            provide(ContainerInset::default(), move || {
                provide(slots, move || {
                    (
                        header.map(region_view),
                        node()
                            .with_node(move |node| node.base_style(move || layout_row_style(slots)))
                            .child((
                                start.map(region_view),
                                start_resize,
                                content.map(region_view),
                                end_resize,
                                end.map(region_view),
                            )),
                        footer.map(region_view),
                    )
                })
            })
        }))
}

/// A caller width limit with the layout root used to centre pixel widths.
#[derive(Clone, Copy)]
struct ContentWidth {
    root: NodeRef,
    width: Value<Length>,
}

#[derive(Clone, Copy, Default)]
struct LayoutSlots {
    header_divider: Option<bool>,
    footer_divider: Option<bool>,
    has_start: bool,
    has_end: bool,
    content_width: Option<ContentWidth>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Region {
    Header,
    Content,
    Footer,
    Start,
    End,
}

/// Content slot with padding, size, scrolling and divider settings for a page layout.
///
/// A region is a value that a [`layout`] places: its side (header, footer,
/// start, end or content) is set by the slot it is given to.
pub struct LayoutRegion {
    children: View,
    kind: Region,
    divider: Option<bool>,
    padding: Option<Space>,
    padding_bottom: Option<Space>,
    scrollable: Value<bool>,
    width: Option<Value<Length>>,
    height: Option<Value<Length>>,
}

fn region(kind: Region, children: View) -> LayoutRegion {
    LayoutRegion {
        children,
        kind,
        divider: None,
        padding: None,
        padding_bottom: None,
        scrollable: (!matches!(kind, Region::Header | Region::Footer)).into_value(),
        width: None,
        height: None,
    }
}

/// Create a non-scrolling header region.
pub fn layout_header(children: impl kivact::view::IntoView) -> LayoutRegion {
    region(Region::Header, children.into_view())
}
/// Create a scrolling central content region.
pub fn layout_content(children: impl kivact::view::IntoView) -> LayoutRegion {
    region(Region::Content, children.into_view())
}
/// Create a scrolling side region; assigning it to start/end determines its logical side.
pub fn layout_panel(children: impl kivact::view::IntoView) -> LayoutRegion {
    region(Region::Start, children.into_view())
}

impl LayoutRegion {
    fn in_region(mut self, kind: Region) -> Self {
        self.kind = kind;
        self
    }
    /// Override whether this region draws its boundary divider.
    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = Some(divider);
        self
    }
    /// Replace this region’s computed insets with uniform theme spacing.
    pub fn padding(mut self, padding: Space) -> Self {
        self.padding = Some(padding);
        self
    }
    /// Override the region’s bottom padding with a theme spacing step.
    pub fn padding_bottom(mut self, padding: Space) -> Self {
        self.padding_bottom = Some(padding);
        self
    }
    /// Choose at view construction whether this region owns a scroll viewport.
    pub fn scrollable(mut self, scrollable: impl kivact::state::IntoValue<bool>) -> Self {
        self.scrollable = scrollable.into_value();
        self
    }
    /// Set constant or reactive width for a leading or trailing panel.
    pub fn width(mut self, width: impl kivact::state::IntoValue<Length>) -> Self {
        self.width = Some(width.into_value());
        self
    }
    /// Set a constant or reactive height using a logical length.
    pub fn height(mut self, height: impl kivact::state::IntoValue<Length>) -> Self {
        self.height = Some(height.into_value());
        self
    }
}

/// Build the view of one region inside the layout that places it.
fn region_view(region: LayoutRegion) -> View {
    let LayoutRegion {
        children,
        kind,
        divider,
        padding,
        padding_bottom,
        scrollable,
        width,
        height,
    } = region;
    kivact::custom::scope(move || {
        let slots = try_use_context::<LayoutSlots>().unwrap_or_default();
        let inset = try_use_context::<LayoutInset>().unwrap_or_default();
        let (inset, inner_x) = region_insets(kind, slots, inset, padding, padding_bottom);
        let content = column(provide(inset, move || children)).with_node(move |node| {
            node.base_style(move || region_content_style(kind, slots, inset))
        });
        let root = node().with_node(move |node| {
            node.base_style(move || region_root_style(kind, divider, padding, width, inner_x))
        });
        let root = match height {
            Some(height) => root.style(move || style().height(height.get())),
            None => root,
        };
        if scrollable.get() {
            root.child(scroll(content).style(style().flex_grow(1).min_height(0).min_width(0)))
        } else {
            root.child(content)
        }
    })
}

fn center_gutter(root: NodeRef, width: f64) -> f64 {
    root.rect()
        .get()
        .map_or(0.0, |rect| ((rect.width - width) / 2.0).max(0.0))
}

fn layout_row_style(slots: LayoutSlots) -> kivact::style::Style {
    let direction = if rtl() {
        FlexDirection::RowReverse
    } else {
        FlexDirection::Row
    };
    let mut base = style()
        .flex_direction(direction)
        .flex_grow(1)
        .min_height(0)
        .min_width(0);
    let Some(ContentWidth { root, width }) = slots.content_width else {
        return base;
    };
    match width.get() {
        Length::Px(width) if !(slots.has_start && slots.has_end) => {
            let gutter = center_gutter(root, width);
            if slots.has_start {
                base = if rtl() {
                    base.padding_right(gutter)
                } else {
                    base.padding_left(gutter)
                };
            } else if slots.has_end {
                base = if rtl() {
                    base.padding_left(gutter)
                } else {
                    base.padding_right(gutter)
                };
            }
        }
        width => {
            base = base
                .width(pct(100))
                .max_width(width)
                .margin_x(kivact::style::auto());
        }
    }
    base
}

fn region_insets(
    kind: Region,
    slots: LayoutSlots,
    inset: LayoutInset,
    padding: Option<Space>,
    padding_bottom: Option<Space>,
) -> (ContainerInset, f64) {
    let fallback = Space::S4.px();
    let (outer_x, outer_y) = (
        inset.own_outer.or(inset.outer_x).unwrap_or(fallback),
        inset.own_outer.or(inset.outer_y).unwrap_or(fallback),
    );
    let (inner_x, inner_y) = (
        inset.inner_x.unwrap_or(fallback),
        inset.inner_y.unwrap_or(fallback),
    );
    let between = |divider| match divider {
        None => outer_y,
        Some(true) => inner_y,
        Some(false) if kind == Region::Content => 0.0,
        Some(false) => inner_y,
    };
    let (start, end, top, bottom) = match kind {
        Region::Header => (outer_x, outer_x, outer_y, inner_y),
        Region::Footer => (outer_x, outer_x, inner_y, outer_y),
        Region::Content => (
            if slots.has_start { inner_x } else { outer_x },
            if slots.has_end { inner_x } else { outer_x },
            between(slots.header_divider),
            between(slots.footer_divider),
        ),
        Region::Start => (
            outer_x,
            inner_x,
            between(slots.header_divider),
            between(slots.footer_divider),
        ),
        Region::End => (
            inner_x,
            outer_x,
            between(slots.header_divider),
            between(slots.footer_divider),
        ),
    };
    let mut inset = padding.map_or(
        ContainerInset {
            start,
            end,
            top,
            bottom,
        },
        |space| ContainerInset::uniform(space.px()),
    );
    if let Some(bottom) = padding_bottom {
        inset.bottom = bottom.px();
    }
    (inset, inner_x)
}

fn region_content_style(
    kind: Region,
    slots: LayoutSlots,
    inset: ContainerInset,
) -> kivact::style::Style {
    let mut base = inset.apply(style().min_width(0));
    if kind == Region::Content
        && !slots.has_start
        && !slots.has_end
        && let Some(ContentWidth { root, width }) = slots.content_width
        && let Length::Px(width) = width.get()
    {
        let gutter = center_gutter(root, width);
        let (left, right) = inset.sides();
        base = base
            .padding_left(left + gutter)
            .padding_right(right + gutter);
    }
    if matches!(kind, Region::Header | Region::Footer)
        && let Some(ContentWidth { width, .. }) = slots.content_width
    {
        base = base
            .width(pct(100))
            .max_width(width.get())
            .margin_x(kivact::style::auto());
    }
    base
}

fn region_root_style(
    kind: Region,
    divider: Option<bool>,
    padding: Option<Space>,
    width: Option<Value<Length>>,
    inner_x: f64,
) -> kivact::style::Style {
    let border = metrics::BORDER_WIDTH;
    let mut base = style()
        .flex_direction(FlexDirection::Column)
        .flex_shrink(0)
        .min_width(0);
    if kind == Region::Content {
        base = base.flex_grow(1).flex_shrink(1).min_height(0).clip(true);
    }
    if matches!(kind, Region::Start | Region::End) {
        base = base.clip(true);
        if let Some(width) = width {
            base = base.width(width.get());
        }
    }
    let divider = divider.unwrap_or_default();
    let right = (kind == Region::Start) != rtl();
    match kind {
        Region::Header if divider => base.border_bottom(border, c::BORDER),
        Region::Footer if divider => base.border_top(border, c::BORDER),
        Region::Start | Region::End if divider && right => base.border_right(border, c::BORDER),
        Region::Start | Region::End if divider => base.border_left(border, c::BORDER),
        Region::Start | Region::End if padding.is_none() && right => base.margin_right(-inner_x),
        Region::Start | Region::End if padding.is_none() => base.margin_left(-inner_x),
        _ => base,
    }
}
