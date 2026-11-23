//! Role: The toast recipe state, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::custom::PartState;
use kivact_core::style::{
    Align, Color, FlexDirection, Justify, Position, Style, Theme, pct, style,
};
use kivact_core::style::{Length, ThemeVar};
use kivact_interaction::{ToastState, Tone};

/// The gap above a toast row that is animating its size, in logical px.
pub static ROW_GAP_VAR: ThemeVar<f64> = ThemeVar::new("toast.row_gap", 0.0);
/// [`ROW_GAP_VAR`] as a length for a style.
pub const ROW_GAP: Length = Length::var(&ROW_GAP_VAR);

/// The regions at the bottom right, above the page.
pub(crate) fn toast_region(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .position(Position::Fixed)
        .right(t.get(&kivact_core::style::space::XL_VAR))
        .bottom(t.get(&kivact_core::style::space::XL_VAR))
        .width(320)
        .max_width(pct(100))
}

/// One live region's toasts in a column; a space above the assertive
/// group while both groups show toasts.
pub(crate) fn toast_group(t: &Theme, state: &Look<ToastGroupState>) -> Style {
    let s = style().gap(t.get(&kivact_core::style::space::SM_VAR));
    if state.count > 0 && state.above > 0 {
        s.margin_top(t.get(&kivact_core::style::space::SM_VAR))
    } else {
        s
    }
}

/// A toast on the raised surface, its edge in the tone's color.
pub(crate) fn toast(t: &Theme, state: &Look<ToastState>) -> Style {
    let edge: Color = match state.tone {
        Tone::Neutral | Tone::Info => kivact_core::style::color::BORDER,
        Tone::Success => kivact_core::style::color::SUCCESS,
        Tone::Warning => kivact_core::style::color::WARNING,
        Tone::Danger => kivact_core::style::color::DANGER,
        Tone::Accent => kivact_core::style::color::ACCENT,
    };
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Start)
        .gap(t.get(&kivact_core::style::space::MD_VAR))
        .padding(t.get(&kivact_core::style::space::LG_VAR))
        .background(kivact_core::style::color::RAISED)
        .border(1, kivact_core::style::color::BORDER)
        .border_left(3, edge)
        .border_radius(t.get(&kivact_core::style::radius::MD_VAR))
        .shadow(kivact_core::style::Shadow::new(
            0,
            4,
            12,
            kivact_core::style::color::SHADOW,
        ))
}

/// The tone's icon box: 16 px in the tone's color.
pub(crate) fn toast_icon(_: &Theme, state: &Look<ToastState>) -> Style {
    let color: Color = match state.tone {
        Tone::Neutral => kivact_core::style::color::TEXT_MUTED,
        Tone::Info | Tone::Accent => kivact_core::style::color::ACCENT,
        Tone::Success => kivact_core::style::color::SUCCESS,
        Tone::Warning => kivact_core::style::color::WARNING,
        Tone::Danger => kivact_core::style::color::DANGER,
    };
    style().size(16, 16).flex_shrink(0).color(color)
}

pub(crate) fn toast_title(t: &Theme, _: &Look<ToastState>) -> Style {
    style().font_weight(t.get(&kivact_core::style::font::WEIGHT_STRONG_VAR))
}

pub(crate) fn toast_body(t: &Theme, _: &Look<ToastState>) -> Style {
    style()
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
}

/// The close button, around its `x` icon.
pub(crate) fn toast_close(t: &Theme, _: &Look<ToastState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .flex_shrink(0)
        .size(24, 24)
        .border_radius(t.get(&kivact_core::style::radius::SM_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
        .hover(style().background(kivact_core::style::color::HOVER))
}

/// Shared by all kits: the row owns size presence independently of card paint.
pub(crate) fn toast_row(t: &Theme, _: &Look<PartState>) -> Style {
    let base = style()
        .flex_shrink(0)
        .enter_size(kivact_core::view::Orientation::Vertical)
        .exit_size(kivact_core::view::Orientation::Vertical);
    let row = t.ext::<ToastTheme>().row_transition_value().clone();
    if let Some(transition) = &row {
        base.padding_top(ROW_GAP)
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
    ToastTheme {
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
    settings {
        /// The transition that shows and collapses a toast row; `None` leaves the size unanimated.
        row_transition(row_transition_value): Option<kivact_core::motion::Transition> = None,
    }
}
