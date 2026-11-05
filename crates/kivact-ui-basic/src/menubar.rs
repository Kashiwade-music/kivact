//! Role: The Lyra menu bar: triggers of standard menus that move an open menu.

use kivact::interaction::MenuEntry;
use kivact::interaction::{MenuApi, use_menubar};
use kivact::prelude::NodeBuilder;
use kivact::state::memo;
use kivact::state::{IntoValue, callback};
use kivact::style::{Align, Color, Direction, style};
use kivact::view::IntoView;
use kivact::view::{HandlerResult, OverlayAlign, Placement, Role, View};
use kivact::view::{Node, node, row, text};
use kivact::view::{ReadingDirection, use_direction};

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
///                     MenuEntry::item("new", "New Tab").with_shortcut("Ctrl+T"),
///                     MenuEntry::separator(),
///                     MenuEntry::item("print", "Print..."),
///                 ],
///             ),
///             ("Edit".to_string(), vec![MenuEntry::item("undo", "Undo")]),
///         ],
///     ).on_activate(|_| {})
/// }
/// ```
/// A menu bar with application-owned activation handling.
pub struct Menubar<K: kivact::state::ItemKey> {
    root: Node,
    menus: kivact::state::Value<Vec<(String, Vec<MenuEntry<K>>)>>,
    choose: kivact::state::Callback<K>,
}
#[track_caller]
pub fn menubar<K: kivact::state::ItemKey>(
    menus: impl IntoValue<Vec<(String, Vec<MenuEntry<K>>)>>,
) -> Menubar<K> {
    Menubar {
        root: row(()),
        menus: menus.into_value(),
        choose: callback(|_| ()),
    }
}
impl<K: kivact::state::ItemKey> Menubar<K> {
    pub fn on_activate<R: HandlerResult>(mut self, choose: impl Fn(K) -> R + 'static) -> Self {
        self.choose = callback(choose);
        self
    }
}
impl<K: kivact::state::ItemKey> NodeBuilder for Menubar<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl<K: kivact::state::ItemKey> kivact::view::IntoView for Menubar<K> {
    fn into_view(self) -> View {
        let menus = self.menus;
        let choose = self.choose;
        let rtl = use_direction().get() == ReadingDirection::Rtl;
        let count = menus.with_untracked(Vec::len);
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
            let menu =
                kivact::interaction::use_menu(entries).on_activate(move |key| choose.try_call(key));
            handles.push(menu);
            parts.push((menu, title));
        }
        let bar = use_menubar(titles, handles);

        let mut children: Vec<View> = Vec::with_capacity(count);
        for (at, (menu, title)) in parts.into_iter().enumerate() {
            let behavior = menu;
            let shown = behavior.is_open();
            let trigger = node()
                .with_node(|root| root.part(bar.item(at)))
                // `Menu::attach` keeps the role the trigger has.
                .with_node(|root| root.role(Role::MenuItem))
                .name(title)
                .style(move || bar_trigger_style(shown.get()))
                .child(text(title));
            let anchor = kivact::view::node_ref();
            let trigger = trigger
                .with_node(|root| root.part(menu.trigger()))
                .node_ref(anchor)
                .with_node(|root| root.role(Role::MenuItem));
            children.push((trigger, menu_surface(menu, anchor, title, false)).into_view());
        }

        self.root
            .child(children)
            .with_node(|root| root.part(bar.bar()))
            .style(move || {
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
                    .border(1, kivact::style::use_tokens::<crate::Tokens>().border)
                    .font_size(12)
                    .color(kivact::style::use_tokens::<crate::Tokens>().foreground)
            })
            .into_view()
    }
}

fn bar_trigger_style(open: bool) -> kivact::style::Style {
    let p = kivact::style::use_tokens::<crate::Tokens>();
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
        .background(if open { p.muted } else { Color::TRANSPARENT })
        .hover(active.clone())
        .focus_visible(active)
}

/// Render a level under the existing menu authority.
fn menu_surface<K: kivact::state::ItemKey>(
    menu: MenuApi<K>,
    anchor: kivact::view::NodeRef,
    label: kivact::state::Value<String>,
    sub: bool,
) -> View {
    menu.show(move || {
        let rows = kivact::view::for_each(
            menu.entries(),
            |(row, _)| row.clone(),
            move |entry| menu_row(menu, entry),
        );
        kivact::view::overlay(
            anchor,
            if sub {
                Placement::End
            } else {
                Placement::Below
            },
            kivact::view::scroll(kivact::view::column(rows)).style(move || {
                crate::recipes_menu::bar_surface(&kivact::style::use_theme().get(), sub)
            }),
        )
        .align(OverlayAlign::Start)
        .offset(if sub { 0.0 } else { 8.0 })
        .with_node(|node| node.part(menu.content()).name(label))
    })
}

fn menu_row<K: kivact::state::ItemKey>(
    menu: MenuApi<K>,
    entry: kivact::state::Value<(kivact::interaction::MenuRow<K>, MenuEntry<K>)>,
) -> View {
    use kivact::interaction::MenuRow;
    match entry.get().0 {
        MenuRow::Item { key, kind } => menu_item(menu, key, kind, memo(move || entry.get().1)),
        MenuRow::Label(_) => text(move || entry.get().1.label())
            .style(
                style()
                    .padding(8)
                    .font_size(12)
                    .color(crate::ColorName::MutedForeground),
            )
            .into_view(),
        MenuRow::Separator(_) => node()
            .role(Role::Separator)
            .style(
                style()
                    .height(1)
                    .margin_y(4)
                    .background(crate::ColorName::Border),
            )
            .into_view(),
    }
}

fn menu_item<K: kivact::state::ItemKey>(
    menu: MenuApi<K>,
    key: K,
    kind: kivact::interaction::MenuItemKind,
    data: kivact::state::Value<MenuEntry<K>>,
) -> View {
    let anchor = kivact::view::node_ref();
    let item = menu_button(menu.item(key.clone()), data, kind, anchor);
    let reason = memo(move || {
        data.with(|entry| {
            if entry.is_disabled() {
                entry.disabled_reason()
            } else {
                None
            }
        })
    });
    let retain_disabled = kivact::style::use_theme()
        .get()
        .ext::<kivact::interaction::InteractionPolicy>()
        .menu
        .disabled_focus
        == kivact::interaction::DisabledMenuFocus::Retain;
    let item = if retain_disabled {
        kivact::ui::tooltip(move || reason.get().unwrap_or_default())
            .enabled(move || {
                reason.with(|reason| reason.as_ref().is_some_and(|reason| !reason.is_empty()))
            })
            .placement(Placement::End)
            .attach(item)
    } else {
        item.into_view()
    };
    let child = (kind == kivact::interaction::MenuItemKind::Submenu).then(|| {
        menu_surface(
            menu.submenu(key),
            anchor,
            memo(move || data.get().label()),
            true,
        )
    });
    (item, child).into_view()
}

fn menu_button<K: kivact::state::ItemKey>(
    part: kivact::custom::Part<kivact::interaction::MenuItemState>,
    data: kivact::state::Value<MenuEntry<K>>,
    kind: kivact::interaction::MenuItemKind,
    anchor: kivact::view::NodeRef,
) -> Node {
    let state = part.state();
    row((
        menu_mark(data, state),
        kivact::view::show(
            move || data.get().is_inset(),
            || node().style(style().width(16).shrink(0)),
        ),
        kivact::view::dynamic(
            move || data.get().icon().cloned(),
            |glyph| glyph.map(kivact::view::icon),
        ),
        kivact::view::column((
            text(move || data.get().label()),
            kivact::view::dynamic(
                move || data.get().description(),
                |description| {
                    description.map(|description| {
                        text(description).style(style().color(crate::ColorName::MutedForeground))
                    })
                },
            ),
        ))
        .style(style().grow(1)),
        text(move || data.get().shortcut().to_owned()).style(move || {
            crate::recipes_menu::bar_shortcut(&kivact::style::use_theme().get(), state.get())
        }),
        (kind == kivact::interaction::MenuItemKind::Submenu)
            .then(|| kivact::view::icon("chevron-right").style(style().size(16, 16))),
    ))
    .part(part)
    .node_ref(anchor)
    .style(move || crate::recipes_menu::bar_item(&kivact::style::use_theme().get(), state.get()))
}

fn menu_mark<K: kivact::state::ItemKey>(
    data: kivact::state::Value<MenuEntry<K>>,
    state: kivact::state::Value<kivact::interaction::MenuItemState>,
) -> Node {
    node()
        .style(move || {
            crate::recipes_menu::bar_check(&kivact::style::use_theme().get(), state.get())
        })
        .child(kivact::view::dynamic(
            move || data.get().check(),
            |check| {
                use kivact::interaction::MenuCheck;
                match check {
                    Some(MenuCheck::Radio(true)) => Some(kivact::view::icon("menu-radio")),
                    Some(MenuCheck::Checkbox(true)) => Some(kivact::view::icon("check")),
                    _ => None,
                }
            },
        ))
}
