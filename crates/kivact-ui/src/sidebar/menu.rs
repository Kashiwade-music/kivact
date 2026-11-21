//! Role: The sidebar's groups, menus, menu buttons, sub-menus, separator, input and skeleton.

use super::look::{SidebarItemState, SidebarTheme};
use super::recipes::themed;
use super::state::{SidebarCollapsible, SidebarCx, SidebarMenuButtonSize, current, part_state};
use crate::skeleton::skeleton;
use crate::text_input;
use kivact_core::custom::{Caller, PartState, part};
use kivact_core::state::{Callback, Signal, Value, callback, memo};
use kivact_core::style::{pct, style};
use kivact_core::view::{
    Element, IntoView, Node, NodeRef, Orientation, OverlayAlign, Placement, ReadingDirection, Role,
    View, column, icon, node, overlay, row, show, text, use_direction,
};
use kivact_interaction::{TooltipApi, use_tooltip};
use kivact_macros::component;

/// Attach hover, focus and press tracking to `root`.
///
/// With `on_press` the node becomes a focusable `role` that reports
/// activation (click, Enter, Space, assistive technology); without it the
/// state still follows the pointer, but the node takes no focus.
pub(super) fn pressable(
    root: Node,
    role: Role,
    on_press: Option<Callback<()>>,
) -> (Node, Value<PartState>) {
    let attachment = part(|input| input);
    let state = attachment.state();
    let root = root.with_node(|root| root.part(attachment));
    let root = match on_press {
        Some(callback) => root
            .with_node(|root| root.role(role))
            .focusable(true)
            .on_press(callback),
        None => root,
    };
    (root, state)
}

/// A labelled section of the sidebar (`SidebarGroup`).
///
/// The label, the description and the action are hidden while the sidebar is
/// an icon column. With `label_hidden` the label is transparent but keeps
/// the group's name, so assistive technology still reads it.
#[component]
pub fn sidebar_group(
    label: Value<String>,
    action: Option<View>,
    content: Option<View>,
    description: Option<Value<String>>,
    #[prop(default = false)] label_hidden: Value<bool>,
) -> impl Element {
    let state = part_state(current());
    let heading = themed(
        node()
            .hidden(move || state.get().icon_only)
            .style(move || style().opacity(if label_hidden.get() { 0.0 } else { 1.0 }))
            .child(text(label).style(style().single_line())),
        |r: &SidebarTheme| r.group_label_recipe(),
        state,
    );
    let description = description.map(|description| {
        themed(
            node()
                .hidden(move || state.get().icon_only)
                .child(text(description)),
            |r: &SidebarTheme| r.group_description_recipe(),
            state,
        )
    });
    let action = action.map(|action| {
        themed(
            node().hidden(move || state.get().icon_only).child(action),
            |r: &SidebarTheme| r.group_action_recipe(),
            state,
        )
    });
    let content = content.map(|content| {
        themed(
            node().child(content),
            |r: &SidebarTheme| r.group_content_recipe(),
            state,
        )
    });
    themed(
        node().with_node(|root| root.role(Role::Group)).name(label),
        |r: &SidebarTheme| r.group_recipe(),
        state,
    )
    .child(heading)
    .child(description)
    .child(action)
    .child(content)
}

/// A list of menu buttons: a `Role::List` column of full-width rows.
#[component]
pub fn sidebar_menu(children: Vec<View>) -> impl Element {
    let state = part_state(current());
    themed(
        column(children).with_node(|root| root.role(Role::List)),
        |r: &SidebarTheme| r.menu_recipe(),
        state,
    )
}

/// A row of a [`sidebar_menu`]: a button with an optional icon, badge, action and sub list.
///
/// While the sidebar is an icon column only the icon shows and hovering the
/// button shows the label in a tooltip on its right. The height is the
/// `SidebarMenuButtonSize` given with `.variant(..)`. `active_icon` replaces
/// `icon` while the row is active. `end` sits after the label, before the
/// badge. `sub_open` makes the row a toggle: a press flips it, and it shows
/// the sub list and a chevron while it is open. Without it the sub list is
/// always shown.
#[component]
pub fn sidebar_menu_button(
    label: Value<String>,
    icon: Option<View>,
    #[prop(default = false)] active: Value<bool>,
    badge: Option<Value<String>>,
    action: Option<View>,
    on_press: Option<Callback<()>>,
    sub: Option<View>,
    active_icon: Option<View>,
    end: Option<View>,
    sub_open: Option<Signal<bool>>,
    #[caller] caller: Caller,
) -> impl Element {
    let cx = current();
    let variants = caller.variants();
    let rtl = use_direction() == ReadingDirection::Rtl;
    let has_action = action.is_some();
    let item = memo(move || SidebarItemState {
        active: active.get(),
        icon_only: cx.is_some_and(SidebarCx::icon_only),
        has_action,
        rtl,
        size: variants.get::<SidebarMenuButtonSize>().unwrap_or_default(),
        expanded: sub_open.map(|open| open.get()),
    });
    let press = on_press.unwrap_or_else(|| callback(|()| {}));
    let press = match sub_open {
        Some(open) => callback(move |()| {
            press.call(());
            open.set(!open.get());
        }),
        None => press,
    };
    let root = node().name(label).selected(active);
    let root = match sub_open {
        Some(open) => root.expanded(open),
        None => root,
    };
    let (button, _) = pressable(root, Role::Button, Some(press));
    let button = themed(button, |r: &SidebarTheme| r.menu_button_recipe(), item);
    let tip = cx
        .is_some_and(|cx| cx.collapsible == SidebarCollapsible::Icon)
        .then(|| use_tooltip(true));
    let tip_part = tip.map(|tip| tip.trigger());
    let tip_anchor = tip_part.as_ref().map(|_| kivact_core::view::node_ref());
    let button = match tip_part {
        Some(part) => button
            .with_node(|root| root.part(part))
            .node_ref(tip_anchor.expect("invariant: tooltip part has its anchor")),
        None => button,
    };
    let button = match active_icon {
        None => button.child(icon),
        Some(active_icon) => button
            .child(icon.map(|icon| node().hidden(move || item.get().active).child(icon)))
            .child(node().hidden(move || !item.get().active).child(active_icon)),
    };
    let button = button
        .child(menu_button_label(label, item))
        .child(end.map(|end| menu_button_end(end, item)))
        .child(sub_open.map(|_| menu_button_chevron(item)))
        .child(badge.map(|badge| menu_badge(badge, item)));
    let tooltip = tip
        .zip(tip_anchor)
        .map(|(tip, anchor)| menu_tooltip(tip, anchor, label, item));
    let action = action.map(|action| menu_action(action, item));
    let sub = sub.map(|sub| menu_sub(sub, item));
    themed(node(), |r: &SidebarTheme| r.menu_item_recipe(), item)
        .with_node(|root| root.role(Role::ListItem))
        .child(button)
        .child(tooltip)
        .child(action)
        .child(sub)
}

/// A link inside the sub-list of a [`sidebar_menu_button`].
#[component]
pub fn sidebar_menu_sub_button(
    label: Value<String>,
    #[prop(default = false)] active: Value<bool>,
    on_press: Option<Callback<()>>,
) -> impl Element {
    let item = memo(move || SidebarItemState {
        active: active.get(),
        icon_only: false,
        has_action: false,
        rtl: false,
        size: SidebarMenuButtonSize::Default,
        expanded: None,
    });
    let press = on_press.unwrap_or_else(|| callback(|()| {}));
    let (link, _) = pressable(node().name(label).selected(active), Role::Link, Some(press));
    themed(link, |r: &SidebarTheme| r.sub_button_recipe(), item)
        .child(text(label).style(style().single_line()))
}

/// A horizontal rule in the sidebar's border color.
#[component]
pub fn sidebar_separator() -> impl Element {
    themed(
        node()
            .with_node(|root| root.role(Role::Separator))
            .orientation(Orientation::Horizontal),
        |r: &SidebarTheme| r.separator_recipe(),
        part_state(current()),
    )
}

/// A text field for the sidebar: the standard input on the page color.
///
/// Name it with `.name(..)` (or `.label(..)` on the field it wraps); the
/// caller's name also names the inner field.
#[component]
pub fn sidebar_input(value: Signal<String>, #[caller] caller: Caller) -> impl Element {
    let field = text_input(value);
    let field = match caller.name() {
        Some(name) => field.label(name),
        None => field,
    };
    themed(
        field,
        |r: &SidebarTheme| r.input_recipe(),
        part_state(current()),
    )
}

/// A loading placeholder for a menu row: a 16 px bar of a width between
/// 50 % and 90 % that depends on where it was written.
#[component]
pub fn sidebar_menu_skeleton(#[caller] caller: Caller) -> impl Element {
    let at = caller.at();
    let width = 50.0 + ((at.line() * 7 + at.column()) % 41) as f64;
    themed(
        row(skeleton().style(style().height(16).flex_grow(1).max_width(pct(width)))),
        |r: &SidebarTheme| r.skeleton_recipe(),
        part_state(current()),
    )
}

/// The label of a menu row, hidden while the sidebar is an icon column.
fn menu_button_label(label: Value<String>, item: Value<SidebarItemState>) -> Node {
    themed(
        node()
            .hidden(move || item.get().icon_only)
            .child(text(label).style(style().single_line())),
        |r: &SidebarTheme| r.menu_button_label_recipe(),
        item,
    )
}

/// The element at the end of a row, before its badge.
fn menu_button_end(end: View, item: Value<SidebarItemState>) -> Node {
    themed(
        node().hidden(move || item.get().icon_only).child(end),
        |r: &SidebarTheme| r.menu_button_end_recipe(),
        item,
    )
}

/// The chevron of a row with a sub list: down while the list is open, right while it is shut.
fn menu_button_chevron(item: Value<SidebarItemState>) -> Node {
    let open = move || item.get().expanded == Some(true);
    themed(
        node()
            .hidden(move || {
                let state = item.get();
                state.icon_only || state.expanded.is_none()
            })
            .child(node().hidden(move || !open()).child(icon("chevron-down")))
            .child(node().hidden(open).child(icon("chevron-right"))),
        |r: &SidebarTheme| r.menu_button_chevron_recipe(),
        item,
    )
}

/// The count at the end of a row.
fn menu_badge(badge: Value<String>, item: Value<SidebarItemState>) -> Node {
    themed(
        node()
            .hidden(move || item.get().icon_only)
            .child(text(badge)),
        |r: &SidebarTheme| r.menu_badge_recipe(),
        item,
    )
}

/// The label shown to the right of an icon-only row while it is hovered.
fn menu_tooltip(
    tip: TooltipApi,
    anchor: NodeRef,
    label: Value<String>,
    item: Value<SidebarItemState>,
) -> impl IntoView {
    let shown = memo(move || tip.is_open().get() && item.get().icon_only);
    show(shown, move || {
        let surface = tip.content();
        overlay(
            anchor,
            Placement::End,
            themed(
                node().child(text(label)),
                |r: &SidebarTheme| r.menu_tooltip_recipe(),
                item,
            ),
        )
        .align(OverlayAlign::Center)
        .offset(4)
        .with_node(|root| root.part(surface))
        .name(label)
    })
}

/// The action at the end of a row.
fn menu_action(action: View, item: Value<SidebarItemState>) -> Node {
    themed(
        node().hidden(move || item.get().icon_only).child(action),
        |r: &SidebarTheme| r.menu_action_recipe(),
        item,
    )
}

/// The sub list under a row: shown unless the sidebar is an icon column or the row is shut.
fn menu_sub(sub: View, item: Value<SidebarItemState>) -> Node {
    themed(
        node()
            .with_node(|root| root.role(Role::List))
            .hidden(move || {
                let state = item.get();
                state.icon_only || state.expanded == Some(false)
            })
            .child(sub),
        |r: &SidebarTheme| r.menu_sub_recipe(),
        item,
    )
}
