//! Role: Creative appearance of canonical alerts, empty states, cards and input groups.
use crate::feedback::ToneExt;
use crate::{NoticeVariant, Tokens};
use kivact::style::{Position, Theme};
use kivact::ui::{AlertRecipes, CardRecipes, EmptyStateRecipes, InputGroupRecipes};

pub(crate) fn alert(r: AlertRecipes) -> AlertRecipes {
    r.root(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            let color = s.tone.color(&p);
            let base = prev(t, s)
                .padding(12)
                .gap(3)
                .radius(9)
                .color(p.text)
                .background(color.with_alpha(0.08))
                .border(1, color.with_alpha(0.22));
            if s.variant::<NoticeVariant>().is_some() {
                base.position(Position::Relative)
                    .padding_left(38)
                    .padding_right(44)
            } else {
                base
            }
        }
    })
    .title(|prev| move |t: &Theme, s: &_| prev(t, s).weight(600).font_size(12.5))
    .description(|prev| {
        move |t: &Theme, s: &_| prev(t, s).font_size(11.5).color(t.ext::<Tokens>().text)
    })
    .icon(|prev| {
        move |t: &Theme, s: &_| {
            let base = prev(t, s).size(16, 16);
            if s.variant::<NoticeVariant>().is_some() {
                base.position(Position::Absolute).left(12).top(12)
            } else {
                base
            }
        }
    })
    .dismiss(|prev| {
        move |t: &Theme, s: &_| {
            if s.variant::<NoticeVariant>().is_some() {
                crate::controls::button_style(t)
                    .position(Position::Absolute)
                    .right(12)
                    .top(12)
                    .size(t.ext::<Tokens>().row, t.ext::<Tokens>().row)
                    .padding(0)
            } else {
                prev(t, s)
            }
        }
    })
    .actions(|prev| move |t: &Theme, s: &_| prev(t, s).gap(6))
    .content(|prev| move |t: &Theme, s: &_| prev(t, s).gap(6).font_size(11.5))
}

pub(crate) fn empty_state(r: EmptyStateRecipes) -> EmptyStateRecipes {
    r.root(|prev| move |t: &Theme, s: &_| prev(t, s).padding(24).gap(8))
        .title(|prev| {
            move |t: &Theme, s: &_| {
                prev(t, s)
                    .font_size(14)
                    .weight(600)
                    .color(t.ext::<Tokens>().text_high)
            }
        })
        .description(|prev| {
            move |t: &Theme, s: &_| prev(t, s).font_size(11.5).color(t.ext::<Tokens>().muted)
        })
        .icon(|prev| move |t: &Theme, s: &_| prev(t, s).color(t.ext::<Tokens>().dim))
        .actions(|prev| move |t: &Theme, s: &_| prev(t, s).gap(6))
}

pub(crate) fn card(r: CardRecipes) -> CardRecipes {
    r.root(|prev| {
        move |t: &Theme, s: &_| {
            let p = t.ext::<Tokens>();
            prev(t, s)
                .radius(9)
                .background(p.island)
                .color(p.text)
                .border(
                    1,
                    if s.selected == Some(true) {
                        p.accent
                    } else {
                        p.hair
                    },
                )
        }
    })
    .header(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .padding(10)
                .gap(6)
                .background(t.ext::<Tokens>().header)
        }
    })
    .content(|prev| move |t: &Theme, s: &_| prev(t, s).padding(10).gap(6))
    .footer(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .padding(10)
                .gap(6)
                .border_top(1, t.ext::<Tokens>().hair)
        }
    })
    .media(|prev| move |t: &Theme, s: &_| prev(t, s).clip(true))
}

pub(crate) fn input_group(r: InputGroupRecipes) -> InputGroupRecipes {
    r.root(|prev| move |t: &Theme, s: &_| prev(t, s).gap(4))
        .label(|prev| {
            move |t: &Theme, s: &_| prev(t, s).font_size(11.5).color(t.ext::<Tokens>().muted)
        })
        .members(|prev| move |t: &Theme, s: &_| prev(t, s).min_width(0))
        .member(|prev| move |t: &Theme, s: &_| prev(t, s))
        .addon(|prev| {
            move |t: &Theme, s: &_| {
                let p = t.ext::<Tokens>();
                prev(t, s)
                    .background(p.header)
                    .color(p.muted)
                    .border_color(p.hair)
                    .font_size(11.5)
            }
        })
}
