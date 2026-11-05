//! Role: Basic visual variants and recipes for standard alert callouts.
use crate::Tokens;
use kivact::style::{Align, ColorToken, Display, Position, Style, Theme, Track, auto, pct, style};
use kivact::ui::{AlertRecipes, AlertState, Look};
/// Colors of a standard alert callout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AlertVariant {
    /// Card colors.
    #[default]
    Default,
    /// Destructive text and icon.
    Destructive,
}
impl kivact::ui::Variant for AlertVariant {}
pub(crate) fn recipes(r: AlertRecipes) -> AlertRecipes {
    r.root(|_| root)
        .title(|_| title)
        .description(|_| description)
        .icon(|_| icon)
        .actions(|_| actions)
        .content(|_| content)
}
fn destructive(s: &Look<AlertState>) -> bool {
    s.variant::<AlertVariant>() == Some(&AlertVariant::Destructive)
}
fn root(t: &Theme, s: &Look<AlertState>) -> Style {
    let p = t.ext::<Tokens>();
    let columns = if s.has_icon {
        vec![Track::from(auto()), Track::minmax(0, Track::fr(1))]
    } else {
        vec![Track::minmax(0, Track::fr(1))]
    };
    let color = if destructive(s) {
        p.destructive
    } else {
        match s.tone {
            kivact::interaction::Tone::Neutral => p.card_foreground,
            kivact::interaction::Tone::Info | kivact::interaction::Tone::Accent => {
                ColorToken::Accent.into()
            }
            kivact::interaction::Tone::Success => ColorToken::Success.into(),
            kivact::interaction::Tone::Warning => ColorToken::Warning.into(),
            kivact::interaction::Tone::Danger => p.destructive,
        }
    };
    style()
        .display(Display::Grid)
        .grid_template_columns(columns)
        .align_items(Align::Start)
        .position(Position::Relative)
        .width(pct(100))
        .gap(2)
        .padding_left(10)
        .padding_y(8)
        .padding_right(if s.has_actions { 72.0 } else { 10.0 })
        .border(1, p.border)
        .radius(0)
        .background(p.card)
        .font_size(12)
        .color(color)
}
fn title(_: &Theme, s: &Look<AlertState>) -> Style {
    style()
        .weight(500)
        .grid_row(1)
        .grid_column(if s.has_icon { 2 } else { 1 })
}
fn description(t: &Theme, s: &Look<AlertState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .line_height(1.625)
        .color(if destructive(s) {
            p.destructive.with_alpha(0.9)
        } else {
            p.muted_foreground
        })
        .grid_row(2)
        .grid_column(if s.has_icon { 2 } else { 1 })
}
fn icon(_: &Theme, s: &Look<AlertState>) -> Style {
    style()
        .shrink(0)
        .margin_right(6)
        .grid_column(1)
        .grid_row((1, if s.has_description { 3 } else { 2 }))
}
fn actions(_: &Theme, _: &Look<AlertState>) -> Style {
    style()
        .position(Position::Absolute)
        .top(0)
        .bottom(0)
        .right(5)
        .align_items(Align::Center)
}
fn content(_: &Theme, _: &Look<AlertState>) -> Style {
    style().grid_column((1, -1)).grid_row(4)
}
