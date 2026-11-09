//! Role: The Theme panel.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;
use crate::widgets::property_row::{color_property_row, property_row};

/// The theme tokens and their overrides.
pub(crate) fn view(session: Session) -> impl IntoView {
    let tokens = signal(None::<p::ThemeTokenList>);
    on_mount(move || load_tokens(session, tokens));
    let tokens_signal = tokens;
    let state = memo(move || {
        (
            tokens.get(),
            session
                .overrides
                .latest()
                .map(|list| list.overrides)
                .unwrap_or_default(),
        )
    });
    dynamic(state, move |(loaded, overrides)| match loaded {
        None => text("Loading theme tokens...").into_view(),
        Some(list) if list.tokens.is_empty() => text("No theme tokens").into_view(),
        Some(list) => column(
            list.tokens
                .iter()
                .map(|token| token_row(session, tokens_signal, token, &overrides))
                .collect::<Vec<_>>(),
        )
        .style(style().gap(2))
        .into_view(),
    })
}

/// One token row; color tokens get the color input.
fn token_row(
    session: Session,
    tokens: Signal<Option<p::ThemeTokenList>>,
    token: &p::ThemeToken,
    overrides: &[p::StyleOverride],
) -> View {
    let name = token.name.clone();
    let existing = token_override(overrides, &name).map(|entry| entry.id);
    let enabled = signal(existing.is_some());
    let value = signal(token.value.clone());
    let toggle_name = name.clone();
    let toggle_value = value;
    effect_on(enabled, move |on| {
        if *on {
            set_token(
                session,
                tokens,
                toggle_name.clone(),
                toggle_value.get_untracked(),
            );
        } else if let Some(id) = existing {
            clear_token(session, tokens, id);
        }
    });
    let commit = move |text: String| set_token(session, tokens, name.clone(), text);
    if token.kind == p::TokenKind::Color {
        color_property_row(token.name.clone(), value, enabled, commit).into_view()
    } else {
        property_row(token.name.clone(), value, enabled, commit).into_view()
    }
}

/// The token override of `name`, if any.
fn token_override<'a>(
    overrides: &'a [p::StyleOverride],
    name: &str,
) -> Option<&'a p::StyleOverride> {
    overrides.iter().rev().find(|entry| {
        entry.property == name
            && matches!(&entry.scope, p::OverrideScope::Token { token } if token == name)
    })
}

/// Add one token override, then reload the tokens and the override list.
fn set_token(
    session: Session,
    tokens: Signal<Option<p::ThemeTokenList>>,
    name: String,
    value: String,
) {
    session.send::<p::SetStyleOverride>(
        p::SetStyleOverrideParams {
            scope: p::OverrideScope::Token {
                token: name.clone(),
            },
            property: name,
            value,
        },
        move |_| {
            session.overrides.refetch();
            session.detail.refetch();
            load_tokens(session, tokens);
        },
    );
}

/// Remove one token override, then reload the tokens and the override list.
fn clear_token(session: Session, tokens: Signal<Option<p::ThemeTokenList>>, id: u64) {
    session.send::<p::ClearStyleOverrides>(
        p::ClearStyleOverridesParams { id: Some(id) },
        move |_| {
            session.overrides.refetch();
            session.detail.refetch();
            load_tokens(session, tokens);
        },
    );
}

/// Ask the target for the theme tokens.
fn load_tokens(session: Session, tokens: Signal<Option<p::ThemeTokenList>>) {
    session.send::<p::ListThemeTokens>(p::Empty {}, move |list| tokens.set(Some(list)));
}
