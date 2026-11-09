//! Role: Configure the standard keyed combobox family.
use super::data::Combobox;
use crate::SelectPresentation;
use crate::variants::{Variant, Variants};
use kivact_core::state::{IntoBinding, IntoText, IntoValue, ItemKey};
use kivact_core::view::{IntoView, Node, NodeBuilder, View, node};
use kivact_interaction::{ComboboxMode, IntoSelection, ListItem};
use std::future::Future;
use std::rc::Rc;
use std::time::Duration;

/// A combobox named `label` over keyed `options`, editing one or many choices.
///
/// Typing filters labels, arrows highlight enabled rows, and Enter chooses.
/// Focus remains in the editor. Multiple selection toggles keys and Backspace
/// in an empty editor removes the last choice. Use `.chips()` to draw removals.
///
/// ```no_run
/// use kivact::prelude::*;
/// fn colors() -> impl IntoView {
///     let chosen = signal(Vec::<&str>::new());
///     kivact::ui::combobox("Colors", vec![ListItem::new("red", "Red")], chosen).chips()
/// }
/// ```
#[track_caller]
pub fn combobox<K: ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    selection: impl IntoSelection<K>,
) -> Combobox<K> {
    Combobox {
        root: node(),
        label: label.into_text_value(),
        options: options.into_value(),
        selection: selection.into_selection(),
        query: None,
        open: None,
        on_edit: None,
        free_text: false,
        mode: None,
        chips: false,
        placeholder: None,
        empty: None,
        clear: false.into_value(),
        matcher: None,
        search: None,
        debounce: Duration::from_millis(150).into_value(),
        exclude_selected: false,
        presentation: SelectPresentation::Popover,
        variants: Variants::default(),
    }
}
impl<K: ItemKey> Combobox<K> {
    /// Observe each committed selection change.
    pub fn on_edit<R: kivact_core::view::HandlerResult>(
        mut self,
        f: impl Fn(kivact_core::view::EditPhase) -> R + 'static,
    ) -> Self {
        self.on_edit = Some(kivact_core::state::callback(f));
        self
    }
    /// Control popup visibility. Inline mode always shows its suggestions.
    pub fn open(mut self, open: impl IntoBinding<bool>) -> Self {
        self.open = Some(open.into_binding());
        self
    }
    /// Bind the text typed into the editor independently of keyed selection.
    pub fn query(mut self, query: impl IntoBinding<String>) -> Self {
        self.query = Some(query.into_binding());
        self
    }
    /// Preserve text that does not correspond to a suggestion when the list closes.
    pub fn free_text(mut self) -> Self {
        self.free_text = true;
        self
    }
    /// Set an inline list or a popup with caller-owned visibility.
    pub fn mode(mut self, mode: ComboboxMode) -> Self {
        self.mode = Some(mode);
        self
    }
    /// Show each selected key as a chip with a remove button.
    pub fn chips(mut self) -> Self {
        self.chips = true;
        self
    }
    /// Hint shown while the editor is empty.
    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.placeholder = Some(text.into_text_value());
        self
    }
    /// Message shown when no suggestion matches.
    pub fn empty_text(mut self, text: impl IntoText) -> Self {
        self.empty = Some(text.into_text_value());
        self
    }
    /// Show a clear button for a single choice.
    pub fn show_clear(mut self, show: impl IntoValue<bool>) -> Self {
        self.clear = show.into_value();
        self
    }
    /// Replace the case-insensitive label matcher.
    pub fn filter(mut self, matches: impl Fn(&str, &str) -> bool + 'static) -> Self {
        self.matcher = Some(Rc::new(move |q, item| matches(q.trim(), &item.label())));
        self
    }
    /// Fetch suggestions when opened and after the configured debounce.
    /// A changed query or a closed popup cancels the previous request.
    pub fn search<F, Fut>(mut self, search: F) -> Self
    where
        F: Fn(String) -> Fut + 'static,
        Fut: Future<Output = kivact_core::Result<Vec<ListItem<K>>>> + 'static,
    {
        self.search = Some(Rc::new(move |query| Box::pin(search(query))));
        self
    }
    /// Delay before fetching typed suggestions; opening fetches immediately.
    pub fn debounce(mut self, delay: impl IntoValue<Duration>) -> Self {
        self.debounce = delay.into_value();
        self
    }
    /// Hide already selected keys from suggestions, as in a tokenizer.
    pub fn exclude_selected(mut self) -> Self {
        self.exclude_selected = true;
        self
    }
    /// Choose an anchored popup or touch sheet.
    pub fn presentation(mut self, presentation: SelectPresentation) -> Self {
        self.presentation = presentation;
        self
    }
    /// Supply a constant theme recipe variant.
    pub fn variant(mut self, variant: impl Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl<K: ItemKey> NodeBuilder for Combobox<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl<K: ItemKey> IntoView for Combobox<K> {
    fn into_view(self) -> View {
        super::render::render(self)
    }
}
