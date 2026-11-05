//! Role: Render nested sidebar items and section rows.

use super::context::{SideNavCx, nav_cx};

use crate::app_shell::{ShellNavMode, shell_nav_mode};
use crate::mobile_nav::use_app_shell_mobile;
use crate::stack::rtl;

use kivact::i18n::message_with;
use kivact::interaction::Delays;
use kivact::interaction::DialogMode;
use kivact::interaction::use_dialog;
use kivact::interaction::use_disclosure;
use kivact::motion::Transition;
use kivact::state::Binding;
use kivact::state::Callback;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::callback;
use kivact::state::provide;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::button;
use kivact::ui::tooltip;
use kivact::view::IntoView;
use kivact::view::IntoViewFn;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::Placement;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::ViewFn;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::overlay;
use kivact::view::row;
use kivact::view::show;
use kivact::view::text;
use std::time::Duration;

#[derive(Clone)]
pub enum ItemCollapse {
    AlwaysOpen,
    Uncontrolled { default_collapsed: bool },
    Controlled(Binding<bool>),
}

pub struct SideNavItem {
    root: Node,
    label: Value<String>,
    icon: Option<View>,
    selected_icon: Option<View>,
    selected: Value<bool>,
    end: Option<View>,
    actions: Option<View>,
    children: Option<ViewFn>,
    collapse: Value<ItemCollapse>,
    size: Value<ControlSize>,
    action: Option<Callback<()>>,
}

pub fn side_nav_item(label: impl IntoText) -> SideNavItem {
    let label = label.into_text_value();
    SideNavItem {
        root: node(),
        label,
        icon: None,
        selected_icon: None,
        selected: false.into_value(),
        end: None,
        actions: None,
        children: None,
        collapse: (ItemCollapse::Uncontrolled {
            default_collapsed: false,
        })
        .into_value(),
        size: (ControlSize::Medium).into_value(),
        action: None,
    }
}

impl SideNavItem {
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    pub fn selected_icon(mut self, icon: impl IntoView) -> Self {
        self.selected_icon = Some(icon.into_view());
        self
    }
    pub fn selected(mut self, selected: impl IntoValue<bool>) -> Self {
        self.selected = selected.into_value();
        self
    }
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
    pub fn end_content(mut self, content: impl IntoView) -> Self {
        self.end = Some(kivact::view::component(move || content.into_view()));
        self
    }
    pub fn actions(mut self, actions: impl IntoView) -> Self {
        self.actions = Some(kivact::view::component(move || actions.into_view()));
        self
    }
    pub fn children<V: IntoView>(mut self, children: impl IntoViewFn<V>) -> Self {
        self.children = Some(children.into_view_fn());
        self
    }
    pub fn collapsible(mut self, collapse: impl IntoValue<ItemCollapse>) -> Self {
        self.collapse = collapse.into_value();
        self
    }
    pub fn on_press(mut self, action: impl Into<Callback<()>>) -> Self {
        let action: Callback<()> = action.into();
        self.action = Some(callback(move |_| action.call(())));
        self
    }
}

impl NodeBuilder for SideNavItem {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SideNavItem {
    fn into_view(mut self) -> View {
        let disabled = self
            .root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        let Self {
            root,
            label,
            icon: leading,
            selected_icon,
            selected,
            end,
            actions,
            children,
            collapse,
            size,
            action,
        } = self;
        kivact::view::component(move || {
            let cx = nav_cx();
            let mode = shell_nav_mode();
            let has_icon = leading.is_some() || selected_icon.is_some();
            let has_selected_icon = selected_icon.is_some();
            let child_slot = children.clone();
            let has_children = children.is_some();
            let mobile = use_app_shell_mobile();
            let collapse = match collapse.get() {
                ItemCollapse::AlwaysOpen => None,
                ItemCollapse::Uncontrolled { default_collapsed } => {
                    Some(signal(default_collapsed).into_binding())
                }
                ItemCollapse::Controlled(binding) => Some(binding),
            };
            let open = collapse.map(|collapse: Binding<bool>| {
                binding(move || !collapse.get(), move |open| collapse.set(!open))
            });
            let disclosure = open
                .filter(|_| has_children)
                .map(|open| use_disclosure(open).disabled(disabled));
            let independent = disclosure.is_some() && action.is_some();
            let anchor = node_ref();
            let popup_ref = node_ref();
            let popup_panel = if children.is_some() && cx.collapsed.is_some() {
                Some(crate::navigation_session::NavPanel::new(
                    move || cx.rail() && !disabled.get(),
                    Delays {
                        open: Duration::from_millis(150),
                        close: Duration::from_millis(200),
                    },
                ))
            } else {
                None
            };
            let popup = popup_panel.map(|panel| panel.open());
            let leading = leading.map(|icon| {
                icon_box(
                    icon,
                    move || has_selected_icon && selected.get(),
                    selected,
                    disabled,
                )
            });
            let selected_glyph = selected_icon
                .map(|icon| icon_box(icon, move || !selected.get(), selected, disabled));
            let label_view = text(label)
                .hidden(move || cx.rail())
                .style(style().grow(1).min_width(0).single_line());
            let chevron = disclosure.map(|disclosure| {
                let glyph = icon("chevron-down").style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    style()
                        .size(t.spacing.s6, t.spacing.s6)
                        .rotate(if disclosure.is_open().get() { 180 } else { 0 })
                        .transition(
                            TransitionProp::Paint,
                            Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
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
                    .icon(glyph)
                    .with_node(move |node| node.part(disclosure.trigger()))
                    .hidden(move || cx.rail())
                    .into_view()
                } else {
                    glyph.hidden(move || cx.rail()).into_view()
                }
            });
            let (inline_chevron, separate_chevron) = if independent {
                (None, chevron)
            } else {
                (chevron, None)
            };
            let end = end.map(|end| node().hidden(move || cx.rail()).child(end));
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
                    !disabled.get() && (action.is_some() || disclosure.is_some() || popup.is_some())
                })
                .disabled(disabled)
                .selected(selected)
                .with_node(move |node| {
                    node.current_item(move || {
                        selected.get().then_some(kivact::view::CurrentItem::Page)
                    })
                })
                .style(move || {
                    style()
                        .direction(logical_row())
                        .grow(1)
                        .min_width(0)
                        .align_self(Align::Stretch)
                        .align_items(Align::Center)
                        .gap(kivact::style::use_tokens::<crate::Tokens>().spacing.s2)
                        .focus_visible(crate::recipe_style::ring(
                            &kivact::style::use_tokens::<crate::Tokens>(),
                            kivact::style::use_tokens::<crate::Tokens>().colors.accent,
                        ))
                        .justify_content(if cx.rail() {
                            Justify::Center
                        } else {
                            Justify::Start
                        })
                })
                .child((leading, selected_glyph, label_view, end, inline_chevron));
            let primary = if !independent {
                if let Some(disclosure) = disclosure.filter(|_| popup.is_none()) {
                    primary.with_node(move |node| node.part(disclosure.trigger()))
                } else {
                    primary
                }
            } else {
                primary
            };
            let primary = primary.on_press(move || {
                if disabled.get() {
                    return;
                }
                if let Some(panel) = popup_panel.filter(|_| cx.rail()) {
                    panel.toggle();
                } else if let Some(action) = action {
                    action.call(());
                    if let Some(popup) = cx.popup {
                        popup.set(false);
                    }
                    if mode == ShellNavMode::Drawer {
                        mobile.close();
                    }
                } else if popup.is_some()
                    && let Some(open) = open
                {
                    open.set(!open.get());
                }
            });
            let primary = if let Some(panel) = popup_panel {
                panel.trigger(primary)
            } else {
                primary
            };
            let primary = if children.is_none() && cx.collapsed.is_some() {
                tooltip(label).placement(Placement::End).attach(primary)
            } else {
                primary.into_view()
            };
            let actions = actions.map(|actions| {
                row(actions)
                    .hidden(move || cx.rail())
                    .style(|| style().gap(kivact::style::use_tokens::<crate::Tokens>().spacing.s1))
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
            let nested = show(
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
                        let group = column(child.map(|slot| slot.call(()))).style(|| {
                            style().gap(kivact::style::use_tokens::<crate::Tokens>().spacing.s0_5)
                        });
                        if let Some(disclosure) = disclosure {
                            let motion = kivact::style::use_tokens::<crate::Tokens>().motion;
                            group
                                .with_node(move |node| node.role(Role::Group))
                                .with_node(move |node| node.part(disclosure.content()))
                                .style(
                                    style()
                                        .shrink(0)
                                        .enter_size(kivact::view::Orientation::Vertical)
                                        .exit_size(kivact::view::Orientation::Vertical)
                                        .transition(
                                            TransitionProp::Size,
                                            Transition::tween(
                                                motion.medium as f64,
                                                motion.ease_standard,
                                            ),
                                        ),
                                )
                        } else {
                            group.with_node(move |node| node.role(Role::Group))
                        }
                    })
                },
            );
            let flyout = popup_panel.zip(children).map(|(nav_panel, children)| {
                let open = nav_panel.open();
                let behavior = use_dialog(open, DialogMode::Popover).anchor(anchor);
                let title =
                    message_with("soft.side_nav.item_submenu", "{label} submenu", move || {
                        [("label", label.get())]
                    });
                show(behavior.is_open(), move || {
                    let children = children.clone();
                    let cx = SideNavCx {
                        popup: Some(open),
                        ..SideNavCx::default()
                    };
                    let surface = nav_panel
                        .surface(column((
                            text(label).style(|| {
                                let t = kivact::style::use_tokens::<crate::Tokens>();
                                style()
                                    .padding_y(t.spacing.s1)
                                    .padding_x(t.spacing.s2)
                                    .font_size(t.typography.supporting.size_px)
                                    .weight(t.typography.weight_semibold)
                                    .color(t.colors.text_secondary)
                            }),
                            provide(cx, move || children.call(())),
                        )))
                        .node_ref(popup_ref)
                        .style(|| {
                            let t = kivact::style::use_tokens::<crate::Tokens>();
                            let mut base = style()
                                .background(t.colors.bg_popover)
                                .min_width(180)
                                .padding(t.spacing.s1)
                                .radius(t.radius.container)
                                .gap(t.spacing.s0_5);
                            for shadow in &t.shadows.low {
                                base = base.shadow(*shadow);
                            }
                            base
                        });
                    overlay(anchor, Placement::End, surface)
                        .offset(4)
                        .with_node(move |node| node.part(behavior.content()))
                        .name(title)
                })
            });
            root.hidden(move || cx.rail() && !has_icon)
                .with_node(move |node| node.base_style(style().direction(Direction::Column)))
                .child((bar, nested, flyout))
        })
    }
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
            let t = kivact::style::use_tokens::<crate::Tokens>();
            style()
                .size(t.sizes.icon_sm, t.sizes.icon_sm)
                .shrink(0)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .color(if disabled.get() {
                    t.colors.icon_disabled
                } else if selected.get() {
                    t.colors.text_primary
                } else {
                    t.colors.icon_secondary
                })
        })
        .child(glyph)
}

pub(crate) fn logical_row() -> Direction {
    if rtl() {
        Direction::RowReverse
    } else {
        Direction::Row
    }
}

pub(crate) fn side_row_style(
    size: ControlSize,
    depth: usize,
    rail: bool,
    selected: bool,
    disabled: bool,
) -> kivact::style::Style {
    let t = kivact::style::use_tokens::<crate::Tokens>();
    let height = match size {
        ControlSize::Small => t.sizes.control_sm,
        ControlSize::Large => t.sizes.control_lg,
        _ => t.sizes.control_md,
    };
    let pad = if size == ControlSize::Small {
        t.spacing.s1
    } else {
        t.spacing.s2
    };
    let base = style()
        .direction(logical_row())
        .align_items(Align::Center)
        .gap(t.spacing.s2)
        .height(height)
        .radius(t.radius.element)
        .font_size(t.typography.label.size_px)
        .line_height(t.typography.label.line_height_px / t.typography.label.size_px)
        .weight(if selected {
            t.typography.weight_medium
        } else {
            t.typography.weight_normal
        })
        .color(if disabled {
            t.colors.text_disabled
        } else {
            t.colors.text_primary
        })
        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent));
    let base = if rail {
        base.width(height).justify_content(Justify::Center)
    } else {
        let start = pad + depth as f64 * t.spacing.s6;
        let (left, right) = if rtl() { (pad, start) } else { (start, pad) };
        base.width(pct(100)).padding_left(left).padding_right(right)
    };
    if selected {
        base.background(t.colors.neutral)
    } else if disabled {
        base
    } else {
        base.hover(style().background(t.colors.overlay_hover))
            .pressed(style().background(t.colors.overlay_pressed))
    }
}

pub struct SideNavSection {
    root: Node,
    title: Value<String>,
    subtitle: Option<Value<String>>,
    end: Option<View>,
    children: View,
    header_hidden: Value<bool>,
}

pub fn side_nav_section(title: impl IntoText, children: impl IntoView) -> SideNavSection {
    let title = title.into_text_value();
    SideNavSection {
        root: node()
            .with_node(move |node| node.role(Role::Group))
            .name(title),
        title,
        subtitle: None,
        end: None,
        children: children.into_view(),
        header_hidden: (false).into_value(),
    }
}

impl SideNavSection {
    pub fn subtitle(mut self, subtitle: impl IntoText) -> Self {
        self.subtitle = Some(subtitle.into_text_value());
        self
    }
    pub fn end_content(mut self, content: impl IntoView) -> Self {
        self.end = Some(kivact::view::component(move || content.into_view()));
        self
    }
    pub fn header_hidden(mut self, hidden: impl IntoValue<bool>) -> Self {
        self.header_hidden = hidden.into_value();
        self
    }
}

impl NodeBuilder for SideNavSection {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SideNavSection {
    fn into_view(self) -> View {
        let Self {
            root,
            title,
            subtitle,
            end,
            children,
            header_hidden,
        } = self;
        kivact::view::component(move || {
            let cx = nav_cx();
            let title = text(title).style(|| {
                style()
                    .weight(
                        kivact::style::use_tokens::<crate::Tokens>()
                            .typography
                            .weight_semibold,
                    )
                    .single_line()
            });
            let header = row((
                column((title, subtitle.map(text))).style(style().grow(1).min_width(0)),
                end,
            ))
            .hidden(move || header_hidden.get() || cx.rail())
            .style(|| {
                let t = kivact::style::use_tokens::<crate::Tokens>();
                style()
                    .direction(logical_row())
                    .padding_y(t.spacing.s1)
                    .padding_x(t.spacing.s2)
                    .gap(t.spacing.s2)
                    .align_items(Align::Center)
                    .font_size(t.typography.supporting.size_px)
                    .line_height(
                        t.typography.supporting.line_height_px / t.typography.supporting.size_px,
                    )
                    .color(t.colors.text_secondary)
            });
            root.with_node(move |node| {
                node.base_style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let base = style()
                        .direction(Direction::Column)
                        .padding_y(t.spacing.s1)
                        .gap(t.spacing.s0_5);
                    if cx.rail() {
                        base.align_items(Align::Center)
                    } else {
                        base
                    }
                })
            })
            .child((header, children))
        })
    }
}

kivact::state::constant_value!(ItemCollapse);
