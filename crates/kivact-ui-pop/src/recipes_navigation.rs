//! Role: The Pop looks of the navigation: breadcrumbs, pagination, navigation menus, steppers and tabs.
//!
//! The diamond progress steps of a stepper are the [`DiamondSteps`](crate::DiamondSteps) variant.

use crate::color;
use crate::recipe_parts::{choice_list, choice_option};
use crate::variants::{DiamondSteps, Hue};
use kivact::draw::Layer;
use kivact::geometry::{Point, Rect};
use kivact::interaction::NavOpen;
use kivact::interaction::{TabListState, TabState};
use kivact::style::Transform;
use kivact::style::{Align, Color, FlexDirection, Justify, Style, Theme, style};
use kivact::ui::theme::{
    BreadcrumbTheme, Look, NavigationMenuTheme, PaginationTheme, StepperItemState, StepperState,
    StepperTheme, TabsTheme,
};
use kivact::view::Orientation;

/// Install the kit's recipes of the navigation.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme
        .with(
            BreadcrumbTheme::default()
                .root(breadcrumb_root)
                .item(breadcrumb_item),
        )
        .with(PaginationTheme::default().item(pagination_item))
        .with(
            NavigationMenuTheme::default()
                .trigger(navigation_trigger)
                .panel(navigation_panel),
        )
        .with(
            StepperTheme::default()
                .root(stepper_root)
                .item(stepper_item)
                .layout(stepper_layout)
                .body(stepper_body)
                .indicator(stepper_indicator)
                .label(stepper_label)
                .description(stepper_description),
        );
    theme.with(
        TabsTheme::default()
            .list(tabs_list)
            .tab(tab)
            .panel(tab_panel),
    )
}

fn breadcrumb_root(_theme: &Theme, _: &Look<()>, base: Style) -> Style {
    base.gap(6).align_items(Align::Center).flex_wrap(true)
}

fn breadcrumb_item(
    _theme: &Theme,
    s: &Look<kivact::ui::theme::BreadcrumbItemState>,
    base: Style,
) -> Style {
    base.font_weight(400).color(if s.is_current() {
        color::TEXT
    } else {
        color::TEXT_MUTED
    })
}

fn pagination_item(
    _theme: &Theme,
    _: &Look<kivact::ui::theme::PaginationItemState>,
    base: Style,
) -> Style {
    base.border_radius(8)
        .padding_x(10)
        .min_height(32)
        .hover(style().background(color::SURFACE_ACTIVE))
}

fn navigation_trigger(
    _theme: &Theme,
    _: &Look<kivact::ui::theme::NavigationTriggerState>,
    base: Style,
) -> Style {
    base.padding_x(12)
        .padding_y(8)
        .border_radius(8)
        .hover(style().background(color::SURFACE_ACTIVE))
}

fn navigation_panel(_theme: &Theme, _: &Look<NavOpen>, base: Style) -> Style {
    base.border_radius(10)
        .border_color(color::LINE)
        .background(color::SURFACE)
}

fn stepper_root(_theme: &Theme, s: &Look<StepperState>, base: Style) -> Style {
    let base = base.color(color::TEXT);
    if s.variant::<DiamondSteps>().is_none() {
        return base;
    }
    let state = **s;
    let (accent, line) = (color::BLUE, color::LINE);
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

fn stepper_item(_theme: &Theme, s: &Look<StepperItemState>, base: Style) -> Style {
    if s.variant::<DiamondSteps>().is_some() {
        base.flex_grow(1).min_width(0).align_items(Align::Center)
    } else {
        base
    }
}

fn stepper_layout(_theme: &Theme, s: &Look<StepperItemState>, base: Style) -> Style {
    if s.variant::<DiamondSteps>().is_some() {
        base.flex_direction(FlexDirection::Column)
            .align_items(Align::Center)
            .gap(2)
    } else {
        base
    }
}

fn stepper_body(_theme: &Theme, s: &Look<StepperItemState>, base: Style) -> Style {
    if s.variant::<DiamondSteps>().is_some() {
        base.align_items(Align::Center).gap(2)
    } else {
        base
    }
}

fn stepper_indicator(_theme: &Theme, s: &Look<StepperItemState>, _base: Style) -> Style {
    let complete = s.index < s.current;
    let active = s.index == s.current;
    let base = style()
        .size(30, 30)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .font_size(13)
        .font_weight(600)
        .margin_bottom(6)
        .color(if complete {
            Color::WHITE
        } else if active {
            color::BLUE
        } else {
            color::TEXT_MUTED
        });
    if s.variant::<DiamondSteps>().is_none() {
        return base
            .border_radius(999)
            .background(if complete {
                color::BLUE
            } else {
                color::SURFACE
            })
            .border(1, color::LINE);
    }
    let (fill, border, ring) = (
        if complete {
            color::BLUE
        } else {
            color::SURFACE
        },
        if complete || active {
            color::BLUE
        } else {
            color::LINE_STRONG
        },
        Hue::Blue.colors().0,
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

fn stepper_label(_theme: &Theme, _: &Look<StepperItemState>, _base: Style) -> Style {
    style().font_size(13).font_weight(500)
}

fn stepper_description(_theme: &Theme, _: &Look<StepperItemState>, _base: Style) -> Style {
    style().font_size(12).color(color::TEXT_MUTED)
}

fn tabs_list(_theme: &Theme, _: &Look<TabListState>, _base: Style) -> Style {
    choice_list(false).flex_direction(FlexDirection::Row)
}

fn tab(_theme: &Theme, _: &Look<TabState>, _base: Style) -> Style {
    choice_option(false)
}

fn tab_panel(_theme: &Theme, _: &Look<TabState>, _base: Style) -> Style {
    style().padding_top(16)
}
