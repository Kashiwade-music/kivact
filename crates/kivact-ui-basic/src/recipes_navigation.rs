//! Role: Lyra recipes of the navigation parts: tabs.
//!
//! The looks of tabs.tsx's `default` variant. The `line` variant joins this
//! module later, selected by the tabs' [`TabsVariant`](kivact::ui::TabsVariant).

use crate::color::{BACKGROUND, FOREGROUND, INPUT, MUTED, MUTED_FOREGROUND};
use crate::recipe_parts::wide_focus_ring;
use kivact::interaction::{TabListState, TabState};
use kivact::style::{Align, Color, FlexDirection, Justify, Style, Theme, ThemeMode, pct, style};
use kivact::ui::theme::{Look, TabsTheme};
use kivact::view::Orientation;

/// `flex-1 text-xs/relaxed outline-none`: a panel draws no focus ring; the
/// root's `gap-2` is the panel's leading padding.
fn tab_panel(_: &Theme, s: &Look<TabState>, _base: Style) -> Style {
    let base = style()
        .flex_grow(1.0)
        .font_size(12.0)
        .line_height(1.625)
        .color(FOREGROUND)
        .focus_visible(style().outline_none());
    if s.orientation == Orientation::Vertical {
        base.padding_left(8.0)
    } else {
        base.padding_top(8.0)
    }
}

/// The tab list: `bg-muted` pill, a row of `h-8`, or a column when vertical.
fn tabs_list(_: &Theme, s: &Look<TabListState>, _base: Style) -> Style {
    let base = style()
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .align_self(Align::Start)
        .padding(3.0)
        .border_radius(0.0)
        .background(MUTED)
        .color(MUTED_FOREGROUND);
    if s.orientation == Orientation::Vertical {
        // `group-data-vertical/tabs:h-fit flex-col`.
        base.flex_direction(FlexDirection::Column)
    } else {
        base.flex_direction(FlexDirection::Row).height(32.0)
    }
}

fn tab(t: &Theme, s: &Look<TabState>, _base: Style) -> Style {
    let dark = t.mode() == ThemeMode::Dark;
    let rest = if dark {
        MUTED_FOREGROUND
    } else {
        FOREGROUND.with_alpha(0.6)
    };
    let base = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(6.0)
        .padding_x(6.0)
        .border_radius(0.0)
        .border(1.0, Color::TRANSPARENT)
        .font_size(12.0)
        .font_weight(500)
        .text_wrap(false)
        .color(rest)
        .hover(style().color(FOREGROUND));
    let base = if s.orientation == Orientation::Vertical {
        // `group-data-vertical/tabs:w-full justify-start py-[calc(--spacing(1.25))]`.
        base.width(pct(100.0))
            .justify_content(Justify::Start)
            .padding_y(5.0)
    } else {
        // `h-[calc(100%-1px)] flex-1 py-0.5`.
        base.flex_grow(1.0).height(pct(100.0) - 1.0).padding_y(2.0)
    };
    let base = if s.selected {
        let (fill, border) = if dark {
            (INPUT.with_alpha(0.3), INPUT)
        } else {
            (BACKGROUND, Color::TRANSPARENT)
        };
        base.background(fill).border_color(border).color(FOREGROUND)
    } else {
        base
    };
    // `focus-visible:ring-[3px] ring-ring/50 outline-1 outline-ring`.
    base.focus_visible(wide_focus_ring())
}

/// The navigation recipes: the tabs.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        TabsTheme::default()
            .list(tabs_list)
            .tab(tab)
            .panel(tab_panel),
    )
}
