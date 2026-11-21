//! Role: The Pop looks of the overlays: select, menu, dialog, tooltip and command palette.

use crate::color;
use crate::recipe_parts::{button_style, chrome};
use kivact::custom::PartState;
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::Point;
use kivact::interaction::{ListItemState, MenuItemState, PopupState};
use kivact::style::Shadow;
use kivact::style::color as core;
use kivact::style::{
    Align, Color, FlexDirection, Justify, Position, Style, Theme, control, pct, style,
};
use kivact::ui::theme::{
    CommandPaletteTheme, DialogState, DialogTheme, Look, MenuTheme, SelectTheme,
    SelectTriggerState, TooltipTheme,
};
use kivact::ui::{ButtonVariant, ControlSize, DialogKind};

/// Install the kit's recipes of the overlays.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme
        .with(
            SelectTheme::default()
                .trigger(select_trigger)
                .surface(select_surface)
                .option(select_option)
                .indicator(select_indicator),
        )
        .with(
            MenuTheme::default()
                .trigger(menu_trigger)
                .surface(menu_surface)
                .separator(menu_separator)
                .item(menu_item)
                .shortcut(menu_shortcut)
                .check(menu_check)
                .submenu(menu_submenu),
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
        .with(TooltipTheme::default().surface(tooltip));
    theme.with(
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

/// The 16 px box of a mark icon, in the text color.
fn mark() -> Style {
    style().size(16, 16).flex_shrink(0).color(color::TEXT)
}

/// The option of a select: a 32 px row, highlighted when hovered or chosen.
fn select_option(_theme: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    style()
        .height(32)
        .padding_x(8)
        .border_radius(4)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .hover(style().background(color::SURFACE_ACTIVE))
        .selected(style().background(color::SURFACE_ACTIVE))
}

/// The list of a select: a bordered, shadowed panel of at most 250 px.
fn select_surface(_theme: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    style()
        .min_width(220)
        .padding(4)
        .background(color::SURFACE)
        .border(1, color::LINE)
        .border_radius(8)
        .shadow(Shadow::new(0, 4, 6, core::SHADOW))
        .max_height(250)
}

fn select_trigger(_theme: &Theme, s: &Look<SelectTriggerState>, _base: Style) -> Style {
    // The heights of the buttons of the same size.
    let height = match s.size {
        ControlSize::ExtraSmall => 26,
        ControlSize::Small => 32,
        ControlSize::Medium => 36,
        ControlSize::Large => 40,
    };
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .gap(8)
        .height(height)
        .padding_x(12)
        .border(1, color::LINE)
        .border_radius(8)
        .background(color::SURFACE)
        .color(color::TEXT)
}

/// The trigger's chevron, the theme's `selector` icon.
fn select_indicator(_theme: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    mark()
}

fn menu_trigger(_theme: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    let color = color::TEXT;
    button_style(ButtonVariant::Secondary, None, ControlSize::Medium, false)
        .padding_right(40)
        .paint(move |d, size| {
            let x = size.width - 32.0;
            let y = (size.height - 16.0) / 2.0;
            let p = Path::new()
                .move_to(Point::new(x + 4.0, y + 6.0))
                .line_to(Point::new(x + 8.0, y + 10.0))
                .line_to(Point::new(x + 12.0, y + 6.0));
            d.stroke_path(
                &p,
                &Stroke::new(4.0 / 3.0)
                    .cap(LineCap::Round)
                    .join(LineJoin::Round),
                color,
            );
        })
}

fn menu_surface(_theme: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    chrome()
        .flex_direction(FlexDirection::Column)
        .min_width(192)
        .padding(4)
        .border_radius(8)
}

fn menu_separator(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style().height(1).margin_y(4).background(core::BORDER)
}

fn menu_item(_theme: &Theme, _: &Look<MenuItemState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(8)
        .height(32)
        .padding_x(8)
        .border_radius(4)
        .hover(style().background(core::SURFACE_ALT))
        .focus(style().background(core::SURFACE_ALT))
        .selected(style().background(core::SURFACE_ALT))
}

fn menu_shortcut(_theme: &Theme, _: &Look<MenuItemState>, _base: Style) -> Style {
    style().font_size(12).color(core::TEXT_MUTED)
}

/// The box of a check item's `check` icon.
fn menu_check(_theme: &Theme, _: &Look<MenuItemState>, _base: Style) -> Style {
    mark()
}

/// The box of a submenu's `chevron-right` icon.
fn menu_submenu(_theme: &Theme, _: &Look<MenuItemState>, _base: Style) -> Style {
    mark()
}

fn dialog_surface(_theme: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    // A modal and a drawer grow in from 96%.
    let enter = |s: Style| {
        s.enter(style().scale(0.96)).transition(
            kivact::style::TransitionProp::All,
            kivact::motion::Transition::tween(200, kivact::motion::Easing::EaseOut),
        )
    };
    match s.kind {
        DialogKind::Drawer(_) => enter(
            chrome()
                .width(400)
                .max_width(pct(100))
                .padding(24)
                .gap(16)
                .border_radius(0),
        ),
        DialogKind::Popover => chrome().width(288).padding(16).gap(6),
        _ => enter(chrome().width(480).max_width(pct(95)).padding(24).gap(16)),
    }
}

fn dialog_title(_theme: &Theme, _: &Look<DialogState>, _base: Style) -> Style {
    style().font_size(18).font_weight(600)
}

fn dialog_description(_theme: &Theme, _: &Look<DialogState>, _base: Style) -> Style {
    style().font_size(14).color(core::TEXT_MUTED)
}

/// The title over the description, clear of the 36 px close button.
fn dialog_header(_theme: &Theme, _: &Look<DialogState>, _base: Style) -> Style {
    style().gap(6).padding_right(44)
}

fn dialog_body(_theme: &Theme, _: &Look<DialogState>, _base: Style) -> Style {
    style().gap(8)
}

fn dialog_footer(_theme: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    let base = style().gap(8).justify_content(Justify::End);
    if matches!(s.kind, DialogKind::Drawer(_)) {
        base.padding_top(16).border_top(1, core::BORDER)
    } else {
        base
    }
}

fn dialog_scrim(_theme: &Theme, _: &Look<DialogState>, _base: Style) -> Style {
    style().background(color::OVERLAY_SCRIM)
}

/// The close button around the theme's `x` icon, muted until hovered.
fn dialog_close(_theme: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    let inset = match s.kind {
        DialogKind::Popover => 8,
        _ => 18,
    };
    style()
        .position(Position::Absolute)
        .top(inset)
        .right(inset)
        .size(36, 36)
        .border_radius(8)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(color::TEXT_MUTED)
        .active(style().opacity(0.7))
        .hover(style().background(core::SURFACE_ALT).color(color::TEXT))
}

fn tooltip(_theme: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    style()
        .padding_x(10)
        .padding_y(6)
        .max_width(320)
        .font_size(12)
        .font_weight(500)
        .background(color::TEXT)
        .color(color::SURFACE)
        .border_radius(6)
}

fn palette_scrim(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .position(Position::Fixed)
        .inset(0)
        .background(color::OVERLAY_SCRIM)
}

fn palette_panel(_theme: &Theme, _: &Look<PopupState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(560)
        .max_width(pct(95))
        .background(color::SURFACE)
        .border(1, color::LINE)
        .border_radius(10)
        .color(color::TEXT)
        .shadow(Shadow::new(0, 10, 15, core::SHADOW))
        .clip(true)
}

fn palette_search(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .border_bottom(1, core::BORDER)
}

/// Pop shows no magnifier.
fn palette_hidden(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style().size(0, 0)
}

fn palette_field(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_grow(1)
        .min_width(0)
        .height(48)
        .padding_x(12)
        .border(0, Color::TRANSPARENT)
        .border_radius(0)
}

fn palette_list(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .max_height(320)
}

fn palette_group(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .padding_x(8)
        .padding_top(10)
        .padding_bottom(4)
        .font_size(11)
        .font_weight(600)
        .color(core::TEXT_MUTED)
}

/// A row is never a focus stop (the field keeps the focus), so a disabled
/// one takes the theme's opacity here.
fn palette_row(theme: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    style()
        .height(36)
        .padding_x(8)
        .border_radius(6)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(8)
        .selected(style().background(core::SURFACE_ALT))
        .disabled(style().opacity(theme.get(&control::DISABLED_OPACITY_VAR)))
}

fn palette_icon(_theme: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    style().size(16, 16).flex_shrink(0)
}

fn palette_hint(_theme: &Theme, _: &Look<ListItemState>, _base: Style) -> Style {
    style().font_size(12).color(core::TEXT_MUTED)
}

fn palette_empty(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style().padding(24).color(core::TEXT_MUTED)
}

fn palette_footer(_theme: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .padding_x(12)
        .padding_y(8)
        .font_size(12)
        .color(core::TEXT_MUTED)
        .border_top(1, core::BORDER)
}
