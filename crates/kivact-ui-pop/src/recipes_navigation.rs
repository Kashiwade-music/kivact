//! Role: Supply Pop navigation surfaces and the diamond progress-step appearance.
use crate::{Hue, Tokens};
use kivact::draw::{Layer, Transform};
use kivact::geometry::{Point, Rect};
use kivact::style::{Align, Color, Direction, Justify, Style, Theme, style};
use kivact::ui::{
    BreadcrumbRecipes, Look, NavigationMenuRecipes, PaginationRecipes, StepperItemState,
    StepperRecipes, StepperState, Variant,
};
use kivact::view::Orientation;

/// Diamond indicators above centered labels, with ruled progress connectors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DiamondSteps;
impl Variant for DiamondSteps {}

pub(crate) fn breadcrumb(r: BreadcrumbRecipes) -> BreadcrumbRecipes {
    r.root(|prev| move |t: &Theme, s: &_| prev(t, s).gap(6).align_items(Align::Center).wrap(true))
        .item(|prev| {
            move |t: &Theme, s: &_| {
                prev(t, s).weight(400).color(if **s {
                    t.ext::<Tokens>().text
                } else {
                    t.ext::<Tokens>().text_muted
                })
            }
        })
}
pub(crate) fn pagination(r: PaginationRecipes) -> PaginationRecipes {
    r.item(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .radius(8)
                .padding_x(10)
                .min_height(32)
                .hover(style().background(t.ext::<Tokens>().surface_active))
        }
    })
}
pub(crate) fn navigation_menu(r: NavigationMenuRecipes) -> NavigationMenuRecipes {
    r.trigger(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .padding_x(12)
                .padding_y(8)
                .radius(8)
                .hover(style().background(t.ext::<Tokens>().surface_active))
        }
    })
    .panel(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .radius(10)
                .border_color(t.ext::<Tokens>().line)
                .background(t.ext::<Tokens>().surface)
        }
    })
}
pub(crate) fn stepper(r: StepperRecipes) -> StepperRecipes {
    r.root(|prev| {
        move |t: &Theme, s: &Look<StepperState>| {
            let base = prev(t, s).color(t.ext::<Tokens>().text);
            if s.variant::<DiamondSteps>().is_none() {
                return base;
            }
            let state = **s;
            let (accent, line) = (t.ext::<Tokens>().blue, t.ext::<Tokens>().line);
            base.gap(0).paint(move |d, size| {
                if state.collapsed || state.orientation == Orientation::Vertical {
                    return;
                }
                let width = size.width / state.count.max(1) as f64;
                for index in 1..state.count {
                    d.line(
                        Point::new((index as f64 - 0.5) * width, 15.0),
                        Point::new((index as f64 + 0.5) * width, 15.0),
                        2.0,
                        if index <= state.current { accent } else { line },
                    );
                }
            })
        }
    })
    .item(|prev| {
        move |t: &Theme, s: &Look<StepperItemState>| {
            let base = prev(t, s);
            if s.variant::<DiamondSteps>().is_some() {
                base.grow(1).min_width(0).align_items(Align::Center)
            } else {
                base
            }
        }
    })
    .layout(|prev| {
        move |t: &Theme, s: &Look<StepperItemState>| {
            let base = prev(t, s);
            if s.variant::<DiamondSteps>().is_some() {
                base.direction(Direction::Column)
                    .align_items(Align::Center)
                    .gap(2)
            } else {
                base
            }
        }
    })
    .body(|prev| {
        move |t: &Theme, s: &Look<StepperItemState>| {
            let base = prev(t, s);
            if s.variant::<DiamondSteps>().is_some() {
                base.align_items(Align::Center).gap(2)
            } else {
                base
            }
        }
    })
    .indicator(|_| indicator)
    .label(|_| |_: &Theme, _: &Look<StepperItemState>| style().font_size(13).weight(500))
    .description(|_| {
        |t: &Theme, _: &Look<StepperItemState>| {
            style().font_size(12).color(t.ext::<Tokens>().text_muted)
        }
    })
}
fn indicator(t: &Theme, s: &Look<StepperItemState>) -> Style {
    let p = t.ext::<Tokens>();
    let complete = s.index < s.current;
    let active = s.index == s.current;
    let base = style()
        .size(30, 30)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .font_size(13)
        .weight(600)
        .margin_bottom(6)
        .color(if complete {
            Color::WHITE
        } else if active {
            p.blue
        } else {
            p.text_muted
        });
    if s.variant::<DiamondSteps>().is_none() {
        return base
            .radius(999)
            .background(if complete { p.blue } else { p.surface })
            .border(1, p.line);
    }
    let (fill, border, ring) = (
        if complete { p.blue } else { p.surface },
        if complete || active {
            p.blue
        } else {
            p.line_strong
        },
        p.hue(Hue::Blue).0,
    );
    base.paint(move |d, _| {
        d.layer(
            Layer::new().transform(
                Transform::new()
                    .translate(15, 15)
                    .rotate(45)
                    .translate(-15, -15),
            ),
            |d| {
                if active {
                    d.rounded_rect(Rect::new(-4.0, -4.0, 38.0, 38.0), 10.0, ring);
                }
                d.rounded_rect(Rect::new(0.0, 0.0, 30.0, 30.0), 6.0, border);
                d.rounded_rect(Rect::new(1.0, 1.0, 28.0, 28.0), 5.0, fill);
            },
        );
    })
}
