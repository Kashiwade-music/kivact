//! Role: The toast recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::style::{
    Align, Color, ColorToken, Direction, Justify, Position, Style, Theme, pct, style,
};
use kivact_interaction::{ToastState, Tone};
/// The regions at the bottom right, above the page.
pub(crate) fn toast_region(t: &Theme, _: &PartState) -> Style {
    style()
        .position(Position::Fixed)
        .right(t.space().xl)
        .bottom(t.space().xl)
        .width(320)
        .max_width(pct(100))
}

/// One live region's toasts in a column; a space above the assertive
/// group while both groups show toasts.
pub(crate) fn toast_group(t: &Theme, state: &ToastGroupState) -> Style {
    let s = style().gap(t.space().sm);
    if state.count > 0 && state.above > 0 {
        s.margin_top(t.space().sm)
    } else {
        s
    }
}

/// A toast on the raised surface, its edge in the tone's color.
pub(crate) fn toast(t: &Theme, state: &ToastState) -> Style {
    let edge: Color = match state.tone {
        Tone::Neutral | Tone::Info => ColorToken::Border.into(),
        Tone::Success => ColorToken::Success.into(),
        Tone::Warning => ColorToken::Warning.into(),
        Tone::Danger => ColorToken::Danger.into(),
        Tone::Accent => ColorToken::Accent.into(),
    };
    style()
        .direction(Direction::Row)
        .align_items(Align::Start)
        .gap(t.space().md)
        .padding(t.space().lg)
        .background(ColorToken::Raised)
        .border(1, ColorToken::Border)
        .border_left(3, edge)
        .radius(t.radius().md)
        .shadow(kivact_core::draw::Shadow::new(0, 4, 12, ColorToken::Shadow))
}

/// The tone's icon box: 16 px in the tone's color.
pub(crate) fn toast_icon(_: &Theme, state: &ToastState) -> Style {
    let color: Color = match state.tone {
        Tone::Neutral => ColorToken::TextMuted.into(),
        Tone::Info | Tone::Accent => ColorToken::Accent.into(),
        Tone::Success => ColorToken::Success.into(),
        Tone::Warning => ColorToken::Warning.into(),
        Tone::Danger => ColorToken::Danger.into(),
    };
    style().size(16, 16).shrink(0).color(color)
}

pub(crate) fn toast_title(t: &Theme, _: &ToastState) -> Style {
    style().weight(t.text().weight_strong)
}

pub(crate) fn toast_body(t: &Theme, _: &ToastState) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}

/// The close button, around its `x` icon.
pub(crate) fn toast_close(t: &Theme, _: &ToastState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .shrink(0)
        .size(24, 24)
        .radius(t.radius().sm)
        .color(ColorToken::TextMuted)
        .hover(style().background(ColorToken::Hover))
}

/// Shared by all kits: the row owns size presence independently of card paint.
pub(crate) fn toast_row(t: &Theme, _: &PartState) -> Style {
    let tokens = t.ext::<crate::tokens::UiTokens>();
    let motion = &tokens.toast.row_motion;
    let base = style()
        .shrink(0)
        .enter_size(kivact_core::view::Orientation::Vertical)
        .exit_size(kivact_core::view::Orientation::Vertical);
    if let Some(transition) = &motion.transition {
        base.padding_top(motion.gap)
            .transition(kivact_core::style::TransitionProp::Size, transition.clone())
    } else {
        base
    }
}

/// The state of one live region's column of toasts in a
/// [`toast_region`](super::toast_region), for its recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToastGroupState {
    /// The actual input state of the column.
    pub input: PartState,
    /// The assertive group (warnings and errors), below the polite one.
    pub assertive: bool,
    /// How many toasts the group shows.
    pub count: usize,
    /// How many toasts the groups above this one show (the polite group's
    /// for the assertive group, 0 for the polite group), so a look puts
    /// space between two groups only when both show toasts.
    pub above: usize,
}

recipe_table! {
    /// The recipes of a [`toast_region`](super::toast_region).
    ToastRecipes {
        /// The region, placed at the window's bottom right.
        region(region_recipe): PartState = toast_region,
        /// The column of one live region's toasts (polite, then assertive).
        group(group_recipe): ToastGroupState = toast_group,
        /// The size-presence shell around one card, including its spacing.
        row(row_recipe): PartState = toast_row,
        /// One toast: its title, its body, its action and its close button.
        toast(toast_recipe): ToastState = toast,
        title(title_recipe): ToastState = toast_title,
        body(body_recipe): ToastState = toast_body,
        /// The icon of the toast's tone, which the theme's `IconSet` names
        /// `toast-info`, `toast-success`, `toast-warning`, `toast-danger`
        /// and `toast-accent` (a neutral toast has none, and a toast with its
        /// own [`Toast::icon`](kivact_interaction::Toast::icon) draws that
        /// view instead).
        icon(icon_recipe): ToastState = toast_icon,
        /// The close button, holding the theme's `x` icon.
        close(close_recipe): ToastState = toast_close,
    }
}
