//! Role: The chat thread recipe state, recipe table and default looks.
use crate::look_common::column;

use crate::recipe::Look;
use crate::recipe::recipe_table;
use kivact_core::style::{Length, ThemeVar};
use kivact_core::style::{Style, Theme, style};

/// The distance from the end that still follows the newest message, in logical px.
pub static FOLLOW_SLACK_VAR: ThemeVar<f64> = ThemeVar::new("chat_thread.follow_slack", 24.0);
/// [`FOLLOW_SLACK_VAR`] as a length for a style.
pub const FOLLOW_SLACK: Length = Length::var(&FOLLOW_SLACK_VAR);

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
    ChatThreadTheme {
        root(root_recipe): ChatThreadState = thread_root,
        viewport(viewport_recipe): ChatThreadState = viewport,
        content(content_recipe): ChatThreadState = crate::look_common::column,
        scroll_button(scroll_button_recipe): ChatThreadState = jump,
        composer(composer_recipe): ChatThreadState = crate::look_common::slot,
    }
}
