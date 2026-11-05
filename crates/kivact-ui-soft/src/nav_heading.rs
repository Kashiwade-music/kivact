//! Role: Render navigation headings and their shared keyboard-operated menus.
use crate::side_navigation::items::logical_row;
use crate::stack::rtl;

use kivact::i18n::{message, message_with};
use kivact::interaction::Delays;
use kivact::interaction::use_menu_with_open;
use kivact::motion::Transition;
use kivact::state::IntoValue;
use kivact::view::IntoViewFn;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ViewFn;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::row;
use kivact::view::text;

use kivact::interaction::MenuEntry;
use kivact::state::Callback;
use kivact::state::IntoText;
use kivact::state::Value;
use kivact::state::callback;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::TransitionProp;
use kivact::style::style;
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

// xtask-allow(state-fields): Independent immutable label, icon, description, caller availability and action metadata; Menu owns interaction state.
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
    pub fn on_press(mut self, action: impl Into<Callback<()>>) -> Self {
        let action: Callback<()> = action.into();
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
    let nav_panel = crate::navigation_session::NavPanel::new(true, delays);
    let behavior = use_menu_with_open(options, nav_panel.open())
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
    let default_min_width = if heading {
        match size {
            ControlSize::Small => 160.0,
            ControlSize::Large => 240.0,
            _ => 200.0,
        }
    } else {
        280.0
    };
    let min_width = min_width.unwrap_or_else(|| default_min_width.into_value());
    let surface = behavior.show(move || {
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
        let popup_header = heading.then(|| {
            row((
                text(label),
                icon("chevron-down").style(style().size(16, 16).rotate(180)),
            ))
            .style(|| {
                style()
                    .direction(logical_row())
                    .padding(kivact::style::use_tokens::<crate::Tokens>().spacing.s2)
                    .gap(kivact::style::use_tokens::<crate::Tokens>().spacing.s2)
                    .align_items(Align::Center)
                    .justify_content(Justify::SpaceBetween)
            })
        });
        let surface = nav_panel
            .surface(column((popup_header, rows)))
            .node_ref(panel_ref)
            .style(move || {
                let t = kivact::style::use_tokens::<crate::Tokens>();
                let anchor_width = anchor.rect().get().map_or(0.0, |rect| rect.width);
                popup_surface()
                    .min_width(if heading {
                        min_width.get().max(anchor_width + t.spacing.s4)
                    } else {
                        min_width.get()
                    })
                    .padding(t.spacing.s1)
                    .gap(if heading {
                        t.spacing.s0_5
                    } else {
                        t.spacing.s1
                    })
            });
        let offset = if heading {
            -anchor.rect().get().map_or(0.0, |rect| rect.height)
                - kivact::style::use_tokens::<crate::Tokens>().spacing.s2
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

pub(crate) fn supporting() -> kivact::style::Style {
    let t = kivact::style::use_tokens::<crate::Tokens>();
    let role = t.typography.supporting;
    style()
        .font_size(role.size_px)
        .line_height(role.line_height_px / role.size_px)
        .color(t.colors.text_secondary)
        .single_line()
}

pub(crate) fn popup_surface() -> kivact::style::Style {
    let t = kivact::style::use_tokens::<crate::Tokens>();
    let hidden = style()
        .opacity(0.0)
        .translate_y(-t.spacing.s2)
        .scale_xy(0.95, 0.95);
    let mut base = style()
        .background(t.colors.bg_popover)
        .radius(t.radius.container)
        .enter(hidden)
        .transition(
            TransitionProp::Paint,
            Transition::tween(t.motion.fast_max as f64, t.motion.ease_standard),
        );
    for shadow in &t.shadows.low {
        base = base.shadow(*shadow);
    }
    base
}

/// A navigation heading with optional icon, supporting labels and action menu.
pub struct SideNavHeading {
    root: Node,
    heading: Value<String>,
    superheading: Option<Value<String>>,
    subheading: Option<Value<String>>,
    icon: Option<View>,
    end: Option<View>,
    menu: Option<NavHeadingMenu>,
    heading_action: Option<Callback<()>>,
    super_action: Option<Callback<()>>,
    sub_action: Option<Callback<()>>,
}

/// Create a heading styled for the sidebar and its collapsed rail.
pub fn side_nav_heading(heading: impl IntoText) -> SideNavHeading {
    SideNavHeading {
        root: node(),
        heading: heading.into_text_value(),
        superheading: None,
        subheading: None,
        icon: None,
        end: None,
        menu: None,
        heading_action: None,
        super_action: None,
        sub_action: None,
    }
}

/// Create a heading with the same behavior for top navigation.
pub fn top_nav_heading(heading: impl IntoText) -> SideNavHeading {
    side_nav_heading(heading)
}

impl SideNavHeading {
    /// Set the supporting label above the main heading.
    pub fn superheading(mut self, heading: impl IntoText) -> Self {
        self.superheading = Some(heading.into_text_value());
        self
    }
    /// Set the supporting label below the main heading.
    pub fn subheading(mut self, heading: impl IntoText) -> Self {
        self.subheading = Some(heading.into_text_value());
        self
    }
    /// Mount a leading heading icon once.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    /// Mount a leading logo once in the heading icon slot.
    pub fn logo(self, logo: impl IntoView) -> Self {
        self.icon(logo)
    }
    /// Mount trailing heading content once; it is hidden in the collapsed rail.
    pub fn end_content(mut self, content: impl IntoView) -> Self {
        self.end = Some(content.into_view());
        self
    }
    /// Attach the menu opened by activating the heading.
    pub fn menu(mut self, menu: NavHeadingMenu) -> Self {
        self.menu = Some(menu);
        self
    }
    /// Handle activation of the main heading label.
    pub fn on_heading_press(mut self, press: impl Into<Callback<()>>) -> Self {
        let press: Callback<()> = press.into();
        self.heading_action = Some(callback(move |_| press.call(())));
        self
    }
    /// Handle activation of the supporting label above the heading.
    pub fn on_superheading_press(mut self, press: impl Into<Callback<()>>) -> Self {
        let press: Callback<()> = press.into();
        self.super_action = Some(callback(move |_| press.call(())));
        self
    }
    /// Handle activation of the supporting label below the heading.
    pub fn on_subheading_press(mut self, press: impl Into<Callback<()>>) -> Self {
        let press: Callback<()> = press.into();
        self.sub_action = Some(callback(move |_| press.call(())));
        self
    }
}

impl NodeBuilder for SideNavHeading {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SideNavHeading {
    fn into_view(self) -> View {
        let Self {
            root,
            heading,
            superheading,
            subheading,
            icon: glyph,
            end,
            menu,
            heading_action,
            super_action,
            sub_action,
        } = self;
        kivact::view::component(move || {
            let rail = crate::side_navigation::context::nav_is_rail();
            let has_icon = glyph.is_some();
            let interactive = menu.is_some() || heading_action.is_some();
            let trigger = move |open: Value<bool>, menu: bool| {
                let heading_view = heading_text(heading, heading_action, true);
                let labels = column((
                    superheading.map(|label| heading_text(label, super_action, false)),
                    heading_view,
                    subheading.map(|label| heading_text(label, sub_action, false)),
                ))
                .hidden(rail)
                .style(style().min_width(0));
                let chevron = menu.then(|| {
                    node()
                        .hidden(rail)
                        .style(|| {
                            let t = kivact::style::use_tokens::<crate::Tokens>();
                            style()
                                .size(t.sizes.control_sm, t.sizes.control_sm)
                                .align_items(Align::Center)
                                .justify_content(Justify::Center)
                                .color(t.colors.icon_secondary)
                        })
                        .child(icon("chevron-down").style(move || {
                            let t = kivact::style::use_tokens::<crate::Tokens>();
                            style()
                                .size(t.sizes.icon_sm, t.sizes.icon_sm)
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
            let content = if let Some(menu) = menu {
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
            };
            root.child(content)
        })
    }
}

fn heading_text(label: Value<String>, action: Option<Callback<()>>, primary: bool) -> Node {
    let label_view = text(label).style(move || {
        if !primary {
            return supporting();
        }
        let t = kivact::style::use_tokens::<crate::Tokens>();
        let role = t.typography.large;
        style()
            .font_size(role.size_px)
            .line_height(role.line_height_px / role.size_px)
            .weight(t.typography.weight_semibold)
            .single_line()
    });
    if let Some(action) = action {
        node()
            .with_node(move |node| node.role(Role::Link))
            .name(label)
            .focusable(true)
            .on_press(move || action.call(()))
            .style(|| {
                let t = kivact::style::use_tokens::<crate::Tokens>();
                style().focus_visible(crate::recipe_style::ring(&t, t.colors.accent))
            })
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
                let t = kivact::style::use_tokens::<crate::Tokens>();
                let base = style()
                    .shrink(0)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .color(t.colors.icon_secondary);
                if heading {
                    base.size(t.sizes.icon_sm, t.sizes.icon_sm)
                } else {
                    base.size(t.spacing.s10, t.spacing.s10)
                        .radius(t.radius.element)
                        .background(t.colors.neutral)
                }
            })
            .child(slot.call(()))
    });
    let label = text(item.label).style(move || {
        let t = kivact::style::use_tokens::<crate::Tokens>();
        let base = style().single_line().color(t.colors.text_primary);
        if heading {
            base
        } else {
            base.weight(t.typography.weight_semibold)
        }
    });
    let description = item
        .description
        .map(|description| text(description).style(supporting));
    row((
        leading,
        column((label, description)).style(move || {
            style().grow(1).min_width(0).gap(if heading {
                0.0
            } else {
                kivact::style::use_tokens::<crate::Tokens>().spacing.s1
            })
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
    let t = kivact::style::use_tokens::<crate::Tokens>();
    let (x, y) = if !heading {
        (t.spacing.s3, t.spacing.s3)
    } else {
        match size {
            ControlSize::Small => (t.spacing.s2, t.spacing.s1),
            ControlSize::Large => (t.spacing.s3, t.spacing.s3),
            _ => (t.spacing.s2, t.spacing.s2),
        }
    };
    style()
        .direction(logical_row())
        .align_items(Align::Center)
        .gap(if heading { t.spacing.s2 } else { t.spacing.s3 })
        .padding_x(x)
        .padding_y(y)
        .font_size(t.typography.body.size_px)
        .line_height(t.typography.body.line_height_px / t.typography.body.size_px)
        .radius(t.radius.element)
        .background(if !input.disabled && (input.focused || input.hovered) {
            t.colors.overlay_hover
        } else {
            kivact::style::Color::TRANSPARENT
        })
        .disabled(style().opacity(t.sizes.disabled_opacity))
        .transition(
            TransitionProp::Paint,
            Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
        )
}

fn heading_trigger_style(
    rail: Value<bool>,
    has_icon: bool,
    interactive: bool,
) -> kivact::style::Style {
    let t = kivact::style::use_tokens::<crate::Tokens>();
    let mut base = style()
        .direction(logical_row())
        .gap(t.spacing.s2)
        .align_items(Align::Center)
        .min_height(t.sizes.control_md)
        .min_width(0)
        .color(t.colors.text_primary);
    if !rail.get() {
        base = base.padding_x(t.spacing.s2);
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
            .radius(t.radius.element)
            .hover(style().background(t.colors.overlay_hover))
            .focus_visible(crate::recipe_style::ring(&t, t.colors.accent));
    }
    base
}
