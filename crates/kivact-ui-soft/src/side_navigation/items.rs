//! Role: Render nested sidebar items and section rows.

use super::context::{NavSurface, SideNavCx, nav_cx};

use crate::app_shell::{ShellNavMode, shell_nav_mode};
use crate::color as c;
use crate::metrics as m;
use crate::motion::{EASE_STANDARD, FAST, MEDIUM};
use crate::radius::{RADIUS_CONTAINER, RADIUS_ELEMENT};
use crate::recipe_parts::{ring, shadowed};
use crate::shadows;
use crate::space::{self as s, Space};
use crate::stack::rtl;
use crate::type_scale as t;

use kivact::Prop;
use kivact::component;
use kivact::custom::Caller;
use kivact::i18n::message_with;
use kivact::interaction::Delays;
use kivact::interaction::use_disclosure;
use kivact::interaction::use_popup;
use kivact::motion::Transition;
use kivact::state::Callback;
use kivact::state::IntoValue;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::provide;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::FlexDirection;
use kivact::style::Justify;
use kivact::style::ThemeMode;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::style;
use kivact::style::use_theme;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::button;
use kivact::ui::tooltip;
use kivact::view::Element;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::Placement;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::ViewFn;
use kivact::view::column;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::overlay;
use kivact::view::row;
use kivact::view::show;
use kivact::view::text;
use std::time::Duration;

#[derive(Clone, Prop)]
/// Ownership and initial expansion of a sidebar item's nested entries.
pub enum ItemCollapse {
    /// Keep nested entries expanded without a collapse toggle.
    AlwaysOpen,
    /// Keep collapse state locally, starting from `default_collapsed`.
    Uncontrolled {
        /// Whether nested entries start collapsed.
        default_collapsed: bool,
    },
    /// Read and write the caller's collapse binding.
    Controlled(Signal<bool>),
}

/// A sidebar entry with optional nested navigation and row actions.
///
/// The label is the entry's accessible name. `on_press` is put on the entry's own row, not on
/// its nested children. The caller's `disabled` is read from `caller` and stops the entry.
#[component]
pub fn side_nav_item(
    label: Value<String>,
    icon: Option<View>,
    selected_icon: Option<View>,
    #[prop(default = false)] selected: Value<bool>,
    end_content: Option<View>,
    actions: Option<View>,
    children: Option<ViewFn>,
    #[prop(default = ItemCollapse::Uncontrolled { default_collapsed: false })] collapsible: Value<
        ItemCollapse,
    >,
    #[prop(default = ControlSize::Medium)] size: Value<ControlSize>,
    on_press: Option<Callback<()>>,
    #[caller] caller: Caller,
) -> impl Element {
    let disabled = caller.disabled().unwrap_or_else(|| false.into_value());
    let has_children = children.is_some();
    let context = ItemContext::new(
        label,
        selected,
        disabled,
        collapsible,
        has_children,
        on_press,
        size,
    );
    context.render(
        node(),
        ItemSlots {
            leading: icon,
            selected_icon,
            end: end_content,
            actions,
            children,
        },
    )
}

fn icon_box(
    glyph: View,
    hidden: impl Fn() -> bool + 'static,
    selected: Value<bool>,
    disabled: Value<bool>,
) -> Node {
    node()
        .hidden(hidden)
        .style(move || {
            style()
                .size(m::ICON_SM, m::ICON_SM)
                .flex_shrink(0)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .color(if disabled.get() {
                    c::ICON_DISABLED
                } else if selected.get() {
                    c::TEXT_PRIMARY
                } else {
                    c::ICON_SECONDARY
                })
        })
        .child(glyph)
}

pub(crate) fn logical_row() -> FlexDirection {
    if rtl() {
        FlexDirection::RowReverse
    } else {
        FlexDirection::Row
    }
}

pub(crate) fn side_row_style(
    size: ControlSize,
    depth: usize,
    rail: bool,
    selected: bool,
    disabled: bool,
) -> kivact::style::Style {
    let height = match size {
        ControlSize::Small => m::CONTROL_SM,
        ControlSize::Large => m::CONTROL_LG,
        _ => m::CONTROL_MD,
    };
    let pad = if size == ControlSize::Small {
        Space::S1.px()
    } else {
        Space::S2.px()
    };
    let base = style()
        .flex_direction(logical_row())
        .align_items(Align::Center)
        .gap(s::SPACE_S2)
        .height(height)
        .border_radius(RADIUS_ELEMENT)
        .font_size(t::LABEL.size_px)
        .line_height(t::LABEL.line_height_px / t::LABEL.size_px)
        .font_weight(if selected {
            t::WEIGHT_MEDIUM
        } else {
            t::WEIGHT_NORMAL
        })
        .color(if disabled {
            c::TEXT_DISABLED
        } else {
            c::TEXT_PRIMARY
        })
        .focus_visible(ring(c::ACCENT));
    let base = if rail {
        base.width(height).justify_content(Justify::Center)
    } else {
        let start = pad + depth as f64 * Space::S6.px();
        let (left, right) = if rtl() { (pad, start) } else { (start, pad) };
        base.width(pct(100)).padding_left(left).padding_right(right)
    };
    if selected {
        base.background(c::NEUTRAL)
    } else if disabled {
        base
    } else {
        base.hover(style().background(c::OVERLAY_HOVER))
            .active(style().background(c::OVERLAY_PRESSED))
    }
}

/// A titled group of sidebar entries. `title` is the group's accessible name. `subtitle` and
/// `end_content` sit in its header, which `header_hidden` hides while keeping the entries.
#[component]
pub fn side_nav_section(
    title: Value<String>,
    children: View,
    subtitle: Option<Value<String>>,
    end_content: Option<View>,
    #[prop(default = false)] header_hidden: Value<bool>,
) -> impl Element {
    let cx = nav_cx();
    let title_view = text(title).style(|| style().font_weight(t::WEIGHT_SEMIBOLD).single_line());
    let header = row((
        column((title_view, subtitle.map(text))).style(style().flex_grow(1).min_width(0)),
        end_content,
    ))
    .hidden(move || header_hidden.get() || cx.rail())
    .style(|| {
        style()
            .flex_direction(logical_row())
            .padding_y(s::SPACE_S1)
            .padding_x(s::SPACE_S2)
            .gap(s::SPACE_S2)
            .align_items(Align::Center)
            .font_size(t::SUPPORTING.size_px)
            .line_height(t::SUPPORTING.line_height_px / t::SUPPORTING.size_px)
            .color(c::TEXT_SECONDARY)
    });
    node()
        .with_node(move |node| node.role(Role::Group))
        .name(title)
        .with_node(move |node| {
            node.base_style(move || {
                let base = style()
                    .flex_direction(FlexDirection::Column)
                    .padding_y(s::SPACE_S1)
                    .gap(s::SPACE_S0_5);
                if cx.rail() {
                    base.align_items(Align::Center)
                } else {
                    base
                }
            })
        })
        .child((header, children))
}

struct ItemSlots {
    leading: Option<View>,
    selected_icon: Option<View>,
    end: Option<View>,
    actions: Option<View>,
    children: Option<ViewFn>,
}
/// The collapse owner of an item that has a children factory and a collapse toggle.
#[derive(Clone, Copy)]
struct ItemExpansion {
    /// Inverted view of the item's collapse binding.
    open: Signal<bool>,
    disclosure: kivact::interaction::DisclosureApi,
}

/// Whether the item has a children factory, with that branch's owners.
#[allow(
    clippy::large_enum_variant,
    reason = "the branch is a Copy, private item description; boxing a variant would drop Copy and break the item API"
)]
#[derive(Clone, Copy)]
enum ItemBranch {
    /// No children factory.
    Leaf,
    /// A children factory exists.
    Parent {
        /// `None` for `ItemCollapse::AlwaysOpen`.
        expansion: Option<ItemExpansion>,
        /// The rail flyout panel; present only inside a collapsible sidebar.
        flyout: Option<crate::navigation_session::NavPanel>,
    },
}

impl ItemBranch {
    fn has_children(self) -> bool {
        matches!(self, Self::Parent { .. })
    }
    fn disclosure(self) -> Option<kivact::interaction::DisclosureApi> {
        match self {
            Self::Parent {
                expansion: Some(expansion),
                ..
            } => Some(expansion.disclosure),
            Self::Parent {
                expansion: None, ..
            }
            | Self::Leaf => None,
        }
    }
    fn open(self) -> Option<Signal<bool>> {
        match self {
            Self::Parent {
                expansion: Some(expansion),
                ..
            } => Some(expansion.open),
            Self::Parent {
                expansion: None, ..
            }
            | Self::Leaf => None,
        }
    }
    fn flyout(self) -> Option<crate::navigation_session::NavPanel> {
        match self {
            Self::Parent { flyout, .. } => flyout,
            Self::Leaf => None,
        }
    }
}

#[derive(Clone, Copy)]
struct ItemContext {
    cx: SideNavCx,
    mode: ShellNavMode,
    mobile: crate::mobile_nav::AppShellMobile,
    branch: ItemBranch,
    anchor: kivact::view::NodeRef,
    popup_ref: kivact::view::NodeRef,
    label: Value<String>,
    selected: Value<bool>,
    disabled: Value<bool>,
    size: Value<ControlSize>,
    action: Option<Callback<()>>,
}
impl ItemContext {
    fn new(
        label: Value<String>,
        selected: Value<bool>,
        disabled: Value<bool>,
        collapse: Value<ItemCollapse>,
        has_children: bool,
        action: Option<Callback<()>>,
        size: Value<ControlSize>,
    ) -> Self {
        let cx = nav_cx();
        let mode = shell_nav_mode();
        let mobile = crate::mobile_nav::use_app_shell_mobile();
        let branch = if has_children {
            let collapse = match collapse.get() {
                ItemCollapse::AlwaysOpen => None,
                ItemCollapse::Uncontrolled { default_collapsed } => Some(signal(default_collapsed)),
                ItemCollapse::Controlled(binding) => Some(binding),
            };
            let expansion = collapse.map(|collapse: Signal<bool>| {
                let open = binding(move || !collapse.get(), move |open| collapse.set(!open));
                ItemExpansion {
                    open,
                    disclosure: use_disclosure(open),
                }
            });
            let flyout = cx.collapsed().is_some().then(|| {
                crate::navigation_session::NavPanel::new(
                    move || cx.rail() && !disabled.get(),
                    Delays {
                        open: Duration::from_millis(150),
                        close: Duration::from_millis(200),
                    },
                    None,
                )
            });
            ItemBranch::Parent { expansion, flyout }
        } else {
            ItemBranch::Leaf
        };
        let anchor = node_ref();
        let popup_ref = node_ref();
        Self {
            cx,
            mode,
            mobile,
            branch,
            anchor,
            popup_ref,
            label,
            selected,
            disabled,
            size,
            action,
        }
    }
    fn independent(self) -> bool {
        self.branch.disclosure().is_some() && self.action.is_some()
    }
    fn render(self, root: Node, slots: ItemSlots) -> Node {
        let cx = self.cx;
        let independent = self.independent();
        let label = self.label;
        let selected = self.selected;
        let disabled = self.disabled;
        let size = self.size;
        let ItemSlots {
            leading,
            selected_icon,
            end,
            actions,
            children,
        } = slots;
        let has_icon = leading.is_some() || selected_icon.is_some();
        let has_selected_icon = selected_icon.is_some();
        let leading = leading.map(|icon| {
            icon_box(
                icon,
                move || has_selected_icon && selected.get(),
                selected,
                disabled,
            )
        });
        let selected_glyph =
            selected_icon.map(|icon| icon_box(icon, move || !selected.get(), selected, disabled));
        let label_view = text(label)
            .hidden(move || cx.rail())
            .style(style().flex_grow(1).min_width(0).single_line());
        let chevron = self.chevron();
        let (inline_chevron, separate_chevron) = if independent {
            (None, chevron)
        } else {
            (chevron, None)
        };
        let end = end.map(|end| node().hidden(move || cx.rail()).child(end));
        let primary = self.primary((leading, selected_glyph, label_view, end, inline_chevron));
        let actions = actions.map(|actions| {
            row(actions)
                .hidden(move || cx.rail())
                .style(|| style().gap(s::SPACE_S1))
        });
        let bar = row((primary, separate_chevron, actions)).style(move || {
            side_row_style(
                size.get(),
                cx.depth,
                cx.rail(),
                selected.get(),
                disabled.get(),
            )
        });
        let nested = self.nested(children.clone());
        let flyout = self.flyout(children);
        root.hidden(move || cx.rail() && !has_icon)
            .with_node(move |node| node.base_style(style().flex_direction(FlexDirection::Column)))
            .child((bar, nested, flyout))
    }
    fn chevron(self) -> Option<View> {
        let cx = self.cx;
        let disclosure = self.branch.disclosure();
        let independent = self.independent();
        let label = self.label;
        disclosure.map(|disclosure| {
            let glyph = kivact::view::icon("chevron-down").style(move || {
                style()
                    .size(s::SPACE_S6, s::SPACE_S6)
                    .rotate(if disclosure.is_open().get() { 180 } else { 0 })
                    .transition(
                        TransitionProp::All,
                        Transition::tween(FAST as f64, EASE_STANDARD),
                    )
            });
            if independent {
                let collapse = message_with(
                    "soft.side_nav.item_collapse",
                    "Collapse {label}",
                    move || [("label", label.get())],
                );
                let expand =
                    message_with("soft.side_nav.item_expand", "Expand {label}", move || {
                        [("label", label.get())]
                    });
                button(move || {
                    if disclosure.is_open().get() {
                        collapse.get()
                    } else {
                        expand.get()
                    }
                })
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Small)
                .hide_label(true)
                .disabled(self.disabled)
                .icon(glyph)
                .with_node(move |node| node.part(disclosure.trigger()))
                .hidden(move || cx.rail())
                .into_view()
            } else {
                glyph.hidden(move || cx.rail()).into_view()
            }
        })
    }
    fn primary(self, content: impl IntoView) -> View {
        let cx = self.cx;
        let has_children = self.branch.has_children();
        let disclosure = self.branch.disclosure();
        let independent = self.independent();
        let anchor = self.anchor;
        let popup_panel = self.branch.flyout();
        let label = self.label;
        let selected = self.selected;
        let disabled = self.disabled;
        let action = self.action;
        let primary = node()
            .node_ref(anchor)
            .name(label)
            .with_node(move |node| {
                node.role(if action.is_some() {
                    Role::Link
                } else {
                    Role::Button
                })
            })
            .focusable(move || {
                !disabled.get()
                    && (action.is_some() || disclosure.is_some() || popup_panel.is_some())
            })
            .disabled(disabled)
            .selected(selected)
            .with_node(move |node| {
                node.current_item(move || selected.get().then_some(kivact::view::CurrentItem::Page))
            })
            .style(move || {
                style()
                    .flex_direction(logical_row())
                    .flex_grow(1)
                    .min_width(0)
                    .align_self(Align::Stretch)
                    .align_items(Align::Center)
                    .gap(s::SPACE_S2)
                    .focus_visible(ring(c::ACCENT))
                    .justify_content(if cx.rail() {
                        Justify::Center
                    } else {
                        Justify::Start
                    })
            })
            .child(content);
        let primary = if !independent {
            if let Some(disclosure) = disclosure.filter(|_| popup_panel.is_none()) {
                primary.with_node(move |node| node.part(disclosure.trigger()))
            } else {
                primary
            }
        } else {
            primary
        };
        let primary = primary.on_press(move || self.activate());
        let primary = if let Some(panel) = popup_panel {
            panel.trigger(primary)
        } else {
            primary
        };
        if !has_children && cx.collapsed().is_some() {
            tooltip(label, primary)
                .placement(Placement::End)
                .into_view()
        } else {
            primary.into_view()
        }
    }
    fn activate(self) {
        let cx = self.cx;
        let mode = self.mode;
        let mobile = self.mobile;
        let open = self.branch.open();
        let popup_panel = self.branch.flyout();
        let disabled = self.disabled;
        let action = self.action;

        if disabled.get() {
            return;
        }
        if let Some(panel) = popup_panel.filter(|_| cx.rail()) {
            panel.toggle();
        } else if let Some(action) = action {
            action.call(());
            if let Some(popup) = cx.popup() {
                popup.set(false);
            }
            if mode == ShellNavMode::Drawer {
                mobile.hide();
            }
        } else if popup_panel.is_some()
            && let Some(open) = open
        {
            open.set(!open.get());
        }
    }
    fn nested(self, children: Option<ViewFn>) -> impl IntoView {
        let cx = self.cx;
        let has_children = self.branch.has_children();
        let disclosure = self.branch.disclosure();
        let child_slot = children;
        show(
            move || {
                !cx.rail()
                    && has_children
                    && disclosure.is_none_or(|disclosure| disclosure.is_open().get())
            },
            move || {
                let child = child_slot.clone();
                let cx = SideNavCx {
                    depth: cx.depth + 1,
                    ..cx
                };
                provide(cx, move || {
                    let group = column(child.map(|slot| slot.call(())))
                        .style(|| style().gap(s::SPACE_S0_5));
                    if let Some(disclosure) = disclosure {
                        group
                            .with_node(move |node| node.role(Role::Group))
                            .with_node(move |node| node.part(disclosure.content()))
                            .style(
                                style()
                                    .flex_shrink(0)
                                    .enter_size(kivact::view::Orientation::Vertical)
                                    .exit_size(kivact::view::Orientation::Vertical)
                                    .transition(
                                        TransitionProp::Size,
                                        Transition::tween(MEDIUM as f64, EASE_STANDARD),
                                    ),
                            )
                    } else {
                        group.with_node(move |node| node.role(Role::Group))
                    }
                })
            },
        )
    }
    fn flyout(self, children: Option<ViewFn>) -> Option<impl IntoView> {
        let anchor = self.anchor;
        let popup_ref = self.popup_ref;
        let popup_panel = self.branch.flyout();
        let label = self.label;
        popup_panel.zip(children).map(|(nav_panel, children)| {
            let open = nav_panel.open();
            let behavior = use_popup(open).anchor(anchor);
            let title = message_with("soft.side_nav.item_submenu", "{label} submenu", move || {
                [("label", label.get())]
            });
            show(behavior.is_open(), move || {
                let children = children.clone();
                let cx = SideNavCx {
                    surface: NavSurface::Flyout { open },
                    ..SideNavCx::default()
                };
                let surface = nav_panel
                    .surface(column((
                        text(label).style(|| {
                            style()
                                .padding_y(s::SPACE_S1)
                                .padding_x(s::SPACE_S2)
                                .font_size(t::SUPPORTING.size_px)
                                .font_weight(t::WEIGHT_SEMIBOLD)
                                .color(c::TEXT_SECONDARY)
                        }),
                        provide(cx, move || children.call(())),
                    )))
                    .node_ref(popup_ref)
                    .style(|| {
                        let dark = use_theme().get().mode() == ThemeMode::Dark;
                        let base = style()
                            .background(c::BG_POPOVER)
                            .min_width(180)
                            .padding(s::SPACE_S1)
                            .border_radius(RADIUS_CONTAINER)
                            .gap(s::SPACE_S0_5);
                        shadowed(base, shadows::low(dark))
                    });
                overlay(anchor, Placement::End, surface)
                    .offset(4)
                    .with_node(move |node| node.part(behavior.content()))
                    .name(title)
            })
        })
    }
}
