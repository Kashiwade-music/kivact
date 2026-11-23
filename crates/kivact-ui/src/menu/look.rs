//! Role: The menu recipe state, recipe table and default looks.
use crate::look_common::{description, mark, trigger};
use crate::theme::Look;

use crate::recipe::recipe_table;
use kivact_core::custom::PartState;
use kivact_core::style::{Align, FlexDirection, Style, Theme, style};
use kivact_interaction::{MenuItemState, PopupState};
/// A menu's own trigger, drawn like a select's.
pub(crate) fn menu_trigger(t: &Theme, _: &Look<PopupState>) -> Style {
    trigger(t)
}

pub(crate) fn menu_item(t: &Theme, state: &Look<MenuItemState>) -> Style {
    let s = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(8)
        .height(t.get(&kivact_core::style::control::HEIGHT_VAR))
        .padding_x(t.get(&kivact_core::style::space::MD_VAR))
        .focus(style().background(kivact_core::style::color::HOVER));
    if state.destructive {
        s.color(kivact_core::style::color::DANGER)
    } else {
        s
    }
}

pub(crate) fn separator(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .height(1)
        .margin_y(4)
        .background(kivact_core::style::color::BORDER)
}

/// A heading: muted small text the keys skip.
pub(crate) fn menu_label(t: &Theme, _: &Look<PartState>) -> Style {
    style()
        .padding_x(t.get(&kivact_core::style::space::MD_VAR))
        .padding_y(t.get(&kivact_core::style::space::XS_VAR))
        .font_size(t.get(&kivact_core::style::font::SIZE_SMALL_VAR))
        .font_weight(t.get(&kivact_core::style::font::WEIGHT_STRONG_VAR))
        .color(kivact_core::style::color::TEXT_MUTED)
}

pub(crate) fn shortcut(_: &Theme, _: &Look<MenuItemState>) -> Style {
    style()
        .flex_grow(1)
        .color(kivact_core::style::color::TEXT_MUTED)
}

pub(crate) fn menu_description(t: &Theme, _: &Look<MenuItemState>) -> Style {
    description(t)
}

/// The check or radio mark's box: 16 px wide even while empty, so labels
/// line up.
pub(crate) fn menu_check(_: &Theme, _: &Look<MenuItemState>) -> Style {
    mark().width(16)
}

pub(crate) fn menu_submenu(_: &Theme, _: &Look<MenuItemState>) -> Style {
    mark()
}

/// The room of an inset row: as wide as the check box.
pub(crate) fn menu_inset(_: &Theme, _: &Look<PartState>) -> Style {
    style().width(16).flex_shrink(0)
}

recipe_table! {
    /// The recipes of a [`menu`](super::menu).
    MenuTheme {
        /// The menu's own trigger button (without `Menu::attach`).
        trigger(trigger_recipe): PopupState = menu_trigger,
        /// The top level's floating surface.
        surface(surface_recipe): PopupState = crate::look_common::surface,
        /// A submenu's floating surface (by default the top level's look).
        sub_surface(sub_surface_recipe): PopupState = crate::look_common::surface,
        /// The top level's surface of a menu opened by
        /// [`attach_context`](super::Menu::attach_context) (by default the
        /// top level's look).
        context_surface(context_surface_recipe): PopupState = crate::look_common::surface,
        /// An item row: its marks, icon, label, shortcut and chevron.
        item(item_recipe): MenuItemState = menu_item,
        separator(separator_recipe): PartState = separator,
        /// A heading: a [`MenuEntry::label`](kivact_interaction::MenuEntry::label)
        /// or the label of a group.
        label(label_recipe): PartState = menu_label,
        shortcut(shortcut_recipe): MenuItemState = shortcut,
        /// The secondary line of an item with a description.
        description(description_recipe): MenuItemState = menu_description,
        /// The leading box of a check or radio item, holding the theme's
        /// `check` or `menu-radio` icon while it is checked. The box gets the
        /// item's actual [`MenuItemState`], as the submenu box does, even for
        /// a checked submenu item.
        check(check_recipe): MenuItemState = menu_check,
        /// The trailing box of a submenu item, holding the theme's
        /// `chevron-right` icon.
        submenu(submenu_recipe): MenuItemState = menu_submenu,
        /// The empty box before the content of an
        /// [`inset`](kivact_interaction::MenuEntry::inset) item or heading:
        /// by default the width of the check box, so it lines up with the
        /// check and radio items.
        inset(inset_recipe): PartState = menu_inset,
    }
}
