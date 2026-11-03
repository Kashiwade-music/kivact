//! Role: Render desktop and mobile Neutral top navigation and its menus.
use crate::app_shell::{ShellNavMode, shell_nav_mode};
use crate::grid::{GridColumns, grid};
use crate::mobile_nav::use_app_shell_mobile;
use crate::nav_heading::{
    NavHeadingMenuItem, NavMenuLook, nav_heading_menu, nav_heading_menu_item, nav_menu_view,
    popup_surface, supporting,
};
use crate::side_nav::{SideNavItem, logical_row, side_nav_item, side_row_style};
use crate::stack::{Space, rtl};
use crate::tokens;
use kivact::control::{Node, NodeBuilder, column, icon, node, row, scroll, text};
use kivact::interaction::{Delays, use_delayed, use_dialog};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, Justify, pct, style};
use kivact::ui::{ControlSize, DialogKind};
use kivact::view::{
    HandlerResult, IntoText, IntoValue, IntoView, NodeRef, OverlayAlign, Placement, Role, View,
    callback, node_ref, overlay, provide, show, try_context,
};
use kivact::{Binding, Callback, Value, binding, effect_on, signal};
use std::rc::Rc;
use std::time::Duration;

type Slot = Rc<dyn Fn() -> View>;

#[derive(Clone, Copy)]
struct TopNavCx {
    anchor: NodeRef,
    align: OverlayAlign,
}

#[derive(Default)]
struct Slots {
    heading: Option<Slot>,
    start: Option<Slot>,
    center: Option<Slot>,
    end: Option<Slot>,
}

pub struct TopNav {
    root: Node,
    slots: Slots,
}

pub fn top_nav() -> TopNav {
    TopNav {
        root: node().role(Role::Navigation).name(kivact::i18n::message(
            "soft.top_nav.label",
            "Top navigation",
        )),
        slots: Slots::default(),
    }
}

impl TopNav {
    pub fn heading<V: IntoView>(mut self, heading: impl Fn() -> V + 'static) -> Self {
        self.slots.heading = Some(Rc::new(move || heading().into_view()));
        self
    }
    pub fn start<V: IntoView>(mut self, start: impl Fn() -> V + 'static) -> Self {
        self.slots.start = Some(Rc::new(move || start().into_view()));
        self
    }
    pub fn center<V: IntoView>(mut self, center: impl Fn() -> V + 'static) -> Self {
        self.slots.center = Some(Rc::new(move || center().into_view()));
        self
    }
    pub fn end<V: IntoView>(mut self, end: impl Fn() -> V + 'static) -> Self {
        self.slots.end = Some(Rc::new(move || end().into_view()));
        self
    }
}

impl NodeBuilder for TopNav {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for TopNav {
    fn into_view(self) -> View {
        let Self {
            root,
            slots:
                Slots {
                    heading,
                    start,
                    center,
                    end,
                },
        } = self;
        provide((), move || {
            let anchor = node_ref();
            let mode = shell_nav_mode();
            let slot = |slot: Option<Slot>, align| {
                slot.map(|slot| {
                    provide(TopNavCx { anchor, align }, move || {
                        node()
                            .style(move || {
                                let t = tokens();
                                style()
                                    .direction(if mode == ShellNavMode::Drawer {
                                        Direction::Column
                                    } else {
                                        logical_row()
                                    })
                                    .gap(t.spacing.s1)
                                    .align_items(if mode == ShellNavMode::Drawer {
                                        Align::Stretch
                                    } else {
                                        Align::Center
                                    })
                            })
                            .child(slot())
                    })
                })
            };
            let heading = heading.map(|heading| heading());
            let end = slot(end, OverlayAlign::End);
            let start = if mode == ShellNavMode::MobileBar {
                None
            } else {
                slot(start, OverlayAlign::Start)
            };
            let center = if mode == ShellNavMode::MobileBar {
                None
            } else {
                slot(center, OverlayAlign::Center)
            };
            let leading = node()
                .style(move || {
                    let t = tokens();
                    style()
                        .direction(if mode == ShellNavMode::Drawer {
                            Direction::Column
                        } else {
                            logical_row()
                        })
                        .align_items(if mode == ShellNavMode::Drawer {
                            Align::Stretch
                        } else {
                            Align::Center
                        })
                        .gap(if mode == ShellNavMode::Drawer {
                            t.spacing.s1
                        } else {
                            t.spacing.s4
                        })
                })
                .child((heading, start));
            root.node_ref(anchor)
                .base_style(move || {
                    let t = tokens();
                    let base = style().min_width(0).gap(t.spacing.s1);
                    if mode == ShellNavMode::Drawer {
                        base.direction(Direction::Column).width(pct(100))
                    } else {
                        let base = base
                            .direction(logical_row())
                            .align_items(Align::Center)
                            .justify_content(Justify::SpaceBetween);
                        if mode == ShellNavMode::Desktop {
                            base.padding(t.spacing.s2)
                        } else {
                            base
                        }
                    }
                })
                .child((leading, center, end))
        })
    }
}

pub struct TopNavItem {
    root: Node,
    label: Value<String>,
    content: Option<View>,
    icon: Option<View>,
    icon_only: bool,
    selected: Value<bool>,
    disabled: Value<bool>,
    action: Option<Callback<()>>,
    size: ControlSize,
}

pub fn top_nav_item(label: impl IntoText) -> TopNavItem {
    TopNavItem {
        root: node(),
        label: label.into_text_value(),
        content: None,
        icon: None,
        icon_only: false,
        selected: false.into_value(),
        disabled: false.into_value(),
        action: None,
        size: ControlSize::Medium,
    }
}

impl TopNavItem {
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    pub fn icon_only(mut self, only: bool) -> Self {
        self.icon_only = only;
        self
    }
    pub fn content<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.content = Some(provide((), content));
        self
    }
    pub fn selected(mut self, selected: impl IntoValue<bool>) -> Self {
        self.selected = selected.into_value();
        self
    }
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    pub fn on_press<R: HandlerResult>(mut self, action: impl Fn() -> R + 'static) -> Self {
        self.action = Some(callback(move |_| action()));
        self
    }
}

impl NodeBuilder for TopNavItem {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for TopNavItem {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            content,
            icon,
            icon_only,
            selected,
            disabled,
            action,
            size,
        } = self;
        provide((), move || {
            let mode = shell_nav_mode();
            let mobile = use_app_shell_mobile();
            let drawer = mode == ShellNavMode::Drawer;
            let label_view =
                (!icon_only || drawer).then(|| content.unwrap_or_else(|| text(label).into_view()));
            let root = root
                .name(label)
                .selected(selected)
                .current_item(move || selected.get().then_some(kivact::view::CurrentItem::Page))
                .disabled(disabled)
                .base_style(move || {
                    if drawer {
                        side_row_style(size, 0, false, selected.get(), disabled.get())
                    } else {
                        top_item_style(icon_only, selected.get(), disabled.get(), false)
                    }
                })
                .child((icon, label_view));
            if let Some(action) = action {
                root.role(Role::Link)
                    .focusable(move || !disabled.get())
                    .on_press(move || {
                        if !disabled.get() {
                            action.call(());
                            if drawer {
                                mobile.close();
                            }
                        }
                    })
            } else {
                root
            }
        })
    }
}

fn top_item_style(
    icon_only: bool,
    selected: bool,
    disabled: bool,
    open: bool,
) -> kivact::style::Style {
    let t = tokens();
    let role = t.typography.label;
    let base = style()
        .direction(logical_row())
        .align_items(Align::Center)
        .gap(t.spacing.s2)
        .padding_x(if icon_only {
            t.spacing.s2
        } else {
            t.spacing.s3
        })
        .padding_y(t.spacing.s1_5)
        .radius(t.radius.element)
        .font_size(role.size_px)
        .line_height(role.line_height_px / role.size_px)
        .weight(if selected {
            t.typography.weight_semibold
        } else {
            role.weight
        })
        .color(if disabled {
            t.colors.text_disabled
        } else if selected || open {
            t.colors.text_primary
        } else {
            t.colors.text_secondary
        })
        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent));
    if selected {
        base.background(t.colors.neutral)
    } else if open {
        base.background(t.colors.overlay_hover)
    } else if disabled {
        base
    } else {
        base.hover(style().background(t.colors.overlay_hover))
            .pressed(style().background(t.colors.overlay_pressed))
    }
}

pub type TopNavMenuItem = NavHeadingMenuItem;

pub fn top_nav_menu_item(title: impl IntoText) -> TopNavMenuItem {
    nav_heading_menu_item(title)
}

pub struct TopNavMenu {
    root: Node,
    label: Value<String>,
    items: Rc<[TopNavMenuItem]>,
    delays: Delays,
}

pub fn top_nav_menu(
    label: impl IntoText,
    items: impl IntoIterator<Item = TopNavMenuItem>,
) -> TopNavMenu {
    TopNavMenu {
        root: node(),
        label: label.into_text_value(),
        items: items.into_iter().collect::<Vec<_>>().into(),
        delays: Delays {
            open: Duration::from_millis(150),
            close: Duration::from_millis(200),
        },
    }
}

impl TopNavMenu {
    pub fn delay_ms(mut self, delay: u64) -> Self {
        self.delays.open = Duration::from_millis(delay);
        self
    }
    pub fn hide_delay_ms(mut self, delay: u64) -> Self {
        self.delays.close = Duration::from_millis(delay);
        self
    }
}

impl NodeBuilder for TopNavMenu {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for TopNavMenu {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            items,
            delays,
        } = self;
        provide((), move || {
            if shell_nav_mode() == ShellNavMode::Drawer {
                let item = side_nav_item(label)
                    .children(move || items.iter().map(drawer_menu_item).collect::<Vec<_>>());
                return root.child(item);
            }
            let align = try_context::<TopNavCx>().map_or(OverlayAlign::Start, |cx| cx.align);
            let menu = nav_heading_menu(items.iter().cloned());
            root.child(nav_menu_view(
                menu,
                label,
                label,
                NavMenuLook::Top,
                delays,
                align,
                move |open| menu_trigger(label, open),
            ))
        })
    }
}

fn menu_trigger(label: Value<String>, open: Value<bool>) -> Node {
    row((
        text(label),
        icon("chevron-down").style(move || {
            let t = tokens();
            style()
                .size(t.sizes.icon_sm, t.sizes.icon_sm)
                .rotate(if open.get() { 180 } else { 0 })
        }),
    ))
    .name(label)
    .style(move || top_item_style(false, false, false, open.get()))
}

fn drawer_menu_item(item: &TopNavMenuItem) -> SideNavItem {
    let mut row = side_nav_item(item.label).disabled(item.disabled);
    if let Some(icon) = &item.icon {
        row = row.icon(icon());
    }
    if let Some(description) = item.description {
        row = row.end_content(move || text(description).style(supporting));
    }
    if let Some(action) = item.action {
        row = row.on_press(move || action.call(()));
    }
    row
}

pub struct TopNavMegaMenu {
    root: Node,
    label: Value<String>,
    items: Option<Slot>,
    featured: Option<Slot>,
    delays: Delays,
    on_change: Option<Callback<bool>>,
}

pub fn top_nav_mega_menu(label: impl IntoText) -> TopNavMegaMenu {
    TopNavMegaMenu {
        root: node(),
        label: label.into_text_value(),
        items: None,
        featured: None,
        delays: Delays {
            open: Duration::from_millis(150),
            close: Duration::from_millis(250),
        },
        on_change: None,
    }
}

impl TopNavMegaMenu {
    pub fn items<V: IntoView>(mut self, items: impl Fn() -> V + 'static) -> Self {
        self.items = Some(Rc::new(move || items().into_view()));
        self
    }
    pub fn featured<V: IntoView>(mut self, featured: impl Fn() -> V + 'static) -> Self {
        self.featured = Some(Rc::new(move || featured().into_view()));
        self
    }
    pub fn delay_ms(mut self, delay: u64) -> Self {
        self.delays.open = Duration::from_millis(delay);
        self
    }
    pub fn hide_delay_ms(mut self, delay: u64) -> Self {
        self.delays.close = Duration::from_millis(delay);
        self
    }
    pub fn on_open_change<R: HandlerResult>(
        mut self,
        change: impl Fn(bool) -> R + 'static,
    ) -> Self {
        self.on_change = Some(callback(change));
        self
    }
}

impl NodeBuilder for TopNavMegaMenu {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

#[derive(Clone, Copy)]
struct MegaClose(Binding<bool>);

impl IntoView for TopNavMegaMenu {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            items,
            featured,
            delays,
            on_change,
        } = self;
        provide((), move || {
            if shell_nav_mode() == ShellNavMode::Drawer {
                let disclosure = side_nav_item(label).children(move || {
                    column((
                        items.as_ref().map(|items| items()),
                        featured.as_ref().map(|featured| featured()),
                    ))
                });
                return root.child(disclosure);
            }
            let trigger_ref = node_ref();
            let surface_ref = node_ref();
            let trigger_hovered = trigger_ref.hovered();
            let surface_hovered = surface_ref.hovered();
            let delayed = use_delayed(
                move || {
                    !kivact::platform::capabilities().touch
                        && (trigger_hovered.get() || surface_hovered.get())
                },
                delays,
            );
            let pinned = signal(false);
            let open = binding(
                move || pinned.get() || delayed.shown().get(),
                move |open| {
                    pinned.set(open);
                    if !open {
                        delayed.dismiss();
                    }
                },
            );
            if let Some(change) = on_change {
                effect_on(open, move |open| {
                    change.call(*open);
                });
            }
            let behavior = use_dialog(open, DialogKind::Popover).anchor(trigger_ref);
            let trigger = menu_trigger(label, open.into_value())
                .node_ref(trigger_ref)
                .role(Role::Button)
                .focusable(true)
                .expanded(open)
                .controls(surface_ref)
                .on_press(move || open.set(!open.get()));
            let cx = try_context::<TopNavCx>();
            let align = cx.map_or(OverlayAlign::Start, |cx| cx.align);
            let anchor = cx.map_or(trigger_ref, |cx| cx.anchor);
            let surface = show(behavior.shown(), move || {
                let grid = items.as_ref().map(|items| {
                    let items = items.clone();
                    grid(provide(MegaClose(open), move || items()))
                        .columns(GridColumns::Fixed(2))
                        .gap(Space::S2)
                        .style(style().grow(2).shrink(1).basis(300).min_width(0))
                });
                let featured = featured.as_ref().map(|featured| {
                    let featured = featured.clone();
                    node()
                        .child(provide(MegaClose(open), move || featured()))
                        .style(style().grow(1).shrink(1).basis(200).min_width(0))
                });
                let body = row((grid, featured)).style(|| {
                    let t = tokens();
                    style()
                        .direction(logical_row())
                        .wrap(true)
                        .gap(t.spacing.s6)
                        .padding(t.spacing.s3)
                        .width(960)
                        .max_width((kivact::platform::window_size().width - t.spacing.s4).max(0.0))
                });
                let panel = scroll(body).node_ref(surface_ref).style(|| {
                    let t = tokens();
                    popup_surface()
                        .max_height(pct(100) - t.spacing.s3)
                        .clip(true)
                        .border_top(t.sizes.border_width, t.colors.border)
                });
                overlay(anchor, Placement::Below, panel)
                    .offset(4)
                    .align(align)
                    .part(behavior.content())
                    .name(label)
            });
            root.child((trigger, surface))
        })
    }
}

pub struct TopNavMegaMenuItem {
    root: Node,
    title: Value<String>,
    description: Option<Value<String>>,
    icon: Option<View>,
    action: Option<Callback<()>>,
}

pub fn top_nav_mega_menu_item(title: impl IntoText) -> TopNavMegaMenuItem {
    TopNavMegaMenuItem {
        root: node(),
        title: title.into_text_value(),
        description: None,
        icon: None,
        action: None,
    }
}

impl TopNavMegaMenuItem {
    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(description.into_text_value());
        self
    }
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    pub fn on_press<R: HandlerResult>(mut self, action: impl Fn() -> R + 'static) -> Self {
        self.action = Some(callback(move |_| action()));
        self
    }
}

impl NodeBuilder for TopNavMegaMenuItem {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for TopNavMegaMenuItem {
    fn into_view(self) -> View {
        let Self {
            root,
            title,
            description,
            icon,
            action,
        } = self;
        provide((), move || {
            let drawer = shell_nav_mode() == ShellNavMode::Drawer;
            let mobile = use_app_shell_mobile();
            let popup = try_context::<MegaClose>();
            let glyph = icon.map(|glyph| {
                node()
                    .style(move || {
                        let t = tokens();
                        let size = if drawer {
                            t.sizes.control_md
                        } else {
                            t.spacing.s10
                        };
                        style()
                            .size(size, size)
                            .shrink(0)
                            .radius(t.radius.element)
                            .background(t.colors.neutral)
                            .color(t.colors.icon_secondary)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                    })
                    .child(glyph)
            });
            let labels = column((
                text(title).style(title_style),
                description.map(|value| text(value).style(supporting)),
            ))
            .style(move || {
                style().min_width(0).gap(if drawer {
                    tokens().spacing.s0_5
                } else {
                    tokens().spacing.s1
                })
            });
            let root = root
                .name(title)
                .base_style(move || {
                    let t = tokens();
                    let base = style()
                        .direction(logical_row())
                        .align_items(Align::Start)
                        .gap(if drawer { t.spacing.s2 } else { t.spacing.s3 })
                        .padding(t.spacing.s3)
                        .radius(t.radius.element)
                        .width(pct(100))
                        .hover(style().background(t.colors.overlay_hover))
                        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent));
                    if drawer {
                        if rtl() {
                            base.padding_right(t.spacing.s6)
                        } else {
                            base.padding_left(t.spacing.s6)
                        }
                    } else {
                        base
                    }
                })
                .child((glyph, labels));
            if let Some(action) = action {
                root.role(Role::Link).focusable(true).on_press(move || {
                    action.call(());
                    if let Some(popup) = popup {
                        popup.0.set(false);
                    }
                    if drawer {
                        mobile.close();
                    }
                })
            } else {
                root
            }
        })
    }
}

fn title_style() -> kivact::style::Style {
    let t = tokens();
    let role = t.typography.body;
    style()
        .font_size(role.size_px)
        .line_height(role.line_height_px / role.size_px)
        .weight(t.typography.weight_semibold)
        .color(t.colors.text_primary)
}

pub struct TopNavMegaMenuFeaturedCard {
    root: Node,
    title: Value<String>,
    description: Option<Value<String>>,
    image: Option<View>,
    content: Option<View>,
    link: Option<(Value<String>, Callback<()>)>,
}

pub fn top_nav_featured_card(title: impl IntoText) -> TopNavMegaMenuFeaturedCard {
    TopNavMegaMenuFeaturedCard {
        root: node(),
        title: title.into_text_value(),
        description: None,
        image: None,
        content: None,
        link: None,
    }
}

impl TopNavMegaMenuFeaturedCard {
    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(description.into_text_value());
        self
    }
    pub fn image(mut self, image: impl IntoView) -> Self {
        self.image = Some(image.into_view());
        self
    }
    pub fn content(mut self, content: impl IntoView) -> Self {
        self.content = Some(content.into_view());
        self
    }
    pub fn link<R: HandlerResult>(
        mut self,
        label: impl IntoText,
        action: impl Fn() -> R + 'static,
    ) -> Self {
        self.link = Some((label.into_text_value(), callback(move |_| action())));
        self
    }
}

impl NodeBuilder for TopNavMegaMenuFeaturedCard {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for TopNavMegaMenuFeaturedCard {
    fn into_view(self) -> View {
        let Self {
            root,
            title,
            description,
            image,
            content,
            link,
        } = self;
        provide((), move || {
            let drawer = shell_nav_mode() == ShellNavMode::Drawer;
            let mobile = use_app_shell_mobile();
            let popup = try_context::<MegaClose>();
            let image = image.map(|image| {
                node()
                    .style(style().width(pct(100)).height(140).clip(true))
                    .child(image)
            });
            let link = link.map(|(label, action)| {
                row((text(label), text(if rtl() { "←" } else { "→" })))
                    .name(label)
                    .role(Role::Link)
                    .focusable(true)
                    .on_press(move || {
                        action.call(());
                        if let Some(popup) = popup {
                            popup.0.set(false);
                        }
                        if drawer {
                            mobile.close();
                        }
                    })
                    .style(|| {
                        let t = tokens();
                        supporting()
                            .direction(logical_row())
                            .gap(t.spacing.s1)
                            .color(t.colors.text_accent)
                            .weight(t.typography.weight_semibold)
                            .focus_visible(crate::recipe_style::ring(&t, t.colors.accent))
                    })
            });
            let body = column((
                text(title).style(title_style),
                description.map(|description| text(description).style(supporting)),
                content,
                link,
            ))
            .style(|| {
                let t = tokens();
                style().padding(t.spacing.s4).gap(t.spacing.s2)
            });
            root.base_style(move || {
                let t = tokens();
                let base = style()
                    .direction(Direction::Column)
                    .clip(true)
                    .radius(t.radius.container)
                    .background(t.colors.bg_muted);
                if drawer {
                    let base = base.margin_top(t.spacing.s2);
                    if rtl() {
                        base.margin_right(t.spacing.s6)
                    } else {
                        base.margin_left(t.spacing.s6)
                    }
                } else {
                    base
                }
            })
            .child((image, body))
        })
    }
}
