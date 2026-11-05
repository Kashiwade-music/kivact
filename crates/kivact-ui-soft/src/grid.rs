//! Role: Provide Neutral grid tracks and item spans.
use crate::stack::Space;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::grid as native_grid;

use kivact::state::provide;
use kivact::style::Align;
use kivact::style::GridPlacement;
use kivact::style::Track;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::View;
use kivact::view::node_ref;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridRepeat {
    #[default]
    Fill,
    Fit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridColumns {
    Fixed(u16),
    Responsive {
        min_width: f64,
        max: Option<u16>,
        repeat: GridRepeat,
    },
}

pub struct LayoutGrid {
    root: Node,
    columns: Value<GridColumns>,
    gap: Value<Space>,
    row_gap: Option<Value<Space>>,
    column_gap: Option<Value<Space>>,
    row_height: Option<Value<f64>>,
}

pub fn layout_grid(children: impl IntoView) -> LayoutGrid {
    LayoutGrid {
        root: native_grid(children),
        columns: (GridColumns::Fixed(1)).into_value(),
        gap: (Space::S0).into_value(),
        row_gap: None,
        column_gap: None,
        row_height: None,
    }
}

impl LayoutGrid {
    pub fn columns(mut self, columns: impl IntoValue<GridColumns>) -> Self {
        self.columns = columns.into_value();
        self
    }
    pub fn gap(mut self, gap: impl IntoValue<Space>) -> Self {
        self.gap = gap.into_value();
        self
    }
    pub fn row_gap(mut self, gap: impl IntoValue<Space>) -> Self {
        self.row_gap = Some(gap.into_value());
        self
    }
    pub fn column_gap(mut self, gap: impl IntoValue<Space>) -> Self {
        self.column_gap = Some(gap.into_value());
        self
    }
    pub fn row_height(mut self, px: impl IntoValue<f64>) -> Self {
        self.row_height = Some(px.into_value());
        self
    }
    pub fn align(mut self, align: Align) -> Self {
        self.root = self.root.style(style().align_items(align));
        self
    }
    pub fn justify(mut self, align: Align) -> Self {
        self.root = self.root.style(style().justify_items(align));
        self
    }
}

impl NodeBuilder for LayoutGrid {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for LayoutGrid {
    fn into_view(self) -> View {
        let Self {
            root,
            columns,
            gap,
            row_gap,
            column_gap,
            row_height,
        } = self;
        provide((), move || {
            let reference = node_ref();
            root.node_ref(reference).with_node(move |node| {
                node.base_style(move || {
                    let column_gap = column_gap.map_or_else(|| gap.get(), |gap| gap.get()).px();
                    let columns = match columns.get() {
                        GridColumns::Fixed(count) => Track::repeat(count.max(1), Track::fr(1)),
                        GridColumns::Responsive {
                            mut min_width,
                            max,
                            repeat,
                        } => {
                            if let Some(max) = max.filter(|max| *max > 0)
                                && let Some(rect) = reference.rect().get()
                            {
                                let max = f64::from(max);
                                let capped = (rect.width - (max - 1.0) * column_gap) / max;
                                min_width = rect.width.min(min_width.max(capped));
                            }
                            let track = Track::minmax(min_width.max(0.0), Track::fr(1));
                            match repeat {
                                GridRepeat::Fill => Track::repeat_auto_fill(track),
                                GridRepeat::Fit => Track::repeat_auto_fit(track),
                            }
                        }
                    };
                    let mut base = style()
                        .grid_template_columns(columns)
                        .row_gap(row_gap.map_or_else(|| gap.get(), |gap| gap.get()).px())
                        .column_gap(column_gap);
                    if let Some(px) = row_height {
                        base = base.grid_auto_rows(px.get());
                    }
                    base
                })
            })
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridSpan {
    Cols(u16),
    Full,
}

pub struct GridSpanItem {
    root: Node,
}

pub fn grid_span(children: impl IntoView) -> GridSpanItem {
    GridSpanItem {
        root: native_grid(children).with_node(move |node| {
            node.base_style(
                style()
                    .min_width(0)
                    .height(pct(100))
                    .grid_template_columns(Track::minmax(0, Track::fr(1))),
            )
        }),
    }
}

impl GridSpanItem {
    pub fn columns(mut self, span: GridSpan) -> Self {
        let span = match span {
            GridSpan::Cols(n) => GridPlacement::span(n.max(1)),
            GridSpan::Full => GridPlacement::from((1, -1)),
        };
        self.root = self.root.style(style().grid_column(span));
        self
    }

    pub fn rows(mut self, n: u16) -> Self {
        self.root = self
            .root
            .style(style().grid_row(GridPlacement::span(n.max(1))));
        self
    }
}

impl NodeBuilder for GridSpanItem {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for GridSpanItem {
    fn into_view(self) -> View {
        self.root.into_view()
    }
}

kivact::state::constant_value!(GridColumns);
