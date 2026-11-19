//! Role: The virtual list's public type, constructor and builder methods.

use std::panic::Location;
use std::rc::Rc;

use kivact_runtime::{IntoValue, Prop as InputProp};
use kivact_types::Role;

use crate::view::descriptor::IntoView;
use crate::view::element_data::{Content, Element, Node};
use crate::view::flow::list::{ListInput, ListSource, RowFn, source_row};

/// Row height policy of a virtual list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RowHeight {
    /// Every row has this height (logical px).
    Fixed(f64),
    /// Rows are measured by the host (`LayoutChanged`) and cached per key;
    /// `estimate` is used until a row has been measured.
    Measured { estimate: f64 },
}

/// Virtualized keyed list. Built by [`virtual_list`].
pub struct VirtualList<T: 'static, K: 'static> {
    pub(super) items: InputProp<Vec<T>>,
    pub(super) key: Rc<dyn Fn(&T) -> K>,
    pub(super) row: RowFn<K, T>,
    pub(super) height: RowHeight,
    pub(super) overscan: f64,
    pub(super) scroll_to: InputProp<Option<K>>,
    pub(super) keep_focused: bool,
    /// Holds the modifiers (`Element`); its content and children are unused.
    pub(super) root: Node,
    pub(super) at: &'static Location<'static>,
}

/// A virtualized keyed list: only the rows in (or near) the viewport are
/// mounted, each as `row(item)` in its own owner. `item` is the source's
/// row handle ([`ListSource::Row`]), the same as for
/// [`for_each`](crate::view::for_each): a `Value<T>`, or a `Binding<T>`
/// that writes back into a `Signal`/`Binding` source.
///
/// ```ignore
/// fn log_row(entry: Value<Entry>) -> impl IntoView { .. }
/// virtual_list(rows, |r| r.id, log_row).row_height(RowHeight::Fixed(22.0))
/// ```
#[track_caller]
pub fn virtual_list<T, K, V, S>(
    items: S,
    key: impl Fn(&T) -> K + 'static,
    row: impl Fn(S::Row) -> V + 'static,
) -> VirtualList<T, K>
where
    T: kivact_runtime::PropValue,
    K: kivact_runtime::ItemKey,
    V: IntoView,
    S: ListSource<T>,
{
    let ListInput {
        items,
        row: make_row,
    } = items.into_list_source();
    let key = Rc::new(key);
    VirtualList {
        items,
        row: source_row(key.clone(), make_row, row),
        key,
        height: RowHeight::Fixed(24.0),
        overscan: 200.0,
        scroll_to: kivact_runtime::Prop::constant(None),
        keep_focused: true,
        root: Node::new(Content::Box, None).role(Role::List),
        at: Location::caller(),
    }
}

impl<T: 'static, K: 'static> VirtualList<T, K> {
    pub fn row_height(mut self, h: RowHeight) -> Self {
        self.height = h;
        self
    }

    /// Extra pixels mounted above and below the viewport (default 200).
    pub fn overscan(mut self, px: impl Into<f64>) -> Self {
        self.overscan = px.into().max(0.0);
        self
    }

    /// Scroll so that the row `request` names is in view, each time the
    /// value changes to a `Some(key)`: another key, or the same key after a
    /// `None`, scrolls again; an unchanged value does not. The scroll is the
    /// smallest one that brings the row into the viewport, as
    /// [`NodeRef::scroll_into_view`](crate::view::NodeRef::scroll_into_view)
    /// does, so a row already in view does not move the list. A key that is
    /// not in the items does nothing.
    ///
    /// The row need not be mounted. Its position comes from the row height
    /// (`Fixed`, or for `Measured` the measured height of the rows above and
    /// the `estimate` for those never measured), the list scrolls there, and
    /// once the row is mounted the host scrolls it into view exactly. A
    /// `Measured` row far below rows that were never measured may therefore
    /// take more than one step to settle.
    ///
    /// ```ignore
    /// let target = signal(None::<u32>);
    /// virtual_list(rows, |r| r.id, log_row).scroll_to(target)
    /// // later: target.set(Some(4_200));
    /// ```
    pub fn scroll_to(mut self, request: impl IntoValue<Option<K>>) -> Self {
        self.scroll_to = request.into_prop();
        self
    }

    /// Whether the row that holds the keyboard focus (the row itself or a
    /// node inside it) stays mounted when it leaves the viewport and the
    /// overscan (default `true`). It stays at its position, outside the
    /// viewport and not hidden, so the focus is not lost while the list
    /// scrolls to another row or the user scrolls with the wheel. Once the
    /// focus has left it, the next update of the visible range unmounts the
    /// row as usual.
    pub fn release_offscreen_focus(mut self) -> Self {
        self.keep_focused = false;
        self
    }
}

impl<T: kivact_runtime::PropValue, K: kivact_runtime::ItemKey> Element for VirtualList<T, K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
