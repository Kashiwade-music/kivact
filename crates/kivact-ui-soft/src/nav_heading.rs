//! Role: Render navigation headings and their shared keyboard-operated menus.
use crate::color as c;
use crate::metrics as m;
use crate::motion::{EASE_STANDARD, FAST, FAST_MAX};
use crate::radius::{RADIUS_CONTAINER, RADIUS_ELEMENT};
use crate::recipe_parts::{ring, shadowed};
use crate::shadows;
use crate::side_navigation::items::logical_row;
use crate::space::{self as s, Space};
use crate::stack::rtl;
use crate::type_scale as t;

use kivact::component;
use kivact::i18n::{message, message_with};
use kivact::interaction::Delays;
use kivact::interaction::use_menu;
use kivact::motion::Transition;
use kivact::state::IntoHandler;
use kivact::state::IntoValue;
use kivact::view::Element;
use kivact::view::IntoViewFn;
use kivact::view::Node;
use kivact::view::ViewFn;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::row;
use kivact::view::show;
use kivact::view::text;

use kivact::interaction::MenuEntry;
use kivact::state::Callback;
use kivact::state::IntoText;
use kivact::state::Value;
use kivact::state::callback;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::ThemeMode;
use kivact::style::TransitionProp;
use kivact::style::style;
use kivact::style::use_theme;
use kivact::ui::ControlSize;
use kivact::view::IntoView;
use kivact::view::OverlayAlign;
use kivact::view::Placement;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::for_each;
use kivact::view::node_ref;
use kivact::view::overlay;
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone)]
/// An action entry for a navigation heading menu, with optional supporting content.
pub struct NavHeadingMenuItem {
    pub(crate) label: Value<String>,
    pub(crate) description: Option<Value<String>>,
    pub(crate) icon: Option<ViewFn>,
    pub(crate) disabled: bool,
    pub(crate) action: Option<Callback<()>>,
}

/// Create a heading-menu action with a reactive label.
pub fn nav_heading_menu_item(label: impl IntoText) -> NavHeadingMenuItem {
    NavHeadingMenuItem {
        label: label.into_text_value(),
        description: None,
        icon: None,
        disabled: false,
        action: None,
    }
}

impl NavHeadingMenuItem {
    /// Set the supporting text below the menu entry label.
    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(description.into_text_value());
        self
    }
    /// Build the entry icon each time the menu content opens.
    pub fn icon<V: IntoView>(mut self, icon: impl IntoViewFn<V>) -> Self {
        self.icon = Some(icon.into_view_fn());
        self
    }
    /// Prevent this entry from being chosen.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Handle selection of this heading-menu entry.
    pub fn on_press<M>(mut self, action: impl IntoHandler<(), M>) -> Self {
        let action: Callback<()> =
            kivact::state::Callback::from_handler(kivact::state::IntoHandler::into_handler(action));
        self.action = Some(callback(move |_| action.call(())));
        self
    }
}

/// The entries and sizing of a menu attached to a navigation heading.
pub struct NavHeadingMenu {
    items: Rc<[NavHeadingMenuItem]>,
    size: ControlSize,
    min_width: Option<kivact::state::Value<f64>>,
}

/// Collect heading-menu entries in their visible order.
pub fn nav_heading_menu(items: impl IntoIterator<Item = NavHeadingMenuItem>) -> NavHeadingMenu {
    NavHeadingMenu {
        items: items.into_iter().collect::<Vec<_>>().into(),
        size: ControlSize::Medium,
        min_width: None,
    }
}

impl NavHeadingMenu {
    /// Choose the menu row size and its default minimum width.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Override the menu's minimum width in pixels.
    pub fn min_width(mut self, width: impl IntoValue<f64>) -> Self {
        self.min_width = Some(width.into_value());
        self
    }
}

#[derive(Clone, Copy)]
pub(crate) enum NavMenuLook {
    Heading,
    Top,
}

pub(crate) fn nav_menu_view(
    menu: NavHeadingMenu,
    label: Value<String>,
    dialog_name: Value<String>,
    look: NavMenuLook,
    delays: Delays,
    align: OverlayAlign,
    trigger: impl FnOnce(Value<bool>) -> Node + 'static,
) -> View {
    let NavHeadingMenu {
        items,
        size,
        min_width,
    } = menu;
    let options = items
        .iter()
        .enumerate()
        .map(|(index, item)| MenuEntry::item(index, item.label).disabled(item.disabled))
        .collect::<Vec<_>>();
    let actions = items.clone();
    let anchor = node_ref();
    let panel_ref = node_ref();
    let nav_panel = crate::navigation_session::NavPanel::new(true, delays, None);
    let behavior = use_menu(options, nav_panel.open())
        .trigger_action(move || nav_panel.toggle())
        .on_activate(move |index| {
            if let Some(action) = actions[index].action {
                action.call(());
            }
        });
    let part = behavior.trigger();
    let opened = behavior.is_open();
    let trigger = nav_panel.trigger(
        trigger(opened)
            .node_ref(anchor)
            .with_node(move |node| node.part(part)),
    );
    let heading = matches!(look, NavMenuLook::Heading);
    let default_min_width = menu_min_width(heading, size);
    let min_width = min_width.unwrap_or_else(|| default_min_width.into_value());
    let surface = show(behavior.is_open(), move || {
        let items = items.clone();
        let rows = for_each(
            behavior.entries(),
            |(row, _)| row.clone(),
            move |entry| {
                let key = *entry.get().1.key().expect("action item");
                let part = behavior.item(key);
                nav_entry(&items[key], part, heading, size)
            },
        );
        let popup_header = heading.then(move || nav_popup_header(label));
        let surface = nav_panel
            .surface(column((popup_header, rows)))
            .node_ref(panel_ref)
            .style(move || {
                let anchor_width = anchor.rect().get().map_or(0.0, |rect| rect.width);
                popup_surface()
                    .min_width(if heading {
                        min_width.get().max(anchor_width + Space::S4.px())
                    } else {
                        min_width.get()
                    })
                    .padding(s::SPACE_S1)
                    .gap(if heading { s::SPACE_S0_5 } else { s::SPACE_S1 })
            });
        let offset = if heading {
            -anchor.rect().get().map_or(0.0, |rect| rect.height) - Space::S2.px()
        } else {
            4.0
        };
        overlay(anchor, Placement::Below, surface)
            .align(align)
            .offset(offset)
            .with_node(move |node| node.part(behavior.content()))
            .name(dialog_name)
    });
    (trigger, surface).into_view()
}

/// The shadows of a floating surface: the kit's low elevation for the current theme mode.
fn low_shadowed(base: kivact::style::Style) -> kivact::style::Style {
    let dark = use_theme().get().mode() == ThemeMode::Dark;
    shadowed(base, shadows::low(dark))
}

pub(crate) fn supporting() -> kivact::style::Style {
    let role = t::SUPPORTING;
    style()
        .font_size(role.size_px)
        .line_height(role.line_height_px / role.size_px)
        .color(c::TEXT_SECONDARY)
        .single_line()
}

pub(crate) fn popup_surface() -> kivact::style::Style {
    let hidden = style()
        .opacity(0.0)
        .translate_y(-Space::S2.px())
        .scale_xy(0.95, 0.95);
    low_shadowed(
        style()
            .background(c::BG_POPOVER)
            .border_radius(RADIUS_CONTAINER)
            .enter(hidden)
            .transition(
                TransitionProp::All,
                Transition::tween(FAST_MAX as f64, EASE_STANDARD),
            ),
    )
}

/// The labels, icon and menu of a navigation heading and its press handlers.
struct HeadingPresses {
    heading: Option<Callback<()>>,
    superheading: Option<Callback<()>>,
    subheading: Option<Callback<()>>,
}

/// A navigation heading with optional icon, supporting labels and action menu.
///
/// `heading`, `superheading` and `subheading` are its labels. `on_heading_press`,
/// `on_superheading_press` and `on_subheading_press` run when the matching label is pressed.
/// `menu` opens from the heading. `end_content` sits at the logical end and is hidden in the
/// collapsed rail.
#[component]
pub fn side_nav_heading(
    heading: Value<String>,
    superheading: Option<Value<String>>,
    subheading: Option<Value<String>>,
    icon: Option<View>,
    end_content: Option<View>,
    menu: Option<NavHeadingMenu>,
    on_heading_press: Option<Callback<()>>,
    on_superheading_press: Option<Callback<()>>,
    on_subheading_press: Option<Callback<()>>,
) -> impl Element {
    let presses = HeadingPresses {
        heading: on_heading_press,
        superheading: on_superheading_press,
        subheading: on_subheading_press,
    };
    node().child(heading_content(
        heading,
        superheading,
        subheading,
        icon,
        end_content,
        menu,
        presses,
    ))
}

/// Create a heading with the same behavior for top navigation.
pub fn top_nav_heading(heading: impl IntoText) -> SideNavHeading {
    side_nav_heading(heading.into_text_value())
}

fn heading_content(
    heading: Value<String>,
    superheading: Option<Value<String>>,
    subheading: Option<Value<String>>,
    glyph: Option<View>,
    end: Option<View>,
    menu: Option<NavHeadingMenu>,
    presses: HeadingPresses,
) -> View {
    let rail = crate::side_navigation::context::nav_is_rail();
    let has_icon = glyph.is_some();
    let interactive = menu.is_some() || presses.heading.is_some();
    let trigger = move |open: Value<bool>, menu: bool| {
        let heading_view = heading_text(heading, presses.heading, true);
        let labels = column((
            superheading.map(|label| heading_text(label, presses.superheading, false)),
            heading_view,
            subheading.map(|label| heading_text(label, presses.subheading, false)),
        ))
        .hidden(rail)
        .style(style().min_width(0));
        let chevron = menu.then(|| {
            node()
                .hidden(rail)
                .style(|| {
                    style()
                        .size(m::CONTROL_SM, m::CONTROL_SM)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                        .color(c::ICON_SECONDARY)
                })
                .child(icon("chevron-down").style(move || {
                    style()
                        .size(m::ICON_SM, m::ICON_SM)
                        .rotate(if open.get() { 180 } else { 0 })
                }))
        });
        row((
            glyph,
            labels,
            end.map(|end| node().hidden(rail).child(end)),
            chevron,
        ))
        .style(move || heading_trigger_style(rail, has_icon, interactive))
    };
    if let Some(menu) = menu {
        let name = message_with(
            "soft.side_nav.heading_open_menu",
            "{heading}, Open menu",
            move || [("heading", heading.get())],
        );
        nav_menu_view(
            menu,
            heading,
            message("soft.side_nav.heading_dialog", "Navigation menu"),
            NavMenuLook::Heading,
            Delays {
                open: Duration::ZERO,
                close: Duration::from_millis(200),
            },
            OverlayAlign::Start,
            move |open| trigger(open, true).name(name),
        )
    } else {
        trigger(false.into_value(), false).into_view()
    }
}

fn heading_text(label: Value<String>, action: Option<Callback<()>>, primary: bool) -> Node {
    let label_view = text(label).style(move || {
        if !primary {
            return supporting();
        }
        let role = t::LARGE;
        style()
            .font_size(role.size_px)
            .line_height(role.line_height_px / role.size_px)
            .font_weight(t::WEIGHT_SEMIBOLD)
            .single_line()
    });
    if let Some(action) = action {
        node()
            .with_node(move |node| node.role(Role::Link))
            .name(label)
            .focusable(true)
            .on_press(move || action.call(()))
            .style(|| style().focus_visible(ring(c::ACCENT)))
            .child(label_view)
    } else {
        label_view
    }
}

fn nav_entry(
    item: &NavHeadingMenuItem,
    part: kivact::custom::Part<kivact::interaction::MenuItemState>,
    heading: bool,
    size: ControlSize,
) -> Node {
    let state = part.state();
    let leading = item.icon.as_ref().map(|slot| {
        node()
            .style(move || {
                let base = style()
                    .flex_shrink(0)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .color(c::ICON_SECONDARY);
                if heading {
                    base.size(m::ICON_SM, m::ICON_SM)
                } else {
                    base.size(s::SPACE_S10, s::SPACE_S10)
                        .border_radius(RADIUS_ELEMENT)
                        .background(c::NEUTRAL)
                }
            })
            .child(slot.call(()))
    });
    let label = text(item.label).style(move || {
        let base = style().single_line().color(c::TEXT_PRIMARY);
        if heading {
            base
        } else {
            base.font_weight(t::WEIGHT_SEMIBOLD)
        }
    });
    let description = item
        .description
        .map(|description| text(description).style(supporting));
    row((
        leading,
        column((label, description)).style(move || {
            style()
                .flex_grow(1)
                .min_width(0)
                .gap(if heading { 0.0 } else { Space::S1.px() })
        }),
    ))
    .with_node(move |node| node.part(part))
    .style(move || nav_entry_style(state.get().input, heading, size))
}

fn nav_entry_style(
    input: kivact::custom::PartState,
    heading: bool,
    size: ControlSize,
) -> kivact::style::Style {
    let (x, y) = if !heading {
        (s::SPACE_S3, s::SPACE_S3)
    } else {
        match size {
            ControlSize::Small => (s::SPACE_S2, s::SPACE_S1),
            ControlSize::Large => (s::SPACE_S3, s::SPACE_S3),
            _ => (s::SPACE_S2, s::SPACE_S2),
        }
    };
    let role = t::BODY;
    style()
        .flex_direction(logical_row())
        .align_items(Align::Center)
        .gap(if heading { s::SPACE_S2 } else { s::SPACE_S3 })
        .padding_x(x)
        .padding_y(y)
        .font_size(role.size_px)
        .line_height(role.line_height_px / role.size_px)
        .border_radius(RADIUS_ELEMENT)
        .background(
            if !input.is_disabled() && (input.is_focused() || input.is_hovered()) {
                c::OVERLAY_HOVER
            } else {
                kivact::style::Color::TRANSPARENT
            },
        )
        .disabled(style().opacity(m::DISABLED_OPACITY))
        .transition(
            TransitionProp::All,
            Transition::tween(FAST as f64, EASE_STANDARD),
        )
}

fn heading_trigger_style(
    rail: Value<bool>,
    has_icon: bool,
    interactive: bool,
) -> kivact::style::Style {
    let mut base = style()
        .flex_direction(logical_row())
        .gap(s::SPACE_S2)
        .align_items(Align::Center)
        .min_height(m::CONTROL_MD)
        .min_width(0)
        .color(c::TEXT_PRIMARY);
    if !rail.get() {
        base = base.padding_x(s::SPACE_S2);
        if has_icon {
            base = if rtl() {
                base.padding_right(0)
            } else {
                base.padding_left(0)
            };
        }
    }
    if interactive {
        base = base
            .border_radius(RADIUS_ELEMENT)
            .hover(style().background(c::OVERLAY_HOVER))
            .focus_visible(ring(c::ACCENT));
    }
    base
}

fn nav_popup_header(label: Value<String>) -> Node {
    row((
        text(label),
        icon("chevron-down").style(style().size(16, 16).rotate(180)),
    ))
    .style(|| {
        style()
            .flex_direction(logical_row())
            .padding(s::SPACE_S2)
            .gap(s::SPACE_S2)
            .align_items(Align::Center)
            .justify_content(Justify::SpaceBetween)
    })
}

fn menu_min_width(heading: bool, size: ControlSize) -> f64 {
    if heading {
        match size {
            ControlSize::Small => 160.0,
            ControlSize::Large => 240.0,
            _ => 200.0,
        }
    } else {
        280.0
    }
}
