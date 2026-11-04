//! Role: Lane extensions, cell layout and menus shared by domain editors.
use crate::editors::shared::model::{EditorContext, Lane};
use kivact::control::Node;
use kivact::prelude::NodeBuilder;
use kivact::view::View;
use kivact::view::{HandlerResult, callback, node_ref};
use kivact::{Callback, Value};
use std::rc::Rc;
pub(crate) type CellRenderer = Rc<dyn Fn(Value<Lane>) -> View>;
pub(crate) type MenuFactory = Rc<dyn Fn(Node, Value<EditorContext>) -> View>;
#[derive(Default)]
pub(crate) struct LaneExtensions {
    pub(crate) cells: Vec<(f64, CellRenderer)>,
    pub(crate) lane_activate: Option<Callback<u64>>,
    pub(crate) item_activate: Option<Callback<u64>>,
    pub(crate) menu: Option<MenuFactory>,
}
pub(crate) const GRAPH_LANE_WIDTH: f64 = crate::editors::shared::metrics::SIDEBAR - 72.0;
pub(crate) fn graph_lane_y(index: usize) -> f64 {
    crate::editors::shared::metrics::RULER
        + crate::editors::shared::metrics::GRAPH_LANE_INSET
        + index as f64 * crate::editors::shared::metrics::GRAPH_LANE_PITCH
}

pub(crate) fn menu_factory<K: Clone + Eq + std::hash::Hash + 'static, R: HandlerResult>(
    build: impl Fn(EditorContext) -> Vec<kivact::ui::MenuEntry<K>> + 'static,
    choose: impl Fn((EditorContext, K)) -> R + 'static,
) -> MenuFactory {
    let build = Rc::new(build);
    let choose = callback(choose);
    Rc::new(move |root, context| {
        let build = build.clone();
        let entries = kivact::memo(move || build(context.get()));
        kivact::view::IntoView::into_view(
            kivact::ui::menu("", entries, move |key| {
                choose.try_call((context.get_untracked(), key))
            })
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
            mut self, width: f64,
            render: impl Fn(kivact::Value<$crate::editors::shared::model::Lane>) -> V + 'static,
        ) -> Self {
            let width = if width.is_finite() && width > 0.0 { width } else { 0.0 };
            self.$($field).+.cells.push((width,
                std::rc::Rc::new(move |lane| kivact::view::IntoView::into_view(render(lane)))));
            self
        }
        /// Run on a lane's Enter, stationary primary double click or accessibility Activate.
        pub fn on_lane_activate<R: kivact::view::HandlerResult>(mut self, f: impl Fn(u64) -> R + 'static) -> Self {
            self.$($field).+.lane_activate = Some(kivact::view::callback(f));
            self
        }
        /// Run on an item, trim handle, key or tangent's Enter or stationary double click.
        /// Accessibility Activate activates the corresponding domain target.
        pub fn on_item_activate<R: kivact::view::HandlerResult>(mut self, f: impl Fn(u64) -> R + 'static) -> Self {
            self.$($field).+.item_activate = Some(kivact::view::callback(f));
            self
        }
        /// Use the standard menu with the request's domain snapshot, even after model updates.
        /// A failing command keeps the menu open and reports its error once.
        /// An assistive menu request cancels the current edit before taking its snapshot.
        pub fn context_menu<K: Clone + Eq + std::hash::Hash + 'static, R: kivact::view::HandlerResult>(
            mut self,
            build: impl Fn($crate::editors::shared::model::EditorContext) -> Vec<kivact::ui::MenuEntry<K>> + 'static,
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
    rect: impl Fn(usize) -> kivact::draw::Rect + 'static,
    center: bool,
) -> (View, Value<bool>) {
    use kivact::control::{node, row};
    use kivact::primitive::NodeExt;
    use kivact::style::{Align, Overflow, Position, style};
    use kivact::view::{Handled, IntoView, Key};
    let reference = node_ref();
    let focus = reference.focus_within();
    let cells = extensions.cells.clone();
    let rect = Rc::new(rect);
    let rows = kivact::view::for_each(
        lanes,
        |lane| lane.id,
        move |_, lane| {
            let rect = rect.clone();
            let index = kivact::memo(move || {
                lanes
                    .get()
                    .iter()
                    .position(|l| l.id == lane.get().id)
                    .unwrap_or(0)
            });
            let total = cells.iter().map(|(width, _)| (*width)).sum::<f64>();
            let mut offset = 0.0;
            let mut views = Vec::new();
            for (width, render) in cells.iter().filter(|(width, _)| *width > 0.0) {
                let rect = rect.clone();
                let cell_offset = offset;
                let width = *width;
                let cell = if center {
                    row(render(lane))
                } else {
                    node().child(render(lane))
                };
                views.push(
                    cell.style(move || {
                        let rect = rect(index.get());
                        let left = (rect.width - total + cell_offset).clamp(-f64::MAX, f64::MAX);
                        let cell = style()
                            .position(Position::Absolute)
                            .left(left)
                            .top(0)
                            .width(width)
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
                offset += width;
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
    (
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
            .into_view(),
        focus,
    )
}
