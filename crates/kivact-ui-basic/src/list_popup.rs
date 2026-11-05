//! Role: The popup list look and list shared by selects, menus and comboboxes.

use crate::theme_access::{CHECK, list_row, lit_row, popover_surface, shadow_md, stroke_glyph};
use crate::tokens::Tokens;
use kivact::geometry::Point;
use kivact::interaction::{ListItemState, SelectApi};
use kivact::prelude::NodeBuilder;
use kivact::state::Value;
use kivact::view::{column, node, scroll, text};

use kivact::interaction::ListItem;
use kivact::style::{Direction, Style};
use kivact::view::{IntoView, NodeRef, Placement, View, for_each, overlay, show};

/// The popup list of select, menus, comboboxes and native selects:
/// [`popover_surface`] with `shadow-md` and no padding.
pub(crate) fn surface_style(p: &Tokens) -> Style {
    shadow_md(popover_surface(p)).direction(Direction::Column)
}

/// An option row of a list popup: [`list_row`] with `pr-8`, leaving room
/// for the `check` at the right, lit while active, pointed at or focused.
pub(crate) fn option_row(p: &Tokens, state: &ListItemState) -> Style {
    list_row(p, state.active)
        .padding_right(32)
        .hover(lit_row(p))
        .focus(lit_row(p))
}

/// [`option_row`] with the `check` painted at the right while selected,
/// for the lists this kit builds itself.
pub(crate) fn row_style(p: &Tokens, state: &ListItemState) -> Style {
    // The check is local paint, which a state style cannot recolor, so it
    // follows the row's state.
    let input = &state.input;
    let lit = (input.hovered || input.focused || state.active) && !input.disabled;
    let check = if lit {
        p.accent_foreground
    } else {
        p.popover_foreground
    };
    let selected = state.selected;
    option_row(p, state).paint(move |d, size| {
        if selected {
            let at = Point::new(size.width - 24.0, (size.height - 16.0) / 2.0);
            stroke_glyph(d, at, 16.0, CHECK, check);
        }
    })
}

/// The open list of `behavior`, anchored below `anchor` and as wide as it.
pub(crate) fn select_popup<K: kivact::state::ItemKey>(
    behavior: SelectApi<K>,
    options: Value<Vec<ListItem<K>>>,
    anchor: NodeRef,
    name: Value<String>,
) -> View {
    show(behavior.is_open(), move || {
        let part = behavior.content();
        let rows = for_each(
            options,
            |item| item.key().clone(),
            move |item| {
                let key = item.get().key().clone();
                let part = behavior.item(key);
                let state = part.state();
                node()
                    .with_node(|root| root.part(part))
                    .style(move || {
                        row_style(&kivact::style::use_tokens::<crate::Tokens>(), &state.get())
                    })
                    .child(text(move || item.get().label()))
            },
        );
        overlay(
            anchor,
            Placement::Below,
            scroll(column(rows)).style(move || {
                surface_style(&kivact::style::use_tokens::<crate::Tokens>()).max_height(320)
            }),
        )
        .with_node(|root| root.part(part))
        .name(name)
        .offset(4.0)
        .at_least_anchor_width()
    })
    .into_view()
}
