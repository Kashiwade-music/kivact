//! Role: Hold passive combobox construction inputs shared by configuration and rendering.
use super::ComboboxMode;
use crate::SelectPresentation;
use kivact_core::state::{ItemKey, Signal, Value};
use kivact_core::view::Node;
use kivact_interaction::{ListItem, Selection};
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

pub(super) type Matcher<K> = Rc<dyn Fn(&str, &ListItem<K>) -> bool>;
pub(super) type Search<K> =
    Rc<dyn Fn(String) -> Pin<Box<dyn Future<Output = kivact_core::Result<Vec<ListItem<K>>>>>>>;
// xtask-allow(state): independent: query and open are optional caller bindings (absent means local signals), free_text, chips and exclude_selected setup flags, and mode, placeholder, empty, matcher and search optional settings, each set only by its by-value setter; Inline mode ignores open, as documented on open(); label is the caller's label, present only when the caller set it (the Option records only that), set only by its by-value label setter; Combobox::query binds the editor text to the caller;
/// A themed editable list of keyed suggestions, created by [`super::handle::combobox`].
/// Single choices show their label; multiple choices can be displayed as chips.
pub struct Combobox<K: ItemKey> {
    pub(super) root: Node,
    pub(super) label: Option<Value<String>>,
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
    pub(super) at: &'static std::panic::Location<'static>,
}
