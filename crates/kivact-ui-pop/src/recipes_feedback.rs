//! Role: Style canonical feedback, card surfaces and grouped inputs with Pop tokens.
use crate::{Hue, Tokens};
use kivact::draw::Shadow;
use kivact::interaction::Tone;
use kivact::style::{Align, ColorToken, Direction, Display, Length, Theme, Track, style};
use kivact::ui::{
    AlertRecipes, AlertState, CardRecipes, EmptyStateRecipes, InputGroupRecipes, Look,
};

pub(crate) fn card(r: CardRecipes) -> CardRecipes {
    r.root(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            prev(t, s)
                .padding(p.panel_padding)
                .background(p.surface)
                .radius(p.panel_radius)
                .shadow(Shadow::new(0, 1, 3, ColorToken::Shadow))
        }
    })
    .content(|_| |_: &Theme, _: &_| style().direction(Direction::Column).padding(0).gap(0))
}
pub(crate) fn empty_state(r: EmptyStateRecipes) -> EmptyStateRecipes {
    r.root(|_| {
        |_: &Theme, _: &_| {
            style()
                .direction(Direction::Column)
                .padding_x(12)
                .padding_top(24)
                .padding_bottom(12)
                .gap(6)
                .align_items(Align::Center)
        }
    })
    .title(|_| |_: &Theme, _: &_| style().font_size(16).weight(600))
    .description(|_| |_: &Theme, _: &_| style().color(ColorToken::TextMuted).margin_bottom(12))
}
fn hue(tone: Tone) -> Hue {
    match tone {
        Tone::Neutral => Hue::Gray,
        Tone::Info => Hue::Blue,
        Tone::Success => Hue::Mint,
        Tone::Warning => Hue::Yellow,
        Tone::Danger => Hue::Red,
        Tone::Accent => Hue::Pink,
    }
}
pub(crate) fn alert(r: AlertRecipes) -> AlertRecipes {
    r.root(|prev| {
        move |t: &Theme, s: &Look<AlertState>| {
            let p = t.ext::<Tokens>();
            let hue = hue(s.tone);
            prev(t, s)
                .display(Display::Grid)
                .grid_template_columns((20, Track::fr(1), Length::Auto))
                .padding_x(16)
                .padding_y(12)
                .gap(12)
                .radius(10)
                .background(p.surface)
                .border(
                    1,
                    if hue == Hue::Red {
                        p.hue(hue).1.mix_in(p.line, 0.7, t)
                    } else {
                        p.line
                    },
                )
        }
    })
    .title(|_| {
        |t: &Theme, s: &Look<AlertState>| {
            let p = t.ext::<Tokens>();
            style()
                .grid_column(2)
                .grid_row(1)
                .font_size(14)
                .weight(500)
                .line_height(1.5)
                .color(if s.tone == Tone::Danger {
                    p.hue(Hue::Red).1
                } else {
                    p.text
                })
        }
    })
    .description(|_| {
        |_: &Theme, _: &_| {
            style()
                .grid_column(2)
                .grid_row(2)
                .margin_top(-10)
                .font_size(13)
                .line_height(1.5)
                .color(ColorToken::TextMuted)
        }
    })
    .icon(|_| {
        |t: &Theme, s: &Look<AlertState>| {
            style()
                .grid_column(1)
                .grid_row((1, 3))
                .align_self(if s.tone == Tone::Danger {
                    Align::Center
                } else {
                    Align::Start
                })
                .color(t.ext::<Tokens>().hue(hue(s.tone)).1)
                .margin_top(2)
        }
    })
    .actions(|prev| move |t: &Theme, s: &_| prev(t, s).grid_column(3).grid_row((1, 3)))
    .content(|prev| move |t: &Theme, s: &_| prev(t, s).grid_column((1, 4)).grid_row(3))
}
pub(crate) fn input_group(r: InputGroupRecipes) -> InputGroupRecipes {
    r.label(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .font_size(13)
                .weight(500)
                .color(t.ext::<Tokens>().text)
        }
    })
    .addon(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .background(t.ext::<Tokens>().surface_muted)
                .border_color(t.ext::<Tokens>().line)
                .radius(8)
        }
    })
}
