//! Role: The Theme panel.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;
use crate::widgets::override_row::{Override, override_row};
use crate::widgets::property_row::RowAccess;

/// The theme tokens and their overrides.
pub(crate) fn view(session: Session) -> impl IntoView {
    let tokens = session.read::<p::ListThemeTokens>();
    let overrides = memo(move || {
        session
            .overrides
            .latest()
            .map(|list| list.overrides)
            .unwrap_or_default()
    });
    let state = memo(move || (tokens.state(), overrides.get()));
    scroll(dynamic(state, move |(state, overrides)| match state {
        AsyncState::Idle | AsyncState::Pending => text("Loading theme tokens...").into_view(),
        AsyncState::Failed(error) => text(error.message().to_owned()).into_view(),
        AsyncState::Ready(list) if list.tokens.is_empty() => text("No theme tokens").into_view(),
        AsyncState::Ready(list) => column(
            list.tokens
                .iter()
                .map(|token| token_row(session, tokens, token, &overrides))
                .collect::<Vec<_>>(),
        )
        .style(style().gap(2))
        .into_view(),
    }))
    .style(style().grow(1.0).min_height(0.0))
}

/// One token row; color tokens get the color input.
fn token_row(
    session: Session,
    tokens: Resource<p::ThemeTokenList>,
    token: &p::ThemeToken,
    overrides: &[p::StyleOverride],
) -> View {
    let name = token.name.clone();
    let existing = token_override(overrides, &name).map(|entry| entry.id);
    let initial = match existing {
        Some(id) => Override::On {
            id: Some(id),
            value: token.value.clone(),
        },
        None => Override::Off {
            value: token.value.clone(),
        },
    };
    let set_name = name.clone();
    override_row(
        name,
        initial,
        token.kind == p::TokenKind::Color,
        RowAccess::Editable,
        move |value| set_token(session, tokens, set_name.clone(), value),
        move |id| clear_token(session, tokens, id),
    )
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
fn set_token(session: Session, tokens: Resource<p::ThemeTokenList>, name: String, value: String) {
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
            tokens.refetch();
        },
    );
}

/// Remove one token override, then reload the tokens and the override list.
fn clear_token(session: Session, tokens: Resource<p::ThemeTokenList>, id: u64) {
    session.send::<p::ClearStyleOverrides>(
        p::ClearStyleOverridesParams { id: Some(id) },
        move |_| {
            session.overrides.refetch();
            session.detail.refetch();
            tokens.refetch();
        },
    );
}
