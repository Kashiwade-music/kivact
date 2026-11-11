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
    // xtask-allow(state-fields): option: Combobox::query binds the editor text to the caller; absent keeps the query inside the combobox behavior
    pub(super) query: Option<Binding<String>>,
    // xtask-allow(state-fields): option: Combobox::open binds popup visibility to the caller; absent uses a local signal, and inline mode ignores it because its list always shows
    pub(super) open: Option<Binding<bool>>,
    pub(super) on_edit: Option<kivact_core::state::Callback<kivact_core::view::EditPhase>>,
    // xtask-allow(state-fields): option: Combobox::free_text; false accepts only listed options
    pub(super) free_text: bool,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(super) mode: Option<ComboboxMode>,
    // xtask-allow(state-fields): option: Combobox::chips; false shows no chips for the selected keys
    pub(super) chips: bool,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(super) placeholder: Option<Value<String>>,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(super) empty: Option<Value<String>>,
    pub(super) clear: Value<bool>,
    // xtask-allow(state-fields): option: absent means the component's default
    pub(super) matcher: Option<Matcher<K>>,
    // xtask-allow(state-fields): option: Combobox::search; absent filters the caller's items locally, present fetches suggestions and skips the matcher
    pub(super) search: Option<Search<K>>,
    pub(super) debounce: Value<Duration>,
    // xtask-allow(state-fields): option: Combobox::exclude_selected; false keeps selected options in the list
    pub(super) exclude_selected: bool,
    pub(super) presentation: SelectPresentation,
    pub(super) variants: Variants,
}
