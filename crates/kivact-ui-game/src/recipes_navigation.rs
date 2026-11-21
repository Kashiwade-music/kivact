//! Role: Draw console navigation with numbered chevrons and compact page controls.
use crate::choice_style;
use crate::color::{PRIMARY_WEAK, SURFACE_3, TEXT_3};
use crate::fonts::{DISPLAY_FAMILY_VAR, MONO_FAMILY_VAR};
use crate::variants::TabsVariant;
use kivact::interaction::{NavOpen, TabListState, TabState};
use kivact::style::color::{ACCENT, ACCENT_TEXT, BORDER, SURFACE_ALT, TEXT, TEXT_MUTED};
use kivact::style::{
    Align, Color, FlexDirection, FontStyle, Justify, Shape, Style, Theme, pct, style,
};
use kivact::ui::theme::{
    BreadcrumbItemState, BreadcrumbTheme, Look, NavigationMenuTheme, PaginationItemState,
    PaginationTheme, StepperItemState, StepperState, StepperTheme, TabsTheme,
};

/// Install the tabs, pagination, breadcrumb, stepper and navigation menu recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(TabsTheme::default().list(tabs_list).tab(tab))
        .with(
            PaginationTheme::default()
                .root(pagination_root)
                .item(pagination_item)
                .ellipsis(pagination_ellipsis),
        )
        .with(
            BreadcrumbTheme::default()
                .root(breadcrumb_root)
                .item(breadcrumb_item)
                .separator(breadcrumb_separator),
        )
        .with(
            StepperTheme::default()
                .root(stepper_root)
                .item(stepper_item)
                .indicator(stepper_indicator)
                .label(stepper_label),
        )
        .with(
            NavigationMenuTheme::default()
                .root(navigation_root)
                .panel(navigation_panel),
        )
}

pub(crate) fn tabs_list(_: &Theme, state: &Look<TabListState>, _base: Style) -> Style {
    if state.variant::<TabsVariant>() == Some(TabsVariant::Boxed) {
        return choice_style::boxed_list();
    }
    if state.variant::<TabsVariant>() == Some(TabsVariant::Navigation) {
        return choice_style::navigation_list();
    }
    style()
        .flex_direction(FlexDirection::Row)
        .width(pct(100))
        .padding(0)
        .background(Color::TRANSPARENT)
        .border(0, BORDER)
        .border_bottom(1, BORDER)
        .gap(8)
}

pub(crate) fn tab(_: &Theme, state: &Look<TabState>, _base: Style) -> Style {
    if state.variant::<TabsVariant>() == Some(TabsVariant::Boxed) {
        return choice_style::boxed_tab();
    }
    if state.variant::<TabsVariant>() == Some(TabsVariant::Navigation) {
        return choice_style::navigation_item();
    }
    style()
        .flex_direction(FlexDirection::Row)
        .gap(7)
        .height(38)
        .padding_x(16)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border_bottom(3, Color::TRANSPARENT)
        .color(TEXT_MUTED)
        .selected(style().border_color(ACCENT).color(ACCENT).font_weight(700))
        .hover(style().background(kivact::style::color::SELECTION))
}

pub(crate) fn pagination_root(_: &Theme, _: &Look<()>, base: Style) -> Style {
    base.gap(4).padding(0)
}

pub(crate) fn pagination_ellipsis(t: &Theme, _: &Look<()>, base: Style) -> Style {
    base.font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
}

pub(crate) fn pagination_item(_: &Theme, s: &Look<PaginationItemState>, _base: Style) -> Style {
    style()
        .size(30, 30)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border(1, BORDER)
        .color(if s.current { ACCENT_TEXT } else { TEXT_MUTED })
        .background(if s.current {
            ACCENT
        } else {
            Color::TRANSPARENT
        })
        .hover(style().background(SURFACE_ALT).color(TEXT))
        .disabled(style().opacity(0.42))
}

pub(crate) fn breadcrumb_root(_: &Theme, _: &Look<()>, base: Style) -> Style {
    base.gap(8).padding(0).align_items(Align::Center)
}

pub(crate) fn breadcrumb_separator(_: &Theme, _: &Look<()>, _base: Style) -> Style {
    style().font_size(20).font_weight(700).color(ACCENT)
}

pub(crate) fn breadcrumb_item(_: &Theme, s: &Look<BreadcrumbItemState>, _base: Style) -> Style {
    style()
        .font_size(12)
        .font_weight(if s.is_current() { 700 } else { 400 })
        .color(if s.is_current() { TEXT } else { TEXT_MUTED })
}

pub(crate) fn stepper_root(_: &Theme, _: &Look<StepperState>, base: Style) -> Style {
    base.width(pct(100)).gap(3).padding(0)
}

pub(crate) fn stepper_item(_: &Theme, s: &Look<StepperItemState>, base: Style) -> Style {
    let fill = if s.index == s.current {
        ACCENT
    } else if s.index < s.current {
        PRIMARY_WEAK
    } else {
        SURFACE_3
    };
    let text = if s.index == s.current {
        ACCENT_TEXT
    } else if s.index < s.current {
        ACCENT
    } else {
        TEXT_3
    };
    base.height(40)
        .padding_left(if s.index == 0 { 14 } else { 22 })
        .padding_right(18)
        .gap(8)
        .flex_grow(1.)
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
        .background(fill)
        .color(text)
}

pub(crate) fn stepper_indicator(t: &Theme, _: &Look<StepperItemState>, base: Style) -> Style {
    base.font_family(t.get(&DISPLAY_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(18)
        .font_weight(800)
        .font_style(FontStyle::Italic)
}

pub(crate) fn stepper_label(_: &Theme, _: &Look<StepperItemState>, base: Style) -> Style {
    base.font_size(12).font_weight(700)
}

pub(crate) fn navigation_root(_: &Theme, _: &Look<NavOpen>, base: Style) -> Style {
    base.gap(2)
}

pub(crate) fn navigation_panel(_: &Theme, _: &Look<NavOpen>, base: Style) -> Style {
    base.border_radius(0)
        .border_color(crate::color::LINE_STRONG)
}
