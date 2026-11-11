//! Role: Render desktop and mobile Neutral top navigation and its menus.
use crate::app_shell::{ShellNavMode, shell_nav_mode};
use crate::grid::{GridColumns, layout_grid};
use crate::mobile_nav::use_app_shell_mobile;
use crate::nav_heading::{
    NavHeadingMenuItem, NavMenuLook, nav_heading_menu, nav_heading_menu_item, nav_menu_view,
    popup_surface, supporting,
};
use crate::side_navigation::items::{logical_row, side_row_style};
use crate::side_navigation::{SideNavItem, side_nav_item};
use crate::stack::{Space, rtl};

use kivact::interaction::Delays;
use kivact::interaction::use_dialog;
use kivact::view::Element;
use kivact::view::IntoViewFn;
use kivact::view::Node;
use kivact::view::ViewFn;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::row;
use kivact::view::scroll;
use kivact::view::text;

use kivact::interaction::DialogMode;
use kivact::state::Binding;
use kivact::state::Callback;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::callback;
use kivact::state::effect_on;
use kivact::state::provide;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ControlSize;
use kivact::view::HandlerResult;
use kivact::view::IntoView;
use kivact::view::NodeRef;
use kivact::view::OverlayAlign;
use kivact::view::Placement;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::node_ref;
use kivact::view::overlay;
use kivact::view::show;
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone, Copy)]
struct TopNavCx {
    anchor: NodeRef,
    align: OverlayAlign,
}

#[derive(Default)]
struct Slots {
    // xtask-allow(state-fields): option: caller content from TopNav::heading, mounted once; absent leaves the heading area empty.
    heading: Option<View>,
    // xtask-allow(state-fields): option: caller content from TopNav::start, mounted once and dropped in the mobile bar; absent leaves the start area empty.
    start: Option<View>,
    // xtask-allow(state-fields): option: caller content from TopNav::center, mounted once and dropped in the mobile bar; absent leaves the center area empty.
    center: Option<View>,
    // xtask-allow(state-fields): option: caller content from TopNav::end, mounted once; absent leaves the end area empty.
    end: Option<View>,
}

/// A top navigation bar with retained heading, start, center and end slots.
pub struct TopNav {
    root: Node,
    slots: Slots,
}

/// Create the top navigation bar; supply its content through the retained slots.
pub fn top_nav() -> TopNav {
    TopNav {
        root: node()
            .with_node(move |node| node.role(Role::Navigation))
            .name(kivact::i18n::message(
                "soft.top_nav.label",
                "Top navigation",
            )),
        slots: Slots::default(),
    }
}

impl TopNav {
    /// Mount the brand or heading once within this navigation owner.
    pub fn heading(mut self, heading: impl IntoView) -> Self {
        self.slots.heading = Some(heading.into_view());
        self
    }
    /// Mount navigation content at the logical start.
    pub fn start(mut self, start: impl IntoView) -> Self {
        self.slots.start = Some(start.into_view());
        self
    }
    /// Mount navigation content in the central region.
    pub fn center(mut self, center: impl IntoView) -> Self {
        self.slots.center = Some(center.into_view());
        self
    }
    /// Mount navigation content at the logical end.
    pub fn end(mut self, end: impl IntoView) -> Self {
        self.slots.end = Some(end.into_view());
        self
    }
}

impl Element for TopNav {
    #[doc(hidden)]
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
        crate::navigation_session::with_navigation_group(move || {
            let anchor = node_ref();
            let mode = shell_nav_mode();
            let slot = move |slot, align| top_slot(slot, align, anchor, mode);
            let heading = heading;
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
                    let t = kivact::style::use_tokens::<crate::Tokens>();
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
                .with_node(move |node| {
                    node.base_style(move || {
                        let t = kivact::style::use_tokens::<crate::Tokens>();
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
                })
                .child((leading, center, end))
        })
    }
}

/// A top navigation action with optional icon and custom retained label content.
pub struct TopNavItem {
    root: Node,
    label: Value<String>,
    content: Option<View>,
    icon: Option<View>,
    icon_only: Value<bool>,
    selected: Value<bool>,
    action: Option<Callback<()>>,
    size: Value<ControlSize>,
}

/// Create a top navigation item with the supplied accessible and visible label.
pub fn top_nav_item(label: impl IntoText) -> TopNavItem {
    TopNavItem {
        root: node(),
        label: label.into_text_value(),
        content: None,
        icon: None,
        icon_only: (false).into_value(),
        selected: false.into_value(),
        action: None,
        size: (ControlSize::Medium).into_value(),
    }
}

impl TopNavItem {
    /// Mount the leading item icon once.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    /// Hide visible label content while keeping the item's accessible name.
    pub fn icon_only(mut self, only: impl IntoValue<bool>) -> Self {
        self.icon_only = only.into_value();
        self
    }
    /// Mount replacement visible label content once.
    pub fn content(mut self, content: impl IntoView) -> Self {
        self.content = Some(kivact::view::scope(move || content.into_view()));
        self
    }
    /// Set whether this item is the current navigation destination.
    pub fn selected(mut self, selected: impl IntoValue<bool>) -> Self {
        self.selected = selected.into_value();
        self
    }
    /// Set the navigation item's control size.
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
    /// Handle activation of the navigation item.
    pub fn on_press(mut self, action: impl Into<Callback<()>>) -> Self {
        let action: Callback<()> = action.into();
        self.action = Some(callback(move |_| action.call(())));
        self
    }
}

impl Element for TopNavItem {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for TopNavItem {
    fn into_view(mut self) -> View {
        let disabled = self
            .root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        let Self {
            root,
            label,
            content,
            icon,
            icon_only,
            selected,
            action,
            size,
        } = self;
        kivact::view::scope(move || {
            let mode = shell_nav_mode();
            let mobile = use_app_shell_mobile();
            let drawer = mode == ShellNavMode::Drawer;
            let label_view = (!icon_only.get() || drawer)
                .then(|| content.unwrap_or_else(|| text(label).into_view()));
            let root = root
                .name(label)
                .with_node(move |node| node.selected(selected))
                .with_node(move |node| {
                    node.current_item(move || {
                        selected.get().then_some(kivact::view::CurrentItem::Page)
                    })
                })
                .disabled(disabled)
                .with_node(move |node| {
                    node.base_style(move || {
                        if drawer {
                            side_row_style(size.get(), 0, false, selected.get(), disabled.get())
                        } else {
                            top_item_style(icon_only.get(), selected.get(), disabled.get(), false)
                        }
                    })
                })
                .child((icon, label_view));
            if let Some(action) = action {
                root.with_node(move |node| node.role(Role::Link))
                    .focusable(move || !disabled.get())
                    .on_press(move || {
                        if !disabled.get() {
                            action.call(());
                            if drawer {
                                mobile.hide();
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
    let t = kivact::style::use_tokens::<crate::Tokens>();
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

/// Create an action entry for a top navigation menu.
pub fn top_nav_menu_item(title: impl IntoText) -> NavHeadingMenuItem {
    nav_heading_menu_item(title)
}

/// A top navigation menu with configurable hover opening and closing delays.
pub struct TopNavMenu {
    root: Node,
    label: Value<String>,
    items: Rc<[NavHeadingMenuItem]>,
    delays: Delays,
}

/// Create a top navigation menu from the supplied heading-menu entries.
pub fn top_nav_menu(
    label: impl IntoText,
    items: impl IntoIterator<Item = NavHeadingMenuItem>,
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
    /// Set the pointer-hover opening delay.
    pub fn open_delay(mut self, delay: Duration) -> Self {
        self.delays.open = delay;
        self
    }
    /// Set the pointer-leave closing delay.
    pub fn close_delay(mut self, delay: Duration) -> Self {
        self.delays.close = delay;
        self
    }
}

impl Element for TopNavMenu {
    #[doc(hidden)]
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
        kivact::view::scope(move || {
            if shell_nav_mode() == ShellNavMode::Drawer {
                let item = side_nav_item(label)
                    .children(move || items.iter().map(drawer_menu_item).collect::<Vec<_>>());
                return root.child(item);
            }
            let align = try_use_context::<TopNavCx>()
                .map(|value| value.get())
                .map_or(OverlayAlign::Start, |cx| cx.align);
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
            let t = kivact::style::use_tokens::<crate::Tokens>();
            style()
                .size(t.sizes.icon_sm, t.sizes.icon_sm)
                .rotate(if open.get() { 180 } else { 0 })
        }),
    ))
    .name(label)
    .style(move || top_item_style(false, false, false, open.get()))
}

fn drawer_menu_item(item: &NavHeadingMenuItem) -> SideNavItem {
    let mut row = side_nav_item(item.label).disabled(item.disabled);
    if let Some(icon) = &item.icon {
        row = row.icon(icon.call(()));
    }
    if let Some(description) = item.description {
        row = row.end_content(text(description).style(supporting));
    }
    if let Some(action) = item.action {
        row = row.on_press(move || action.call(()));
    }
    row
}

/// A wide top navigation flyout with recreated item and featured-content columns.
pub struct TopNavMegaMenu {
    root: Node,
    label: Value<String>,
    items: Option<ViewFn>,
    featured: Option<ViewFn>,
    delays: Delays,
    on_change: Option<Callback<bool>>,
}

/// Create a named mega-menu flyout with independently configured content columns.
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
    /// Build the main item column each time the mega menu opens.
    pub fn items<V: IntoView>(mut self, items: impl IntoViewFn<V>) -> Self {
        self.items = Some(items.into_view_fn());
        self
    }
    /// Build the featured column each time the mega menu opens.
    pub fn featured<V: IntoView>(mut self, featured: impl IntoViewFn<V>) -> Self {
        self.featured = Some(featured.into_view_fn());
        self
    }
    /// Set the pointer-hover opening delay.
    pub fn open_delay(mut self, delay: Duration) -> Self {
        self.delays.open = delay;
        self
    }
    /// Set the pointer-leave closing delay.
    pub fn close_delay(mut self, delay: Duration) -> Self {
        self.delays.close = delay;
        self
    }
    /// Observe changes to the flyout's open state.
    pub fn on_open_change<R: HandlerResult>(
        mut self,
        change: impl Fn(bool) -> R + 'static,
    ) -> Self {
        self.on_change = Some(callback(change));
        self
    }
}

impl Element for TopNavMegaMenu {
    #[doc(hidden)]
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
        kivact::view::scope(move || {
            if shell_nav_mode() == ShellNavMode::Drawer {
                let disclosure = side_nav_item(label).children(move || {
                    column((
                        items.as_ref().map(|items| items.call(())),
                        featured.as_ref().map(|featured| featured.call(())),
                    ))
                });
                return root.child(disclosure);
            }
            let trigger_ref = node_ref();
            let surface_ref = node_ref();
            let nav_panel = crate::navigation_session::NavPanel::new(true, delays);
            let open = nav_panel.open();
            if let Some(change) = on_change {
                effect_on(open, move |open| {
                    change.call(*open);
                });
            }
            let behavior = use_dialog(open, DialogMode::Popover).anchor(trigger_ref);
            let trigger = nav_panel.trigger(
                menu_trigger(label, open.into_value())
                    .node_ref(trigger_ref)
                    .with_node(move |node| node.role(Role::Button))
                    .focusable(true)
                    .with_node(move |node| node.with_node(move |node| node.expanded(open)))
                    .with_node(move |node| node.controls(surface_ref))
                    .on_press(move || nav_panel.toggle()),
            );
            let cx = try_use_context::<TopNavCx>().map(|value| value.get());
            let align = cx.map_or(OverlayAlign::Start, |cx| cx.align);
            let anchor = cx.map_or(trigger_ref, |cx| cx.anchor);
            let surface = show(behavior.is_open(), move || {
                let body = mega_columns(items.as_ref(), featured.as_ref(), open);
                let panel = nav_panel
                    .surface(scroll(body))
                    .node_ref(surface_ref)
                    .style(|| {
                        let t = kivact::style::use_tokens::<crate::Tokens>();
                        popup_surface()
                            .max_height(pct(100) - t.spacing.s3)
                            .clip(true)
                            .border_top(t.sizes.border_width, t.colors.border)
                    });
                overlay(anchor, Placement::Below, panel)
                    .offset(4)
                    .align(align)
                    .with_node(move |node| node.part(behavior.content()))
                    .name(label)
            });
            root.child((trigger, surface))
        })
    }
}

/// An action item with a title, supporting text and an optional icon in a mega menu.
pub struct TopNavMegaMenuItem {
    root: Node,
    title: Value<String>,
    description: Option<Value<String>>,
    icon: Option<View>,
    action: Option<Callback<()>>,
}

/// Create a mega-menu action item with a title.
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
    /// Set supporting text below the item title.
    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(description.into_text_value());
        self
    }
    /// Mount the item's leading icon once.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    /// Handle activation of the mega-menu item.
    pub fn on_press(mut self, action: impl Into<Callback<()>>) -> Self {
        let action: Callback<()> = action.into();
        self.action = Some(callback(move |_| action.call(())));
        self
    }
}

impl Element for TopNavMegaMenuItem {
    #[doc(hidden)]
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
        kivact::view::scope(move || {
            let drawer = shell_nav_mode() == ShellNavMode::Drawer;
            let mobile = use_app_shell_mobile();
            let popup = try_use_context::<MegaClose>().map(|value| value.get());
            let glyph = icon.map(move |glyph| mega_glyph(glyph, drawer));
            let labels = column((
                text(title).style(title_style),
                description.map(|value| text(value).style(supporting)),
            ))
            .style(move || {
                style().min_width(0).gap(if drawer {
                    kivact::style::use_tokens::<crate::Tokens>().spacing.s0_5
                } else {
                    kivact::style::use_tokens::<crate::Tokens>().spacing.s1
                })
            });
            let root = root
                .name(title)
                .with_node(move |node| {
                    node.base_style(move || {
                        let t = kivact::style::use_tokens::<crate::Tokens>();
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
                })
                .child((glyph, labels));
            if let Some(action) = action {
                root.with_node(move |node| node.role(Role::Link))
                    .focusable(true)
                    .on_press(move || {
                        action.call(());
                        if let Some(popup) = popup {
                            popup.0.set(false);
                        }
                        if drawer {
                            mobile.hide();
                        }
                    })
            } else {
                root
            }
        })
    }
}

fn title_style() -> kivact::style::Style {
    let t = kivact::style::use_tokens::<crate::Tokens>();
    let role = t.typography.body;
    style()
        .font_size(role.size_px)
        .line_height(role.line_height_px / role.size_px)
        .weight(t.typography.weight_semibold)
        .color(t.colors.text_primary)
}

/// A featured mega-menu card with retained image, body and link slots.
pub struct TopNavMegaMenuFeaturedCard {
    root: Node,
    title: Value<String>,
    description: Option<Value<String>>,
    image: Option<View>,
    content: Option<View>,
    link: Option<(Value<String>, Callback<()>)>,
}

/// Create a featured card with the supplied title.
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
    /// Set supporting text below the featured title.
    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(description.into_text_value());
        self
    }
    /// Mount featured image content once.
    pub fn image(mut self, image: impl IntoView) -> Self {
        self.image = Some(image.into_view());
        self
    }
    /// Mount additional featured-card body content once.
    pub fn content(mut self, content: impl IntoView) -> Self {
        self.content = Some(content.into_view());
        self
    }
    /// Set the featured-card link label and activation action.
    pub fn link(mut self, label: impl IntoText, action: impl Into<Callback<()>>) -> Self {
        let action: Callback<()> = action.into();
        self.link = Some((label.into_text_value(), callback(move |_| action.call(()))));
        self
    }
}

impl Element for TopNavMegaMenuFeaturedCard {
    #[doc(hidden)]
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
        kivact::view::scope(move || {
            let drawer = shell_nav_mode() == ShellNavMode::Drawer;
            let mobile = use_app_shell_mobile();
            let popup = try_use_context::<MegaClose>().map(|value| value.get());
            let image = image.map(|image| {
                node()
                    .style(style().width(pct(100)).height(140).clip(true))
                    .child(image)
            });
            let link = link.map(|(label, action)| {
                row((text(label), text(if rtl() { "←" } else { "→" })))
                    .name(label)
                    .with_node(move |node| node.role(Role::Link))
                    .focusable(true)
                    .on_press(move || {
                        action.call(());
                        if let Some(popup) = popup {
                            popup.0.set(false);
                        }
                        if drawer {
                            mobile.hide();
                        }
                    })
                    .style(|| {
                        let t = kivact::style::use_tokens::<crate::Tokens>();
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
                let t = kivact::style::use_tokens::<crate::Tokens>();
                style().padding(t.spacing.s4).gap(t.spacing.s2)
            });
            root.with_node(move |node| {
                node.base_style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
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
            })
            .child((image, body))
        })
    }
}

fn top_slot(
    slot: Option<View>,
    align: OverlayAlign,
    anchor: NodeRef,
    mode: ShellNavMode,
) -> Option<View> {
    slot.map(|slot| {
        provide(TopNavCx { anchor, align }, move || {
            node()
                .style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
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
                .child(slot)
        })
    })
}

fn mega_columns(items: Option<&ViewFn>, featured: Option<&ViewFn>, open: Binding<bool>) -> Node {
    let grid = items.map(|items| {
        let items = items.clone();
        layout_grid(provide(MegaClose(open), move || items.call(())))
            .columns(GridColumns::Fixed(2))
            .gap(Space::S2)
            .style(style().grow(2).shrink(1).basis(300).min_width(0))
    });
    let featured = featured.map(|featured| {
        let featured = featured.clone();
        node()
            .child(provide(MegaClose(open), move || featured.call(())))
            .style(style().grow(1).shrink(1).basis(200).min_width(0))
    });
    row((grid, featured)).style(|| {
        let t = kivact::style::use_tokens::<crate::Tokens>();
        style()
            .direction(logical_row())
            .wrap(true)
            .gap(t.spacing.s6)
            .padding(t.spacing.s3)
            .width(960)
            .max_width((kivact::platform::window_size().get().width - t.spacing.s4).max(0.0))
    })
}

fn mega_glyph(glyph: View, drawer: bool) -> Node {
    node()
        .style(move || {
            let t = kivact::style::use_tokens::<crate::Tokens>();
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
}
