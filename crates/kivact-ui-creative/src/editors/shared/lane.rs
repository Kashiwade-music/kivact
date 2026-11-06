//! Role: Lane extensions, cell layout and menus shared by domain editors.
use crate::editors::shared::model::{EditorContext, Lane};
use kivact::state::callback;
use kivact::state::{Callback, Value};
use kivact::view::Node;

use kivact::view::View;
use kivact::view::{HandlerResult, node_ref};
use std::rc::Rc;
pub(crate) type CellRenderer = kivact::view::ViewFn<Value<Lane>>;
pub(crate) type MenuFactory = kivact::view::ViewFn<(Node, Value<EditorContext>)>;
#[derive(Default)]
pub(crate) struct LaneExtensions {
    pub(crate) cells: Vec<(Value<f64>, CellRenderer)>,
    // xtask-allow(state-fields): Independent optional setup callback or menu factory.
    pub(crate) lane_activate: Option<Callback<u64>>,
    // xtask-allow(state-fields): Independent optional setup callback or menu factory.
    pub(crate) item_activate: Option<Callback<u64>>,
    // xtask-allow(state-fields): Independent optional setup callback or menu factory.
    pub(crate) menu: Option<MenuFactory>,
}
pub(crate) const GRAPH_LANE_WIDTH: f64 = crate::editors::shared::metrics::SIDEBAR - 72.0;
pub(crate) fn graph_lane_y(index: usize) -> f64 {
    crate::editors::shared::metrics::RULER
        + crate::editors::shared::metrics::GRAPH_LANE_INSET
        + index as f64 * crate::editors::shared::metrics::GRAPH_LANE_PITCH
}

pub(crate) fn menu_factory<K: kivact::state::ItemKey, R: HandlerResult>(
    build: impl Fn(EditorContext) -> Vec<kivact::interaction::MenuEntry<K>> + 'static,
    choose: impl Fn((EditorContext, K)) -> R + 'static,
) -> MenuFactory {
    let build = Rc::new(build);
    let choose = callback(choose);
    kivact::view::ViewFn::from(move |(root, context): (Node, Value<EditorContext>)| {
        let build = build.clone();
        let entries = kivact::state::memo(move || build(context.get()));
        kivact::view::IntoView::into_view(
            kivact::ui::menu("", entries)
                .on_activate(move |key| choose.try_call((context.get_untracked(), key)))
                .attach_context(root),
        )
    })
}

macro_rules! lane_extension_methods {
    ($($field:ident).+) => {
        /// Append a fixed-width lane cell; invalid widths hide the cell.
        /// The current lane value preserves keyed row identity.
        /// Disabling the editor leaves enabled descendants interactive, as for ordinary nodes.
        /// Pass the same disabled value to a cell control when it should also be disabled.
        pub fn lane_cell<V: kivact::view::IntoView>(
            mut self, width: impl kivact::state::IntoValue<f64>,
            render: impl Fn(kivact::state::Value<$crate::editors::shared::model::Lane>) -> V + 'static,
        ) -> Self {
            let width = kivact::state::IntoValue::into_value(width);
let width = kivact::state::memo(move || { let width = width.get(); if width.is_finite() && width > 0.0 { width } else { 0.0 } });
            self.$($field).+.cells.push((width,
                kivact::view::ViewFn::from(render)));
            self
        }
        /// Run on a lane's Enter, stationary primary double click or accessibility Activate.
        pub fn on_lane_activate<R: kivact::view::HandlerResult>(mut self, f: impl Fn(u64) -> R + 'static) -> Self {
            self.$($field).+.lane_activate = Some(kivact::state::callback(f));
            self
        }
        /// Run on an item, trim handle, key or tangent's Enter or stationary double click.
        /// Accessibility Activate activates the corresponding domain target.
        pub fn on_item_activate<R: kivact::view::HandlerResult>(mut self, f: impl Fn(u64) -> R + 'static) -> Self {
            self.$($field).+.item_activate = Some(kivact::state::callback(f));
            self
        }
        /// Use the standard menu with the request's domain snapshot, even after model updates.
        /// A failing command keeps the menu open and reports its error once.
        /// An assistive menu request cancels the current edit before taking its snapshot.
        pub fn context_menu<K: kivact::state::ItemKey, R: kivact::view::HandlerResult>(
            mut self,
            build: impl Fn($crate::editors::shared::model::EditorContext) -> Vec<kivact::interaction::MenuEntry<K>> + 'static,
            choose: impl Fn(($crate::editors::shared::model::EditorContext, K)) -> R + 'static,
        ) -> Self {
            self.$($field).+.menu = Some($crate::editors::shared::lane::menu_factory(build, choose));
            self
        }
    };
}
pub(crate) use lane_extension_methods;

pub(crate) fn lane_cells(
    lanes: Value<Vec<Lane>>,
    extensions: &LaneExtensions,
    rect: impl Fn(usize) -> kivact::geometry::Rect + 'static,
    center: bool,
) -> (View, Value<bool>) {
    use kivact::view::{node, row};

    use kivact::style::{Align, Overflow, Position, style};
    use kivact::view::{Handled, Key};
    let reference = node_ref();
    let focus = reference.focus_within();
    let cells = extensions.cells.clone();
    let rect = Rc::new(rect);
    let rows = kivact::view::for_each(
        lanes,
        |lane| lane.id,
        move |lane| {
            let rect = rect.clone();
            let index = kivact::state::memo(move || {
                lanes
                    .get()
                    .iter()
                    .position(|l| l.id == lane.get().id)
                    .unwrap_or(0)
            });
            let widths: Vec<Value<f64>> = cells.iter().map(|(width, _)| *width).collect();
            let totals = widths.clone();
            let total =
                kivact::state::memo(move || totals.iter().map(|width| width.get()).sum::<f64>());
            let mut views = Vec::new();
            for (cell_index, (width, render)) in cells.iter().enumerate() {
                let rect = rect.clone();
                let preceding = widths[..cell_index].to_vec();
                let cell_offset = kivact::state::memo(move || {
                    preceding.iter().map(|width| width.get()).sum::<f64>()
                });
                let width = *width;
                let cell = if center {
                    row(render.call(lane))
                } else {
                    node().child(render.call(lane))
                };
                views.push(
                    cell.hidden(move || width.get() <= 0.0)
                        .style(move || {
                            let rect = rect(index.get());
                            let left = (rect.width - total.get() + cell_offset.get())
                                .clamp(-f64::MAX, f64::MAX);
                            let cell = style()
                                .position(Position::Absolute)
                                .left(left)
                                .top(0)
                                .width(width.get())
                                .height(rect.height)
                                .shrink(0)
                                .overflow(Overflow::Hidden);
                            if center {
                                cell.align_items(Align::Center)
                            } else {
                                cell
                            }
                        })
                        .on_pointer_default(|_| Handled::Yes)
                        .on_key_default(|event| match event.key() {
                            Key::Tab | Key::Enter | Key::Space => Handled::No,
                            _ => Handled::Yes,
                        }),
                );
            }
            node().child(views).style(move || {
                let rect = rect(index.get());
                style()
                    .position(Position::Absolute)
                    .left(rect.x)
                    .top(rect.y)
                    .width(rect.width)
                    .height(rect.height)
                    .overflow(Overflow::Hidden)
            })
        },
    );
    (cells_layer(reference, rows), focus)
}

impl LaneExtensions {
    pub(crate) fn width(&self, limit: f64) -> Value<f64> {
        let widths: Vec<_> = self.cells.iter().map(|(width, _)| *width).collect();
        kivact::state::IntoValue::into_value(kivact::state::memo(move || {
            widths
                .iter()
                .map(|width| width.get())
                .sum::<f64>()
                .min(limit)
        }))
    }
}

fn cells_layer(reference: kivact::view::NodeRef, rows: impl kivact::view::IntoView) -> View {
    use kivact::style::{Position, style};
    use kivact::view::{IntoView, node};
    node()
        .node_ref(reference)
        .child(rows)
        .style(
            style()
                .position(Position::Absolute)
                .left(0)
                .top(0)
                .size(0, 0),
        )
        .into_view()
}
