//! Role: Compose plain, pressable, or selectable card surfaces.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use kivact_core::state::{Binding, IntoBinding, IntoValue, Value};
use kivact_core::view::{
    Handled, IntoView, Key, KeyEvent, Modifiers, Node, NodeBuilder, View, node, node_ref,
};
use kivact_interaction::{use_button, use_checkbox};

use crate::component_values::CardState;

// xtask-allow(state-fields): Setup-only independent slots and behavior inputs; selection belongs to the caller binding.
/// A surface containing optional header, media, content and footer regions.
pub struct Card {
    root: Node,
    content: View,
    header: Option<View>,
    media: Option<View>,
    footer: Option<View>,
    action: CardAction,
    disabled: Value<bool>,
    variants: Variants,
}
enum CardAction {
    Plain,
    Press,
    Select(Binding<bool>),
}
/// Build a card around content.
pub fn card(children: impl IntoView) -> Card {
    Card {
        root: node(),
        content: children.into_view(),
        header: None,
        media: None,
        footer: None,
        action: CardAction::Plain,
        disabled: false.into_value(),
        variants: Variants::default(),
    }
}
impl Card {
    /// Set the header region.
    pub fn header(mut self, value: impl IntoView) -> Self {
        self.header = Some(value.into_view());
        self
    }
    /// Set the media region.
    pub fn media(mut self, value: impl IntoView) -> Self {
        self.media = Some(value.into_view());
        self
    }
    /// Set the footer region.
    pub fn footer(mut self, value: impl IntoView) -> Self {
        self.footer = Some(value.into_view());
        self
    }
    /// Activate the card with click, Enter, Space or assistive activation.
    pub fn on_press(mut self, f: impl Into<kivact_core::state::Callback<()>>) -> Self {
        self.root = self.root.on_press(f);
        if matches!(self.action, CardAction::Plain) {
            self.action = CardAction::Press;
        }
        self
    }
    /// Make the card a checkbox backed by the supplied binding.
    pub fn selected(mut self, selected: impl IntoBinding<bool>) -> Self {
        self.action = CardAction::Select(selected.into_binding());
        self
    }
    /// Disable activation.
    pub fn disabled(mut self, value: impl IntoValue<bool>) -> Self {
        self.disabled = value.into_value();
        self
    }
    /// Add a theme-defined appearance.
    pub fn variant(self, value: impl Variant) -> Self {
        self.variants.add(value);
        self
    }
}
impl NodeBuilder for Card {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Card {
    fn into_view(self) -> View {
        let disabled = self.disabled;
        let selected = match self.action {
            CardAction::Select(binding) => Some(binding),
            _ => None,
        };
        let interactive = !matches!(self.action, CardAction::Plain);
        let state = move || CardState {
            selected: selected.map(|selected| selected.get()),
            disabled: disabled.get(),
            interactive,
        };
        let root = match self.action {
            CardAction::Plain => self.root,
            CardAction::Press => self.root.part(use_button().disabled(disabled).control()),
            CardAction::Select(selected) => {
                let reference = node_ref();
                let focused = reference.focused();
                self.root
                    .node_ref(reference)
                    .part(use_checkbox(selected).disabled(disabled).control())
                    .on_key(move |event: KeyEvent| {
                        if event.key() == Key::Enter
                            && focused.get()
                            && !event.repeat()
                            && event.modifiers() == Modifiers::default()
                            && !disabled.get()
                        {
                            reference.activate();
                            Handled::Yes
                        } else {
                            Handled::No
                        }
                    })
            }
        };
        let variants = &self.variants;
        let header = self.header.map(|view| {
            with_recipe_style(
                node().child(view),
                |r| r.card_recipes().header_recipe(),
                variants,
                state,
            )
        });
        let media = self.media.map(|view| {
            with_recipe_style(
                node().child(view),
                |r| r.card_recipes().media_recipe(),
                variants,
                state,
            )
        });
        let content = with_recipe_style(
            node().child(self.content),
            |r| r.card_recipes().content_recipe(),
            variants,
            state,
        );
        let footer = self.footer.map(|view| {
            with_recipe_style(
                node().child(view),
                |r| r.card_recipes().footer_recipe(),
                variants,
                state,
            )
        });
        with_base_recipe_style(root, |r| r.card_recipes().root_recipe(), variants, state)
            .child((header, media, content, footer))
            .into_view()
    }
}
