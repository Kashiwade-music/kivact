//! Role: The Card builder, its pressable and selectable part wiring and rendering.

use crate::card::look::CardAction;
use crate::card::look::CardState;
use crate::card::look::CardTheme;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::state::{IntoValue, Signal, Value};
use kivact_core::view::{
    Element, Handled, IntoView, Key, KeyEvent, Modifiers, Node, View, node, node_ref,
};
use kivact_interaction::{use_button, use_checkbox};

// xtask-allow(state): legacy: header, media, footer
/// A surface containing optional header, media, content and footer regions.
pub struct Card {
    root: Node,
    content: View,
    header: Option<View>,
    media: Option<View>,
    footer: Option<View>,
    action: CardKind,
}
enum CardKind {
    Plain,
    Press,
    Select(Signal<bool>),
}
/// Build a card around content.
pub fn card(children: impl IntoView) -> Card {
    Card {
        root: node(),
        content: children.into_view(),
        header: None,
        media: None,
        footer: None,
        action: CardKind::Plain,
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
    /// Make the card a checkbox backed by the supplied binding.
    pub fn selected(mut self, selected: Signal<bool>) -> Self {
        self.action = CardKind::Select(selected);
        self
    }
}
impl Element for Card {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Card {
    fn into_view(mut self) -> View {
        let variants = self.root.authored().variants();
        // The press and the disabled state are the root's (`Element::on_press`
        // and `Element::disabled`): a card with a press handler is a button.
        let disabled = self
            .root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        if matches!(self.action, CardKind::Plain) && self.root.authored().has_press() {
            self.action = CardKind::Press;
        }
        let has_media = self.media.is_some();
        let has_footer = self.footer.is_some();
        let (root, input) = card_target(self.root, self.action, disabled);
        let state = move || CardState {
            action: match input {
                CardInput::Plain => CardAction::Plain,
                CardInput::Press(input) => CardAction::Press { input: input.get() },
                CardInput::Select(input, selected) => CardAction::Select {
                    input: input.get(),
                    selected: selected.get(),
                },
            },
            disabled: disabled.get(),
            has_media,
            has_footer,
        };
        let variants = &variants;
        let header = self.header.map(|view| {
            with_recipe_style(
                node().child(view),
                |r: &CardTheme| r.header_recipe(),
                variants,
                state,
            )
        });
        let media = self.media.map(|view| {
            with_recipe_style(
                node().child(view),
                |r: &CardTheme| r.media_recipe(),
                variants,
                state,
            )
        });
        let content = (!kivact_core::custom::view_is_empty(&self.content)).then(|| {
            with_recipe_style(
                node().child(self.content),
                |r: &CardTheme| r.content_recipe(),
                variants,
                state,
            )
        });
        let footer = self.footer.map(|view| {
            with_recipe_style(
                node().child(view),
                |r: &CardTheme| r.footer_recipe(),
                variants,
                state,
            )
        });
        let overlay = with_recipe_style(
            node().hit_testable(false).focusable(false),
            |r: &CardTheme| r.overlay_recipe(),
            variants,
            state,
        )
        .style(
            kivact_core::style::style()
                .position(kivact_core::style::Position::Absolute)
                .inset(0),
        );
        with_base_recipe_style(root, |r: &CardTheme| r.root_recipe(), variants, state)
            .style(kivact_core::style::style().position(kivact_core::style::Position::Relative))
            .child((media, header, content, footer, overlay))
            .into_view()
    }
}

/// The behavior a card's root got, with the live input state of its part.
#[derive(Clone, Copy)]
enum CardInput {
    Plain,
    Press(Value<kivact_core::custom::PartState>),
    Select(Value<kivact_core::custom::PartState>, Signal<bool>),
}

/// Attach the one canonical behavior part and expose its actual input snapshot.
fn card_target(root: Node, action: CardKind, disabled: Value<bool>) -> (Node, CardInput) {
    match action {
        CardKind::Plain => (root, CardInput::Plain),
        CardKind::Press => {
            let part = use_button().disabled(disabled).control();
            let input = part.state();
            (root.part(part), CardInput::Press(input))
        }
        CardKind::Select(selected) => {
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
                CardInput::Select(input, selected),
            )
        }
    }
}
