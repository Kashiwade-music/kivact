//! Role: The alert recipe state, tokens, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::variants::Look;
use kivact_core::style::{Align, ColorToken, Direction, Style, Theme, pct, style};
use kivact_interaction::Tone;
pub(crate) fn alert_root(t: &Theme, s: &Look<AlertState>) -> Style {
    let color = match s.tone {
        Tone::Neutral => ColorToken::Border,
        Tone::Info | Tone::Accent => ColorToken::Accent,
        Tone::Success => ColorToken::Success,
        Tone::Warning => ColorToken::Warning,
        Tone::Danger => ColorToken::Danger,
    };
    let base = style()
        .direction(Direction::Column)
        .gap(t.space().sm)
        .padding(t.space().md)
        .radius(t.radius().md)
        .border(1, color)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
        .min_width(0);
    if s.variant::<AlertVariant>() == Some(&AlertVariant::Banner) {
        base.width(pct(100))
    } else {
        base
    }
}

pub(crate) fn alert_title(t: &Theme, _: &Look<AlertState>) -> Style {
    style()
        .weight(t.text().weight_strong)
        .font_size(t.text().size)
}

pub(crate) fn alert_description(t: &Theme, _: &Look<AlertState>) -> Style {
    style()
        .font_size(t.text().size)
        .color(ColorToken::TextMuted)
}

pub(crate) fn alert_icon(_: &Theme, _: &Look<AlertState>) -> Style {
    style().shrink(0)
}

pub(crate) fn alert_actions(t: &Theme, _: &Look<AlertState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().sm)
}

pub(crate) fn alert_content(t: &Theme, _: &Look<AlertState>) -> Style {
    style().direction(Direction::Column).gap(t.space().sm)
}

pub(crate) fn alert_control(_: &Theme, _: &Look<AlertState>) -> Style {
    style()
}

pub(crate) fn alert_wrapper(_: &Theme, _: &Look<AlertState>) -> Style {
    style().display(kivact_core::style::Display::Contents)
}

/// Alert meaning and disclosure state supplied to recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlertState {
    pub tone: Tone,
    /// The content slot ([`Alert::content`](super::Alert::content)): `None`
    /// without content, otherwise whether it can collapse and is open.
    pub content: Option<Expansion>,
    pub dismissible: bool,
    pub has_icon: bool,
    pub has_actions: bool,
    pub has_description: bool,
}

impl AlertState {
    /// Whether the alert has a content slot ([`Alert::content`](super::Alert::content)),
    /// collapsible or not.
    pub fn has_content(&self) -> bool {
        self.content.is_some()
    }
    /// Whether the content can collapse and, if so, whether it is open;
    /// [`NotExpandable`](Expansion::NotExpandable) without content or when
    /// it cannot collapse.
    pub fn expansion(&self) -> Expansion {
        self.content.unwrap_or(Expansion::NotExpandable)
    }
}

/// Whether a part can expand and, if so, whether it is expanded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expansion {
    /// The part has no expandable content.
    NotExpandable,
    /// The part has content and shows it closed.
    Collapsed,
    /// The part shows its content.
    Expanded,
}

impl Expansion {
    /// The expansion of a part which may have content, open or closed.
    pub fn new(expandable: bool, open: bool) -> Self {
        match (expandable, open) {
            (false, _) => Self::NotExpandable,
            (true, false) => Self::Collapsed,
            (true, true) => Self::Expanded,
        }
    }
}

recipe_table! {
    /// Recipes for a persistent alert and optional details.
    AlertRecipes {
        root(root_recipe): AlertState = alert_root,
        header(header_recipe): AlertState = alert_wrapper,
        message(message_recipe): AlertState = alert_wrapper,
        controls(controls_recipe): AlertState = alert_wrapper,
        title(title_recipe): AlertState = alert_title,
        description(description_recipe): AlertState = alert_description,
        icon(icon_recipe): AlertState = alert_icon,
        actions(actions_recipe): AlertState = alert_actions,
        content(content_recipe): AlertState = alert_content,
        dismiss(dismiss_recipe): AlertState = alert_control,
        toggle(toggle_recipe): AlertState = alert_control,
    }
}

/// The structural presentation of an alert.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertVariant {
    #[default]
    Default,
    Banner,
}

impl crate::variants::Variant for AlertVariant {}
