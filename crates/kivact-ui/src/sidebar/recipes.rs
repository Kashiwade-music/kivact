//! Role: The default recipes of the sidebar's parts, read from the theme's variables.

use super::look::{SidebarItemState, SidebarState, SidebarTheme};
use super::state::{SidebarCollapsible, SidebarFrame, SidebarMenuButtonSize, SidebarSide};
use super::vars;
use crate::layer::with_base_recipe_style;
use crate::recipe::{Look, Recipe};
use kivact_core::state::Value;
use kivact_core::style::{
    Align, Color, Cursor, FlexDirection, Justify, Overflow, Position, Shadow, Style, Theme, color,
    font, pct, space, style,
};
use kivact_core::view::{Element, Variants};

/// The part `node` with the recipe `pick` chooses, for `state`, under the caller's styles.
pub(super) fn themed<N: Element, S: Clone + PartialEq + 'static>(
    node: N,
    pick: fn(&SidebarTheme) -> &Recipe<S>,
    state: Value<S>,
) -> N {
    with_base_recipe_style(node, pick, &Variants::default(), move || state.get())
}

/// Whether the frame insets the panel by 8 px (`Floating` and `Inset`).
fn framed(frame: SidebarFrame) -> bool {
    matches!(frame, SidebarFrame::Floating | SidebarFrame::Inset)
}

/// The room a sidebar takes in the layout for `state`.
fn width_of(theme: &Theme, state: &SidebarState) -> f64 {
    if state.collapsible == SidebarCollapsible::None || state.open {
        return theme.get(&vars::WIDTH_VAR);
    }
    let icon = theme.get(&vars::WIDTH_ICON_VAR);
    match state.collapsible {
        SidebarCollapsible::Icon if framed(state.frame) => icon + 16.0,
        SidebarCollapsible::Icon => icon,
        _ => 0.0,
    }
}

/// The drop shadow of a framed panel and of an inset page.
fn card_shadow() -> Shadow {
    Shadow::new(0, 1, 2, Color::BLACK.with_alpha(0.05))
}

/// The row of the provider: the sidebar and the page, filling the parent.
pub(super) fn provider(_theme: &Theme, state: &Look<SidebarState>) -> Style {
    let s = style()
        .width(pct(100))
        .height(pct(100))
        .min_width(0)
        .min_height(0)
        .align_items(Align::Stretch);
    if state.frame == SidebarFrame::Inset {
        s.background(vars::BACKGROUND)
    } else {
        s
    }
}

/// The box of a sidebar: the room it takes in the layout.
pub(super) fn sidebar(theme: &Theme, state: &Look<SidebarState>) -> Style {
    let s = style()
        .position(Position::Relative)
        .flex_direction(FlexDirection::Column)
        .flex_shrink(0)
        .z_index(10)
        .height(pct(100))
        .min_height(0)
        .color(vars::FOREGROUND)
        .width(width_of(theme, state));
    if framed(state.frame) {
        s.padding(theme.get(&space::MD_VAR))
    } else {
        s
    }
}

/// The panel: the sidebar's surface and its edge.
pub(super) fn panel(theme: &Theme, state: &Look<SidebarState>) -> Style {
    let s = style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100))
        .height(pct(100))
        .min_width(0)
        .background(vars::BACKGROUND)
        .color(vars::FOREGROUND)
        .font_size(theme.get(&font::SIZE_SMALL_VAR));
    match (state.frame, state.side) {
        (SidebarFrame::Sidebar, SidebarSide::Left) => s.border_right(1, vars::BORDER),
        (SidebarFrame::Sidebar, SidebarSide::Right) => s.border_left(1, vars::BORDER),
        (SidebarFrame::Floating, _) => s
            .border_radius(vars::ITEM_RADIUS)
            .outline(1, vars::BORDER)
            .shadow(card_shadow()),
        (SidebarFrame::Inset, _) => s,
    }
}

/// The header, footer and other sections of the panel: a column with 8 px of padding and gap.
pub(super) fn section(theme: &Theme, _state: &Look<SidebarState>) -> Style {
    let gap = theme.get(&space::MD_VAR);
    style().flex_shrink(0).gap(gap).padding(gap)
}

/// The scrolling middle section of the panel.
pub(super) fn content(_theme: &Theme, _state: &Look<SidebarState>) -> Style {
    style().flex_grow(1).flex_shrink(1).min_height(0).gap(0)
}

/// The strip on the inner edge: the hit area of the rail, with its hover look while closed.
pub(super) fn rail(_theme: &Theme, state: &Look<SidebarState>) -> Style {
    let offcanvas = state.collapsible == SidebarCollapsible::Offcanvas;
    let s = style()
        .position(Position::Absolute)
        .top(0)
        .bottom(0)
        .width(16)
        .z_index(20)
        .cursor(Cursor::EwResize);
    let s = if offcanvas && !state.open {
        s.hover(style().background(vars::BACKGROUND))
    } else {
        s
    };
    match state.side {
        SidebarSide::Left => s.right(-8),
        SidebarSide::Right => s.left(-8),
    }
}

/// The page beside an inset sidebar: a card that keeps its gap to the sidebar.
pub(super) fn inset(theme: &Theme, state: &Look<SidebarState>) -> Style {
    let s = style()
        .position(Position::Relative)
        .flex_direction(FlexDirection::Column)
        .flex_grow(1)
        .flex_shrink(1)
        .min_width(0)
        .min_height(0)
        .align_items(Align::Stretch)
        .background(vars::PAGE)
        .color(color::TEXT);
    if state.frame != SidebarFrame::Inset {
        return s;
    }
    // `m-2` without the margin on the sidebar's side, which returns while the sidebar is closed.
    let gap = theme.get(&space::MD_VAR);
    let apart = if state.open { 0.0 } else { gap };
    let s = s
        .border_radius(vars::ITEM_RADIUS)
        .margin_y(gap)
        .shadow(card_shadow());
    match state.side {
        SidebarSide::Left => s.margin_right(gap).margin_left(apart),
        SidebarSide::Right => s.margin_left(gap).margin_right(apart),
    }
}

/// The sidebar's trigger: the button's own look, with no addition.
pub(super) fn trigger(_theme: &Theme, _state: &Look<SidebarState>) -> Style {
    style()
}

/// A group: a column of its label row and its body.
pub(super) fn group(theme: &Theme, _state: &Look<SidebarState>) -> Style {
    style()
        .position(Position::Relative)
        .flex_direction(FlexDirection::Column)
        .width(pct(100))
        .min_width(0)
        .padding(theme.get(&space::MD_VAR))
}

/// A group's label row: a 32 px row of small, muted text.
pub(super) fn group_label(theme: &Theme, _state: &Look<SidebarState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .flex_shrink(0)
        .height(32)
        .padding_x(theme.get(&space::MD_VAR))
        .border_radius(vars::ITEM_RADIUS)
        .font_size(theme.get(&font::SIZE_SMALL_VAR))
        .color(vars::FOREGROUND.with_alpha(0.7))
}

/// A group's action, a 20 px square at the end of the label row.
pub(super) fn group_action(_theme: &Theme, _state: &Look<SidebarState>) -> Style {
    style()
        .position(Position::Absolute)
        .top(14)
        .right(12)
        .size(20, 20)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

/// A group's body.
pub(super) fn group_content(theme: &Theme, _state: &Look<SidebarState>) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100))
        .font_size(theme.get(&font::SIZE_SMALL_VAR))
}

/// A menu: a full-width column.
pub(super) fn menu(_theme: &Theme, _state: &Look<SidebarState>) -> Style {
    style().width(pct(100)).min_width(0)
}

/// A horizontal rule in the sidebar's border color, inset by 8 px.
pub(super) fn separator(theme: &Theme, _state: &Look<SidebarState>) -> Style {
    style()
        .height(1)
        .flex_shrink(0)
        .margin_x(theme.get(&space::MD_VAR))
        .background(vars::BORDER)
}

/// A text field in the sidebar: full width, 32 px high, on the page color.
pub(super) fn input(_theme: &Theme, _state: &Look<SidebarState>) -> Style {
    style()
        .width(pct(100))
        .min_width(0)
        .height(32)
        .background(vars::PAGE)
}

/// A loading placeholder row: 32 px high, with its bar centred.
pub(super) fn skeleton(theme: &Theme, _state: &Look<SidebarState>) -> Style {
    let gap = theme.get(&space::MD_VAR);
    style()
        .align_items(Align::Center)
        .gap(gap)
        .height(32)
        .padding_x(gap)
}

/// The wrapper of a menu row: a relative, full-width box.
pub(super) fn menu_item(_theme: &Theme, _state: &Look<SidebarItemState>) -> Style {
    style()
        .position(Position::Relative)
        .width(pct(100))
        .min_width(0)
}

/// The button of a menu row: the row's height and colors, with the accent when it is active.
pub(super) fn menu_button(theme: &Theme, state: &Look<SidebarItemState>) -> Style {
    let lit = state.active;
    let gap = theme.get(&space::MD_VAR);
    let lit_style = style()
        .background(vars::ACCENT)
        .color(vars::ACCENT_FOREGROUND);
    let s = style()
        .group()
        .flex_direction(if state.rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .align_items(Align::Center)
        .gap(gap)
        .width(pct(100))
        .height(state.size.height())
        .padding(gap)
        .overflow(Overflow::Hidden)
        .border_radius(vars::ITEM_RADIUS)
        .font_size(theme.get(&font::SIZE_SMALL_VAR))
        .font_weight(if lit { 500 } else { 400 })
        .color(if lit {
            vars::ACCENT_FOREGROUND
        } else {
            vars::FOREGROUND
        })
        .background(if lit {
            vars::ACCENT
        } else {
            Color::TRANSPARENT
        })
        .hover(lit_style.clone())
        .active(lit_style)
        .focus_visible(style().outline(2, vars::RING));
    let s = match (state.has_action, state.rtl) {
        (true, false) => s.padding_right(32),
        (true, true) => s.padding_left(32),
        _ => s,
    };
    if state.icon_only {
        let s = s.size(32, 32).flex_shrink(0);
        if state.size == SidebarMenuButtonSize::Lg {
            s.padding(0)
        } else {
            s
        }
    } else {
        s
    }
}

/// The label of a menu row: it fills the row, and starts at its reading side.
pub(super) fn menu_button_label(_theme: &Theme, state: &Look<SidebarItemState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .justify_content(if state.rtl {
            Justify::End
        } else {
            Justify::Start
        })
        .flex_grow(1)
        .min_width(0)
}

/// The count at the end of a menu row: a 20 px pill, tabular.
pub(super) fn menu_badge(theme: &Theme, state: &Look<SidebarItemState>) -> Style {
    let s = style()
        .position(Position::Absolute)
        .top(state.size.corner())
        .height(20)
        .min_width(20)
        .padding_x(theme.get(&space::SM_VAR))
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .font_size(theme.get(&font::SIZE_SMALL_VAR))
        .font_weight(500)
        .tabular_nums()
        .color(if state.active {
            vars::ACCENT_FOREGROUND
        } else {
            vars::FOREGROUND
        })
        .group_hover(style().color(vars::ACCENT_FOREGROUND));
    if state.rtl { s.left(4) } else { s.right(4) }
}

/// The label beside an icon-only row: the sidebar's inverted text.
pub(super) fn menu_tooltip(theme: &Theme, _state: &Look<SidebarItemState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(6)
        .max_width(320)
        .padding_x(12)
        .padding_y(6)
        .border_radius(vars::ITEM_RADIUS)
        .background(vars::FOREGROUND)
        .color(vars::BACKGROUND)
        .font_size(theme.get(&font::SIZE_SMALL_VAR))
}

/// The action at the end of a menu row, a 20 px square.
pub(super) fn menu_action(_theme: &Theme, state: &Look<SidebarItemState>) -> Style {
    let s = style()
        .position(Position::Absolute)
        .top(state.size.corner())
        .size(20, 20)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center);
    if state.rtl { s.left(4) } else { s.right(4) }
}

/// The sub list under a menu row: indented behind a rule.
pub(super) fn menu_sub(_theme: &Theme, state: &Look<SidebarItemState>) -> Style {
    let s = style()
        .flex_direction(FlexDirection::Column)
        .gap(4)
        .min_width(0)
        .margin_x(14)
        .padding_x(10)
        .padding_y(2);
    if state.rtl {
        s.border_right(1, vars::BORDER)
    } else {
        s.border_left(1, vars::BORDER)
    }
}

/// A link in a sub list: 28 px high, with the accent when it is the current page.
pub(super) fn sub_button(theme: &Theme, state: &Look<SidebarItemState>) -> Style {
    let lit = state.active;
    let gap = theme.get(&space::MD_VAR);
    let lit_style = style()
        .background(vars::ACCENT)
        .color(vars::ACCENT_FOREGROUND);
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(gap)
        .height(28)
        .min_width(0)
        .padding_x(gap)
        .overflow(Overflow::Hidden)
        .border_radius(vars::ITEM_RADIUS)
        .font_size(theme.get(&font::SIZE_SMALL_VAR))
        .color(if lit {
            vars::ACCENT_FOREGROUND
        } else {
            vars::FOREGROUND
        })
        .background(if lit {
            vars::ACCENT
        } else {
            Color::TRANSPARENT
        })
        .hover(lit_style.clone())
        .active(lit_style)
        .focus_visible(style().outline(2, vars::RING))
}
