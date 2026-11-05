//! Role: Apply Pop metric cards, heading typography and term-detail rows.
use crate::{Hue, Tokens};
use kivact::draw::Shadow;
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, ColorToken, Direction, Display, Theme, Track, style};
use kivact::ui::{
    DescriptionListRecipes, HeadingState, Look, StatRecipes, StatState, Trend, Variant,
};

/// Reactive hue of the application-formatted statistic delta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatHue(pub Value<Hue>);
impl StatHue {
    pub fn new(hue: impl IntoValue<Hue>) -> Self {
        Self(hue.into_value())
    }
}
impl Variant for StatHue {}
/// A skewed section-number badge beside a standard semantic heading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SectionNumber;
impl Variant for SectionNumber {}

pub(crate) fn heading(_: &Theme, s: &Look<HeadingState>) -> kivact::style::Style {
    style()
        .font_size(match s.level {
            1 => 28,
            2 => 20,
            3 => 18,
            4 => 16,
            _ => 14,
        })
        .weight(600)
        .line_height(1.2)
        .color(ColorToken::Text)
}
pub(crate) fn stat(r: StatRecipes) -> StatRecipes {
    r.root(|_| {
        |t: &Theme, _: &Look<StatState>| {
            let p = t.ext::<Tokens>();
            style()
                .display(Display::Grid)
                .grid_template_columns((Track::fr(1), Track::fr(1)))
                .gap(4)
                .padding_top(20)
                .min_height(185)
                .clip(true)
                .background(p.surface)
                .border(1, p.line)
                .radius(p.panel_radius)
                .shadow(Shadow::new(0, 1, 3, ColorToken::Shadow))
        }
    })
    .icon(|_| {
        |t: &Theme, _: &Look<StatState>| {
            style()
                .grid_column(1)
                .grid_row(1)
                .margin_left(20)
                .margin_bottom(8)
                .size(32, 32)
                .radius(8)
                .background(t.ext::<Tokens>().hue(Hue::Blue).0)
                .color(t.ext::<Tokens>().text_muted)
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(kivact::style::Justify::Center)
        }
    })
    .label(|_| {
        |_: &Theme, _: &Look<StatState>| {
            style()
                .grid_column((1, 3))
                .grid_row(2)
                .margin_x(20)
                .font_size(13)
                .weight(500)
                .color(ColorToken::TextMuted)
        }
    })
    .value(|_| {
        |_: &Theme, _: &Look<StatState>| {
            style()
                .grid_column((1, 3))
                .grid_row(3)
                .margin_x(20)
                .font_size(26)
                .weight(600)
                .line_height(1.2)
        }
    })
    .delta(|_| {
        |t: &Theme, s: &Look<StatState>| {
            let hue = s.variant::<StatHue>().map_or(
                match s.trend {
                    Trend::Up => Hue::Mint,
                    Trend::Down => Hue::Red,
                    Trend::Neutral => Hue::Gray,
                },
                |h| h.0.get(),
            );
            style()
                .grid_column(2)
                .grid_row(1)
                .margin_right(20)
                .align_self(Align::Center)
                .justify_self(Align::End)
                .font_size(11)
                .padding_x(6)
                .padding_y(1)
                .radius(4)
                .border(1, ColorToken::Border)
                .background(ColorToken::Surface)
                .color(t.ext::<Tokens>().hue(hue).1)
        }
    })
    .chart(|_| {
        |_: &Theme, _: &Look<StatState>| {
            style()
                .grid_column((1, 3))
                .grid_row(4)
                .margin_top(8)
                .min_width(1)
        }
    })
}
pub(crate) fn description_list(r: DescriptionListRecipes) -> DescriptionListRecipes {
    r.root(|prev| move |t: &Theme, s: &_| prev(t, s).gap(0))
        .item(|prev| {
            move |t: &Theme, s: &_| {
                prev(t, s)
                    .gap(12)
                    .align_items(Align::Center)
                    .min_height(44)
                    .border_bottom(1, t.ext::<Tokens>().line)
            }
        })
        .term(|_| {
            |t: &Theme, _: &_| {
                style()
                    .width(110)
                    .font_size(13)
                    .color(t.ext::<Tokens>().text_muted)
            }
        })
        .detail(|prev| {
            move |t: &Theme, s: &_| {
                prev(t, s)
                    .direction(Direction::Row)
                    .align_items(Align::Center)
            }
        })
}
