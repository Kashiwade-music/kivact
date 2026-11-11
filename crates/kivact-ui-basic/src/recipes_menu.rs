//! Role: Lyra recipes for the standard menu: dropdown, use_context and menu bar looks.
//!
//! The theme's menu table is dropdown-menu.tsx. use_context-menu.tsx and
//! menubar.tsx differ in their widths and, in the menu bar, the place of the
//! check mark: a menu opened with `attach_context` draws its top level with
//! the `context_surface` recipe (`min-w-36`), and [`menubar`](crate::menubar)
//! gives its menus [`MenuVariant::Bar`].

use crate::ColorName;
use crate::theme_access::{list_row, popover_surface, shadow_lg, shadow_md};
use crate::tokens::Tokens;
use kivact::custom::PartState;
use kivact::interaction::{MenuItemState, PopupState};
use kivact::style::{Direction, Position, Style, Theme, pct, style};
use kivact::ui::MenuRecipes;

/// The measures that differ between the dropdown, use_context and bar menus.
#[derive(Clone, Copy)]
struct MenuLook {
    /// The least width of the top level (`min-w-32`, `min-w-36`).
    min_width: f64,
    /// The least width of a submenu.
    sub_min_width: f64,
    /// The menu bar's rows: check marks at the start, margins around
    /// separators and a wider inset (`pl-8`).
    bar: bool,
}

const DROPDOWN: MenuLook = MenuLook {
    min_width: 128.0,
    sub_min_width: 96.0,
    bar: false,
};
/// use_context-menu.tsx's top level (`min-w-36`).
const CONTEXT_MIN_WIDTH: f64 = 144.0;
const BAR: MenuLook = MenuLook {
    min_width: 144.0,
    sub_min_width: 128.0,
    bar: true,
};

/// The kind of a kit menu, which the menu recipes read from the menu's
/// `.variant(..)`: a menu without one is a dropdown or a use_context menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuVariant {
    /// menubar.tsx: wider levels and check marks at the start.
    Bar,
}

impl kivact::ui::Variant for MenuVariant {}

/// The measures of the kind of menu `look` belongs to.
fn kind<S>(look: &kivact::ui::Look<S>) -> MenuLook {
    match look.variant::<MenuVariant>() {
        Some(MenuVariant::Bar) => BAR,
        None => DROPDOWN,
    }
}

/// The theme's menu table: dropdown-menu.tsx, use_context-menu.tsx's top level,
/// and menubar.tsx for the menus with [`MenuVariant::Bar`].
pub(crate) fn menu(r: MenuRecipes) -> MenuRecipes {
    r.surface(|_| {
        |t: &Theme, s: &kivact::ui::Look<PopupState>| {
            shadow_md(surface(t)).min_width(kind(s).min_width)
        }
    })
    .context_surface(|_| {
        |t: &Theme, _: &kivact::ui::Look<PopupState>| {
            shadow_md(surface(t)).min_width(CONTEXT_MIN_WIDTH)
        }
    })
    .sub_surface(|_| {
        |t: &Theme, s: &kivact::ui::Look<PopupState>| {
            shadow_lg(surface(t)).min_width(kind(s).sub_min_width)
        }
    })
    .item(|_| |t: &Theme, s: &kivact::ui::Look<MenuItemState>| item(t, s, kind(s)))
    .separator(|_| {
        |t: &Theme, s: &kivact::ui::Look<PartState>| {
            let line = style()
                .height(1)
                .shrink(0)
                .background(t.ext::<Tokens>().border);
            if kind(s).bar { line.margin_y(4) } else { line }
        }
    })
    .label(|_| label)
    .shortcut(|_| shortcut)
    .description(|_| {
        |_: &Theme, _: &kivact::ui::Look<MenuItemState>| style().color(ColorName::MutedForeground)
    })
    .check(|_| |t: &Theme, s: &kivact::ui::Look<MenuItemState>| check(t, s, kind(s)))
    .submenu(|_| submenu)
    .inset(|_| {
        // `pl-7` (`pl-8` in the menu bar): the inset box, then the row's gap.
        |_: &Theme, s: &kivact::ui::Look<PartState>| {
            style().width(if kind(s).bar { 16 } else { 12 }).shrink(0)
        }
    })
}

/// `bg-popover text-popover-foreground ring-1 ring-foreground/10`.
fn surface(t: &Theme) -> Style {
    popover_surface(&t.ext::<Tokens>()).direction(Direction::Column)
}

/// `style` for a part inside a row (its shortcut, mark or chevron) while
/// the row is lit: open, or (while enabled) hovered or focused, which the
/// part reads from the row, a style group.
fn when_lit(s: &kivact::ui::Look<MenuItemState>, base: Style, lit: Style) -> Style {
    if s.is_expanded() {
        base.merge(&lit)
    } else if s.input.is_disabled() {
        base
    } else {
        base.group_hover(lit.clone()).group_focus(lit)
    }
}

/// `gap-2 py-2 px-2 text-xs`, `bg-accent text-accent-foreground` while
/// pointed at, focused or open; `text-destructive` with `bg-destructive/10`
/// for a destructive item. A check or radio item keeps room for its mark
/// at the end (`pr-8`), or at the start in the menu bar (`pl-8`).
fn item(t: &Theme, s: &kivact::ui::Look<MenuItemState>, look: MenuLook) -> Style {
    let p = t.ext::<Tokens>();
    let (left, right) = match (s.check().is_some(), look.bar) {
        (true, true) => (32.0, 8.0),
        (true, false) => (8.0, 32.0),
        (false, _) => (8.0, 8.0),
    };
    let lit_style = if s.destructive {
        style()
            .background(
                p.destructive
                    .with_alpha(if p.is_dark() { 0.2 } else { 0.1 }),
            )
            .color(p.destructive)
    } else {
        style().background(p.accent).color(p.accent_foreground)
    };
    // The row is the style group its shortcut, mark and chevron follow.
    let base = list_row(&p, false)
        .group()
        .padding_left(left)
        .padding_right(right)
        .color(if s.destructive {
            p.destructive
        } else {
            p.popover_foreground
        })
        .hover(lit_style.clone())
        .focus(lit_style.clone());
    if s.is_expanded() {
        base.merge(&lit_style)
    } else {
        base
    }
}

/// A heading: `px-2 py-2 text-xs text-muted-foreground`, with the inset box
/// before its text.
fn label(t: &Theme, _: &kivact::ui::Look<PartState>) -> Style {
    style()
        .direction(Direction::Row)
        .gap(8)
        .padding(8)
        .font_size(12)
        .text_wrap(false)
        .color(t.ext::<Tokens>().muted_foreground)
}

/// `ml-auto text-xs tracking-widest`, muted unless its row is lit.
fn shortcut(t: &Theme, s: &kivact::ui::Look<MenuItemState>) -> Style {
    let p = t.ext::<Tokens>();
    let lit = if s.destructive {
        p.destructive
    } else {
        p.accent_foreground
    };
    when_lit(
        s,
        crate::theme_access::menu_shortcut(&p, false),
        style().color(lit),
    )
}

/// The 16 px box of a check or radio item's `check`, at the row's end (at
/// its start in the menu bar), in the row's color.
fn check(t: &Theme, s: &kivact::ui::Look<MenuItemState>, look: MenuLook) -> Style {
    let p = t.ext::<Tokens>();
    let at = style()
        .position(Position::Absolute)
        .top(pct(50) - 8)
        .size(16, 16)
        .color(p.popover_foreground);
    let at = if look.bar { at.left(6) } else { at.right(8) };
    if s.destructive {
        at
    } else {
        when_lit(s, at, style().color(p.accent_foreground))
    }
}

/// The submenu's 16 px `chevron-right`, in the row's color.
fn submenu(t: &Theme, s: &kivact::ui::Look<MenuItemState>) -> Style {
    let p = t.ext::<Tokens>();
    when_lit(
        s,
        style().size(16, 16).shrink(0).color(p.popover_foreground),
        style().color(p.accent_foreground),
    )
}

/// Render the kit's menu bar using the same recipes as standard menus.
pub(crate) fn bar_surface(theme: &Theme, sub: bool) -> Style {
    let base = surface(theme);
    if sub {
        shadow_lg(base).min_width(BAR.sub_min_width)
    } else {
        shadow_md(base).min_width(BAR.min_width)
    }
}
pub(crate) fn bar_item(theme: &Theme, state: MenuItemState) -> Style {
    item(
        theme,
        &kivact::ui::Look::new(state).with_variant(MenuVariant::Bar),
        BAR,
    )
}
pub(crate) fn bar_check(theme: &Theme, state: MenuItemState) -> Style {
    check(
        theme,
        &kivact::ui::Look::new(state).with_variant(MenuVariant::Bar),
        BAR,
    )
}
pub(crate) fn bar_shortcut(theme: &Theme, state: MenuItemState) -> Style {
    shortcut(
        theme,
        &kivact::ui::Look::new(state).with_variant(MenuVariant::Bar),
    )
}
