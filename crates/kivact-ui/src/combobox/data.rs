//! Role: Hold passive combobox construction inputs shared by configuration and rendering.
use crate::SelectPresentation;
use crate::variants::Variants;
use kivact_core::state::{Binding, ItemKey, Value};
use kivact_core::view::Node;
use kivact_interaction::{ComboboxMode, ListItem, Selection};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

pub(super) type Matcher<K> = Rc<dyn Fn(&str, &ListItem<K>) -> bool>;
pub(super) type Search<K> =
    Rc<dyn Fn(String) -> Pin<Box<dyn Future<Output = kivact_core::Result<Vec<ListItem<K>>>>>>>;
/// A themed editable list of keyed suggestions, created by [`super::handle::combobox`].
/// Single choices show their label; multiple choices can be displayed as chips.
pub struct Combobox<K: ItemKey> {
    pub(super) root: Node,
    pub(super) label: Value<String>,
    pub(super) options: Value<Vec<ListItem<K>>>,
    pub(super) selection: Selection<K>,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(super) query: Option<Binding<String>>,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(super) open: Option<Binding<bool>>,
    pub(super) on_edit: Option<kivact_core::state::Callback<kivact_core::view::EditPhase>>,
    // xtask-allow(state-fields): independent: a construction option; every combination is valid
    pub(super) free_text: bool,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(super) mode: Option<ComboboxMode>,
    // xtask-allow(state-fields): independent: a construction option; every combination is valid
    pub(super) chips: bool,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(super) placeholder: Option<Value<String>>,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(super) empty: Option<Value<String>>,
    pub(super) clear: Value<bool>,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(super) matcher: Option<Matcher<K>>,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(super) search: Option<Search<K>>,
    pub(super) debounce: Value<Duration>,
    // xtask-allow(state-fields): independent: a construction option; every combination is valid
    pub(super) exclude_selected: bool,
    pub(super) presentation: SelectPresentation,
    pub(super) variants: Variants,
}
