//! Role: The card recipe state, recipe table and default looks.

use crate::recipe::Look;
use crate::recipe::recipe_table;
use kivact_core::style::{FlexDirection, Style, Theme, style};
pub(crate) fn card_root(t: &Theme, s: &Look<CardState>) -> Style {
    let color = if s.selection() == Selectable::Selected {
        kivact_core::style::color::ACCENT
    } else {
        kivact_core::style::color::BORDER
    };
    let base = style()
        .flex_direction(FlexDirection::Column)
        .min_width(0)
        .border_radius(t.get(&kivact_core::style::radius::MD_VAR))
        .border(1, color)
        .background(kivact_core::style::color::SURFACE)
        .color(kivact_core::style::color::TEXT);
    if s.is_interactive() {
        base.focus_visible(
            style()
                .outline(t.focus_ring(), kivact_core::style::color::FOCUS)
                .outline_offset(t.focus_ring_offset()),
        )
        .hover(style().background(kivact_core::style::color::SURFACE_ALT))
        .active(style().background(kivact_core::style::color::PRESSED))
        .disabled(style().opacity(t.get(&kivact_core::style::control::DISABLED_OPACITY_VAR)))
    } else {
        base
    }
}

pub(crate) fn card_region(t: &Theme, _: &Look<CardState>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .gap(t.get(&kivact_core::style::space::SM_VAR))
        .padding(t.get(&kivact_core::style::space::MD_VAR))
        .min_width(0)
}

pub(crate) fn card_media(_: &Theme, _: &Look<CardState>) -> Style {
    style().min_width(0)
}

pub(crate) fn card_overlay(_: &Theme, _: &Look<CardState>) -> Style {
    style()
}

/// Card semantics supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardState {
    /// What activating the card does; `match` it, or read
    /// [`input`](Self::input), [`is_interactive`](Self::is_interactive) and
    /// [`selection`](Self::selection).
    pub action: CardAction,
    pub disabled: bool,
    pub has_media: bool,
    pub has_footer: bool,
}

impl CardState {
    /// The actual input state of the card's button or check box part;
    /// `None` for a plain card.
    pub fn input(&self) -> Option<kivact_core::custom::PartState> {
        match self.action {
            CardAction::Plain => None,
            CardAction::Press { input } | CardAction::Select { input, .. } => Some(input),
        }
    }
    /// Whether the card can be pressed or selected.
    pub fn is_interactive(&self) -> bool {
        !matches!(self.action, CardAction::Plain)
    }
    /// Whether the card can be selected and, if so, whether it is.
    pub fn selection(&self) -> Selectable {
        match self.action {
            CardAction::Select { selected, .. } => Selectable::new(true, selected),
            CardAction::Plain | CardAction::Press { .. } => Selectable::NotSelectable,
        }
    }
}

/// Whether a part can be selected and, if so, whether it is selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selectable {
    /// The part has no selected state.
    NotSelectable,
    /// The part can be selected and is not.
    Unselected,
    /// The part is selected.
    Selected,
}

impl Selectable {
    /// The selection of a part which may be selectable, selected or not.
    pub fn new(selectable: bool, selected: bool) -> Self {
        match (selectable, selected) {
            (false, _) => Self::NotSelectable,
            (true, false) => Self::Unselected,
            (true, true) => Self::Selected,
        }
    }
}

recipe_table! {
    /// Recipes for each region of a card.
    CardTheme {
        root(root_recipe): CardState = card_root,
        header(header_recipe): CardState = card_region,
        media(media_recipe): CardState = card_media,
        content(content_recipe): CardState = card_region,
        footer(footer_recipe): CardState = card_region,
        overlay(overlay_recipe): CardState = card_overlay,
    }
}

/// What activating a card does, with the actual input of the part that does it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CardAction {
    /// A plain surface; nothing activates it.
    Plain,
    /// Pressing activates it ([`Card::on_press`](super::Card::on_press)).
    Press {
        /// The actual input state of the card's button part.
        input: kivact_core::custom::PartState,
    },
    /// It is a check box ([`Card::selected`](super::Card::selected)).
    Select {
        /// The actual input state of the card's check box part.
        input: kivact_core::custom::PartState,
        /// Whether the card is selected.
        selected: bool,
    },
}
