//! Role: Creative appearance of the standard select, menu, dialog and tooltip.
//! These are the floating surfaces of the kit.
use crate::color;
use crate::fonts::mono_family;
use crate::recipe_parts::{button_style, field_style, glass_style};
use kivact::custom::PartState;
use kivact::interaction::{ListItemState, MenuItemState, PopupState};
use kivact::style::Shadow;
use kivact::style::{Align, FlexDirection, Justify, Position, Style, Theme, pct, style};
use kivact::ui::DialogKind;
use kivact::ui::theme::{
    DialogState, DialogTheme, Look, MenuTheme, SelectTheme, SelectTriggerState, TooltipTheme,
};

/// The theme of a select: a field trigger, a dropdown with checked options.
pub(crate) fn select() -> SelectTheme {
    SelectTheme::default()
        .trigger(select_trigger)
        .surface(select_surface)
        .option(select_option)
        .check(select_check)
        .indicator(select_indicator)
}

/// The heights of the trigger's buttons of the same size.
fn select_trigger(_t: &Theme, s: &Look<SelectTriggerState>, _base: Style) -> Style {
    let field = field_style();
    match s.size {
        kivact::ui::ControlSize::ExtraSmall => field.height(16).font_size(10),
        kivact::ui::ControlSize::Small => field.height(20).font_size(11),
        kivact::ui::ControlSize::Medium => field,
        kivact::ui::ControlSize::Large => field.height(30).font_size(13),
    }
}

fn select_surface(_t: &Theme, _s: &Look<PopupState>, _base: Style) -> Style {
    dropdown_surface().max_height(214.5)
}

fn select_option(_t: &Theme, _s: &Look<ListItemState>, _base: Style) -> Style {
    dropdown_option().padding_right(45)
}

/// The `check` of the selected option, in the text color.
fn select_check(_t: &Theme, _s: &Look<ListItemState>, _base: Style) -> Style {
    style()
        .position(Position::Absolute)
        .right(5)
        .top(pct(50) - 8)
        .size(16, 16)
        .color(color::TEXT)
}

fn select_indicator(_t: &Theme, _s: &Look<PopupState>, _base: Style) -> Style {
    mark()
}

/// The theme of a menu: a button trigger and a glass surface of rows.
pub(crate) fn menu() -> MenuTheme {
    MenuTheme::default()
        .trigger(menu_trigger)
        .surface(menu_surface)
        .item(menu_item)
        .shortcut(menu_shortcut)
        .separator(menu_separator)
        .check(menu_check)
        .submenu(menu_check)
}

fn menu_trigger(t: &Theme, _s: &Look<PopupState>, _base: Style) -> Style {
    button_style(t)
}

fn menu_surface(_t: &Theme, _s: &Look<PopupState>, _base: Style) -> Style {
    glass_style()
        .flex_direction(FlexDirection::Column)
        .color(color::TEXT)
}

fn menu_item(_t: &Theme, _s: &Look<MenuItemState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .height(27)
        .gap(9)
        .padding_x(7)
        .border_radius(6)
        .align_items(Align::Center)
        .hover(style().background(kivact::style::color::HOVER))
        .selected(style().background(color::SELECTION))
}

fn menu_shortcut(t: &Theme, _s: &Look<MenuItemState>, _base: Style) -> Style {
    style().font_family(mono_family(t)).font_size(10)
}

fn menu_separator(_t: &Theme, _s: &Look<PartState>, _base: Style) -> Style {
    style().height(1).background(color::HAIR).margin_y(4)
}

fn menu_check(_t: &Theme, _s: &Look<MenuItemState>, _base: Style) -> Style {
    mark()
}

/// The theme of a dialog: a glass panel, a scrim and a title over the description.
pub(crate) fn dialog() -> DialogTheme {
    DialogTheme::default()
        .surface(dialog_surface)
        .scrim(dialog_scrim)
        // The title over the description, clear of the close button.
        .header(dialog_header)
        .title(dialog_title)
        .description(dialog_description)
        .body(dialog_body)
        .footer(dialog_footer)
}

fn dialog_surface(_t: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    match s.kind {
        DialogKind::Drawer(_) => glass_style()
            .padding(18)
            .gap(14)
            .width(400)
            .max_width(pct(100))
            .border_radius(0),
        _ => glass_style()
            .padding(18)
            .gap(14)
            .min_width(320)
            .max_width(640),
    }
}

fn dialog_scrim(_t: &Theme, _s: &Look<DialogState>, _base: Style) -> Style {
    style().background(color::MODAL_BACKDROP)
}

fn dialog_header(_t: &Theme, _s: &Look<DialogState>, _base: Style) -> Style {
    style().gap(4).padding_right(28)
}

fn dialog_title(_t: &Theme, _s: &Look<DialogState>, _base: Style) -> Style {
    style().font_weight(600)
}

fn dialog_description(_t: &Theme, _s: &Look<DialogState>, _base: Style) -> Style {
    style().color(color::MUTED)
}

fn dialog_body(_t: &Theme, _s: &Look<DialogState>, _base: Style) -> Style {
    style().gap(8)
}

fn dialog_footer(_t: &Theme, _s: &Look<DialogState>, _base: Style) -> Style {
    style().gap(8).justify_content(Justify::End)
}

/// The theme of a tooltip: a small dark glass card.
pub(crate) fn tooltip() -> TooltipTheme {
    TooltipTheme::default().surface(tooltip_surface)
}

fn tooltip_surface(_t: &Theme, _s: &Look<PopupState>, _base: Style) -> Style {
    style()
        .padding(10)
        .max_width(320)
        .border_radius(8)
        .color(color::TEXT)
        .background(color::OVERLAY_SURFACE)
        .backdrop_blur(22)
        .border(1, color::HAIR_STRONG)
        .shadow(Shadow::new(0., 10., 24., color::DIALOG_SHADOW))
}

/// The dropdown of a select: the glass surface, in the text color.
fn dropdown_surface() -> Style {
    style()
        .padding(5)
        .min_width(160)
        .border_radius(12)
        .background(color::OVERLAY_SURFACE)
        .backdrop_blur(22)
        .border(1, color::OVERLAY_BORDER)
        .color(color::TEXT)
        .shadow(Shadow::new(0., 10., 24., color::MENU_SHADOW))
}

/// One option of a dropdown: the chosen one in the accent.
fn dropdown_option() -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .height(27)
        .padding_x(9)
        .gap(20)
        .align_items(Align::Center)
        .border_radius(6)
        .hover(style().background(kivact::style::color::HOVER))
        .selected(style().color(color::ACCENT_HIGH))
}

/// The 16 px box of a mark icon (a check, a chevron), in the text color.
fn mark() -> Style {
    style().size(16, 16).flex_shrink(0)
}

/// Install the overlays' recipes into `theme`.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(select())
        .with(menu())
        .with(dialog())
        .with(tooltip())
}
