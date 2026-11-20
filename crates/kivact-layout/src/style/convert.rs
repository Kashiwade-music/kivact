//! Role: Convert layout properties into taffy styles, allocating calc lengths.
//!
//! Shared dimensions stay f64 until this adapter constructs Taffy's f32 style.
//! Percentages use the shared 0..100 units; Taffy receives fractions in 0..1.

use taffy::prelude::{
    Dimension as TDim, FlexDirection as TDir, JustifyContent, LengthPercentage,
    LengthPercentageAuto,
};
use taffy::{
    AlignItems, FlexWrap, GridAutoFlow as TFlow, GridTemplateComponent, GridTemplateRepetition,
    MaxTrackSizingFunction, MinTrackSizingFunction, Overflow as TOverflow, Position as TPos,
    RepetitionCount, TrackSizingFunction,
};

use crate::calcs::CalcAlloc;
use crate::style::node::{GridArea, NodeDisplay, NodeStyle, TStyle};
use kivact_types::{
    Align, Breadth, FlexDirection, GridAutoFlow, GridContainer, GridRepeat, GridTrack, Justify,
    LayoutProps, Length, Overflow, Position, TrackSize,
};

/// Converts `shared` lengths, making calc handles.
struct Lengths<'a, 'b> {
    alloc: &'a mut CalcAlloc<'b>,
}

impl Lengths<'_, '_> {
    fn dim(&mut self, d: Length) -> TDim {
        match d {
            Length::Auto => TDim::auto(),
            Length::Px(v) => TDim::length(v as f32),
            Length::Percent(v) => TDim::percent((v / 100.0) as f32),
            Length::Calc { percent, px } => TDim::calc(self.alloc.calc(percent, px)),
            Length::MinContent => TDim::min_content(),
            Length::MaxContent => TDim::max_content(),
            Length::FitContent => TDim::fit_content(),
        }
    }

    fn lp(&mut self, d: Length) -> LengthPercentage {
        match d {
            Length::Percent(v) => LengthPercentage::percent((v / 100.0) as f32),
            Length::Px(v) => LengthPercentage::length(v as f32),
            Length::Auto | Length::MinContent | Length::MaxContent | Length::FitContent => {
                LengthPercentage::length(0.0)
            }
            Length::Calc { percent, px } => LengthPercentage::calc(self.alloc.calc(percent, px)),
        }
    }

    fn lpa(&mut self, d: Length) -> LengthPercentageAuto {
        match d {
            Length::Auto | Length::MinContent | Length::MaxContent | Length::FitContent => {
                LengthPercentageAuto::auto()
            }
            Length::Px(v) => LengthPercentageAuto::length(v as f32),
            Length::Percent(v) => LengthPercentageAuto::percent((v / 100.0) as f32),
            Length::Calc { percent, px } => {
                LengthPercentageAuto::calc(self.alloc.calc(percent, px))
            }
        }
    }
}

/// A border width as taffy takes it (not finite or below zero: 0).
fn border(width: f64) -> LengthPercentage {
    LengthPercentage::length(
        (if width.is_finite() {
            width.max(0.0)
        } else {
            0.0
        }) as f32,
    )
}

fn align(a: Align) -> AlignItems {
    match a {
        Align::Start => AlignItems::FLEX_START,
        Align::Center => AlignItems::CENTER,
        Align::End => AlignItems::FLEX_END,
        Align::Stretch => AlignItems::STRETCH,
        Align::Baseline => AlignItems::BASELINE,
    }
}

fn overflow(o: Overflow) -> TOverflow {
    match o {
        Overflow::Visible => TOverflow::Visible,
        Overflow::Hidden => TOverflow::Hidden,
        Overflow::Scroll => TOverflow::Scroll,
    }
}

fn content(j: Justify) -> JustifyContent {
    match j {
        Justify::Start => JustifyContent::FLEX_START,
        Justify::Center => JustifyContent::CENTER,
        Justify::End => JustifyContent::FLEX_END,
        Justify::SpaceBetween => JustifyContent::SPACE_BETWEEN,
        Justify::SpaceAround => JustifyContent::SPACE_AROUND,
        Justify::SpaceEvenly => JustifyContent::SPACE_EVENLY,
        Justify::Stretch => JustifyContent::STRETCH,
    }
}

/// The taffy style of a node (`hidden` = the `Hidden` prop), its calc
/// lengths made into handles by `alloc`: the compact one, with a grid
/// container's full style boxed in it.
pub(crate) fn taffy_style(l: &LayoutProps, hidden: bool, alloc: &mut CalcAlloc<'_>) -> NodeStyle {
    let mut x = Lengths { alloc };
    let item = l.grid.as_deref();
    let container = item.and_then(|g| g.container.as_ref());
    let mut style = NodeStyle {
        display: display(l, hidden),
        flex_direction: direction(l.direction),
        flex_wrap: if l.wrap {
            FlexWrap::Wrap
        } else {
            FlexWrap::NoWrap
        },
        gap: taffy::Size {
            width: x.lp(l.column_gap),
            height: x.lp(l.row_gap),
        },
        padding: taffy::Rect {
            left: x.lp(l.padding.left),
            right: x.lp(l.padding.right),
            top: x.lp(l.padding.top),
            bottom: x.lp(l.padding.bottom),
        },
        border: taffy::Rect {
            left: border(l.border.left),
            right: border(l.border.right),
            top: border(l.border.top),
            bottom: border(l.border.bottom),
        },
        margin: taffy::Rect {
            left: x.lpa(l.margin.left),
            right: x.lpa(l.margin.right),
            top: x.lpa(l.margin.top),
            bottom: x.lpa(l.margin.bottom),
        },
        size: taffy::Size {
            width: x.dim(l.width),
            height: x.dim(l.height),
        },
        min_size: taffy::Size {
            width: x.lpa(l.min_width),
            height: x.lpa(l.min_height),
        },
        min_width_floor: 0.0,
        max_size: taffy::Size {
            width: x.lpa(l.max_width),
            height: x.lpa(l.max_height),
        },
        flex_grow: l.grow as f32,
        flex_shrink: l.shrink as f32,
        flex_basis: x.dim(l.basis),
        aspect_ratio: l
            .aspect_ratio
            .map(|r| r as f32)
            .filter(|r| r.is_finite() && *r > 0.0),
        // A sticky box is laid out in flow (its insets move it after
        // layout, by the integration); a fixed one is an absolute child of the
        // window's synthetic viewport.
        position: match l.position {
            Position::Relative | Position::Sticky => TPos::Relative,
            Position::Absolute | Position::Fixed => TPos::Absolute,
        },
        inset: x.inset(l),
        overflow: taffy::Point {
            x: overflow(l.overflow_x),
            y: overflow(l.overflow_y),
        },
        align_items: l.align_items.map_or(AlignItems::NORMAL, align),
        align_self: l.align_self.map(align),
        justify_content: l.justify.map_or(JustifyContent::NORMAL, content),
        align_content: l.align_content.map_or(JustifyContent::NORMAL, content),
        grid_area: item.map(|g| {
            Box::new(GridArea {
                column: g.column,
                row: g.row,
                justify_self: g.justify_self.map(align),
            })
        }),
    };
    if let Some(c) = container
        && let NodeDisplay::Grid(_) = style.display
    {
        style.display = NodeDisplay::Grid(Some(Box::new(x.grid(&style, c))));
    }
    style
}

impl Lengths<'_, '_> {
    fn inset(&mut self, l: &LayoutProps) -> taffy::Rect<LengthPercentageAuto> {
        if l.position == Position::Sticky {
            taffy::Rect::auto()
        } else {
            taffy::Rect {
                left: self.lpa(l.inset.left),
                right: self.lpa(l.inset.right),
                top: self.lpa(l.inset.top),
                bottom: self.lpa(l.inset.bottom),
            }
        }
    }
    /// The full style of the grid container `c` whose box properties are
    /// `node`'s.
    fn grid(&mut self, node: &NodeStyle, c: &GridContainer) -> TStyle {
        let mut style = TStyle {
            display: node.display.taffy(),
            position: node.position,
            overflow: node.overflow,
            inset: node.inset,
            size: node.size,
            min_size: node.min_size,
            max_size: node.max_size,
            aspect_ratio: node.aspect_ratio,
            margin: node.margin,
            padding: node.padding,
            border: node.border,
            gap: node.gap,
            align_items: node.align_items,
            align_content: node.align_content,
            justify_content: node.justify_content,
            scrollbar_width: 0.0,
            ..TStyle::DEFAULT
        };
        style.grid_template_columns = c.columns.iter().map(|t| self.template(t)).collect();
        style.grid_template_rows = c.rows.iter().map(|t| self.template(t)).collect();
        style.grid_auto_columns = c.auto_columns.iter().map(|t| self.track(*t)).collect();
        style.grid_auto_rows = c.auto_rows.iter().map(|t| self.track(*t)).collect();
        style.grid_auto_flow = match c.flow {
            GridAutoFlow::Row => TFlow::Row,
            GridAutoFlow::Column => TFlow::Column,
            GridAutoFlow::RowDense => TFlow::RowDense,
            GridAutoFlow::ColumnDense => TFlow::ColumnDense,
        };
        style.justify_items = c.justify_items.map_or(AlignItems::NORMAL, align);
        style
    }

    fn template(&mut self, t: &GridTrack) -> GridTemplateComponent<String> {
        match t {
            GridTrack::Single(s) => GridTemplateComponent::Single(self.track(*s)),
            GridTrack::Repeat(count, tracks) => {
                GridTemplateComponent::Repeat(GridTemplateRepetition {
                    count: match count {
                        GridRepeat::Count(n) => RepetitionCount::Count((*n).max(1)),
                        GridRepeat::AutoFill => RepetitionCount::AutoFill,
                        GridRepeat::AutoFit => RepetitionCount::AutoFit,
                    },
                    tracks: tracks.iter().map(|s| self.track(*s)).collect(),
                    line_names: Vec::new(),
                })
            }
        }
    }

    fn track(&mut self, t: TrackSize) -> TrackSizingFunction {
        taffy::MinMax {
            min: self.min_track(t.min),
            max: self.max_track(t.max),
        }
    }

    fn min_track(&mut self, b: Breadth) -> MinTrackSizingFunction {
        type M = MinTrackSizingFunction;
        match b {
            Breadth::Size(Length::Px(v)) => M::length(v as f32),
            Breadth::Size(Length::Percent(v)) => M::percent((v / 100.0) as f32),
            Breadth::Size(Length::Calc { percent, px }) => M::calc(self.alloc.calc(percent, px)),
            Breadth::Size(Length::MinContent) => M::min_content(),
            Breadth::Size(Length::MaxContent) => M::max_content(),
            Breadth::Size(Length::Auto | Length::FitContent)
            | Breadth::Fr(_)
            | Breadth::FitContent(_) => M::auto(),
        }
    }

    fn max_track(&mut self, b: Breadth) -> MaxTrackSizingFunction {
        type M = MaxTrackSizingFunction;
        match b {
            Breadth::Size(Length::Px(v)) => M::length(v as f32),
            Breadth::Size(Length::Percent(v)) => M::percent((v / 100.0) as f32),
            Breadth::Size(Length::Calc { percent, px }) => M::calc(self.alloc.calc(percent, px)),
            Breadth::Size(Length::Auto) => M::auto(),
            Breadth::Size(Length::MinContent) => M::min_content(),
            Breadth::Size(Length::MaxContent) => M::max_content(),
            Breadth::Size(Length::FitContent) => M::fit_content_percent(1.0),
            Breadth::Fr(f) => M::fr(f.max(0.0) as f32),
            Breadth::FitContent(Length::Px(v)) => M::fit_content_px(v as f32),
            Breadth::FitContent(Length::Percent(v)) => M::fit_content_percent((v / 100.0) as f32),
            Breadth::FitContent(_) => M::fit_content_percent(1.0),
        }
    }
}

fn display(l: &LayoutProps, hidden: bool) -> NodeDisplay {
    if hidden {
        NodeDisplay::None
    } else if l.display == kivact_types::Display::Grid {
        NodeDisplay::Grid(None)
    } else {
        NodeDisplay::Flex
    }
}

fn direction(direction: FlexDirection) -> TDir {
    match direction {
        FlexDirection::Row => TDir::Row,
        FlexDirection::Column => TDir::Column,
        FlexDirection::RowReverse => TDir::RowReverse,
        FlexDirection::ColumnReverse => TDir::ColumnReverse,
    }
}
