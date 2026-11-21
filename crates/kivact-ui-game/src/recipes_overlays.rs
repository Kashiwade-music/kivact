//! Role: Draw the console's select, menu, dialog, tooltip and command palette recipes.
use crate::color::{CYAN, NAVY, ON_DARK, PRIMARY_WEAK};
use crate::fonts::MONO_FAMILY_VAR;
use crate::size::CONTROL_HEIGHT;

use kivact::custom::PartState;
use kivact::geometry::Edge;
use kivact::interaction::{ListItemState, MenuItemState, PopupState};
use kivact::motion::{Easing, Transition};
use kivact::style::Shadow;
use kivact::style::color::{
    ACCENT, ACCENT_TEXT, BORDER, FOCUS, OVERLAY, SELECTION, SHADOW, SURFACE, SURFACE_ALT, TEXT,
    TEXT_MUTED,
};
use kivact::style::{
    Align, FlexDirection, Justify, Position, Style, Theme, TransitionProp, control, pct, style,
};
use kivact::ui::theme::{
    CommandPaletteTheme, DialogState, DialogTheme, Look, MenuTheme, SelectTheme,
    SelectTriggerState, TooltipTheme,
};
use kivact::ui::{ControlSize, DialogKind};

/// Install the select, menu, dialog, tooltip and command palette recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            SelectTheme::default()
                .trigger(select_trigger)
                .surface(select_surface)
                .option(select_option)
                .check(select_check)
                .indicator(select_indicator),
        )
        .with(
            MenuTheme::default()
                .trigger(menu_trigger)
                .surface(menu_surface)
                .item(menu_item)
                .shortcut(menu_shortcut)
                .separator(menu_separator),
        )
        .with(
            DialogTheme::default()
                .surface(dialog_surface)
                .title(dialog_title)
                .description(dialog_description)
                .header(dialog_header)
                .body(dialog_body)
                .footer(dialog_footer)
                .scrim(dialog_scrim)
                .close(dialog_close),
        )
        .with(TooltipTheme::default().surface(tooltip_surface))
        .with(
            CommandPaletteTheme::default()
                .scrim(palette_scrim)
                .panel(palette_panel)
                .search(palette_search)
                .search_icon(palette_hidden)
                .field(palette_field)
                .list(palette_list)
                .group(palette_group)
                .row(palette_row)
                .icon(palette_icon)
                .hint(palette_hint)
                .empty(palette_empty)
                .footer(palette_footer),
        )
}

/// The trigger of a select or a menu: a row beside the console's control.
fn trigger() -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .height(CONTROL_HEIGHT)
        .padding_x(12)
        .gap(8)
        .align_items(Align::Center)
        .background(SURFACE)
        .border(1, crate::color::LINE_STRONG)
        .focus(style().border_color(ACCENT).outline(3, PRIMARY_WEAK))
}

pub(crate) fn select_trigger(_: &Theme, s: &Look<SelectTriggerState>, _base: Style) -> Style {
    // The heights of the buttons of the same size.
    trigger().height(match s.size {
        ControlSize::ExtraSmall => CONTROL_HEIGHT - 12.,
        ControlSize::Small => CONTROL_HEIGHT - 6.,
        ControlSize::Medium => CONTROL_HEIGHT,
        ControlSize::Large => CONTROL_HEIGHT + 8.,
    })
}

pub(crate) fn menu_trigger(_: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    trigger()
}

pub(crate) fn select_surface(_: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    style()
        .padding_y(4)
        .min_width(220)
        .max_height(248)
        .background(SURFACE)
        .border(1, BORDER)
}

pub(crate) fn select_option(_: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .padding_left(12)
        .padding_right(26)
        .height(32)
        .align_items(Align::Center)
        .hover(style().background(SURFACE_ALT))
        .selected(style().background(SELECTION).color(ACCENT))
        .focus(
            style()
                .outline(2, FOCUS)
                .background(ACCENT)
                .color(ACCENT_TEXT),
        )
}

/// The `check` of the selected option, in the row's color.
pub(crate) fn select_check(_: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    style()
        .position(Position::Absolute)
        .right(10)
        .top(pct(50) - 8)
        .size(16, 16)
}

/// The box of the trigger's `chevron-down`, turned over while open.
pub(crate) fn select_indicator(_: &Theme, state: &Look<PopupState>, _base: Style) -> Style {
    let s = style().size(16, 16).flex_shrink(0.).color(TEXT);
    if state.open { s.rotate(180.) } else { s }
}

pub(crate) fn menu_surface(_: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .min_width(220)
        .padding(4)
        .background(SURFACE)
        .border(1, crate::color::LINE_STRONG)
        .shadow(Shadow::new(0, 14, 36, SHADOW))
}

pub(crate) fn menu_item(_: &Theme, _: &Look<MenuItemState>, _base: Style) -> Style {
    let hot = style().background(ACCENT).color(ACCENT_TEXT);
    style()
        .flex_direction(FlexDirection::Row)
        .height(32)
        .padding_x(10)
        .gap(10)
        .align_items(Align::Center)
        .color(TEXT)
        .hover(hot.clone())
        .focus(hot)
}

pub(crate) fn menu_shortcut(t: &Theme, _: &Look<MenuItemState>, _base: Style) -> Style {
    style()
        .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(10.5)
        .font_weight(600)
        .color(TEXT_MUTED)
}

pub(crate) fn menu_separator(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style().height(1).margin_y(4).background(BORDER)
}

pub(crate) fn dialog_surface(_: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    let ease = Easing::CubicBezier(0.2, 0.8, 0.2, 1.);
    match s.kind {
        // A drawer slides in from its edge.
        DialogKind::Drawer(side) => style()
            .width(420)
            .max_width(pct(100))
            .background(SURFACE)
            .border_left(1, crate::color::LINE_STRONG)
            .enter(style().opacity(0.).translate(
                match side {
                    Edge::Left => -40.,
                    Edge::Right => 40.,
                    _ => 0.,
                },
                match side {
                    Edge::Top => -40.,
                    Edge::Bottom => 40.,
                    _ => 0.,
                },
            ))
            .transition(TransitionProp::All, Transition::tween(240, ease)),
        DialogKind::Popover => style()
            .width(280)
            .padding(14)
            .gap(8)
            .background(SURFACE)
            .border(1, BORDER),
        // A modal rises a little as it fades in.
        _ => style()
            .width(520)
            .max_width(pct(100) - 32)
            .background(SURFACE)
            .border(1, crate::color::LINE_STRONG)
            .shadow(Shadow::new(0, 14, 36, SHADOW))
            .enter(style().opacity(0.).scale(0.985).translate(0, 10))
            .transition(TransitionProp::All, Transition::tween(220, ease)),
    }
}

pub(crate) fn dialog_header(_: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    if s.kind == DialogKind::Popover {
        style().align_items(Align::Center)
    } else {
        // The title over the description, clear of the 32 px close button.
        style()
            .padding_left(20)
            .padding_right(56)
            .padding_top(14)
            .padding_bottom(12)
            .gap(8)
            .border_top(4, ACCENT)
            .border_bottom(1, BORDER)
    }
}

pub(crate) fn dialog_title(t: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    if s.kind == DialogKind::Popover {
        style().font_weight(700)
    } else {
        style()
            .font_family(
                t.get(&crate::fonts::DISPLAY_FAMILY_VAR)
                    .as_str()
                    .unwrap_or_default(),
            )
            .font_size(22)
            .font_weight(700)
            .font_style(kivact::style::FontStyle::Italic)
            .line_height(1.)
    }
}

pub(crate) fn dialog_description(_: &Theme, _: &Look<DialogState>, _base: Style) -> Style {
    style().font_size(13).color(TEXT_MUTED)
}

pub(crate) fn dialog_body(_: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    if s.kind == DialogKind::Popover {
        style().gap(6)
    } else {
        style().padding_x(20).padding_y(18)
    }
}

pub(crate) fn dialog_footer(_: &Theme, _: &Look<DialogState>, _base: Style) -> Style {
    style()
        .flex_shrink(0.)
        .padding_x(18)
        .padding_y(14)
        .gap(10)
        .justify_content(Justify::End)
        .align_items(Align::Center)
        .background(SURFACE_ALT)
        .border_top(1, BORDER)
}

pub(crate) fn dialog_scrim(_: &Theme, _: &Look<DialogState>, _base: Style) -> Style {
    style().background(OVERLAY).backdrop_blur(2)
}

/// The close button around the theme's `x` icon.
pub(crate) fn dialog_close(_: &Theme, _: &Look<DialogState>, _base: Style) -> Style {
    style()
        .size(32, 32)
        .position(Position::Absolute)
        .top(16)
        .right(12)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(TEXT_MUTED)
        .hover(
            style()
                .background(SURFACE_ALT)
                .border(1, BORDER)
                .color(TEXT),
        )
}

pub(crate) fn tooltip_surface(_: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    style()
        .padding_x(9)
        .padding_y(5)
        .max_width(320)
        .background(NAVY)
        .color(ON_DARK)
        .border_bottom(2, CYAN)
        .font_size(11)
}

fn palette_scrim(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .position(Position::Fixed)
        .inset(0)
        .background(OVERLAY)
        .backdrop_blur(2)
}

fn palette_panel(_: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(600)
        .max_width(pct(100) - 32)
        .background(SURFACE)
        .border(1, crate::color::LINE_STRONG)
        .shadow(Shadow::new(0, 14, 36, SHADOW))
        .clip(true)
}

fn palette_search(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .border_bottom(2, ACCENT)
}

/// The console shows no magnifier.
fn palette_hidden(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style().size(0, 0)
}

fn palette_field(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_grow(1.)
        .min_width(0)
        .height(52)
        .font_size(15)
        .border_width(0)
}

fn palette_list(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .max_height(360)
}

fn palette_group(t: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .padding_x(10)
        .padding_top(10)
        .padding_bottom(4)
        .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(10.5)
        .font_weight(600)
        .color(TEXT_MUTED)
}

fn palette_row(t: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    // A row is never a focus stop (the field keeps the focus), so a
    // disabled one takes the theme's opacity here.
    style()
        .flex_direction(FlexDirection::Row)
        .height(38)
        .padding_x(10)
        .gap(10)
        .align_items(Align::Center)
        .selected(
            style()
                .background(SELECTION)
                .border_left(3, ACCENT)
                .color(ACCENT),
        )
        .disabled(style().opacity(t.get(&control::DISABLED_OPACITY_VAR)))
}

fn palette_icon(_: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    style().size(16, 16).flex_shrink(0.)
}

fn palette_hint(t: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    style()
        .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(10.5)
        .font_weight(600)
        .color(TEXT_MUTED)
}

fn palette_empty(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style().padding(28).color(TEXT_MUTED)
}

fn palette_footer(t: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .padding_x(14)
        .padding_y(8)
        .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(10.5)
        .font_weight(600)
        .background(SURFACE_ALT)
        .border_top(1, BORDER)
}
