//! Role: One data table column: title, cell text and view, order, match, width.
use crate::menu::Menu;
use kivact_core::Value;
use kivact_core::control::Node;
use kivact_core::view::{IntoText, IntoValue, IntoView, View};
use kivact_interaction::ColumnDef;
use std::cmp::Ordering;
use std::hash::Hash;
use std::rc::Rc;

/// Names a row in text.
type RowText<R> = Rc<dyn Fn(&R) -> String>;
/// The view of one row's cell.
type CellView<R> = Rc<dyn Fn(&R) -> View>;
type HeaderMenu = Rc<dyn Fn(Node) -> View>;

/// One column of a [`data_table`](crate::data_table): what its cells show, and
/// how the rows sort and match a filter by it.
///
/// A builder of the interaction layer's [`ColumnDef`]: the title heads the
/// column; the text is what the cells show and what names them for assistive
/// technology. A column neither sorts nor takes part in the filter until
/// [`compare`](Self::compare) (or [`sortable`](Self::sortable)) and
/// [`filter`](Self::filter) say how, and it is sortable exactly when it has
/// a comparison. The column is plain data the app builds; the table gives its
/// [`definition`](Self::definition) to
/// [`use_table_model`](kivact_interaction::use_table_model), which sorts and
/// filters the rows.
///
/// ```no_run
/// use kivact::ui::DataColumn;
///
/// #[derive(Clone, PartialEq)]
/// struct File { name: String, size: u64 }
///
/// let columns = vec![
///     DataColumn::new("Name", |f: &File| f.name.clone())
///         .sortable()
///         .filter(|f, query| f.name.to_lowercase().contains(&query.to_lowercase())),
///     DataColumn::new("Size", |f: &File| f.size.to_string())
///         .compare(|a, b| a.size.cmp(&b.size))
///         .numeric(true)
///         .width(96.0),
/// ];
/// # let _ = columns;
/// ```
pub struct DataColumn<R> {
    def: ColumnDef<R, ()>,
    text: RowText<R>,
    cell: Option<CellView<R>>,
    header_menu: Option<HeaderMenu>,
    numeric: bool,
    resizable: bool,
    flex: Option<f32>,
    min_width: Option<f32>,
}

impl<R> Clone for DataColumn<R> {
    fn clone(&self) -> Self {
        Self {
            def: self.def.clone(),
            text: self.text.clone(),
            cell: self.cell.clone(),
            header_menu: self.header_menu.clone(),
            numeric: self.numeric,
            resizable: self.resizable,
            flex: self.flex,
            min_width: self.min_width,
        }
    }
}

impl<R: 'static> DataColumn<R> {
    /// A column titled `title` whose cells show `text` of their row.
    pub fn new(title: impl IntoText, text: impl Fn(&R) -> String + 'static) -> Self {
        Self {
            def: ColumnDef::new((), title),
            text: Rc::new(text),
            cell: None,
            header_menu: None,
            numeric: false,
            resizable: true,
            flex: None,
            min_width: None,
        }
    }

    /// Show the view `cell` builds from the row in the column's cells instead
    /// of its text; the text still names the cell.
    pub fn cell<V: IntoView>(mut self, cell: impl Fn(&R) -> V + 'static) -> Self {
        self.cell = Some(Rc::new(move |row| cell(row).into_view()));
        self
    }

    /// Open a menu from this column header's secondary click or context-menu key.
    pub fn header_context_menu<K: Clone + Eq + Hash + 'static>(
        mut self,
        menu: impl Fn() -> Menu<K> + 'static,
    ) -> Self {
        self.header_menu = Some(Rc::new(move |header| menu().attach_context(header)));
        self
    }

    pub(crate) fn render_header(&self, header: Node) -> View {
        match &self.header_menu {
            Some(menu) => menu(header),
            None => header.into_view(),
        }
    }

    /// Let the header sort the rows with `compare`, ascending; activating it
    /// again sorts descending. The sort is stable.
    pub fn compare(mut self, compare: impl Fn(&R, &R) -> Ordering + 'static) -> Self {
        self.def = self.def.compare(compare);
        self
    }

    /// Let the header sort the rows by the column's text: numerically when
    /// both texts are numbers, otherwise ignoring case. A shortcut for a
    /// [`compare`](Self::compare) of the text.
    pub fn sortable(mut self) -> Self {
        let text = self.text.clone();
        self.def = self
            .def
            .compare(move |a, b| compare_text(&text(a), &text(b)));
        self
    }

    /// Keep the rows whose `matches(row, query)` is true while the table's
    /// filter has text; a row stays when any column matches. `query` is the
    /// filter text trimmed of surrounding white space.
    pub fn filter(mut self, matches: impl Fn(&R, &str) -> bool + 'static) -> Self {
        self.def = self.def.filter(matches);
        self
    }

    /// Show the column while true (a constant or a reactive value). A hidden
    /// column is left out of the table and of filtering.
    pub fn visible(mut self, visible: impl IntoValue<bool>) -> Self {
        self.def = self.def.visible(visible);
        self
    }

    /// Align the header and the cells to the end, as numbers and amounts
    /// are.
    pub fn numeric(mut self, numeric: bool) -> Self {
        self.numeric = numeric;
        self
    }

    /// Give the column a fixed `width` in logical pixels (the unit a table's
    /// column resizing works in); a width that is not positive leaves the
    /// width to the table.
    pub fn width(mut self, width: f32) -> Self {
        self.flex = None;
        self.def = self.def.width((width > 0.0).then_some(f64::from(width)));
        self
    }

    /// Share available space by this positive weight. Bound resize widths
    /// take precedence. Invalid weights restore automatic sizing.
    pub fn flex(mut self, weight: f32) -> Self {
        self.flex = (weight.is_finite() && weight > 0.0).then_some(weight);
        self.def = self.def.width(None);
        self
    }

    /// Set the lower bound in logical pixels, including during resizing.
    /// Flexible columns default to 120 pixels when no minimum is supplied.
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = (width.is_finite() && width >= 0.0).then_some(width);
        self
    }

    pub(crate) fn flex_weight(&self) -> Option<f32> {
        self.flex
    }

    pub(crate) fn minimum_width(&self) -> Option<f32> {
        self.min_width.or(self.flex.map(|_| 120.0))
    }

    /// Whether to show a resize handle when the table has bound widths.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub(crate) fn is_resizable(&self) -> bool {
        self.resizable
    }

    /// The title as a value of the current owner. Setup only.
    #[track_caller]
    pub fn title(&self) -> Value<String> {
        self.def.label_value()
    }

    /// The text of `row`'s cell, which also names the cell.
    pub fn text_of(&self, row: &R) -> String {
        (self.text)(row)
    }

    /// The view of `row`'s cell given to [`cell`](Self::cell); `None` when
    /// the cell shows its text.
    pub fn cell_of(&self, row: &R) -> Option<View> {
        self.cell.as_ref().map(|cell| cell(row))
    }

    /// Whether the cells show a [`cell`](Self::cell) view instead of their
    /// text, so a table can choose how to draw them before it has a row.
    pub fn has_cell(&self) -> bool {
        self.cell.is_some()
    }

    /// Whether the header sorts the rows ([`compare`](Self::compare) or
    /// [`sortable`](Self::sortable)).
    pub fn is_sortable(&self) -> bool {
        self.def.is_sortable()
    }

    /// Whether the column takes part in the filter ([`filter`](Self::filter)).
    pub fn is_filtered(&self) -> bool {
        self.def.is_filtered()
    }

    /// Whether the column is [`numeric`](Self::numeric).
    pub fn is_numeric(&self) -> bool {
        self.numeric
    }

    /// The fixed width given to [`width`](Self::width).
    pub fn width_value(&self) -> Option<f32> {
        self.def.width_value().map(|width| width as f32)
    }

    /// The column of the headless row model, keyed by `key` (a table uses
    /// the column's position): its title, width, ordering, match and
    /// visibility.
    pub fn definition<C: Clone>(&self, key: C) -> ColumnDef<R, C> {
        self.def
            .clone()
            .min_width(self.minimum_width().map(f64::from))
            .with_key(key)
    }
}

/// Two cell texts in order: numerically when both are numbers, else ignoring
/// case.
fn compare_text(a: &str, b: &str) -> Ordering {
    match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(a), Ok(b)) => a.total_cmp(&b),
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}
