//! Role: The card component: a surface with header, body and actions.

use super::look::CardAction;
use crate::card::look::CardState;
use crate::card::look::CardTheme;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::custom::{Caller, view_is_empty};
use kivact_core::state::{IntoValue, Signal, Value};
use kivact_core::view::{Handled, Key, KeyEvent, Modifiers, Node, View, node, node_ref};
use kivact_interaction::{use_button, use_checkbox};
use kivact_macros::component;

/// A surface containing optional header, media, content and footer regions.
///
/// `selected` makes the card a checkbox backed by the binding. A card with an
/// `on_press` handler is a button.
#[component]
pub fn card(
    children: View,
    header: Option<View>,
    media: Option<View>,
    footer: Option<View>,
    selected: Option<Signal<bool>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    // The press and the disabled state are the root's (`Element::on_press`
    // and `Element::disabled`): a card with a press handler is a button.
    let disabled = caller.disabled().unwrap_or_else(|| false.into_value());
    let action = match selected {
        Some(selected) => CardKind::Select(selected),
        None if caller.has_press() => CardKind::Press,
        None => CardKind::Plain,
    };
    let has_media = media.is_some();
    let has_footer = footer.is_some();
    let (root, input) = card_target(node(), action, disabled);
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
    let header = header.map(|view| {
        with_recipe_style(
            node().child(view),
            |r: &CardTheme| r.header_recipe(),
            variants,
            state,
        )
    });
    let media = media.map(|view| {
        with_recipe_style(
            node().child(view),
            |r: &CardTheme| r.media_recipe(),
            variants,
            state,
        )
    });
    let content = (!view_is_empty(&children)).then(|| {
        with_recipe_style(
            node().child(children),
            |r: &CardTheme| r.content_recipe(),
            variants,
            state,
        )
    });
    let footer = footer.map(|view| {
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
}

enum CardKind {
    Plain,
    Press,
    Select(Signal<bool>),
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
