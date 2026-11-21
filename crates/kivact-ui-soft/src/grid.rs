//! Role: Provide Neutral grid tracks and item spans.
use crate::space::Space;
use kivact::Prop;
use kivact::component;
use kivact::state::Value;
use kivact::style::Align;
use kivact::style::Track;
use kivact::style::style;
use kivact::view::Element;
use kivact::view::View;
use kivact::view::grid as native_grid;
use kivact::view::node_ref;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Treatment of empty tracks in a responsive grid.
pub enum GridRepeat {
    #[default]
    /// Keep empty tracks in the responsive column pattern; the default.
    Fill,
    /// Collapse empty tracks so occupied tracks can expand.
    Fit,
}

#[derive(Clone, Copy, Debug, PartialEq, Prop)]
/// Equal-width columns, counted explicitly or fitted to available width.
pub enum GridColumns {
    /// Use this many equal fractional tracks, with a minimum of one.
    Fixed(u16),
    /// Fit tracks using a minimum width and optional count limit.
    Responsive {
        /// Minimum track width in logical pixels; negative values become zero.
        min_width: f64,
        /// Optional maximum column count; zero is ignored.
        max: Option<u16>,
        /// Whether empty responsive tracks retain space or collapse.
        repeat: GridRepeat,
    },
}

/// Grid container with theme spacing and reactive column configuration.
///
/// `gap` sets both axes; `row_gap`, `column_gap` and `row_height` override it.
/// `align` and `justify` set the items' alignment on each axis.
#[component]
pub fn layout_grid(
    children: View,
    #[prop(default = GridColumns::Fixed(1))] columns: Value<GridColumns>,
    #[prop(default = Space::S0)] gap: Value<Space>,
    row_gap: Option<Value<Space>>,
    column_gap: Option<Value<Space>>,
    row_height: Option<Value<f64>>,
    align: Option<Align>,
    justify: Option<Align>,
) -> impl Element {
    let reference = node_ref();
    let mut root = native_grid(children).node_ref(reference);
    if let Some(align) = align {
        root = root.style(style().align_items(align));
    }
    if let Some(justify) = justify {
        root = root.style(style().justify_items(justify));
    }
    root.with_node(move |node| {
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
}
