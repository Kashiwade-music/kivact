//! Role: The chat composer recipe state, tokens, recipe table and default looks.
use crate::look_common::{column, muted};

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{ColorToken, Style, Theme, style};
pub(crate) fn composer_surface(t: &Theme, s: &Look<ChatComposerState>) -> Style {
    column(t, s)
        .padding(t.space().md)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
        .background(ColorToken::Surface)
}

pub(crate) fn composer_status(t: &Theme, s: &Look<ChatComposerState>) -> Style {
    muted(t, s).color(match s.status {
        Some(ComposerStatus::Error(_)) => ColorToken::Danger,
        _ => ColorToken::Warning,
    })
}

pub(crate) fn input<S>(_: &Theme, _: &Look<S>) -> Style {
    style().grow(1).min_width(0)
}

/// Composer facts supplied to its recipes.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatComposerState {
    pub stopped: bool,
    pub status: Option<ComposerStatus>,
}

recipe_table! {
    /// Styles for the persistent parts of ChatComposer.
    ChatComposerRecipes {
        root(root_recipe): ChatComposerState = crate::look_common::column,
        surface(surface_recipe): ChatComposerState = composer_surface,
        input(input_recipe): ChatComposerState = input,
        send(send_recipe): ChatComposerState = crate::look_common::slot,
        header(header_recipe): ChatComposerState = crate::look_common::row,
        footer(footer_recipe): ChatComposerState = crate::look_common::row,
        status(status_recipe): ChatComposerState = composer_status,
        drawer(drawer_recipe): ChatComposerState = crate::look_common::slot,
        header_actions(header_actions_recipe): ChatComposerState = crate::look_common::slot,
        header_context(header_context_recipe): ChatComposerState = crate::look_common::slot,
        footer_actions(footer_actions_recipe): ChatComposerState = crate::look_common::slot,
        send_actions(send_actions_recipe): ChatComposerState = crate::look_common::slot,
    }
}

/// A composer diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComposerStatus {
    Error(String),
    Warning(String),
}

kivact_core::state::constant_value!(ComposerStatus);
