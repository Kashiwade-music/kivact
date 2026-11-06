//! Role: Creative typography and read-only description styling for canonical metrics.
use crate::Tokens;
use kivact::style::Theme;
use kivact::ui::{DescriptionListRecipes, HeadingState, Look, StatRecipes, UiRecipes};

pub(crate) fn install(r: UiRecipes) -> UiRecipes {
    r.heading(|prev| {
        move |t: &Theme, s: &Look<HeadingState>| {
            prev(t, s).weight(600).color(t.ext::<Tokens>().text_high)
        }
    })
    .stat(stat)
    .description_list(description_list)
}

fn stat(r: StatRecipes) -> StatRecipes {
    r.root(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            prev(t, s)
                .padding(12)
                .gap(4)
                .radius(9)
                .background(p.island)
                .border(1, p.hair)
        }
    })
    .label(|prev| move |t: &Theme, s: &_| prev(t, s).font_size(10.5).color(t.ext::<Tokens>().muted))
    .value(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            prev(t, s)
                .font_family(p.mono_font.as_str())
                .font_size(24)
                .weight(600)
                .color(p.text_high)
        }
    })
    .delta(|prev| move |t: &Theme, s: &_| prev(t, s).font_size(10.5).color(t.ext::<Tokens>().muted))
    .icon(|prev| move |t: &Theme, s: &_| prev(t, s).color(t.ext::<Tokens>().dim))
    .chart(|prev| move |t: &Theme, s: &_| prev(t, s).min_width(0))
}

fn description_list(r: DescriptionListRecipes) -> DescriptionListRecipes {
    r.root(|prev| move |t: &Theme, s: &_| prev(t, s).gap(4))
        .item(|prev| move |t: &Theme, s: &_| prev(t, s).gap(8).min_height(t.ext::<Tokens>().row))
        .term(|prev| {
            move |t: &Theme, s: &_| prev(t, s).font_size(11.5).color(t.ext::<Tokens>().muted)
        })
        .detail(|prev| {
            move |t: &Theme, s: &_| prev(t, s).font_size(11.5).color(t.ext::<Tokens>().text)
        })
}
