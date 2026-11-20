//! Role: Hold passive combobox construction inputs shared by configuration and rendering.
use crate::SelectPresentation;
use crate::variants::Variants;
use kivact_core::state::{ItemKey, Signal, Value};
use kivact_core::view::Node;
use kivact_interaction::{ComboboxMode, ListItem, Selection};
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

pub(super) type Matcher<K> = Rc<dyn Fn(&str, &ListItem<K>) -> bool>;
pub(super) type Search<K> =
    Rc<dyn Fn(String) -> Pin<Box<dyn Future<Output = kivact_core::Result<Vec<ListItem<K>>>>>>>;
// xtask-allow(state): legacy: query, open, free_text, mode, chips, placeholder, empty, matcher, search, exclude_selected; Combobox::query binds the editor text to the caller;
/// A themed editable list of keyed suggestions, created by [`super::handle::combobox`].
/// Single choices show their label; multiple choices can be displayed as chips.
pub struct Combobox<K: ItemKey> {
    pub(super) root: Node,
    pub(super) label: Value<String>,
    pub(super) options: Value<Vec<ListItem<K>>>,
    pub(super) selection: Selection<K>,
    pub(super) query: Option<Signal<String>>,
    pub(super) open: Option<Signal<bool>>,
    pub(super) on_edit: Option<kivact_core::state::Callback<kivact_core::view::EditPhase>>,
    pub(super) free_text: bool,
    pub(super) mode: Option<ComboboxMode>,
    pub(super) chips: bool,
    pub(super) placeholder: Option<Value<String>>,
    pub(super) empty: Option<Value<String>>,
    pub(super) clear: Value<bool>,
    pub(super) matcher: Option<Matcher<K>>,
    pub(super) search: Option<Search<K>>,
    pub(super) debounce: Value<Duration>,
    pub(super) exclude_selected: bool,
    pub(super) presentation: SelectPresentation,
    pub(super) variants: Variants,
}
