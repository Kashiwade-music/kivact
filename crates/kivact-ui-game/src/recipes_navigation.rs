//! Role: Draw console navigation with numbered chevrons and compact page controls.
use crate::Tokens;
use kivact::draw::Shape;
use kivact::style::{Align, ColorToken, pct, style};
use kivact::ui::{BreadcrumbRecipes, NavigationMenuRecipes, PaginationRecipes, StepperRecipes};

pub(crate) fn pagination(r: PaginationRecipes) -> PaginationRecipes {
    r.root(|base| move |t, s| base(t, s).gap(4).padding(0))
        .item(|_| {
            move |t, s| {
                style()
                    .size(30, 30)
                    .align_items(Align::Center)
                    .justify_content(kivact::style::Justify::Center)
                    .border(1, ColorToken::Border)
                    .color(if **s {
                        ColorToken::AccentText
                    } else {
                        ColorToken::TextMuted
                    })
                    .background(if **s {
                        t.colors().accent
                    } else {
                        kivact::style::Color::TRANSPARENT
                    })
                    .hover(
                        style()
                            .background(ColorToken::SurfaceAlt)
                            .color(ColorToken::Text),
                    )
                    .disabled(style().opacity(0.42))
            }
        })
        .ellipsis(|base| move |t, s| base(t, s).font_family("monospace"))
}
pub(crate) fn breadcrumb(r: BreadcrumbRecipes) -> BreadcrumbRecipes {
    r.root(|base| move |t, s| base(t, s).gap(8).padding(0).align_items(Align::Center))
        .item(|_| {
            move |_, s| {
                style()
                    .font_size(12)
                    .weight(if **s { 700 } else { 400 })
                    .color(if **s {
                        ColorToken::Text
                    } else {
                        ColorToken::TextMuted
                    })
            }
        })
        .separator(|_| move |_, _| style().font_size(20).weight(700).color(ColorToken::Accent))
}
pub(crate) fn stepper(r: StepperRecipes) -> StepperRecipes {
    r.root(|base| move |t, s| base(t, s).width(pct(100)).gap(3).padding(0))
        .item(|base| {
            move |t, s| {
                let p = t.ext::<Tokens>();
                base(t, s)
                    .height(40)
                    .padding_left(if s.index == 0 { 14 } else { 22 })
                    .padding_right(18)
                    .gap(8)
                    .grow(1)
                    .min_width(0)
                    .clip_path(
                        Shape::polygon()
                            .point(0, 0)
                            .point(pct(100) - 12, 0)
                            .point(pct(100), pct(50))
                            .point(pct(100) - 12, pct(100))
                            .point(0, pct(100))
                            .point(if s.index == 0 { 0 } else { 12 }, pct(50)),
                    )
                    .background(if s.index == s.current {
                        t.colors().accent
                    } else if s.index < s.current {
                        p.primary_weak
                    } else {
                        p.surface_3
                    })
                    .color(if s.index == s.current {
                        t.colors().accent_text
                    } else if s.index < s.current {
                        t.colors().accent
                    } else {
                        p.text_3
                    })
            }
        })
        .indicator(|base| {
            move |t, s| {
                base(t, s)
                    .font_family(t.ext::<Tokens>().display_family.as_str())
                    .font_size(18)
                    .weight(800)
                    .italic(true)
            }
        })
        .label(|base| move |t, s| base(t, s).font_size(12).weight(700))
}
pub(crate) fn navigation_menu(r: NavigationMenuRecipes) -> NavigationMenuRecipes {
    r.root(|base| move |t, s| base(t, s).gap(2)).panel(|base| {
        move |t, s| {
            base(t, s)
                .radius(0)
                .border_color(t.ext::<Tokens>().line_strong)
        }
    })
}
