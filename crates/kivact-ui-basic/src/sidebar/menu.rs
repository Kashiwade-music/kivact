//! Role: The sidebar's groups, menu buttons, sub-menus, input, separator and skeleton.

use super::state::{SidebarCollapsible, SidebarCx, current, deferred};
use crate::parts::pressable;
use crate::primitives::skeleton;
use crate::slot::{Slot, slot};
use crate::theme_access::tokens;
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::interaction::use_tooltip;
use kivact::primitive::NodeExt;
use kivact::style::{
    Align, Color, Direction, Justify, Overflow, Position, current_theme, pct, style,
};
use kivact::ui::{TextInput, text_input};
use kivact::view::HandlerResult;
use kivact::view::{
    IntoText, IntoValue, IntoView, Orientation, OverlayAlign, Placement, Role, View, callback,
    overlay, show,
};
use kivact::view::{ReadingDirection, current_direction};
use kivact::{Binding, Callback, Value};

fn icon_only(cx: Option<SidebarCx>) -> impl Fn() -> bool + Copy + 'static {
    move || cx.is_some_and(|cx| cx.icon_only())
}

/// A labelled section of the sidebar (`SidebarGroup`). Created by [`sidebar_group`].
pub struct SidebarGroup {
    root: Node,
    label: Value<String>,
    action: Option<Slot>,
    content: Option<Slot>,
}

/// A group titled `label`.
///
/// The label and the action are hidden while the sidebar is an icon column.
#[track_caller]
pub fn sidebar_group(label: impl IntoText) -> SidebarGroup {
    SidebarGroup {
        root: node().style(
            style()
                .position(Position::Relative)
                .direction(Direction::Column)
                .width(pct(100))
                .min_width(0)
                .padding(8),
        ),
        label: label.into_text_value(),
        action: None,
        content: None,
    }
}

impl SidebarGroup {
    /// A control at the end of the label's row, such as an add button (20 px square).
    pub fn action<V: IntoView>(mut self, action: impl FnOnce() -> V + 'static) -> Self {
        self.action = Some(slot(action));
        self
    }

    /// The group's body, usually a [`sidebar_menu`].
    pub fn content<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.content = Some(slot(content));
        self
    }
}

impl NodeBuilder for SidebarGroup {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SidebarGroup {
    fn into_view(self) -> View {
        deferred(move || {
            let hidden = icon_only(current());
            let label = self.label;
            let heading = node()
                .hidden(hidden)
                .style(move || {
                    let p = tokens();
                    style()
                        .direction(Direction::Row)
                        .align_items(Align::Center)
                        .shrink(0)
                        .height(32)
                        .padding_x(8)
                        .radius(0)
                        .font_size(12)
                        .color(p.sidebar_foreground.with_alpha(0.7))
                })
                .child(text(label).style(style().single_line()));
            let action = self.action.map(|action| {
                node()
                    .hidden(hidden)
                    .style(
                        style()
                            .position(Position::Absolute)
                            .top(14)
                            .right(12)
                            .size(20, 20)
                            .direction(Direction::Row)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center),
                    )
                    .child(action)
            });
            let content = self.content.map(|content| {
                node()
                    .style(
                        style()
                            .direction(Direction::Column)
                            .width(pct(100))
                            .font_size(12),
                    )
                    .child(content)
            });
            self.root.child(heading).child(action).child(content)
        })
    }
}

/// A list of menu buttons (`SidebarMenu`): a `Role::List` column of full-width rows.
#[track_caller]
pub fn sidebar_menu(children: impl IntoView) -> Node {
    column(children)
        .role(Role::List)
        .style(style().width(pct(100)).min_width(0))
}

/// The height of a [`SidebarMenuButton`] (`size` of sidebar.tsx).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SidebarMenuButtonSize {
    /// 32 px.
    #[default]
    Default,
    /// 28 px.
    Sm,
    /// 48 px, for rows with a two-line label or an avatar.
    Lg,
}

impl SidebarMenuButtonSize {
    fn height(self) -> f32 {
        match self {
            SidebarMenuButtonSize::Default => 32.0,
            SidebarMenuButtonSize::Sm => 28.0,
            SidebarMenuButtonSize::Lg => 48.0,
        }
    }

    /// The top of the badge and the action, centered in the row.
    fn corner(self) -> f32 {
        match self {
            SidebarMenuButtonSize::Default => 6.0,
            SidebarMenuButtonSize::Sm => 4.0,
            SidebarMenuButtonSize::Lg => 10.0,
        }
    }
}

/// A row of a [`sidebar_menu`] (`SidebarMenuItem` with `SidebarMenuButton`).
/// Created by [`sidebar_menu_button`].
pub struct SidebarMenuButton {
    root: Node,
    label: Value<String>,
    icon: Option<View>,
    active: Value<bool>,
    badge: Option<Value<String>>,
    action: Option<Slot>,
    size: SidebarMenuButtonSize,
    on_press: Option<Callback<()>>,
    sub: Option<Slot>,
}

/// What the root style of a [`SidebarMenuButton`] reads.
#[derive(Clone, Copy)]
struct ButtonCfg {
    cx: Option<SidebarCx>,
    active: Value<bool>,
    size: SidebarMenuButtonSize,
    has_action: bool,
}

/// A menu button labelled `label`.
///
/// While the sidebar is an icon column only the icon shows and hovering the
/// button shows the label in a tooltip on its right.
#[track_caller]
pub fn sidebar_menu_button(label: impl IntoText) -> SidebarMenuButton {
    let label = label.into_text_value();
    let active = false.into_value();
    SidebarMenuButton {
        root: node().name(label).selected(active),
        label,
        icon: None,
        active,
        badge: None,
        action: None,
        size: SidebarMenuButtonSize::Default,
        on_press: None,
        sub: None,
    }
}

impl SidebarMenuButton {
    /// Show `icon` (16 px) before the label.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    /// Mark the button as the current page: accent fill and medium weight.
    pub fn active(mut self, active: impl IntoValue<bool>) -> Self {
        let active = active.into_value();
        self.active = active;
        self.root = self.root.selected(active);
        self
    }

    /// Show `text` as a count at the end of the row.
    pub fn badge(mut self, text: impl IntoText) -> Self {
        self.badge = Some(text.into_text_value());
        self
    }

    /// A control at the end of the row, such as a "more" button (20 px square).
    pub fn action<V: IntoView>(mut self, action: impl FnOnce() -> V + 'static) -> Self {
        self.action = Some(slot(action));
        self
    }

    /// Choose the height.
    pub fn size(mut self, size: SidebarMenuButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Report presses (click, Enter, Space) to `f`.
    pub fn on_press<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.on_press = Some(callback(move |_| f()));
        self
    }

    /// Show `children` (usually [`sidebar_menu_sub_button`]s) under the button,
    /// indented behind a rule. The list is hidden while the sidebar is an icon column.
    #[allow(
        clippy::should_implement_trait,
        reason = "the shadcn part is called sub; this is not an operator"
    )]
    pub fn sub<V: IntoView>(mut self, children: impl FnOnce() -> V + 'static) -> Self {
        self.sub = Some(slot(children));
        self
    }
}

impl NodeBuilder for SidebarMenuButton {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SidebarMenuButton {
    fn into_view(self) -> View {
        deferred(move || build_button(self))
    }
}

/// The look of the button row for the sidebar's current state.
fn button_style(cfg: ButtonCfg, rtl: bool) -> kivact::style::Style {
    let p = tokens();
    let active = cfg.active;
    let lit = active.get();
    let lit_style = style()
        .background(p.sidebar_accent)
        .color(p.sidebar_accent_foreground);
    let collapsed = cfg.cx.is_some_and(|cx| cx.icon_only());
    // The row is the style group its badge follows.
    let s = style()
        .group()
        .direction(if rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .align_items(Align::Center)
        .gap(8)
        .width(pct(100))
        .height(cfg.size.height())
        .padding(8)
        .overflow(Overflow::Hidden)
        .radius(0)
        .font_size(12)
        .weight(if active.get() { 500 } else { 400 })
        .color(if lit {
            p.sidebar_accent_foreground
        } else {
            p.sidebar_foreground
        })
        .background(if lit {
            p.sidebar_accent
        } else {
            Color::TRANSPARENT
        })
        .hover(lit_style.clone())
        .pressed(lit_style)
        .focus_visible(style().ring(2, p.sidebar_ring));
    let s = match (cfg.has_action, rtl) {
        (true, false) => s.padding_right(32),
        (true, true) => s.padding_left(32),
        _ => s,
    };
    if collapsed {
        let s = s.size(32, 32).shrink(0);
        if cfg.size == SidebarMenuButtonSize::Lg {
            s.padding(0)
        } else {
            s
        }
    } else {
        s
    }
}

fn build_button(this: SidebarMenuButton) -> impl IntoView {
    let cx = current();
    let hidden = icon_only(cx);
    let rtl = current_direction() == ReadingDirection::Rtl;
    let SidebarMenuButton {
        root,
        label,
        icon,
        active,
        badge,
        action,
        size,
        on_press,
        sub,
    } = this;
    let has_action = action.is_some();
    let press = on_press.unwrap_or_else(|| callback(|()| {}));
    let (button, _) = pressable(root, Role::Button, Some(press));
    let cfg = ButtonCfg {
        cx,
        active,
        size,
        has_action,
    };
    let button = button.base_style(move || button_style(cfg, rtl));

    let badge = badge.map(|badge| {
        node()
            .hidden(hidden)
            .style(move || {
                let p = tokens();
                let s = style()
                    .position(Position::Absolute)
                    .top(size.corner())
                    .height(20)
                    .min_width(20)
                    .padding_x(4)
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .font_size(12)
                    .weight(500)
                    .tabular_nums()
                    .color(if active.get() {
                        p.sidebar_accent_foreground
                    } else {
                        p.sidebar_foreground
                    })
                    .group_hover(style().color(p.sidebar_accent_foreground));
                if rtl { s.left(4) } else { s.right(4) }
            })
            .child(text(badge))
    });
    let tip = (cx.is_some_and(|cx| cx.collapsible == SidebarCollapsible::Icon)).then(use_tooltip);
    let tip_part = tip.map(|tip| tip.trigger());
    let tip_anchor = tip_part.as_ref().map(|part| part.node_ref());
    let button = match tip_part {
        Some(part) => button.part(part),
        None => button,
    };
    let button = button
        .child(icon)
        .child(
            node()
                .hidden(hidden)
                .style(
                    style()
                        .direction(Direction::Row)
                        .justify_content(if rtl { Justify::End } else { Justify::Start })
                        .grow(1)
                        .min_width(0),
                )
                .child(text(label).style(style().single_line())),
        )
        .child(badge);

    let tooltip_view = tip.zip(tip_anchor).map(|(tip, anchor)| {
        let shown = kivact::memo(move || tip.shown().get() && hidden());
        show(shown, move || {
            let surface = tip.content();
            let surface_state = surface.state();
            overlay(
                anchor,
                Placement::End,
                node()
                    .style(move || {
                        crate::recipes_popup::tooltip(
                            &current_theme(),
                            &kivact::ui::Look::new(surface_state.get()),
                        )
                    })
                    .child(text(label)),
            )
            .align(OverlayAlign::Center)
            .offset(4)
            .part(surface)
            .name(label)
        })
    });

    let action = action.map(|action| {
        node()
            .hidden(hidden)
            .style(move || {
                let s = style()
                    .position(Position::Absolute)
                    .top(size.corner())
                    .size(20, 20)
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center);
                if rtl { s.left(4) } else { s.right(4) }
            })
            .child(action)
    });
    let sub = sub.map(|sub| {
        node()
            .role(Role::List)
            .hidden(hidden)
            .style(move || {
                let p = tokens();
                let s = style()
                    .direction(Direction::Column)
                    .gap(4)
                    .min_width(0)
                    .margin_x(14)
                    .padding_x(10)
                    .padding_y(2);
                if rtl {
                    s.border_right(1, p.sidebar_border)
                } else {
                    s.border_left(1, p.sidebar_border)
                }
            })
            .child(sub)
    });
    node()
        .role(Role::ListItem)
        .style(
            style()
                .position(Position::Relative)
                .width(pct(100))
                .min_width(0),
        )
        .child(button)
        .child(tooltip_view)
        .child(action)
        .child(sub)
}

/// A link inside the sub-list of a [`SidebarMenuButton`] (`SidebarMenuSubButton`).
/// Created by [`sidebar_menu_sub_button`].
pub struct SidebarMenuSubButton {
    root: Node,
    label: Value<String>,
    active: Value<bool>,
    on_press: Option<Callback<()>>,
}

/// A sub-menu link labelled `label` (28 px high).
#[track_caller]
pub fn sidebar_menu_sub_button(label: impl IntoText) -> SidebarMenuSubButton {
    let label = label.into_text_value();
    let active = false.into_value();
    SidebarMenuSubButton {
        root: node().name(label).selected(active),
        label,
        active,
        on_press: None,
    }
}

impl SidebarMenuSubButton {
    /// Mark the link as the current page.
    pub fn active(mut self, active: impl IntoValue<bool>) -> Self {
        let active = active.into_value();
        self.active = active;
        self.root = self.root.selected(active);
        self
    }

    /// Report presses (click, Enter, Space) to `f`.
    pub fn on_press<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.on_press = Some(callback(move |_| f()));
        self
    }
}

impl NodeBuilder for SidebarMenuSubButton {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SidebarMenuSubButton {
    fn into_view(self) -> View {
        let label = self.label;
        let (link, _) = pressable(
            self.root,
            Role::Link,
            self.on_press.or(Some(callback(|()| {}))),
        );
        let active = self.active;
        link.base_style(move || {
            let p = tokens();
            let lit = active.get();
            let lit_style = style()
                .background(p.sidebar_accent)
                .color(p.sidebar_accent_foreground);
            style()
                .direction(Direction::Row)
                .align_items(Align::Center)
                .gap(8)
                .height(28)
                .min_width(0)
                .padding_x(8)
                .overflow(Overflow::Hidden)
                .radius(0)
                .font_size(12)
                .color(if lit {
                    p.sidebar_accent_foreground
                } else {
                    p.sidebar_foreground
                })
                .background(if lit {
                    p.sidebar_accent
                } else {
                    Color::TRANSPARENT
                })
                .hover(lit_style.clone())
                .pressed(lit_style)
                .focus_visible(style().ring(2, p.sidebar_ring))
        })
        .child(text(label).style(style().single_line()))
        .into_view()
    }
}

/// A horizontal rule in the sidebar color (`SidebarSeparator`), inset by 8 px.
#[track_caller]
pub fn sidebar_separator() -> Node {
    node()
        .role(Role::Separator)
        .orientation(Orientation::Horizontal)
        .style(
            style()
                .height(1)
                .shrink(0)
                .margin_x(8)
                .background(crate::Color::SidebarBorder),
        )
}

/// A text field for the sidebar (`SidebarInput`): the standard input, 32 px
/// high, on the page color.
#[track_caller]
pub fn sidebar_input(label: impl IntoText, value: impl Into<Binding<String>>) -> TextInput {
    text_input(label, value).style(
        style()
            .width(pct(100))
            .min_width(0)
            .height(32)
            .background(crate::Color::Background),
    )
}

/// A loading placeholder for a menu row (`SidebarMenuSkeleton`): a 16 px
/// bar of a width between 50 % and 90 % that depends on where it was written.
#[track_caller]
pub fn sidebar_menu_skeleton() -> Node {
    let at = std::panic::Location::caller();
    let width = 50.0 + ((at.line() * 7 + at.column()) % 41) as f32;
    row(skeleton().style(style().height(16).grow(1).max_width(pct(width)))).style(
        style()
            .align_items(Align::Center)
            .gap(8)
            .height(32)
            .padding_x(8),
    )
}
