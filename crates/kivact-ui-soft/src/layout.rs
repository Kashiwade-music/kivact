//! Role: Define responsive application layout insets.
use crate::stack::ContainerInset;
use kivact::state::IntoValue;
use kivact::state::Value;

/// Insets inherited by Layout regions from a parent surface.
#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutInset {
    pub outer_x: Option<f64>,
    pub outer_y: Option<f64>,
    pub inner_x: Option<f64>,
    pub inner_y: Option<f64>,
    pub own_outer_x: Option<f64>,
    pub own_outer_y: Option<f64>,
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

use crate::resizable::{ResizableState, ResizeHandle};
use crate::stack::{Space, rtl};
use crate::tokens;
use kivact::view::Node;
use kivact::view::NodeBuilder;
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
pub enum LayoutHeight {
    #[default]
    Fill,
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
    start_resize: Option<ResizeHandle>,
    end_resize: Option<ResizeHandle>,
    height: Value<LayoutHeight>,
    padding: Option<Space>,
    content_width: Option<Value<Length>>,
    default_dividers: Value<bool>,
}

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
    pub fn header(mut self, slot: LayoutRegion) -> Self {
        self.header = Some(slot);
        self
    }
    pub fn footer(mut self, slot: LayoutRegion) -> Self {
        self.footer = Some(slot);
        self
    }
    pub fn start(mut self, slot: LayoutRegion) -> Self {
        self.start = Some(slot);
        self
    }
    pub fn end(mut self, slot: LayoutRegion) -> Self {
        self.end = Some(slot);
        self
    }
    pub fn content(mut self, slot: LayoutRegion) -> Self {
        self.content = Some(slot);
        self
    }
    pub fn height(mut self, height: impl IntoValue<LayoutHeight>) -> Self {
        self.height = height.into_value();
        self
    }
    pub fn padding(mut self, padding: Space) -> Self {
        self.padding = Some(padding);
        self
    }
    pub fn content_width(mut self, width: impl IntoValue<Length>) -> Self {
        self.content_width = Some(width.into_value());
        self
    }
    pub fn start_resize_handle(mut self, handle: ResizeHandle) -> Self {
        self.start_resize = Some(handle);
        self
    }
    pub fn end_resize_handle(mut self, handle: ResizeHandle) -> Self {
        self.end_resize = Some(handle.reversed(true));
        self
    }
    pub fn default_dividers(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.default_dividers = enabled.into_value();
        self
    }
}

impl NodeBuilder for Layout {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

#[derive(Clone, Copy, Default)]
struct LayoutSlots {
    root_ref: Option<NodeRef>,
    header_divider: Option<bool>,
    footer_divider: Option<bool>,
    has_start: bool,
    has_end: bool,
    content_width: Option<Value<Length>>,
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
        provide((), move || {
            let parent = try_use_context::<ContainerInset>()
                .map(|value| value.get())
                .unwrap_or_default();
            let mut insets = try_use_context::<LayoutInset>()
                .map(|value| value.get())
                .unwrap_or_default();
            insets.own_outer_x = padding.map(Space::px);
            insets.own_outer_y = insets.own_outer_x;
            let mut header = header.map(|slot| slot);
            let mut footer = footer.map(|slot| slot);
            for region in [&mut header, &mut footer].into_iter().flatten() {
                region.divider.get_or_insert(default_dividers.get());
            }
            let root_ref = node_ref();
            let slots = LayoutSlots {
                root_ref: Some(root_ref),
                header_divider: header.as_ref().and_then(|region| region.divider),
                footer_divider: footer.as_ref().and_then(|region| region.divider),
                has_start: start.is_some(),
                has_end: end.is_some(),
                content_width,
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
                .base_style(base)
                .child(provide(insets, move || {
                    provide(ContainerInset::default(), move || {
                        provide(slots, move || {
                            (
                                header,
                                node()
                                    .base_style(move || {
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
                                        match content_width.map(|value| value.get()) {
                                            Some(Length::Px(width))
                                                if !(slots.has_start && slots.has_end) =>
                                            {
                                                let gutter = center_gutter(root_ref, width);
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
                                            Some(width) => {
                                                base = base
                                                    .width(pct(100))
                                                    .max_width(width)
                                                    .margin_x(kivact::style::auto());
                                            }
                                            None => {}
                                        }
                                        base
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

pub struct LayoutRegion {
    root: Node,
    children: View,
    kind: Region,
    divider: Option<bool>,
    padding: Option<Space>,
    padding_bottom: Option<Space>,
    scrollable: Value<bool>,
    resizable: Option<ResizableState>,
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
        resizable: None,
        width: None,
    }
}

pub fn layout_header(children: impl IntoView) -> LayoutRegion {
    region(Region::Header, children)
}
pub fn layout_content(children: impl IntoView) -> LayoutRegion {
    region(Region::Content, children)
}
pub fn layout_footer(children: impl IntoView) -> LayoutRegion {
    region(Region::Footer, children)
}
pub fn layout_panel(children: impl IntoView) -> LayoutRegion {
    region(Region::Start, children)
}

impl LayoutRegion {
    fn in_region(mut self, kind: Region) -> Self {
        self.kind = kind;
        self
    }
    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = Some(divider);
        self
    }
    pub fn padding(mut self, padding: Space) -> Self {
        self.padding = Some(padding);
        self
    }
    pub fn padding_bottom(mut self, padding: Space) -> Self {
        self.padding_bottom = Some(padding);
        self
    }
    pub fn resizable(mut self, state: ResizableState) -> Self {
        self.resizable = Some(state);
        self
    }
    pub fn scrollable(mut self, scrollable: impl IntoValue<bool>) -> Self {
        self.scrollable = scrollable.into_value();
        self
    }
    pub fn width(mut self, width: impl IntoValue<Length>) -> Self {
        self.width = Some(width.into_value());
        self
    }
    pub fn height(mut self, height: impl IntoValue<Length>) -> Self {
        let height = height.into_value();
        self.root = self.root.style(move || style().height(height.get()));
        self
    }
}

impl NodeBuilder for LayoutRegion {
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
            resizable,
            width,
        } = self;
        provide((), move || {
            let slots = try_use_context::<LayoutSlots>()
                .map(|value| value.get())
                .unwrap_or_default();
            let inset = try_use_context::<LayoutInset>()
                .map(|value| value.get())
                .unwrap_or_default();
            let fallback = tokens().spacing.s4;
            let (outer_x, outer_y) = (
                inset.own_outer_x.or(inset.outer_x).unwrap_or(fallback),
                inset.own_outer_y.or(inset.outer_y).unwrap_or(fallback),
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
            let content = column(provide(inset, move || children)).base_style(move || {
                let mut base = inset.apply(style().min_width(0));
                if kind == Region::Content
                    && !slots.has_start
                    && !slots.has_end
                    && let (Some(root), Some(Length::Px(width))) =
                        (slots.root_ref, slots.content_width.map(|value| value.get()))
                {
                    let gutter = center_gutter(root, width);
                    let (left, right) = inset.sides();
                    base = base
                        .padding_left(left + gutter)
                        .padding_right(right + gutter);
                }
                if matches!(kind, Region::Header | Region::Footer)
                    && let Some(width) = slots.content_width
                {
                    base = base
                        .width(pct(100))
                        .max_width(width.get())
                        .margin_x(kivact::style::auto());
                }
                base
            });
            let root = root.base_style(move || {
                let t = tokens();
                let mut base = style().direction(Direction::Column).shrink(0).min_width(0);
                if kind == Region::Content {
                    base = base.grow(1).shrink(1).min_height(0).clip(true);
                }
                if matches!(kind, Region::Start | Region::End) {
                    base = base.clip(true);
                    if let Some(state) = &resizable {
                        base = base.width(state.size());
                    } else if let Some(width) = width {
                        base = base.width(width.get());
                    }
                }
                let divider = divider.unwrap_or_default();
                let right = (kind == Region::Start) != rtl();
                match kind {
                    Region::Header if divider => {
                        base.border_bottom(t.sizes.border_width, t.colors.border)
                    }
                    Region::Footer if divider => {
                        base.border_top(t.sizes.border_width, t.colors.border)
                    }
                    Region::Start | Region::End if divider && right => {
                        base.border_right(t.sizes.border_width, t.colors.border)
                    }
                    Region::Start | Region::End if divider => {
                        base.border_left(t.sizes.border_width, t.colors.border)
                    }
                    Region::Start | Region::End if padding.is_none() && right => {
                        base.margin_right(-inner_x)
                    }
                    Region::Start | Region::End if padding.is_none() => base.margin_left(-inner_x),
                    _ => base,
                }
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
