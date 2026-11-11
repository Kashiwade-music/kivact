//! Role: Define responsive application layout insets.
use crate::stack::ContainerInset;
use kivact::state::IntoValue;
use kivact::state::Value;

/// Insets inherited by Layout regions from a parent surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutInset {
    // xtask-allow(state-fields): option: inline outer padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback.
    /// Inherited inline outer padding in logical pixels; None uses the region fallback.
    pub outer_x: Option<f64>,
    // xtask-allow(state-fields): option: block outer padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback.
    /// Inherited block outer padding in logical pixels; None uses the region fallback.
    pub outer_y: Option<f64>,
    // xtask-allow(state-fields): option: inline gap padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback.
    /// Padding between inline regions in logical pixels; None uses the region fallback.
    pub inner_x: Option<f64>,
    // xtask-allow(state-fields): option: block gap padding announced by the enclosing AppShell, Card or Section (LayoutInset::from_padding); None uses the spacing.s4 fallback.
    /// Padding between block regions in logical pixels; None uses the region fallback.
    pub inner_y: Option<f64>,
    // xtask-allow(state-fields): option: this Layout's padding from Layout::padding, written only in Layout::into_view; absent uses the inherited outer_x and outer_y.
    /// This layout's own outer padding (`Layout::padding`) on both axes in
    /// logical pixels; when set it replaces `outer_x` and `outer_y`.
    pub own_outer: Option<f64>,
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

use crate::stack::{Space, rtl};

use kivact::view::Element;
use kivact::view::Node;
use kivact::view::column;
use kivact::view::node;
use kivact::view::scroll;

use kivact::state::provide;
use kivact::state::try_use_context;
use kivact::style::Direction;
use kivact::style::Length;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::NodeRef;
use kivact::view::View;
use kivact::view::node_ref;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Whether a page layout fills its parent or follows intrinsic content height.
pub enum LayoutHeight {
    #[default]
    /// Fill the available parent height; the default.
    Fill,
    /// Follow content height while retaining at least the parent height.
    Auto,
}

/// Page regions arranged around a central content area.
pub struct Layout {
    root: Node,
    header: Option<LayoutRegion>,
    footer: Option<LayoutRegion>,
    start: Option<LayoutRegion>,
    end: Option<LayoutRegion>,
    content: Option<LayoutRegion>,
    start_resize: Option<View>,
    end_resize: Option<View>,
    height: Value<LayoutHeight>,
    padding: Option<Space>,
    content_width: Option<Value<Length>>,
    default_dividers: Value<bool>,
}

/// Create an empty fill-height page layout with no default dividers.
pub fn layout() -> Layout {
    Layout {
        root: node(),
        header: None,
        footer: None,
        start: None,
        end: None,
        content: None,
        start_resize: None,
        end_resize: None,
        height: (LayoutHeight::Fill).into_value(),
        padding: None,
        content_width: None,
        default_dividers: (false).into_value(),
    }
}

impl Layout {
    /// Place a region above the central row; usually built with `layout_header`.
    pub fn header(mut self, slot: LayoutRegion) -> Self {
        self.header = Some(slot);
        self
    }
    /// Place a region below the central row; usually built with `layout_footer`.
    pub fn footer(mut self, slot: LayoutRegion) -> Self {
        self.footer = Some(slot);
        self
    }
    /// Place a panel at the logical leading side, following reading direction.
    pub fn start(mut self, slot: LayoutRegion) -> Self {
        self.start = Some(slot);
        self
    }
    /// Place a panel at the logical trailing side, following reading direction.
    pub fn end(mut self, slot: LayoutRegion) -> Self {
        self.end = Some(slot);
        self
    }
    /// Place the central region, which grows between optional side panels.
    pub fn content(mut self, slot: LayoutRegion) -> Self {
        self.content = Some(slot);
        self
    }
    /// Choose fill or intrinsic height at view construction; defaults to fill.
    pub fn height(mut self, height: impl IntoValue<LayoutHeight>) -> Self {
        self.height = height.into_value();
        self
    }
    /// Override the layout’s outer padding on both axes with a theme spacing step.
    pub fn padding(mut self, padding: Space) -> Self {
        self.padding = Some(padding);
        self
    }
    /// Set a reactive content-width limit; pixel lengths also center content without side panels.
    pub fn content_width(mut self, width: impl IntoValue<Length>) -> Self {
        self.content_width = Some(width.into_value());
        self
    }
    /// Place a caller-owned resize handle after the leading panel; no resize behavior is added.
    pub fn start_resize_handle(mut self, handle: impl IntoView) -> Self {
        self.start_resize = Some(handle.into_view());
        self
    }
    /// Place a caller-owned resize handle before the trailing panel; no resize behavior is added.
    pub fn end_resize_handle(mut self, handle: impl IntoView) -> Self {
        self.end_resize = Some(handle.into_view());
        self
    }
    /// Choose default header/footer dividers at construction unless a region overrides them.
    pub fn default_dividers(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.default_dividers = enabled.into_value();
        self
    }
}

impl Element for Layout {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// A caller width limit with the layout root used to centre pixel widths.
#[derive(Clone, Copy)]
struct ContentWidth {
    root: NodeRef,
    width: Value<Length>,
}

#[derive(Clone, Copy, Default)]
struct LayoutSlots {
    // xtask-allow(state-fields): option: header divider resolved at into_view from LayoutRegion::divider or Layout::default_dividers; None means the layout has no header.
    header_divider: Option<bool>,
    // xtask-allow(state-fields): option: footer divider resolved at into_view from LayoutRegion::divider or Layout::default_dividers; None means the layout has no footer.
    footer_divider: Option<bool>,
    // xtask-allow(state-fields): option: whether the caller supplied Layout::start, captured once in into_view; false means no leading panel.
    has_start: bool,
    // xtask-allow(state-fields): option: whether the caller supplied Layout::end, captured once in into_view; false means no trailing panel.
    has_end: bool,
    // xtask-allow(state-fields): option: caller limit from Layout::content_width paired with the layout root ref used to centre pixel widths; absent means no width limit.
    content_width: Option<ContentWidth>,
}

impl IntoView for Layout {
    fn into_view(self) -> View {
        let Self {
            root,
            header,
            footer,
            start,
            end,
            content,
            start_resize,
            end_resize,
            height,
            padding,
            content_width,
            default_dividers,
        } = self;
        kivact::view::scope(move || {
            let parent = try_use_context::<ContainerInset>()
                .map(|value| value.get())
                .unwrap_or_default();
            let mut insets = try_use_context::<LayoutInset>()
                .map(|value| value.get())
                .unwrap_or_default();
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
                .direction(Direction::Column)
                .min_width(0)
                .margin_left(-left)
                .margin_right(-right)
                .margin_top(-parent.top)
                .margin_bottom(-parent.bottom);
            let base = match height.get() {
                LayoutHeight::Fill => base
                    .grow(1)
                    .min_height(0)
                    .height(pct(100) + parent.top + parent.bottom),
                LayoutHeight::Auto => base.min_height(pct(100)),
            };
            root.node_ref(root_ref)
                .with_node(move |node| node.base_style(base))
                .child(provide(insets, move || {
                    provide(ContainerInset::default(), move || {
                        provide(slots, move || {
                            (
                                header,
                                node()
                                    .with_node(move |node| {
                                        node.base_style(move || layout_row_style(slots))
                                    })
                                    .child((start, start_resize, content, end_resize, end)),
                                footer,
                            )
                        })
                    })
                }))
        })
    }
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
pub struct LayoutRegion {
    root: Node,
    children: View,
    kind: Region,
    divider: Option<bool>,
    padding: Option<Space>,
    padding_bottom: Option<Space>,
    scrollable: Value<bool>,
    width: Option<Value<Length>>,
}

fn region(kind: Region, children: impl IntoView) -> LayoutRegion {
    LayoutRegion {
        root: node(),
        children: children.into_view(),
        kind,
        divider: None,
        padding: None,
        padding_bottom: None,
        scrollable: (!matches!(kind, Region::Header | Region::Footer)).into_value(),
        width: None,
    }
}

/// Create a non-scrolling header region.
pub fn layout_header(children: impl IntoView) -> LayoutRegion {
    region(Region::Header, children)
}
/// Create a scrolling central content region.
pub fn layout_content(children: impl IntoView) -> LayoutRegion {
    region(Region::Content, children)
}
/// Create a non-scrolling footer region.
pub fn layout_footer(children: impl IntoView) -> LayoutRegion {
    region(Region::Footer, children)
}
/// Create a scrolling side region; assigning it to start/end determines its logical side.
pub fn layout_panel(children: impl IntoView) -> LayoutRegion {
    region(Region::Start, children)
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
    pub fn scrollable(mut self, scrollable: impl IntoValue<bool>) -> Self {
        self.scrollable = scrollable.into_value();
        self
    }
    /// Set constant or reactive width for a leading or trailing panel.
    pub fn width(mut self, width: impl IntoValue<Length>) -> Self {
        self.width = Some(width.into_value());
        self
    }
    /// Set a constant or reactive height using a logical length.
    pub fn height(mut self, height: impl IntoValue<Length>) -> Self {
        let height = height.into_value();
        self.root = self.root.style(move || style().height(height.get()));
        self
    }
}

impl Element for LayoutRegion {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for LayoutRegion {
    fn into_view(self) -> View {
        let Self {
            root,
            children,
            kind,
            divider,
            padding,
            padding_bottom,
            scrollable,
            width,
        } = self;
        kivact::view::scope(move || {
            let slots = try_use_context::<LayoutSlots>()
                .map(|value| value.get())
                .unwrap_or_default();
            let inset = try_use_context::<LayoutInset>()
                .map(|value| value.get())
                .unwrap_or_default();
            let (inset, inner_x) = region_insets(kind, slots, inset, padding, padding_bottom);
            let content = column(provide(inset, move || children)).with_node(move |node| {
                node.base_style(move || region_content_style(kind, slots, inset))
            });
            let root = root.with_node(move |node| {
                node.base_style(move || region_root_style(kind, divider, padding, width, inner_x))
            });
            if scrollable.get() {
                root.child(scroll(content).style(style().grow(1).min_height(0).min_width(0)))
            } else {
                root.child(content)
            }
        })
    }
}

fn center_gutter(root: NodeRef, width: f64) -> f64 {
    root.rect()
        .get()
        .map_or(0.0, |rect| ((rect.width - width) / 2.0).max(0.0))
}

kivact::state::constant_value!(LayoutHeight);

fn layout_row_style(slots: LayoutSlots) -> kivact::style::Style {
    let direction = if rtl() {
        Direction::RowReverse
    } else {
        Direction::Row
    };
    let mut base = style()
        .direction(direction)
        .grow(1)
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
    let fallback = kivact::style::use_tokens::<crate::Tokens>().spacing.s4;
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
    let t = kivact::style::use_tokens::<crate::Tokens>();
    let mut base = style().direction(Direction::Column).shrink(0).min_width(0);
    if kind == Region::Content {
        base = base.grow(1).shrink(1).min_height(0).clip(true);
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
        Region::Header if divider => base.border_bottom(t.sizes.border_width, t.colors.border),
        Region::Footer if divider => base.border_top(t.sizes.border_width, t.colors.border),
        Region::Start | Region::End if divider && right => {
            base.border_right(t.sizes.border_width, t.colors.border)
        }
        Region::Start | Region::End if divider => {
            base.border_left(t.sizes.border_width, t.colors.border)
        }
        Region::Start | Region::End if padding.is_none() && right => base.margin_right(-inner_x),
        Region::Start | Region::End if padding.is_none() => base.margin_left(-inner_x),
        _ => base,
    }
}
