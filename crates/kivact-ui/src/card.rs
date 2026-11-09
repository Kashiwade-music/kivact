//! Role: Compose plain, pressable, or selectable card surfaces.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use kivact_core::state::{Binding, IntoBinding, IntoValue, Value};
use kivact_core::view::{
    Element, Handled, IntoView, Key, KeyEvent, Modifiers, Node, View, node, node_ref,
};
use kivact_interaction::{use_button, use_checkbox};

use crate::component_values::{CardState, Selectable};

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
    pub fn variant(mut self, value: impl Variant) -> Self {
        self.variants.add(value);
        self
    }
}
impl Element for Card {
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
        let has_media = self.media.is_some();
        let has_footer = self.footer.is_some();
        let (root, input) = card_target(self.root, self.action, disabled);
        let state = move || CardState {
            selection: Selectable::new(
                selected.is_some(),
                selected.is_some_and(|selected| selected.get()),
            ),
            input: input.map(|input| input.get()),
            disabled: disabled.get(),
            interactive,
            has_media,
            has_footer,
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
        let content = (!kivact_core::custom::view_is_empty(&self.content)).then(|| {
            with_recipe_style(
                node().child(self.content),
                |r| r.card_recipes().content_recipe(),
                variants,
                state,
            )
        });
        let footer = self.footer.map(|view| {
            with_recipe_style(
                node().child(view),
                |r| r.card_recipes().footer_recipe(),
                variants,
                state,
            )
        });
        let overlay = with_recipe_style(
            node().hit_testable(false).focusable(false),
            |r| r.card_recipes().overlay_recipe(),
            variants,
            state,
        )
        .style(
            kivact_core::style::style()
                .position(kivact_core::style::Position::Absolute)
                .inset(0),
        );
        with_base_recipe_style(root, |r| r.card_recipes().root_recipe(), variants, state)
            .style(kivact_core::style::style().position(kivact_core::style::Position::Relative))
            .child((media, header, content, footer, overlay))
            .into_view()
    }
}

/// Attach the one canonical behavior part and expose its actual input snapshot.
fn card_target(
    root: Node,
    action: CardAction,
    disabled: Value<bool>,
) -> (Node, Option<Value<kivact_core::custom::PartState>>) {
    match action {
        CardAction::Plain => (root, None),
        CardAction::Press => {
            let part = use_button().disabled(disabled).control();
            let input = part.state();
            (root.part(part), Some(input))
        }
        CardAction::Select(selected) => {
            let reference = node_ref();
            let focused = reference.focused();
            let part = use_checkbox(selected).disabled(disabled).control();
            let state = part.state();
            let input = kivact_core::state::memo(move || state.get().input);
            (
                root.node_ref(reference)
                    .part(part)
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
                    }),
                Some(input),
            )
        }
    }
}
