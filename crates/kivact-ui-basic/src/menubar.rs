//! Role: The Lyra menu bar: triggers of standard menus that move an open menu.

use crate::theme_access::tokens;
use kivact::control::{Node, node, row, text};
use kivact::interaction::{MenuApi, use_menubar};
use kivact::prelude::NodeBuilder;
use kivact::style::{Align, Color, Direction, style};
use kivact::ui::MenuEntry;
use kivact::view::{HandlerResult, IntoValue, OverlayAlign, Placement, Role, View, callback};
use kivact::view::{ReadingDirection, current_direction};
use kivact::{memo, ui};
use std::hash::Hash;

/// A menu bar (menubar.tsx): a row of triggers of standard menus.
///
/// `menus` pairs each trigger's title with its [`MenuEntry`] rows. The number
/// of menus is read once, when the bar is built; titles and rows follow the
/// value. The arrow keys (Left and Right, mirrored in a right-to-left
/// [`direction`](kivact::view::provide_direction)) move the focus between
/// the triggers, Home and End to the first and last; in an open menu they
/// close that menu and open the neighbour, so a menu stays open while
/// moving. With a menu open, resting the pointer on another trigger opens
/// that menu. The rows behave as in every standard menu
/// ([`kivact::ui::menu`]), drawn in menubar.tsx's look (check marks at the
/// start).
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn bar() -> impl IntoView {
///     basic::menubar(
///         vec![
///             (
///                 "File".to_string(),
///                 vec![
///                     MenuEntry::item("new", "New Tab").shortcut("Ctrl+T"),
///                     MenuEntry::separator(),
///                     MenuEntry::item("print", "Print..."),
///                 ],
///             ),
///             ("Edit".to_string(), vec![MenuEntry::item("undo", "Undo")]),
///         ],
///         |_| {},
///     )
/// }
/// ```
#[track_caller]
pub fn menubar<K: Clone + Eq + Hash + 'static, R: HandlerResult>(
    menus: impl IntoValue<Vec<(String, Vec<MenuEntry<K>>)>>,
    choose: impl Fn(K) -> R + 'static,
) -> Node {
    let menus = menus.into_value();
    let rtl = current_direction() == ReadingDirection::Rtl;
    let count = menus.with_untracked(Vec::len);
    let choose = callback(choose);
    let titles = memo(move || {
        menus.with(|all| {
            all.iter()
                .map(|(title, _)| title.clone())
                .collect::<Vec<_>>()
        })
    });
    let mut handles: Vec<MenuApi<K>> = Vec::with_capacity(count);
    let mut parts = Vec::with_capacity(count);
    for at in 0..count {
        let title = memo(move || {
            menus.with(|all| {
                all.get(at)
                    .map(|(title, _)| title.clone())
                    .unwrap_or_default()
            })
        });
        let entries = memo(move || {
            menus.with(|all| {
                all.get(at)
                    .map(|(_, rows)| rows.clone())
                    .unwrap_or_default()
            })
        });
        let menu = ui::menu(title, entries, move |key| choose.try_call(key))
            .placement(Placement::Below)
            .align(OverlayAlign::Start)
            .offset(8.0)
            .variant(crate::recipes_menu::MenuVariant::Bar);
        handles.push(menu.behavior());
        parts.push((menu, title));
    }
    let bar = use_menubar(titles, handles);

    let mut children: Vec<View> = Vec::with_capacity(count);
    for (at, (menu, title)) in parts.into_iter().enumerate() {
        let behavior = menu.behavior();
        let shown = behavior.shown();
        let trigger = node()
            .part(bar.item(at))
            // `Menu::attach` keeps the role the trigger has.
            .role(Role::MenuItem)
            .name(title)
            .style(move || {
                let p = tokens();
                let active = style().background(p.muted);
                style()
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .padding_x(6)
                    .padding_y(3.2)
                    .radius(0)
                    .font_size(12)
                    .weight(500)
                    .text_wrap(false)
                    .color(p.foreground)
                    .background(if shown.get() {
                        p.muted
                    } else {
                        Color::TRANSPARENT
                    })
                    .hover(active.clone())
                    .focus_visible(active)
            })
            .child(text(title));
        children.push(menu.attach(trigger));
    }

    row(children).part(bar.bar()).style(move || {
        style()
            .direction(if rtl {
                Direction::RowReverse
            } else {
                Direction::Row
            })
            .align_items(Align::Center)
            .gap(2)
            .height(32)
            .padding(5)
            .radius(0)
            .border(1, tokens().border)
            .font_size(12)
            .color(tokens().foreground)
    })
}
