//! Role: The chat thread recipe state, tokens, recipe table and default looks.
use crate::look_common::column;

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{Style, Theme, style};
pub(crate) fn thread_root<S>(t: &Theme, s: &Look<S>) -> Style {
    column(t, s)
        .min_height(0)
        .position(kivact_core::style::Position::Relative)
}

pub(crate) fn viewport<S>(_: &Theme, _: &Look<S>) -> Style {
    style().flex_grow(1).min_height(0).min_width(0)
}

pub(crate) fn jump<S>(_: &Theme, _: &Look<S>) -> Style {
    style()
        .position(kivact_core::style::Position::Absolute)
        .right(12)
        .bottom(12)
}

/// Scroll-follow facts supplied by the shared follow behavior.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatThreadState {
    pub distance: f64,
}

recipe_table! {
    /// Styles for the persistent parts of ChatThread.
    ChatThreadRecipes {
        root(root_recipe): ChatThreadState = thread_root,
        viewport(viewport_recipe): ChatThreadState = viewport,
        content(content_recipe): ChatThreadState = crate::look_common::column,
        scroll_button(scroll_button_recipe): ChatThreadState = jump,
        composer(composer_recipe): ChatThreadState = crate::look_common::slot,
    }
}
