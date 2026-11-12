//! Role: Style semantic statistics, headings and description rows as console displays.
use crate::Tokens;
use crate::display::ToneExt;
use kivact::style::{Align, ColorToken, Direction, Style, Theme, pct, style};
use kivact::ui::{DescriptionListRecipes, HeadingState, Look, StatRecipes};

/// A console tile for an application-formatted statistic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatVariant {
    Kpi,
}
impl kivact::ui::Variant for StatVariant {}
/// Display typography for a numbered section's title.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingVariant {
    Section,
}
impl kivact::ui::Variant for HeadingVariant {}

pub(crate) fn stat(r: StatRecipes) -> StatRecipes {
    r.root(|base| {
        move |t, s| {
            let kpi = s.variant::<StatVariant>() == Some(&StatVariant::Kpi);
            let tone = s
                .variant::<kivact::interaction::Tone>()
                .copied()
                .unwrap_or_default();
            base(t, s)
                .display(kivact::style::Display::Grid)
                .grid_template_columns((kivact::style::Track::fr(1), kivact::style::auto()))
                .padding_x(16)
                .padding_top(14)
                .padding_bottom(12)
                .gap(6)
                .background(t.colors().surface)
                .border(1, t.colors().border)
                .border_top(if kpi { 3 } else { 1 }, tone.color(t))
        }
    })
    .label(|_| {
        move |t, _| {
            style()
                .grid_column((1, 3))
                .grid_row(1)
                .font_family(t.ext::<Tokens>().mono_family.as_str())
                .font_size(11.5)
                .color(ColorToken::TextMuted)
        }
    })
    .value(|_| {
        move |t, _| {
            style()
                .grid_column((1, 3))
                .grid_row(2)
                .font_family(t.ext::<Tokens>().display_family.as_str())
                .font_size(36)
                .weight(700)
                .italic(true)
                .line_height(1.)
        }
    })
    .delta(|_| {
        move |t, s| {
            style()
                .grid_column(1)
                .grid_row(3)
                .align_self(Align::Center)
                .font_family(t.ext::<Tokens>().mono_family.as_str())
                .font_size(11.5)
                .color(
                    s.variant::<kivact::interaction::Tone>()
                        .copied()
                        .unwrap_or_default()
                        .color(t),
                )
        }
    })
    .chart(|_| move |_, _| style().grid_column(2).grid_row(3).align_self(Align::Center))
}
/// The console's trend chart: a 96 by 28 box.
pub(crate) fn sparkline(_: &Theme, _: &Look<()>) -> Style {
    style().size(96, 28)
}
pub(crate) fn heading(t: &Theme, s: &Look<HeadingState>) -> Style {
    style()
        .font_family(t.ext::<Tokens>().display_family.as_str())
        .font_size(if s.variant::<HeadingVariant>().is_some() {
            24.
        } else {
            t.text().size_large
        })
        .line_height(1.)
        .letter_spacing(0.96)
        .weight(700)
        .italic(true)
}
pub(crate) fn description_list(r: DescriptionListRecipes) -> DescriptionListRecipes {
    r.root(|_| {
        move |_, _| {
            style()
                .direction(Direction::Column)
                .width(pct(100))
                .border_top(1, ColorToken::Border)
        }
    })
    .item(|_| {
        move |_, _| {
            style()
                .direction(Direction::Row)
                .border_bottom(1, ColorToken::Border)
        }
    })
    .term(|_| {
        move |t, _| {
            style()
                .width(120)
                .padding_x(10)
                .padding_y(8)
                .font_family(t.ext::<Tokens>().mono_family.as_str())
                .font_size(11.5)
                .background(ColorToken::SurfaceAlt)
        }
    })
    .detail(|_| move |_, _| style().grow(1).padding_x(10).padding_y(8).font_size(12.5))
}
