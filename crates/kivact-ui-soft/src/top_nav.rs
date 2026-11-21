//! Role: Render desktop and mobile Neutral top navigation and its menus.
use crate::app_shell::{ShellNavMode, shell_nav_mode};
use crate::color as c;
use crate::grid::{GridColumns, layout_grid};
use crate::metrics as m;
use crate::mobile_nav::use_app_shell_mobile;
use crate::nav_heading::{
    NavHeadingMenuItem, NavMenuLook, nav_heading_menu, nav_heading_menu_item, nav_menu_view,
    popup_surface, supporting,
};
use crate::radius::{RADIUS_CONTAINER, RADIUS_ELEMENT};
use crate::recipe_parts::ring;
use crate::side_navigation::items::{logical_row, side_row_style};
use crate::side_navigation::{SideNavItem, side_nav_item};
use crate::space::{self as s, Space};
use crate::stack::rtl;
use crate::type_scale as t;

use kivact::component;
use kivact::custom::Caller;
use kivact::interaction::Delays;
use kivact::interaction::use_popup;
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

use kivact::state::Callback;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::state::callback;
use kivact::state::effect_on;
use kivact::state::provide;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::FlexDirection;
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

/// A top navigation bar with heading, start, center and end slots.
///
/// The slots are mounted in one navigation group, so their menus share one owner. The bar
/// carries the caller's modifiers and the navigation role.
#[component]
pub fn top_nav(
    heading: Option<View>,
    start: Option<View>,
    center: Option<View>,
    end: Option<View>,
) -> impl Element {
    let anchor = node_ref();
    let mode = shell_nav_mode();
    let slots = crate::navigation_session::with_navigation_group(move || {
        let slot = move |slot, align| top_slot(slot, align, anchor, mode);
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
                style()
                    .flex_direction(if mode == ShellNavMode::Drawer {
                        FlexDirection::Column
                    } else {
                        logical_row()
                    })
                    .align_items(if mode == ShellNavMode::Drawer {
                        Align::Stretch
                    } else {
                        Align::Center
                    })
                    .gap(if mode == ShellNavMode::Drawer {
                        s::SPACE_S1
                    } else {
                        s::SPACE_S4
                    })
            })
            .child((heading, start));
        (leading, center, end)
    });
    node()
        .with_node(move |node| node.role(Role::Navigation))
        .name(kivact::i18n::message(
            "soft.top_nav.label",
            "Top navigation",
        ))
        .node_ref(anchor)
        .with_node(move |node| {
            node.base_style(move || {
                let base = style().min_width(0).gap(s::SPACE_S1);
                if mode == ShellNavMode::Drawer {
                    base.flex_direction(FlexDirection::Column).width(pct(100))
                } else {
                    let base = base
                        .flex_direction(logical_row())
                        .align_items(Align::Center)
                        .justify_content(Justify::SpaceBetween);
                    if mode == ShellNavMode::Desktop {
                        base.padding(s::SPACE_S2)
                    } else {
                        base
                    }
                }
            })
        })
        .child(slots)
}

/// A top navigation action with an optional icon and custom visible content.
///
/// The label is the accessible name and, unless `content` replaces it, the visible text.
/// `icon_only` hides the visible label and keeps the name. Activation runs the caller's
/// `on_press` (set with `.on_press(..)`), and only a press makes the item a link.
#[component]
pub fn top_nav_item(
    label: Value<String>,
    icon: Option<View>,
    content: Option<View>,
    #[prop(default = false)] icon_only: Value<bool>,
    #[prop(default = false)] selected: Value<bool>,
    #[prop(default = ControlSize::Medium)] size: Value<ControlSize>,
    #[caller] caller: Caller,
) -> impl Element {
    let disabled = caller.disabled().unwrap_or_else(|| false.into_value());
    let mode = shell_nav_mode();
    let mobile = use_app_shell_mobile();
    let drawer = mode == ShellNavMode::Drawer;
    let label_view =
        (!icon_only.get() || drawer).then(|| content.unwrap_or_else(|| text(label).into_view()));
    let root = node()
        .name(label)
        .with_node(move |node| node.selected(selected))
        .with_node(move |node| {
            node.current_item(move || selected.get().then_some(kivact::view::CurrentItem::Page))
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
    if caller.has_press() {
        root.with_node(move |node| node.role(Role::Link))
            .focusable(move || !disabled.get())
            .on_press(move || {
                if drawer && !disabled.get() {
                    mobile.hide();
                }
            })
    } else {
        root
    }
}

fn top_item_style(
    icon_only: bool,
    selected: bool,
    disabled: bool,
    open: bool,
) -> kivact::style::Style {
    let base = style()
        .flex_direction(logical_row())
        .align_items(Align::Center)
        .gap(s::SPACE_S2)
        .padding_x(if icon_only { s::SPACE_S2 } else { s::SPACE_S3 })
        .padding_y(s::SPACE_S1_5)
        .border_radius(RADIUS_ELEMENT)
        .font_size(t::LABEL.size_px)
        .line_height(t::LABEL.line_height_px / t::LABEL.size_px)
        .font_weight(if selected {
            t::WEIGHT_SEMIBOLD
        } else {
            t::LABEL.weight
        })
        .color(if disabled {
            c::TEXT_DISABLED
        } else if selected || open {
            c::TEXT_PRIMARY
        } else {
            c::TEXT_SECONDARY
        })
        .focus_visible(ring(c::ACCENT));
    if selected {
        base.background(c::NEUTRAL)
    } else if open {
        base.background(c::OVERLAY_HOVER)
    } else if disabled {
        base
    } else {
        base.hover(style().background(c::OVERLAY_HOVER))
            .active(style().background(c::OVERLAY_PRESSED))
    }
}

/// Create an action entry for a top navigation menu.
pub fn top_nav_menu_item(title: impl IntoText) -> NavHeadingMenuItem {
    nav_heading_menu_item(title)
}

/// A top navigation menu of heading-menu entries.
///
/// The menu opens on pointer hover or press after `open_delay` and closes after `close_delay`.
/// In the drawer it becomes a disclosure of the same entries.
#[component]
pub fn top_nav_menu<I: IntoIterator<Item = NavHeadingMenuItem>>(
    label: Value<String>,
    items: I,
    #[prop(default = Duration::from_millis(150))] open_delay: Duration,
    #[prop(default = Duration::from_millis(200))] close_delay: Duration,
) -> impl Element {
    let items: Rc<[NavHeadingMenuItem]> = items.into_iter().collect::<Vec<_>>().into();
    let delays = Delays {
        open: open_delay,
        close: close_delay,
    };
    if shell_nav_mode() == ShellNavMode::Drawer {
        let item = side_nav_item(label)
            .children(move || items.iter().map(drawer_menu_item).collect::<Vec<_>>());
        return node().child(item);
    }
    let align = try_use_context::<TopNavCx>().map_or(OverlayAlign::Start, |cx| cx.align);
    let menu = nav_heading_menu(items.iter().cloned());
    node().child(nav_menu_view(
        menu,
        label,
        label,
        NavMenuLook::Top,
        delays,
        align,
        move |open| menu_trigger(label, open),
    ))
}

fn menu_trigger(label: Value<String>, open: Value<bool>) -> Node {
    row((
        text(label),
        icon("chevron-down").style(move || {
            style()
                .size(m::ICON_SM, m::ICON_SM)
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
struct MegaClose(Signal<bool>);

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
        kivact::custom::scope(move || {
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
            let behavior = use_popup(open).anchor(trigger_ref);
            let trigger = nav_panel.trigger(
                menu_trigger(label, open.into_value())
                    .node_ref(trigger_ref)
                    .with_node(move |node| node.role(Role::Button))
                    .focusable(true)
                    .with_node(move |node| node.with_node(move |node| node.expanded(open)))
                    .with_node(move |node| node.controls(surface_ref))
                    .on_press(move || nav_panel.toggle()),
            );
            let cx = try_use_context::<TopNavCx>();
            let align = cx.map_or(OverlayAlign::Start, |cx| cx.align);
            let anchor = cx.map_or(trigger_ref, |cx| cx.anchor);
            let surface = show(behavior.is_open(), move || {
                let body = mega_columns(items.as_ref(), featured.as_ref(), open);
                let panel = nav_panel.surface(scroll(body)).node_ref(surface_ref).style(
                    popup_surface()
                        .max_height(pct(100) - s::SPACE_S3)
                        .clip(true)
                        .border_top(m::BORDER_WIDTH, c::BORDER),
                );
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

/// An action item with a title, optional supporting text and an optional icon in a mega menu.
///
/// `description` is the supporting text under the title; it sits in the label, not the root.
/// Activation runs the caller's `on_press`, then closes the flyout and the mobile drawer.
#[component]
pub fn top_nav_mega_menu_item(
    title: Value<String>,
    description: Option<Value<String>>,
    icon: Option<View>,
    #[caller] caller: Caller,
) -> impl Element {
    let drawer = shell_nav_mode() == ShellNavMode::Drawer;
    let mobile = use_app_shell_mobile();
    let popup = try_use_context::<MegaClose>();
    let glyph = icon.map(move |glyph| mega_glyph(glyph, drawer));
    let labels = column((
        text(title).style(title_style),
        description.map(|value| text(value).style(supporting)),
    ))
    .style(
        style()
            .min_width(0)
            .gap(if drawer { s::SPACE_S0_5 } else { s::SPACE_S1 }),
    );
    let root = node()
        .name(title)
        .with_node(move |node| {
            node.base_style(move || {
                let base = style()
                    .flex_direction(logical_row())
                    .align_items(Align::Start)
                    .gap(if drawer { s::SPACE_S2 } else { s::SPACE_S3 })
                    .padding(s::SPACE_S3)
                    .border_radius(RADIUS_ELEMENT)
                    .width(pct(100))
                    .hover(style().background(c::OVERLAY_HOVER))
                    .focus_visible(ring(c::ACCENT));
                if drawer {
                    if rtl() {
                        base.padding_right(s::SPACE_S6)
                    } else {
                        base.padding_left(s::SPACE_S6)
                    }
                } else {
                    base
                }
            })
        })
        .child((glyph, labels));
    if caller.has_press() {
        root.with_node(move |node| node.role(Role::Link))
            .focusable(true)
            .on_press(move || {
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
}

fn title_style() -> kivact::style::Style {
    style()
        .font_size(t::BODY.size_px)
        .line_height(t::BODY.line_height_px / t::BODY.size_px)
        .font_weight(t::WEIGHT_SEMIBOLD)
        .color(c::TEXT_PRIMARY)
}

/// A featured mega-menu card with a title, optional supporting text, image, body and link.
///
/// `link` is the label of the link row, and `on_press` runs when that row is activated (the
/// link is a part of the card, so its press is not the card's). Activation closes the flyout
/// and the mobile drawer.
#[component]
pub fn top_nav_featured_card(
    title: Value<String>,
    description: Option<Value<String>>,
    image: Option<View>,
    content: Option<View>,
    link: Option<Value<String>>,
    on_press: Option<Callback<()>>,
) -> impl Element {
    let drawer = shell_nav_mode() == ShellNavMode::Drawer;
    let mobile = use_app_shell_mobile();
    let popup = try_use_context::<MegaClose>();
    let image = image.map(|image| {
        node()
            .style(style().width(pct(100)).height(140).clip(true))
            .child(image)
    });
    let link = link.map(|label| {
        row((text(label), text(if rtl() { "←" } else { "→" })))
            .name(label)
            .with_node(move |node| node.role(Role::Link))
            .focusable(true)
            .on_press(move || {
                if let Some(action) = &on_press {
                    action.call(());
                }
                if let Some(popup) = popup {
                    popup.0.set(false);
                }
                if drawer {
                    mobile.hide();
                }
            })
            .style(
                supporting()
                    .flex_direction(logical_row())
                    .gap(s::SPACE_S1)
                    .color(c::TEXT_ACCENT)
                    .font_weight(t::WEIGHT_SEMIBOLD)
                    .focus_visible(ring(c::ACCENT)),
            )
    });
    let body = column((
        text(title).style(title_style),
        description.map(|description| text(description).style(supporting)),
        content,
        link,
    ))
    .style(style().padding(s::SPACE_S4).gap(s::SPACE_S2));
    node()
        .with_node(move |node| {
            node.base_style(move || {
                let base = style()
                    .flex_direction(FlexDirection::Column)
                    .clip(true)
                    .border_radius(RADIUS_CONTAINER)
                    .background(c::BG_MUTED);
                if drawer {
                    let base = base.margin_top(s::SPACE_S2);
                    if rtl() {
                        base.margin_right(s::SPACE_S6)
                    } else {
                        base.margin_left(s::SPACE_S6)
                    }
                } else {
                    base
                }
            })
        })
        .child((image, body))
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
                    style()
                        .flex_direction(if mode == ShellNavMode::Drawer {
                            FlexDirection::Column
                        } else {
                            logical_row()
                        })
                        .gap(s::SPACE_S1)
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

fn mega_columns(items: Option<&ViewFn>, featured: Option<&ViewFn>, open: Signal<bool>) -> Node {
    let grid = items.map(|items| {
        let items = items.clone();
        layout_grid(provide(MegaClose(open), move || items.call(())))
            .columns(GridColumns::Fixed(2))
            .gap(Space::S2)
            .style(
                style()
                    .flex_grow(2)
                    .flex_shrink(1)
                    .flex_basis(300)
                    .min_width(0),
            )
    });
    let featured = featured.map(|featured| {
        let featured = featured.clone();
        node()
            .child(provide(MegaClose(open), move || featured.call(())))
            .style(
                style()
                    .flex_grow(1)
                    .flex_shrink(1)
                    .flex_basis(200)
                    .min_width(0),
            )
    });
    row((grid, featured)).style(move || {
        style()
            .flex_direction(logical_row())
            .flex_wrap(true)
            .gap(s::SPACE_S6)
            .padding(s::SPACE_S3)
            .width(960)
            .max_width((kivact::platform::window_size().get().width - Space::S4.px()).max(0.0))
    })
}

fn mega_glyph(glyph: View, drawer: bool) -> Node {
    let base = style()
        .flex_shrink(0)
        .border_radius(RADIUS_ELEMENT)
        .background(c::NEUTRAL)
        .color(c::ICON_SECONDARY)
        .align_items(Align::Center)
        .justify_content(Justify::Center);
    let sized = if drawer {
        base.size(m::CONTROL_MD, m::CONTROL_MD)
    } else {
        base.size(s::SPACE_S10, s::SPACE_S10)
    };
    node().style(sized).child(glyph)
}
